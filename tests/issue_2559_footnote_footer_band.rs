//! Issue #2559: 각주 있는 문서의 +N 과다분할 — 빈 꼬리말 밴드를 각주가 사용.
//!
//! `samples/issue2559/1341000_research_report_footnotes.hwp` — 각주가 많은 정책
//! 연구보고서(꼬리말 정의 없음). 한글은 꼬리말 콘텐츠가 없으면 각주를 꼬리말
//! 여백 밴드(footer_area)에 배치해 본문을 침범하지 않는다. 종전 rhwp 는 각주
//! 높이를 본문 영역에서 그대로 차감해 페이지마다 조기 개행 → 누적 +N 과다분할
//! (10k r17: .hwp 연구보고서 41건, 대표 1450000-201700178 은 한글 274 vs
//! rhwp 294). 수정: 각주는 빈 꼬리말 밴드를 먼저 소비하고 초과분만 본문을 줄인다.
//!
//! 이 문서: 한글 정답 92쪽. 수정 전 98쪽(+6) → 수정 후 94쪽(+2, 잔여는 꼬리말
//! 밴드와 무관한 별원인). 각주 없는 문서(결재문서 92 컨트롤셋)는 penalty=0 으로
//! 완전 불변이라 회귀 없음.

use std::fs;
use std::path::Path;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

#[test]
fn research_report_footnote_footer_band_page_count_pin() {
    let pages = page_count_of("samples/issue2559/1341000_research_report_footnotes.hwp");
    assert_eq!(
        pages, 94,
        "issue2559 1341000 핀 94쪽 (한글 92, 수정 전 98). 각주가 빈 꼬리말 밴드를 \
         소비해 본문 조기개행이 완화됐다. 98p 부근이면 꼬리말 밴드 미회수(#2559) \
         회귀, 92p 도달 시 잔여 별원인까지 해소된 것이니 핀을 낮출 것. 실측 {}p.",
        pages
    );
}
