//! Report-only shaping observations never take ownership of layout decisions.

use std::cell::Cell;

use rhwp::paint::*;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::TextStyle;
use serde_json::{json, Value};

struct Resolver {
    calls: Cell<usize>,
    mismatch: bool,
}

impl FontResolver for Resolver {
    fn resolve_font(&self, _: &FontRequest) -> ResolvedFontFace {
        ResolvedFontFace {
            portability: FontPortabilityKind::PortableBlob,
        }
    }

    fn shape_glyph_run(
        &self,
        _: &FontRequest,
        run: &TextRunNode,
        _: &ResolvedFontFace,
    ) -> Option<ResolvedGlyphRun> {
        self.calls.set(self.calls.get() + 1);
        Some(ResolvedGlyphRun {
            shape_key: ShapeKey {
                font_instance: FontInstanceKey {
                    face_key: FontFaceKey("fixture-face".to_string()),
                    size_px: run.style.font_size,
                    variations: Vec::new(),
                    synthetic_bold: false,
                    synthetic_italic: false,
                },
                direction: TextDirection::Ltr,
                writing_mode: WritingMode::HorizontalTb,
                script: None,
                language: None,
                features: Vec::new(),
                shaping_engine: ShapingEngineId("fixture".to_string()),
                fallback_policy: FontFallbackPolicyId("none".to_string()),
            },
            glyph_ids: vec![42],
            positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
            advances: Some(vec![LayerVector { dx: 12.0, dy: 0.0 }]),
            clusters: vec![GlyphCluster {
                source_range_utf8: TextSourceRange::new(0, 1),
                source_range_utf16: Some(TextSourceRange::new(0, 1)),
                text_range_utf8: Some(TextSourceRange::new(0, 1)),
                glyph_range: GlyphRange::new(0, 1),
                flags: Vec::new(),
            }],
            diagnostics: GlyphRunDiagnostics {
                quality: TextVariantQuality::Exact,
                replay_eligibility: GlyphRunReplayEligibility::Portable,
                strict_visual_eligible: !self.mismatch,
                max_origin_delta_px: 0.0,
                max_advance_delta_px: 0.0,
                max_residual_after_adjustment_px: 0.0,
                cluster_mismatch_count: u32::from(self.mismatch),
                missing_glyph_count: 0,
                used_fallback_font_count: u32::from(self.mismatch),
                reason: None,
            },
        })
    }
}

fn tree() -> PageLayerTree {
    let bbox = BoundingBox::new(10.0, 20.0, 50.0, 16.0);
    let run = TextRunNode {
        text: "A".to_string(),
        style: TextStyle {
            font_family: "fixture".to_string(),
            font_size: 16.0,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: Some(0),
        para_index: Some(3),
        char_start: Some(9),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: Some(vec![0.0, 10.0]),
        display_text: None,
    };
    PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
    )
}

#[test]
fn measurement_observes_existing_shaping_once_and_keeps_layout_positions() {
    let resolver = Resolver {
        calls: Cell::new(0),
        mismatch: false,
    };
    let lowerer = TextShapeLowerer::new(&resolver);
    let mut tree = tree();
    let original: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let analysis = lowerer.analyze_root(&tree.root);
    assert!(analysis.shaped_measurements.is_empty());
    assert_eq!(resolver.calls.get(), 0);

    let report = lowerer.lower_root(&mut tree.root);
    assert_eq!(resolver.calls.get(), 1);
    assert_eq!(report.public_glyph_run_count(), 1);
    assert_eq!(report.shaped_measurements.len(), 1);
    let measurement = &report.shaped_measurements[0];
    assert_eq!(measurement.legacy_width_px, 10.0);
    assert_eq!(measurement.shaped_width_px, 12.0);
    assert_eq!(measurement.delta_px, 2.0);
    assert_eq!(measurement.delta_ratio, 0.2);
    assert_eq!(measurement.shaping_quality, Some(GlyphRunQuality::Exact));
    let artifact: Value = serde_json::from_str(&report.shaped_measurements_json()).unwrap();
    assert_eq!(
        artifact["shapedMeasurements"][0]["source"],
        original["root"]["ops"][0]["source"]
    );
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        panic!("leaf")
    };
    let PaintOp::TextRun { bbox, run, .. } = &ops[0] else {
        panic!("fallback")
    };
    assert_eq!(
        run.layout_positions.as_deref(),
        Some([0.0, 10.0].as_slice())
    );
    assert_eq!(bbox.width, 50.0);
    assert_eq!(ops.len(), 2);
    assert!(!tree.to_json().contains("shapedMeasurements"));
}

#[test]
fn rejected_visual_candidate_still_reports_measurement_without_promoting_it() {
    let resolver = Resolver {
        calls: Cell::new(0),
        mismatch: true,
    };
    let mut tree = tree();
    let before = tree.to_json();
    let report = TextShapeLowerer::new(&resolver).lower_root(&mut tree.root);
    assert_eq!(resolver.calls.get(), 1);
    assert_eq!(report.public_glyph_run_count(), 0);
    assert_eq!(report.shaped_measurements[0].cluster_mismatch_count, 1);
    assert!(report.shaped_measurements[0].fallback_font_difference);
    assert_eq!(tree.to_json(), before);
}

fn measurement(legacy: f64, shaped: f64) -> ShapedMeasurementRunReport {
    ShapedMeasurementRunReport {
        document_id: None,
        sample_id: None,
        page_index: None,
        text_op_id: None,
        source: None,
        legacy_width_px: legacy,
        shaped_width_px: shaped,
        delta_px: shaped - legacy,
        delta_ratio: if legacy == 0.0 {
            0.0
        } else {
            (shaped - legacy) / legacy
        },
        cluster_mismatch_count: 1,
        fallback_font_difference: true,
        vertical_metric_difference: false,
        shaping_quality: Some(GlyphRunQuality::PositionAdjusted),
    }
}

#[test]
fn aggregates_only_caller_selected_run_line_and_paragraph_membership() {
    let mut report = TextShapeReport::default();
    report.shaped_measurements = vec![measurement(10.0, 12.0), measurement(20.0, 17.0)];
    assert!(report
        .summarize_shaped_measurement_line(None, None, None, None, 0, &[99])
        .is_none());
    let line = report
        .summarize_shaped_measurement_line(None, None, Some(0), Some("p".into()), 0, &[0, 1, 99])
        .unwrap();
    assert_eq!(line.legacy_line_width_px, 30.0);
    assert_eq!(line.shaped_line_width_px, 29.0);
    assert_eq!(line.contributing_run_count, 2);
    assert_eq!(line.max_run_delta_px, 3.0);
    assert_eq!(line.cluster_mismatch_count_sum, 2);
    report
        .summarize_shaped_measurement_line(None, None, Some(0), Some("p".into()), 1, &[0])
        .unwrap();
    let paragraph = report
        .summarize_shaped_measurement_paragraph(None, None, Some(0), Some("p".into()), &[0, 1])
        .unwrap();
    assert_eq!(paragraph.run_count, 3);
    assert_eq!(paragraph.line_count, 2);
    assert_eq!(paragraph.max_line_delta_px, 2.0);
    assert_eq!(paragraph.max_run_delta_px, 3.0);
    assert_eq!(paragraph.total_abs_delta_px, 3.0);
    let page = report
        .summarize_shaped_measurement_page(None, None, Some(0), &[0])
        .unwrap();
    assert_eq!(page.paragraph_count, 1);
    assert_eq!(page.line_count, 2);
    assert_eq!(page.run_count, 3);
    assert_eq!(page.fallback_font_difference_count, 3);
    assert_eq!(page.total_abs_delta_px, 3.0);
    assert!(report
        .summarize_shaped_measurement_paragraph(None, None, None, None, &[99])
        .is_none());
    assert!(report
        .summarize_shaped_measurement_page(None, None, None, &[99])
        .is_none());
    let artifact: Value = serde_json::from_str(&report.shaped_measurements_json()).unwrap();
    assert_eq!(artifact["shapedMeasurementPages"][0]["runCount"], 3);
    assert!(artifact["lineBreakShadows"].as_array().unwrap().is_empty());
    assert!(!report
        .shaped_measurements_json()
        .contains("lineBreakWouldChange"));
}

#[test]
fn shadow_json_distinguishes_known_absent_from_missing_context_and_gates_risk() {
    let mut shadow = LineBreakShadowReport {
        document_id: None,
        sample_id: None,
        page_index: Some(0),
        paragraph_id: None,
        line_index: 0,
        has_full_layout_context: true,
        legacy_available_width_px: LineBreakContextValue::Known(14.0),
        paragraph_width_px: LineBreakContextValue::Known(16.0),
        container_width_px: LineBreakContextValue::KnownAbsent,
        table_cell_constraint: LineBreakContextValue::KnownAbsent,
        tab_stop_summary: LineBreakContextValue::KnownAbsent,
        justification: LineBreakContextValue::Known(JustificationMode::Justify),
        legacy_line_segmentation_available: LineBreakContextValue::Known(false),
        legacy_line_width_px: 10.0,
        shaped_line_width_px: 12.0,
        overflow_delta_px: Some(-2.0),
        risk: LineBreakChangeRisk::NoChangeLikely,
        reason: Some("caller supplied context".to_string()),
    };
    let value = serde_json::to_value(&shadow).unwrap();
    assert_eq!(value["risk"], "noChangeLikely");
    assert_eq!(value["containerWidthPx"], json!({"state":"knownAbsent"}));
    assert_eq!(
        value["legacyLineSegmentationAvailable"],
        json!({"state":"known","value":false})
    );
    assert_eq!(
        value["justification"],
        json!({"state":"known","value":"justify"})
    );
    shadow.tab_stop_summary = LineBreakContextValue::Unknown;
    assert!(!shadow.has_minimum_layout_context());
    assert_eq!(
        shadow.reported_risk(),
        LineBreakChangeRisk::InsufficientContext
    );
    let mut report = TextShapeReport::default();
    report.line_break_shadows.push(shadow);
    let artifact: Value = serde_json::from_str(&report.shaped_measurements_json()).unwrap();
    assert_eq!(
        artifact["lineBreakShadows"][0]["risk"],
        "insufficientContext"
    );
    assert_eq!(
        artifact["lineBreakShadows"][0]["tabStopSummary"],
        json!({"state":"unknown"})
    );
}

#[test]
fn report_json_escapes_identifiers_and_serializes_nonfinite_values_as_null() {
    let mut observation = measurement(0.0, 0.0);
    observation.document_id = Some("doc\"\n\\".to_string());
    observation.shaped_width_px = f64::INFINITY;
    observation.delta_px = f64::NAN;
    let mut report = TextShapeReport::default();
    report.shaped_measurements.push(observation);
    let artifact: Value = serde_json::from_str(&report.shaped_measurements_json()).unwrap();
    let value = &artifact["shapedMeasurements"][0];
    assert_eq!(value["documentId"], "doc\"\n\\");
    assert_eq!(value["shapedWidthPx"], Value::Null);
    assert_eq!(value["deltaPx"], Value::Null);
    assert_eq!(value["deltaRatio"], 0.0);
    assert!(value.get("source").is_none());
}
