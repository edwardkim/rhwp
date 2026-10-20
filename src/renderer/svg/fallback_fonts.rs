//! 실제 주 글꼴에 없는 문자에만 정식 serif 대체 글꼴을 공급한다.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use base64::Engine;

use super::{
    escape_css_font_name, find_font_file, font_data_uri_format, plan_svg_font_file_lookup,
    svg_outline_font_data, FontEmbedMode, SvgRenderer,
};

pub(super) fn append_missing_serif_font_css(
    css: &mut String,
    renderer: &SvgRenderer,
    font_paths: &[PathBuf],
    embedded_fonts: &HashMap<String, Vec<u8>>,
) {
    const FALLBACK: &str = "Haansoft Batang";
    if renderer.font_embed_mode == FontEmbedMode::None
        || renderer.font_codepoints().contains_key(FALLBACK)
    {
        return;
    }
    let mut missing = HashSet::new();
    for (family, chars) in renderer.font_codepoints() {
        if !crate::renderer::generic_fallback(family).ends_with(",serif") {
            continue;
        }
        // 주 글꼴의 cmap를 실제로 읽을 수 없는 경우에는 누락을 추측하지 않는다.
        let bytes = embedded_fonts.get(family).cloned().or_else(|| {
            find_font_file(&plan_svg_font_file_lookup(family, font_paths, false))
                .and_then(|path| std::fs::read(path).ok())
        });
        let Some(bytes) = bytes else { continue };
        let Ok(face) = ttf_parser::Face::parse(&bytes, 0) else {
            continue;
        };
        missing.extend(
            chars
                .iter()
                .copied()
                .filter(|ch| !ch.is_whitespace() && face.glyph_index(*ch).is_none()),
        );
    }
    if missing.is_empty() {
        return;
    }
    let Some(path) = find_font_file(&plan_svg_font_file_lookup(FALLBACK, font_paths, false)) else {
        return;
    };
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let Ok(face) = ttf_parser::Face::parse(&bytes, 0) else {
        return;
    };
    missing.retain(|ch| face.glyph_index(*ch).is_some());
    if missing.is_empty() {
        return;
    }
    // 원본 파일을 수정하지 않는다. 일반 문자 매핑과 윤곽선은 보존하고,
    // 기존 내장 글꼴 경로로 잘못된 비문자 종료 매핑·bitmap strike만 보정한다.
    let bytes = svg_outline_font_data(&bytes, &missing);
    let (mime, format) = font_data_uri_format(&bytes);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    css.push_str(&format!(
        "@font-face {{ font-family: \"{}\"; src: url(\"data:{};base64,{}\") format(\"{}\"); }}\n",
        escape_css_font_name(FALLBACK),
        mime,
        b64,
        format,
    ));
}
