//! Logical paint intent — what the operator dispatcher hands to the pipeline.
//!
//! A [`PaintIntent`] captures *what* the content stream wants to paint without
//! resolving *how* it will be painted. The resolution pipeline reads the
//! intent, the borrowed [`GraphicsState`], and the surrounding context, and
//! produces a [`super::ResolvedPaintCmd`] that the backend can consume
//! verbatim.
//!
//! Lifetimes are tight on purpose: the pipeline takes references into the
//! operator walker's owned state (graphics state, paths, fonts, clip stack)
//! and never clones them. This keeps the per-operator overhead to a handful
//! of stack copies on the hot path.

use std::sync::Arc;

use crate::content::graphics_state::{GraphicsState, Matrix};
use crate::fonts::FontInfo;
use crate::object::Object;
use smallvec::SmallVec;

/// The kind of paint operation being intended. Each variant corresponds to a
/// family of PDF operators that all need the same resolved-colour evaluation.
///
/// All variants hold only `&'a T` references and primitive `Copy` types
/// (`u16`, `f32`, `tiny_skia::FillRule`), so the enum derives `Copy`:
/// the pipeline composer can copy a `PaintKind` into the
/// [`super::ResolvedPaintCmd`] memberwise without an explicit clone
/// match.
#[derive(Clone, Copy)]
pub(crate) enum PaintKind<'a> {
    /// Colour resolution with no associated geometry. Used by callers
    /// that only need the resolver's colour output — e.g. the page
    /// renderer's fill / stroke / image-mask dispatcher, which paints
    /// geometry through its own (non-pipeline) rasteriser after
    /// splicing the resolved colour into the graphics state.
    ///
    /// Carries no fields because the colour stage reads only `color`,
    /// `gs`, and `side` from the [`PaintIntent`]; emitting a fake path
    /// to satisfy [`PaintKind::Path`] just to drive the colour stage
    /// pollutes both the type and the hot path with allocations the
    /// pipeline can't even observe. `ColorOnly` lets that caller
    /// express what it actually means.
    ColorOnly,
    /// Path fill / stroke (`f`, `F`, `S`, `B`, `b`, `f*`, `B*`, `b*`).
    /// `fill_rule` is meaningful only for fill sides; stroke sides ignore it.
    Path {
        path: &'a tiny_skia::Path,
        fill_rule: tiny_skia::FillRule,
    },
    /// **Provisional — not yet emitted by any operator dispatcher.**
    ///
    /// Reserved for a future per-glyph resolution stage. Today the
    /// text-showing operators (`Tj`, `TJ`, `'`, `"`) drive one
    /// resolve-per-`Tj` through [`PaintKind::ColorOnly`] (via the
    /// `pipeline_resolve_text_colors` helper on the operator-walker
    /// side) and hand the resolved RGBA to the shared text rasteriser;
    /// the colour stage does not read the glyph payload, so no
    /// per-glyph schema needs to be committed to here. This variant
    /// exists because subsequent waves are expected to consume it:
    ///
    /// * **Per-glyph clip composition** — text rendering modes 4-7
    ///   add the glyph outline to the clipping path. A per-glyph
    ///   intent is the natural home for that composition, since the
    ///   accumulated clip is glyph-shaped rather than path-shaped.
    /// * **Per-glyph antialias overrides** — `gs.smoothness` and ICC
    ///   text-rendering rules can flip antialiasing at glyph
    ///   granularity in some PDF profiles.
    /// * **Font-specific overprint simulation** — overprint of spot
    ///   inks against an embedded font's anti-aliased halo wants to
    ///   key off the glyph outline rather than a generic path.
    ///
    /// Until those waves arrive the variant is unused; the pipeline
    /// composer copies it through verbatim alongside every other
    /// variant.
    Glyph {
        glyph_id: u16,
        font: &'a Arc<FontInfo>,
        /// Horizontal advance in user units (post text-matrix).
        advance_user: f32,
    },
    /// **Provisional — not yet emitted by any operator dispatcher.**
    ///
    /// Reserved for a future per-image colour-plane resolution stage.
    /// Today the `Do` dispatcher for `Subtype /Image` either routes
    /// through [`PaintKind::ColorOnly`] (for `/ImageMask true`, where
    /// the fill colour comes from graphics state) or hands the pixel
    /// data straight to the image rasteriser without going through the
    /// pipeline (since standard image colour comes from sampled pixel
    /// data, not the current GS fill colour). This variant exists
    /// because subsequent waves are expected to consume it:
    ///
    /// * **Wave 5 separation backend** — per-channel routing of
    ///   colour-space-bearing image XObjects (CMYK / ICCBased N=4 /
    ///   Indexed / DeviceN images) to the right plate, which requires
    ///   the backend to see the image as a paint intent rather than a
    ///   rasteriser-only call.
    /// * **Per-pixel colour resolution** for images with Separation or
    ///   DeviceN colour spaces, where the tint transform must run on
    ///   every sample rather than only on the fill register.
    ///
    /// This is distinct from [`PaintKind::Path`] because image colour
    /// comes from sampled pixel data, not from the current GS fill
    /// colour. Until those waves arrive the variant is unused; the
    /// pipeline composer copies it through verbatim alongside every
    /// other variant.
    Image { xobj_name: &'a str },
    /// **Provisional — not yet emitted by any operator dispatcher.**
    ///
    /// Reserved for a future shading-aware resolution stage. Today the
    /// `sh` dispatcher drives endpoint-colour resolution out-of-band
    /// (via the `pipeline_resolve_shading_endpoints` helper on the
    /// page-renderer side) and hands the two resolved RGBAs to the
    /// gradient backend; the colour stage does not see the shading
    /// geometry, so no shading-shape schema needs to be committed to
    /// here. This variant exists because subsequent waves are
    /// expected to consume it:
    ///
    /// * **Wave 5 separation backend** — per-plate routing of gradient
    ///   endpoints. A Type 4 Separation gradient resolves to one
    ///   plate; the backend may need "this is a gradient on plate X
    ///   spanning tint A to tint B" rather than two unrelated tint
    ///   values, which requires the backend to see the gradient as a
    ///   single paint intent rather than two endpoint-colour resolves.
    /// * **Future overprint resolver** — ISO 32000-1 §11.7.4 has
    ///   gradient-specific overprint semantics that key off the
    ///   gradient axis / radii, not the per-stop colour.
    /// * **Preflight / measurement backends** — integrating ink
    ///   coverage across a gradient axis needs the gradient geometry,
    ///   not just its endpoints.
    ///
    /// Until those waves arrive the variant is unused; the pipeline
    /// composer copies it through verbatim alongside every other
    /// variant.
    Shading { shading_name: &'a str },
}

/// Whether the intent applies to the fill side or the stroke side of the
/// current paint operation. A `B`/`b` (fill-then-stroke) operator emits two
/// separate intents — one `Fill`, then one `Stroke` — so a single
/// `ResolvedPaintCmd` is always one-sided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaintSide {
    Fill,
    Stroke,
}

/// Logical colour as it appeared in the content stream, before tint transform
/// evaluation, ICC conversion, or ink routing. The resolution pipeline
/// consumes this and produces a [`super::ResolvedColor`].
///
/// The `'a` lifetime ties the colour-space reference to the resource map
/// owned by the operator walker, so we don't clone resolved space arrays per
/// intent — colour-space arrays can be many KiB for ICCBased entries that
/// embed the profile by reference.
pub(crate) enum LogicalColor<'a> {
    /// Device-family colour already evaluated by the operator (g, rg, k, K,
    /// SC, SCN). For `g`/`rg`/`k`/`K` the operator emits this variant
    /// directly; for `SCN` with a Device* colour space the dispatcher also
    /// picks this variant.
    Device(DeviceColor),

    /// `SCN`/`scn` against a non-device colour space. The pipeline reads
    /// `space` (the resolved colour-space array or name from the resources
    /// dict) and evaluates the components against it.
    Spaced {
        /// Resolved colour-space object — either an `Object::Name` for a
        /// device alias or an `Object::Array` for compound spaces
        /// (`Separation`, `DeviceN`, `ICCBased`, `Indexed`, `Lab`, `CalRGB`,
        /// `CalGray`, `Pattern`).
        space: &'a Object,
        /// Components from the operator. Stack-allocated for the
        /// overwhelmingly common case (≤8 inks); the spec doesn't impose
        /// an upper bound on `DeviceN` colorants but real-world packaging
        /// files top out around 8-10.
        components: SmallVec<[f32; 8]>,
    },
}

/// Already-evaluated device-family colour. The operator dispatcher emits
/// these directly for the device-family operators; the pipeline passes them
/// through verbatim into [`super::ResolvedColor::Rgba`] or
/// [`super::ResolvedColor::Cmyk`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DeviceColor {
    Gray(f32),
    Rgb(f32, f32, f32),
    Cmyk(f32, f32, f32, f32),
}

/// A single intent produced by the operator dispatcher for the pipeline to
/// resolve. The struct is allocation-free: every field is either a primitive
/// or a borrow. The lifetime is the operator-walker's snapshot lifetime.
pub(crate) struct PaintIntent<'a> {
    pub(crate) kind: PaintKind<'a>,
    pub(crate) side: PaintSide,
    pub(crate) gs: &'a GraphicsState,
    pub(crate) color: LogicalColor<'a>,
    /// Current CTM at the moment the operator fired. The pipeline does *not*
    /// compose this with the page's base transform — that's the backend's
    /// concern, since it depends on the device-space coordinate system.
    pub(crate) ctm: Matrix,
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn device_gray_round_trip() {
        let c = DeviceColor::Gray(0.5);
        assert_eq!(c, DeviceColor::Gray(0.5));
        assert_ne!(c, DeviceColor::Gray(0.6));
    }

    #[test]
    fn device_rgb_inequality_per_channel() {
        let a = DeviceColor::Rgb(1.0, 0.0, 0.0);
        let b = DeviceColor::Rgb(1.0, 0.0, 0.0);
        let c = DeviceColor::Rgb(1.0, 0.5, 0.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn device_cmyk_construction() {
        let c = DeviceColor::Cmyk(0.1, 0.2, 0.3, 0.4);
        let DeviceColor::Cmyk(c0, m0, y0, k0) = c else {
            panic!("expected Cmyk variant");
        };
        assert!((c0 - 0.1).abs() < 1e-6);
        assert!((m0 - 0.2).abs() < 1e-6);
        assert!((y0 - 0.3).abs() < 1e-6);
        assert!((k0 - 0.4).abs() < 1e-6);
    }

    #[test]
    fn logical_color_spaced_holds_components_inline() {
        // SmallVec<[f32; 8]> must keep ≤8 components inline (no heap
        // allocation). DeviceN colorant counts in real PDFs top out around
        // 8; this lets the hot path stay alloc-free.
        let space = Object::Name("DeviceCMYK".to_string());
        let comps: SmallVec<[f32; 8]> = smallvec![0.1, 0.2, 0.3, 0.4];
        let lc = LogicalColor::Spaced {
            space: &space,
            components: comps,
        };
        match lc {
            LogicalColor::Spaced { components, .. } => {
                assert_eq!(components.len(), 4);
                assert!(!components.spilled(), "≤8 components must stay inline");
            },
            _ => panic!("expected Spaced variant"),
        }
    }

    #[test]
    fn paint_side_is_two_valued() {
        // Sanity: PaintSide must be a strict two-state enum. A `B`-style op
        // emits two intents (one Fill, one Stroke); the pipeline never has
        // to handle a "both" variant.
        assert_ne!(PaintSide::Fill, PaintSide::Stroke);
    }
}
