//! Issue #2070 잔여 — RowBreak/CellBreak 대형 표 분할 밀도 핀.
//!
//! 검증 축 (maintainer 요청, PR #2198 리뷰 후속):
//!
//! | 문서 | 기준 PDF | rhwp 핀 | 잔여 |
//! |---|---|---|---|
//! | 시장구조조사 (RowBreak 변종 최대 인스턴스, pi=1298 2195행×8열 외 3표) | 315쪽 (`pdf/task2070/...-2022.pdf`) | 307 (잠정) | −8 |
//! | 화성시 별표2 (CellBreak 원문 타깃) | 162쪽 (`pdf/issue2063_huge_cellbreak_table-2020.pdf`) | 159 (잠정) | −3 |
//!
//! 본 수정(행미 공백 유령 줄 + aim=true 패딩 0 존중 + 비-Percent 줄간격
//! 2×스케일 /2)으로 시장구조조사가 606→307쪽 회복 (행 피치 50.4→22.0px =
//! 선언 셀높이 = 한글 PDF 실측 21.9px; 본문 Fixed 3320HU 줄 pitch 44.3→22.1px).
//! 잠정 핀은 잔여 축 해소 시 기준 PDF 값으로 복귀시킨다.

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
fn sijang_rowbreak_density_pin() {
    let pages = page_count_of(
        "samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp",
    );
    // [#2287] RowBreak rowspan 블록 연속 조각의 잔여 증발 보정으로 307→309
    // (정답 315 방향 +2, 잔여 −6). 연속 조각 구간(p72 전후) 오버플로 부재
    // SVG 실측 (ymax ≤ 975 < 페이지). 잠정 핀 갱신.
    assert_eq!(
        pages, 309,
        "시장구조조사 잠정 309쪽 (PDF 정답 315, 잔여 −6 — #2070/#2287). 실측 {pages}p: \
         310p+면 행미 공백 hanging/aim 패딩 0 존중/비-Percent 줄간격 스케일 회귀, \
         309p 미만이면 #2287 잔여 증발 재발 의심."
    );
}

#[test]
fn huge_cellbreak_table_pin() {
    let pages = page_count_of("samples/issue2063_huge_cellbreak_table.hwp");
    assert_eq!(
        pages, 159,
        "화성시 별표2 잠정 159쪽 (PDF 정답 162, 잔여 −3 — #2070 원문). 실측 {pages}p."
    );
}
