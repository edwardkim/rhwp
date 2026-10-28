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

// Source HWP table44: default inline image alignment uses its 85% ascent.
// Compare it with the actual containing row, rather than pinning a page origin.
fn picture_row(core: &DocumentCore) -> (BoundingBox, BoundingBox) {
    fn find(node: &RenderNode) -> Option<(BoundingBox, BoundingBox)> {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            let image = node
                .children
                .iter()
                .find(|n| matches!(n.node_type, RenderNodeType::Image(_)));
            let row = node
                .children
                .iter()
                .find(|n| matches!(n.node_type, RenderNodeType::TextLine(_)));
            if let (Some(image), Some(row)) = (image, row) {
                return Some((image.bbox, row.bbox));
            }
        }
        node.children.iter().find_map(find)
    }
    find(&core.build_page_render_tree(0).unwrap().root).unwrap()
}
fn assert_source_baseline(core: &DocumentCore, source_baseline_hu: i32) {
    let (image, row) = picture_row(core);
    let image_baseline = image.y + image.height * 0.85;
    let source_baseline = row.y + f64::from(source_baseline_hu) * 96.0 / 7200.0;
    assert!(
        (image_baseline - source_baseline).abs() < 0.3,
        "source row baseline and image ascent: {image_baseline} vs {source_baseline}"
    );
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
    assert_source_baseline(&core, 19800 * 85 / 100);
}

#[test]
fn varying_font_height_follows_independent_hancom_oracle() {
    // Same picture and cell, only source font/line metrics changed. Hancom MCP
    // engine 2020 rendered the three-section control document on 2026-09-06.
    let original = picture(&font_variant(19800));
    for (size, pdf_y_pt) in [(19800, 687.21), (10000, 652.9275), (6000, 638.9025)] {
        let core = font_variant(size);
        assert_eq!(
            core.page_count(),
            66,
            "font changes must preserve source page ownership"
        );
        assert_source_baseline(&core, size * 85 / 100);
        let height = core.build_page_render_tree(0).unwrap().root.bbox.height;
        // The independent three-font Print gives the relative movement as the
        // containing row changes height, not an absolute page-position pin.
        let expected_delta = (687.21 - pdf_y_pt) * height / 841.0;
        let actual_delta = original.y - picture(&core).y;
        assert!(
            (actual_delta - expected_delta).abs() < 0.3,
            "font {size}: relative movement {actual_delta}, independent Print {expected_delta}"
        );
    }
}

#[test]
fn computed_baseline_preserves_source_alignment_and_invalid_baselines_keep_top_alignment() {
    use rhwp::model::paragraph::LineSeg;
    // A computed row with a valid baseline uses the same alignment as the
    // independent Hancom-qualified stored row above. The implementation tag
    // alone is not evidence that the row has no baseline.
    let qualified = picture(&font_variant(19800));
    let mut computed = font_document(19800);
    let Control::Table(table) = &mut computed.sections[0].paragraphs[1].controls[0] else {
        panic!("cover table");
    };
    table.cells[5].paragraphs[0].line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    let actual = picture(&core_from(computed));
    assert!(
        (actual.y - qualified.y).abs() < 0.01,
        "{actual:?} vs {qualified:?}"
    );
    for (tag, baseline) in [(0, 0), (0, 19801)] {
        let mut doc = font_document(19800);
        let Control::Table(table) = &mut doc.sections[0].paragraphs[1].controls[0] else {
            panic!("cover table");
        };
        let seg = &mut table.cells[5].paragraphs[0].line_segs[0];
        seg.tag = tag;
        seg.baseline_distance = baseline;
        let (image, row) = picture_row(&core_from(doc));
        assert!(
            (image.y - row.y).abs() < 0.1,
            "{tag}/{baseline}: image must start at its row"
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
    let (image, row) = picture_row(&core_from(doc));
    assert!(
        (image.y - row.y).abs() < 0.1,
        "a full-height image starts at its own row"
    );
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
