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

// =====================================================================
// [#3545 잔여 축] 초기 상태(dirty="0") 안내문 본문 run 의 파일 수준 보존
//
// 한컴 정준형(form-01.hwpx 실물)은 미기입 누름틀의 안내문을 **본문 run 으로 파일에
// 유지**하고 렌더/인쇄에서만 구분 취급한다. rhwp 는 적재 정규화가 이 텍스트를 IR 에서
// 지우므로, 저장기가 잔재를 복원 방출하지 않으면 저장할 때마다 파일에서 영구 소실된다
// (XSD 는 통과하는 조용한 내용 소실 — 재적재 시 다시 지워져 값 API 표면에는 안 보인다).
// =====================================================================

/// form-01.hwpx 원본의 안내문 본문 run (charPrIDRef="6" 전용 run 내부).
const BODY_GUIDE_RUN: &str = "<hp:t>여기에 입력</hp:t>";

/// 저장 축: 초기 상태 누름틀의 안내문 본문 run 이 저장본에 1회 보존되어야 한다.
#[test]
fn initial_guide_body_run_survives_hwpx_save() {
    let doc = load(&sample_bytes());
    let xml = section_xml(&doc.export_hwpx().expect("export_hwpx"));
    assert_eq!(
        xml.matches(BODY_GUIDE_RUN).count(),
        1,
        "초기 상태 누름틀의 안내문 본문 run 이 저장에서 소실/중복됐다"
    );
    // 원본 형상 그대로 — 잔재를 담던 run 의 charPrIDRef 까지 복원되어야 서식이 산다.
    assert!(
        xml.contains(r#"<hp:run charPrIDRef="6"><hp:t>여기에 입력</hp:t></hp:run>"#),
        "복원된 안내문 run 의 char shape 가 원본(charPrIDRef=\"6\")과 다르다"
    );
    assert!(
        xml.contains(FIELD_ANCHOR),
        "잔재 복원은 초기 상태(dirty=\"0\") 표식을 바꾸면 안 된다"
    );
}

/// 고정점: 저장→재적재→재저장에서도 본문 run 이 1회 유지되고, 값 API 는 빈 값을 유지한다.
#[test]
fn initial_guide_body_run_save_reload_save_fixed_point() {
    let doc = load(&sample_bytes());
    let saved1 = doc.export_hwpx().expect("export 1");
    let reloaded = load(&saved1);
    assert_eq!(
        field_value(&reloaded, FIELD),
        "",
        "재적재 후에도 필드 값 API 는 빈 값이어야 한다 (정규화 계약 유지)"
    );
    let xml2 = section_xml(&reloaded.export_hwpx().expect("export 2"));
    assert_eq!(
        xml2.matches(BODY_GUIDE_RUN).count(),
        1,
        "저장→재적재→재저장 고정점에서 안내문 본문 run 이 소실/중복됐다"
    );
}

/// 무회귀 가드: 채운 필드(dirty="1")는 실값 run 만 방출한다 — 잔재 복원이 중복 주입되면 안 된다.
#[test]
fn filled_field_body_run_not_duplicated() {
    let mut doc = load(&sample_bytes());
    doc.set_field_value_by_name_api(FIELD, GUIDE)
        .expect("set_field_value_by_name");
    let xml = section_xml(&doc.export_hwpx().expect("export_hwpx"));
    assert_eq!(
        xml.matches(BODY_GUIDE_RUN).count(),
        1,
        "채운 필드의 값 run 은 정확히 1회여야 한다 (잔재 복원 중복 주입 금지)"
    );
    assert!(
        xml.contains(r#"name="myMsg01" editable="1" dirty="1""#),
        "채운 필드는 dirty=\"1\" 이어야 한다"
    );
}

/// 실물 행정 서식 코퍼스 대표본(issue1893 fixture): 안내문 run 보존 + 원본부터 빈 스팬은
/// 잔재가 없으므로 텍스트를 주입하면 안 된다.
#[test]
fn gov_form_guide_runs_preserved_and_empty_spans_untouched() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1893_clickhere_field_roundtrip.hwpx");
    let bytes = std::fs::read(&path).expect("read issue1893 fixture");
    let doc = load(&bytes);
    let xml = section_xml(&doc.export_hwpx().expect("export_hwpx"));

    // 소속관서 필드(id=1549188898): 원본은 begin~end 사이에 <hp:t>소속관서</hp:t> 보유.
    let span = field_span(&xml, "1549188898");
    assert!(
        span.contains(r#"<hp:run charPrIDRef="20"><hp:t>소속관서</hp:t></hp:run>"#),
        "초기 상태 누름틀(소속관서)의 안내문 본문 run 이 원본 형상으로 복원되지 않았다: {span}"
    );

    // id=1549188905: 원본부터 빈 스팬(begin 직후 end, 텍스트 run 없음) — 주입 금지.
    let empty_span = field_span(&xml, "1549188905");
    assert!(
        !contains_nonempty_hp_t(empty_span),
        "원본부터 빈 누름틀 스팬에 텍스트가 주입됐다: {empty_span}"
    );
}

/// fieldBegin(id=..)부터 대응 fieldEnd(beginIDRef=..) 직전까지의 XML 조각.
fn field_span<'a>(xml: &'a str, id: &str) -> &'a str {
    let begin = xml
        .find(&format!(r#"id="{id}""#))
        .unwrap_or_else(|| panic!("fieldBegin id={id} 없음"));
    let end = xml
        .find(&format!(r#"beginIDRef="{id}""#))
        .unwrap_or_else(|| panic!("fieldEnd beginIDRef={id} 없음"));
    assert!(begin < end, "fieldBegin 이 fieldEnd 앞이어야 함 (id={id})");
    &xml[begin..end]
}

/// 조각 안에 내용이 있는 `<hp:t>` 가 존재하는지 (빈 `<hp:t></hp:t>`/`<hp:t/>` 는 무시).
fn contains_nonempty_hp_t(fragment: &str) -> bool {
    let mut rest = fragment;
    while let Some(i) = rest.find("<hp:t>") {
        rest = &rest[i + "<hp:t>".len()..];
        if !rest.starts_with("</hp:t>") {
            return true;
        }
    }
    false
}
