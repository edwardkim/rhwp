//! 독립 Print의 원 글꼴·영문 슬롯 소유를 source 글자와 최종 전진 관계로 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::path::Path;

fn load(path: &str) -> DocumentCore {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("정식 원본 sample");
    DocumentCore::from_bytes(&bytes).expect("원본 문서 로드")
}

/// 전16쪽 Native/fresh 90%와 제목 확대 확인 뒤 추가한다.
/// 독립 Print의 HFT bullet은 인접 굵은 한글과 같은 전각 전진을 갖는다.
#[test]
fn converted_hft_bullet_shares_the_adjacent_fullwidth_advance() {
    let core = load("samples/hwp3-sample-hwp5.hwp");
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(4).unwrap()).unwrap();
    let title = "•서버 측면의 응용 프로그램 계층 작업 할당 방법";
    let rows: Vec<_> = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["text"].as_str() == Some(title))
        .collect();
    assert_eq!(rows.len(), 1, "원문 제목의 누락·중복 없음");
    let run = rows[0];
    assert!(run["bold"].as_bool().unwrap(), "독립 Print의 굵은 제목");
    let xs = run["charX"].as_array().unwrap();
    assert_eq!(xs.len(), title.chars().count() + 1, "scalar 소유 보존");
    let bullet = xs[1].as_f64().unwrap() - xs[0].as_f64().unwrap();
    let hangul = xs[2].as_f64().unwrap() - xs[1].as_f64().unwrap();
    // Print의 기준선 원점 709→805→901.9956, 같은 glyph scale92이다.
    // 절대 위치를 고정하지 않고 실제 글줄에서 두 전진의 관계를 검사한다.
    assert!(
        bullet > hangul * 0.95 && bullet < hangul * 1.05,
        "bullet을 반각으로 축소하면 안 된다: bullet={bullet}, 인접 한글={hangul}"
    );
    assert!(run["fontFamily"]
        .as_str()
        .unwrap()
        .starts_with("한양신명조"));
}

/// 전2쪽 Native/fresh 90%·쪽수 일치 및 대괄호 확대 확인 뒤 추가한다.
/// 명시 영문 대체가 CJK도 포함하더라도 원래 LegacyLatin 슬롯을 잃지 않는다.
#[test]
fn explicit_reference_latin_slot_owns_the_bracket_width_and_paint() {
    let mut core = load("samples/task2097/1741000_project_application.hwp");
    let original_fonts = core.document().doc_info.font_faces.clone();
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/project2097-reference-font-environment.json"
    )).expect("독립 Print 참조 환경");
    core.set_font_environment(Some(environment)).unwrap();
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let runs = layout["runs"].as_array().unwrap();
    let text: String = runs
        .iter()
        .map(|run| run["text"].as_str().unwrap())
        .collect();
    assert!(
        text.contains("※ [  ]에는 해당되는 곳에 √표를 합니다."),
        "안내 문구·체크 기호 보존"
    );
    let brackets: Vec<_> = runs
        .iter()
        .filter(|run| {
            run["charShapeId"].as_u64() == Some(39) && run["text"].as_str().unwrap().contains('[')
        })
        .collect();
    assert_eq!(brackets.len(), 1, "해당 안내문 대괄호 소유가 하나");
    let bracket = brackets[0];
    assert_eq!(
        bracket["fontFamily"].as_str(),
        Some("Batang"),
        "독립 Print의 영문 공급자가 실제 paint run에도 적용되어야 한다"
    );
    assert_eq!(
        bracket["charStart"].as_u64(),
        Some(2),
        "원문 scalar offset 보존"
    );
    let xs = bracket["charX"].as_array().unwrap();
    let width = xs[1].as_f64().unwrap() - xs[0].as_f64().unwrap();
    let em = bracket["fontSize"].as_f64().unwrap();
    // PDF의 Batang bracketleft /Widths는0.5em이다. 원점 px를 역산하지 않는다.
    assert!(
        width > em * 0.45 && width < em * 0.55,
        "독립 Print 반각 관계: bracket={width}, em={em}"
    );
    // 세션의 공급 선택은 원본 face 선언을 바꾸지 않는다.
    for (actual, original) in core
        .document()
        .doc_info
        .font_faces
        .iter()
        .zip(&original_fonts)
    {
        assert_eq!(
            actual.iter().map(|font| &font.name).collect::<Vec<_>>(),
            original.iter().map(|font| &font.name).collect::<Vec<_>>()
        );
    }
}
