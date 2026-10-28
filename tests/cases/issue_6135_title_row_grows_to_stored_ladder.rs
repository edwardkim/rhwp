//! [Issue #6135] 표 제목 행(16pt)을 내용에 맞게 키우지 않아, 바로 아래 행
//! "(단위: 명, %, %p)" 칸이 제목 글자 아랫부분과 오른쪽 `❙` 를 덮어 가린다
//! (156544683 8쪽).
//!
//! 근인은 **측정↔페인트 비대칭**이다. 셀 안 첫 줄의 저장 `vertical_pos`(700HU=9.3px)를
//! layout 은 존중해 줄을 그만큼 내려 그리는데(렌더 692.7 = 셀 상단 680.3 + pad 3.0 +
//! vpos 9.3), 측정기는 줄높이 합만 세어 행이 그만큼 모자랐다. 그래서 다음 행 칸이
//! 제목 글자 위에 채우기와 함께 그려졌다.
//!
//! 수정: 저장 ladder extent(`vpos + line_height`)가 줄높이 합과 **선언 셀높이**를
//! 모두 넘을 때 그 extent 를 내용 높이로 쓴다. 첫 줄이 셀 안에서 시작할 것을 함께
//! 요구한다 — 별지 서식(74312)처럼 저장 vpos 가 셀 상대가 아니라 **표 누적**인
//! 문서가 있어, 그 값을 쓰면 행이 수백 px 로 부푼다.
//!
//! 한글 2024 PDF(2-up, 축척 0.7099) 실측: 제목 글줄 위끝 → "(단위…)" 글줄 위끝이
//! 23.8px. 수정 후 rhwp 는 23.4px(수정 전 14.0px, 겹침).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6135/156544683_title_row_underfit.hwp";

#[test]
fn issue_6135_title_row_grows_so_the_next_row_does_not_cover_it() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let mut cells = cells_by_top(&page.root);
    cells.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(cells.len() >= 2, "표의 앞 두 행을 찾지 못했다: {cells:?}");
    let (title_top, title_bottom) = cells[0];
    let (unit_top, _) = cells[1];

    let title_line_bottom = text_lines(&page.root)
        .into_iter()
        .filter(|(top, _)| *top >= title_top - 0.5 && *top < unit_top)
        .map(|(_, bottom)| bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(title_line_bottom.is_finite(), "제목 글줄을 찾지 못했다");

    assert!(
        title_line_bottom <= title_bottom + 0.5,
        "제목 글줄이 자기 칸을 넘었다 — 칸 {title_top:.1}..{title_bottom:.1}, \
         글줄 아래끝 {title_line_bottom:.1}"
    );
    assert!(
        title_line_bottom <= unit_top + 0.5,
        "다음 행 칸이 제목 글자를 덮는다 — 제목 글줄 아래끝 {title_line_bottom:.1}, \
         다음 칸 위끝 {unit_top:.1}"
    );
}

/// 페이지의 표 셀 (위끝, 아래끝).
fn cells_by_top(node: &RenderNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        out.extend(cells_by_top(child));
    }
    out
}

/// 페이지의 텍스트 줄 (위끝, 아래끝).
fn text_lines(node: &RenderNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        out.extend(text_lines(child));
    }
    out
}
