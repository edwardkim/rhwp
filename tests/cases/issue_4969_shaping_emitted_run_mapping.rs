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

// Product symbols stay crate-private. The source integration case includes
// kerning.rs directly, so mirror the narrow paint surface used by that module.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
}

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use shaping_composition::{
    attach_horizontal_shaping_mapped_run, map_horizontal_shaping_emitted_run,
    prepare_horizontal_shaping_no_lineseg_owner_transaction, project_horizontal_shaping_run_range,
    HorizontalShapingEmittedRunCandidate, HorizontalShapingEmittedRunRejectReason,
    HorizontalShapingLegacyGeometry, HorizontalShapingNoLineSegOwnerRejectReason,
    HorizontalShapingNoLineSegSurface,
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

fn qualified_context_and_outcome() -> (
    HorizontalShapingContext,
    Arc<shaping_paragraph::HorizontalShapingLineOutcome>,
) {
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
    let outcome = Arc::new(run_horizontal_shaping_line_transaction(
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
    ));
    drop(transaction);
    (context, outcome)
}

fn qualified_outcome() -> Arc<shaping_paragraph::HorizontalShapingLineOutcome> {
    qualified_context_and_outcome().1
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

fn no_lineseg_surface() -> HorizontalShapingNoLineSegSurface {
    HorizontalShapingNoLineSegSurface {
        model_line_seg_count: 0,
        frame_interval_count: 1,
        edit_reflow: false,
        stored_prefix: false,
        split_cell: false,
        has_inline_control: false,
    }
}

fn legacy_geometry() -> HorizontalShapingLegacyGeometry {
    HorizontalShapingLegacyGeometry {
        line_width_px: 16.0,
        bbox_width_px: 16.0,
        next_origin_x_px: 39.5,
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d5_n0_no_lineseg_owner_keeps_four_consumers_on_one_arc() {
    let (context, outcome) = qualified_context_and_outcome();
    let target_measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let transaction = prepare_horizontal_shaping_no_lineseg_owner_transaction(
        &context,
        Arc::clone(&outcome),
        no_lineseg_surface(),
        candidate(TEXT, TEXT),
        legacy_geometry(),
    )
    .expect("ordinary no-LineSeg owner transaction");

    assert!(Arc::ptr_eq(transaction.outcome(), &outcome));
    assert!(Arc::ptr_eq(
        transaction.line_selection_measurement(),
        &target_measurement
    ));
    assert!(Arc::ptr_eq(
        transaction.line_selection_measurement(),
        transaction.bbox_measurement()
    ));
    assert!(Arc::ptr_eq(
        transaction.line_selection_measurement(),
        transaction.next_origin_measurement()
    ));
    assert!(Arc::ptr_eq(
        transaction.line_selection_measurement(),
        transaction.sidecar_measurement()
    ));
    assert_eq!(
        transaction.line_width_px(),
        target_measurement.total_advance_px
    );
    assert_eq!(
        transaction.bbox_width_px(),
        target_measurement.total_advance_px
    );
    assert!(
        ((transaction.next_origin_x_px() - 23.5) - target_measurement.total_advance_px).abs()
            <= 1.0e-12
    );
    assert_eq!(transaction.fallback_geometry(), legacy_geometry());
    assert!(!transaction.product_published());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d5_n0_feature_detection_rejects_unsupported_surfaces_by_type() {
    let cases = [
        (
            HorizontalShapingNoLineSegSurface {
                model_line_seg_count: 1,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::ModelLineSegPresent,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                frame_interval_count: 0,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::FrameIntervalCountUnsupported,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                frame_interval_count: 2,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::FrameIntervalCountUnsupported,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                edit_reflow: true,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::EditReflowUnsupported,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                stored_prefix: true,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::StoredPrefixUnsupported,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                split_cell: true,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::SplitCellUnsupported,
        ),
        (
            HorizontalShapingNoLineSegSurface {
                has_inline_control: true,
                ..no_lineseg_surface()
            },
            HorizontalShapingNoLineSegOwnerRejectReason::InlineControlUnsupported,
        ),
    ];

    for (surface, expected) in cases {
        let (context, outcome) = qualified_context_and_outcome();
        let rejection = prepare_horizontal_shaping_no_lineseg_owner_transaction(
            &context,
            outcome,
            surface,
            candidate(TEXT, TEXT),
            legacy_geometry(),
        )
        .unwrap_err();
        assert_eq!(rejection.reason(), expected);
        assert_eq!(rejection.fallback_geometry(), legacy_geometry());
        assert!(!rejection.product_published());
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d5_n0_late_source_failure_rolls_back_all_consumers() {
    let (_, outcome) = qualified_context_and_outcome();
    let stale_context = HorizontalShapingContext::new(ExactFontSourceRegistry::default());
    let rejection = prepare_horizontal_shaping_no_lineseg_owner_transaction(
        &stale_context,
        outcome,
        no_lineseg_surface(),
        candidate(TEXT, TEXT),
        legacy_geometry(),
    )
    .unwrap_err();

    assert_eq!(
        rejection.reason(),
        HorizontalShapingNoLineSegOwnerRejectReason::ReplaySourceRejected(
            shaping_context::HorizontalShapingReplaySourceCertificateRejectReason::StaleRegistryGeneration
        )
    );
    assert_eq!(rejection.fallback_geometry(), legacy_geometry());
    assert!(!rejection.product_published());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d5_n0_multiple_targets_are_rejected_before_publication() {
    let (context, outcome) = qualified_context_and_outcome();
    let mut multiple = (*outcome).clone();
    let duplicate = multiple.lines[0].target_runs[0].clone();
    multiple.lines[0].target_runs.push(duplicate);
    let rejection = prepare_horizontal_shaping_no_lineseg_owner_transaction(
        &context,
        Arc::new(multiple),
        no_lineseg_surface(),
        candidate(TEXT, TEXT),
        legacy_geometry(),
    )
    .unwrap_err();

    assert_eq!(
        rejection.reason(),
        HorizontalShapingNoLineSegOwnerRejectReason::TargetCountUnsupported
    );
    assert_eq!(rejection.fallback_geometry(), legacy_geometry());
    assert!(!rejection.product_published());
}
