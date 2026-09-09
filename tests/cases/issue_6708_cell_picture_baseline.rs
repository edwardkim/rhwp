//! A small fallback TAC picture aligns its own baseline within the stored line.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn sample() -> Document {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/tac-img-02.hwp"),
    )
    .unwrap();
    rhwp::parser::parse_document(&bytes).unwrap()
}

fn picture(core: &DocumentCore) -> BoundingBox {
    fn find(node: &RenderNode) -> Option<BoundingBox> {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            return Some(node.bbox);
        }
        node.children.iter().find_map(find)
    }
    find(&core.build_page_render_tree(0).unwrap().root).unwrap()
}

fn font_document(size: i32) -> Document {
    let mut doc = sample();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
        panic!("cover table");
    };
    let para = &mut table.cells[5].paragraphs[0];
    let cs = &mut doc.doc_info.char_shapes[para.char_shapes[0].char_shape_id as usize];
    cs.base_size = size;
    cs.raw_data = None;
    let seg = &mut para.line_segs[0];
    seg.line_height = size;
    seg.text_height = size;
    seg.baseline_distance = size * 85 / 100;
    seg.line_spacing = size * 60 / 100;
    doc
}

fn core_from(doc: Document) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    core
}

fn font_variant(size: i32) -> DocumentCore {
    core_from(font_document(size))
}

#[test]
fn original_cover_picture_matches_hancom_baseline_without_page_growth() {
    let core = font_variant(19800);
    assert_eq!(core.page_count(), 66);
    let bbox = picture(&core);
    // Hancom 12.0.0.4605 PDF y=687.21pt on an 841pt page. Normalize page size,
    // rather than treating rounded PDF media dimensions as exact A4 dimensions.
    let height = core.build_page_render_tree(0).unwrap().root.bbox.height;
    let expected_y = 687.21 * height / 841.0;
    assert!(
        (bbox.y - expected_y).abs() < 0.3,
        "picture y={} expected={expected_y}",
        bbox.y
    );
}

#[test]
fn varying_font_height_follows_independent_hancom_oracle() {
    // Same picture and cell, only source font/line metrics changed. Hancom MCP
    // engine 2020 rendered the three-section control document on 2026-09-06.
    for (size, pdf_y_pt) in [(19800, 687.21), (10000, 652.9275), (6000, 638.9025)] {
        let core = font_variant(size);
        let height = core.build_page_render_tree(0).unwrap().root.bbox.height;
        let expected_y = pdf_y_pt * height / 841.0;
        let bbox = picture(&core);
        assert!(
            (bbox.y - expected_y).abs() < 0.3,
            "font {size}: picture y={} expected={expected_y}",
            bbox.y
        );
    }
}

#[test]
fn synthetic_or_invalid_baselines_keep_the_existing_top_alignment() {
    use rhwp::model::paragraph::LineSeg;
    for (tag, baseline) in [
        (LineSeg::TAG_IMPLEMENTATION_PROPERTY, 16830),
        (0, 0),
        (0, 19801),
    ] {
        let mut doc = font_document(19800);
        let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
            panic!("cover table");
        };
        let seg = &mut table.cells[5].paragraphs[0].line_segs[0];
        seg.tag = tag;
        seg.baseline_distance = baseline;
        let bbox = picture(&core_from(doc));
        assert!(
            (bbox.y - 750.6266666667).abs() < 0.1,
            "{tag}/{baseline}: {bbox:?}"
        );
    }
}

#[test]
fn a_picture_filling_the_line_does_not_get_an_extra_offset() {
    let mut doc = font_document(19800);
    let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
        panic!("cover table");
    };
    for control in &mut table.cells[5].paragraphs[0].controls {
        if let Control::Picture(pic) = control {
            pic.common.height = 19800;
        }
    }
    let bbox = picture(&core_from(doc));
    assert!((bbox.y - 750.6266666667).abs() < 0.1, "{bbox:?}");
}

#[test]
fn same_line_pictures_do_not_accumulate_the_baseline_offset() {
    let mut doc = font_document(19800);
    let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
        panic!("cover table");
    };
    let para = &mut table.cells[5].paragraphs[0];
    let pic = para
        .controls
        .iter()
        .find(|c| matches!(c, Control::Picture(_)))
        .unwrap()
        .clone();
    para.controls.push(pic);
    fn collect(node: &RenderNode, ys: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            ys.push(node.bbox.y);
        }
        for child in &node.children {
            collect(child, ys);
        }
    }
    let page = core_from(doc).build_page_render_tree(0).unwrap();
    let mut ys = Vec::new();
    collect(&page.root, &mut ys);
    assert_eq!(ys.len(), 2);
    assert!((ys[0] - ys[1]).abs() < 0.01, "{ys:?}");
}
