//! Issue #5846: 혼재 문단(텍스트+중첩 표) 셀의 컷 조각이 꼬리 행을 반쪽만 담지 않는다.
//!
//! `samples/task2097/75544_pii_bunseok.hwpx` 59쪽(0-based 58)의 `pi=527` 은
//! 1×1 바깥 표 → 셀 → 2행 중첩 표 구조다. 컷은 바깥 셀의 유닛 45개까지를 59쪽에
//! 주고(`end_cut=[45]`), 60쪽이 유닛 45 부터 이어받는다(`start_cut=[45]`).
//!
//! `calc_nested_split_rows` 는 연속 조각의 `start_row` 를 **행 처음부터** 다시
//! 그린다. 그래서 컷 조각이 남긴 부분 행은 다음 쪽이 반드시 통째로 재렌더한다.
//! 그런데 종전 탈락 규칙은 슬라이버가 `min(행높이*0.5, 10.0)` 보다 클 때 부분 행을
//! 그대로 뒀다 — 2행 중첩 표(행높이 650.9 / 767.6)에 가시 688.0 이 오면 행 1 이
//! 35.3px 스텁으로 붙고, 그 안의 25문단 전체가 그 스텁에 그려졌다.
//!
//! 실측(수정 전): 59쪽 `<text>` 1,083개 중 549개가 본문 하한 1,046.9px 밖
//! (최하단 y=1,725.6px), `LAYOUT_OVERFLOW_CELL` 3줄. 같은 내용은 60쪽이 이미
//! 온전히 그린다 — 즉 순수 중복이다.
//!
//! 정답지 `pdf/task2097/75544_pii_bunseok-2020.pdf`(한글 2020) 59쪽은
//! `③ 신용도판단정보 > 채무불이행정보` 표까지만 그리고 상자를 닫으며,
//! `단기연체정보` 는 60쪽 첫 항목이다. 쪽 총수는 양쪽 66.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/75544_pii_bunseok.hwpx";
/// 한글 2020 정답지 쪽수.
const ORACLE_PAGES: u32 = 66;
/// 결함 쪽(0-based). 정답지 59쪽.
const CUT_PAGE: u32 = 58;
/// 연속 쪽(0-based). 정답지 60쪽.
const CONT_PAGE: u32 = 59;
/// 60쪽 첫 항목 — 59쪽에는 나오면 안 된다.
const CONT_FIRST_ITEM: &str = "단기연체정보";

/// 본문(Body) 영역의 아래 경계.
fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

/// 본문 영역 아래 경계를 윗변이 넘어선 `TextRun` 을 모은다.
///
/// 윗변으로 재는 이유는 `LAYOUT_OVERFLOW_CELL` 진단과 같다 — 마지막 줄
/// 디센더가 경계를 스치는 정상 상태를 잡지 않는다.
fn runs_below(node: &RenderNode, limit: f64, found: &mut Vec<(f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if node.bbox.y > limit + 0.5 && !run.text.trim().is_empty() {
            found.push((node.bbox.y, run.text.clone()));
        }
    }
    for child in &node.children {
        runs_below(child, limit, found);
    }
}

/// 페이지 본문 안의 모든 텍스트를 이어 붙인다.
fn body_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        body_text(child, out);
    }
}

fn body_node(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body_node)
}

#[test]
fn issue_5846_cut_fragment_defers_partial_nested_tail_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse #5846 fixture");

    let pages = doc.page_count();
    assert_eq!(
        pages, ORACLE_PAGES,
        "#5846 전제 붕괴 — 쪽수 {pages} (한글 2020 정답지 {ORACLE_PAGES})"
    );

    let cut = doc
        .build_page_render_tree(CUT_PAGE)
        .expect("#5846 컷 쪽 render tree");
    let cut_body = body_node(&cut.root).expect("#5846 컷 쪽 Body 영역");
    let limit = body_bottom(&cut.root).expect("#5846 컷 쪽 Body 하한");

    // 본체 판정: 컷 쪽 본문 아래로 새어 나간 글자가 하나도 없어야 한다.
    // 수정 전에는 549개(최하단 y=1,725.6px)가 셀 스텁 안에 그려졌다.
    // 꼬리말(`- 59 -`)은 본문 밖 아래 여백에 정상적으로 보이므로 Body 안만 센다.
    let mut escaped = Vec::new();
    runs_below(cut_body, limit, &mut escaped);
    let deepest = escaped.iter().map(|(y, _)| *y).fold(0.0f64, f64::max);
    assert!(
        escaped.is_empty(),
        "#5846 회귀 — {}쪽 본문 하한 {limit:.1}px 밖 TextRun {}개 (최하단 y={deepest:.1}px)",
        CUT_PAGE + 1,
        escaped.len(),
    );

    // 정답지 대조: `단기연체정보` 는 60쪽 몫이다. 컷 쪽에 남으면 중복이다.
    let mut cut_text = String::new();
    body_text(cut_body, &mut cut_text);
    let cut_text: String = cut_text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !cut_text.contains(CONT_FIRST_ITEM),
        "#5846 회귀 — {}쪽에 {}쪽 첫 항목 `{CONT_FIRST_ITEM}` 이 중복 렌더됐다",
        CUT_PAGE + 1,
        CONT_PAGE + 1,
    );

    // 이월 확인: 그 내용은 사라지지 않고 60쪽에 온전히 남는다.
    let cont = doc
        .build_page_render_tree(CONT_PAGE)
        .expect("#5846 연속 쪽 render tree");
    let mut cont_text = String::new();
    if let Some(body) = body_node(&cont.root) {
        body_text(body, &mut cont_text);
    }
    let cont_text: String = cont_text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        cont_text.contains(CONT_FIRST_ITEM),
        "#5846 내용 유실 — {}쪽에서 뺀 `{CONT_FIRST_ITEM}` 이 {}쪽에도 없다",
        CUT_PAGE + 1,
        CONT_PAGE + 1,
    );
}
