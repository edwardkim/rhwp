//! Immutable full-page raster snapshots for exact repeated region reads.
//!
//! The surface is released after its snapshot is taken, before any consumer can
//! mutate it. Retention therefore shares the N32 backing allocation without a
//! second full-page RGBA conversion or copy-on-write surface mutation.

use skia_safe::{surfaces, Color, Image, ImageInfo};

use super::region_plan::SkiaRegionPlan;
use super::renderer::colorref_to_skia;
use super::SkiaLayerRenderer;
use crate::error::HwpError;
use crate::paint::PageLayerTree;
use crate::renderer::layer_renderer::{LayerRenderResult, RasterRenderOptions};
use crate::renderer::raster_pixels::{
    RasterPixelFormat, RasterPixels, RasterRegion, RasterRegionPixels,
};

/// A completed CPU page raster, immutable for its lifetime.
///
/// Every read keeps the original page surface origin, scale and clear policy.
/// The snapshot owns its pixels independently of the renderer and source tree.
/// It does not establish a cross-thread or GPU contract. The retained N32 image
/// plus each requested RGBA output must fit the 128 MiB bitmap peak limit,
/// including reads wholly outside the page.
pub struct PreparedSkiaPage {
    image: Image,
    page_width: f64,
    page_height: f64,
    options: RasterRenderOptions,
    retained_bytes: usize,
}

impl PreparedSkiaPage {
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(crate) fn plan_region(&self, region: RasterRegion) -> LayerRenderResult<SkiaRegionPlan> {
        SkiaRegionPlan::new(self.page_width, self.page_height, region, self.options)
    }

    pub fn render_region(&self, region: RasterRegion) -> LayerRenderResult<RasterRegionPixels> {
        let plan = self.plan_region(region)?;
        if !self.can_render_plan(plan) {
            return Err(HwpError::ResourceLimit(
                "prepared raster plus region output exceeds bitmap budget".into(),
            ));
        }
        read_region(Some(&self.image), plan, self.options)
    }

    pub(crate) fn can_render_plan(&self, plan: SkiaRegionPlan) -> bool {
        Self::fits_bitmap_budget(self.retained_bytes, plan.output_bytes())
    }

    pub(crate) fn fits_bitmap_budget(retained_bytes: usize, output_bytes: usize) -> bool {
        let peak = u64::try_from(retained_bytes).ok().and_then(|retained| {
            u64::try_from(output_bytes)
                .ok()
                .and_then(|output| retained.checked_add(output))
        });
        peak.is_some_and(|bytes| bytes <= SkiaRegionPlan::MAX_BITMAP_BYTES)
    }
}

impl SkiaLayerRenderer {
    /// Rasterize once and return the first region with its reusable snapshot.
    ///
    /// All geometry/output/working limits are checked before allocation. A
    /// wholly off-page request returns its clear output with no snapshot and
    /// performs no page replay. It preserves the direct region API's allowance
    /// for off-page requests against pages larger than the working-raster cap.
    pub fn prepare_page_for_region(
        &self,
        tree: &PageLayerTree,
        region: RasterRegion,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<(RasterRegionPixels, Option<PreparedSkiaPage>)> {
        let plan = SkiaRegionPlan::new(tree.page_width, tree.page_height, region, options)?;
        let prepared = if plan.intersection().is_some() {
            let mut surface = surfaces::raster_n32_premul((plan.page_width(), plan.page_height()))
                .ok_or_else(|| HwpError::RenderError("Skia raster surface 생성 실패".into()))?;
            let canvas = surface.canvas();
            canvas.clear(clear_color(options));
            if options.scale != 1.0 {
                canvas.scale((options.scale as f32, options.scale as f32));
            }
            self.render_prepared_raster(canvas, tree, options.scale as f32)?;
            let image = surface.image_snapshot();
            let retained_bytes = image.image_info().compute_min_byte_size();
            // No surface escapes this block. Subsequent reads cannot trigger a
            // copy-on-write allocation through a retained mutable surface.
            Some(PreparedSkiaPage {
                image,
                page_width: tree.page_width,
                page_height: tree.page_height,
                options,
                retained_bytes,
            })
        } else {
            None
        };
        if prepared
            .as_ref()
            .is_some_and(|page| !page.can_render_plan(plan))
        {
            return Err(HwpError::ResourceLimit(
                "prepared raster plus region output exceeds bitmap budget".into(),
            ));
        }
        let output = read_region(prepared.as_ref().map(|page| &page.image), plan, options)?;
        Ok((output, prepared))
    }
}

fn clear_color(options: RasterRenderOptions) -> Color {
    if let Some(color) = options.background_color {
        colorref_to_skia(color, 1.0)
    } else if options.transparent {
        Color::from_argb(0, 0, 0, 0)
    } else {
        Color::WHITE
    }
}

fn read_region(
    image: Option<&Image>,
    plan: SkiaRegionPlan,
    options: RasterRenderOptions,
) -> LayerRenderResult<RasterRegionPixels> {
    let clear = clear_color(options);
    let mut bytes = vec![0; plan.output_bytes()];
    if clear.a() != 0 {
        let rgba = [clear.r(), clear.g(), clear.b(), clear.a()];
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.copy_from_slice(&rgba);
        }
    }
    if let Some((left, top, right, bottom)) = plan.intersection() {
        let image = image.ok_or_else(|| {
            HwpError::RenderError("missing prepared raster for intersecting region".into())
        })?;
        let info = ImageInfo::new(
            (right - left, bottom - top),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let destination_x = (i64::from(left) - i64::from(plan.origin_x())) as usize;
        let destination_y = (i64::from(top) - i64::from(plan.origin_y())) as usize;
        let destination = destination_y
            .checked_mul(plan.stride())
            .and_then(|row| {
                destination_x
                    .checked_mul(4)
                    .and_then(|column| row.checked_add(column))
            })
            .ok_or_else(|| HwpError::ResourceLimit("raster copy offset overflow".into()))?;
        if !image.read_pixels(
            &info,
            &mut bytes[destination..],
            plan.stride(),
            (left, top),
            skia_safe::image::CachingHint::Disallow,
        ) {
            return Err(HwpError::RenderError("Skia RGBA pixel read failed".into()));
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
