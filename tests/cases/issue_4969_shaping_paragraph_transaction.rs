//! Issue #4969 W10-Q2-C: cluster-aware paragraph/line transaction is bounded and dormant.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_context.rs"]
mod shaping_context;
#[path = "../../src/renderer/shaping_paragraph.rs"]
mod shaping_paragraph;

// Product symbols stay crate-private. This source integration case includes
// kerning.rs directly, so mirror only the paint surface that module consumes.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
}

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use shaping::TerminalShapingDisposition;
use shaping_context::HorizontalShapingContext;
use shaping_paragraph::{
    decide_horizontal_shaping_activation, prepare_horizontal_shaping_paragraph,
    run_horizontal_shaping_line_transaction, HorizontalShapingActivationDisposition,
    HorizontalShapingActivationReason, HorizontalShapingFallbackOwner,
    HorizontalShapingLineDisposition, HorizontalShapingLineRequest,
    HorizontalShapingParagraphRequest, HorizontalShapingParagraphScalarStyle,
    MAX_HORIZONTAL_SHAPING_LINE_ATTEMPTS, MAX_HORIZONTAL_SHAPING_LINE_CANDIDATES,
    MAX_HORIZONTAL_SHAPING_LINE_RETRIES, MAX_HORIZONTAL_SHAPING_PARAGRAPH_SEGMENTS,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");

const SOURCE_HAN_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};

fn registry() -> ExactFontSourceRegistry {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            SOURCE_HAN_SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register Source Han exact source");
    registry
}

fn style(slot: ExactFontSlot) -> HorizontalShapingParagraphScalarStyle {
    HorizontalShapingParagraphScalarStyle {
        slot,
        effective_font_size_px: 10.0,
        width_ratio: 0.8,
        letter_spacing_px: 0.0,
        kerning: true,
        bold: false,
        italic: false,
        superscript: false,
        subscript: false,
    }
}

fn request<'a>(
    attempt_id_base: u32,
    text: &'a str,
    fallback_positions: &'a [f64],
    scalar_styles: &'a [HorizontalShapingParagraphScalarStyle],
    hard_boundaries: &'a [bool],
) -> HorizontalShapingParagraphRequest<'a> {
    HorizontalShapingParagraphRequest {
        attempt_id_base,
        text,
        fallback_positions,
        scalar_styles,
        hard_boundaries,
        fallback_owner: HorizontalShapingFallbackOwner::W9K1,
        model_text_matches_shaping_text: true,
        horizontal_ltr_bidi0: true,
        condense_min_space: 0,
        has_inline_controls: false,
        has_tabs: false,
        has_rotation: false,
        has_char_overlap: false,
    }
}

fn assert_reason(
    request: &HorizontalShapingParagraphRequest<'_>,
    disposition: HorizontalShapingActivationDisposition,
    reason: HorizontalShapingActivationReason,
) {
    let decision = decide_horizontal_shaping_activation(request);
    assert_eq!(decision.disposition, disposition);
    assert_eq!(decision.reason, Some(reason));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c0_activation_is_feature_detected_and_fail_closed() {
    assert_eq!(MAX_HORIZONTAL_SHAPING_PARAGRAPH_SEGMENTS, 4_096);
    assert_eq!(MAX_HORIZONTAL_SHAPING_LINE_CANDIDATES, 4_097);
    assert_eq!(MAX_HORIZONTAL_SHAPING_LINE_ATTEMPTS, 4_096);
    assert_eq!(MAX_HORIZONTAL_SHAPING_LINE_RETRIES, 2);

    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];
    let base = request(1, text, &positions, &styles, &hard);
    let decision = decide_horizontal_shaping_activation(&base);
    assert!(decision.is_eligible());
    assert_eq!(decision.target_scalar_count, 4);
    assert_eq!(decision.target_ranges[0].scalar_start, 0);
    assert_eq!(decision.target_ranges[0].scalar_end, 4);

    let latin_positions = [0.0, 4.0];
    let latin_styles = [style(SOURCE_HAN_SLOT)];
    let latin_hard = [false; 2];
    assert_reason(
        &request(2, "A", &latin_positions, &latin_styles, &latin_hard),
        HorizontalShapingActivationDisposition::NotTarget,
        HorizontalShapingActivationReason::NoComplexRequiredText,
    );

    let mut variant = base.clone();
    variant.horizontal_ltr_bidi0 = false;
    assert_reason(
        &variant,
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::BidiAuthorityPending,
    );
    variant = base.clone();
    variant.model_text_matches_shaping_text = false;
    assert_reason(
        &variant,
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::DisplayProjectionNotSupported,
    );
    variant = base.clone();
    variant.condense_min_space = 1;
    assert_reason(
        &variant,
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::CondenseSemanticsPending,
    );
    for (set_flag, reason) in [
        (
            0,
            HorizontalShapingActivationReason::InlineControlNotSupported,
        ),
        (1, HorizontalShapingActivationReason::TabNotSupported),
        (2, HorizontalShapingActivationReason::RotationNotSupported),
        (
            3,
            HorizontalShapingActivationReason::CharOverlapNotSupported,
        ),
    ] {
        variant = base.clone();
        match set_flag {
            0 => variant.has_inline_controls = true,
            1 => variant.has_tabs = true,
            2 => variant.has_rotation = true,
            _ => variant.has_char_overlap = true,
        }
        assert_reason(
            &variant,
            HorizontalShapingActivationDisposition::Unsupported,
            reason,
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c0_style_and_paragraph_boundaries_cannot_be_crossed() {
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let base_styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];

    let mut styled = base_styles.clone();
    styled[1].kerning = false;
    assert_reason(
        &request(10, text, &positions, &styled, &hard),
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::StyleBoundaryCrossed,
    );

    let hard_inside = [false, true, false, false, false];
    assert_reason(
        &request(11, text, &positions, &base_styles, &hard_inside),
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::HardBoundaryCrossed,
    );

    let mut bold = base_styles.clone();
    bold.iter_mut().for_each(|style| style.bold = true);
    assert_reason(
        &request(12, text, &positions, &bold, &hard),
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::SyntheticStyleNotSupported,
    );

    let mut spaced = base_styles.clone();
    spaced
        .iter_mut()
        .for_each(|style| style.letter_spacing_px = 0.25);
    assert_reason(
        &request(13, text, &positions, &spaced, &hard),
        HorizontalShapingActivationDisposition::Unsupported,
        HorizontalShapingActivationReason::LetterSpacingSemanticsPending,
    );

    let malformed_positions = [0.0, 4.0, 3.0, 12.0, 16.0];
    assert_reason(
        &request(14, text, &malformed_positions, &base_styles, &hard),
        HorizontalShapingActivationDisposition::Malformed,
        HorizontalShapingActivationReason::FallbackPositionNonMonotonic,
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c1_paragraph_width_has_one_cluster_owner() {
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];
    let context = HorizontalShapingContext::new(registry());
    let outcome = prepare_horizontal_shaping_paragraph(
        &mut context.transaction(),
        &request(20, text, &positions, &styles, &hard),
    );
    let paragraph = outcome.measurement.expect("qualified paragraph shadow");

    assert_eq!(outcome.attempts.len(), 1);
    assert_eq!(paragraph.targets.len(), 1);
    assert_eq!(
        paragraph.fallback_owner,
        HorizontalShapingFallbackOwner::W9K1
    );
    assert!((paragraph.fallback_total_width_px - 16.0).abs() < 1.0e-9);
    assert!((paragraph.total_width_px - 15.456).abs() < 1.0e-9);
    assert!((paragraph.range_width(0, 3).expect("old Jamo cluster") - 7.728).abs() < 1.0e-9);
    assert!(paragraph.range_width(1, 3).is_none());
    assert!((paragraph.range_width(3, 4).expect("modern syllable") - 7.728).abs() < 1.0e-9);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c1_mixed_text_keeps_fallback_width_outside_target() {
    let text = "Aᄒᆞᆫ글B";
    let positions = [0.0, 5.0, 9.0, 13.0, 17.0, 21.0, 26.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 6];
    let hard = [false; 7];
    let context = HorizontalShapingContext::new(registry());
    let outcome = prepare_horizontal_shaping_paragraph(
        &mut context.transaction(),
        &request(30, text, &positions, &styles, &hard),
    );
    let paragraph = outcome.measurement.expect("mixed paragraph shadow");

    assert_eq!(paragraph.activation.target_ranges[0].scalar_start, 1);
    assert_eq!(paragraph.activation.target_ranges[0].scalar_end, 5);
    assert!((paragraph.range_width(0, 1).expect("prefix fallback") - 5.0).abs() < 1.0e-9);
    assert!((paragraph.range_width(1, 5).expect("shaped target") - 15.456).abs() < 1.0e-9);
    assert!((paragraph.range_width(5, 6).expect("suffix fallback") - 5.0).abs() < 1.0e-9);
    assert!((paragraph.total_width_px - 25.456).abs() < 1.0e-9);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c2_filters_fake_breaks_and_keeps_final_runs_dormant() {
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];
    let paragraph = request(40, text, &positions, &styles, &hard);
    let context = HorizontalShapingContext::new(registry());
    let outcome = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph,
            candidate_boundaries: &[0, 1, 2, 3, 4],
            available_widths_px: &[10.0],
        },
    );

    assert_eq!(
        outcome.trace.disposition,
        HorizontalShapingLineDisposition::DormantQualified
    );
    assert_eq!(
        outcome.trace.reason,
        Some(HorizontalShapingActivationReason::PublicationOwnerPending)
    );
    assert_eq!(outcome.trace.retry_count, 0);
    assert!(outcome.trace.retry_count <= MAX_HORIZONTAL_SHAPING_LINE_RETRIES);
    assert!(!outcome.trace.product_published);
    assert_eq!(outcome.lines.len(), 2);
    assert_eq!(outcome.lines[0].scalar_start, 0);
    assert_eq!(outcome.lines[0].scalar_end, 3);
    assert_eq!(outcome.lines[1].scalar_start, 3);
    assert_eq!(outcome.lines[1].scalar_end, 4);
    assert_eq!(outcome.lines[0].target_runs.len(), 1);
    assert_eq!(outcome.lines[1].target_runs.len(), 1);
    assert_eq!(outcome.lines[0].target_runs[0].scalar_start, 0);
    assert_eq!(outcome.lines[0].target_runs[0].scalar_end, 3);
    assert!((outcome.lines[0].target_runs[0].measurement.total_advance_px - 7.728).abs() < 1.0e-9);
    assert!((outcome.lines[0].width_px - 7.728).abs() < 1.0e-9);
    assert!((outcome.lines[1].width_px - 7.728).abs() < 1.0e-9);
    assert_eq!(outcome.trace.attempt_count, outcome.attempts.len());
    assert!(outcome.paragraph_measurement.is_some());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c2_rolls_back_to_original_owner_on_rejection() {
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];
    let mut paragraph = request(50, text, &positions, &styles, &hard);
    paragraph.fallback_owner = HorizontalShapingFallbackOwner::ExistingK0;
    let context = HorizontalShapingContext::new(ExactFontSourceRegistry::default());
    let outcome = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph,
            candidate_boundaries: &[0, 1, 2, 3, 4],
            available_widths_px: &[10.0],
        },
    );

    assert_eq!(
        outcome.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        outcome.trace.reason,
        Some(HorizontalShapingActivationReason::ExactSourceUnavailable)
    );
    assert_eq!(
        outcome.trace.fallback_owner,
        HorizontalShapingFallbackOwner::ExistingK0
    );
    assert!(!outcome.trace.product_published);
    assert_eq!(outcome.attempts.len(), 1);
    assert_eq!(
        outcome.attempts[0].disposition,
        TerminalShapingDisposition::Unsupported
    );
    assert!(outcome.lines.iter().all(|line| line.target_runs.is_empty()));
    assert_eq!(outcome.lines[0].scalar_start, 0);
    assert_eq!(outcome.lines[0].scalar_end, 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c2_discards_earlier_success_when_a_later_segment_fails() {
    let text = "ᄒᆞᆫAᄒᆞᆫ";
    let positions = [0.0, 4.0, 8.0, 12.0, 17.0, 21.0, 25.0, 29.0];
    let missing_slot = ExactFontSlot {
        char_shape_id: 4969,
        language_index: 7,
    };
    let mut styles = vec![style(SOURCE_HAN_SLOT); 7];
    styles[4..].fill(style(missing_slot));
    let hard = [false; 8];
    let context = HorizontalShapingContext::new(registry());
    let outcome = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph: request(55, text, &positions, &styles, &hard),
            candidate_boundaries: &[0, 3, 4, 7],
            available_widths_px: &[20.0],
        },
    );

    assert_eq!(
        outcome.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        outcome.trace.reason,
        Some(HorizontalShapingActivationReason::ExactSourceUnavailable)
    );
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(
        outcome.attempts[0].disposition,
        TerminalShapingDisposition::Applied
    );
    assert_eq!(
        outcome.attempts[1].disposition,
        TerminalShapingDisposition::Unsupported
    );
    assert!(outcome.paragraph_measurement.is_none());
    assert!(outcome.lines.iter().all(|line| line.target_runs.is_empty()));
    assert!(!outcome.trace.product_published);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_c2_limits_are_deterministic_and_do_not_publish() {
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![style(SOURCE_HAN_SLOT); 4];
    let hard = [false; 5];
    let context = HorizontalShapingContext::new(registry());

    let too_many_candidates = (0..=MAX_HORIZONTAL_SHAPING_LINE_CANDIDATES).collect::<Vec<_>>();
    let outcome = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph: request(60, text, &positions, &styles, &hard),
            candidate_boundaries: &too_many_candidates,
            available_widths_px: &[10.0],
        },
    );
    assert_eq!(
        outcome.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        outcome.trace.reason,
        Some(HorizontalShapingActivationReason::CandidateLimitExceeded)
    );
    assert!(!outcome.trace.product_published);

    let malformed_candidates = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph: request(65, text, &positions, &styles, &hard),
            candidate_boundaries: &[0, 3, 2, 4],
            available_widths_px: &[10.0],
        },
    );
    assert_eq!(
        malformed_candidates.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        malformed_candidates.trace.reason,
        Some(HorizontalShapingActivationReason::CandidateInputMalformed)
    );
    assert!(!malformed_candidates.trace.product_published);

    let attempt_overflow = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph: request(u32::MAX, text, &positions, &styles, &hard),
            candidate_boundaries: &[0, 3, 4],
            available_widths_px: &[10.0],
        },
    );
    assert_eq!(
        attempt_overflow.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        attempt_overflow.trace.reason,
        Some(HorizontalShapingActivationReason::AttemptLimitExceeded)
    );
    assert_eq!(attempt_overflow.attempts.len(), 1);
    assert_eq!(attempt_overflow.attempts[0].attempt_id, u32::MAX);
    assert!(!attempt_overflow.trace.product_published);

    let malformed_width = run_horizontal_shaping_line_transaction(
        &mut context.transaction(),
        &HorizontalShapingLineRequest {
            paragraph: request(70, text, &positions, &styles, &hard),
            candidate_boundaries: &[0, 3, 4],
            available_widths_px: &[f64::NAN],
        },
    );
    assert_eq!(
        malformed_width.trace.disposition,
        HorizontalShapingLineDisposition::RolledBack
    );
    assert_eq!(
        malformed_width.trace.reason,
        Some(HorizontalShapingActivationReason::AvailableWidthMalformed)
    );
    assert!(malformed_width.attempts.is_empty());
    assert!(!malformed_width.trace.product_published);
}
