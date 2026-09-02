//! Issue #2070 잔여 — RowBreak/CellBreak 대형 표 분할 밀도 핀.
//!
//! 검증 축 (maintainer 요청, PR #2198 리뷰 후속):
//!
//! | 문서 | 기준 PDF | rhwp 핀 | 잔여 |
//! |---|---|---|---|
//! | 시장구조조사 (RowBreak 변종 최대 인스턴스, pi=1298 2195행×8열 외 3표) | 315쪽 (`pdf/task2070/...-2022.pdf`) | **315 (정답)** | 0 |
//! | 화성시 별표2 (CellBreak 원문 타깃) | 162쪽 (`pdf/issue2063_huge_cellbreak_table-2020.pdf`) | 161 | #6360 이후 `issue_2063` sentinel에서 관리 |
//!
//! 본 수정(행미 공백 유령 줄 + aim=true 패딩 0 존중 + 비-Percent 줄간격
//! 2×스케일 /2)으로 시장구조조사가 606→307쪽 회복 (행 피치 50.4→22.0px =
//! 선언 셀높이 = 한글 PDF 실측 21.9px; 본문 Fixed 3320HU 줄 pitch 44.3→22.1px).
//! 잠정 핀은 잔여 축 해소 시 기준 PDF 값으로 복귀시킨다.
//! [#5922] 연속 조각 바깥 여백 재개방으로 화성시 별표2 의 여백 축이 해소됐고
//! 핀을 159→161 로 갱신했다. 남은 −1 은 여백이 아니라 행 경계 sub-pt 적산 축이다.
//! 같은 대형 문서의 중복 page_count는 #6360에서 `issue_2063` sentinel로 합쳤다.

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
    // [#2319] lineseg 없는 문단의 tac 표 높이 붕괴 보정으로 309→312 (정답 315
    // 방향 +3, 잔여 −3). 본 문서 전 문단 ls=0 — p3 의 6×6 tac 표(렌더 279.8px)가
    // 종전 88px 로 계상되던 것이 정상화. p3 픽셀 정합 96.78% (권위 PDF 대조),
    // 변경 페이지 오버플로 부재 (used ≤ body). 잠정 핀 갱신.
    // [#5169] 이 문서는 BodyText 와 ViewText 를 모두 갖는다. ViewText 우선 채택으로
    // 한글이 실제 렌더하는 본문을 읽게 되어 313→315 — **기준 PDF 정답에 도달**했다.
    // 잔여 −2 는 조판 결함이 아니라 "다른 본문을 읽고 있었음"이었다. 파일 머리말의
    // "잠정 핀은 잔여 축 해소 시 기준 PDF 값으로 복귀시킨다" 에 따라 정답 핀으로 전환한다.
    assert_eq!(
        pages, 315,
        "시장구조조사 정답 315쪽 (기준 PDF `pdf/task2070/...-2022.pdf`; \
         #2070/#2287/#2319 조판 축 + #5169 ViewText 우선으로 잔여 0). 실측 {pages}p: \
         313p 면 #5169 ViewText 우선 회귀(BodyText 를 읽고 있음) 의심, \
         그 밖의 값이면 #2287 잔여 증발/#2319 tac 높이 붕괴/#2279 자간 landing 재발 의심."
    );
}

// 화성시 별표2 CellBreak pin은 #6360에서
// `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup`으로 통합했다.
// 여기서 같은 `samples/issue2063_huge_cellbreak_table.hwp`를 다시 조판하면 CI wall time만
// 늘어나고, #2070의 RowBreak 고유 축 검증과는 별도 이득이 없다.
