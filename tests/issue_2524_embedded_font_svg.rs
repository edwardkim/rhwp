//! [#2524] 문서 임베디드(BinData) 폰트를 export-svg 폰트 임베딩에서 실제로
//! data-URI 로 방출하는지 회귀 가드.
//!
//! 종전: SVG 폰트 임베더(`generate_font_style`)가 `find_font_file`(디스크)만
//! 조회 → 미설치 임베디드 폰트는 `src: local(...)` 폴백 → blink(chrome) 가
//! 해결 못해 글리프 두부(□). 샘플 `render-p35-font-native-bitmap.hwpx` 는
//! 폰트 "RHWP Bitmap SVG Glyph Smoke" 를 BinData 에 임베딩(isEmbedded="1").
//!
//! 수정 후: 문서 임베디드 폰트를 face명→bytes 로 수집해 임베드 모드에서
//! `src: url("data:font/...;base64,...")` 로 원본 전체 임베딩한다.

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::svg::FontEmbedMode;

const SAMPLE: &str = "samples/render-p35-font-native-bitmap.hwpx";
const EMBEDDED_FACE: &str = "RHWP Bitmap SVG Glyph Smoke";

fn render_with_embed(mode: FontEmbedMode) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    core.render_page_svg_with_fonts(0, mode, &[])
        .expect("render svg with fonts")
}

fn embedded_face_rule(svg: &str) -> &str {
    let prefix = format!("@font-face {{ font-family: \"{EMBEDDED_FACE}\";");
    let start = svg
        .find(&prefix)
        .expect("임베디드 face의 @font-face 규칙이 있어야 함");
    let end = svg[start..]
        .find('}')
        .expect("임베디드 face의 @font-face 규칙이 닫혀야 함");
    &svg[start..=start + end]
}

#[test]
fn embedded_font_is_emitted_as_data_uri_not_local() {
    let svg = render_with_embed(FontEmbedMode::Subset);
    let rule = embedded_face_rule(&svg);
    // 원본 바이트가 요청한 face의 data-URI 로 임베딩되어야 한다.
    assert!(
        rule.contains("src: url(\"data:font/"),
        "임베디드 face가 data-URI 로 임베딩되어야 함 (local() 폴백 아님). 규칙: {rule}"
    );
    assert!(
        !rule.contains("local("),
        "임베디드 face가 local() 폴백을 함께 쓰면 안 됨 (#2524). 규칙: {rule}"
    );
}

#[test]
fn embedded_font_embedded_in_style_mode_too() {
    // --font-style(local 참조 전용) 모드라도 미설치 임베디드 폰트는 embed 해야 한다.
    let svg = render_with_embed(FontEmbedMode::Style);
    let rule = embedded_face_rule(&svg);
    assert!(
        rule.contains("src: url(\"data:font/"),
        "Style 모드에서도 요청한 embedded face는 data-URI 로 embed 되어야 함 (#2524). 규칙: {rule}"
    );
    assert!(
        !rule.contains("local("),
        "Style 모드의 embedded face가 local() 폴백을 쓰면 안 됨 (#2524). 규칙: {rule}"
    );
}

#[test]
fn embedded_font_embedded_in_full_mode_too() {
    let svg = render_with_embed(FontEmbedMode::Full);
    let rule = embedded_face_rule(&svg);
    assert!(
        rule.contains("src: url(\"data:font/"),
        "Full 모드에서도 요청한 embedded face는 data-URI 로 embed 되어야 함 (#2524). 규칙: {rule}"
    );
    assert!(
        !rule.contains("local("),
        "Full 모드의 embedded face가 local() 폴백을 쓰면 안 됨 (#2524). 규칙: {rule}"
    );
}

#[test]
fn embedded_font_is_preserved_in_profiled_print_svg() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("render profiled print svg");
    let rule = embedded_face_rule(&svg);
    assert!(
        rule.contains("src: url(\"data:font/"),
        "print profile도 #2524 embedded face를 data-URI로 보존해야 함. 규칙: {rule}"
    );
    assert!(
        !rule.contains("local("),
        "print profile의 embedded face가 local() fallback을 쓰면 안 됨. 규칙: {rule}"
    );
}
