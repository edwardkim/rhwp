//! 공개 입력 생성 → Hancom 2020 저장 → 같은 입력의 독립 Print PDF를 근거로 한다.
//! 저장 셀의 가로 조각/배경 배치와 편집 후 자동/명시적 나누기를 최종 RenderTree로 검사한다.
//! See tests/fixtures/issue_7618_stored_frames/README.md for generation and visual evidence.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue_7618_stored_frames")
        .join(format!("{name}.hwpx"));
    DocumentCore::from_bytes(&std::fs::read(path).expect("public fixture")).expect("parse")
}
fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}
fn cell_frame(name: &str) -> (BoundingBox, BoundingBox, BoundingBox) {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "one-page independent Print");
    let page = core.build_page_render_tree(0).expect("page");
    let all = nodes(&page.root);
    let outer = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(_)))
        .expect("outer cell")
        .bbox;
    let image = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Image(i) if i.cell_context.is_some()))
        .expect("cell image")
        .bbox;
    let nested = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.cell_context.is_some()))
        .expect("nested table")
        .bbox;
    for content in [image, nested] {
        assert!(
            content.y >= outer.y && content.y + content.height <= outer.y + outer.height,
            "frame remains in owning cell: {content:?} / {outer:?}"
        );
    }
    (outer, image, nested)
}
#[test]
fn signed_flow_offsets_share_the_centered_frame_and_background_keeps_its_contract() {
    let (_, negative, negative_title) = cell_frame("cell-negative");
    let (_, zero, zero_title) = cell_frame("cell-zero");
    let (_, positive, positive_title) = cell_frame("cell-positive");
    assert!(
        (negative.y - zero.y).abs() < 0.5,
        "Hancom negative and zero share flow start"
    );
    assert!(
        (negative_title.y - zero_title.y).abs() < 0.5,
        "same stored title origin"
    );
    assert!(
        positive_title.y > zero_title.y,
        "positive offset advances centered title too"
    );
    assert!(
        positive.y > zero.y,
        "positive offset advances image in same frame"
    );
    assert!(
        negative.y + negative.height <= negative_title.y,
        "flow image precedes title"
    );
    let (_, background, background_title) = cell_frame("cell-overlay");
    assert!(
        background_title.y < zero_title.y,
        "background does not reserve flow-image height"
    );
    assert!(
        background.y < background_title.y,
        "signed background positioning is retained"
    );
}
#[test]
fn genuine_horizontal_fragments_do_not_reset_the_following_nested_table() {
    let core = load("cell-fragments");
    let table = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .expect("outer table");
    let segments = &table.cells[0].paragraphs[0].line_segs;
    assert_eq!(segments.len(), 2, "genuine saved horizontal fragments");
    assert_eq!(segments[0].vertical_pos, segments[1].vertical_pos);
    assert!(segments[1].column_start > segments[0].column_start);
    let (_, image, title) = cell_frame("cell-fragments");
    assert!(
        image.y + image.height <= title.y,
        "title belongs after the image, not the second fragment"
    );
}
fn body_lines(name: &str, pages: u32) -> Vec<(u32, usize, f64)> {
    let core = load(name);
    assert_eq!(
        core.page_count(),
        pages,
        "independent Print page ownership: {name}"
    );
    let mut lines = Vec::new();
    let mut text = String::new();
    for page_index in 0..pages {
        let page = core.build_page_render_tree(page_index).expect("page");
        for node in nodes(&page.root) {
            match &node.node_type {
                RenderNodeType::TextLine(line) => {
                    if let Some(para) = line.para_index {
                        lines.push((page_index, para, node.bbox.y));
                    }
                }
                RenderNodeType::TextRun(run) => text.push_str(&run.text),
                _ => {}
            }
        }
    }
    let expected_text: String = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.text.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    let actual_text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(
        actual_text, expected_text,
        "all body content preserved in order without duplication"
    );
    for expected in ["Original flow", "Automatic boundary", "Tail preserved"] {
        assert_eq!(
            text.matches(expected).count(),
            1,
            "preserve text once: {expected}"
        );
    }
    for para in 0..6 {
        assert!(
            lines.iter().any(|(_, p, _)| *p == para),
            "preserve paragraph {para}, including blanks"
        );
    }
    lines
}
#[test]
fn automatic_boundary_reflows_and_keeps_empty_line_ownership() {
    let lines = body_lines("body-reflow", 1);
    for para in 0..5 {
        let y = lines.iter().find(|(_, p, _)| *p == para).unwrap().2;
        let next = lines.iter().find(|(_, p, _)| *p == para + 1).unwrap().2;
        assert!(next > y, "paragraph order and blank-line occupancy");
    }
}
#[test]
fn authored_page_break_and_saved_original_keep_their_page_ownership() {
    for name in ["body-original", "body-explicit"] {
        let lines = body_lines(name, 2);
        assert!(lines.iter().any(|(page, para, _)| *page == 0 && *para == 3));
        assert!(lines.iter().any(|(page, para, _)| *page == 1 && *para == 4));
        assert!(lines.iter().any(|(page, para, _)| *page == 1 && *para == 5));
    }
}
#[test]
fn multiline_reflow_does_not_restore_an_old_automatic_boundary() {
    let lines = body_lines("body-single-symbol", 1);
    assert_eq!(
        lines.iter().filter(|(_, p, _)| *p == 1).count(),
        3,
        "Print: symbol line and two body lines"
    );
}
#[test]
fn single_symbol_keeps_natural_leading_space() {
    let core = load("body-single-symbol");
    let page = core.build_page_render_tree(0).expect("page");
    let all = nodes(&page.root);
    let symbol = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.trim() == "-"))
        .expect("symbol line");
    let RenderNodeType::TextRun(run) = &symbol.node_type else {
        unreachable!()
    };
    assert_eq!(
        run.style.extra_word_spacing, 0.0,
        "leading spaces are not inter-word justification slots"
    );
    let symbol_line = all
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(1)))
        .expect("symbol owner");
    assert!(
        symbol.bbox.width < symbol_line.bbox.width / 2.0,
        "symbol stays at natural left spacing, not distributed across line"
    );
}
