//! 검사 유형: 관계형. 선택적 글꼴 등록과 해제의 공개 API 계약이다.
//! 주석 갱신: 2026-10-31T02:27:30+09:00; 직전 파일 커밋: 2026-10-27T12:55:48+09:00 (`0bb537818bc04638bf9e51f335d337f105d56555`).
//! 라이선스를 확인한 저장소 글꼴로 공급·해제 관계를 검사한다. 실물 페이지 좌표나 사설 글꼴을 요구하지 않는다.
use std::collections::HashMap;

use quick_xml::{events::Event, Reader};
use rhwp::{document_core::DocumentCore, paint::RenderProfile};

const FAMILY: &str = "Coverage Source";

fn source_font() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ttfs/opensource/NotoSansKR-Regular.ttf"
    ))
    .expect("저장소의 공개 정체 글꼴")
}

fn document(text: &str, italic: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    let font_id = core.find_or_create_font_id_native(FAMILY);
    core.apply_char_format_native(
        0,
        0,
        0,
        text.chars().count(),
        &format!(r#"{{"fontId":{font_id},"italic":{italic}}}"#),
    )
    .unwrap();
    core
}

fn painted_text_attributes(core: &DocumentCore) -> HashMap<String, String> {
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .unwrap();
    let mut reader = Reader::from_str(&svg);
    let mut texts = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) if element.local_name().as_ref() == "text" => {
                texts.push(
                    element
                        .attributes()
                        .map(|attribute| {
                            let attribute = attribute.unwrap();
                            (
                                attribute.key.as_ref().to_owned(),
                                attribute
                                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                    .unwrap()
                                    .into_owned(),
                            )
                        })
                        .collect(),
                );
            }
            Event::Eof => break,
            _ => {}
        }
    }
    assert_eq!(texts.len(), 1, "검증 문서는 본문 H 하나만 가진다");
    texts.pop().unwrap()
}

#[test]
fn registered_upright_face_resolves_quoted_css_and_keeps_document_source() {
    let mut core = document("H", true);
    let hwp = core.export_hwp_native().unwrap();
    let hwpx = core.export_hwpx_native().unwrap();
    assert!(core
        .register_text_font_glyph_coverage_native(FAMILY, &source_font(), 0)
        .unwrap());

    let attributes = painted_text_attributes(&core);
    assert_eq!(attributes["font-family"], "'coverage source'");
    assert_eq!(attributes["font-style"], "normal");
    assert!(attributes["transform"].contains("matrix("));
    assert_eq!(core.export_hwp_native().unwrap(), hwp);
    assert_eq!(core.export_hwpx_native().unwrap(), hwpx);
}

#[test]
fn oversized_optional_face_keeps_loading_and_invalidates_prior_style_inference() {
    let mut core = document("H", true);
    let hwp = core.export_hwp_native().unwrap();
    core.register_text_font_glyph_coverage_native(FAMILY, &source_font(), 0)
        .unwrap();
    assert_eq!(painted_text_attributes(&core)["font-style"], "normal");

    let oversized = vec![0; 32 * 1024 * 1024 + 1];
    assert!(!core
        .register_text_font_glyph_coverage_native(FAMILY, &oversized, 0)
        .unwrap());
    let fallback = painted_text_attributes(&core);
    assert_ne!(fallback["font-style"], "normal");
    assert!(fallback["font-family"].starts_with("'Coverage Source',"));
    assert!(!fallback
        .get("transform")
        .is_some_and(|transform| transform.contains("matrix(")));

    // 다음 정상 프로그램도 앞서 미검증인 같은 family의 선택을 확정하지 않는다.
    core.register_text_font_glyph_coverage_native(FAMILY, &source_font(), 0)
        .unwrap();
    assert_eq!(painted_text_attributes(&core), fallback);
    assert_eq!(core.export_hwp_native().unwrap(), hwp);
}

#[test]
fn clearing_one_family_preserves_the_other_and_allows_reregistration() {
    let mut core = document("H", true);
    let first_id = core.find_or_create_font_id_native(FAMILY);
    let second_id = core.find_or_create_font_id_native("Coverage Second");
    let program = source_font();
    core.register_text_font_glyph_coverage_native(FAMILY, &program, 0)
        .unwrap();
    core.register_text_font_glyph_coverage_native("Coverage Second", &program, 0)
        .unwrap();
    let first = painted_text_attributes(&core)["font-family"].clone();
    assert_eq!(first, "'coverage source'");
    core.clear_text_font_family_native(FAMILY);
    let after_clear = painted_text_attributes(&core);
    assert_ne!(
        after_clear["font-family"], first,
        "해제된 family 대신 대체 글꼴을 선택한다"
    );
    core.apply_char_format_native(
        0,
        0,
        0,
        1,
        &format!(r#"{{"fontId":{second_id},"italic":true}}"#),
    )
    .unwrap();
    assert_eq!(
        painted_text_attributes(&core)["font-family"],
        "'coverage second'",
        "다른 family는 유지된다"
    );
    core.apply_char_format_native(
        0,
        0,
        0,
        1,
        &format!(r#"{{"fontId":{first_id},"italic":true}}"#),
    )
    .unwrap();
    assert_eq!(
        painted_text_attributes(&core)["font-family"],
        after_clear["font-family"]
    );
    core.register_text_font_glyph_coverage_native(FAMILY, &program, 0)
        .unwrap();
    assert_eq!(
        painted_text_attributes(&core)["font-family"],
        first,
        "해제한 family를 다시 등록할 수 있다"
    );
}

#[test]
fn oversized_optional_face_cannot_reuse_visible_pua_after_later_registration() {
    let mut core = document("\u{1112}\u{119e}\u{11ab}", false);
    let source = core.export_hwp_native().unwrap();
    // 배포 원본의 BMP 전용 영역 표시를 사용해 적재 실패 전후의 선택만 검사한다.
    let program = include_bytes!("../fixtures/fonts/last-resort/LastResortHE-Regular.ttf");
    let face = ttf_parser::Face::parse(program, 0).unwrap();
    let glyph = face
        .glyph_index('\u{f53a}')
        .expect("배포 원본의 전용 영역 글리프");
    assert!(face.glyph_bounding_box(glyph).is_some());
    core.register_text_font_glyph_coverage_native(FAMILY, program, 0)
        .unwrap();
    assert!(core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));

    core.register_text_font_glyph_coverage_native(FAMILY, &vec![0; 32 * 1024 * 1024 + 1], 0)
        .unwrap();
    assert!(!core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));
    core.register_text_font_glyph_coverage_native(FAMILY, program, 0)
        .unwrap();
    assert!(!core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));
    assert_eq!(core.export_hwp_native().unwrap(), source);
}
