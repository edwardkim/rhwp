//! [Issue #6342] 본문을 거의 채운 TAC 결재 표 뒤 붙임 두 줄은 다음 쪽이다.
//!
//! `samples/hwpx/opengov/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.).hwpx`
//! 는 표 899.5px / 본문 952.5px 뒤에 붙임 28.8+28.8px 가 온다. 한글 정답지
//! (`pdf/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.)-2024.pdf`)
//! 는 2쪽. 첫 줄을 잔여 칸에 끼워 넣으면 used=964.3px 로 넘치고 rhwp 는 1쪽이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::path::Path;

const SAMPLE: &str =
    "samples/hwpx/opengov/36385445_결재문서본문_화재발생종합보고서(제2026-189호, 2026. 5. 14.).hwpx";

#[test]
fn issue_6342_approval_table_attachments_use_second_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    assert_eq!(
        core.page_count(),
        2,
        "한글 2024 정답지는 2쪽 (결함 시 표+붙임을 한 쪽에 담아 1쪽)"
    );
}
