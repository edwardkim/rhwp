//! [Issue #3545] 안내문과 같은 본문 텍스트를 가진 누름틀이 HWPX 로드만 해도 텍스트를
//! 잃고, 저장하면 파일에서 영구 소실된다.
//!
//! 적재 정규화 `clear_initial_field_texts` 는 properties 비트 15(수정됨 표식) == 0 인
//! ClickHere 필드의 안내문 잔재를 지운다. HWP5 축은 #3380 이 채움 경로에서 비트를 세워
//! 해소했지만 HWPX 축은 왕복 통로가 없었다 — 파서가 `<hp:fieldBegin dirty="...">` 를
//! 버리고 저장기도 dirty 를 방출하지 않아, HWPX 로드 후 비트 15 는 항상 0 이다.
//! OWPML fieldBegin 의 `dirty` 가 이 표식의 대응물이다 (한컴 실물: 초기 상태
//! `samples/hwpx/form-01.hwpx` 는 dirty="0", 입력된 `samples/누름틀-2024.hwpx` 는
//! dirty="1").
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read, Write};
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/form-01.hwpx";
const FIELD: &str = "myMsg01";
/// `samples/hwpx/form-01.hwpx` 의 `myMsg01` 안내문. 본문 텍스트도 이 값과 같다(초기 상태).
const GUIDE: &str = "여기에 입력";
/// 유일 누름틀의 fieldBegin 속성 앵커 — 표 셀 등 다른 요소의 dirty 와 겹치지 않는다.
const FIELD_ANCHOR: &str = r#"type="CLICK_HERE" name="myMsg01" editable="1" dirty="0""#;

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn load(bytes: &[u8]) -> rhwp::wasm_api::HwpDocument {
    rhwp::wasm_api::HwpDocument::from_bytes(bytes).expect("parse")
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

/// 샘플의 누름틀 fieldBegin 만 dirty="1" 로 바꾼 HWPX 를 만든다 (파서 축 단독 재현용).
fn with_dirty_flag(bytes: &[u8]) -> Vec<u8> {
    let mut src = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).expect("zip 열기");
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..src.len() {
        let mut entry = src.by_index(i).expect("zip 엔트리");
        let name = entry.name().to_string();
        if name == "Contents/section0.xml" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).expect("section XML");
            let patched = xml.replace(
                FIELD_ANCHOR,
                &FIELD_ANCHOR.replace("dirty=\"0\"", "dirty=\"1\""),
            );
            assert_ne!(patched, xml, "샘플에 앵커 누름틀이 있어야 한다");
            out.start_file(name, zip::write::SimpleFileOptions::default())
                .expect("start_file");
            out.write_all(patched.as_bytes()).expect("write section");
        } else {
            out.raw_copy_file(entry).expect("raw copy");
        }
    }
    out.finish().expect("finish").into_inner()
}

/// 저장본의 본문 section XML 을 연결해 돌려준다.
fn section_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx.to_vec())).expect("저장본 ZIP");
    let names: Vec<String> = zip.file_names().map(str::to_string).collect();
    let mut xml = String::new();
    for name in names {
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            zip.by_name(&name)
                .expect("section 엔트리")
                .read_to_string(&mut xml)
                .expect("section XML 은 UTF-8");
        }
    }
    xml
}

/// #3380 의 HWPX 쌍둥이: 안내문과 같은 값을 채워도 HWPX 저장·재적재에서 살아남아야 한다.
#[test]
fn value_equal_to_guide_survives_hwpx_roundtrip() {
    let mut doc = load(&sample_bytes());
    doc.set_field_value_by_name_api(FIELD, GUIDE)
        .expect("set_field_value_by_name");
    let saved = doc.export_hwpx().expect("export_hwpx");
    let reloaded = load(&saved);
    assert_eq!(
        field_value(&reloaded, FIELD),
        GUIDE,
        "안내문과 같은 값이 HWPX 저장·재적재에서 유실됐다"
    );
}

/// 파서 축 단독: dirty="1" 인 누름틀은 본문이 안내문과 같아도 로드에서 지워지지 않는다.
#[test]
fn dirty_field_text_survives_load() {
    let doc = load(&with_dirty_flag(&sample_bytes()));
    assert_eq!(
        field_value(&doc, FIELD),
        GUIDE,
        "dirty=\"1\" 누름틀의 텍스트가 로드만으로 유실됐다"
    );
}

/// 저장기 축: fieldBegin 이 dirty 속성을 상태 그대로 방출해야 한다.
#[test]
fn export_emits_dirty_attribute() {
    // 초기 상태 → dirty="0"
    let doc = load(&sample_bytes());
    let xml = section_xml(&doc.export_hwpx().expect("export_hwpx"));
    assert!(
        xml.contains(FIELD_ANCHOR),
        "초기 상태 누름틀은 dirty=\"0\" 로 방출되어야 한다: {}",
        &xml[..xml.len().min(2000)]
    );
    // 채운 상태(비트 15) → dirty="1"
    let mut doc = load(&sample_bytes());
    doc.set_field_value_by_name_api(FIELD, "주식회사 가나다")
        .expect("set_field_value_by_name");
    let xml = section_xml(&doc.export_hwpx().expect("export_hwpx"));
    assert!(
        xml.contains(r#"name="myMsg01" editable="1" dirty="1""#),
        "채운 누름틀은 dirty=\"1\" 로 방출되어야 한다"
    );
}

/// 무회귀: dirty="0" 초기 안내문 잔재의 정규화(제거)는 종전대로 유지된다.
#[test]
fn initial_state_still_normalized_on_load() {
    let doc = load(&sample_bytes());
    assert_eq!(
        field_value(&doc, FIELD),
        "",
        "초기 상태(dirty=\"0\") 안내문 잔재는 로드 시 정규화되어야 한다"
    );
    let reloaded = load(&doc.export_hwpx().expect("export_hwpx"));
    assert_eq!(
        field_value(&reloaded, FIELD),
        "",
        "왕복 후에도 초기 상태 유지"
    );
}
