//! 관계형.
//! 주석 갱신: 2026-11-07T03:36:59+09:00.
//! 입력은 공개 HWP5 문서의 글꼴 레코드만 바꾼 합성 문서와 저장소 fixture 글꼴이라 이 검사 자체의
//! 시각 측정은 해당하지 않는다. 같은 결함의 실물 문서 측정은 다음과 같다.
//! 2026-11-07 측정 raw(source 75555763c4, 96dpi, visual_sweep silhouette_raw_match_percent, 한컴 2020 Print PDF,
//! 서울한강체·서울남산체 공급): sign_board_register_card.hwp 1쪽 Native 100.00 / fresh WASM 100.00,
//! 전 1쪽 최저 100.00 / 100.00(수정 전 90.30 / 90.84), PDF와 쪽수 일치.
//! 근거: mydocs/pr/assets/issue_8661/README.md.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::model::style::SubstFont;
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::native_webfonts;
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;
use std::path::PathBuf;

const LEGACY_FAMILY: &str = "검사용 가족 이름";
const FULL_FACE: &str = "검사용 전체 얼굴 이름";
const UNSUPPLIED: &str = "검사용 미공급 이름";
const POSTSCRIPT_NAME: &str = "RhwpFamilyContract-Regular";
const FIXTURE_A_ADVANCE_EM: f64 = 0.608;
const HEURISTIC_HALF_EM: f64 = 0.5;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/native_font_family_names")
}

/// 공개 HWP5 문서의 글꼴 레코드만 바꾼 합성 입력. 저장 줄 기대값은 없다.
fn hwp5_document(face: &str, declared_type: u8) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public HWP5 document");
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = "A".into();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0];
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    paragraph.line_segs.clear();
    doc.sections[0].paragraphs = vec![paragraph];
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = face.into();
        font.alt_type = declared_type;
        font.is_embedded = false;
        font.alt_name = None;
        font.default_name = None;
        font.subst_font = None;
    }
    core.set_document(doc);
    core
}

/// 실제 공급한 글꼴을 준비한 뒤 합성 글자 `A`의 측정 결정을 읽는다.
fn measure_a(core: &mut DocumentCore) -> (String, f64) {
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[fixture_dir()]);
    renderer.prepare_native_otf_fonts(core).expect("prepare");
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(core, 0, r#"{"maxCharacters":64}"#)
            .expect("renderer-bound font observation"),
    )
    .expect("trace JSON");
    let record = trace["records"]
        .as_array()
        .expect("font records")
        .iter()
        .find(|record| record["source"]["character"] == "A")
        .expect("source character")
        .clone();
    let metric = &record["layoutMetric"];
    (
        metric["widthSource"].as_str().expect("width source").into(),
        metric["baseAdvanceHwpunit"].as_f64().expect("advance"),
    )
}

#[test]
fn hwp5_ttf_without_metric_table_measures_with_the_supplied_program() {
    let (source, supplied) = measure_a(&mut hwp5_document(LEGACY_FAMILY, 1));
    assert_eq!(source, "nativeTrueType");
    // 공급 없는 같은 입력의 추정 폭이 대조군이다. 두 전진의 비는 글자 크기와 무관하다.
    let (control_source, guessed) = measure_a(&mut hwp5_document(UNSUPPLIED, 1));
    assert_eq!(control_source, "heuristicHalfwidth");
    let ratio = supplied / guessed;
    let expected = FIXTURE_A_ADVANCE_EM / HEURISTIC_HALF_EM;
    assert!(
        (ratio - expected).abs() < 0.01,
        "공급 프로그램의 hmtx 비율이어야 한다: {ratio} != {expected}"
    );
}

#[test]
fn declared_hft_face_keeps_the_existing_estimate() {
    let (source, _) = measure_a(&mut hwp5_document(LEGACY_FAMILY, 2));
    assert_ne!(source, "nativeTrueType");
}

#[test]
fn hwp5_face_with_a_metric_table_stays_out_of_the_truetype_request() {
    // 폭 표가 있는 face 의 HWP5 하위 비트만으로 HFT 공급 순서를 덮어쓰지 않는다.
    let core = hwp5_document("한양신명조", 1);
    assert!(!core
        .native_otf_font_requests()
        .iter()
        .any(|request| request.source_ttf));
    let core = hwp5_document(LEGACY_FAMILY, 1);
    assert!(core
        .native_otf_font_requests()
        .iter()
        .any(|request| request.source_ttf));
}

#[test]
fn hwp5_postscript_name_does_not_select_the_truetype_program() {
    // 한/글은 HWP5 글꼴 이름을 family로 찾는다. 같은 PostScript 이름의 프로그램은
    // 그 글꼴이 아니므로 전진을 가져오지 않는다.
    let (source, _) = measure_a(&mut hwp5_document(POSTSCRIPT_NAME, 1));
    assert_ne!(source, "nativeTrueType");
}

#[test]
fn supplied_family_name_id_1_is_not_reported_unavailable() {
    let core = hwp5_document(LEGACY_FAMILY, 1);
    let supply =
        native_webfonts::prepare(&core.document().doc_info, &[fixture_dir()]).expect("font supply");
    assert!(
        !supply.unavailable.iter().any(|name| name == LEGACY_FAMILY),
        "name ID 16 에 가려진 ID 1 family 도 공급된 글꼴이다"
    );
    assert!(!supply.mappings.contains_key(LEGACY_FAMILY));
}

#[test]
fn full_face_name_is_not_a_supplied_family() {
    let core = hwp5_document(FULL_FACE, 1);
    let supply =
        native_webfonts::prepare(&core.document().doc_info, &[fixture_dir()]).expect("font supply");
    assert!(supply.unavailable.iter().any(|name| name == FULL_FACE));
}

#[test]
fn supplied_family_name_id_1_keeps_the_primary_over_the_document_substitute() {
    let mut core = hwp5_document(LEGACY_FAMILY, 1);
    let mut doc = core.document().clone();
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.subst_font = Some(SubstFont {
            face: "Noto Sans KR".into(),
            font_type: 1,
            ..Default::default()
        });
    }
    core.set_document(doc);
    let environment = FontEnvironment::from_native_document_substitutions(
        &core.document().doc_info,
        &[fixture_dir()],
    )
    .expect("environment");
    let environment = serde_json::to_value(environment).expect("environment JSON");
    assert!(
        environment["substitutions"][LEGACY_FAMILY].is_null(),
        "공급된 주 글꼴을 문서 대체로 바꾸면 안 된다: {environment}"
    );
    assert!(
        environment["documentSubstitutions"]
            .as_array()
            .is_none_or(|entries| entries.iter().all(|entry| entry["source"] != LEGACY_FAMILY)),
        "공급된 주 글꼴을 슬롯 대체로 바꾸면 안 된다: {environment}"
    );
}
