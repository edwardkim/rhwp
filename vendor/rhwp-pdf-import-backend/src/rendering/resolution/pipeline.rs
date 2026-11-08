//! Pipeline composer — orchestrates the resolution stages.
//!
//! [`ResolutionPipeline::resolve`] runs each stage in sequence, feeding the
//! output of one into the input of the next where the data flow demands it,
//! and produces the final [`ResolvedPaintCmd`] the backend consumes.
//!
//! The order is:
//!
//! 1. **Colour** — `LogicalColor` → `ResolvedColor`. Reads `ctx` (for ICC,
//!    OutputIntent, tint-transform streams) and the intent's components.
//!    Folds in `gs.fill_alpha` / `gs.stroke_alpha` per `side`.
//! 2. **Overprint** — produces an `OverprintPlan` from `gs` + the resolved
//!    colour. Reads channel values from the colour to populate the
//!    participating-channels list.
//! 3. **Blend** — produces a `BlendPlan` from `gs.blend_mode`. Doesn't
//!    depend on colour or overprint.
//! 4. **Clip** — wraps the operator walker's composed clip mask reference
//!    into a `ClipPlan`. The composition itself is the walker's
//!    responsibility (see `apply_pending_clip` in the existing renderer);
//!    the resolver just packages the result.
//!
//! The `InkRouter` stage is not invoked here — it runs per target ink
//! inside the backend's [`super::PaintBackend::paint`] implementation for
//! per-plate backends. Composite backends don't call it at all.

use std::sync::Arc;

use crate::error::Result;

use super::blend::BlendResolver;
use super::clip::ClipResolver;
use super::color::ColorResolver;
use super::context::ResolutionContext;
use super::intent::{LogicalColor, PaintIntent, PaintSide};
use super::overprint::OverprintResolver;
use super::resolved::{InkSelector, ResolvedPaintCmd};

/// Composable resolution pipeline. Holds one instance of each stage.
///
/// Stages are stateless, so a single `ResolutionPipeline` can be shared
/// across all intents for all pages.
pub(crate) struct ResolutionPipeline {
    pub(crate) color: ColorResolver,
    pub(crate) overprint: OverprintResolver,
    pub(crate) blend: BlendResolver,
    pub(crate) clip: ClipResolver,
}

impl ResolutionPipeline {
    /// Build a default pipeline with every stage's stateless constructor.
    pub(crate) const fn new() -> Self {
        Self {
            color: ColorResolver::new(),
            overprint: OverprintResolver::new(),
            blend: BlendResolver::new(),
            clip: ClipResolver::new(),
        }
    }

    /// Resolve a single paint intent.
    ///
    /// `clip_mask` is the composed clip mask the operator walker maintains.
    /// We pass it through `ClipResolver` rather than reaching into the walker
    /// state directly, so the same code path works when the walker has no
    /// active clip (passes `None`).
    pub(crate) fn resolve<'a>(
        &self,
        intent: &PaintIntent<'a>,
        ctx: &ResolutionContext,
        clip_mask: Option<Arc<tiny_skia::Mask>>,
    ) -> Result<ResolvedPaintCmd<'a>> {
        let alpha = match intent.side {
            PaintSide::Fill => intent.gs.fill_alpha,
            PaintSide::Stroke => intent.gs.stroke_alpha,
        };

        let color = self.color.resolve(&intent.color, ctx, alpha)?;
        let mut overprint = self.overprint.resolve(intent.gs, intent.side, &color);
        // §8.6.6.3 reserved Separation colorant-name override: stamp the
        // per-plate routing selector by inspecting the source colour
        // space. The OverprintResolver doesn't see the colour space
        // (only the resolved colour), so the override happens here on
        // the composer where both are available.
        apply_inks_selector_override(&intent.color, &mut overprint, ctx);
        let blend = self.blend.resolve(intent.gs);
        let clip = self.clip.resolve_with_mask(clip_mask);

        Ok(ResolvedPaintCmd {
            // PaintKind is `Copy` — every variant holds only borrows
            // and primitive copy types — so the memberwise copy is a
            // single dereference.
            kind: intent.kind,
            side: intent.side,
            color,
            overprint,
            blend,
            clip,
            ctm: intent.ctm,
        })
    }
}

/// Inspect the source [`LogicalColor`] for an ISO 32000-1 §8.6.6.3
/// reserved Separation colorant name (`/All`, `/None`) and stamp the
/// per-plate routing selector on the overprint plan. Composite (RGB)
/// backends ignore the selector; the per-plate [`super::InkRouter`]
/// honours it.
///
/// Also rewrites `participating` so per-plate routing of a non-reserved
/// Separation source targets the named spot plate at the source tint —
/// rather than the alternate-CMYK decomposition the resolver evaluates
/// for composite output. The spec model: Separation `/Pantone-185 1 scn`
/// paints the Pantone-185 plate at 1.0 (and per §11.7.4 knocks out
/// other plates under OP=false, leaves them alone under OP=true). The
/// alternate CMYK is for composite preview only — it must not drive
/// the C/M/Y/K plates.
///
/// DeviceN sources keep the per-channel participating list the
/// OverprintResolver produced from `ResolvedColor::PerChannel`.
fn apply_inks_selector_override(
    color: &LogicalColor,
    overprint: &mut super::resolved::OverprintPlan,
    ctx: &ResolutionContext,
) {
    let LogicalColor::Spaced { space, components } = color else {
        return;
    };
    let Some(arr) = space.as_array() else {
        return;
    };
    let type_name = arr.first().and_then(|o| o.as_name());
    if type_name == Some("DeviceN") {
        apply_devicen_override(arr, components, overprint);
        return;
    }
    if type_name != Some("Separation") {
        return;
    }
    match arr.get(1).and_then(|o| o.as_name()) {
        Some("All") => {
            overprint.selector = InkSelector::All;
            overprint.all_tint = components.first().copied().unwrap_or(0.0);
        },
        Some("None") => {
            overprint.selector = InkSelector::None;
            overprint.all_tint = 0.0;
        },
        Some(spot_name) => {
            // §8.6.6.3: a conforming device with the named colorant
            // paints that colorant directly; without it, the alternate
            // colour space and tint transform are used to approximate
            // the colorant. Record the spot identity here; the alt-CMYK
            // decomposition (computed from the source's tint transform
            // when the alternate is DeviceCMYK) is recorded too so the
            // per-plate backend can pick the right routing per-surface.
            use super::resolved::{InkName, ParticipatingChannel, SpotSource};
            let tint = components.first().copied().unwrap_or(0.0);
            // Replace participating with the spot-only entry. The
            // per-plate backend reads `spot_source` to decide whether
            // to honour this entry (device has spot plate) or fall
            // through to `alt_cmyk_fallback` (device doesn't).
            let mut v = smallvec::SmallVec::<[ParticipatingChannel; 8]>::new();
            v.push(ParticipatingChannel {
                ink: InkName::new(spot_name),
                value: tint,
            });
            overprint.participating = v;
            overprint.spot_source = Some(SpotSource {
                ink: InkName::new(spot_name),
                tint,
            });
            // Stash the alternate-CMYK decomposition for the §8.6.6.3
            // fallback (device lacks the spot plate). Evaluating the
            // tint transform twice — once here, once inside
            // ColorResolver — is wasteful but keeps the resolver
            // surface unchanged.
            if let Some(alt) = eval_separation_alt_cmyk(arr, components.first().copied(), ctx) {
                overprint.alt_cmyk_fallback = Some(alt);
            }
        },
        None => {},
    }
}

/// ISO 32000-1 §8.6.6.4: a DeviceN source declares an ordered list of
/// colorant names. Each operator component is a per-colorant tint; the
/// per-plate router maps each tint to the plate sharing the colorant's
/// name. Stamp `participating` with `(name_i, tint_i)` pairs so the
/// router walks them directly. Channels literally named "None" are
/// dropped per spec.
fn apply_devicen_override(
    arr: &[crate::object::Object],
    components: &[f32],
    overprint: &mut super::resolved::OverprintPlan,
) {
    use super::resolved::{InkName, ParticipatingChannel};
    let Some(names_obj) = arr.get(1) else {
        return;
    };
    let Some(names_arr) = names_obj.as_array() else {
        return;
    };
    let mut v = smallvec::SmallVec::new();
    for (i, n) in names_arr.iter().enumerate() {
        let Some(name) = n.as_name() else { continue };
        if name == "None" {
            continue;
        }
        let value = components.get(i).copied().unwrap_or(0.0);
        v.push(ParticipatingChannel {
            ink: InkName::new(name),
            value,
        });
    }
    overprint.participating = v;
}

/// Evaluate a Separation source's tint transform at `tint` and return
/// the resulting CMYK quadruple if the alternate space is DeviceCMYK.
/// Returns `None` for non-CMYK alternates (the spec §8.6.6.3 fallback
/// to alt-CMYK only applies when the alternate is, in fact, CMYK).
fn eval_separation_alt_cmyk(
    arr: &[crate::object::Object],
    tint: Option<f32>,
    ctx: &ResolutionContext,
) -> Option<[f32; 4]> {
    use crate::object::Object;
    let tint = tint?;
    let alt_cs = arr.get(2)?;
    if alt_cs.as_name() != Some("DeviceCMYK") && alt_cs.as_name() != Some("CMYK") {
        return None;
    }
    let func_obj_raw = arr.get(3)?;
    let func_obj_owned;
    let func_obj: &Object = match ctx.doc.resolve_object(func_obj_raw) {
        Ok(resolved) => {
            func_obj_owned = resolved;
            &func_obj_owned
        },
        Err(_) => func_obj_raw,
    };
    let func_dict = func_obj.as_dict()?;
    let func_type = func_dict.get("FunctionType").and_then(|o| o.as_integer())?;
    match func_type {
        2 => {
            // Type 2 exponential: y_j = C0_j + tint^N * (C1_j - C0_j).
            let n = func_dict
                .get("N")
                .and_then(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                .unwrap_or(1.0) as f32;
            let c0 = func_dict.get("C0").and_then(|o| o.as_array());
            let c1 = func_dict.get("C1").and_then(|o| o.as_array());
            let pow = if n == 1.0 { tint } else { tint.powf(n) };
            let mut out = [0.0f32; 4];
            for j in 0..4 {
                let c0j = c0
                    .and_then(|a| a.get(j))
                    .and_then(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                    .unwrap_or(0.0) as f32;
                let c1j = c1
                    .and_then(|a| a.get(j))
                    .and_then(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                    .unwrap_or(if j == 3 { 0.0 } else { 1.0 }) as f32;
                out[j] = (c0j + pow * (c1j - c0j)).clamp(0.0, 1.0);
            }
            Some(out)
        },
        4 => {
            // Type 4 PostScript calculator: invoke the shared evaluator.
            let Object::Stream { dict, .. } = func_obj else {
                return None;
            };
            let bytes = func_obj.decode_stream_data().ok()?;
            let domain = dict
                .get("Domain")
                .and_then(|o| o.as_array())
                .map(|a| {
                    a.as_chunks::<2>()
                        .0
                        .iter()
                        .map(|c| {
                            let lo = c[0]
                                .as_real()
                                .or_else(|| c[0].as_integer().map(|i| i as f64))
                                .unwrap_or(0.0);
                            let hi = c[1]
                                .as_real()
                                .or_else(|| c[1].as_integer().map(|i| i as f64))
                                .unwrap_or(1.0);
                            [lo, hi]
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let range = dict
                .get("Range")
                .and_then(|o| o.as_array())
                .map(|a| {
                    a.as_chunks::<2>()
                        .0
                        .iter()
                        .map(|c| {
                            let lo = c[0]
                                .as_real()
                                .or_else(|| c[0].as_integer().map(|i| i as f64))
                                .unwrap_or(0.0);
                            let hi = c[1]
                                .as_real()
                                .or_else(|| c[1].as_integer().map(|i| i as f64))
                                .unwrap_or(1.0);
                            [lo, hi]
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let inputs = vec![tint as f64];
            let out =
                crate::functions::evaluate_type4_clamped(&bytes, &inputs, &domain, &range).ok()?;
            if out.len() < 4 {
                return None;
            }
            Some([
                out[0].clamp(0.0, 1.0) as f32,
                out[1].clamp(0.0, 1.0) as f32,
                out[2].clamp(0.0, 1.0) as f32,
                out[3].clamp(0.0, 1.0) as f32,
            ])
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::graphics_state::{GraphicsState, Matrix};
    use crate::object::Object;
    use smallvec::smallvec;
    use std::collections::HashMap;

    use super::super::intent::{DeviceColor, LogicalColor, PaintKind};
    use super::super::resolved::{BlendPlan, ClipPlan, ResolvedColor};
    use super::super::test_support::fixture_doc;

    fn rectangle_path() -> tiny_skia::Path {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.line_to(10.0, 0.0);
        pb.line_to(10.0, 10.0);
        pb.line_to(0.0, 10.0);
        pb.close();
        pb.finish().expect("non-empty path")
    }

    #[test]
    fn pipeline_resolves_device_gray_path_fill() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();

        let path = rectangle_path();
        let mut gs = GraphicsState::new();
        gs.fill_alpha = 0.8;
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Gray(0.25)),
            ctm: Matrix::identity(),
        };

        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();

        // Colour: Gray(0.25) folded with fill_alpha=0.8 → Rgba(0.25, 0.25, 0.25, 0.8).
        match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => {
                assert!((r - 0.25).abs() < 1e-6);
                assert!((g - 0.25).abs() < 1e-6);
                assert!((b - 0.25).abs() < 1e-6);
                assert!((a - 0.8).abs() < 1e-6);
            },
            _ => panic!("expected Rgba"),
        }

        // Default GS: overprint disabled, mode 0.
        assert!(!cmd.overprint.enabled);
        assert_eq!(cmd.overprint.mode, 0);

        // Default GS blend = Normal → SourceOver native.
        match cmd.blend {
            BlendPlan::Native(tiny_skia::BlendMode::SourceOver) => {},
            other => panic!("expected SourceOver, got {other:?}"),
        }

        // No clip mask passed.
        match cmd.clip {
            ClipPlan::None => {},
            _ => panic!("expected ClipPlan::None"),
        }
    }

    #[test]
    fn pipeline_passes_through_clip_mask_arc() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let gs = GraphicsState::new();
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Gray(0.0)),
            ctm: Matrix::identity(),
        };

        let mask = Arc::new(tiny_skia::Mask::new(4, 4).unwrap());
        let cmd = pipeline.resolve(&intent, &ctx, Some(mask.clone())).unwrap();
        match cmd.clip {
            ClipPlan::Mask(m) => assert!(Arc::ptr_eq(&m, &mask)),
            _ => panic!("expected ClipPlan::Mask"),
        }
    }

    #[test]
    fn pipeline_picks_stroke_alpha_for_stroke_side() {
        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let mut gs = GraphicsState::new();
        gs.fill_alpha = 0.4;
        gs.stroke_alpha = 0.6;
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Stroke,
            gs: &gs,
            color: LogicalColor::Device(DeviceColor::Rgb(1.0, 0.0, 0.0)),
            ctm: Matrix::identity(),
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();
        match cmd.color {
            ResolvedColor::Rgba { a, .. } => assert!((a - 0.6).abs() < 1e-6),
            _ => panic!("expected Rgba"),
        }
    }

    #[test]
    fn pipeline_resolves_spaced_separation_with_type4_end_to_end() {
        // Full pipeline path for the regression case: an `scn` against a
        // Separation/DeviceCMYK/Type-4 space must resolve to a non-black
        // RGBA — not the `1.0 - tint = 0` solid black the existing inline
        // path produces. This is the same logic exercised in
        // `color::tests::separation_with_type4_calculator_evaluates_program`
        // but here we run it through the whole pipeline so we also verify
        // the resolver composition (alpha fold, overprint plan, blend plan,
        // clip plan) doesn't interfere.
        let program = b"{ 0.0 exch 0.0 0.0 }";
        let mut func_dict: HashMap<String, Object> = HashMap::new();
        func_dict.insert("FunctionType".into(), Object::Integer(4));
        let func_obj = Object::Stream {
            dict: func_dict,
            data: program.to_vec().into(),
        };
        let space = Object::Array(vec![
            Object::Name("Separation".into()),
            Object::Name("MagentaSpot".into()),
            Object::Name("DeviceCMYK".into()),
            func_obj,
        ]);

        let doc = fixture_doc();
        let spaces = HashMap::new();
        let ctx = ResolutionContext::new(&doc, &spaces);
        let pipeline = ResolutionPipeline::new();
        let path = rectangle_path();
        let gs = GraphicsState::new();
        let intent = PaintIntent {
            kind: PaintKind::Path {
                path: &path,
                fill_rule: tiny_skia::FillRule::Winding,
            },
            side: PaintSide::Fill,
            gs: &gs,
            color: LogicalColor::Spaced {
                space: &space,
                components: smallvec![1.0],
            },
            ctm: Matrix::identity(),
        };
        let cmd = pipeline.resolve(&intent, &ctx, None).unwrap();
        // Separation with a DeviceCMYK alternate now emits Cmyk so the
        // per-plate router has the channel decomposition. Project to
        // RGBA through the same composite converter the renderer uses
        // (process-ink `cmyk_to_rgb_via_intent`) and pin the process-ink
        // magenta corner #EC008C = (0.9255, 0, 0.5490).
        let (r, g, b, a) = match cmd.color {
            ResolvedColor::Rgba { r, g, b, a } => (r, g, b, a),
            ResolvedColor::Cmyk { c, m, y, k, a } => {
                let (rr, gg, bb) =
                    crate::rendering::resolution::color::cmyk_to_rgb_via_intent(c, m, y, k, &ctx);
                (rr, gg, bb, a)
            },
            other => panic!("expected Rgba or Cmyk; got {other:?}"),
        };
        assert!((r - 0.9255).abs() < 1e-3, "r={r}");
        assert!((g - 0.0).abs() < 1e-3, "g={g}");
        assert!((b - 0.5490).abs() < 1e-3, "b={b}");
        assert!((a - 1.0).abs() < 1e-3, "a={a}");
    }
}
