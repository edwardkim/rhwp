//! [Issue #6368] 표 행 컷 기본 용량 비교의 부동소수 끝자리 관용.
//!
//! `samples/hwpctl_API_v2.4.hwp` Example 코드 상자(1×1 RowBreak 표)의 마지막
//! 코드 줄은 유닛 합 vs 예산이 **0.07px대** 초과다(이슈 서술 0.07px, RHWP_DIAG_6368
//! 실측 0.0267px). 관용이 0이면 그 줄만 13쪽 머리의 고아로 이월된다
//! (p12→13 table_fragment 24자). 현재 devel 의
//! `ROW_CUT_CAPACITY_FP_EPSILON_PX = 0.1` 은 그 끝자리만 흡수해 한글처럼 12쪽에
//! 남긴다. 0.5px 로 올리면 실제 경계 초과(giant_cell 0.1867px · issue2439 0.4px)
//! 까지 삼키므로 이 핀은 관용 폭을 키우지 않는다.
//!
//! 한글 정답지 총쪽수 105는 유지해야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";
/// 한컴 정답지 12쪽에 남는 Example 코드 상자 마지막 줄.
const LAST_CODE_LINE: &str = r#"tbset.SetItem("Cols", 5);"#;
const HANGUL_PAGE_COUNT: u32 = 105;

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

fn page_text(doc: &DocumentCore, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트 추출: {e}", page + 1))
}

fn first_nonempty_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
}

#[test]
fn issue_6368_hwpctl_api_example_last_line_stays_on_page_12() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        HANGUL_PAGE_COUNT,
        "#6368: 행 컷 0.1px 관용은 한글 총쪽수 105를 건드리면 안 된다"
    );

    let page12 = page_text(&doc, 11);
    let page13 = page_text(&doc, 12);

    assert!(
        page12.contains(LAST_CODE_LINE),
        "#6368: 0.07px 초과 코드줄이 12쪽에 남아야 한다 \
         (관용 0이면 13쪽으로 이월)\n--- 12쪽 ---\n{page12}"
    );
    // 같은 문서 13쪽 Remarks 예제에도 `SetItem("Cols", 5)` 가 다시 나온다.
    // 결함은 "12쪽 마지막 줄이 13쪽 **머리**의 고아로 이월" 이므로 첫 줄만 본다.
    assert_ne!(
        first_nonempty_line(&page13),
        LAST_CODE_LINE,
        "#6368: 12쪽 마지막 코드줄이 13쪽 머리의 고아로 이월되면 안 된다\n\
         --- 13쪽 ---\n{page13}"
    );
    assert_eq!(
        first_nonempty_line(&page13),
        r#"var table = HwpCtrl.InsertCtrl("tbl", tbset);"#,
        "#6368: 한글 정답지처럼 13쪽은 Example 다음 줄로 시작해야 한다\n\
         --- 13쪽 ---\n{page13}"
    );
}
