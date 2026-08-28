//! Issue #4969 W10-Q2-D4-B: the first product lane activates atomically.

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::style::Alignment;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
// `ᄒᆞᆫ글`은 legacy 제품명 display projection 대상이므로 최초 direct-text lane의
// 양성 fixture로 쓰지 않는다. 이 문자열은 같은 옛한글 자모 shaping을 요구하지만
// model text와 replay text가 동일하다.
const TEXT: &str = "ᄒᆞᆫ말";

fn core_with_surface(text: &str, alignment: Alignment, char_border_fill_id: u16) -> DocumentCore {
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
    document.sections[0].paragraphs = vec![paragraph];
    document.sections[0].section_def.page_def.width = 50_000;
    document.sections[0].section_def.page_def.height = 100_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);
    core.register_exact_font_source_native(char_shape_id, 0, SOURCE_HAN, 0)
        .expect("register exact old-Hangul source");
    core
}

fn collect_text_ops<'a>(node: &'a LayerNode, ops: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_text_ops(child, ops);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_text_ops(child, ops),
        LayerNodeKind::Leaf { ops: leaf_ops } => ops.extend(
            leaf_ops
                .iter()
                .filter(|op| matches!(op, PaintOp::TextRun { .. } | PaintOp::GlyphRun { .. })),
        ),
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_b_one_line_run_publishes_one_common_alternative() {
    let core = core_with_surface(TEXT, Alignment::Left, 0);
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
        let core = core_with_surface(text, alignment, char_border_fill_id);
        let layer_tree = core
            .build_page_layer_tree(0)
            .expect("build rejected surface layer tree");
        let mut ops = Vec::new();
        collect_text_ops(&layer_tree.root, &mut ops);
        assert!(ops.iter().any(|op| {
            matches!(op, PaintOp::TextRun { run, .. } if run.text == text)
        }));
        assert!(!ops.iter().any(|op| {
            matches!(op, PaintOp::GlyphRun { run, .. }
                if run.diagnostics.reason.as_deref()
                    == Some("q2CommonShapingCondensedDrawProjectionV1"))
        }));
        assert_eq!(layer_tree.resources.font_blob_count(), 0);
    }
}
