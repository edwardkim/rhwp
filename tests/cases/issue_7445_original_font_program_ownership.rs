//! 원본의 부분 글리프 전진과 기호 슬롯은 공개 글꼴 결정 API로 검증한다.
//! 물리 페이지의 임시 배치를 고정하지 않는 프로그램 메트릭 계약이다.
//! 독립 Print 프로그램과 전쪽/중요 px 증적은 source-program-and-inline-spacing/validation.json.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::Value;

fn core(face: &str, text: &str, symbol_face: Option<&str>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for font in fonts {
            font.name = face.into();
            font.alt_type = 1;
        }
    }
    if let Some(symbol) = symbol_face {
        for font in &mut document.doc_info.font_faces[5] {
            font.name = symbol.into();
        }
    }
    for style in &mut document.doc_info.char_shapes {
        style.base_size = 7500;
        style.ratios = [100; 7];
        style.spacings = [0; 7];
        style.relative_sizes = [100; 7];
    }
    core.set_document(document);
    core
}

fn record(core: &DocumentCore, character: &str) -> Value {
    let value: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":100}"#)
            .unwrap(),
    )
    .unwrap();
    value["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == character)
        .expect("원본 글자 기록")
        .clone()
}

fn advance(record: &Value) -> f64 {
    record["layoutMetric"]["baseAdvanceHwpunit"]
        .as_f64()
        .unwrap()
}

#[test]
fn gulim_missing_point_publishes_the_measured_supplier_to_paint() {
    // 프로그램 cmap·독립 PDF 공급자 계약이며 법원 서식의 미달 배치를 고정하지 않는다.
    let mut program = core("굴림", "가․가", None);
    let original = program.export_hwp_native().unwrap();
    let dot = record(&program, "․");
    assert_eq!(dot["document"]["face"], "굴림");
    assert_eq!(
        dot["layoutName"]["cssFamilyChain"],
        serde_json::json!(["굴림", "Haansoft Dotum"]),
        "확인된 Gulim cmap 결측은 독립 Print의 측정 공급자에 연결"
    );
    let hangul = record(&program, "가");
    assert!(
        (advance(&dot) / advance(&hangul) - 178.0 / 1024.0).abs() <= 4.0 / advance(&hangul),
        "실제 HDOTUM 프로그램의 자연 전진을 유지"
    );
    program
        .set_font_environment(Some(
            FontEnvironment::from_json(
                r#"{"id":"caller-gulim-glyphs","substitutions":{},"glyphFallbacks":{"굴림":["HCR Dotum"]}}"#,
            )
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(
        record(&program, "․")["layoutName"]["cssFamilyChain"],
        serde_json::json!(["굴림", "HCR Dotum"]),
        "호출자가 지정한 공급 체인은 기본 체인보다 우선"
    );
    assert_eq!(program.export_hwp_native().unwrap(), original);
}

#[test]
fn measured_original_glyph_survives_legacy_type1_display_fallback() {
    let core = core("HCI Poppy", "A/A·", None);
    let slash = record(&core, "/");
    assert_eq!(
        slash["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    // 독립2020/2024 Print의 동일 CharProc SHA와 Widths=605/1000em.
    // 기존4HU 장치 격자의 반 격자만 허용하며 전체 HFT 표를 승격하지 않는다.
    assert!((advance(&slash) - 7500.0 * 0.605).abs() <= 2.0);
    let fallback = core_with_palatino_program();
    for character in ["A", "·"] {
        assert_eq!(
            advance(&record(&core, character)),
            advance(&record(&fallback, character)),
            "부분 표에 없는 글자는 표시 프로그램의 폭을 유지함:{character}"
        );
    }
}

fn core_with_palatino_program() -> DocumentCore {
    core("Palatino Linotype", "A/A·", None)
}

#[test]
fn explicit_true_type_program_keeps_its_own_slash_advance() {
    let mut core = core("HCI Poppy", "A/A·", None);
    let original = core.export_hwp_native().unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"verified-palatino-program","substitutions":{"HCI Poppy":"Palatino Linotype"},"trueTypeFaces":["Palatino Linotype"]}"#,
    ).unwrap())).unwrap();
    let slash = record(&core, "/");
    assert_eq!(
        advance(&slash),
        advance(&record(&core_with_palatino_program(), "/"))
    );
    assert_ne!(
        slash["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "선택 환경은 원본을 수정하지 않음"
    );
}

#[test]
fn onepoint_uses_its_symbol_program_before_glyph_resolution() {
    let core = core("휴먼명조", "가․나", Some("한양신명조"));
    let dot = record(&core, "․");
    assert_eq!(
        dot["document"]["face"], "한양신명조",
        "독립 Type3의 원본 기호 슬롯"
    );
    assert!(
        (advance(&dot) - 7500.0).abs() <= 2.0,
        "독립 전각 프로그램 전진"
    );
}

#[test]
fn explicit_onepoint_provider_uses_its_own_advance() {
    let mut core = core("휴먼명조", "가․나", None);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"selected-provider","substitutions":{},"trueTypeFaces":["휴먼명조"],"glyphFallbacks":{"휴먼명조":["Haansoft Batang"]}}"#,
    ).unwrap())).unwrap();
    let dot = record(&core, "․");
    assert!(
        (advance(&dot) - 7500.0 * 178.0 / 1024.0).abs() <= 2.0,
        "명시 공급자의 자연 전진"
    );
    assert_ne!(dot["layoutMetric"]["widthSource"], "heuristicFullwidth");
}
