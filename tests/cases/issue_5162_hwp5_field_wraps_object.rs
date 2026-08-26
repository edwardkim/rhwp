//! [Issue #5162] HWPX → HWP5 저장에서 **표·그림을 감싼 0길이 누름틀**의 `FIELD_END` 가
//! 개체 앞에 찍혀 개체가 필드 밖으로 밀려난다.
//!
//! 텍스트 없이 표 하나만 감싼 `CLICK_HERE` 누름틀은 텍스트 축에서 0길이
//! (`start_char_idx == end_char_idx == text_len`)라 HWP5 직렬화기의 **후행 컨트롤
//! 방출 경로**(`serializer/body_text.rs` 의 `trailing_end_after_ctrl`)로 온다. 그 경로가
//! `FIELD_END` 를 자기 `FIELD_BEGIN` 바로 뒤(감싼 개체 앞)에 닫으면 PARA_TEXT 배치가
//! `FIELD_BEGIN → FIELD_END → 개체` 가 되어 누름틀이 비고, 한글 2022 는 빈 누름틀 자리에
//! 안내문("이곳을 마우스로 누르고 내용을 입력하세요.")을 본문으로 렌더한다.
//!
//! 파서는 이 경우를 위해 `FieldRange::inner_slot_count`(필드 안 컨트롤 슬롯 수)를 채워
//! 두는데, HWPX 직렬화기(`serializer/hwpx/section.rs`)만 이를 읽고 HWP5 직렬화기는 읽지
//! 않았다. 수정은 후행 `FIELD_END` 를 `control_idx + inner_slot_count` 뒤에서 닫아 개체를
//! 필드 안에 남긴다 — HWPX 축과 동형이다.
//!
//! 계약: HWPX → HWP5 저장 후 재파싱하면, 표를 감싼 누름틀의 `FieldRange` 가 그 표 컨트롤
//! 슬롯을 자기 범위 안에 품어야 한다(`inner_slot_count >= 1`). 버그 상태에서는 표가 필드
//! 밖으로 밀려 `inner_slot_count == 0` 이 된다.
//!
//! 재현 표본은 `samples/issue5162_field_wraps_table.hwpx` — 표 하나를 `CLICK_HERE`
//! 누름틀로 감싼 최소 문서다. 재현 절차는 `tools/issue5162_field_wraps_table.py`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const SAMPLE: &str = "samples/issue5162_field_wraps_table.hwpx";

/// HWPX 를 HWP5 로 저장한 뒤 다시 파싱한다.
fn hwpx_to_hwp5_reparsed() -> rhwp::model::document::Document {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(SAMPLE);
    let hwpx = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let core = DocumentCore::from_bytes(&hwpx).expect("HWPX 로드");
    let hwp5 = core.export_hwp_native().expect("HWP5 직렬화");
    rhwp::parser::parse_document(&hwp5).expect("HWP5 재파싱")
}

#[test]
fn hwp5_field_wraps_its_table_slot() {
    let doc = hwpx_to_hwp5_reparsed();

    // 표와 누름틀 필드를 함께 가진 문단을 찾는다.
    let para = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .find(|p| {
            p.controls.iter().any(|c| matches!(c, Control::Table(_)))
                && p.controls.iter().any(|c| matches!(c, Control::Field(_)))
        })
        .expect("표를 감싼 누름틀 문단이 있어야 한다");

    let table_idx = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .expect("표 컨트롤 index");

    // 그 표를 자기 슬롯 범위 안에 품는 FieldRange 가 있어야 한다.
    // 버그(FIELD_END 를 개체 앞에 방출)면 필드가 0길이로 비어 표가 범위 밖으로 밀린다.
    let wraps = para
        .field_ranges
        .iter()
        .any(|fr| fr.control_idx < table_idx && table_idx <= fr.control_idx + fr.inner_slot_count);

    assert!(
        wraps,
        "표(ctrl[{table_idx}])를 감싼 누름틀이 없다 — field_ranges={:?} (FIELD_END 가 개체 앞에 찍혀 누름틀이 빈 #5162 회귀)",
        para.field_ranges
            .iter()
            .map(|fr| (fr.control_idx, fr.start_char_idx, fr.end_char_idx, fr.inner_slot_count))
            .collect::<Vec<_>>()
    );
}
