//! [#6024] 쪽을 넘어간 rowspan 병합 셀의 내용을 연속 조각이 다시 그리던 중복
//! 가드.
//!
//! 10857(위임전결규정, 105쪽 HWP5) — 세로 병합 밴드가 쪽 경계에서 갈라질 때
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

fn count_runs_containing(node: &RenderNode, needle: &str) -> usize {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) => 1,
        _ => 0,
    };
    own + node
        .children
        .iter()
        .map(|child| count_runs_containing(child, needle))
        .sum::<usize>()
}

#[test]
fn issue_6024_band_continuation_does_not_redraw_merged_cell_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6024 fixture");
    assert_eq!(core.page_count(), 105, "흐름 전체 쪽수 고정");

    // RowBreak 표: 밴드 10('사무분장 조정')은 p8 에서 시작 — p8 이 1회 소유,
    // 연속 조각(p9)은 빈 칸이어야 한다.
    let p8 = core.build_page_render_tree(7).expect("render p8");
    assert_eq!(
        count_runs_containing(&p8.root, "사무분장 조정"),
        1,
        "p8: 밴드 시작 조각이 라벨을 1회 소유해야 한다"
    );
    let p9 = core.build_page_render_tree(8).expect("render p9");
    assert_eq!(
        count_runs_containing(&p9.root, "사무분장 조정"),
        0,
        "p9: 연속 조각이 병합 셀 라벨을 재렌더하면 안 된다 (한글: 빈 칸)"
    );

    // CellBreak 표: '조사관 교육훈련' 밴드는 p66 시작 — p67 연속 조각은 빈 칸.
    let p66 = core.build_page_render_tree(65).expect("render p66");
    assert_eq!(
        count_runs_containing(&p66.root, "교육훈련"),
        1,
        "p66: CellBreak 밴드 시작 조각이 라벨을 1회 소유해야 한다"
    );
    let p67 = core.build_page_render_tree(66).expect("render p67");
    assert_eq!(
        count_runs_containing(&p67.root, "교육훈련"),
        0,
        "p67: CellBreak 연속 조각이 병합 셀 라벨을 재렌더하면 안 된다"
    );
}
