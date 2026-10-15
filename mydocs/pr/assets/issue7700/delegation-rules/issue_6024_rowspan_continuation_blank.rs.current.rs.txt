//! [#6024] 쪽을 넘어간 rowspan 병합 셀의 내용을 연속 조각이 다시 그리던 중복
//! 가드.
//!
//! 10857(위임전결규정) — 세로 병합 밴드가 쪽 경계에서 갈라질 때
//! 한글은 병합 셀 내용(일련번호·단위업무 라벨)을 밴드가 시작한 조각에만 그리고
//! 연속 조각은 빈 칸으로 둔다(한글 2020 PDF 전문 계수: '사무분장조정'·
//! '조사관교육훈련' 각 1회). rhwp 는 연속 조각에 처음부터 재렌더했다 — 근인은
//! straddle 높이-유닛 컷(is_rowbreak_straddle)의 두 게이트 구멍: ① block-split
//! 조각에서 **컷이 비어 있는 쪽 경계**(행 경계 컷)의 straddle 셀은 split-block
//! 범위에도, straddle 경로(!is_block_split 게이트)에도 안 잡힘(p9 RowBreak 표)
//! ② CellBreak 표는 match 에서 제외돼 있었음(p67). 내용이 첫 조각에 다 들어가지
//! 않는 다문단 라벨 꼬리는 유닛 컷 산술이 그대로 연속 조각에 흘린다(76076
//! p18/p19 계약, issue_3820 핀이 가드).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

// #5679(PR #6043)와 같은 재현물 — 같은 경로·같은 내용이라 두 PR 이 어느 순서로
// 랜딩해도 충돌 없이 합류한다.
const SAMPLE: &str = "samples/issue5679/10857_delegation_rules.hwp";

fn body_text(node: &RenderNode, in_body: bool, out: &mut String) {
    if !node.visible || node.editor_only {
        return;
    }
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    if in_body {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
    }
    for child in &node.children {
        body_text(child, in_body, out);
    }
}

#[test]
fn issue_6024_band_continuation_does_not_redraw_merged_cell_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6024 fixture");
    let mut text = String::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("전쪽 렌더 성공");
        body_text(&tree.root, false, &mut text);
    }
    for needle in ["사무분장조정", "조사관교육훈련"] {
        assert_eq!(
            text.matches(needle).count(),
            1,
            "독립 Print에 한 번 있는 병합 셀 라벨 {needle}의 누락·중복"
        );
    }
    // 같은 원본의 tracked 한컴 Print PDF는 103쪽이다. 이전 rhwp 출력의
    // 105쪽을 정상값으로 고정하지 않는다. 이 PASS도 전쪽 시각 적격성을 대신하지 않는다.
    // pdf/semanticist21-20261008/issue5679-10857_delegation_rules-de43607f0507-2020.pdf
    assert_eq!(core.page_count(), 103, "독립 Print의 전체 쪽수와 다릅니다");
}
