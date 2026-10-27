//! 실제 italic 프로그램을 synthetic oblique로 덮지 않는 SVG 공급.
use super::{escape_css_font_name, font_data_uri_format, FontEmbedMode, SvgRenderer};
use base64::Engine;
use std::path::PathBuf;

pub(super) fn append_font_css(css: &mut String, renderer: &SvgRenderer, paths: &[PathBuf]) {
    if renderer.font_embed_mode == FontEmbedMode::None {
        return;
    }
    let mut requests = renderer.font_italic_styles.iter().collect::<Vec<_>>();
    requests.sort_unstable();
    for (family, bold) in requests {
        let names = vec![family.clone()];
        // 이 face가 없는 문자는 원래 family 사다리가 처리한다. 모든 Unicode를
        // 포함하지 않는다는 이유로 실제 Latin italic 프로그램을 버리지 않는다.
        let Some(path) = super::system_fonts::italic_family_file(
            paths,
            &names,
            *bold,
            &std::collections::HashSet::new(),
        ) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let Ok(face) = ttf_parser::Face::parse(&bytes, 0) else {
            continue;
        };
        let source = if renderer.font_embed_mode == FontEmbedMode::Style {
            let Some(name) = face.names().into_iter().find_map(|name| {
                (name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
                    .then(|| name.to_string())
                    .flatten()
            }) else {
                continue;
            };
            format!("local(\"{}\")", escape_css_font_name(&name))
        } else {
            let (mime, format) = font_data_uri_format(&bytes);
            format!(
                "url(\"data:{mime};base64,{}\") format(\"{format}\")",
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            )
        };
        css.push_str(&format!(
            "@font-face {{ font-family: \"{}\"; src: {}; font-weight: {}; font-style: italic; }}\n",
            escape_css_font_name(family),
            source,
            face.weight().to_number()
        ));
    }
}
