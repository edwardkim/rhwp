//! Overprint-resolution stage.
//!
//! Reads `/OP`, `/op`, and `/OPM` from the graphics state (already parsed and
//! stamped by [`super::super::ext_gstate`]) and projects them into an
//! [`OverprintPlan`] the backend can consume. ISO 32000-1:2008 §11.7.4
//! defines the model.
//!
//! Today this stage carries informational metadata for the composite
//! backend — the composite renderer has never honoured overprint and we
//! aren't changing that behaviour in this branch (a deliberate
//! out-of-scope item documented in the scope review). The plan exists so
//! that:
//!
//! 1. The separation backend, once migrated onto the pipeline, gets a
//!    single resolution call instead of the inline branching in
//!    `separation_renderer.rs:714-822`.
//! 2. A future composite backend can opt into overprint simulation
//!    (PDF/X-style press preview) by consuming the same plan without any
//!    additional resolver work.

use smallvec::SmallVec;

use crate::content::graphics_state::GraphicsState;

use super::intent::PaintSide;
use super::resolved::{InkName, InkSelector, OverprintPlan, ParticipatingChannel, ResolvedColor};

pub(crate) struct OverprintResolver;

impl OverprintResolver {
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Compute the [`OverprintPlan`] for an intent.
    ///
    /// `side` selects whether we read `/OP` (stroke) or `/op` (fill); `color`
    /// supplies the resolved channels, which become the plan's participating
    /// set.
    pub(crate) fn resolve(
        &self,
        gs: &GraphicsState,
        side: PaintSide,
        color: &ResolvedColor,
    ) -> OverprintPlan {
        let enabled = match side {
            PaintSide::Fill => gs.fill_overprint,
            PaintSide::Stroke => gs.stroke_overprint,
        };
        let mode = gs.overprint_mode;

        let participating: SmallVec<[ParticipatingChannel; 8]> = match color {
            ResolvedColor::Rgba { .. } => {
                // RGB sources don't route to ink plates per §11.7.4.
                // Backends that act on plates skip RGB intents entirely;
                // backends that act on composite RGB ignore the plan.
                SmallVec::new()
            },
            ResolvedColor::Cmyk { c, m, y, k, .. } | ResolvedColor::IccCmyk { c, m, y, k, .. } => {
                // `IccCmyk` carries the same four-channel decomposition
                // as `Cmyk` for the per-plate path; the only difference
                // is the side-payload `(r, g, b)` the composite path
                // reads. The plate router doesn't care about RGB.
                let mut v = SmallVec::new();
                v.push(ParticipatingChannel {
                    ink: InkName::new("Cyan"),
                    value: *c,
                });
                v.push(ParticipatingChannel {
                    ink: InkName::new("Magenta"),
                    value: *m,
                });
                v.push(ParticipatingChannel {
                    ink: InkName::new("Yellow"),
                    value: *y,
                });
                v.push(ParticipatingChannel {
                    ink: InkName::new("Black"),
                    value: *k,
                });
                v
            },
            ResolvedColor::PerChannel { channels, .. } => channels
                .iter()
                .map(|(ink, v)| ParticipatingChannel {
                    ink: ink.clone(),
                    value: *v,
                })
                .collect(),
        };

        OverprintPlan {
            enabled,
            mode,
            participating,
            // Default routing selector. The pipeline composer overrides
            // this when the source colour space is `/Separation /All`
            // or `/Separation /None` (ISO 32000-1 §8.6.6.3); that's the
            // only place the reserved colorant names are recognised so
            // the OverprintResolver stays stateless and source-agnostic.
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_gs() -> GraphicsState {
        GraphicsState::new()
    }

    #[test]
    fn default_state_yields_disabled_plan() {
        // ISO 32000-1 §11.7.4 default: OP/op false, OPM 0. The plan is a
        // marker only; backends short-circuit.
        let gs = fresh_gs();
        let color = ResolvedColor::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let r = OverprintResolver::new();
        let plan = r.resolve(&gs, PaintSide::Fill, &color);
        assert!(!plan.enabled);
        assert_eq!(plan.mode, 0);
    }

    #[test]
    fn fill_op_reads_lowercase_op() {
        // /op is the non-stroking overprint per §11.7.4; the resolver reads
        // it for Fill intents.
        let mut gs = fresh_gs();
        gs.fill_overprint = true;
        gs.stroke_overprint = false;
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        let plan = OverprintResolver::new().resolve(&gs, PaintSide::Fill, &color);
        assert!(plan.enabled);
    }

    #[test]
    fn stroke_op_reads_uppercase_op() {
        let mut gs = fresh_gs();
        gs.fill_overprint = false;
        gs.stroke_overprint = true;
        let color = ResolvedColor::Cmyk {
            c: 0.5,
            m: 0.0,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        let plan = OverprintResolver::new().resolve(&gs, PaintSide::Stroke, &color);
        assert!(plan.enabled);
    }

    #[test]
    fn cmyk_color_lists_four_process_inks_with_values() {
        let mut gs = fresh_gs();
        gs.fill_overprint = true;
        let color = ResolvedColor::Cmyk {
            c: 0.1,
            m: 0.2,
            y: 0.3,
            k: 0.4,
            a: 1.0,
        };
        let plan = OverprintResolver::new().resolve(&gs, PaintSide::Fill, &color);
        assert_eq!(plan.participating.len(), 4);
        assert_eq!(plan.participating[0].ink, InkName::new("Cyan"));
        assert!((plan.participating[0].value - 0.1).abs() < 1e-6);
        assert_eq!(plan.participating[1].ink, InkName::new("Magenta"));
        assert!((plan.participating[1].value - 0.2).abs() < 1e-6);
        assert_eq!(plan.participating[3].ink, InkName::new("Black"));
        assert!((plan.participating[3].value - 0.4).abs() < 1e-6);
    }

    #[test]
    fn rgb_color_lists_no_participating_channels() {
        // §11.7.4 overprint is a separation concept; RGB sources don't
        // route to plates. The resolver returns an empty participating
        // set so per-plate backends naturally skip the intent.
        let mut gs = fresh_gs();
        gs.fill_overprint = true;
        let color = ResolvedColor::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let plan = OverprintResolver::new().resolve(&gs, PaintSide::Fill, &color);
        assert!(plan.participating.is_empty());
    }

    #[test]
    fn opm_passthrough() {
        // /OPM is opaque to this stage — we just pass it through. The
        // per-channel routing logic (the OPM=1 "zero = unspecified"
        // rule) lives in the InkRouter stage where it can consult the
        // target ink.
        let mut gs = fresh_gs();
        gs.overprint_mode = 1;
        gs.fill_overprint = true;
        let color = ResolvedColor::Cmyk {
            c: 0.0,
            m: 0.5,
            y: 0.0,
            k: 0.0,
            a: 1.0,
        };
        let plan = OverprintResolver::new().resolve(&gs, PaintSide::Fill, &color);
        assert_eq!(plan.mode, 1);
    }
}
