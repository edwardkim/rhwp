//! 고정형(진단 API의 수치/null 계약)·관계형(두 필드의 동시 존재·저장 출처). px 좌표 검사는 없다.
//! 주석 갱신: 2026-11-17 19:34:36 KST. 직전 파일 변경: 2026-11-17T18:30:07+09:00, 8d91a0c4b2924e63b3927ce3018b54bcf2a9eda2.
//! 비시각 진단 계약으로 PDF 일치율 대상이 아니다. 저장·재열기한 실제 CLI 대조를 확인했다.
//! 근거: mydocs/pr/assets/open_prs_20261117/diagnostic9144-actual-cli.json.
//! #9142 dump-pages 의 `hwpUsedHeight`/`usedDiff` 는 한/글이 저장한 LINE_SEG 만 근거로 삼는다.
//!
//! rhwp 가 계산해 넣은 합성 줄(`LineSeg::TAG_IMPLEMENTATION_PROPERTY`, bit 31)을 한/글 저장값처럼
//! 읽으면, 표를 넣고 저장한 문서에서 조판이 맞는데도 가짜 차이(`diff=+20.4px`)를 보고했다.
//! #8939 이후 IR 의 bit 31 은 `/RhwpLineSegTags` 로 인증된 rhwp 합성 줄만 뜻한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const SAMPLE: &str = "samples/hwpx/hancom-hwp/blank_hwpx.hwp";

fn load_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// 첫 쪽 첫 단의 `hwpUsedHeight`.
fn first_column_hwp_used(core: &DocumentCore) -> Value {
    let pages = core.dump_page_items_json(Some(0));
    let page = &pages.as_array().expect("pages")[0];
    let column = &page["columns"].as_array().expect("columns")[0];
    assert_eq!(
        column["hwpUsedHeight"].is_null(),
        column["usedDiff"].is_null(),
        "hwpUsedHeight 와 usedDiff 는 함께 있거나 함께 null 이어야 한다: {column}"
    );
    column["hwpUsedHeight"].clone()
}

#[test]
fn hancom_stored_lines_still_estimate_hwp_used_height() {
    let core = load_sample();
    let used = first_column_hwp_used(&core);
    assert!(
        used.as_f64().is_some(),
        "한/글이 저장한 줄만 읽는 단은 hwpUsedHeight 를 추정해야 한다: {used}"
    );
}

#[test]
fn rhwp_synthetic_lines_after_save_are_not_read_as_hancom_evidence() {
    let mut core = load_sample();
    core.create_table_native(0, 0, 0, 2, 2)
        .expect("2x2 표 삽입");
    let saved = core.export_hwp_native().expect("HWP 저장");
    let reloaded = DocumentCore::from_bytes(&saved).expect("저장본 다시 열기");

    let used = first_column_hwp_used(&reloaded);
    assert!(
        used.is_null(),
        "rhwp 합성 줄(bit 31)을 읽는 단은 hwpUsedHeight 를 추정하지 않아야 한다(null): {used}"
    );
}
