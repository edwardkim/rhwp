//! Resolved paint command — what the backend consumes.
//!
//! After the [`super::ResolutionPipeline`] runs, every dimension of the paint
//! operation has been fully evaluated:
//!
//! - **Colour** is concrete: RGBA for composite backends, CMYK for CMYK
//!   backends, or per-channel tints for separation / DeviceN backends.
//! - **Overprint** is a finished plan: which channels participate, and (for
//!   OPM=1 DeviceCMYK sources) which zero components pass through to leave
//!   the target plate untouched.
//! - **Blend** is classified into "native — use this tiny-skia mode" or
//!   "simulated — the backend must run the compositing op manually".
//! - **Clip** is a concrete reference to the composed mask, or `None`.
//!
//! The backend never sees `LogicalColor` or raw `GraphicsState`. It receives a
//! [`ResolvedPaintCmd`] and translates it into a draw call.

use std::sync::Arc;

use smallvec::SmallVec;

use crate::content::graphics_state::Matrix;

use super::intent::{PaintKind, PaintSide};

/// Named ink for per-channel routing. Process inks use the canonical PDF
/// names; spot inks carry the colorant name from the `Separation`/`DeviceN`
/// colour-space declaration. The `String` is the colorant name as it appears
/// in the PDF (e.g. `"Cyan"`, `"PANTONE 185 C"`, `"Dieline"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InkName(pub(crate) String);

impl InkName {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Final colour the backend will paint with. Variants give backends a choice
/// of representation: a composite RGB backend ignores the `Cmyk` and
/// `PerChannel` variants (the pipeline guarantees it produces `Rgba` for
/// such backends); a separation backend ignores `Rgba` and reads
/// `PerChannel` (or `Cmyk` for process plates).
#[derive(Debug, Clone)]
pub(crate) enum ResolvedColor {
    /// sRGB-tagged colour with straight alpha. The alpha component already
    /// folds in `gs.fill_alpha` / `gs.stroke_alpha`.
    Rgba { r: f32, g: f32, b: f32, a: f32 },
    /// DeviceCMYK colour with straight alpha. Backends that emit CMYK
    /// directly (PDF/X-ready preflight, plate output) consume this.
    Cmyk {
        c: f32,
        m: f32,
        y: f32,
        k: f32,
        a: f32,
    },
    /// Dual-payload variant for `/ICCBased` N=4 sources whose embedded
    /// profile compiled into a usable CMM. Carries both the composite-
    /// ready RGB (pre-converted via the embedded profile per
    /// ISO 32000-1:2008 §8.6.5.5) and the raw CMYK quadruple from the
    /// operator, so:
    ///
    /// - Composite consumers paint `(r, g, b, a)` as-is; the embedded
    ///   profile already drove the CMYK→RGB conversion. No further
    ///   OutputIntent-aware projection is needed (and would be wrong —
    ///   §8.6.5.5 says the embedded profile is the conversion source).
    /// - Per-plate consumers route the four CMYK components by named
    ///   channel exactly as for the `Cmyk` variant. The ICC conversion
    ///   is composite-only — the plates ARE the press-target ink
    ///   coverage, so the original CMYK is what each C/M/Y/K plate must
    ///   carry.
    ///
    /// The two payloads describe the same logical paint command for
    /// two different surfaces. Keeping them on a single variant means
    /// the resolver still has a single emit site for ICCBased N=4 and
    /// every downstream consumer reads the field it needs.
    IccCmyk {
        r: f32,
        g: f32,
        b: f32,
        c: f32,
        m: f32,
        y: f32,
        k: f32,
        a: f32,
    },
    /// Per-channel tints for separation / DeviceN backends. The pipeline
    /// orders the channels to match the source colour space's declared
    /// colorant order.
    ///
    /// The channel vector is boxed so the enum's footprint stays small —
    /// most resolved colours are `Rgba` (16 bytes), and forcing the enum
    /// to allocate per-call for the dominant case just because the
    /// per-channel variant carries a `SmallVec<[(InkName, f32); 8]>` would
    /// hurt the hot path with no benefit.
    PerChannel {
        channels: Box<SmallVec<[(InkName, f32); 8]>>,
        a: f32,
    },
}

/// Whether the per-plate router should walk participating channels
/// normally, paint every plate at a single tint, or skip every plate.
///
/// Lives on [`OverprintPlan`] rather than on [`ResolvedColor`] so the
/// composite (RGB) backend continues to consume the tint-transform-
/// evaluated RGBA for `/All` Separation sources unchanged. The selector
/// only affects the [`super::InkRouter`] decision; composite backends
/// ignore it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum InkSelector {
    /// Default: route per-plate by walking the participating set
    /// against the target ink. This is the §11.7.4 per-channel
    /// behaviour every non-reserved Separation / DeviceN source takes.
    #[default]
    Listed,
    /// ISO 32000-1:2008 §8.6.6.3 `/All` Separation source: paint every
    /// plate (process + spot) at the same single tint value, carried
    /// alongside in [`OverprintPlan::all_tint`]. The participating
    /// list is empty for these sources.
    All,
    /// ISO 32000-1:2008 §8.6.6.3 `/None` Separation source: skip every
    /// plate; produce no visible output. The participating list is
    /// empty for these sources.
    None,
}

/// Per-channel overprint participation, as evaluated against the current
/// graphics state per ISO 32000-1:2008 §11.7.4.
#[derive(Debug, Clone)]
pub(crate) struct OverprintPlan {
    /// Whether overprint is in effect at all (`/OP` for stroke, `/op` for
    /// fill). When `false`, this plan is a no-op marker and backends paint
    /// every channel unconditionally.
    pub(crate) enabled: bool,
    /// `/OPM` value (0 = standard, 1 = Adobe nonzero). Meaningful only for
    /// DeviceCMYK sources per §11.7.4.3.
    pub(crate) mode: u8,
    /// Channels the source colour space declared. Backends use this to
    /// decide which plates to touch when overprint is enabled: channels in
    /// this list participate; channels outside it are left untouched.
    pub(crate) participating: SmallVec<[ParticipatingChannel; 8]>,
    /// Per-plate routing selector (§8.6.6.3 reserved Separation colorant
    /// names). The composite backend ignores this field; only the
    /// per-plate [`super::InkRouter`] consumes it.
    pub(crate) selector: InkSelector,
    /// Tint to use when [`Self::selector`] is [`InkSelector::All`]. Carried
    /// alongside the colour-resolution output because /All bypasses
    /// alternate-space tint-transform evaluation for the per-plate path —
    /// the operator's raw component value is what every plate receives.
    pub(crate) all_tint: f32,
    /// Per ISO 32000-1 §8.6.6.3 conformance: a Separation source whose
    /// colorant name *is in the device's plate set* paints that plate
    /// directly; otherwise the alternate colour space + tint transform
    /// are used to approximate the colorant. The pipeline composer
    /// records the spot-source identity here so the per-plate backend
    /// can decide per-surface whether to use the direct routing
    /// ([`Self::participating`] interpreted as `[(spot, tint)]`) or
    /// the alt-CMYK fallback ([`Self::alt_cmyk_fallback`]).
    pub(crate) spot_source: Option<SpotSource>,
    /// Alternate-CMYK decomposition the resolver evaluated for a
    /// Separation / DeviceN source. Backends consult this when the
    /// device lacks the source's named colorant — per §8.6.6.3 the alt
    /// is composite-only on conforming devices, but per-plate devices
    /// that don't have the spot plate fall through to the alt-CMYK
    /// channels.
    pub(crate) alt_cmyk_fallback: Option<[f32; 4]>,
}

/// Identity of the source Separation colorant the pipeline composer
/// recorded for the per-plate fallback decision.
#[derive(Debug, Clone)]
pub(crate) struct SpotSource {
    /// Source colorant name (e.g. `"Pantone-185"`, `"MagentaSpot"`).
    pub(crate) ink: InkName,
    /// Operator tint for the source — `components[0]` from the `scn`
    /// operator.
    pub(crate) tint: f32,
}

/// One element of an [`OverprintPlan::participating`] list. The component
/// value carried through lets the per-plate ink router decide whether to
/// paint or skip under OPM=1 ("zero component on DeviceCMYK = colorant not
/// specified" → skip the matching plate).
#[derive(Debug, Clone)]
pub(crate) struct ParticipatingChannel {
    pub(crate) ink: InkName,
    pub(crate) value: f32,
}

/// Blend-mode plan. The pipeline classifies the requested blend mode into one
/// of two paths so the backend never repeats the classification.
#[derive(Debug, Clone, Copy)]
pub(crate) enum BlendPlan {
    /// tiny-skia supports this blend mode natively — backends pass it
    /// straight through into `tiny_skia::Paint::blend_mode`.
    Native(tiny_skia::BlendMode),
    /// The mode requires manual compositing the backend must run after
    /// painting (e.g., a separation backend implementing `Multiply`
    /// per-plate). The marker carries the mode name verbatim so the
    /// backend can dispatch without re-parsing.
    ///
    /// This branch is never emitted by the in-tree pipeline today — all PDF
    /// blend modes the composite backend handles map to a `tiny_skia::BlendMode`.
    /// The variant exists so future backends can opt into simulation
    /// without changing the resolver surface.
    Simulated(&'static str),
}

/// Clip plan. The pipeline composes the active clip stack at evaluation time
/// rather than at paint time, so the backend just consumes the result.
#[derive(Debug, Clone)]
pub(crate) enum ClipPlan {
    None,
    /// Composed clip mask. `Arc` so a single composition can be shared
    /// across the fill and stroke intents of a `B`/`b` operator without
    /// reallocation.
    Mask(Arc<tiny_skia::Mask>),
}

/// Backend-ready paint command. Every dimension is fully evaluated; the
/// backend's job is purely "translate this into a tiny-skia draw call" (or
/// per-plate, or future).
pub(crate) struct ResolvedPaintCmd<'a> {
    pub(crate) kind: PaintKind<'a>,
    pub(crate) side: PaintSide,
    pub(crate) color: ResolvedColor,
    pub(crate) overprint: OverprintPlan,
    pub(crate) blend: BlendPlan,
    pub(crate) clip: ClipPlan,
    pub(crate) ctm: Matrix,
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn ink_name_round_trip() {
        let i = InkName::new("PANTONE 185 C");
        assert_eq!(i.as_str(), "PANTONE 185 C");
    }

    #[test]
    fn ink_name_equality_is_string_equality() {
        // Spot ink routing depends on case-sensitive name matching per
        // ISO 32000-1 §8.6.6.4; the canonical inks ("Cyan", "Magenta",
        // …) are compared as their exact names.
        assert_eq!(InkName::new("Cyan"), InkName::new("Cyan"));
        assert_ne!(InkName::new("Cyan"), InkName::new("cyan"));
    }

    #[test]
    fn overprint_plan_disabled_is_no_op_marker() {
        // When OP/op is false, the plan is a marker only — backends short
        // circuit and paint every channel.
        let plan = OverprintPlan {
            enabled: false,
            mode: 0,
            participating: SmallVec::new(),
            selector: InkSelector::Listed,
            all_tint: 0.0,
            spot_source: None,
            alt_cmyk_fallback: None,
        };
        assert!(!plan.enabled);
    }

    #[test]
    fn overprint_plan_participating_inline_capacity() {
        // 4-process DeviceCMYK stays inline.
        let plan = OverprintPlan {
            enabled: true,
            mode: 0,
            participating: smallvec![
                ParticipatingChannel {
                    ink: InkName::new("Cyan"),
                    value: 0.5
                },
                ParticipatingChannel {
                    ink: InkName::new("Magenta"),
                    value: 0.0
                },
                ParticipatingChannel {
                    ink: InkName::new("Yellow"),
                    value: 0.3
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
        };
        assert_eq!(plan.participating.len(), 4);
        assert!(!plan.participating.spilled());
    }

    #[test]
    fn resolved_color_rgba_includes_alpha() {
        let c = ResolvedColor::Rgba {
            r: 1.0,
            g: 0.5,
            b: 0.25,
            a: 0.75,
        };
        match c {
            ResolvedColor::Rgba { a, .. } => assert!((a - 0.75).abs() < 1e-6),
            _ => panic!("expected Rgba"),
        }
    }

    #[test]
    fn blend_plan_native_carries_skia_mode() {
        let p = BlendPlan::Native(tiny_skia::BlendMode::Multiply);
        match p {
            BlendPlan::Native(m) => assert_eq!(m, tiny_skia::BlendMode::Multiply),
            BlendPlan::Simulated(_) => panic!("expected Native"),
        }
    }

    #[test]
    fn clip_plan_mask_is_arc_shared() {
        let mask = Arc::new(tiny_skia::Mask::new(4, 4).expect("4x4 mask allocates"));
        let plan_a = ClipPlan::Mask(mask.clone());
        let plan_b = ClipPlan::Mask(mask.clone());
        // The point of using Arc is that the fill and stroke sides of a
        // single B/b operator share one composed mask without copying.
        match (&plan_a, &plan_b) {
            (ClipPlan::Mask(a), ClipPlan::Mask(b)) => assert!(Arc::ptr_eq(a, b)),
            _ => panic!("both should be Mask"),
        }
    }
}
