//! Issue #4969 W10-Q2-D1: qualified shaping survives the composition handoff by Arc.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_composition.rs"]
mod shaping_composition;
#[path = "../../src/renderer/shaping_context.rs"]
mod shaping_context;
#[path = "../../src/renderer/shaping_paragraph.rs"]
mod shaping_paragraph;
#[path = "../../src/renderer/shaping_publication.rs"]
mod shaping_publication;

// Product symbols stay crate-private. This source integration case includes
// kerning.rs directly, so mirror only the paint surface that module consumes.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
}

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry, KerningMeasurementContext};
use shaping_composition::retain_qualified_horizontal_shaping_outcome;
use shaping_context::HorizontalShapingContext;
use shaping_paragraph::{
    is_bounded_horizontal_shaping_candidate_text, run_horizontal_shaping_line_transaction,
    HorizontalShapingFallbackOwner, HorizontalShapingLineDisposition, HorizontalShapingLineRequest,
    HorizontalShapingParagraphRequest, HorizontalShapingParagraphScalarStyle,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};

fn registry() -> ExactFontSourceRegistry {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register exact old-Hangul source");
    registry
}

fn qualified_outcome() -> Arc<shaping_paragraph::HorizontalShapingLineOutcome> {
    let context = HorizontalShapingContext::new(registry());
    let mut transaction = context.transaction();
    let text = "ᄒᆞᆫ글";
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![
        HorizontalShapingParagraphScalarStyle {
            slot: SLOT,
            effective_font_size_px: 10.0,
            width_ratio: 0.8,
            letter_spacing_px: 0.0,
            kerning: true,
            bold: false,
            italic: false,
            superscript: false,
            subscript: false,
        };
        4
    ];
    let hard_boundaries = [false; 5];
    let candidate_boundaries = [0, 4];
    let available_widths_px = [100.0];
    Arc::new(run_horizontal_shaping_line_transaction(
        &mut transaction,
        &HorizontalShapingLineRequest {
            paragraph: HorizontalShapingParagraphRequest {
                attempt_id_base: 1,
                text,
                fallback_positions: &positions,
                scalar_styles: &styles,
                hard_boundaries: &hard_boundaries,
                fallback_owner: HorizontalShapingFallbackOwner::W9K1,
                model_text_matches_shaping_text: true,
                horizontal_ltr_bidi0: true,
                condense_min_space: 0,
                has_inline_controls: false,
                has_tabs: false,
                has_rotation: false,
                has_char_overlap: false,
            },
            candidate_boundaries: &candidate_boundaries,
            available_widths_px: &available_widths_px,
        },
    ))
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d1_composition_owner_retains_the_exact_q2_c_arc() {
    let outcome = qualified_outcome();
    assert_eq!(
        outcome.trace.disposition,
        HorizontalShapingLineDisposition::DormantQualified
    );
    assert!(!outcome.trace.product_published);
    let final_target = Arc::clone(&outcome.lines[0].target_runs[0].measurement);

    let retained = retain_qualified_horizontal_shaping_outcome(Arc::clone(&outcome))
        .expect("qualified outcome is retained");
    assert!(Arc::ptr_eq(&retained, &outcome));
    assert!(Arc::ptr_eq(
        &retained.lines[0].target_runs[0].measurement,
        &final_target
    ));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d1_nonqualified_outcome_is_not_attached() {
    let mut outcome = Arc::try_unwrap(qualified_outcome()).expect("single outcome owner");
    outcome.trace.disposition = HorizontalShapingLineDisposition::RolledBack;
    assert!(retain_qualified_horizontal_shaping_outcome(Arc::new(outcome)).is_none());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d1_paired_contexts_share_one_registry_generation() {
    let registry = registry();
    let kerning = KerningMeasurementContext::new(registry.clone());
    let shaping = HorizontalShapingContext::new(registry);

    assert_eq!(kerning.registry_generation(), shaping.registry_generation());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d1_candidate_gate_is_bounded_before_projection() {
    assert!(is_bounded_horizontal_shaping_candidate_text("ᄒᆞᆫ글"));
    assert!(!is_bounded_horizontal_shaping_candidate_text("가변"));
    assert!(!is_bounded_horizontal_shaping_candidate_text("Typography"));
    assert!(!is_bounded_horizontal_shaping_candidate_text(
        "ordinary text"
    ));

    let oversized = format!("{}ᄒ", "a".repeat(shaping::MAX_SHAPING_TEXT_CODE_POINTS));
    assert!(!is_bounded_horizontal_shaping_candidate_text(&oversized));
}
