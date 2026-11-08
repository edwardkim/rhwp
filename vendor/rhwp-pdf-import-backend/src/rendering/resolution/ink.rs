//! Per-channel ink-routing stage.
//!
//! Subsumes the role of `separation_renderer.rs:714-822` (`tint_for_ink`):
//! given a fully-resolved colour and a target ink, decide whether the backend
//! paints into the plate (and at what tint) or skips it.
//!
//! Today this stage is dead code at the integration layer — the separation
//! renderer still uses its own `tint_for_ink`. The stage is here so that when
//! the separation backend migrates onto the pipeline (follow-up branch) the
//! per-plate decision can be taken by reading the [`ResolvedColor`]
//! produced by [`super::ColorResolver`] plus the [`OverprintPlan`] produced
//! by [`super::OverprintResolver`] without re-walking the source colour
//! space.

use crate::content::graphics_state::GraphicsState;

use super::resolved::{InkName, InkSelector, OverprintPlan, ResolvedColor};

pub(crate) struct InkRouter;

/// Per-plate decision returned by [`InkRouter::route`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InkAction {
    /// Paint into the target plate with the given tint (0.0 = knock out the
    /// plate at the touched pixels; 1.0 = full ink coverage).
    Paint(f32),
    /// Leave the target plate completely untouched (overprint-skip).
    Skip,
}

impl InkRouter {
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Decide what to do with `target_ink` for the given resolved colour.
    ///
    /// Implements the decision tree from ISO 32000-1:2008 §11.7.4 plus
    /// the `/All` and `/None` reserved-name handling from §8.6.6.3:
    ///
    /// - [`InkSelector::All`] (Separation `/All`): paint *every* plate at
    ///   the single tint value carried on [`OverprintPlan::all_tint`],
    ///   including spot plates the source doesn't name. The spec calls
    ///   out this is the one case where a single colorant operator
    ///   targets every output separation.
    /// - [`InkSelector::None`] (Separation `/None`): produce no visible
    ///   output; skip every plate.
    /// - [`InkSelector::Listed`] (every other case): if the colour's
    ///   participating channel set names `target_ink`, paint with the
    ///   channel value; if it doesn't and overprint is enabled, leave
    ///   the plate untouched; if it doesn't and overprint is disabled
    ///   (the spec default), paint 0.0 — "areas of unspecified
    ///   colorants are erased" (the per-plate knockout rule).
    /// - For OPM=1 sources, a zero-valued channel for `target_ink` means
    ///   "colorant not specified" — leave the plate untouched even when
    ///   the channel is in the participating set.
    /// - For DeviceN, a channel literally named `"None"` is dropped per
    ///   §8.6.6.4 and never matches.
    pub(crate) fn route(
        &self,
        _gs: &GraphicsState,
        target_ink: &InkName,
        color: &ResolvedColor,
        overprint: &OverprintPlan,
    ) -> InkAction {
        // /All and /None are reserved Separation colorant names per
        // §8.6.6.3 — the OverprintResolver marks them on the plan's
        // `selector` so the router can short-circuit before walking
        // the per-channel participating list.
        match overprint.selector {
            InkSelector::All => return InkAction::Paint(overprint.all_tint),
            InkSelector::None => return InkAction::Skip,
            InkSelector::Listed => {},
        }

        // Pull the participating channels from the appropriate variant.
        let participating = &overprint.participating;
        if participating.is_empty() {
            // RGB sources don't route to plates at all.
            return InkAction::Skip;
        }

        // Look for our target ink in the participating channels. Per
        // §8.6.6.4 a DeviceN channel named "None" is dropped — we don't
        // even consider it a match.
        if let Some(ch) = participating
            .iter()
            .find(|c| c.ink == *target_ink && c.ink.as_str() != "None")
        {
            // OPM=1 "Adobe nonzero overprint": a zero channel value on
            // DeviceCMYK means "colorant not specified" → skip.
            // §11.7.4.3 limits OPM=1 to DeviceCMYK sources; we identify
            // those by the colour variant. `IccCmyk` is a CMYK source
            // for OPM purposes — the embedded ICC profile only changes
            // the composite-RGB path; the per-plate model is identical.
            let is_cmyk =
                matches!(color, ResolvedColor::Cmyk { .. } | ResolvedColor::IccCmyk { .. });
            if overprint.enabled && overprint.mode == 1 && is_cmyk && ch.value == 0.0 {
                return InkAction::Skip;
            }
            return InkAction::Paint(ch.value);
        }

        // Target ink is outside the source's colorant set. Overprint=true
        // leaves the plate untouched; overprint=false knocks it out.
        if overprint.enabled {
            InkAction::Skip
        } else {
            InkAction::Paint(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    use super::super::resolved::ParticipatingChannel;

    fn fresh_gs() -> GraphicsState {
        GraphicsState::new()
    }

    fn cmyk_color() -> ResolvedColor {
        ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.25,
            y: 0.0,
            k: 0.1,
            a: 1.0,
        }
    }

    fn cmyk_plan(enabled: bool, mode: u8) -> OverprintPlan {
        OverprintPlan {
            enabled,
            mode,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("Cyan"),
                    value: 0.5
                },
                ParticipatingChannel {
                    ink: InkName::new("Magenta"),
                    value: 0.25
                },
                ParticipatingChannel {
                    ink: InkName::new("Yellow"),
                    value: 0.0
                },
                ParticipatingChannel {
                    ink: InkName::new("Black"),
                    value: 0.1
                },
            ],
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        }
    }

    #[test]
    fn cmyk_paints_named_channel() {
        let gs = fresh_gs();
        let plan = cmyk_plan(false, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.25));
    }

    #[test]
    fn spot_plate_outside_cmyk_knocks_out_by_default() {
        // §11.7.4 default: overprint=false → unspecified plates knock out
        // (paint 0.0 to erase underlying ink).
        let gs = fresh_gs();
        let plan = cmyk_plan(false, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("PANTONE 185 C"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.0));
    }

    #[test]
    fn spot_plate_outside_cmyk_skips_when_overprint() {
        // §11.7.4 with OP=true: unspecified plates are left untouched.
        let gs = fresh_gs();
        let plan = cmyk_plan(true, 0);
        let color = cmyk_color();
        let action = InkRouter::new().route(&gs, &InkName::new("PANTONE 185 C"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn opm_one_skips_zero_components_on_cmyk() {
        // §11.7.4.3 OPM=1: a zero channel on DeviceCMYK is "colorant not
        // specified" → leave the matching plate alone.
        let gs = fresh_gs();
        let plan = cmyk_plan(true, 1);
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        // Plan reflects the zero values; ensure routing acts on them.
        let mut plan = plan;
        plan.participating[1].value = 0.0; // Magenta = 0
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn opm_zero_paints_zero_components_normally() {
        // §11.7.4 OPM=0 (default): zero is *not* special — paint it
        // (which knocks the plate out at the painted pixels).
        let gs = fresh_gs();
        let mut plan = cmyk_plan(true, 0);
        plan.participating[1].value = 0.0;
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Magenta"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.0));
    }

    #[test]
    fn rgb_source_skips_all_plates() {
        // §11.7.4 doesn't define overprint for RGB sources. The plan's
        // participating set is empty (by construction in OverprintResolver),
        // so every plate gets Skip.
        let gs = fresh_gs();
        let plan = OverprintPlan {
            enabled: true,
            mode: 0,
            participating: smallvec![],
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let color = ResolvedColor::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Cyan"), &color, &plan);
        assert_eq!(action, InkAction::Skip);
    }

    #[test]
    fn all_inks_paints_every_plate_at_single_tint() {
        // §8.6.6.3: Separation /All names every output plate. Both
        // process and spot plates receive the same tint, regardless of
        // overprint state and regardless of whether participating
        // happens to list them. The router consults the
        // `selector: InkSelector::All` marker to short-circuit.
        let gs = fresh_gs();
        // Composite colour resolution may still produce gray-at-tint;
        // the router does not read `color` when selector is All/None.
        let color = ResolvedColor::Rgba {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: smallvec![],
            selector: InkSelector::All,
            all_tint: 0.6,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let router = InkRouter::new();
        for ink_name in [
            "Cyan",
            "Magenta",
            "Yellow",
            "Black",
            "PANTONE 185 C",
            "Dieline",
        ] {
            let action = router.route(&gs, &InkName::new(ink_name), &color, &plan);
            assert_eq!(action, InkAction::Paint(0.6), "/All must paint plate {ink_name}");
        }
    }

    #[test]
    fn all_inks_paints_even_when_overprint_enabled() {
        // /All is unconditional: spec doesn't carve out an overprint
        // exception. Same tint, every plate, OP=true.
        let gs = fresh_gs();
        let color = ResolvedColor::Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let plan = OverprintPlan {
            enabled: true,
            mode: 1, // even with OPM=1 active
            participating: smallvec![],
            selector: InkSelector::All,
            all_tint: 1.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let router = InkRouter::new();
        for ink_name in ["Cyan", "Magenta", "Yellow", "Black", "PANTONE Reflex Blue"] {
            let action = router.route(&gs, &InkName::new(ink_name), &color, &plan);
            assert_eq!(action, InkAction::Paint(1.0), "/All ignores overprint; plate {ink_name}");
        }
    }

    #[test]
    fn none_inks_skips_every_plate() {
        // §8.6.6.3: Separation /None produces no visible output. Every
        // plate skips, regardless of overprint state.
        let gs = fresh_gs();
        let color = ResolvedColor::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: smallvec![],
            selector: InkSelector::None,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let router = InkRouter::new();
        for ink_name in [
            "Cyan",
            "Magenta",
            "Yellow",
            "Black",
            "PANTONE 185 C",
            "Dieline",
        ] {
            let action = router.route(&gs, &InkName::new(ink_name), &color, &plan);
            assert_eq!(action, InkAction::Skip, "/None must skip plate {ink_name}");
        }
    }

    #[test]
    fn devicen_channel_named_none_is_dropped() {
        // §8.6.6.4: a DeviceN channel literally named "None" is dropped
        // from per-plate routing. Even if a target plate happens to be
        // named "None", the router does not treat that as a match — it
        // falls through to the unspecified-plate path (knock out when
        // overprint is off).
        let gs = fresh_gs();
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("None"),
                    value: 0.5,
                },
                ParticipatingChannel {
                    ink: InkName::new("PANTONE 185 C"),
                    value: 0.75,
                },
            ],
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let color = ResolvedColor::PerChannel {
            channels: Box::new(smallvec![
                (InkName::new("None"), 0.5),
                (InkName::new("PANTONE 185 C"), 0.75),
            ]),
            a: 1.0,
        };
        let router = InkRouter::new();
        // "None" target falls through to other_plate_action — knock out.
        let action = router.route(&gs, &InkName::new("None"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.0));
        // Real ink still routes normally.
        let action = router.route(&gs, &InkName::new("PANTONE 185 C"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.75));
    }

    #[test]
    fn per_channel_devicen_routes_by_ink_name() {
        // DeviceN with named channels: route by exact ink name.
        let gs = fresh_gs();
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("PANTONE 185 C"),
                    value: 0.75
                },
                ParticipatingChannel {
                    ink: InkName::new("Dieline"),
                    value: 0.1
                },
            ],
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        let color = ResolvedColor::PerChannel {
            channels: Box::new(smallvec![
                (InkName::new("PANTONE 185 C"), 0.75),
                (InkName::new("Dieline"), 0.1),
            ]),
            a: 1.0,
        };
        let action = InkRouter::new().route(&gs, &InkName::new("Dieline"), &color, &plan);
        assert_eq!(action, InkAction::Paint(0.1));
    }
}
