//! 선택적 글꼴 등록의 공개 API 계약. 픽셀 좌표나 특정 사설 글꼴을 요구하지 않는다.
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

/// 실제 공개 프로그램의 H 윤곽에 합성 cmap 주소만 부여한다.
/// 원본 파일은 바꾸지 않으며 사설 글꼴·사설 glyph 비트셋을 요구하지 않는다.
fn visible_pua_program() -> Vec<u8> {
    let mut bytes = source_font();
    let glyph = ttf_parser::Face::parse(&bytes, 0)
        .unwrap()
        .glyph_index('H')
        .unwrap()
        .0;
    let be16 = |bytes: &[u8], at| u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap());
    let be32 =
        |bytes: &[u8], at| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
    let cmap = (0..usize::from(be16(&bytes, 4)))
        .map(|index| 12 + index * 16)
        .find(|&at| &bytes[at..at + 4] == b"cmap")
        .map(|at| be32(&bytes, at + 8))
        .unwrap();
    let words = [
        4,
        32,
        0,
        4,
        4,
        1,
        0,
        0xf53a,
        0xffff,
        0,
        0xf53a,
        0xffff,
        glyph.wrapping_sub(0xf53a),
        1,
        0,
        0,
    ];
    let mut changed = 0;
    for index in 0..usize::from(be16(&bytes, cmap + 2)) {
        let at = cmap + be32(&bytes, cmap + 4 + index * 8 + 4);
        if be16(&bytes, at) != 4 {
            continue;
        }
        for (index, word) in words.iter().enumerate() {
            bytes[at + index * 2..at + index * 2 + 2].copy_from_slice(&word.to_be_bytes());
        }
        changed += 1;
    }
    assert!(changed > 0);
    let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
    assert_eq!(parsed.glyph_index('\u{f53a}').unwrap().0, glyph);
    assert!(parsed
        .glyph_bounding_box(ttf_parser::GlyphId(glyph))
        .is_some());
    bytes
}

#[test]
fn oversized_optional_face_cannot_reuse_visible_pua_after_later_registration() {
    let mut core = document("\u{1112}\u{119e}\u{11ab}", false);
    let source = core.export_hwp_native().unwrap();
    let program = visible_pua_program();
    core.register_text_font_glyph_coverage_native(FAMILY, &program, 0)
        .unwrap();
    assert!(core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));

    core.register_text_font_glyph_coverage_native(FAMILY, &vec![0; 32 * 1024 * 1024 + 1], 0)
        .unwrap();
    assert!(!core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));
    core.register_text_font_glyph_coverage_native(FAMILY, &program, 0)
        .unwrap();
    assert!(!core.render_page_svg_native(0).unwrap().contains('\u{f53a}'));
    assert_eq!(core.export_hwp_native().unwrap(), source);
}
