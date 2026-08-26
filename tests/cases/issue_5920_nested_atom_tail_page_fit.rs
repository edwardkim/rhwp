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
