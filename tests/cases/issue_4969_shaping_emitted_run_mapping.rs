//! Issue #4969 W10-Q2-D2: final emitted-run mapping stays dormant and fail-closed.

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

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use shaping_composition::{
    attach_horizontal_shaping_mapped_run, map_horizontal_shaping_emitted_run,
    project_horizontal_shaping_run_range, HorizontalShapingEmittedRunCandidate,
    HorizontalShapingEmittedRunRejectReason,
};
use shaping_context::HorizontalShapingContext;
use shaping_paragraph::{
    run_horizontal_shaping_line_transaction, HorizontalShapingFallbackOwner,
    HorizontalShapingLineRequest, HorizontalShapingParagraphRequest,
    HorizontalShapingParagraphScalarStyle,
};
use shaping_publication::HorizontalShapingPageSidecars;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};
const TEXT: &str = "ᄒᆞᆫ글";

fn qualified_outcome() -> Arc<shaping_paragraph::HorizontalShapingLineOutcome> {
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
    let context = HorizontalShapingContext::new(registry);
    let mut transaction = context.transaction();
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
                text: TEXT,
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

fn candidate<'a>(
    paragraph_text: &'a str,
    emitted_text: &'a str,
) -> HorizontalShapingEmittedRunCandidate<'a> {
    HorizontalShapingEmittedRunCandidate {
        node_id: 17,
        paragraph_text,
        emitted_text,
        scalar_start: 0,
        origin_x_px: 23.5,
        layout_positions_present: false,
        display_projection_present: false,
        horizontal_ltr_bidi0: true,
        has_field_or_note_split: false,
        has_char_overlap: false,
        has_border_or_background: false,
        has_decoration: false,
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d2_maps_three_coordinates_and_one_geometry_owner() {
    let outcome = qualified_outcome();
    let measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let mapped =
        map_horizontal_shaping_emitted_run(&outcome, candidate(TEXT, TEXT)).expect("exact mapping");

    assert_eq!(mapped.range.scalar_start, 0);
    assert_eq!(mapped.range.scalar_end, 4);
    assert_eq!(mapped.range.utf8_start, 0);
    assert_eq!(mapped.range.utf8_end, 12);
    assert_eq!(mapped.range.utf16_start, 0);
    assert_eq!(mapped.range.utf16_end, 4);
    assert_eq!(mapped.line_width_px, measurement.total_advance_px);
    assert_eq!(mapped.bbox_width_px, measurement.total_advance_px);
    assert!(((mapped.next_origin_x_px - 23.5) - measurement.total_advance_px).abs() <= 1.0e-12);
    assert!(Arc::ptr_eq(
        mapped.decision.measurement().expect("mapped measurement"),
        &measurement
    ));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d2_attaches_the_same_node_and_measurement_arc() {
    let outcome = qualified_outcome();
    let measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let mapped =
        map_horizontal_shaping_emitted_run(&outcome, candidate(TEXT, TEXT)).expect("exact mapping");
    let decision = Arc::clone(&mapped.decision);
    let mut sidecars = HorizontalShapingPageSidecars::default();

    attach_horizontal_shaping_mapped_run(&mut sidecars, &mapped).expect("atomic attach");
    assert!(Arc::ptr_eq(
        sidecars.get(17).expect("same NodeId"),
        &decision
    ));
    assert!(Arc::ptr_eq(
        sidecars
            .get(17)
            .and_then(|owned| owned.measurement())
            .expect("same measurement"),
        &measurement
    ));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d2_rejects_partial_or_conflicting_emitted_surfaces() {
    let outcome = qualified_outcome();
    let mut partial = candidate(TEXT, TEXT);
    partial.emitted_text = "ᄒᆞᆫ";
    assert_eq!(
        map_horizontal_shaping_emitted_run(&outcome, partial).unwrap_err(),
        HorizontalShapingEmittedRunRejectReason::ExactTargetNotFound
    );

    let mut w9_conflict = candidate(TEXT, TEXT);
    w9_conflict.layout_positions_present = true;
    assert_eq!(
        map_horizontal_shaping_emitted_run(&outcome, w9_conflict).unwrap_err(),
        HorizontalShapingEmittedRunRejectReason::UnsupportedSurface
    );

    let mut projected = candidate(TEXT, TEXT);
    projected.display_projection_present = true;
    assert_eq!(
        map_horizontal_shaping_emitted_run(&outcome, projected).unwrap_err(),
        HorizontalShapingEmittedRunRejectReason::UnsupportedSurface
    );

    let oversized_text = "ᄒ".repeat(shaping::MAX_SHAPING_TEXT_CODE_POINTS + 1);
    assert_eq!(
        map_horizontal_shaping_emitted_run(
            &outcome,
            candidate(oversized_text.as_str(), oversized_text.as_str()),
        )
        .unwrap_err(),
        HorizontalShapingEmittedRunRejectReason::TextLimitExceeded
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d2_utf8_and_utf16_offsets_are_not_scalar_aliases() {
    let range = project_horizontal_shaping_run_range("😀ᄒᆞᆫ글", 1, 5).expect("bounded projection");
    assert_eq!(range.scalar_start, 1);
    assert_eq!(range.scalar_end, 5);
    assert_eq!(range.utf8_start, 4);
    assert_eq!(range.utf8_end, 16);
    assert_eq!(range.utf16_start, 2);
    assert_eq!(range.utf16_end, 6);
}
