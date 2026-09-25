//! [#5920] 쪽 하단 중첩 표 atom 은 상자 아래 **보이지 않는 이송 여백** 때문에
//! 다음 쪽으로 밀리지 않아야 한다.
//!
//! `press_release_split_cell_nested_table` 은 본문 전체가 3행 1열 `RowBreak` 표의
//! 한 셀 안에 들어 있고, 8쪽의 두 상자는 그 셀 안의 중첩 표(문단당 1개 atom)다.
//! 한글 2020 정본(`pdf/issue3637/press_release_split_cell_nested_table-2020.pdf`)
//! 8쪽은 `< 은행의 위탁보증 포트폴리오 구성(가상사례) >` 상자와 그 아래
//! `☞ 그동안 지속적인 노력에도 …` 결론 상자를 **함께** 담는다
//! (결론 상자 테두리 512.0~778.0pt, 본문 하단 785.2pt — 7.2pt 여유).
//!
//! 수정 전 rhwp 는 결론 상자 유닛(369.507px)의 상자 **아래 이송 여백**까지 쪽
//! 예산에 넣어 `avail 997.827px` 를 **0.467px** 넘겼고, 상자를 9쪽으로 밀어
//! 8쪽 아래 절반이 통째로 비었다.
//!
//! 이 문서의 총 쪽수(13)는 9쪽의 별개 결함(본문이 정본보다 세 줄 높게 쌓인다)
//! 때문에 그대로다 — 총 쪽수는 `tests/fixtures/render_page_samples.tsv` 가 고정한다.
//! 여기서는 8쪽 정합만 고정한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue3637/press_release_split_cell_nested_table.hwpx";

/// 앞 상자 표제 — 정본 8쪽 위쪽.
const BOX_TITLE: &str = "은행의 위탁보증 포트폴리오 구성";
/// 결론 상자 첫 줄 — 정본 8쪽 아래쪽, 같은 쪽에 있어야 한다.
const CONCLUSION_HEAD: &str = "그동안 지속적인 노력에도";
/// 결론 상자 마지막 줄 — 정본 8쪽 맨 아래.
const CONCLUSION_TAIL: &str = "심사 여력을 확대";

fn page_text(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = DocumentCore::from_bytes(&bytes).expect("parse sample");
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트 추출: {e}", page + 1))
}

#[test]
fn issue_5920_page8_holds_both_nested_boxes() {
    let page8 = page_text(7);
    assert!(
        page8.contains(BOX_TITLE),
        "#5920 전제: 8쪽에 포트폴리오 상자가 있어야 한다\n--- 8쪽 ---\n{page8}"
    );
    assert!(
        page8.contains(CONCLUSION_HEAD),
        "#5920: 결론 상자는 정본처럼 같은 8쪽에 앉아야 한다 \
         (상자 아래 이송 여백만 쪽 경계를 넘는다)\n--- 8쪽 ---\n{page8}"
    );
    assert!(
        page8.contains(CONCLUSION_TAIL),
        "#5920: 결론 상자 마지막 줄까지 8쪽에 들어가야 한다\n--- 8쪽 ---\n{page8}"
    );
}

#[test]
fn issue_5920_page9_does_not_restart_with_the_conclusion_box() {
    let page9 = page_text(8);
    assert!(
        !page9.contains(CONCLUSION_HEAD),
        "#5920: 8쪽에 앉은 결론 상자가 9쪽에 다시 나오면 안 된다\n--- 9쪽 ---\n{page9}"
    );
}

/// [#7206] 이어받은 표 조각의 상자는 **본문보다 클 수 없다** — 물리 4쪽.
///
/// 같은 문서의 `pi=5`(3행 1열 `RowBreak`, 본문 전체를 담은 칸)는 여러 쪽에 걸친다.
/// 수정 전 rhwp 는 물리 4쪽에서 그 조각 상자를 `45.30 → 1053.90`(높이 1008.60)으로
/// 칠했는데 본문은 `45.30 → 1046.90`(높이 1001.60)이다 — **조각이 본문 전체보다
/// 7.0px 컸다.** 페이지네이터 자신도 `consumed=1008.6 avail=1001.6` 으로 기록했다.
///
/// 원인은 `table/scan/runner/row_step.rs` 의 쪽 면적 초과 가드가 `r > cursor_row`
/// 일 때만 열려, 조각이 **커서 행 안에서 시작해 같은 행에서 끝나는** 흔한 형상에서
/// 진입조차 하지 못한 것이다. 그 경로는 칠할 높이를 가용 높이와 대조 없이 받았다.
/// 가드를 그 형상에도 열자 예산 재시도가 더 작은 컷을 찾아 상자가 `1039.00` 이 됐다.
///
/// 기대값은 한/글 출력이 아니라 **자기 완결적 불변식**이다 — 어떤 조각도 본문보다
/// 클 수 없다. 그래서 이 검사는 정본 PDF 없이도 성립한다.
///
/// **아직 전 쪽에서 성립하지는 않는다.** 같은 문서 물리 8쪽은 `+4.57px` 가 남아 있다
/// (`cand=1006.2 avail=1001.6`). 그 조각은 재시도가 더 작은 컷을 찾지 못해 기존
/// `continuation_row_must_advance` 갈래가 원래 컷을 수용한다 — 칠할 높이가 논리 컷보다
/// `~20px` 큰데 재시도 예산은 그 차이를 예약하지 않기 때문이다(`retry_uses_painted_tail`
/// 은 `mixed_nested_owner_guard`·`native_split_continuation_row_tail` 에서만 선다).
/// 그 축은 별도 변경이라 여기서는 **이 수정이 실제로 고친 4쪽만** 잠근다.
#[test]
fn issue_7206_page4_fragment_does_not_exceed_the_body() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = DocumentCore::from_bytes(&bytes).expect("parse sample");

    fn body_bottom(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node.bbox.y + node.bbox.height);
        }
        node.children.iter().find_map(body_bottom)
    }
    fn table_bottoms(node: &RenderNode, para: usize, out: &mut Vec<f64>) {
        if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(para)) {
            out.push(node.bbox.y + node.bbox.height);
        }
        for child in &node.children {
            table_bottoms(child, para, out);
        }
    }

    // 물리 4쪽(0-based 3) — 이 수정이 고친 조각.
    const PAGE: u32 = 3;
    let tree = doc
        .build_page_render_tree(PAGE)
        .unwrap_or_else(|e| panic!("{}쪽 render tree: {e}", PAGE + 1));
    let bottom = body_bottom(&tree.root).expect("본문 상자");
    let mut bottoms = Vec::new();
    table_bottoms(&tree.root, 5, &mut bottoms);
    let table_bottom = bottoms.into_iter().fold(f64::MIN, f64::max);
    assert!(
        table_bottom > f64::MIN,
        "#7206 전제: 물리 {}쪽에 pi=5 조각이 있어야 한다",
        PAGE + 1
    );
    let over = table_bottom - bottom;
    assert!(
        over <= 0.5,
        "#7206: 이어받은 조각 상자가 본문을 넘었다 — 물리 {}쪽 표 하단 {table_bottom:.2}px, \
         본문 하단 {bottom:.2}px, 초과 {over:.2}px. 조각은 본문보다 클 수 없다 \
         (수정 전 1053.90 vs 1046.90 = +7.00, 수정 후 1039.00).",
        PAGE + 1
    );
}
