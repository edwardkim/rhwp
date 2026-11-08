//! Borrowed host targets, including GPU-backed Skia surfaces.

use skia_safe::{AutoCanvasRestore, Canvas, Matrix, Rect, Surface};

use super::renderer::colorref_to_skia;
use super::SkiaLayerRenderer;
use crate::error::HwpError;
use crate::model::ColorRef;
use crate::paint::{validate_text_variant_scope, PageLayerTree};
use crate::renderer::layer_renderer::LayerRenderResult;

/// Placement of a page on a host-owned Skia canvas.
///
/// Placement composes with the canvas's existing affine transform and clip.
/// `origin_x`/`origin_y` are in its current local units, before `scale` maps
/// page CSS pixels to those units. The page rectangle is always clipped even
/// when the tree disables its internal body/cell clips.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkiaSurfaceRenderOptions {
    pub scale: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    /// Optional opaque COLORREF page background. `None` preserves the target.
    pub background_color: Option<ColorRef>,
}

impl Default for SkiaSurfaceRenderOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            origin_x: 0.0,
            origin_y: 0.0,
            background_color: None,
        }
    }
}

impl SkiaLayerRenderer {
    /// Replay directly into a host-owned surface, with strict resource errors.
    ///
    /// This delegates to [`Self::render_to_canvas_strict`]. It neither allocates
    /// a destination surface nor reads back its pixels, flushes a GPU context,
    /// submits work, or presents a frame. The host owns those operations and
    /// must keep the surface's context alive and usable on the calling thread.
    pub fn render_to_surface_strict(
        &self,
        surface: &mut Surface,
        tree: &PageLayerTree,
        options: SkiaSurfaceRenderOptions,
    ) -> LayerRenderResult<()> {
        self.render_to_canvas_strict(surface.canvas(), tree, options)
    }

    /// Replay a page into a borrowed canvas without a final CPU RGBA readback.
    ///
    /// The existing affine transform and clip are respected. Matrix, clip, and
    /// save count are restored on success and error. Geometry, transform, and
    /// text-contract errors are rejected before any target pixels are changed.
    /// Perspective, singular, non-finite, and unrepresentable transforms fail.
    /// The largest device scale controls resolution-dependent SVG fallbacks.
    ///
    /// This uses the same ordered paint replay as CPU raster/PDF, with the PDF
    /// path's strict resource policy: missing/undecodable images and failed SVG
    /// raster fallbacks return an error instead of the CPU raster placeholder.
    /// Replay is **not atomic**: a late resource error can leave earlier paint
    /// and the requested background in the target. Hosts requiring atomic
    /// frames should render into their own offscreen surface before publishing.
    ///
    /// GPU-backed surfaces can consume this directly when the host enables a
    /// compatible skia-safe backend. Some individual resources still use CPU
    /// raster fallbacks; this is not a promise that every paint operation runs
    /// on a GPU. Context creation, synchronization, and presentation stay with
    /// the host. No backend feature is enabled implicitly.
    pub fn render_to_canvas_strict(
        &self,
        canvas: &Canvas,
        tree: &PageLayerTree,
        options: SkiaSurfaceRenderOptions,
    ) -> LayerRenderResult<()> {
        let invalid = || HwpError::RenderError("invalid Skia host surface geometry".into());
        let scale = options.scale as f32;
        let origin_x = options.origin_x as f32;
        let origin_y = options.origin_y as f32;
        let width = tree.page_width as f32;
        let height = tree.page_height as f32;
        if [
            options.scale,
            options.origin_x,
            options.origin_y,
            tree.page_width,
            tree.page_height,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
            || [scale, origin_x, origin_y, width, height]
                .into_iter()
                .any(|value| !value.is_finite())
            || scale <= 0.0
            || width <= 0.0
            || height <= 0.0
        {
            return Err(invalid());
        }
        let page = Rect::from_wh(width, height);
        let placement = Matrix::scale_translate((scale, scale), (origin_x, origin_y));
        let device = Matrix::concat(&canvas.local_to_device_as_3x3(), &placement);
        let fallback_raster_scale = device.max_scale();
        let (device_page, _) = device.map_rect(page);
        if !device.is_finite()
            || device.has_perspective()
            || !device.min_scale().is_finite()
            || device.min_scale() <= 0.0
            || !fallback_raster_scale.is_finite()
            || fallback_raster_scale <= 0.0
            || !device_page.is_finite()
            || device_page.is_empty()
        {
            return Err(invalid());
        }
        // Validate before an optional clear. The shared replay validates again
        // for its internal callers, which do not all use this host boundary.
        validate_text_variant_scope(tree).map_err(|error| {
            HwpError::RenderError(format!("invalid PageLayerTree text contract: {error}"))
        })?;

        let _restore = AutoCanvasRestore::guard(canvas, true);
        canvas.concat(&placement);
        canvas.clip_rect(page, None, false);
        if let Some(color) = options.background_color {
            // clear ignores the CTM but respects both the host and page clips.
            canvas.clear(colorref_to_skia(color, 1.0));
        }
        self.render_page_to_canvas_strict(canvas, tree, fallback_raster_scale)
    }
}
