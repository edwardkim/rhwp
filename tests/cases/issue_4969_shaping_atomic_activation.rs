//! Issue #4969 W10-Q2-D4-B: the first product lane activates atomically.

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::style::Alignment;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, TextVariantKind};
#[cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
#[cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantSelectionBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const HAPPINESS: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");
// `ᄒᆞᆫ글`은 legacy 제품명 display projection 대상이므로 최초 direct-text lane의
// 양성 fixture로 쓰지 않는다. 이 문자열은 같은 옛한글 자모 shaping을 요구하지만
// model text와 replay text가 동일하다.
const TEXT: &str = "ᄒᆞᆫ말";

fn core_with_surface(
    text: &str,
    alignment: Alignment,
    char_border_fill_id: u16,
    no_stored_line_seg: bool,
) -> DocumentCore {
    core_with_surface_and_source(
        text,
        alignment,
        char_border_fill_id,
        no_stored_line_seg,
        SOURCE_HAN,
    )
    .0
}

fn core_with_surface_and_source(
    text: &str,
    alignment: Alignment,
    char_border_fill_id: u16,
    no_stored_line_seg: bool,
    exact_source: &[u8],
) -> (DocumentCore, u32) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.base_size = 1_000;
    char_shape.ratios = [80; 7];
    char_shape.spacings = [0; 7];
    char_shape.bold = false;
    char_shape.italic = false;
    char_shape.kerning = true;
    char_shape.border_fill_id = char_border_fill_id;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);

    document.doc_info.para_shapes[0].alignment = alignment;
    document.doc_info.para_shapes[0].border_fill_id = 0;
    document.doc_info.para_shapes[0].tab_def_id = 0;
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = text.to_string();
    paragraph.char_count = text.encode_utf16().count() as u32;
    paragraph.char_offsets = (0..text.chars().count() as u32).collect();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    paragraph.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: 1_500,
        text_height: 1_000,
        baseline_distance: 1_000,
        line_spacing: 500,
        column_start: 0,
        segment_width: 48_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
    }];
    if no_stored_line_seg {
        paragraph.line_segs.clear();
    }
    document.sections[0].paragraphs = vec![paragraph];
    document.sections[0].section_def.page_def.width = 50_000;
    document.sections[0].section_def.page_def.height = 100_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);
    core.register_exact_font_source_native(char_shape_id, 0, exact_source, 0)
        .expect("register exact source");
    (core, char_shape_id)
}

fn collect_text_ops<'a>(node: &'a LayerNode, ops: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_text_ops(child, ops);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_text_ops(child, ops),
        LayerNodeKind::Leaf { ops: leaf_ops } => ops.extend(leaf_ops.iter().filter(|op| {
            matches!(
                op,
                PaintOp::TextRun { .. } | PaintOp::GlyphRun { .. } | PaintOp::GlyphOutline { .. }
            )
        })),
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_b_one_line_run_publishes_one_common_alternative() {
    let core = core_with_surface(TEXT, Alignment::Left, 0, false);
    let layer_tree = core
        .build_page_layer_tree(0)
        .expect("build activated page layer tree");
    let mut ops = Vec::new();
    collect_text_ops(&layer_tree.root, &mut ops);
    let text_runs = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::TextRun { bbox, run } if run.text == TEXT => Some((*bbox, run.as_ref())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let glyph_runs = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::GlyphRun { bbox, run }
                if run.diagnostics.reason.as_deref()
                    == Some("q2CommonShapingCondensedDrawProjectionV1") =>
            {
                Some((*bbox, run.as_ref()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(text_runs.len(), 1, "one TextRun fallback must remain");
    assert_eq!(glyph_runs.len(), 1, "one common GlyphRun must be published");
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
            .count(),
        1,
        "the nominal GlyphRun must not duplicate the common claim"
    );
    let (text_bbox, text_run) = text_runs[0];
    let (glyph_bbox, glyph_run) = glyph_runs[0];
    let local_advance = glyph_run
        .advances
        .as_ref()
        .expect("common replay advances")
        .iter()
        .map(|advance| advance.dx)
        .sum::<f64>();
    let page_advance = local_advance * glyph_run.placement.run_to_page.a;
    assert_eq!(text_run.layout_positions, None);
    assert_eq!(text_bbox.x, glyph_bbox.x);
    assert_eq!(text_bbox.y, glyph_bbox.y);
    assert_eq!(text_bbox.width, glyph_bbox.width);
    assert_eq!(text_bbox.height, glyph_bbox.height);
    assert!((text_bbox.width - page_advance).abs() <= 1.0e-9);
    assert!(glyph_run.paint_style.font_size < text_run.style.font_size);
    assert_eq!(glyph_run.paint_style.ratio, 1.0);
    assert!(glyph_run.diagnostics.strict_visual_eligible);
    assert_eq!(layer_tree.text_sources.entries.len(), 1);
    assert_eq!(layer_tree.resources.font_blob_count(), 1);
    assert_eq!(layer_tree.resources.font_resources().blobs.len(), 1);
    assert_eq!(layer_tree.resources.font_resources().faces.len(), 1);

    let serialized: serde_json::Value =
        serde_json::from_str(&layer_tree.to_json()).expect("serialized product layer tree");
    assert_eq!(
        serialized["fontResources"]["blobs"]
            .as_array()
            .expect("font blob metadata")
            .len(),
        1
    );
    assert_eq!(
        serialized["fontResources"]["faces"]
            .as_array()
            .expect("font face metadata")
            .len(),
        1
    );
    assert_eq!(
        serialized["resources"]["fontBlobs"]
            .as_array()
            .expect("portable font payload")
            .len(),
        1
    );
    let serialized_text = serialized.to_string();
    assert_eq!(
        serialized_text
            .matches("q2CommonShapingCondensedDrawProjectionV1")
            .count(),
        1,
        "the product JSON must carry exactly one common replay alternative"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_b_rejected_surfaces_keep_only_legacy_text() {
    let cases = [
        (TEXT, Alignment::Center, 0),
        ("ᄒᆞᆫ글", Alignment::Left, 0),
        (TEXT, Alignment::Left, 2),
    ];
    for (text, alignment, char_border_fill_id) in cases {
        let core = core_with_surface(text, alignment, char_border_fill_id, false);
        let layer_tree = core
            .build_page_layer_tree(0)
            .expect("build rejected surface layer tree");
        let mut ops = Vec::new();
        collect_text_ops(&layer_tree.root, &mut ops);
        assert!(ops
            .iter()
            .any(|op| { matches!(op, PaintOp::TextRun { run, .. } if run.text == text) }));
        assert!(!ops.iter().any(|op| {
            matches!(op, PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref()
                    == Some("q2CommonShapingCondensedDrawProjectionV1"))
        }));
        assert_eq!(layer_tree.resources.font_blob_count(), 0);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d5_n1_no_lineseg_publishes_one_atomic_common_alternative() {
    let core = core_with_surface(TEXT, Alignment::Left, 0, true);
    let layer_tree = core
        .build_page_layer_tree(0)
        .expect("build no-LineSeg page layer tree");
    let mut ops = Vec::new();
    collect_text_ops(&layer_tree.root, &mut ops);
    let text_runs = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::TextRun { bbox, run } if run.text == TEXT => Some(*bbox),
            _ => None,
        })
        .collect::<Vec<_>>();
    let glyph_runs = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::GlyphRun { bbox, run }
                if run.diagnostics.reason.as_deref()
                    == Some("q2CommonShapingCondensedDrawProjectionV1") =>
            {
                Some((*bbox, run.as_ref()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(text_runs.len(), 1, "one TextRun fallback must remain");
    assert_eq!(glyph_runs.len(), 1, "one common GlyphRun must be published");
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
            .count(),
        1,
        "nominal GlyphRun must not duplicate the N1 common claim"
    );
    let (glyph_bbox, glyph_run) = glyph_runs[0];
    let local_advance = glyph_run
        .advances
        .as_ref()
        .expect("common replay advances")
        .iter()
        .map(|advance| advance.dx)
        .sum::<f64>();
    let page_advance = local_advance * glyph_run.placement.run_to_page.a;
    assert_eq!(text_runs[0].x, glyph_bbox.x);
    assert_eq!(text_runs[0].width, glyph_bbox.width);
    assert!((text_runs[0].width - page_advance).abs() <= 1.0e-9);
    assert_eq!(layer_tree.resources.font_blob_count(), 1);
    assert_eq!(layer_tree.resources.font_resources().faces.len(), 1);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q3_e0_default_product_baseline_receipt() {
    use std::time::Instant;

    const ITERATIONS: u32 = 64;
    let core = core_with_surface(TEXT, Alignment::Left, 0, false);
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        core.build_page_layer_tree(0)
            .expect("build Q2 default baseline layer tree");
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let layer_tree = core
        .build_page_layer_tree(0)
        .expect("build Q2 default baseline receipt");
    let layer_json = layer_tree.to_json();
    let plan = analyze_canvaskit_replay_plan(&layer_tree, CanvasKitReplayMode::Default);
    let plan_json = serde_json::to_string(&plan).expect("serialize CanvasKit baseline plan");

    assert!(layer_json.contains("q2CommonShapingCondensedDrawProjectionV1"));
    assert!(!layer_json.contains("\"type\":\"glyphOutline\""));
    assert_eq!(layer_tree.resources.font_blob_count(), 1);
    println!(
        "{{\"kind\":\"q3-e0-default-product-baseline\",\"iterations\":{ITERATIONS},\"elapsedNs\":{elapsed_ns},\"layerJsonBytes\":{},\"layerJsonBlake3\":\"{}\",\"canvasKitPlanBytes\":{},\"canvasKitPlanBlake3\":\"{}\"}}",
        layer_json.len(),
        blake3::hash(layer_json.as_bytes()).to_hex(),
        plan_json.len(),
        blake3::hash(plan_json.as_bytes()).to_hex(),
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q3_e4_native_instance_publishes_atomic_portable_outline() {
    let (mut core, char_shape_id) =
        core_with_surface_and_source("가변", Alignment::Left, 0, false, HAPPINESS);
    let baseline_tree = core
        .build_page_layer_tree(0)
        .expect("build default variable-font surface");
    let baseline = baseline_tree.to_json();
    assert!(!baseline.contains("\"type\":\"glyphOutline\""));
    let mut baseline_ops = Vec::new();
    collect_text_ops(&baseline_tree.root, &mut baseline_ops);
    let baseline_bbox = baseline_ops
        .iter()
        .find_map(|op| match op {
            PaintOp::TextRun { bbox, run } if run.text == "가변" => Some(*bbox),
            _ => None,
        })
        .expect("baseline TextRun bbox");
    let baseline_width = baseline_bbox.width;

    let title = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1",
        "axes": [
            { "tag": "wght", "value": 900.0 },
            { "tag": "opsz", "value": 900.0 }
        ]
    });
    let registered: serde_json::Value = serde_json::from_str(
        &core
            .set_exact_font_instance_native(&title.to_string())
            .expect("register strict native instance"),
    )
    .expect("registered response JSON");
    assert_eq!(registered["status"], "registered");
    assert_eq!(registered["requestGeneration"], 1);
    assert_eq!(registered["requestCount"], 1);
    assert!(registered["sourceGeneration"].as_u64().unwrap_or(0) > 0);
    assert_eq!(registered["axes"][0]["tag"], "opsz");
    assert_eq!(registered["axes"][1]["tag"], "wght");
    assert_eq!(registered["axes"][0]["value"], 900.0);
    assert_eq!(registered["axes"][1]["value"], 900.0);
    let selected_tree = core
        .build_page_layer_tree(0)
        .expect("build explicit-instance geometry surface");
    let selected_json = selected_tree.to_json();
    let mut selected_ops = Vec::new();
    collect_text_ops(&selected_tree.root, &mut selected_ops);
    let selected_width = selected_ops
        .iter()
        .find_map(|op| match op {
            PaintOp::TextRun { bbox, run } if run.text == "가변" => Some(bbox.width),
            _ => None,
        })
        .expect("selected TextRun width");
    assert_ne!(
        selected_width, baseline_width,
        "instance geometry must change"
    );
    let selected_glyph_runs = selected_ops
        .iter()
        .filter(|op| {
            matches!(op, PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref()
                    == Some("q3ExplicitInstanceGlyphRunProjectionV1"))
        })
        .count();
    let selected_outlines = selected_ops
        .iter()
        .filter(|op| {
            matches!(op, PaintOp::GlyphOutline { outline, .. }
                if outline.diagnostics.reason.as_deref()
                    == Some("q3VariableOutlineProjectionV1"))
        })
        .count();
    assert_eq!(selected_glyph_runs, 1);
    assert_eq!(selected_outlines, 1);
    let selected_glyph_run = selected_ops
        .iter()
        .find_map(|op| match op {
            PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref()
                    == Some("q3ExplicitInstanceGlyphRunProjectionV1") =>
            {
                Some(run.as_ref())
            }
            _ => None,
        })
        .expect("explicit GlyphRun");
    let selected_outline = selected_ops
        .iter()
        .find_map(|op| match op {
            PaintOp::GlyphOutline { outline, .. }
                if outline.diagnostics.reason.as_deref()
                    == Some("q3VariableOutlineProjectionV1") =>
            {
                Some(outline.as_ref())
            }
            _ => None,
        })
        .expect("explicit GlyphOutline");
    assert_eq!(selected_glyph_run.source.id, selected_outline.source.id);
    assert_eq!(
        selected_glyph_run.variant.equivalence_group,
        selected_outline.variant.equivalence_group
    );
    assert_eq!(
        selected_glyph_run.variant.anchor_op_id,
        selected_outline.variant.anchor_op_id
    );
    assert!(selected_json.contains("\"type\":\"glyphOutline\""));
    assert!(!selected_json.contains("q2CommonShapingCondensedDrawProjectionV1"));
    let canvas_kit_plan =
        analyze_canvaskit_replay_plan(&selected_tree, CanvasKitReplayMode::Default);
    let canvas_kit_json =
        serde_json::to_string(&canvas_kit_plan).expect("serialize explicit CanvasKit replay plan");
    assert!(canvas_kit_json.contains("glyphOutline"));
    for (backend, expected_kind, expected_fallback) in [
        (
            VariantSelectionBackend::CanvasKit,
            TextVariantKind::GlyphOutline,
            false,
        ),
        (
            VariantSelectionBackend::CanvasKitBrowser,
            TextVariantKind::GlyphOutline,
            false,
        ),
        (
            VariantSelectionBackend::NativeSkia,
            TextVariantKind::TextRun,
            true,
        ),
        (VariantSelectionBackend::Svg, TextVariantKind::TextRun, true),
        (
            VariantSelectionBackend::Canvas2D,
            TextVariantKind::TextRun,
            true,
        ),
    ] {
        let reports = analyze_text_variant_selection(
            &selected_tree,
            TextVariantSelectionOptions {
                backend,
                prefer_strict_outline: true,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        let report = reports
            .iter()
            .find(|report| report.equivalence_group == selected_glyph_run.variant.equivalence_group)
            .expect("explicit variant selection report");
        assert_eq!(
            report.selected_variant_kind,
            Some(expected_kind),
            "{backend:?}"
        );
        assert_eq!(report.fallback_required, expected_fallback, "{backend:?}");
    }
    println!(
        "{}",
        serde_json::json!({
            "kind": "q3-e4-atomic-publication-receipt",
            "baselineWidthPx": baseline_width,
            "selectedWidthPx": selected_width,
            "deltaPx": selected_width - baseline_width,
            "glyphRunPublished": selected_glyph_runs,
            "glyphOutlinePublished": selected_outlines,
            "canvasKitSelectsOutline": canvas_kit_json.contains("glyphOutline")
        })
    );

    let canonical_title = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1",
        "axes": [
            { "tag": "opsz", "value": 900.0 },
            { "tag": "wght", "value": 900.0 }
        ]
    });
    let already: serde_json::Value = serde_json::from_str(
        &core
            .set_exact_font_instance_native(&canonical_title.to_string())
            .expect("idempotent native instance"),
    )
    .expect("idempotent response JSON");
    assert_eq!(already["status"], "already-registered");
    assert_eq!(already["requestGeneration"], 1);

    let explicit_default = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1",
        "axes": [
            { "tag": "wght", "value": 400.0 },
            { "tag": "opsz", "value": 400.0 }
        ]
    });
    let updated: serde_json::Value = serde_json::from_str(
        &core
            .set_exact_font_instance_native(&explicit_default.to_string())
            .expect("update to explicit default"),
    )
    .expect("updated response JSON");
    assert_eq!(updated["status"], "updated");
    assert_eq!(updated["requestGeneration"], 2);
    assert_eq!(updated["axes"], serde_json::json!([]));
    let explicit_default_tree = core
        .build_page_layer_tree(0)
        .expect("build explicit-default product surface");
    assert_eq!(explicit_default_tree.to_json(), baseline);
    let mut explicit_default_ops = Vec::new();
    collect_text_ops(&explicit_default_tree.root, &mut explicit_default_ops);
    let explicit_default_bbox = explicit_default_ops
        .iter()
        .find_map(|op| match op {
            PaintOp::TextRun { bbox, run } if run.text == "가변" => Some(*bbox),
            _ => None,
        })
        .expect("explicit-default TextRun bbox");
    assert_eq!(explicit_default_bbox.x, baseline_bbox.x);
    assert_eq!(explicit_default_bbox.y, baseline_bbox.y);
    assert_eq!(explicit_default_bbox.width, baseline_bbox.width);
    assert_eq!(explicit_default_bbox.height, baseline_bbox.height);
    assert!(!explicit_default_ops.iter().any(|op| matches!(
        op,
        PaintOp::GlyphRun { run, .. }
            if run.diagnostics.reason.as_deref()
                == Some("q3ExplicitInstanceGlyphRunProjectionV1")
    )));
    assert!(!explicit_default_ops
        .iter()
        .any(|op| matches!(op, PaintOp::GlyphOutline { .. })));

    let clear = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1"
    });
    let cleared: serde_json::Value = serde_json::from_str(
        &core
            .clear_exact_font_instance_native(&clear.to_string())
            .expect("clear native instance"),
    )
    .expect("cleared response JSON");
    assert_eq!(cleared["status"], "cleared");
    assert_eq!(cleared["requestGeneration"], 3);
    assert_eq!(cleared["requestCount"], 0);

    let already_cleared: serde_json::Value = serde_json::from_str(
        &core
            .clear_exact_font_instance_native(&clear.to_string())
            .expect("idempotent clear"),
    )
    .expect("idempotent clear response JSON");
    assert_eq!(already_cleared["status"], "already-cleared");
    assert_eq!(already_cleared["requestGeneration"], 3);
    assert_eq!(
        core.build_page_layer_tree(0)
            .expect("build surface after reversible clear")
            .to_json(),
        baseline
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q3_e3_negative_surfaces_roll_back_the_whole_paragraph() {
    for (text, alignment) in [
        ("가변Typography", Alignment::Left),
        ("가변", Alignment::Center),
    ] {
        let (mut core, char_shape_id) =
            core_with_surface_and_source(text, alignment, 0, false, HAPPINESS);
        let baseline = core
            .build_page_layer_tree(0)
            .expect("build negative baseline")
            .to_json();
        let request = serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [{ "tag": "wght", "value": 900.0 }]
        });
        core.set_exact_font_instance_native(&request.to_string())
            .expect("register negative-surface request");
        assert_eq!(
            core.build_page_layer_tree(0)
                .expect("build negative requested surface")
                .to_json(),
            baseline,
            "unsupported surface must keep the complete default paragraph: {text:?}"
        );
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q3_e1_strict_native_dto_rejects_without_mutation() {
    let (mut core, char_shape_id) =
        core_with_surface_and_source("Typography", Alignment::Left, 0, false, HAPPINESS);
    let too_many_axes = (0..17)
        .map(|_| serde_json::json!({ "tag": "wght", "value": 400.0 }))
        .collect::<Vec<_>>();
    let invalid = [
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "unknown",
            "axes": []
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [],
            "fontBytes": [1, 2, 3]
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 7,
            "mode": "boundedHorizontalLtrV1",
            "axes": []
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": too_many_axes
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [
                { "tag": "wght", "value": 650.0 },
                { "tag": "wght", "value": 700.0 }
            ]
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [{ "tag": "wght", "value": 901.0 }]
        })
        .to_string(),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [{ "tag": "wgt", "value": 650.0 }]
        })
        .to_string(),
        format!(
            "{{\"charShapeId\":{char_shape_id},\"languageIndex\":0,\"mode\":\"boundedHorizontalLtrV1\",\"axes\":[{{\"tag\":\"wght\",\"value\":1e400}}]}}"
        ),
        serde_json::json!({
            "charShapeId": char_shape_id + 1,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": []
        })
        .to_string(),
        format!(
            "{{\"charShapeId\":{char_shape_id},\"languageIndex\":0,\"mode\":\"boundedHorizontalLtrV1\",\"axes\":[],\"padding\":\"{}\"}}",
            "x".repeat(16 * 1024)
        ),
    ];
    for options in invalid {
        assert!(
            core.set_exact_font_instance_native(&options).is_err(),
            "invalid strict DTO must fail: {}",
            &options[..options.len().min(160)]
        );
    }

    let valid = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1",
        "axes": [{ "tag": "wght", "value": 650.0 }]
    });
    let registered: serde_json::Value = serde_json::from_str(
        &core
            .set_exact_font_instance_native(&valid.to_string())
            .expect("first valid request after rejects"),
    )
    .expect("valid response JSON");
    assert_eq!(registered["requestGeneration"], 1);

    for invalid_clear in [
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "unknown"
        }),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": []
        }),
        serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 7,
            "mode": "boundedHorizontalLtrV1"
        }),
    ] {
        assert!(core
            .clear_exact_font_instance_native(&invalid_clear.to_string())
            .is_err());
    }
    let clear = serde_json::json!({
        "charShapeId": char_shape_id,
        "languageIndex": 0,
        "mode": "boundedHorizontalLtrV1"
    });
    let cleared: serde_json::Value = serde_json::from_str(
        &core
            .clear_exact_font_instance_native(&clear.to_string())
            .expect("valid clear after rejects"),
    )
    .expect("clear response JSON");
    assert_eq!(cleared["requestGeneration"], 2);
    assert_eq!(cleared["requestCount"], 0);
}
