//! Issue #1166: HWPX 가로 편집 용지가 세로로 렌더링되는 결함.
//!
//! OWPML pagePr `landscape` 값: WIDELY=세로(Portrait), NARROWLY=가로(Landscape).
//! (hwplib ForSecPr: Portrait→WIDELY, Landscape→NARROWLY.)
//! 종전 HWPX 파서는 landscape 를 무시(false 고정)해 가로 용지가 세로로 렌더됐다.
//!
//! width/height 는 HWP 바이너리와 동일하게 짧은변=width/긴변=height 로 저장되고,
//! landscape=true 일 때 렌더러가 swap 한다.

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn page_def_landscape(doc: &rhwp::wasm_api::HwpDocument) -> bool {
    let json = doc.get_page_def_native(0).expect("page def");
    let key = "\"landscape\":";
    let s = json.find(key).expect("landscape key") + key.len();
    let rest = &json[s..];
    let end = rest.find(|c: char| c == ',' || c == '}').expect("delim");
    rest[..end].trim() == "true"
}

#[test]
fn landscape_001_hwpx_is_landscape() {
    // landscape-001.hwpx 는 가로 편집 용지 (pagePr landscape="NARROWLY").
    // HWPX 파서가 NARROWLY → landscape=true 로 정합해야 한다.
    let doc = load("samples/hwpx/landscape-001.hwpx");
    assert!(
        page_def_landscape(&doc),
        "가로 편집 용지 HWPX(landscape=NARROWLY)가 landscape=true 로 파싱되어야 함"
    );
}

#[test]
fn portrait_hwpx_stays_portrait() {
    // para-001.hwpx 는 세로 용지 (pagePr landscape="WIDELY") — 회귀 가드.
    let doc = load("samples/hwpx/para-001.hwpx");
    assert!(
        !page_def_landscape(&doc),
        "세로 용지 HWPX(landscape=WIDELY)는 landscape=false 여야 함"
    );
}

#[test]
fn landscape_hwp5_matches_hwpx() {
    // 동일 문서의 HWP5(가로)도 landscape=true (HWP5 는 이미 정상, 정합 확인).
    let doc = load("samples/landscape-001.hwp");
    assert!(
        page_def_landscape(&doc),
        "가로 편집 용지 HWP5 는 landscape=true (HWPX 와 정합)"
    );
}

/// HWPX 저장본(zip) 에서 section0.xml 의 pagePr landscape 값을 추출.
fn saved_hwpx_landscape(bytes: &[u8]) -> Option<String> {
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).ok()?;
    use std::io::Read;
    let mut xml = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).ok()?;
        if f.name().contains("section0.xml") {
            f.read_to_string(&mut xml).ok()?;
            break;
        }
    }
    let key = "<hp:pagePr landscape=\"";
    let s = xml.find(key)? + key.len();
    let rest = &xml[s..];
    let e = rest.find('"')?;
    Some(rest[..e].to_string())
}

#[test]
fn landscape_hwpx_roundtrip_preserves_narrowly() {
    // [#1166] 가로 HWPX 로드 → HWPX 저장 시 landscape="NARROWLY" 유지.
    // 한컴 저장 정답지(saved/landscape-001-s.hwpx)도 NARROWLY 로 확인됨.
    let doc = load("samples/hwpx/landscape-001.hwpx");
    let saved = doc.export_hwpx_native().expect("export hwpx");
    let ls = saved_hwpx_landscape(&saved).expect("pagePr landscape");
    assert_eq!(
        ls, "NARROWLY",
        "가로 HWPX 저장 시 pagePr landscape 가 NARROWLY 여야 함 (실제 {ls})"
    );
    // 재파싱 round-trip: 저장본을 다시 로드 → landscape=true
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&saved).expect("reparse");
    assert!(
        page_def_landscape(&reloaded),
        "저장본 재로드 시 landscape=true 유지"
    );
}

#[test]
fn portrait_hwpx_roundtrip_preserves_widely() {
    // 세로 HWPX → 저장 시 WIDELY 유지 (회귀 가드).
    let doc = load("samples/hwpx/para-001.hwpx");
    let saved = doc.export_hwpx_native().expect("export hwpx");
    let ls = saved_hwpx_landscape(&saved).expect("pagePr landscape");
    assert_eq!(ls, "WIDELY", "세로 HWPX 저장 시 WIDELY 유지 (실제 {ls})");
}

#[test]
fn landscape_hwpx_to_hwp5_save_preserves_landscape() {
    // [#1166] 가로 HWPX 로드 → HWP5(saveDocument) 저장 → 재파싱 시 landscape=true.
    // 작업지시자 보고: HWPX→HWP 저장이 가로/세로를 반영하지 않음.
    let mut doc = load("samples/hwpx/landscape-001.hwpx");
    let hwp5 = doc.export_hwp_with_adapter().expect("save hwp5");
    let reloaded = rhwp::wasm_api::HwpDocument::from_bytes(&hwp5).expect("reparse hwp5");
    assert!(
        page_def_landscape(&reloaded),
        "HWPX→HWP5 저장 후 재파싱 시 landscape=true 유지되어야 함"
    );
}
