//! Issue #1921 — 59043 규제영향분석서 페이지네이션 드리프트 핀.
//!
//! `samples/issue1921/59043_regulatory_analysis.hwp` — 부동(자리차지) 표·rowspan
//! 블록이 밀집한 규제영향분석서. PR #2092(RowBreak 블록컷 sliver 흡수)로
//! 48쪽 → 42쪽 (수정 전 pi=160 3×3 rowspan 블록에서 컷 진동 `+46,+1` 교대).
//!
//! 권위 정답지는 한글 2022 편집기 37쪽
//! (`pdf/issue1921/59043_regulatory_analysis-2022.pdf`, 편집기 PageCount=37 정합).
//! 잔여 +5는 2단 배치 밀도(부동 표 흐름 패킹) 축으로 #1921 후속 과제 — 본 테스트는
//! 현재 도달값 42를 핀해 개선(37 방향)과 회귀(43+)를 모두 표면화한다.

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
fn regulatory_59043_page_count_pin() {
    let pages = page_count_of("samples/issue1921/59043_regulatory_analysis.hwp");
    assert_eq!(
        pages, 42,
        "issue1921 59043 핀 42쪽 (한글 2022 정답지 37쪽, 잔여 +5=배치 밀도 축). \
         실측 {}p — 43p+면 sliver/과분할 회귀, 42p 미만이면 개선이므로 핀과 \
         정답지(37)를 갱신할 것.",
        pages
    );
}
