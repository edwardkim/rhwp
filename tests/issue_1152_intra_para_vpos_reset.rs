//! Issue #1152: 호스트 문단 내 TAC 표 line_seg vpos=0 (intra-paragraph vpos-reset)
//! 페이지 분할 미적용.
//!
//! `samples/2022년 국립국어원 업무계획.hwp` 의 pi=586 호스트 문단(empty-text,
//! controls=2)에서:
//! - ci=0: 12×5 본문 표 (wrap=위아래, 비-TAC, RowBreak 분할)
//! - ci=1: 1×3 별첨 박스 (wrap=위아래, TAC=treat_as_char=true)
//! - ls[0] vpos=69196 (호스트 닻줄)
//! - ls[1] vpos=0      ← HWP 가 "새 페이지 상단부터" 라고 명시한 intra-para reset
//!
//! 한컴 한글 2022 PDF 정합:
//! - page 32 (page_num=30): 12×5 PartialTable(8~12행) 까지만, 별첨 박스 없음
//! - page 33 (page_num=31): 1×3 별첨 박스 → 본문 표 → "□ 연 혁" 시작
//!
//! 회귀 (수정 전 버그):
//! - page 32 에 PartialTable + 별첨 박스 모두 배치 (used=926.8/933.5px 로 가까스로 fit)
//!
//! 정정: `typeset.rs:typeset_tac_table()` 진입부 intra-paragraph vpos-reset 가드 추가.

use std::fs;
use std::path::Path;

#[test]
fn issue_1152_별첨_box_starts_page_33_not_32() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/2022년 국립국어원 업무계획.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 2022년 국립국어원 업무계획.hwp");

    let dump = doc.dump_page_items(None);

    // 페이지 32 영역 (page_num=30) 추출
    let page32 = extract_page(&dump, "global_idx=31").unwrap_or_else(|| {
        panic!(
            "페이지 32(global_idx=31) 영역을 찾지 못함.\n--- dump ---\n{}",
            dump
        )
    });
    // 페이지 33 영역 (page_num=31) 추출
    let page33 = extract_page(&dump, "global_idx=32").unwrap_or_else(|| {
        panic!(
            "페이지 33(global_idx=32) 영역을 찾지 못함.\n--- dump ---\n{}",
            dump
        )
    });

    // 페이지 32 에는 1×3 별첨 박스 (pi=586 ci=1) 가 없어야 함.
    // 회귀 시: "Table   pi=586 ci=1 ... tac=true" 가 등장.
    assert!(
        !page32.contains("pi=586 ci=1"),
        "페이지 32 에 별첨 박스(pi=586 ci=1)가 포함됨 — intra-paragraph vpos-reset 가드 회귀.\n\
         --- page 32 ---\n{}",
        page32
    );

    // 페이지 33 에는 1×3 별첨 박스 (pi=586 ci=1) 가 포함되어야 함.
    assert!(
        page33.contains("pi=586 ci=1"),
        "페이지 33 에 별첨 박스(pi=586 ci=1)가 없음.\n--- page 33 ---\n{}",
        page33
    );
}

/// dump_page_items 출력에서 `marker` 가 포함된 `=== 페이지 ...` 헤더부터
/// 다음 페이지 헤더 직전까지의 본문을 반환.
fn extract_page<'a>(dump: &'a str, marker: &str) -> Option<&'a str> {
    let header_pos = dump
        .lines()
        .scan(0usize, |off, line| {
            let cur = *off;
            *off += line.len() + 1;
            Some((cur, line))
        })
        .find(|(_, l)| l.starts_with("=== 페이지") && l.contains(marker))
        .map(|(o, _)| o)?;

    let after_header = &dump[header_pos..];
    let end = after_header
        .lines()
        .scan(0usize, |off, line| {
            let cur = *off;
            *off += line.len() + 1;
            Some((cur, line))
        })
        .skip(1)
        .find(|(_, l)| l.starts_with("=== 페이지"))
        .map(|(o, _)| o)
        .unwrap_or(after_header.len());
    Some(&after_header[..end])
}
