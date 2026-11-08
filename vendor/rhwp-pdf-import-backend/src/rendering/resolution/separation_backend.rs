//! Per-plate separation backend on top of the resolution pipeline.
//!
//! Implements [`super::PaintBackend`] for the prepress separation case: every
//! [`super::ResolvedPaintCmd`] is rasterised once per target ink, with the
//! per-plate decision (paint this plate with this tint, or skip it) delegated
//! to [`super::InkRouter`].
//!
//! # How this fits in
//!
//! The existing free-function entry point in
//! [`super::super::separation_renderer`] still drives the page-walk for the
//! shipping separation API; it carries its own per-operator dispatch and
//! reproduces the per-plate routing decision inline (`tint_for_ink`). This
//! backend is the pipeline-driven equivalent: given a fully-resolved
//! [`super::ResolvedPaintCmd`], it produces the same plate output without any
//! of the per-operator branching. Once the operator walker upstream emits
//! `ResolvedPaintCmd`s through the pipeline, the existing renderer's
//! per-operator arms become redundant and can call into this backend
//! instead — a follow-up branch tracked separately from wave 5.
//!
//! # Contracts honoured
//!
//! - **Per-plate routing**: [`super::InkRouter`] is the single source of truth
//!   for "does this command touch this plate, and if so at what tint?". The
//!   backend itself owns no overprint or DeviceCMYK / Separation / DeviceN
//!   knowledge — every per-channel decision flows through the router.
//! - **Overprint per §11.7.4**: the router consumes `cmd.overprint` (built by
//!   [`super::OverprintResolver`]) directly. OPM=1 zero-component skip, OP=true
//!   leave-untouched, OP=false knock-out — all centralised there.
//! - **Plate writes are deterministic**: each `paint` call walks plates in
//!   the order the caller provides; the backend never reorders.
//!
//! # What this backend does NOT do
//!
//! - Walk content streams. The pipeline composer is the input; the operator
//!   walker is upstream.
//! - Honour `cmd.blend` beyond the implicit `SourceOver` plate convention.
//!   Separation plates are per-ink coverage maps; transparent blending is a
//!   composite concern (and the existing renderer treats `/BM` and `/CA` /
//!   `/ca` as `Normal` / `1.0` for the same reason — see the module-level
//!   doc on `separation_renderer`).
//! - Honour `cmd.color`'s alpha channel for the same reason. Plate coverage
//!   is binary in spirit — paint or skip — modulated by tint, not alpha.

use std::sync::Arc;

use tiny_skia::{FillRule, Mask, Pixmap, Transform};

use crate::content::graphics_state::Matrix;
use crate::error::Result;
use crate::rendering::{guarded_fill_path, guarded_stroke_path};

use super::backend::PaintBackend;
use super::ink::{InkAction, InkRouter};
use super::intent::{PaintKind, PaintSide};
use super::resolved::{InkName, ResolvedPaintCmd};

/// Borrowed view of the per-plate output surface.
///
/// Caller-side construction lets the backend stay alloc-free: the pixmaps
/// and ink names already exist in the caller's owned state, and the backend
/// just borrows them for the lifetime of a `paint` call.
pub(crate) struct SeparationSurface<'a> {
    /// Per-plate output buffers. `pixmaps[i]` is written for `inks[i]`.
    pub(crate) pixmaps: &'a mut [Pixmap],
    /// Names of the inks this surface is targeting. Parallel to `pixmaps`.
    pub(crate) inks: &'a [InkName],
    /// Composition of the page's base transform with any further mapping
    /// the operator walker imposes (Form XObject `/Matrix`, etc.). The
    /// command's own `ctm` is *post*-composed onto this when painting.
    pub(crate) base_transform: Transform,
}

/// Per-plate paint backend driven by [`super::ResolutionPipeline`] output.
///
/// Holds an [`InkRouter`] instance so callers don't have to thread one
/// through. The router is stateless so the backend is too — one instance
/// can be shared across pages and across calls.
pub(crate) struct SeparationBackend {
    router: InkRouter,
}

impl SeparationBackend {
    pub(crate) const fn new() -> Self {
        Self {
            router: InkRouter::new(),
        }
    }
}

impl PaintBackend for SeparationBackend {
    type Surface<'s>
        = SeparationSurface<'s>
    where
        Self: 's;

    fn paint(&mut self, cmd: &ResolvedPaintCmd, surface: Self::Surface<'_>) -> Result<()> {
        // Resolve the clip mask once. Plates share clip geometry because
        // the clip path depends on the CTM and pixmap dimensions, both of
        // which are constant across plates.
        let shared_clip: Option<&Mask> = match &cmd.clip {
            super::resolved::ClipPlan::None => None,
            super::resolved::ClipPlan::Mask(arc) => Some(arc.as_ref()),
        };

        // §8.6.6.3 conformance decision: for a Separation source, does
        // the device have the named colorant plate? If yes, the
        // OverprintPlan's `participating` (which the composer wrote as
        // `[(spot, tint)]`) drives routing directly. If no, the per-plate
        // path falls through to `alt_cmyk_fallback` so the CMYK
        // approximation reaches the standard plates.
        let device_has_spot_plate = match &cmd.overprint.spot_source {
            Some(spot) => surface.inks.iter().any(|i| i == &spot.ink),
            None => false,
        };

        // Build a per-call overprint plan reflecting the device fallback.
        // The router doesn't see surface state, so we surface the
        // §8.6.6.3 fallback to it via the participating list it walks.
        let fallback_plan;
        let effective_plan: &super::resolved::OverprintPlan =
            if cmd.overprint.spot_source.is_some() && !device_has_spot_plate {
                // Device lacks the spot plate → use alt-CMYK approximation.
                let alt = cmd.overprint.alt_cmyk_fallback.unwrap_or([0.0; 4]);
                let mut v = smallvec::SmallVec::new();
                for (j, name) in ["Cyan", "Magenta", "Yellow", "Black"].iter().enumerate() {
                    v.push(super::resolved::ParticipatingChannel {
                        ink: InkName::new(*name),
                        value: alt[j],
                    });
                }
                fallback_plan = super::resolved::OverprintPlan {
                    enabled: cmd.overprint.enabled,
                    mode: cmd.overprint.mode,
                    participating: v,
                    selector: cmd.overprint.selector,
                    all_tint: cmd.overprint.all_tint,
                    spot_source: None,
                    alt_cmyk_fallback: None,
                };
                &fallback_plan
            } else {
                &cmd.overprint
            };

        // Per-plate routing decision and rasterisation.
        for (plate_idx, ink) in surface.inks.iter().enumerate() {
            // The router needs a `&GraphicsState` for its API contract, but
            // doesn't actually read any of its fields — `ResolvedColor` and
            // `OverprintPlan` carry all the info it needs. We use a default
            // GraphicsState so the call compiles without changing the
            // router's surface in this wave.
            let gs = crate::content::graphics_state::GraphicsState::new();
            let action = self.router.route(&gs, ink, &cmd.color, effective_plan);
            let tint = match action {
                InkAction::Skip => continue,
                InkAction::Paint(t) => t,
            };
            let pixmap = &mut surface.pixmaps[plate_idx];
            paint_one_plate(pixmap, cmd, surface.base_transform, tint, shared_clip);
        }
        Ok(())
    }
}

/// Rasterise a single resolved command onto a single plate at the given
/// tint, honouring the command's kind, side, ctm, and (shared) clip mask.
fn paint_one_plate(
    pixmap: &mut Pixmap,
    cmd: &ResolvedPaintCmd,
    base_transform: Transform,
    tint: f32,
    clip: Option<&Mask>,
) {
    let transform = combine_transforms(base_transform, &cmd.ctm);
    match cmd.kind {
        PaintKind::Path { path, fill_rule } => match cmd.side {
            PaintSide::Fill => fill_plate(pixmap, path, transform, tint, fill_rule, clip),
            PaintSide::Stroke => {
                // Stroke parameters (line width, cap, join, miter, dash) are
                // not carried in the resolved command yet — wave 5 stays
                // RGBA-side. Until those land on the pipeline, the stroke
                // is rendered with default tiny_skia stroke settings; the
                // tint and geometry are still correct, the stroke style is
                // the gap. This is the same scope boundary as the inline
                // separation renderer's stroke handling — it pulls those
                // fields off `gs` directly. See follow-up branch.
                let stroke = tiny_skia::Stroke::default();
                stroke_plate(pixmap, path, transform, &stroke, tint, clip);
            },
        },
        // ColorOnly intents are colour-resolution-only — there is no
        // geometry to paint. The pipeline still produces a resolved
        // command for them (the caller may need the resolved RGBA in
        // some non-paint context); the backend skips them.
        PaintKind::ColorOnly => {},
        // Glyph, Image, and Shading variants are provisional in the
        // intent enum today — the operator walker doesn't emit them.
        // Once it does, this backend will need per-variant rasterisation
        // paths (per-plate text raster, per-plate image sample
        // routing, per-plate gradient endpoint routing). Documented
        // gap; surfaced rather than silently dropped because the
        // wave 5 acceptance does not require these to be live.
        PaintKind::Glyph { .. } | PaintKind::Image { .. } | PaintKind::Shading { .. } => {},
    }
}

/// Fill a path into a single plate with the given tint value.
///
/// Mirrors `super::super::separation_renderer::fill_separation`: the tint is
/// encoded as a grayscale colour, alpha=255, `SourceOver` blend so overlapping
/// paints overwrite (last-writer-wins per plate). This matches the per-plate
/// "ink coverage" model — alpha and PDF blend modes are deliberately ignored
/// at the plate level (see module doc).
fn fill_plate(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    transform: Transform,
    tint: f32,
    fill_rule: FillRule,
    clip: Option<&Mask>,
) {
    let gray = (tint.clamp(0.0, 1.0) * 255.0).round() as u8;
    let color = tiny_skia::Color::from_rgba8(gray, gray, gray, 255);
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    paint.blend_mode = tiny_skia::BlendMode::SourceOver;
    guarded_fill_path(pixmap, path, &paint, fill_rule, transform, clip);
}

/// Stroke a path into a single plate with the given tint value.
///
/// Mirrors `super::super::separation_renderer::stroke_separation` for the
/// tint encoding; the stroke parameters come from the caller (the resolved
/// command does not yet carry them — see [`paint_one_plate`]).
fn stroke_plate(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    transform: Transform,
    stroke: &tiny_skia::Stroke,
    tint: f32,
    clip: Option<&Mask>,
) {
    let gray = (tint.clamp(0.0, 1.0) * 255.0).round() as u8;
    let color = tiny_skia::Color::from_rgba8(gray, gray, gray, 255);
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    guarded_stroke_path(pixmap, path, &paint, stroke, transform, clip);
}

/// Compose a base device transform with a PDF CTM. Matches the
/// `combine_transforms` helper in `separation_renderer.rs` so the backend's
/// output is geometrically identical to the existing renderer for the same
/// (path, transform, plate) triple.
fn combine_transforms(base: Transform, ctm: &Matrix) -> Transform {
    base.pre_concat(Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f))
}

// Suppress the unused-Arc warning; the `Arc` import is needed because
// `ClipPlan::Mask` carries `Arc<Mask>` and the backend dereferences it.
const _: Option<Arc<Mask>> = None;

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::{smallvec, SmallVec};

    use super::super::intent::{PaintKind, PaintSide};
    use super::super::resolved::{
        BlendPlan, ClipPlan, InkSelector, OverprintPlan, ParticipatingChannel, ResolvedColor,
        ResolvedPaintCmd,
    };

    fn rect_path() -> tiny_skia::Path {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.line_to(10.0, 0.0);
        pb.line_to(10.0, 10.0);
        pb.line_to(0.0, 10.0);
        pb.close();
        pb.finish().expect("non-empty path")
    }

    fn fresh_pixmap() -> Pixmap {
        Pixmap::new(16, 16).expect("16x16 pixmap allocates")
    }

    fn cmyk_cmd<'a>(
        path: &'a tiny_skia::Path,
        c: f32,
        m: f32,
        y: f32,
        k: f32,
    ) -> ResolvedPaintCmd<'a> {
        ResolvedPaintCmd {
            kind: PaintKind::Path {
                path,
                fill_rule: FillRule::Winding,
            },
            side: PaintSide::Fill,
            color: ResolvedColor::Cmyk { c, m, y, k, a: 1.0 },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: smallvec![
                    ParticipatingChannel {
                        ink: InkName::new("Cyan"),
                        value: c,
                    },
                    ParticipatingChannel {
                        ink: InkName::new("Magenta"),
                        value: m,
                    },
                    ParticipatingChannel {
                        ink: InkName::new("Yellow"),
                        value: y,
                    },
                    ParticipatingChannel {
                        ink: InkName::new("Black"),
                        value: k,
                    },
                ],
                selector: InkSelector::Listed,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        }
    }

    #[test]
    fn fill_routes_cmyk_to_matching_plates() {
        // A DeviceCMYK fill at (0.5, 0.25, 0.0, 1.0) paints the Cyan,
        // Magenta, and Black plates at the respective tints. Yellow gets
        // 0.0 (knock-out under default OP=false), painted as a zero-tint
        // rectangle. This is the per-plate routing the existing inline
        // renderer's tint_for_ink performs — now driven via the pipeline.
        let path = rect_path();
        let cmd = cmyk_cmd(&path, 0.5, 0.25, 0.0, 1.0);
        let mut plates = vec![
            fresh_pixmap(),
            fresh_pixmap(),
            fresh_pixmap(),
            fresh_pixmap(),
        ];
        let inks = [
            InkName::new("Cyan"),
            InkName::new("Magenta"),
            InkName::new("Yellow"),
            InkName::new("Black"),
        ];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();

        // Sample pixel (5, 5), which sits inside the 10x10 rect. The
        // R channel of each plate carries the per-ink tint.
        let sample = |p: &Pixmap| p.data()[(5 * 16 + 5) * 4];
        assert_eq!(sample(&plates[0]), 128, "Cyan tint ≈ 0.5");
        assert_eq!(sample(&plates[1]), 64, "Magenta tint ≈ 0.25");
        // Yellow under default OP=false: painted with 0.0 (knock-out).
        // The plate was zero before; painting zero leaves it zero.
        assert_eq!(sample(&plates[2]), 0, "Yellow tint = 0.0 knock-out");
        assert_eq!(sample(&plates[3]), 255, "Black tint = 1.0 full ink");
    }

    #[test]
    fn fill_skips_spot_plates_when_overprint_enabled() {
        // §11.7.4 with OP=true: the spot plate (not named by the source)
        // is left untouched. We pre-fill it with a sentinel to verify
        // it's not overwritten.
        let path = rect_path();
        let mut cmd = cmyk_cmd(&path, 0.5, 0.0, 0.0, 0.0);
        cmd.overprint.enabled = true;
        let mut plates = vec![fresh_pixmap(), fresh_pixmap()];
        // Pre-fill the spot plate with red so we can detect overwrites.
        let sentinel = tiny_skia::Color::from_rgba8(200, 0, 0, 255);
        let mut spot_paint = tiny_skia::Paint::default();
        spot_paint.set_color(sentinel);
        let full_rect = tiny_skia::Rect::from_xywh(0.0, 0.0, 16.0, 16.0).unwrap();
        plates[1].fill_path(
            &tiny_skia::PathBuilder::from_rect(full_rect),
            &spot_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        let inks = [InkName::new("Cyan"), InkName::new("PANTONE 185 C")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();

        // Cyan painted with tint 0.5 -> 128.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 128);
        // Spot plate untouched -> sentinel R=200 still visible.
        assert_eq!(plates[1].data()[(5 * 16 + 5) * 4], 200);
    }

    #[test]
    fn per_channel_devicen_routes_named_plates() {
        // DeviceN with named channels: each plate paints from the
        // channel matching its ink name. The PerChannel variant is the
        // separation-side colour the pipeline produces for DeviceN
        // sources (once the resolver grows the backend-aware shape;
        // today this test constructs it directly).
        let path = rect_path();
        let cmd = ResolvedPaintCmd {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: FillRule::Winding,
            },
            side: PaintSide::Fill,
            color: ResolvedColor::PerChannel {
                channels: Box::new(smallvec![
                    (InkName::new("PANTONE 185 C"), 0.75),
                    (InkName::new("Dieline"), 0.1),
                ]),
                a: 1.0,
            },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: smallvec![
                    ParticipatingChannel {
                        ink: InkName::new("PANTONE 185 C"),
                        value: 0.75,
                    },
                    ParticipatingChannel {
                        ink: InkName::new("Dieline"),
                        value: 0.1,
                    },
                ],
                selector: InkSelector::Listed,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        };
        let mut plates = vec![fresh_pixmap(), fresh_pixmap()];
        let inks = [InkName::new("PANTONE 185 C"), InkName::new("Dieline")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // 0.75 -> 191 (round half away from zero), 0.1 -> 26.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 191);
        assert_eq!(plates[1].data()[(5 * 16 + 5) * 4], 26);
    }

    #[test]
    fn rgb_color_routes_to_no_plates() {
        // §11.7.4: RGB sources don't route to plates. The router yields
        // Skip for every plate, so every plate stays untouched.
        let path = rect_path();
        let cmd = ResolvedPaintCmd {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: FillRule::Winding,
            },
            side: PaintSide::Fill,
            color: ResolvedColor::Rgba {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            // OverprintResolver produces empty participating for RGB.
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: SmallVec::new(),
                selector: InkSelector::Listed,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        };
        let mut plates = vec![fresh_pixmap()];
        let inks = [InkName::new("Cyan")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // Plate untouched.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 0);
    }

    #[test]
    fn opm1_zero_component_on_cmyk_skips_matching_plate() {
        // §11.7.4.3 OPM=1 Adobe nonzero overprint: a zero source
        // component on DeviceCMYK skips that plate even when overprint
        // is enabled. Pre-fill Magenta with sentinel to verify.
        let path = rect_path();
        let mut cmd = cmyk_cmd(&path, 0.5, 0.0, 0.0, 0.0);
        cmd.overprint.enabled = true;
        cmd.overprint.mode = 1;
        let mut plates = vec![fresh_pixmap(), fresh_pixmap()];
        // Pre-fill Magenta plate with sentinel.
        let sentinel = tiny_skia::Color::from_rgba8(99, 0, 0, 255);
        let mut spot_paint = tiny_skia::Paint::default();
        spot_paint.set_color(sentinel);
        let full_rect = tiny_skia::Rect::from_xywh(0.0, 0.0, 16.0, 16.0).unwrap();
        plates[1].fill_path(
            &tiny_skia::PathBuilder::from_rect(full_rect),
            &spot_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        let inks = [InkName::new("Cyan"), InkName::new("Magenta")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // Cyan painted at 0.5 -> 128.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 128);
        // Magenta untouched under OPM=1 (zero source component).
        assert_eq!(plates[1].data()[(5 * 16 + 5) * 4], 99);
    }

    #[test]
    fn color_only_intent_paints_nothing() {
        // ColorOnly intents carry no geometry — the backend must not
        // attempt to rasterise anything.
        let cmd = ResolvedPaintCmd {
            kind: PaintKind::ColorOnly,
            side: PaintSide::Fill,
            color: ResolvedColor::Cmyk {
                c: 1.0,
                m: 0.0,
                y: 0.0,
                k: 0.0,
                a: 1.0,
            },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: smallvec![ParticipatingChannel {
                    ink: InkName::new("Cyan"),
                    value: 1.0,
                }],
                selector: InkSelector::Listed,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        };
        let mut plates = vec![fresh_pixmap()];
        let inks = [InkName::new("Cyan")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // No geometry painted -> plate stays at zero.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 0);
    }

    /// Drive the per-plate fill through `SeparationBackend::fill_plate` and
    /// in parallel through `separation_renderer::fill_separation`, then
    /// assert each plate's pixel buffer matches byte-for-byte.
    ///
    /// `inks` and `tints` are parallel slices: `tints[i]` is the value the
    /// backend would have routed to `inks[i]`. The caller computes them so
    /// the test specifies exactly what the comparison reference is, instead
    /// of trusting an internal copy of the routing logic.
    fn assert_backend_matches_inline(
        path: &tiny_skia::Path,
        ctm: Matrix,
        cmd: ResolvedPaintCmd<'_>,
        inks: &[InkName],
        tints: &[f32],
        fill_rule: FillRule,
    ) {
        assert_eq!(inks.len(), tints.len());
        // Backend route: call into the real public `paint` API.
        let mut backend_plates: Vec<Pixmap> = (0..inks.len()).map(|_| fresh_pixmap()).collect();
        let surface = SeparationSurface {
            pixmaps: &mut backend_plates,
            inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();

        // Reference route: call `separation_renderer::fill_separation`
        // directly for each plate with the expected per-ink tint and
        // the same composed transform the backend would have used.
        let transform = combine_transforms(Transform::identity(), &ctm);
        let mut inline_plates: Vec<Pixmap> = (0..inks.len()).map(|_| fresh_pixmap()).collect();
        for (i, &tint) in tints.iter().enumerate() {
            crate::rendering::separation_renderer::fill_separation(
                &mut inline_plates[i],
                path,
                transform,
                tint,
                fill_rule,
                None,
            );
        }

        for (i, ink) in inks.iter().enumerate() {
            assert_eq!(
                backend_plates[i].data(),
                inline_plates[i].data(),
                "plate {:?} (index {i}) must match separation_renderer::fill_separation byte-for-byte",
                ink.as_str(),
            );
        }
    }

    #[test]
    fn all_inks_paints_every_plate_at_same_tint() {
        // §8.6.6.3 Separation /All: every plate (process + spot) carries
        // the same tint. The override is carried on OverprintPlan; the
        // colour-resolution output is the alternate-space-evaluated
        // RGBA (composite-only), but the InkRouter consults the
        // selector and ignores the colour for routing.
        let path = rect_path();
        let cmd = ResolvedPaintCmd {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: FillRule::Winding,
            },
            side: PaintSide::Fill,
            color: ResolvedColor::Rgba {
                r: 0.6,
                g: 0.6,
                b: 0.6,
                a: 1.0,
            },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: SmallVec::new(),
                selector: InkSelector::All,
                all_tint: 0.6,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        };
        let mut plates = vec![
            fresh_pixmap(),
            fresh_pixmap(),
            fresh_pixmap(),
            fresh_pixmap(),
        ];
        let inks = [
            InkName::new("Cyan"),
            InkName::new("Magenta"),
            InkName::new("PANTONE 185 C"),
            InkName::new("Dieline"),
        ];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // 0.6 -> 153 (0.6 * 255 = 153.0).
        for (i, ink) in inks.iter().enumerate() {
            assert_eq!(
                plates[i].data()[(5 * 16 + 5) * 4],
                153,
                "/All must paint plate {:?} at the single tint",
                ink.as_str(),
            );
        }
    }

    #[test]
    fn none_inks_paints_no_plates() {
        // §8.6.6.3 Separation /None: nothing visible. Every plate stays
        // at its initial zero value.
        let path = rect_path();
        let cmd = ResolvedPaintCmd {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: FillRule::Winding,
            },
            side: PaintSide::Fill,
            color: ResolvedColor::Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            overprint: OverprintPlan {
                enabled: false,
                mode: 0,
                participating: SmallVec::new(),
                selector: InkSelector::None,
                all_tint: 0.0,
                spot_source: None,
                alt_cmyk_fallback: None,
            },
            blend: BlendPlan::Native(tiny_skia::BlendMode::SourceOver),
            clip: ClipPlan::None,
            ctm: Matrix::identity(),
        };
        let mut plates = vec![fresh_pixmap(), fresh_pixmap()];
        let inks = [InkName::new("Cyan"), InkName::new("PANTONE 185 C")];
        let surface = SeparationSurface {
            pixmaps: &mut plates,
            inks: &inks,
            base_transform: Transform::identity(),
        };
        let mut backend = SeparationBackend::new();
        backend.paint(&cmd, surface).unwrap();
        // Both plates untouched.
        assert_eq!(plates[0].data()[(5 * 16 + 5) * 4], 0);
        assert_eq!(plates[1].data()[(5 * 16 + 5) * 4], 0);
    }

    #[test]
    fn cmyk_cyan_only_matches_fill_separation_byte_for_byte() {
        // Single Cyan-only plate: backend paints Cyan at 0.5, knock-outs
        // other process plates at 0.0 (OP=false). Reference is
        // separation_renderer::fill_separation for each.
        let path = rect_path();
        let cmd = cmyk_cmd(&path, 0.5, 0.0, 0.0, 0.0);
        let inks = [
            InkName::new("Cyan"),
            InkName::new("Magenta"),
            InkName::new("Yellow"),
            InkName::new("Black"),
        ];
        let tints = [0.5, 0.0, 0.0, 0.0];
        assert_backend_matches_inline(
            &path,
            Matrix::identity(),
            cmd,
            &inks,
            &tints,
            FillRule::Winding,
        );
    }

    #[test]
    fn cmyk_mixed_fill_matches_fill_separation_byte_for_byte() {
        // DeviceCMYK fill at (0.5, 0.25, 0.0, 0.7). Every process plate
        // must match its independent fill_separation invocation.
        let path = rect_path();
        let cmd = cmyk_cmd(&path, 0.5, 0.25, 0.0, 0.7);
        let inks = [
            InkName::new("Cyan"),
            InkName::new("Magenta"),
            InkName::new("Yellow"),
            InkName::new("Black"),
        ];
        let tints = [0.5, 0.25, 0.0, 0.7];
        assert_backend_matches_inline(
            &path,
            Matrix::identity(),
            cmd,
            &inks,
            &tints,
            FillRule::Winding,
        );
    }

    #[test]
    fn cmyk_rotated_ctm_matches_fill_separation_byte_for_byte() {
        // Non-identity CTM: 30-degree rotation about origin, applied via
        // the command's `ctm` field. The backend composes ctm with
        // `base_transform`; the reference uses the same composition.
        // Mirrors the wave 5 inline-path rotated-rect probe.
        let path = rect_path();
        let theta = 30.0_f32.to_radians();
        let (s, c) = theta.sin_cos();
        let rotation = Matrix {
            a: c,
            b: s,
            c: -s,
            d: c,
            e: 0.0,
            f: 0.0,
        };
        let mut cmd = cmyk_cmd(&path, 0.5, 0.25, 0.0, 0.7);
        cmd.ctm = rotation;
        let inks = [
            InkName::new("Cyan"),
            InkName::new("Magenta"),
            InkName::new("Yellow"),
            InkName::new("Black"),
        ];
        let tints = [0.5, 0.25, 0.0, 0.7];
        assert_backend_matches_inline(&path, rotation, cmd, &inks, &tints, FillRule::Winding);
    }
}
