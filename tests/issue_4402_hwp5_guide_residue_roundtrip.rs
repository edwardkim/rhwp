//! [Issue #4402] HWP5 저장 경로가 미기입 누름틀 안내문(`Field.guide_residue`)을
//! 복원하지 않는다 — HWPX 축만 #3545 로 해소.
//!
//! `#3545` 는 이 결함을 HWPX 축에서 고쳤다: 적재 정규화(`clear_initial_field_texts`,
//! `document_core::commands::document`)가 미기입 누름틀(ClickHere, 수정됨 표식 bit 15 == 0)의
//! 안내문 본문을 `para.text` 에서 지우고 `Field.guide_residue`(`char_shape_id` 동반)에
//! 보관한 뒤, HWPX 저장기(`emit_guide_residue`, `serializer/hwpx/section.rs`)가 그 잔재를
//! 저장 시점에 되살린다. HWP5 저장 경로(`serialize_para_text`,
//! `serializer/body_text.rs`)에는 그 배선이 없어, HWP5 로 저장(또는 HWP→HWPX→HWP5 왕복,
//! HWPX 원본을 HWP5 로 `convert`)만 해도 안내문이 **파일에서 영구 소실**됐다.
//!
//! 재현: `samples/form-01.hwp` (HWP5 네이티브, ClickHere 필드 `myMsg01` 안내문
//! "여기에 입력" 이 본문 run 으로 실재)를 로드 → 저장 → raw 파서(정규화를 거치지 않는
//! `parser::parse_hwp`)로 재파싱하면, 수정 전에는 안내문 텍스트가 파일 어디에도 없다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const SAMPLE: &str = "samples/form-01.hwp";
const FIELD: &str = "myMsg01";
/// `samples/form-01.hwp` 의 `myMsg01` 안내문. 본문 텍스트도 이 값과 같다(초기 상태).
const GUIDE: &str = "여기에 입력";

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

/// 저장본을 **정규화를 거치지 않는** raw 파서로 다시 읽어, 실제 파일 바이트 수준에서
/// 안내문이 살아있는지 확인한다 — `rhwp ir-diff` 가 이 결함을 발견한 것과 같은 층위다.
/// `document_core` 를 거친 값 API(`field_value`)는 항상 정규화되어 빈 값이라 이 결함을
/// 가리지 못한다(왕복해서 재적재해도 다시 지워지므로).
fn raw_paragraph_texts(bytes: &[u8]) -> Vec<String> {
    let doc = rhwp::parser::parse_hwp(bytes).expect("raw parse");
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter().map(|p| p.text.clone()))
        .collect()
}

fn count_guide_occurrences(texts: &[String]) -> usize {
    texts.iter().filter(|t| t.contains(GUIDE)).count()
}

/// 무회귀: 값 API 계약(정규화)은 그대로 유지된다 — 안내문 복원은 IR 이 아니라 저장 바이트
/// 층위에서만 일어나야 한다.
#[test]
fn initial_state_still_normalized_on_load() {
    let doc = load(&sample_bytes());
    assert_eq!(
        field_value(&doc, FIELD),
        "",
        "초기 상태 안내문 잔재는 로드 시 정규화되어야 한다"
    );
}

/// 저장 축(직접 저장, 편집 없음): `export_hwp` 는 편집이 전혀 없으면 `section.raw_stream`
/// 원본 바이트를 통째로 복사하는 지름길을 탄다(`serialize_section`) — 이 경로는
/// `serialize_para_text` 를 아예 거치지 않으므로 이 결함을 가리지 못한다(안내문이 원본에
/// 이미 있으니 통과해 버린다). 이 테스트는 그 지름길이 계속 성립하는지 확인하는 무회귀
/// 가드일 뿐, 이 이슈의 회귀 판정에는 아래 `hwp_to_hwpx_to_hwp5_roundtrip_*` /
/// `sibling_field_edit_*` 테스트를 본다(둘 다 `raw_stream` 이 무효화되어 실제
/// `serialize_para_text` 경로를 태운다).
#[test]
fn initial_guide_text_survives_hwp5_save_no_edit_raw_passthrough() {
    let mut doc = load(&sample_bytes());
    let saved = doc.export_hwp().expect("export_hwp");
    let texts = raw_paragraph_texts(&saved);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "초기 상태 누름틀의 안내문이 HWP5 저장본에서 소실/중복됐다: {texts:?}"
    );
}

/// 저장 축(핵심 재현): **다른** 필드를 채우는 편집이 `section.raw_stream` 을 무효화해
/// (`wasm_api::set_field_value_by_name_api` 는 대상 필드를 찾을 때까지 훑는 모든 섹션의
/// `raw_stream` 을 지운다) `export_hwp` 가 `serialize_para_text` 로 진짜 재구성하게
/// 강제한다 — 실물 서식(일부 필드는 채우고 일부는 비워 둔 채 저장)과 같은 조건이다.
/// 대상 누름틀(회사명) 자신은 건드리지 않으므로 dirty 표식도 안 선다.
///
/// 수정 전 실패: 회사명 문단의 안내문 "여기에 입력" 이 저장본 어디에도 없다(0회).
#[test]
fn sibling_field_edit_forces_reserialize_and_guide_text_survives() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/field-01-memo.hwp");
    let bytes = std::fs::read(&path).expect("read samples/field-01-memo.hwp");
    let mut doc = load(&bytes);
    // 회사명은 그대로 두고 작성자만 채운다 — raw_stream 무효화 유발 + 회사명 dirty 회피.
    doc.set_field_value_by_name_api("작성자", "홍길동")
        .expect("set_field_value_by_name(작성자)");
    let saved = doc.export_hwp().expect("export_hwp");
    let texts = raw_paragraph_texts(&saved);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "다른 필드(작성자) 편집으로 재구성된 HWP5 저장본에서 회사명의 안내문이 소실/중복됐다: {texts:?}"
    );
}

/// HWP → HWPX → HWP5 왕복 (이슈가 실측으로 제시한 경로). #3545 가 hop1(HWP→HWPX)은 이미
/// 고쳤으므로, 이 테스트의 실질 관심사는 hop2(HWPX→HWP5)다. HWPX 경유는 raw_stream 이
/// 없어 편집 여부와 무관하게 항상 `serialize_para_text` 를 태운다.
#[test]
fn hwp_to_hwpx_to_hwp5_roundtrip_preserves_guide_text() {
    let doc = load(&sample_bytes());
    let hwpx = doc.export_hwpx().expect("export_hwpx (hop1)");
    let mut reloaded_from_hwpx = load(&hwpx);
    let hwp5 = reloaded_from_hwpx.export_hwp().expect("export_hwp (hop2)");
    let texts = raw_paragraph_texts(&hwp5);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "HWP→HWPX→HWP5 왕복에서 안내문이 소실/중복됐다: {texts:?}"
    );
}

/// 고정점: HWPX 경유 저장 → 재적재(정규화) → 다시 HWPX 경유 저장에서도 안내문이 1회
/// 유지되고, 값 API 는 빈 값을 유지한다(정규화 계약 불변). 두 hop 모두 raw_stream 이
/// 없는 HWPX 를 거치므로 편집 여부와 무관하게 `serialize_para_text` 를 태운다.
#[test]
fn initial_guide_text_save_reload_save_fixed_point() {
    let doc = load(&sample_bytes());
    let hwpx1 = doc.export_hwpx().expect("export_hwpx 1");
    let mut reloaded = load(&hwpx1);
    assert_eq!(
        field_value(&reloaded, FIELD),
        "",
        "재적재 후에도 필드 값 API 는 빈 값이어야 한다 (정규화 계약 유지)"
    );
    let saved2 = reloaded.export_hwp().expect("export_hwp 2");
    let texts = raw_paragraph_texts(&saved2);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "저장→재적재→재저장 고정점에서 안내문이 소실/중복됐다: {texts:?}"
    );
}

/// 무회귀 가드: 값을 채운 필드는 실값 run 만 방출한다 — 잔재 복원이 중복 주입되면 안 된다.
#[test]
fn filled_field_not_duplicated() {
    let mut doc = load(&sample_bytes());
    doc.set_field_value_by_name_api(FIELD, GUIDE)
        .expect("set_field_value_by_name");
    let saved = doc.export_hwp().expect("export_hwp");
    let texts = raw_paragraph_texts(&saved);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "채운 필드의 값이 중복 주입됐다: {texts:?}"
    );
}

/// 두 번째 실물 샘플(form-02.hwp) — 단일 픽스처 우연 적합 배제. HWP→HWPX→HWP5 왕복으로
/// raw_stream 지름길 없이 `serialize_para_text` 를 강제한다.
#[test]
fn form_02_survives_hwp_to_hwpx_to_hwp5_roundtrip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/form-02.hwp");
    let bytes = std::fs::read(&path).expect("read samples/form-02.hwp");
    let doc = load(&bytes);
    let hwpx = doc.export_hwpx().expect("export_hwpx (hop1)");
    let mut reloaded_from_hwpx = load(&hwpx);
    let hwp5 = reloaded_from_hwpx.export_hwp().expect("export_hwp (hop2)");
    let texts = raw_paragraph_texts(&hwp5);
    assert_eq!(
        count_guide_occurrences(&texts),
        1,
        "form-02.hwp 안내문이 HWP→HWPX→HWP5 왕복에서 소실/중복됐다: {texts:?}"
    );
}
