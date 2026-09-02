//! Issue #4969 W10-Q2-D4-B: the first product lane activates atomically.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::style::Alignment;
use rhwp::model::table::{Cell, Table, VerticalAlign};
use rhwp::paint::{
    GlyphRunOrientation, GlyphTransform, LayerBuilder, LayerGlyphRunPaint, LayerNode,
    LayerNodeKind, PaintOp, RenderProfile, TextDirection, TextV2Diagnostics, TextVariantKind,
    WritingMode,
};
#[cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
#[cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantSelectionBackend,
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const HAPPINESS: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");
const NOTO: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
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
fn issue_4969_q3_e5_hiding_header_invalidates_cached_layer_output() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    core.create_header_footer_native(0, true, 0)
        .expect("create header");
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "CACHE_HEADER_SENTINEL")
        .expect("insert header sentinel");

    let visible = core
        .get_page_layer_tree_with_profile_native(0, RenderProfile::Screen)
        .expect("visible header layer JSON");
    assert!(visible.contains("CACHE_HEADER_SENTINEL"));

    core.toggle_hide_header_footer_native(0, true)
        .expect("hide header");
    let hidden = core
        .get_page_layer_tree_with_profile_native(0, RenderProfile::Screen)
        .expect("hidden header layer JSON");
    assert!(!hidden.contains("CACHE_HEADER_SENTINEL"));
    assert_ne!(hidden, visible);
}

#[test]
fn issue_4969_q3_e5_blank_replacement_drops_cached_layer_output() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    core.insert_text_native(0, 0, 0, "CACHE_BODY_SENTINEL")
        .expect("insert body sentinel");

    let populated = core
        .get_page_layer_tree_with_profile_native(0, RenderProfile::Screen)
        .expect("populated layer JSON");
    assert!(populated.contains("CACHE_BODY_SENTINEL"));

    core.create_blank_document_native()
        .expect("replace with another blank document");
    let blank = core
        .get_page_layer_tree_with_profile_native(0, RenderProfile::Screen)
        .expect("replacement layer JSON");
    assert!(!blank.contains("CACHE_BODY_SENTINEL"));
    assert_ne!(blank, populated);
}

fn collect_layer_json_ops<'a>(value: &'a serde_json::Value, ops: &mut Vec<&'a serde_json::Value>) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                collect_layer_json_ops(value, ops);
            }
        }
        serde_json::Value::Object(values) => {
            if values
                .get("type")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| matches!(kind, "textRun" | "glyphRun" | "glyphOutline"))
            {
                ops.push(value);
            }
            for value in values.values() {
                collect_layer_json_ops(value, ops);
            }
        }
        _ => {}
    }
}

fn mutate_bounded_vertical_glyph_runs(
    node: &mut LayerNode,
    mutate: &mut impl FnMut(&mut LayerGlyphRunPaint),
) {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                mutate_bounded_vertical_glyph_runs(child, mutate);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            mutate_bounded_vertical_glyph_runs(child, mutate);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::GlyphRun { run, .. } = op {
                    if run.diagnostics.reason.as_deref() == Some("boundedVerticalHwp5TableCellV1") {
                        mutate(run);
                    }
                }
            }
        }
    }
}

fn bounded_vertical_table_core(register_exact_source: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank HWP5 template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.base_size = 1_000;
    char_shape.ratios = [100; 7];
    char_shape.spacings = [0; 7];
    char_shape.bold = false;
    char_shape.italic = false;
    char_shape.kerning = false;
    char_shape.border_fill_id = 0;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);

    let mut cell_para = Paragraph::new_empty();
    cell_para.text = "한글".to_string();
    cell_para.char_count = 2;
    cell_para.char_offsets = vec![0, 1];
    cell_para.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    cell_para.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: 1_500,
        text_height: 1_000,
        baseline_distance: 1_000,
        line_spacing: 0,
        column_start: 0,
        segment_width: 18_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
    }];
    let cell = Cell {
        row: 0,
        col: 0,
        row_span: 1,
        col_span: 1,
        width: 12_000,
        height: 20_000,
        paragraphs: vec![cell_para],
        text_direction: 2,
        vertical_align: VerticalAlign::Top,
        ..Default::default()
    };
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };
    table.common.width = 12_000;
    table.common.height = 20_000;
    table.common.treat_as_char = true;
    let mut host = Paragraph::new_empty();
    host.controls.push(Control::Table(Box::new(table)));
    host.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: 20_000,
        text_height: 1_000,
        baseline_distance: 1_000,
        line_spacing: 0,
        column_start: 0,
        segment_width: 48_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
    }];
    document.sections[0].paragraphs = vec![host];
    core.set_document(document);
    if register_exact_source {
        core.register_exact_font_source_native(char_shape_id, 0, NOTO, 0)
            .expect("register public Noto exact source");
    }
    core
}

fn vertical_text_line_child_counts(node: &RenderNode, counts: &mut Vec<usize>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let vertical_children = node
            .children
            .iter()
            .filter(|child| {
                matches!(
                    &child.node_type,
                    RenderNodeType::TextRun(run) if run.is_vertical && (run.text == "한" || run.text == "글")
                )
            })
            .count();
        if vertical_children > 0 {
            counts.push(vertical_children);
        }
    }
    for child in &node.children {
        vertical_text_line_child_counts(child, counts);
    }
}

fn collect_vertical_line_geometry(
    node: &RenderNode,
    geometry: &mut Vec<(
        rhwp::renderer::render_tree::BoundingBox,
        Vec<rhwp::renderer::render_tree::BoundingBox>,
    )>,
) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let runs = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) if run.is_vertical => Some(child.bbox),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !runs.is_empty() {
            geometry.push((node.bbox, runs));
        }
    }
    for child in &node.children {
        collect_vertical_line_geometry(child, geometry);
    }
}

fn collect_vertical_line_ids(node: &RenderNode, ids: &mut Vec<u32>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && node.children.iter().any(
            |child| matches!(&child.node_type, RenderNodeType::TextRun(run) if run.is_vertical),
        )
    {
        ids.push(node.id);
    }
    for child in &node.children {
        collect_vertical_line_ids(child, ids);
    }
}

fn replace_nth_vertical_run_text(node: &mut RenderNode, target: usize, seen: &mut usize) -> bool {
    if let RenderNodeType::TextRun(run) = &mut node.node_type {
        if run.is_vertical {
            if *seen == target {
                run.text = "가".to_string();
                return true;
            }
            *seen += 1;
        }
    }
    node.children
        .iter_mut()
        .any(|child| replace_nth_vertical_run_text(child, target, seen))
}

fn find_layer_source_node(node: &LayerNode, source_node_id: u32) -> Option<&LayerNode> {
    if node.source_node_id == Some(source_node_id) {
        return Some(node);
    }
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children
            .iter()
            .find_map(|child| find_layer_source_node(child, source_node_id)),
        LayerNodeKind::ClipRect { child, .. } => find_layer_source_node(child, source_node_id),
        LayerNodeKind::Leaf { .. } => None,
    }
}

fn assert_bbox(actual: rhwp::renderer::render_tree::BoundingBox, expected: (f64, f64, f64, f64)) {
    let (x, y, width, height) = expected;
    assert!((actual.x - x).abs() <= 1.0e-9, "x: {actual:?}");
    assert!((actual.y - y).abs() <= 1.0e-9, "y: {actual:?}");
    assert!((actual.width - width).abs() <= 1.0e-9, "width: {actual:?}");
    assert!(
        (actual.height - height).abs() <= 1.0e-9,
        "height: {actual:?}"
    );
}

#[test]
fn issue_4969_q4_d2_target_commits_one_line_while_no_source_keeps_legacy_tree() {
    let target = bounded_vertical_table_core(true)
        .build_page_render_tree(0)
        .expect("build D2 target page tree");
    let mut target_geometry = Vec::new();
    collect_vertical_line_geometry(&target.root, &mut target_geometry);
    assert_eq!(target_geometry.len(), 1);
    assert_bbox(
        target_geometry[0].0,
        (
            257.92,
            132.98666666666668,
            11.133333333333326,
            25.24000000000001,
        ),
    );
    assert_eq!(target_geometry[0].1.len(), 2);
    assert_bbox(
        target_geometry[0].1[0],
        (
            257.94666666666666,
            132.98666666666668,
            11.106666666666683,
            11.786666666666662,
        ),
    );
    assert_bbox(
        target_geometry[0].1[1],
        (
            257.92,
            146.82666666666668,
            10.933333333333337,
            11.400000000000006,
        ),
    );
    let mut target_counts = Vec::new();
    vertical_text_line_child_counts(&target.root, &mut target_counts);
    assert_eq!(
        target_counts,
        vec![2],
        "one shaped owner line, two fallback runs"
    );

    let control = bounded_vertical_table_core(false)
        .build_page_render_tree(0)
        .expect("build no-source legacy control");
    let mut control_counts = Vec::new();
    vertical_text_line_child_counts(&control.root, &mut control_counts);
    assert_eq!(
        control_counts,
        vec![1, 1],
        "failed target preparation must preserve the legacy per-character tree"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_d3_b_target_publishes_one_vertical_glyph_run_per_fallback_leaf() {
    let core = bounded_vertical_table_core(true);
    let render_tree = core
        .build_page_render_tree(0)
        .expect("build D3-B target render tree");
    let mut line_geometry = Vec::new();
    collect_vertical_line_geometry(&render_tree.root, &mut line_geometry);
    assert_eq!(line_geometry.len(), 1);
    assert_eq!(line_geometry[0].1.len(), 2);
    let mut line_ids = Vec::new();
    collect_vertical_line_ids(&render_tree.root, &mut line_ids);
    assert_eq!(line_ids.len(), 1);

    let layer_tree = core
        .build_page_layer_tree(0)
        .expect("build D3-B published layer tree");
    let layer_line = find_layer_source_node(&layer_tree.root, line_ids[0])
        .expect("D2 line must retain its source node in the layer tree");
    let LayerNodeKind::Group { children, .. } = &layer_line.kind else {
        panic!("D2 line must lower to one group");
    };
    assert_eq!(children.len(), 2);
    for (index, child) in children.iter().enumerate() {
        assert_eq!(child.source_node_id, Some(line_ids[0] + index as u32 + 1));
        let LayerNodeKind::Leaf { ops } = &child.kind else {
            panic!("each D2 fallback must remain one direct leaf");
        };
        let [PaintOp::TextRun { run: fallback, .. }, PaintOp::GlyphRun { run: glyph, .. }] =
            ops.as_slice()
        else {
            panic!("each D2 fallback leaf must publish exactly one vertical GlyphRun alternative");
        };
        assert!(fallback.is_vertical);
        assert_eq!(glyph.source.id.0, index as u32);
        assert_eq!(glyph.source.utf8_range.start, 0);
        assert_eq!(glyph.source.utf8_range.end, fallback.text.len() as u32);
        assert_eq!(glyph.source.utf16_range.start, 0);
        assert_eq!(
            glyph.source.utf16_range.end,
            fallback.text.encode_utf16().count() as u32
        );
        assert_eq!(glyph.variant.equivalence_group, format!("text-{index}"));
        assert_eq!(glyph.variant.variant_id, "verticalGlyphRun");
        assert_eq!(glyph.variant.variant_kind, TextVariantKind::GlyphRun);
        assert!(!glyph.variant.is_default_fallback);
        assert_eq!(
            glyph.variant.anchor_op_id.as_deref(),
            Some(glyph.variant.equivalence_group.as_str())
        );
        assert_eq!(glyph.glyph_ids.len(), 1);
        assert_eq!(glyph.positions.len(), 1);
        assert_eq!(glyph.advances.as_ref().map(Vec::len), Some(1));
        assert_eq!(glyph.clusters.len(), 1);
        assert_eq!(glyph.direction, TextDirection::Ltr);
        assert_eq!(glyph.writing_mode, WritingMode::VerticalRl);
        assert_eq!(glyph.orientation, GlyphRunOrientation::VerticalUpright);
        assert_eq!(
            glyph.diagnostics.reason.as_deref(),
            Some("boundedVerticalHwp5TableCellV1")
        );
    }
    assert_eq!(layer_tree.text_sources.entries.len(), 2);
    let vertical_sources = layer_tree
        .text_sources
        .entries
        .iter()
        .filter(|entry| matches!(entry.text.as_str(), "한" | "글"))
        .collect::<Vec<_>>();
    assert_eq!(vertical_sources.len(), 2);
    assert_eq!(vertical_sources[1].id.0, vertical_sources[0].id.0 + 1);

    let mut ops = Vec::new();
    collect_text_ops(&layer_tree.root, &mut ops);
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::TextRun { run, .. } if run.is_vertical))
            .count(),
        2
    );
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
            .count(),
        2,
        "D3-B must publish one GlyphRun alternative for every fallback leaf"
    );
    assert_eq!(layer_tree.resources.font_blob_count(), 1);
    assert_eq!(layer_tree.resources.font_resources().blobs.len(), 1);
    assert_eq!(layer_tree.resources.font_resources().faces.len(), 1);
    let expected_digest = blake3::hash(NOTO).to_hex().to_string();
    let expected_resource_key = format!("font:blake3:{}:{expected_digest}", NOTO.len());
    let published_blob = &layer_tree.resources.font_resources().blobs[0];
    let published_face = &layer_tree.resources.font_resources().faces[0];
    let expected_face = ttf_parser::Face::parse(NOTO, 0).expect("parse fixture face metadata");
    assert_eq!(published_blob.id.0, expected_resource_key);
    assert_eq!(published_face.blob_key, published_blob.id);
    assert_eq!(published_face.face_index, 0);
    assert_eq!(
        published_face.weight_class,
        Some(expected_face.weight().to_number())
    );
    assert_eq!(
        published_face.width_class,
        Some(expected_face.width().to_number())
    );
    assert_eq!(published_face.italic, Some(expected_face.is_italic()));
    rhwp::paint::validate_text_variant_scope(&layer_tree)
        .expect("leaf-scoped vertical alternatives must satisfy variant scope");
    let text_v2 = TextV2Diagnostics::from_layer_tree(&layer_tree);
    let vertical_slots = text_v2
        .slot_diagnostics
        .iter()
        .filter(|slot| matches!(slot.equivalence_group.as_str(), "text-0" | "text-1"))
        .collect::<Vec<_>>();
    assert!(text_v2.fallback_required);
    assert_eq!(vertical_slots.len(), 2);
    assert!(vertical_slots.iter().all(|slot| {
        slot.fallback_present
            && !slot.strict_variant_available
            && slot.fallback_reason.as_deref() == Some("verticalGlyphOrientationAuthorityPending")
    }));
    #[cfg(not(target_arch = "wasm32"))]
    {
        for backend in [
            VariantSelectionBackend::CanvasKit,
            VariantSelectionBackend::CanvasKitBrowser,
            VariantSelectionBackend::NativeSkia,
            VariantSelectionBackend::Svg,
            VariantSelectionBackend::Canvas2D,
        ] {
            let reports = analyze_text_variant_selection(
                &layer_tree,
                TextVariantSelectionOptions {
                    backend,
                    ..TextVariantSelectionOptions::canvaskit()
                },
            );
            let vertical_reports = reports
                .iter()
                .filter(|report| matches!(report.equivalence_group.as_str(), "text-0" | "text-1"))
                .collect::<Vec<_>>();
            assert_eq!(vertical_reports.len(), 2, "{backend:?}");
            if matches!(
                backend,
                VariantSelectionBackend::CanvasKit | VariantSelectionBackend::CanvasKitBrowser
            ) {
                assert!(vertical_reports.iter().all(|report| {
                    report.selected_variant_id.as_deref() == Some("verticalGlyphRun")
                        && report.selected_variant_kind == Some(TextVariantKind::GlyphRun)
                        && !report.fallback_required
                }));
            } else {
                assert!(vertical_reports.iter().all(|report| {
                    report.selected_variant_kind == Some(TextVariantKind::TextRun)
                        && report.fallback_required
                }));
            }
        }

        let browser_plan = analyze_canvaskit_replay_plan(&layer_tree, CanvasKitReplayMode::Default);
        let bounded_reports = browser_plan
            .text_variants
            .iter()
            .filter(|report| matches!(report.equivalence_group.as_str(), "text-0" | "text-1"))
            .collect::<Vec<_>>();
        assert_eq!(bounded_reports.len(), 2);
        assert!(bounded_reports.iter().all(|report| {
            report.selected_variant_id.as_deref() == Some("verticalGlyphRun")
                && report.selected_variant_kind == Some("glyphRun")
                && !report.fallback_required
        }));
        assert_eq!(
            browser_plan
                .items
                .iter()
                .filter(|item| {
                    item.op_type == "glyphRun"
                        && item.detail.as_deref() == Some("selectedVariant=verticalGlyphRun")
                })
                .count(),
            2,
            "both bounded leaves must be direct-required CanvasKit replay items"
        );
        assert_eq!(
            browser_plan
                .items
                .iter()
                .filter(|item| item.op_type == "textRun")
                .count(),
            2,
            "fallback leaves remain published while the variant report chooses GlyphRun"
        );

        for mutation in [
            "wrongProvenance",
            "wrongVariant",
            "verticalSideways",
            "horizontalTuple",
            "glyphTransforms",
        ] {
            let mut malformed = layer_tree.clone();
            mutate_bounded_vertical_glyph_runs(&mut malformed.root, &mut |run| match mutation {
                "wrongProvenance" => {
                    run.diagnostics.reason = Some("untrustedVerticalCandidate".to_string());
                }
                "wrongVariant" => {
                    run.variant.variant_id = "untrustedVerticalGlyphRun".to_string();
                }
                "verticalSideways" => {
                    run.orientation = GlyphRunOrientation::VerticalSideways;
                }
                "horizontalTuple" => {
                    run.writing_mode = WritingMode::HorizontalTb;
                    run.shape_key.writing_mode = WritingMode::HorizontalTb;
                    run.orientation = GlyphRunOrientation::Horizontal;
                }
                "glyphTransforms" => {
                    run.glyph_transforms = Some(vec![GlyphTransform {
                        xx: 1.0,
                        xy: 0.0,
                        yx: 0.0,
                        yy: 1.0,
                        tx: 0.0,
                        ty: 0.0,
                    }]);
                }
                _ => unreachable!(),
            });
            let reports = analyze_text_variant_selection(
                &malformed,
                TextVariantSelectionOptions::canvaskit(),
            );
            let bounded_reports = reports
                .iter()
                .filter(|report| matches!(report.equivalence_group.as_str(), "text-0" | "text-1"))
                .collect::<Vec<_>>();
            assert_eq!(bounded_reports.len(), 2, "{mutation}");
            assert!(
                bounded_reports.iter().all(|report| {
                    report.selected_variant_kind == Some(TextVariantKind::TextRun)
                        && report.fallback_required
                }),
                "{mutation}"
            );
        }
    }

    let mut rejected_tree = render_tree;
    assert!(replace_nth_vertical_run_text(
        &mut rejected_tree.root,
        1,
        &mut 0
    ));
    let rejected_layer = LayerBuilder::new(RenderProfile::Screen).build(&rejected_tree);
    let mut rejected_ops = Vec::new();
    collect_text_ops(&rejected_layer.root, &mut rejected_ops);
    assert_eq!(
        rejected_ops
            .iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref() == Some("boundedVerticalHwp5TableCellV1")))
            .count(),
        0,
        "one mismatched leaf must reject the entire vertical publication"
    );
    assert_eq!(rejected_layer.resources.font_blob_count(), 0);
    assert!(rejected_layer.resources.font_resources().blobs.is_empty());
    assert!(rejected_layer.resources.font_resources().faces.is_empty());

    let accepted_json = layer_tree.to_json();
    let rejected_json = rejected_layer.to_json();
    let accepted: serde_json::Value =
        serde_json::from_str(&accepted_json).expect("parse accepted layer JSON");
    assert_eq!(accepted["textSources"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        accepted["fontResources"]["blobs"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        accepted["fontResources"]["faces"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        accepted["resources"]["fontBlobs"].as_array().map(Vec::len),
        Some(1)
    );
    let mut json_ops = Vec::new();
    collect_layer_json_ops(&accepted["root"], &mut json_ops);
    let vertical_json_glyphs = json_ops
        .iter()
        .filter(|op| {
            op["type"] == "glyphRun"
                && op["diagnostics"]["reason"] == "boundedVerticalHwp5TableCellV1"
        })
        .collect::<Vec<_>>();
    assert_eq!(vertical_json_glyphs.len(), 2);
    for (index, glyph) in vertical_json_glyphs.iter().enumerate() {
        assert_eq!(glyph["source"]["id"], index as u32);
        assert_eq!(
            glyph["variant"]["equivalenceGroup"],
            format!("text-{index}")
        );
        assert_eq!(glyph["variant"]["variantId"], "verticalGlyphRun");
        assert_eq!(glyph["writingMode"], "vertical-rl");
        assert_eq!(glyph["orientation"], "vertical-upright");
        assert_eq!(glyph["glyphIds"].as_array().map(Vec::len), Some(1));
        assert_eq!(glyph["positions"].as_array().map(Vec::len), Some(1));
        assert_eq!(glyph["advances"].as_array().map(Vec::len), Some(1));
        assert_eq!(glyph["clusters"].as_array().map(Vec::len), Some(1));
    }
    let font_payload_bytes = layer_tree
        .resources
        .font_blob_resources()
        .map(|(_, bytes)| bytes.len())
        .sum::<usize>();
    assert_eq!(font_payload_bytes, 2_519_996);
    assert_eq!(accepted_json.len(), 3_375_713);
    assert_eq!(rejected_json.len(), 9_161);
    assert_eq!(accepted_json.len() - rejected_json.len(), 3_366_552);
    println!(
        "{}",
        serde_json::json!({
            "kind": "q4-d3-c-native-publication-receipt",
            "linePublicationAttempts": 1,
            "uniquePreparedSources": layer_tree.resources.font_blob_count(),
            "fontPayloadBytes": font_payload_bytes,
            "acceptedLayerJsonBytes": accepted_json.len(),
            "rejectedLayerJsonBytes": rejected_json.len(),
            "layerJsonIncreaseBytes": accepted_json.len() - rejected_json.len(),
            "fallbackTextRuns": 2,
            "verticalGlyphRuns": vertical_json_glyphs.len(),
            "textV2FallbackSlots": vertical_slots.len(),
        })
    );
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
            PaintOp::TextRun { bbox, run, .. } if run.text == TEXT => Some((*bbox, run.as_ref())),
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
            PaintOp::TextRun { bbox, run, .. } if run.text == TEXT => Some(*bbox),
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
    let page_tree_started = Instant::now();
    std::hint::black_box(
        core.build_page_render_tree(0)
            .expect("warm Q2 default baseline page render tree"),
    );
    let page_tree_warm_elapsed_ns = page_tree_started.elapsed().as_nanos();
    let cold_started = Instant::now();
    std::hint::black_box(
        core.build_page_layer_tree(0)
            .expect("warm Q2 default baseline layer tree"),
    );
    let cold_elapsed_ns = cold_started.elapsed().as_nanos();
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(
            core.build_page_layer_tree(0)
                .expect("build Q2 default baseline layer tree"),
        );
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
        "{{\"kind\":\"q3-e0-default-product-baseline\",\"iterations\":{ITERATIONS},\"pageTreeWarmElapsedNs\":{page_tree_warm_elapsed_ns},\"coldElapsedNs\":{cold_elapsed_ns},\"elapsedNs\":{elapsed_ns},\"layerJsonBytes\":{},\"layerJsonBlake3\":\"{}\",\"canvasKitPlanBytes\":{},\"canvasKitPlanBlake3\":\"{}\"}}",
        layer_json.len(),
        blake3::hash(layer_json.as_bytes()).to_hex(),
        plan_json.len(),
        blake3::hash(plan_json.as_bytes()).to_hex(),
    );
}

#[test]
#[ignore = "Q3-E5 local performance receipt; run serially with RHWP_Q3_E5_CASE"]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q3_e5_local_variable_instance_performance_receipt() {
    use std::time::Instant;

    const ITERATIONS: u32 = 64;
    let case = std::env::var("RHWP_Q3_E5_CASE").unwrap_or_else(|_| "exact-default".to_string());
    let (mut core, char_shape_id) =
        core_with_surface_and_source("가변", Alignment::Left, 0, false, HAPPINESS);
    let axes = match case.as_str() {
        "exact-default" => None,
        "explicit-default" => Some((400.0, 400.0)),
        "interior" => Some((650.0, 650.0)),
        "max" => Some((900.0, 900.0)),
        other => panic!("unsupported RHWP_Q3_E5_CASE: {other}"),
    };
    if let Some((opsz, wght)) = axes {
        let request = serde_json::json!({
            "charShapeId": char_shape_id,
            "languageIndex": 0,
            "mode": "boundedHorizontalLtrV1",
            "axes": [
                { "tag": "opsz", "value": opsz },
                { "tag": "wght", "value": wght }
            ]
        });
        core.set_exact_font_instance_native(&request.to_string())
            .expect("register Q3-E5 measurement instance");
    }

    let page_tree_started = Instant::now();
    std::hint::black_box(
        core.build_page_render_tree(0)
            .expect("warm Q3-E5 variable-font page render tree"),
    );
    let page_tree_warm_elapsed_ns = page_tree_started.elapsed().as_nanos();
    let cold_started = Instant::now();
    std::hint::black_box(
        core.build_page_layer_tree(0)
            .expect("warm Q3-E5 variable-font surface"),
    );
    let cold_elapsed_ns = cold_started.elapsed().as_nanos();
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(
            core.build_page_layer_tree(0)
                .expect("build Q3-E5 variable-font surface"),
        );
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let layer_tree = core
        .build_page_layer_tree(0)
        .expect("build Q3-E5 variable-font receipt");
    let layer_json = layer_tree.to_json();
    let plan = analyze_canvaskit_replay_plan(&layer_tree, CanvasKitReplayMode::Default);
    let plan_json = serde_json::to_string(&plan).expect("serialize Q3-E5 CanvasKit plan");
    let mut ops = Vec::new();
    collect_text_ops(&layer_tree.root, &mut ops);
    let text_runs = ops
        .iter()
        .filter(|op| matches!(op, PaintOp::TextRun { .. }))
        .count();
    let glyph_runs = ops
        .iter()
        .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
        .count();
    let glyph_outlines = ops
        .iter()
        .filter(|op| matches!(op, PaintOp::GlyphOutline { .. }))
        .count();
    match case.as_str() {
        "exact-default" | "explicit-default" => {
            assert_eq!((text_runs, glyph_runs, glyph_outlines), (1, 0, 0));
            assert_eq!(layer_tree.resources.font_blob_count(), 0);
        }
        "interior" | "max" => {
            assert_eq!((text_runs, glyph_runs, glyph_outlines), (1, 1, 1));
            assert_eq!(layer_tree.resources.font_blob_count(), 1);
        }
        _ => unreachable!(),
    }
    println!(
        "{}",
        serde_json::json!({
            "kind": "q3-e5-variable-instance-performance",
            "case": case,
            "iterations": ITERATIONS,
            "pageTreeWarmElapsedNs": page_tree_warm_elapsed_ns,
            "coldElapsedNs": cold_elapsed_ns,
            "elapsedNs": elapsed_ns,
            "layerJsonBytes": layer_json.len(),
            "layerJsonBlake3": blake3::hash(layer_json.as_bytes()).to_hex().to_string(),
            "canvasKitPlanBytes": plan_json.len(),
            "canvasKitPlanBlake3": blake3::hash(plan_json.as_bytes()).to_hex().to_string(),
            "textRuns": text_runs,
            "glyphRuns": glyph_runs,
            "glyphOutlines": glyph_outlines,
            "fontBlobs": layer_tree.resources.font_blob_count(),
            "fontFaces": layer_tree.resources.font_resources().faces.len()
        })
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
            PaintOp::TextRun { bbox, run, .. } if run.text == "가변" => Some(*bbox),
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
            PaintOp::TextRun { bbox, run, .. } if run.text == "가변" => Some(bbox.width),
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
            PaintOp::TextRun { bbox, run, .. } if run.text == "가변" => Some(*bbox),
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

#[cfg(not(target_arch = "wasm32"))]
fn q4_d5_measure_vertical_case(label: &str, core: &DocumentCore) -> serde_json::Value {
    use std::time::Instant;

    const WARMUPS: usize = 20;
    const ITERATIONS: usize = 64;

    let cold_layout_started = Instant::now();
    let cold_render_tree = core
        .build_page_render_tree(0)
        .expect("build Q4-D5 cold render tree");
    let cold_layout_ns = cold_layout_started.elapsed().as_nanos();

    for _ in 0..WARMUPS {
        std::hint::black_box(
            core.build_page_render_tree(0)
                .expect("warm Q4-D5 render tree"),
        );
    }
    let warm_layout_started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(
            core.build_page_render_tree(0)
                .expect("measure Q4-D5 render tree"),
        );
    }
    let warm_layout_ns = warm_layout_started.elapsed().as_nanos();

    for _ in 0..WARMUPS {
        std::hint::black_box(LayerBuilder::new(RenderProfile::Screen).build(&cold_render_tree));
    }
    let layer_started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(LayerBuilder::new(RenderProfile::Screen).build(&cold_render_tree));
    }
    let layer_build_ns = layer_started.elapsed().as_nanos();

    let layer_tree = LayerBuilder::new(RenderProfile::Screen).build(&cold_render_tree);
    for _ in 0..WARMUPS {
        std::hint::black_box(layer_tree.to_json());
    }
    let json_started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(layer_tree.to_json());
    }
    let json_serialize_ns = json_started.elapsed().as_nanos();

    for _ in 0..WARMUPS {
        std::hint::black_box(analyze_canvaskit_replay_plan(
            &layer_tree,
            CanvasKitReplayMode::Default,
        ));
    }
    let replay_plan_started = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(analyze_canvaskit_replay_plan(
            &layer_tree,
            CanvasKitReplayMode::Default,
        ));
    }
    let replay_plan_ns = replay_plan_started.elapsed().as_nanos();

    let layer_json = layer_tree.to_json();
    let replay_plan = analyze_canvaskit_replay_plan(&layer_tree, CanvasKitReplayMode::Default);
    let replay_plan_json =
        serde_json::to_string(&replay_plan).expect("serialize Q4-D5 replay plan");
    let mut ops = Vec::new();
    collect_text_ops(&layer_tree.root, &mut ops);
    let fallback_text_runs = ops
        .iter()
        .filter(|op| matches!(op, PaintOp::TextRun { run, .. } if run.is_vertical))
        .count();
    let vertical_glyph_runs = ops
        .iter()
        .filter(|op| {
            matches!(op, PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref() == Some("boundedVerticalHwp5TableCellV1"))
        })
        .count();
    let font_payload_bytes = layer_tree
        .resources
        .font_blob_resources()
        .map(|(_, bytes)| bytes.len())
        .sum::<usize>();

    serde_json::json!({
        "label": label,
        "warmups": WARMUPS,
        "iterations": ITERATIONS,
        "coldLayoutNs": cold_layout_ns,
        "warmLayoutNs": warm_layout_ns,
        "layerBuildNs": layer_build_ns,
        "jsonSerializeNs": json_serialize_ns,
        "replayPlanNs": replay_plan_ns,
        "layerJsonBytes": layer_json.len(),
        "layerJsonBlake3": blake3::hash(layer_json.as_bytes()).to_hex().to_string(),
        "replayPlanBytes": replay_plan_json.len(),
        "replayPlanBlake3": blake3::hash(replay_plan_json.as_bytes()).to_hex().to_string(),
        "fallbackTextRuns": fallback_text_runs,
        "verticalGlyphRuns": vertical_glyph_runs,
        "fontBlobs": layer_tree.resources.font_blob_count(),
        "fontFaces": layer_tree.resources.font_resources().faces.len(),
        "fontPayloadBytes": font_payload_bytes
    })
}

#[test]
#[ignore = "Q4-D5 local A/B/A performance receipt; run serially in release profile"]
#[cfg(not(target_arch = "wasm32"))]
fn issue_4969_q4_d5_local_vertical_activation_aba_receipt() {
    let a1 = q4_d5_measure_vertical_case(
        "A1-legacy-no-exact-source",
        &bounded_vertical_table_core(false),
    );
    let b =
        q4_d5_measure_vertical_case("B-bounded-exact-source", &bounded_vertical_table_core(true));
    let a2 = q4_d5_measure_vertical_case(
        "A2-legacy-no-exact-source",
        &bounded_vertical_table_core(false),
    );

    assert_eq!(a1["layerJsonBlake3"], a2["layerJsonBlake3"]);
    assert_eq!(a1["replayPlanBlake3"], a2["replayPlanBlake3"]);
    for control in [&a1, &a2] {
        assert_eq!(control["fallbackTextRuns"], 2);
        assert_eq!(control["verticalGlyphRuns"], 0);
        assert_eq!(control["fontBlobs"], 0);
        assert_eq!(control["fontFaces"], 0);
        assert_eq!(control["fontPayloadBytes"], 0);
    }
    assert_eq!(b["fallbackTextRuns"], 2);
    assert_eq!(b["verticalGlyphRuns"], 2);
    assert_eq!(b["fontBlobs"], 1);
    assert_eq!(b["fontFaces"], 1);
    assert_eq!(b["fontPayloadBytes"], NOTO.len());

    println!(
        "{}",
        serde_json::json!({
            "kind": "q4-d5-vertical-activation-aba-performance",
            "fixture": "public-synthetic-HWP5-code2-one-cell-one-paragraph-one-line-one-column-pure-CJK",
            "fontBytes": NOTO.len(),
            "fontBlake3": blake3::hash(NOTO).to_hex().to_string(),
            "cases": [a1, b, a2]
        })
    );
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
