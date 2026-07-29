//! Issue #2559: 각주가 빈 꼬리말 밴드를 쓰지 못해 장문 문서가 과다 분할되는 회귀.
//!
//! 한글 기준은 92쪽이며, 수정 전 rhwp는 98쪽이었다. 빈 꼬리말 밴드 회수(+4,
//! PR #2627)에 이어 셀 저장-ls1 재래핑 임계 ×1.8 완화(#2430, PR #2714)가 잔여
//! +2쪽을 해소해 **한글 정답 92쪽 정합**에 도달 — 이를 고정한다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

#[test]
fn research_report_reclaims_empty_footer_band_for_footnotes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2559/1341000_research_report_footnotes.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let document = HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()));

    assert_eq!(
        document.page_count(),
        92,
        "#2559 샘플의 한글 정답은 92쪽. 94쪽 부근이면 #2430 셀 재래핑 임계(PR #2714) 회귀, \
         98쪽 부근이면 빈 꼬리말 밴드 회수(PR #2627) 회귀다. 실측 {}쪽.",
        document.page_count()
    );
}
