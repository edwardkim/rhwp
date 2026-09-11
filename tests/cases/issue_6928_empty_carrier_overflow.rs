//! #6928: an empty trailing paragraph is not painted body overflow.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{
    BoundingBox, PageNode, RenderNode, RenderNodeType, TextLineNode, TextRunNode,
};
use rhwp::renderer::TextStyle;

fn run(text: &str) -> RenderNode {
    RenderNode::new(
        3,
        RenderNodeType::TextRun(TextRunNode {
            text: text.to_string(),
            style: TextStyle::default(),
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 16.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        }),
        BoundingBox::new(10.0, 95.0, 30.0, 20.0),
    )
}

fn root(children: Vec<RenderNode>) -> RenderNode {
    let mut line = RenderNode::new(
        2,
        RenderNodeType::TextLine(TextLineNode::new(20.0, 16.0)),
        BoundingBox::new(10.0, 95.0, 80.0, 20.0),
    );
    line.children = children;
    let mut body = RenderNode::new(
        1,
        RenderNodeType::Body { clip_rect: None },
        BoundingBox::new(0.0, 0.0, 100.0, 100.0),
    );
    body.children.push(line);
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 200.0,
            height: 200.0,
            section_index: 0,
        }),
        BoundingBox::new(0.0, 0.0, 200.0, 200.0),
    );
    root.children.push(body);
    root
}

fn overflow(children: Vec<RenderNode>) -> usize {
    scan_page(0, &root(children), 1, &AnomalyOptions::default())
        .overflow
        .len()
}

#[test]
fn empty_carrier_is_not_body_overflow() {
    for shade in [0x00ff_ffff, 0xffff_ffff, 0] {
        let mut carrier = run("");
        if let RenderNodeType::TextRun(run) = &mut carrier.node_type {
            run.style.shade_color = shade;
        }
        assert_eq!(overflow(vec![carrier]), 0, "shade={shade:x}");
    }
}

#[test]
fn visible_whitespace_and_display_runs_still_report_overflow() {
    for text in ["text", " ", "\t"] {
        assert_eq!(overflow(vec![run(text)]), 1, "{text:?}");
    }
    let mut display = run("");
    if let RenderNodeType::TextRun(run) = &mut display.node_type {
        run.display_text = Some("1".to_string());
    }
    assert_eq!(overflow(vec![display]), 1);
}

#[test]
fn decoration_and_object_children_are_not_empty_carriers() {
    let mut shaded = run("");
    if let RenderNodeType::TextRun(run) = &mut shaded.node_type {
        run.style.shade_color = 0xffff00;
    }
    assert_eq!(overflow(vec![shaded]), 1);
    let mut bordered = run("");
    if let RenderNodeType::TextRun(run) = &mut bordered.node_type {
        run.border_fill_id = 1;
    }
    assert_eq!(overflow(vec![bordered]), 1);
    let object = RenderNode::new(
        4,
        RenderNodeType::TextBox,
        BoundingBox::new(10.0, 95.0, 30.0, 20.0),
    );
    assert_eq!(overflow(vec![run(""), object.clone()]), 1);
    let mut carrier = run("");
    carrier.children.push(object);
    assert_eq!(overflow(vec![carrier]), 1);
    assert_eq!(overflow(vec![]), 1);
}

#[test]
fn semiconductor_sample_keeps_five_pages_without_empty_tail_overflow() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6928/148769979_274 반도체포럼 개최(이귀남 최종).hwp"
    ))
    .expect("registered sample");
    let doc = DocumentCore::from_bytes(&bytes).expect("parse sample");
    assert_eq!(doc.page_count(), 5);
    let opts = AnomalyOptions {
        overflow_tolerance_px: 2.0,
        ..AnomalyOptions::default()
    };
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render sample");
        let result = scan_page(page, &tree.root, doc.page_count(), &opts);
        assert!(
            result.overflow.iter().all(|item| item.over_bottom <= 2.0),
            "page {}: {:?}",
            page + 1,
            result.overflow
        );
        if page == 4 {
            assert!(result
                .overflow
                .iter()
                .any(|item| { item.node_type == "Table" && item.over_right > 2.0 }));
        }
    }
}
