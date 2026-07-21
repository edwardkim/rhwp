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

#[test]
fn embedded_font_is_emitted_as_data_uri_not_local() {
    let svg = render_with_embed(FontEmbedMode::Subset);
    // 임베디드 face 의 @font-face 가 존재해야 한다.
    assert!(
        svg.contains(EMBEDDED_FACE),
        "SVG 에 임베디드 폰트 face 참조가 있어야 함"
    );
    // 원본 바이트가 data-URI 로 임베딩되어야 한다.
    assert!(
        svg.contains("src: url(\"data:font/"),
        "임베디드 폰트가 data-URI 로 임베딩되어야 함 (local() 폴백 아님). SVG:\n{}",
        &svg[..svg.len().min(1200)]
    );
    // 임베디드 face 에 대한 local()-only @font-face 가 남아 있으면 안 된다.
    let local_only = format!("font-family: \"{EMBEDDED_FACE}\"; src: local(");
    assert!(
        !svg.contains(&local_only),
        "임베디드 폰트가 local() 폴백으로 남으면 안 됨 (#2524)"
    );
}

#[test]
fn embedded_font_embedded_in_style_mode_too() {
    // --font-style(local 참조 전용) 모드라도 미설치 임베디드 폰트는 embed 해야 한다.
    let svg = render_with_embed(FontEmbedMode::Style);
    assert!(
        svg.contains("src: url(\"data:font/"),
        "Style 모드에서도 임베디드 폰트는 data-URI 로 embed 되어야 함 (#2524)"
    );
}
