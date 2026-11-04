//! 실제 공급 face가 있는 경우에만 호환 PUA 글리프를 선택한다.
use super::SvgRenderer;
use crate::renderer::TextStyle;
use std::borrow::Cow;

impl SvgRenderer {
    pub(super) fn prepare_pua_glyphs(&mut self, text: &str, style: &TextStyle) {
        if !style.italic && !crate::renderer::contains_old_hangul_jamo(text) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        for family in crate::renderer::internal_font_family_members(
            &crate::renderer::render_font_family_chain_for_weight(
                &style.font_family,
                style.is_visually_bold(),
            ),
        ) {
            if !style.italic
                && self
                    .source_font_coverage
                    .has_style(family, style.is_visually_bold(), false)
            {
                break;
            }
            if style.italic
                && self
                    .source_font_coverage
                    .text_faces
                    .select(
                        &crate::renderer::render_font_family_chain_for_weight(
                            &style.font_family,
                            style.is_visually_bold(),
                        ),
                        text,
                        style.is_visually_bold(),
                        true,
                    )
                    .is_some()
            {
                break;
            }
            if !self.source_font_coverage_requests.insert((
                family.to_owned(),
                style.is_visually_bold(),
                style.italic,
            )) {
                if !style.italic {
                    break;
                }
                continue;
            }
            let primary = family;
            let plan = super::plan_svg_font_file_lookup(
                primary,
                &self.font_paths,
                style.is_visually_bold(),
            );
            let italic = style
                .italic
                .then(|| {
                    super::system_fonts::italic_family_file(
                        &self.font_paths,
                        &[primary.to_owned()],
                        style.is_visually_bold(),
                        &std::collections::HashSet::new(),
                    )
                })
                .flatten();
            if let Some(file) = italic.or_else(|| super::find_font_file(&plan)) {
                if file.metadata().is_ok_and(|m| m.len() <= 32 * 1024 * 1024) {
                    if let Ok(bytes) = std::fs::read(file) {
                        let _ = self.source_font_coverage.register(primary, &bytes, 0);
                    }
                }
            }
            if !style.italic
                || self
                    .source_font_coverage
                    .text_faces
                    .select(
                        &crate::renderer::render_font_family_chain_for_weight(
                            &style.font_family,
                            style.is_visually_bold(),
                        ),
                        text,
                        style.is_visually_bold(),
                        true,
                    )
                    .is_some()
            {
                break;
            }
        }
        if self.font_embed_mode != super::FontEmbedMode::None {
            for (_, cluster) in crate::renderer::layout::split_into_clusters(text) {
                let glyph = display(&self.source_font_coverage, &cluster, style);
                if style.italic {
                    if let Some(face) = self.source_font_coverage.text_faces.select(
                        &crate::renderer::render_font_family_chain_for_weight(
                            &style.font_family,
                            style.is_visually_bold(),
                        ),
                        &glyph,
                        style.is_visually_bold(),
                        true,
                    ) {
                        self.font_codepoints
                            .entry(face.family.clone())
                            .or_default()
                            .extend(glyph.chars());
                        if face.slanted {
                            self.font_italic_styles
                                .insert((face.family, style.is_visually_bold()));
                        }
                    }
                }
                if let Some(ch) = self
                    .source_font_coverage
                    .glyph(&style.font_family, &cluster)
                {
                    for family in style.font_family.split(',').map(str::trim) {
                        self.font_codepoints
                            .entry(family.to_owned())
                            .or_default()
                            .insert(ch);
                    }
                }
            }
        }
    }
}

pub(super) fn display<'a>(
    coverage: &crate::renderer::pua_glyph::FontCoverage,
    text: &'a str,
    style: &TextStyle,
) -> Cow<'a, str> {
    coverage
        .glyph(&style.font_family, text)
        .map_or(Cow::Borrowed(text), |ch| Cow::Owned(ch.to_string()))
}
