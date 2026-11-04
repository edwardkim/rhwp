//! Allocation-free geometry and bitmap budgeting for exact region replay.

use crate::error::HwpError;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterRenderOptions};
use crate::renderer::raster_pixels::RasterRegion;

use super::SkiaLayerRenderer;

enum WorkingRaster {
    FullPage,
    Tiles,
}

/// A validated region on the scaled full-page pixel grid.
///
/// Constructing a plan does not create a renderer, discover fonts, allocate a
/// bitmap, or replay the page. The same plan is used by Skia region rendering.
/// It bounds N32 working pixels plus RGBA output, not Skia intermediates,
/// existing resource caches, caller-owned buffers, or total process memory.
///
/// Fields cannot be changed after validation. A wholly off-page window has no
/// working raster, while its output and coordinate bounds still apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkiaRegionPlan {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
    page_width: i32,
    page_height: i32,
    stride: usize,
    output_bytes: usize,
    working_bytes: u64,
    peak_bitmap_bytes: u64,
    intersection: Option<(i32, i32, i32, i32)>,
}

impl SkiaRegionPlan {
    /// Independent full-page working-raster dimension limit.
    pub const MAX_WORKING_DIMENSION: i32 = 16_384;
    /// Independent full-page working-raster pixel limit.
    pub const MAX_WORKING_PIXELS: u64 = 16 * 1024 * 1024;
    /// Maximum simultaneous working N32 and requested RGBA bitmap bytes.
    pub const MAX_BITMAP_BYTES: u64 = 128 * 1024 * 1024;
    /// Inclusive absolute endpoint limit on the integer device-pixel grid.
    pub const MAX_EXACT_PIXEL: i32 = 16_777_216;

    /// Validate logical page/region geometry and compute the enclosing window.
    ///
    /// `options.max_dimension` and `max_pixels` constrain the output only. The
    /// full-page working raster has its own limits. Output limits are checked
    /// before working limits so a small viewport cannot hide an invalid output.
    pub fn new(
        page_width: f64,
        page_height: f64,
        region: RasterRegion,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<Self> {
        Self::plan(
            page_width,
            page_height,
            region,
            options,
            WorkingRaster::FullPage,
        )
    }

    pub(super) fn for_tiles(
        page_width: f64,
        page_height: f64,
        region: RasterRegion,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<Self> {
        Self::plan(
            page_width,
            page_height,
            region,
            options,
            WorkingRaster::Tiles,
        )
    }

    fn plan(
        page_width: f64,
        page_height: f64,
        region: RasterRegion,
        options: RasterRenderOptions,
        working: WorkingRaster,
    ) -> LayerRenderResult<Self> {
        SkiaLayerRenderer::validate_raster_options(options)?;
        let invalid = || HwpError::RenderError("invalid raster region geometry".into());
        let page_extent = |value: f64| -> LayerRenderResult<i32> {
            let pixels = (value * options.scale).ceil();
            if !value.is_finite()
                || value <= 0.0
                || !pixels.is_finite()
                || pixels <= 0.0
                || pixels > f64::from(Self::MAX_EXACT_PIXEL)
            {
                return Err(invalid());
            }
            Ok(pixels as i32)
        };
        let page_width = page_extent(page_width)?;
        let page_height = page_extent(page_height)?;
        let right = region.x + region.width;
        let bottom = region.y + region.height;
        if [
            region.x,
            region.y,
            region.width,
            region.height,
            right,
            bottom,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
            || region.width <= 0.0
            || region.height <= 0.0
            || right <= region.x
            || bottom <= region.y
        {
            return Err(invalid());
        }
        let endpoints = [
            (region.x * options.scale).floor(),
            (region.y * options.scale).floor(),
            (right * options.scale).ceil(),
            (bottom * options.scale).ceil(),
        ];
        let exact = f64::from(Self::MAX_EXACT_PIXEL);
        if endpoints
            .iter()
            .any(|value| !value.is_finite() || !(-exact..=exact).contains(value))
        {
            return Err(invalid());
        }
        let [origin_x, origin_y, right, bottom] = endpoints.map(|value| value as i32);
        let width = right.checked_sub(origin_x).ok_or_else(invalid)?;
        let height = bottom.checked_sub(origin_y).ok_or_else(invalid)?;
        if width <= 0
            || height <= 0
            || width > options.max_dimension
            || height > options.max_dimension
        {
            return Err(HwpError::RenderError(
                "raster window dimensions out of range".into(),
            ));
        }
        let pixels = (width as u64)
            .checked_mul(height as u64)
            .ok_or_else(|| HwpError::RenderError("raster pixel count overflow".into()))?;
        if pixels > options.max_pixels {
            return Err(HwpError::RenderError(format!(
                "raster pixel count out of range: {pixels}"
            )));
        }
        let byte_limit = || HwpError::ResourceLimit("raster byte count overflow".into());
        let stride = (width as usize).checked_mul(4).ok_or_else(byte_limit)?;
        let output_bytes = stride.checked_mul(height as usize).ok_or_else(byte_limit)?;
        // Wide endpoints preserve the full-page clipping of negative windows.
        let left = i64::from(origin_x).max(0);
        let top = i64::from(origin_y).max(0);
        let right = i64::from(right).min(i64::from(page_width));
        let bottom = i64::from(bottom).min(i64::from(page_height));
        let intersection = (left < right && top < bottom).then_some((
            left as i32,
            top as i32,
            right as i32,
            bottom as i32,
        ));
        let limit = || HwpError::ResourceLimit("raster region working budget exceeded".into());
        let working_bytes = if intersection.is_some() {
            let (working_width, working_height) = match working {
                WorkingRaster::FullPage => (page_width, page_height),
                WorkingRaster::Tiles => {
                    let dimension = super::tiled_region::SkiaTilePlan::MAX_WORKING_DIMENSION;
                    (page_width.min(dimension), page_height.min(dimension))
                }
            };
            let working_pixels = (working_width as u64)
                .checked_mul(working_height as u64)
                .ok_or_else(limit)?;
            if working_width > Self::MAX_WORKING_DIMENSION
                || working_height > Self::MAX_WORKING_DIMENSION
                || working_pixels > Self::MAX_WORKING_PIXELS
            {
                return Err(limit());
            }
            working_pixels.checked_mul(4).ok_or_else(limit)?
        } else {
            0
        };
        let peak_bitmap_bytes = working_bytes
            .checked_add(u64::try_from(output_bytes).map_err(|_| limit())?)
            .ok_or_else(limit)?;
        if peak_bitmap_bytes > Self::MAX_BITMAP_BYTES {
            return Err(limit());
        }
        Ok(Self {
            origin_x,
            origin_y,
            width,
            height,
            page_width,
            page_height,
            stride,
            output_bytes,
            working_bytes,
            peak_bitmap_bytes,
            intersection,
        })
    }

    pub fn origin_x(self) -> i32 {
        self.origin_x
    }

    pub fn origin_y(self) -> i32 {
        self.origin_y
    }

    pub fn width(self) -> i32 {
        self.width
    }

    pub fn height(self) -> i32 {
        self.height
    }

    pub fn page_width(self) -> i32 {
        self.page_width
    }

    pub fn page_height(self) -> i32 {
        self.page_height
    }

    pub fn stride(self) -> usize {
        self.stride
    }

    pub fn output_bytes(self) -> usize {
        self.output_bytes
    }

    pub fn working_bytes(self) -> u64 {
        self.working_bytes
    }

    pub fn peak_bitmap_bytes(self) -> u64 {
        self.peak_bitmap_bytes
    }

    /// Page-pixel `(left, top, right, bottom)`, with exclusive right/bottom.
    pub fn intersection(self) -> Option<(i32, i32, i32, i32)> {
        self.intersection
    }
}
