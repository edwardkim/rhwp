//! Canvas measurement lifetime and output isolation, using the original #7084 fixtures.
use rhwp::document_core::{CanvasMetricBatch, CanvasMetricReply};
use rhwp::paint::layer_tree::{LayerNode, LayerNodeKind};
use rhwp::paint::paint_op::PaintOp;
use rhwp::paint::profile::RenderProfile;
use rhwp::renderer::canvas_text_font::CanvasTextFont;
use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::TextRunNode;
use rhwp::renderer::supplemental_metrics::{
    MetricBackend, MetricContext, MetricError, SupplementalMetric,
};
use rhwp::DocumentCore;

fn replies(batch: &CanvasMetricBatch) -> Vec<CanvasMetricReply> {
    batch
        .requests
        .iter()
        .map(|r| CanvasMetricReply {
            key: r.key.clone(),
            cluster: r.cluster.clone(),
            font: r.font.clone(),
            resolved_font: r.font.clone(),
            measured_advance_px: 18.302703857,
        })
        .collect()
}

#[test]
fn control_display_text_does_not_shift_logical_cluster_membership() {
    use rhwp::model::control::{CharOverlap, Control};
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    let text = "\u{f02ce}😀\u{fe0f}😀";
    let mut offset = 8u32; // The control payload precedes the paragraph's visible scalars.
    let char_offsets = text
        .chars()
        .map(|ch| {
            let start = offset;
            offset += ch.len_utf16() as u32;
            start
        })
        .collect();
    let para = Paragraph {
        text: text.into(),
        char_offsets,
        char_count: offset + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            line_height: 400,
            ..Default::default()
        }],
        controls: vec![Control::CharOverlap(CharOverlap {
            chars: vec!['A', 'B'],
            ..Default::default()
        })],
        ..Default::default()
    };
    let composed = rhwp::renderer::composer::compose_paragraph(&para);
    let runs = &composed.lines[0].runs;
    assert!(runs
        .iter()
        .any(|r| r.inserted_control_text && r.text == "AB"));
    assert!(runs
        .iter()
        .any(|r| !r.inserted_control_text && r.char_overlap.is_some() && r.text == "\u{f02ce}"));
    let mut document = core("hwp").document().clone();
    document.sections[0].paragraphs = vec![para];
    let mut core = DocumentCore::new_empty();
    core.set_document(document);
    let batch = core.collect_canvas_metric_requests(context()).unwrap();
    assert_eq!(
        batch.requests.len(),
        1,
        "Only the standalone emoji is eligible after both kinds of display text"
    );
    assert_eq!(batch.requests[0].cluster, "😀");
}

#[test]
fn style_boundaries_do_not_publish_partial_grapheme_requests() {
    for ext in ["hwp", "hwpx"] {
        for suffix in ["\u{fe0f}", "\u{200d}😀"] {
            let mut core = core(ext);
            core.insert_text_in_cell_native(0, 12, 1, 5, 0, 3, suffix)
                .unwrap();
            core.apply_char_format_in_cell_native(0, 12, 1, 5, 0, 3, 4, r#"{"bold":true}"#)
                .unwrap();
            let batch = core.collect_canvas_metric_requests(context()).unwrap();
            assert!(
                batch.requests.is_empty(),
                "{ext} {suffix:?}: {:?}",
                batch.requests
            );
        }
    }
}

#[test]
fn active_emoji_metric_preserves_unrelated_nominal_grapheme_run() {
    use rhwp::model::paragraph::{CharShapeRef, Paragraph};
    fn has_whole_run(node: &LayerNode) -> bool {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().any(has_whole_run),
            LayerNodeKind::ClipRect { child, .. } => has_whole_run(child),
            LayerNodeKind::Leaf { ops } => ops.iter().any(|op| {
                matches!(op,
                PaintOp::TextRun { run, .. } if run.text == "ᄒᆞᆫ말")
            }),
        }
    }
    let mut core = core("hwp");
    let mut document = core.document().clone();
    document.sections[0].paragraphs.push(Paragraph {
        text: "ᄒᆞᆫ말".into(),
        char_count: 5,
        char_offsets: vec![0, 1, 2, 3],
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    });
    core.set_document(document);
    let preserved = |core: &DocumentCore| {
        (0..core.page_count()).any(|page| {
            let tree = core
                .build_canvas_page_layer_tree_with_profile(page, RenderProfile::Screen)
                .unwrap();
            has_whole_run(&tree.root)
        })
    };
    assert!(
        preserved(&core),
        "Portable composition retains its original nominal run"
    );
    let batch = core.collect_canvas_metric_requests(context()).unwrap();
    core.register_canvas_metric_replies(context(), batch.revision, replies(&batch))
        .unwrap();
    core.select_canvas_metrics(true).unwrap();
    assert!(core.canvas_metrics_active());
    assert!(
        preserved(&core),
        "An emoji elsewhere must not subdivide the unrelated shaping fallback"
    );
}

#[test]
fn cached_standalone_metric_does_not_leak_into_a_style_split_cluster() {
    for ext in ["hwp", "hwpx"] {
        let mut core = core(ext);
        core.insert_text_in_cell_native(0, 12, 1, 5, 0, 6, " 😀")
            .unwrap();
        core.insert_text_in_cell_native(0, 12, 1, 5, 0, 3, "\u{fe0f}")
            .unwrap();
        core.apply_char_format_in_cell_native(0, 12, 1, 5, 0, 3, 4, r#"{"bold":true}"#)
            .unwrap();
        let original = format!("{:?}", core.document());
        let baseline = run(&core);
        let emoji_measurement = |run: &TextRunNode| {
            let index = run.text.chars().position(|ch| ch == '😀').unwrap();
            let positions = positions(run);
            (
                run.char_start.unwrap() + index,
                positions[index + 1] - positions[index],
            )
        };
        let (baseline_offset, baseline_advance) = emoji_measurement(&baseline);
        let batch = core.collect_canvas_metric_requests(context()).unwrap();
        assert_eq!(
            batch.requests.len(),
            1,
            "only the standalone emoji is eligible: {ext}"
        );
        assert_eq!(batch.requests[0].cluster, "😀");
        core.register_canvas_metric_replies(context(), batch.revision, replies(&batch))
            .unwrap();
        core.select_canvas_metrics(true).unwrap();
        let (active_offset, active_advance) = emoji_measurement(&run(&core));
        assert_eq!(active_offset, baseline_offset, "same source emoji: {ext}");
        assert!(
            (active_advance - baseline_advance).abs() < 1e-9,
            "protected first emoji advance: {ext}: {baseline_advance} -> {active_advance}"
        );
        let mut sources = Vec::new();
        for page in 0..2 {
            let trace: serde_json::Value =
                serde_json::from_str(&core.get_font_decision_trace_native(page, "{}").unwrap())
                    .unwrap();
            for record in trace["records"].as_array().unwrap() {
                if record["source"]["character"] == "😀" {
                    sources.push(
                        record["layoutMetric"]["widthSource"]
                            .as_str()
                            .unwrap()
                            .to_owned(),
                    );
                }
            }
        }
        assert_eq!(
            sources,
            ["heuristicHalfwidth", "supplementalBackendMeasured"],
            "{ext}"
        );
        assert_eq!(format!("{:?}", core.document()), original);
    }
}

#[test]
fn collected_original_requests_register_and_preserve_ir_and_portable_svg() {
    for ext in ["hwp", "hwpx"] {
        let mut core = core(ext);
        let original = format!("{:?}", core.document());
        let baseline = run(&core);
        let svg = core.render_page_svg_native(0).unwrap();
        let batch = core.collect_canvas_metric_requests(context()).unwrap();
        assert!(batch.requests.iter().any(|r| r.cluster == "😀"), "{ext}");
        assert!(
            batch
                .requests
                .iter()
                .all(|r| r.cluster.chars().count() == 1
                    && !r.cluster.chars().any(char::is_whitespace))
        );
        assert!(core
            .register_canvas_metric_replies(context(), batch.revision, replies(&batch))
            .unwrap());
        core.select_canvas_metrics(true).unwrap();
        assert_ne!(positions(&run(&core)), positions(&baseline));
        let warm = core.collect_canvas_metric_requests(context()).unwrap();
        assert!(!core
            .register_canvas_metric_replies(context(), warm.revision, replies(&warm))
            .unwrap());
        core.select_canvas_metrics(false).unwrap();
        assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
        assert_eq!(format!("{:?}", core.document()), original);
    }
}

#[test]
fn request_envelope_rejects_partial_duplicate_and_descriptor_mismatch_atomically() {
    let mut core = core("hwp");
    let baseline = run(&core);
    let batch = core.collect_canvas_metric_requests(context()).unwrap();
    let mut partial = replies(&batch);
    partial.pop().unwrap();
    assert_eq!(
        core.register_canvas_metric_replies(context(), batch.revision, partial),
        Err(MetricError::ContextMismatch)
    );
    let mut invalid = replies(&batch);
    invalid[0].font = "99px serif".into();
    assert_eq!(
        core.register_canvas_metric_replies(context(), batch.revision, invalid),
        Err(MetricError::InvalidDescriptor)
    );
    if batch.requests.len() > 1 {
        let mut duplicate = replies(&batch);
        duplicate[1].key = duplicate[0].key.clone();
        assert_eq!(
            core.register_canvas_metric_replies(context(), batch.revision, duplicate),
            Err(MetricError::DuplicateKey)
        );
    }
    assert!(!core.canvas_metrics_active());
    assert_eq!(positions(&run(&core)), positions(&baseline));
    assert!(core
        .register_canvas_metric_replies(context(), batch.revision, replies(&batch))
        .unwrap());
}

#[test]
fn requests_are_bound_to_latest_ticket_and_source_revision() {
    let mut core = core("hwp");
    let old = core.collect_canvas_metric_requests(context()).unwrap();
    let current = core.collect_canvas_metric_requests(context()).unwrap();
    assert_ne!(old.revision, current.revision);
    assert_eq!(
        core.register_canvas_metric_replies(context(), old.revision, replies(&old)),
        Err(MetricError::ContextMismatch)
    );
    core.apply_char_format_in_cell_native(0, 12, 1, 5, 0, 0, 3, r##"{"textColor":"#ff0000"}"##)
        .unwrap();
    assert_eq!(
        core.register_canvas_metric_replies(context(), current.revision, replies(&current)),
        Err(MetricError::ContextMismatch)
    );
    let fresh = core.collect_canvas_metric_requests(context()).unwrap();
    assert!(core
        .register_canvas_metric_replies(context(), fresh.revision, replies(&fresh))
        .unwrap());
}

fn context() -> MetricContext {
    MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: MetricBackend::Canvas2d,
    }
}

fn core(ext: &str) -> DocumentCore {
    let path = format!(
        "{}/samples/issue3587/c-form-labnote-001-stage11-filled.{ext}",
        env!("CARGO_MANIFEST_DIR")
    );
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn find(node: &LayerNode) -> Option<TextRunNode> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children.iter().find_map(find),
        LayerNodeKind::ClipRect { child, .. } => find(child),
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| match op {
            PaintOp::TextRun { run, .. } if run.text.contains('😀') => Some((**run).clone()),
            _ => None,
        }),
    }
}

fn run(core: &DocumentCore) -> TextRunNode {
    for page in 0..2 {
        let tree = core
            .build_canvas_page_layer_tree_with_profile(page, RenderProfile::Screen)
            .unwrap();
        if let Some(run) = find(&tree.root) {
            return run;
        }
    }
    panic!("fixture must contain the reported emoji");
}

fn entries(run: &TextRunNode) -> Vec<SupplementalMetric> {
    let font = CanvasTextFont::for_positioned_text(&run.style, 0.0);
    vec![SupplementalMetric::from_canvas_measurement(
        &run.style,
        "😀",
        font.descriptor(),
        font.descriptor().into(),
        18.302703857,
    )
    .unwrap()]
}

fn positions(run: &TextRunNode) -> Vec<f64> {
    // K0 runs intentionally omit layout_positions. Their painter computes the
    // replay positions from the tree's resolved style, not a stored position array.
    run.layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style))
}

#[test]
fn canvas_session_switch_rebuilds_positions_and_protects_portable_output() {
    for ext in ["hwp", "hwpx"] {
        let mut core = core(ext);
        let original = format!("{:?}", core.document());
        let baseline = run(&core);
        let svg = core.render_page_svg_native(0).unwrap();
        core.begin_canvas_metric_session(context()).unwrap();
        assert!(core
            .register_canvas_metrics(context(), entries(&baseline))
            .unwrap());
        assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
        assert!(core.select_canvas_metrics(true).unwrap());
        let active = run(&core);
        assert!(active.style.supplemental_metrics.is_some());
        assert_ne!(
            positions(&active),
            positions(&baseline),
            "cached positions must be rebuilt: {ext}"
        );
        assert!(core.render_page_svg_native(0).is_err());
        assert!(core.build_page_render_tree(0).is_err());
        assert!(core
            .build_page_layer_tree_with_profile(0, RenderProfile::Screen)
            .is_err());
        assert!(!core
            .register_canvas_metrics(context(), entries(&baseline))
            .unwrap());
        assert!(!core.select_canvas_metrics(true).unwrap());
        assert!(core.select_canvas_metrics(false).unwrap());
        assert_eq!(positions(&run(&core)), positions(&baseline));
        assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
        assert_eq!(
            format!("{:?}", core.document()),
            original,
            "metrics must not edit document IR"
        );
    }
}

#[test]
fn new_generation_invalidates_retained_styles_and_rejects_late_results() {
    let mut core = core("hwp");
    let baseline = run(&core);
    core.begin_canvas_metric_session(context()).unwrap();
    core.register_canvas_metrics(context(), entries(&baseline))
        .unwrap();
    core.select_canvas_metrics(true).unwrap();
    let retained = run(&core);
    core.begin_canvas_metric_session(MetricContext {
        font_generation: 2,
        ..context()
    })
    .unwrap();
    assert!(!core.canvas_metrics_active());
    assert_eq!(
        core.register_canvas_metrics(context(), entries(&baseline)),
        Err(MetricError::ContextMismatch)
    );
    assert_eq!(positions(&run(&core)), positions(&baseline));
    assert_eq!(
        EmbeddedTextMeasurer.compute_char_positions("😀", &retained.style),
        EmbeddedTextMeasurer.compute_char_positions("😀", &baseline.style)
    );
}

#[test]
fn document_replacement_drops_owner_and_batch_rejects_metric_mutation() {
    let mut core = core("hwpx");
    let baseline = run(&core);
    core.begin_canvas_metric_session(context()).unwrap();
    core.register_canvas_metrics(context(), entries(&baseline))
        .unwrap();
    core.select_canvas_metrics(true).unwrap();
    core.begin_batch_native().unwrap();
    assert_eq!(
        core.select_canvas_metrics(false),
        Err(MetricError::EditInProgress)
    );
    assert_eq!(
        core.register_canvas_metrics(context(), entries(&baseline)),
        Err(MetricError::EditInProgress)
    );
    assert_eq!(
        core.begin_canvas_metric_session(context()),
        Err(MetricError::EditInProgress)
    );
    core.end_batch_native().unwrap();
    let doc = core.document().clone();
    core.set_document(doc);
    assert!(!core.canvas_metrics_active());
    assert_eq!(
        core.select_canvas_metrics(true),
        Err(MetricError::ContextMismatch)
    );
    assert_eq!(positions(&run(&core)), positions(&baseline));
}

#[test]
fn cell_format_batch_keeps_metric_binding_when_style_ids_change() {
    let mut core = core("hwp");
    let baseline = run(&core);
    core.begin_canvas_metric_session(context()).unwrap();
    core.register_canvas_metrics(context(), entries(&baseline))
        .unwrap();
    core.select_canvas_metrics(true).unwrap();
    let before = run(&core);
    core.begin_batch_native().unwrap();
    core.apply_char_format_in_cell_native(0, 12, 1, 5, 0, 0, 3, r##"{"textColor":"#ff0000"}"##)
        .unwrap();
    core.end_batch_native().unwrap();
    let after = run(&core);
    assert_ne!(after.char_shape_id, before.char_shape_id);
    assert!(after.style.supplemental_metrics.is_some());
    assert_eq!(positions(&after), positions(&before));
    core.select_canvas_metrics(false).unwrap();
    assert_ne!(positions(&run(&core)), positions(&after));
}
