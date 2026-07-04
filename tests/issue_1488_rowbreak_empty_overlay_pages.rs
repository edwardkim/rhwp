//! Issue #1488: RowBreak 표 셀 내부 빈 오버레이 문단이 여분 빈 연속 페이지를 양산.
//!
//! `samples/rowbreak-problem-pages.hwpx` 섹션 1, 문단 28 의 1×1 RowBreak 표는
//! 단일 셀에 18개 문단을 담는다:
//! - p[0..5]: 본문 텍스트
//! - p[6..15]: 빈(text_len=0) 오버레이 스페이서 문단 — 본문 위에 동일/역방향 vpos 로 겹침
//! - p[16..17]: TAC 사각형(다이어그램)
//!
//! 회귀 (수정 전 버그):
//! - `cell_units` 가 빈 오버레이 문단의 vpos 리셋을 `hard_break_before` 로 표시 →
//!   `advance_row_cut` 가 가용 예산과 무관하게 리셋마다 컷을 끊어, 954px 가용 페이지에
//!   32~85px 만 배치하고 페이지를 넘기는 거의 빈 연속 페이지를 5장 양산 (총 22페이지).
//!
//! 정정: 비가시(빈 텍스트) 오버레이 문단의 vpos 리셋은 하드 브레이크에서 제외.
//! - 한컴 한글 2024 PDF(`pdf/rowbreak-problem-pages-2024.pdf`) = 18페이지 정합.
//! - pi=28 표는 8페이지(여분 빈 페이지 포함)가 아니라 3페이지로 분할.

use std::fs;
use std::path::Path;

#[test]
fn issue_1488_no_extra_empty_continuation_pages() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path = Path::new(repo_root).join("samples/rowbreak-problem-pages.hwpx");
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse rowbreak-problem-pages.hwpx");

    // 정답지 PDF(한글 2024) 18페이지 정합 — 여분 빈 연속 페이지가 제거되어야 함.
    assert_eq!(
        doc.page_count(),
        18,
        "여분 빈 연속 페이지 회귀 — 페이지 수가 18이 아님 (수정 전 22)"
    );

    // pi=28 의 1×1 RowBreak 표(셀 내부 빈 오버레이 문단)는 과도하게 분할되면 안 된다.
    // 수정 전: 8페이지에 걸쳐 거의 빈 연속 페이지 양산. 수정 후: 3페이지.
    let dump = doc.dump_page_items(None);
    let pi28_pages = dump
        .lines()
        .filter(|l| l.contains("PartialTable") && l.contains("pi=28 ci=0"))
        .count();
    assert!(
        pi28_pages <= 4,
        "pi=28 RowBreak 표가 {}페이지에 걸쳐 분할됨 — 빈 오버레이 hard-break 회귀 (기대 ≤4)",
        pi28_pages
    );
}
