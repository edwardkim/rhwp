//! Issue #643: 페이지 분할 드리프트 — pi=80 마지막 줄 다음 페이지 누락
//!
//! `samples/2022년 국립국어원 업무계획.hwp` 6페이지 마지막 문단(pi=80) 의
//! 두 번째 줄 (' 및 점자 해당 분야 전문인력 확보 어려움') 이 다음 페이지로
//! 부당 분리됨. HWP 원본은 같은 페이지에 배치.
//!
//! Root cause: `pagination/engine.rs` fit 루프가 마지막 줄의 트레일링
//! `line_spacing` 까지 누적 → 잔여 공간 산정 왜곡.
//!
//! 정정 후 기대: pi=80 가 페이지 6 에 `FullParagraph` 로 배치 (또는
//! `PartialParagraph { end_line: 2 }` — 어떤 형태든 line 1 이 페이지 6 에 포함).

use std::fs;
use std::path::Path;

#[test]
fn page6_pi80_last_line_stays_on_page6() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/2022년 국립국어원 업무계획.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 2022년 국립국어원 업무계획.hwp");

    // 페이지 6 (0-based index 5) 만 덤프
    let dump = doc.dump_page_items(Some(5));

    // 페이지 6 섹션을 추출 — "=== 페이지 6 " 로 시작, 다음 "=== 페이지" 까지
    let page6_start = dump.find("=== 페이지 6 ")
        .expect("페이지 6 덤프 섹션을 찾지 못함");
    let after = &dump[page6_start..];
    let page6_section = match after[10..].find("=== 페이지") {
        Some(end_rel) => &after[..10 + end_rel],
        None => after,
    };

    // pi=80 항목이 페이지 6 에 존재하는지 확인
    let pi80_line = page6_section.lines()
        .find(|l| l.contains("pi=80"))
        .unwrap_or_else(|| panic!("페이지 6 에 pi=80 항목이 없음.\n--- page6 dump ---\n{}", page6_section));

    // 정정 기대:
    // - FullParagraph pi=80  ... (전체 배치)
    // - 또는 PartialParagraph pi=80  lines=0..2  ... (line 1 포함)
    //
    // 회귀 (현재 버그):
    // - PartialParagraph pi=80  lines=0..1  ... (line 1 누락)

    let is_full = pi80_line.contains("FullParagraph") && pi80_line.contains("pi=80");
    let is_complete_partial = pi80_line.contains("PartialParagraph")
        && pi80_line.contains("pi=80")
        && pi80_line.contains("lines=0..2");

    assert!(
        is_full || is_complete_partial,
        "pi=80 line 1 (' 및 점자 해당 분야 전문인력 확보 어려움') 이 페이지 6 에 포함되어야 함.\n\
         실제 항목: {}\n\
         (회귀 패턴: 'lines=0..1' — line 1 이 다음 페이지로 분리됨)",
        pi80_line.trim()
    );
}
