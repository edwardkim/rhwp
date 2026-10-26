//! Opt-in region replay on a fixed page-pixel tile grid.

use skia_safe::{surfaces, ImageInfo};

use super::prepared_page::clear_color;
use super::{SkiaLayerRenderer, SkiaRegionPlan};
use crate::error::HwpError;
use crate::paint::PageLayerTree;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterRenderOptions};
use crate::renderer::raster_pixels::{
    RasterPixelFormat, RasterPixels, RasterRegion, RasterRegionPixels,
};

/// Allocation-free preflight for a bounded tiled region.
///
/// Every page pixel belongs to a fixed 512-pixel tile, independent of the
/// requested window. Only one tile surface exists at a time. Its two-pixel
/// guard is clipped to the scaled page, never to the requested window.
///
/// This is a separate raster contract from the full-page exact-crop API:
/// translating Skia's device can change antialiasing and floating-point
/// rounding. The guard is not a bound on arbitrary filter footprints. It does
/// not establish subtree culling, a persistent tile cache, or GPU ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkiaTilePlan {
    region: SkiaRegionPlan,
}

impl SkiaTilePlan {
    pub const TILE_SIZE: i32 = 512;
    pub const GUARD_PIXELS: i32 = 2;
    pub const MAX_WORKING_DIMENSION: i32 = Self::TILE_SIZE + 2 * Self::GUARD_PIXELS;

    pub fn new(
        page_width: f64,
        page_height: f64,
        region: RasterRegion,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<Self> {
        Ok(Self {
            region: SkiaRegionPlan::for_tiles(page_width, page_height, region, options)?,
        })
    }

    pub fn origin_x(self) -> i32 {
        self.region.origin_x()
    }
    pub fn origin_y(self) -> i32 {
        self.region.origin_y()
    }
    pub fn width(self) -> i32 {
        self.region.width()
    }
    pub fn height(self) -> i32 {
        self.region.height()
    }
    pub fn stride(self) -> usize {
        self.region.stride()
    }
    pub fn output_bytes(self) -> usize {
        self.region.output_bytes()
    }

    /// Conservative upper bound for one expanded N32 tile, zero off-page.
    pub fn working_bytes(self) -> u64 {
        self.region.working_bytes()
    }
    pub fn peak_bitmap_bytes(self) -> u64 {
        self.region.peak_bitmap_bytes()
    }

    /// Number of complete tile replays required by this request.
    pub fn tile_count(self) -> u64 {
        self.region
            .intersection()
            .map_or(0, |(left, top, right, bottom)| {
                let columns = (right - 1) / Self::TILE_SIZE - left / Self::TILE_SIZE + 1;
                let rows = (bottom - 1) / Self::TILE_SIZE - top / Self::TILE_SIZE + 1;
                columns as u64 * rows as u64
            })
    }
}

impl SkiaLayerRenderer {
    /// Replay only the fixed tiles intersecting a region, one surface at a time.
    ///
    /// Overlapping requests use the same tile origin, transform and clip for
    /// each page pixel. Results need not equal a crop of a full-page raster;
    /// use `render_region_pixels_with_options` when that contract is required.
    /// Each tile replays the complete ordered tree. The bitmap bound includes
    /// final RGBA output and one N32 tile, not retained renderer resources,
    /// caller-owned outputs, Skia intermediates or whole-process memory.
    pub fn render_tiled_region_pixels_with_options(
        &self,
        tree: &PageLayerTree,
        region: RasterRegion,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRegionPixels> {
        let tile_plan = SkiaTilePlan::new(tree.page_width, tree.page_height, region, options)?;
        let plan = tile_plan.region;
        let clear = clear_color(options);
        let mut bytes = vec![0; plan.output_bytes()];
        if clear.a() != 0 {
            for pixel in bytes.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[clear.r(), clear.g(), clear.b(), clear.a()]);
            }
        }
        if let Some((left, top, right, bottom)) = plan.intersection() {
            for tile_y in top / SkiaTilePlan::TILE_SIZE..=(bottom - 1) / SkiaTilePlan::TILE_SIZE {
                for tile_x in left / SkiaTilePlan::TILE_SIZE..=(right - 1) / SkiaTilePlan::TILE_SIZE
                {
                    let x = tile_x * SkiaTilePlan::TILE_SIZE;
                    let y = tile_y * SkiaTilePlan::TILE_SIZE;
                    let tile_right = (x + SkiaTilePlan::TILE_SIZE).min(plan.page_width());
                    let tile_bottom = (y + SkiaTilePlan::TILE_SIZE).min(plan.page_height());
                    let expanded_left = (x - SkiaTilePlan::GUARD_PIXELS).max(0);
                    let expanded_top = (y - SkiaTilePlan::GUARD_PIXELS).max(0);
                    let expanded_right =
                        (tile_right + SkiaTilePlan::GUARD_PIXELS).min(plan.page_width());
                    let expanded_bottom =
                        (tile_bottom + SkiaTilePlan::GUARD_PIXELS).min(plan.page_height());
                    // These bounds depend only on page size and tile index.
                    let mut surface = surfaces::raster_n32_premul((
                        expanded_right - expanded_left,
                        expanded_bottom - expanded_top,
                    ))
                    .ok_or_else(|| {
                        HwpError::RenderError("Skia tile surface allocation failed".into())
                    })?;
                    let canvas = surface.canvas();
                    canvas.clear(clear);
                    canvas.translate((-(expanded_left as f32), -(expanded_top as f32)));
                    if options.scale != 1.0 {
                        canvas.scale((options.scale as f32, options.scale as f32));
                    }
                    self.render_prepared_raster(canvas, tree, options.scale as f32)?;

                    let copy_left = left.max(x);
                    let copy_top = top.max(y);
                    let copy_right = right.min(tile_right);
                    let copy_bottom = bottom.min(tile_bottom);
                    let info = ImageInfo::new(
                        (copy_right - copy_left, copy_bottom - copy_top),
                        skia_safe::ColorType::RGBA8888,
                        skia_safe::AlphaType::Premul,
                        None,
                    );
                    let destination_x =
                        (i64::from(copy_left) - i64::from(plan.origin_x())) as usize;
                    let destination_y = (i64::from(copy_top) - i64::from(plan.origin_y())) as usize;
                    let destination = destination_y
                        .checked_mul(plan.stride())
                        .and_then(|row| {
                            destination_x
                                .checked_mul(4)
                                .and_then(|col| row.checked_add(col))
                        })
                        .ok_or_else(|| {
                            HwpError::ResourceLimit("tile copy offset overflow".into())
                        })?;
                    if !surface.image_snapshot().read_pixels(
                        &info,
                        &mut bytes[destination..],
                        plan.stride(),
                        (copy_left - expanded_left, copy_top - expanded_top),
                        skia_safe::image::CachingHint::Disallow,
                    ) {
                        return Err(HwpError::RenderError("Skia tile RGBA read failed".into()));
                    }
                }
            }
        }
        Ok(RasterRegionPixels {
            pixels: RasterPixels {
                bytes,
                width: plan.width(),
                height: plan.height(),
                stride: plan.stride(),
                format: RasterPixelFormat::Rgba8Premultiplied,
                color_space: options.color_space,
                dpi: options.dpi,
            },
            origin_x: plan.origin_x(),
            origin_y: plan.origin_y(),
        })
    }
}
