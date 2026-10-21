//! 독립 TAC26 Print의 원본 번호 메트릭은 공급된 glyph fallback 목록으로 바뀌지 않는다.
//! Stage31: 동일 원본 전66쪽 Native/fresh WASM ≥90%, 26쪽 양쪽 raw100% 및 review/overlay 판독.
//! 독립 Print26쪽798pt 기준선을 ±2CSSpx로 보호하며 원본 번호의 소속·누락·중복도 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerJsonOptions, RenderProfile},
    renderer::font_environment::FontEnvironment,
    DocumentCore,
};
use serde_json::{json, Value};
fn numbers<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(m) => {
            if m.get("type").and_then(Value::as_str) == Some("textRun")
                && m.get("text").and_then(Value::as_str) == Some("- 26 -")
            {
                out.push(v);
            }
            for x in m.values() {
                numbers(x, out)
            }
        }
        Value::Array(a) => {
            for x in a {
                numbers(x, out)
            }
        }
        _ => {}
    }
}
#[test]
fn declared_fallback_preserves_the_original_supplied_number_face_and_baseline() {
    let mut c = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/tac-img-02.hwp"
        ))
        .unwrap(),
    )
    .unwrap();
    c.set_hangul2024_compat(false);
    // 공개 caller 선언으로 TrueType 원본 공급을 모델링한다. 로컬 폰트 설치는 검사 조건이 아니다.
    let environment =
        json!({"id":"number-source-metric","substitutions":{},"trueTypeFaces":["굴림"]});
    c.set_font_environment(Some(
        FontEnvironment::from_json(&environment.to_string()).unwrap(),
    ))
    .unwrap();
    assert_eq!(c.page_count(), 66, "독립 Print의 전체 쪽수");
    let page: Value = serde_json::from_str(
        &c.get_page_layer_tree_with_options_native(
            25,
            RenderProfile::Print,
            LayerJsonOptions {
                omit_image_bytes: true,
                omit_font_bytes: true,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let mut found = vec![];
    numbers(&page["root"], &mut found);
    assert_eq!(found.len(), 1, "빈26쪽 원본 자동번호의 누락·중복 방지");
    let family = found[0]["style"]["fontFamily"].as_str().unwrap();
    assert_eq!(
        family.split(',').next(),
        Some("굴림"),
        "원본 primary 프로그램을 보존한다"
    );
    assert!(
        family.split(',').any(|name| name == "Haansoft Dotum"),
        "실제로 공급 목록을 사용하는 경로를 검사한다: {family}"
    );
    let y = found[0]["placement"]["runToPage"]["f"].as_f64().unwrap();
    // 독립 Hancom2020 Print26쪽 기준선798pt. 후보bbox에서 기대값을 만들지 않는다.
    assert!(
        (y - 798.0 * 96.0 / 72.0).abs() <= 2.0,
        "독립 Print 자동번호 baseline ±2CSSpx: {y}"
    );
}
