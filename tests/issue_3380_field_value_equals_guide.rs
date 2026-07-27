//! Issue #3380: 채울 값이 누름틀 안내문과 같으면 저장 후 조용히 유실된다.
//!
//! 적재 시 `clear_initial_field_texts` 는 **properties 비트 15 == 0**(초기 상태)인 ClickHere
//! 필드의 텍스트가 안내문과 같으면 "한컴이 남긴 안내문 잔재"로 보고 지운다. 그런데 값을
//! 채우는 경로가 그 비트를 세우지 않아, 하필 안내문과 같은 값(행정 서식의 "주무관"·"공개"·
//! "해당없음" 등 흔한 실값)을 넣으면 저장·재적재 후 그 칸만 비었다. `fill-fields` 는 성공으로
//! 보고하므로 재독 대조 없이는 드러나지 않는다.
//!
//! 트리거는 길이가 아니라 **안내문과의 문자열 일치**다 — 같은 길이의 다른 값은 살아남는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const SAMPLE: &str = "samples/field-01.hwp";
const FIELD: &str = "회사명";
/// `samples/field-01.hwp` 의 `회사명` 안내문.
const GUIDE: &str = "여기에 입력";

fn load() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn field_value(doc: &rhwp::wasm_api::HwpDocument, name: &str) -> String {
    let json = doc.get_field_list();
    let v: serde_json::Value = serde_json::from_str(&json).expect("field list JSON");
    let fields = v["fields"]
        .as_array()
        .or_else(|| v.as_array())
        .expect("fields 배열");
    fields
        .iter()
        .find(|f| f["name"] == name)
        .map(|f| f["value"].as_str().unwrap_or("").to_string())
        .unwrap_or_else(|| panic!("필드 {name} 없음"))
}

/// 값을 채우고 저장·재적재한 뒤의 값을 돌려준다.
fn fill_and_roundtrip(value: &str) -> String {
    let mut doc = load();
    doc.set_field_value_by_name_api(FIELD, value)
        .expect("set_field_value_by_name");
    assert_eq!(
        field_value(&doc, FIELD),
        value,
        "메모리 반영은 종전에도 정상이었다"
    );
    let saved = doc.export_hwp().expect("export_hwp");
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&saved).expect("재적재");
    field_value(&reloaded, FIELD)
}

/// 안내문과 **같은** 값도 저장 후 살아남아야 한다.
#[test]
fn value_equal_to_guide_survives_roundtrip() {
    assert_eq!(
        fill_and_roundtrip(GUIDE),
        GUIDE,
        "안내문과 같은 값이 저장·재적재에서 유실됐다"
    );
}

/// 안내문과 다른 값은 종전대로 보존된다(무회귀). 같은 길이도 함께 확인해, 트리거가 길이가
/// 아니라 문자열 일치임을 계약으로 남긴다.
#[test]
fn other_values_survive_roundtrip() {
    for value in ["주식회사 가나다", "가나다라마바", "가나"] {
        assert_eq!(
            fill_and_roundtrip(value),
            value,
            "안내문과 다른 값 {value:?} 이 유실됐다"
        );
    }
}

/// 빈 값으로 되돌리면 안내문 상태(빈 값)로 남아야 한다 — 채움 표시가 굳어버리지 않는다.
#[test]
fn clearing_value_returns_to_empty() {
    let mut doc = load();
    doc.set_field_value_by_name_api(FIELD, "주식회사 가나다")
        .expect("채우기");
    doc.set_field_value_by_name_api(FIELD, "").expect("비우기");
    let saved = doc.export_hwp().expect("export_hwp");
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&saved).expect("재적재");
    assert_eq!(
        field_value(&reloaded, FIELD),
        "",
        "비운 필드는 빈 값이어야 함"
    );
}
