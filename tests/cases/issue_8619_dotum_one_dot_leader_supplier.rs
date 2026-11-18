//! 관계형: 돋움(Dotum)의 `․`(U+2024)는 Haansoft Dotum 이 측정·그리기를 공급한다.
//! 주석 갱신: 2026-11-05T16:40:00+09:00 (새 파일).
//! 입력은 합성 빈 문서의 글꼴 결정 trace 라 이 파일의 assertion 자체는 쪽 시각 출력을 검증하지 않는다.
//! 변경 근거 문서 docs#648 평가결과서(HWP5, 한컴2022 Print, 전2쪽 쪽수 일치)의 2026-11-05 측정
//! raw(96dpi, visual_sweep silhouette_raw_match_percent): 1쪽 Native 93.64% / fresh WASM 96.78%
//! (base 62.12% / 64.38%), 2쪽 각 100.00%; 전2쪽 최저 93.64% / 96.78%.
//! 근거: mydocs/pr/assets/issue_8619/head/{native,wasm}/i648_silhouette.tsv, png/i648_p1_*.png.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;
use serde_json::Value;

/// HDOTUM.TTF `onedotenleader` hmtx 178 / unitsPerEm 1024.
const SUPPLIER_EM: f64 = 178.0 / 1024.0;

fn core(face: &str, text: &str) -> DocumentCore {
    core_with_type(face, text, 1)
}

/// `alt_type`: 0 알 수 없음, 1 TTF, 2 HFT (원본 FaceName 유형).
fn core_with_type(face: &str, text: &str, alt_type: u8) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for font in fonts {
            font.name = face.into();
            font.alt_type = alt_type;
        }
    }
    for style in &mut document.doc_info.char_shapes {
        style.base_size = 1300;
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
fn dotum_missing_one_dot_leader_uses_haansoft_dotum_for_measure_and_paint() {
    let program = core("돋움", "감독․검사‧가");
    let original = program.export_hwp_native().unwrap();
    let dot = record(&program, "․");
    let hangul = record(&program, "감");
    assert_eq!(dot["document"]["face"], "돋움");
    assert_eq!(
        dot["layoutName"]["cssFamilyChain"],
        serde_json::json!(["돋움", "Haansoft Dotum"]),
        "gulim.ttc Dotum 의 U+2024 결측은 독립 Print 의 공급 face 로 그린다"
    );
    assert!(
        (advance(&dot) / advance(&hangul) - SUPPLIER_EM).abs() <= 4.0 / advance(&hangul),
        "HDOTUM 의 자연 전진(178/1024em): 점 {} / 한글 {}",
        advance(&dot),
        advance(&hangul)
    );
    // 반례: 공급자를 독립 Print 로 확인하지 않은 U+2027 은 이 수정으로 바꾸지 않는다.
    let hyphenation = record(&program, "‧");
    assert!(
        (advance(&hyphenation) / advance(&hangul) - SUPPLIER_EM).abs() > 4.0 / advance(&hangul),
        "U+2027 은 HDOTUM 전진으로 옮기지 않는다: {}",
        advance(&hyphenation)
    );
    assert_eq!(program.export_hwp_native().unwrap(), original, "원본 불변");
}

#[test]
fn dotum_supplier_keeps_the_dotumche_contract() {
    // 반례: 같은 gulim.ttc 의 돋움체는 기존 독립 Print 대로 한컴바탕이 공급한다.
    let dotum_che = record(&core("돋움체", "가․가"), "․");
    assert_eq!(
        dotum_che["layoutName"]["cssFamilyChain"],
        serde_json::json!(["돋움체", "Haansoft Batang"]),
        "돋움체 → 한컴바탕 기존 계약"
    );
}

#[test]
fn hft_declared_source_does_not_borrow_the_dotum_program_gap() {
    // 반례: HFT 로 선언한 원본은 대체 메트릭이 가리키는 gulim.ttc 프로그램을 쓰지 않는다.
    // 식중독 보고서(docs#793) 117쪽 `환자집계․분석`의 기호 슬롯 `#세고딕`(HFT)은
    // 독립 한컴2020 Print 가 Type3 전각 점(약 1em)으로 그린다.
    let program = core_with_type("#세고딕", "감독․검사", 2);
    let dot = record(&program, "․");
    let hangul = record(&program, "감");
    assert!(
        (advance(&dot) / advance(&hangul) - SUPPLIER_EM).abs() > 4.0 / advance(&hangul),
        "HFT 원본의 `․`에 HDOTUM 자연 전진을 빌려 오지 않는다: 점 {} / 한글 {}",
        advance(&dot),
        advance(&hangul)
    );
}
