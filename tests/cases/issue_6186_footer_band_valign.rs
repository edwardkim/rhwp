//! [Issue #6186] 꼬리말 `vertAlign="BOTTOM"` 미적용으로 쪽번호가 밴드 위쪽에 붙어
//! 21.8px 올라간다 — 겹쳐 놓인 글상자와 두 줄로 갈라진다 (156755659 2쪽).
//!
//! 이 문서는 쪽번호를 **두 곳**에서 그린다 — 꼬리말(`2 - ` + `autoNum PAGE`)과, 같은
//! 자리에 겹쳐 놓은 글상자(`2 - 2`). 한글은 둘이 포개져 한 줄로 보인다.
//!
//! 근인: `layout_header_footer_paragraphs` 가 `y_offset = area.y` 로 시작하고 세로
//! 정렬을 아예 보지 않았다. 문서의 `<hp:subList vertAlign="BOTTOM">` 은
//! LIST_HEADER `list_attr` bit 21~22 에 실려 있다(표 셀과 같은 규약).
//!
//! 밴드 아래끝이 `footer_area` 의 아래끝이 아니라는 점도 함께 고친다 — `footer_area`
//! 는 본문 하단부터 **꼬리말 여백 선**까지라 아래쪽 여백만큼 더 길다(이 문서 56.7px
//! 대 실제 37.8px).
//!
//! 잠금은 문서 자신의 증인을 쓴다 — 두 쪽번호가 **같은 줄**에 있어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6186/156755659_defense_press_release.hwpx";
/// 결함이 나타나는 쪽(0-based). 1쪽도 같은 양만큼 어긋난다.
const PAGE: u32 = 1;

#[test]
fn issue_6186_footer_page_number_sits_on_the_textbox_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 2 render tree");
    let footer = find_footer(&page.root).expect("Footer 노드");

    let in_footer = first_page_number_run(footer).expect("꼬리말 쪽번호 줄");
    let mut outside = Vec::new();
    collect_page_number_runs_outside_footer(&page.root, &mut outside);
    let in_textbox = outside.first().copied().expect("글상자 쪽번호 줄");

    // 세로로 겹쳐야 한다 — 종전에는 꼬리말이 21.8px 위에 있어 완전히 분리됐다.
    let overlap = (in_footer.1).min(in_textbox.1) - (in_footer.0).max(in_textbox.0);
    assert!(
        overlap > 5.0,
        "꼬리말 쪽번호({:.1}..{:.1})와 글상자 쪽번호({:.1}..{:.1})가 두 줄로 갈라졌다 \
         (겹침 {overlap:.1}px)",
        in_footer.0,
        in_footer.1,
        in_textbox.0,
        in_textbox.1,
    );
    // 밴드 맨 위에 붙어 있지 않아야 한다.
    assert!(
        in_footer.0 > footer.bbox.y + 5.0,
        "꼬리말 줄이 밴드 맨 위({:.1})에 그대로 붙어 있다: {:.1}",
        footer.bbox.y,
        in_footer.0,
    );
}

fn find_footer(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Footer) {
        return Some(node);
    }
    node.children.iter().find_map(find_footer)
}

/// 쪽번호 문자열을 가진 첫 `TextRun` 의 (위끝, 아래끝).
fn first_page_number_run(node: &RenderNode) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.starts_with("2 -") {
            return Some((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    node.children.iter().find_map(first_page_number_run)
}

fn collect_page_number_runs_outside_footer(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Footer) {
        return;
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.starts_with("2 -") {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    for child in &node.children {
        collect_page_number_runs_outside_footer(child, out);
    }
}
