//! 관계형: 공개 웹폰트 변환과 Native 로컬 우선 공급 계약을 검사한다.
//! 주석 갱신: 2026-11-04 00:44:24 KST; 신규 파일.
//! 합성 글꼴 입력 검사이며 실물 페이지의 일치율이나 고정 px를 주장하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::native_webfonts::{
    decode_font, prepare, NativeWebfontStyle, NativeWebfontSupply,
};
use rhwp::DocumentCore;

fn document_with_face(name: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for faces in &mut document.doc_info.font_faces {
        for face in faces {
            face.name = name.into();
            face.is_embedded = false;
            face.subst_font = None;
        }
    }
    core.set_document(document);
    core
}

#[test]
fn woff2_decodes_to_a_readable_program_without_losing_korean_glyph_advance() {
    let bytes = include_bytes!("../../assets/fonts/NotoSansKR-Regular.woff2");
    let native = decode_font(bytes).expect("공개 WOFF2를 Native SFNT로 변환");
    let face = ttf_parser::Face::parse(&native, 0).unwrap();
    let glyph = face.glyph_index('가').expect("한글 glyph 보존");
    assert!(face.glyph_hor_advance(glyph).is_some_and(|width| width > 0));
    assert!(decode_font(b"wOF2").is_err(), "잘린 입력은 거부");
    let mut oversized = vec![0u8; 48];
    oversized[..4].copy_from_slice(b"wOF2");
    oversized[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(
        decode_font(&oversized).is_err(),
        "과도한 압축 해제 크기는 변환 전에 거부"
    );
}

#[test]
fn local_face_wins_and_generic_fallback_does_not_claim_to_supply_the_unknown_original() {
    let local = document_with_face("Noto Sans KR ExtraLight");
    let supplied_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-ExtraLight.ttf");
    let supply = prepare(&local.document().doc_info, &[supplied_path]).unwrap();
    assert!(supply.mappings.is_empty());
    assert!(
        supply.paths.is_empty(),
        "로컬 원본이 있으면 다운로드하지 않음"
    );
    let unknown = document_with_face("RHWP Missing Font 8533");
    let generic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-Regular.ttf");
    let supply = prepare(&unknown.document().doc_info, &[generic]).unwrap();
    assert_eq!(
        supply
            .mappings
            .get("RHWP Missing Font 8533")
            .map(String::as_str),
        Some("NotoSansKR-Regular")
    );
    assert_eq!(supply.generic_fallbacks, ["RHWP Missing Font 8533"]);
    assert!(
        supply.paths.is_empty(),
        "확인된 로컬 Noto를 다시 다운로드하지 않음"
    );
    assert!(supply
        .unavailable
        .iter()
        .any(|name| name == "RHWP Missing Font 8533"));

    let mut declared = unknown.document().doc_info.clone();
    for face in declared.font_faces.iter_mut().flatten() {
        face.subst_font = Some(rhwp::model::style::SubstFont {
            face: "NotoSansKR-Regular".into(),
            font_type: 1,
            ..Default::default()
        });
    }
    let generic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-Regular.ttf");
    let supplied = prepare(&declared, &[generic]).unwrap();
    assert!(
        supplied.mappings.is_empty() && supplied.generic_fallbacks.is_empty(),
        "설치된 문서 선언 대체는 일반 웹 대체로 덮어쓰지 않음"
    );
}

#[test]
fn svg_font_reference_preserves_local_priority_and_escapes_markup() {
    let mut supply = NativeWebfontSupply::default();
    supply.web_fonts.insert(
        "RHWP <font> & \"name\"".into(),
        "https://cdn.jsdelivr.net/gh/example/fonts@fixed/test.woff2".into(),
    );
    supply.web_font_styles.insert(
        "RHWP <font> & \"name\"".into(),
        NativeWebfontStyle {
            weight: 700,
            italic: false,
            weight_range: None,
        },
    );
    let style = supply.svg_style();
    assert!(style.contains("src:local("));
    assert!(style.find("local(").unwrap() < style.find("url(").unwrap());
    assert!(
        !style.contains("<font>"),
        "글꼴 이름이 SVG 요소로 해석되지 않음"
    );
    assert!(style.contains("test.woff2"));
    assert!(
        style.contains("font-weight:700"),
        "실제 Bold를 Regular로 등록하지 않음"
    );
    assert!(NativeWebfontSupply::default().svg_style().is_empty());
}
