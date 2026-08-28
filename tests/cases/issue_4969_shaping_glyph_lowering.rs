//! Issue #4969 W10-Q2-D3: page sidecars lower losslessly but remain dormant.

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

// The product implementation stays crate-private. This narrow wrapper lets the
// source integration case compile that exact module without widening rhwp's API.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
    pub(crate) const MAX_PORTABLE_GLYPHS_PER_RUN: usize = 4096;
}

mod renderer {
    pub(crate) use crate::shaping_publication;
    pub use rhwp::renderer::render_tree;
    pub use rhwp::renderer::style_resolver;
}

#[path = "../../src/paint/shaping_glyph.rs"]
mod shaping_glyph;

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use rhwp::paint::{EmbeddedFontFace, LayerNode, PaintOp, ResourceArena};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::TextStyle;
use shaping_composition::{
    attach_horizontal_shaping_mapped_run, map_horizontal_shaping_emitted_run,
    HorizontalShapingEmittedRunCandidate,
};
use shaping_context::HorizontalShapingContext;
use shaping_glyph::{
    lower_horizontal_shaping_layer_node_shadow, HorizontalShapingGlyphLoweringRejectReason,
};
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
    Arc::new(run_horizontal_shaping_line_transaction(
        &mut transaction,
        &HorizontalShapingLineRequest {
            paragraph: HorizontalShapingParagraphRequest {
                attempt_id_base: 1,
                text: TEXT,
                fallback_positions: &positions,
                scalar_styles: &styles,
                hard_boundaries: &[false; 5],
                fallback_owner: HorizontalShapingFallbackOwner::W9K1,
                model_text_matches_shaping_text: true,
                horizontal_ltr_bidi0: true,
                condense_min_space: 0,
                has_inline_controls: false,
                has_tabs: false,
                has_rotation: false,
                has_char_overlap: false,
            },
            candidate_boundaries: &[0, 4],
            available_widths_px: &[100.0],
        },
    ))
}

fn prepared_sidecar() -> (
    HorizontalShapingPageSidecars,
    Arc<shaping_context::HorizontalShapingMeasurement>,
) {
    let outcome = qualified_outcome();
    let measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let mapped = map_horizontal_shaping_emitted_run(
        &outcome,
        HorizontalShapingEmittedRunCandidate {
            node_id: 17,
            paragraph_text: TEXT,
            emitted_text: TEXT,
            scalar_start: 0,
            origin_x_px: 0.0,
            layout_positions_present: false,
            display_projection_present: false,
            horizontal_ltr_bidi0: true,
            has_field_or_note_split: false,
            has_char_overlap: false,
            has_border_or_background: false,
            has_decoration: false,
        },
    )
    .expect("exact final-run mapping");
    let mut sidecars = HorizontalShapingPageSidecars::default();
    attach_horizontal_shaping_mapped_run(&mut sidecars, &mapped).expect("attach exact sidecar");
    (sidecars, measurement)
}

fn text_run() -> TextRunNode {
    let mut style = TextStyle::default();
    style.font_family = "Source Han Serif K Old Hangul".to_string();
    style.font_size = 10.0;
    style.ratio = 0.8;
    style.kerning = true;
    TextRunNode {
        text: TEXT.to_string(),
        style,
        char_shape_id: Some(SLOT.char_shape_id),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: Some(0),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 10.0,
        field_marker: FieldMarkerType::None,
        layout_positions: None,
        display_text: None,
    }
}

fn embedded_font(bytes: &[u8]) -> EmbeddedFontFace<'_> {
    EmbeddedFontFace {
        char_shape_id: SLOT.char_shape_id,
        language_index: SLOT.language_index,
        family: "Source Han Serif K Old Hangul",
        alternate_family: None,
        bytes,
        face_index: 0,
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d3_lowers_exact_glyph_geometry_and_clusters() {
    let (sidecars, measurement) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run.clone())]);
    let mut resources = ResourceArena::default();
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        23,
        &sidecars,
        &[embedded_font(SOURCE_HAN)],
        &mut resources,
    );
    let lowered = report.glyph_run.expect("dormant exact glyph run");

    assert_eq!(report.source_node_id, Some(17));
    assert_eq!(report.reject_reason, None);
    assert!(report.claims_glyph_run_slot);
    let expected_ids = [614, 1230, 1497, 2085];
    let expected_x = [0.0, 7.728, 15.456, 15.456];
    let expected_advance_x = [7.728, 7.728, 0.0, 0.0];
    assert_eq!(lowered.glyph_ids, expected_ids);
    assert_eq!(lowered.positions.len(), expected_x.len());
    assert_eq!(
        lowered.advances.as_ref().expect("advances").len(),
        expected_advance_x.len()
    );
    for index in 0..expected_ids.len() {
        assert_eq!(measurement.glyphs_px[index].glyph_id, expected_ids[index]);
        assert_eq!(measurement.glyphs_px[index].x, expected_x[index]);
        assert_eq!(measurement.glyphs_px[index].y, 0.0);
        assert_eq!(
            measurement.glyphs_px[index].advance_x,
            expected_advance_x[index]
        );
        assert_eq!(measurement.glyphs_px[index].advance_y, 0.0);
        assert_eq!(lowered.positions[index].x, expected_x[index]);
        assert_eq!(lowered.positions[index].y, 0.0);
        assert_eq!(
            lowered.advances.as_ref().unwrap()[index].dx,
            expected_advance_x[index]
        );
        assert_eq!(lowered.advances.as_ref().unwrap()[index].dy, 0.0);
    }
    assert_eq!(lowered.clusters.len(), 2);
    assert_eq!(lowered.clusters[0].source_range_utf8.start, 0);
    assert_eq!(lowered.clusters[0].source_range_utf8.end, 9);
    assert_eq!(lowered.clusters[0].source_range_utf16.unwrap().start, 0);
    assert_eq!(lowered.clusters[0].source_range_utf16.unwrap().end, 3);
    assert_eq!(lowered.clusters[0].glyph_range.start, 0);
    assert_eq!(lowered.clusters[0].glyph_range.end, 1);
    assert_eq!(lowered.clusters[1].source_range_utf8.start, 9);
    assert_eq!(lowered.clusters[1].source_range_utf8.end, 12);
    assert_eq!(lowered.clusters[1].source_range_utf16.unwrap().start, 3);
    assert_eq!(lowered.clusters[1].source_range_utf16.unwrap().end, 4);
    assert_eq!(lowered.clusters[1].glyph_range.start, 1);
    assert_eq!(lowered.clusters[1].glyph_range.end, 4);
    assert_eq!(lowered.glyph_transforms.as_ref().unwrap().len(), 4);
    assert_eq!(lowered.glyph_transforms.as_ref().unwrap()[0].xx, 0.8);
    assert!(!lowered.diagnostics.strict_visual_eligible);
    assert_eq!(
        lowered.diagnostics.reason.as_deref(),
        Some("q2CommonShapingReplayAuthorityPending")
    );
    assert_eq!(resources.font_blob_count(), 1);
    assert_eq!(resources.font_resources().blobs.len(), 1);
    assert_eq!(resources.font_resources().faces.len(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d3_fails_closed_without_node_or_exact_embedded_identity() {
    let (sidecars, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(0.0, 0.0, 20.0, 14.0);
    let mut resources = ResourceArena::default();
    let missing_node = LayerNode::leaf(bbox, None, Vec::new());
    let report = lower_horizontal_shaping_layer_node_shadow(
        &missing_node,
        bbox,
        &run,
        1,
        &sidecars,
        &[embedded_font(SOURCE_HAN)],
        &mut resources,
    );
    assert_eq!(report.source_node_id, None);
    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::MissingSourceNode)
    );
    assert!(!report.claims_glyph_run_slot);

    let node = LayerNode::leaf(bbox, Some(17), Vec::new());
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        1,
        &sidecars,
        &[embedded_font(b"not the measured font")],
        &mut resources,
    );
    assert_eq!(report.source_node_id, Some(17));
    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::EmbeddedFaceNotFound)
    );
    assert!(!report.claims_glyph_run_slot);
    assert_eq!(resources.font_blob_count(), 0);
    assert!(resources.font_resources().blobs.is_empty());
    assert!(resources.font_resources().faces.is_empty());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d3_preserves_text_run_fallback_and_has_no_product_caller() {
    let (sidecars, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(0.0, 0.0, 20.0, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run.clone())]);
    let mut resources = ResourceArena::default();
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        5,
        &sidecars,
        &[embedded_font(SOURCE_HAN)],
        &mut resources,
    );

    assert!(report.glyph_run.is_some());
    let rhwp::paint::LayerNodeKind::Leaf { ops } = &node.kind else {
        panic!("fixture node must remain a leaf");
    };
    assert_eq!(ops.len(), 1);
    assert!(matches!(ops[0], PaintOp::TextRun { .. }));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d3_reject_reason_names_are_stable_and_non_sensitive() {
    assert_eq!(
        HorizontalShapingGlyphLoweringRejectReason::MissingSidecar.as_str(),
        "missingSidecar"
    );
    assert_eq!(
        HorizontalShapingGlyphLoweringRejectReason::MeasurementGeometryInvalid.as_str(),
        "measurementGeometryInvalid"
    );
}
