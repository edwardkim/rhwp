//! 이미 적재한 실제 문자 fallback을 원 PUA의 조판/paint 공급자로 준비한다.

use super::super::font_lookup::{
    font_style_for_text, select_typeface_for_character, text_typeface_candidates_for_source,
};
use super::SkiaLayerRenderer;
use crate::{document_core::DocumentCore, renderer::TextStyle, HwpError};

impl SkiaLayerRenderer {
    pub fn prepare_native_otf_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        core.prepare_native_otf_fonts_multiple_with(|request| {
            let style = font_style_for_text(&TextStyle {
                font_family: request.family.clone(),
                // 명시적 TrueType의 굵은 글자 전진은 같은 family의 보통 면을 쓴다.
                bold: request.bold && !request.source_ttf,
                italic: request.italic,
                ..Default::default()
            });
            let (_, candidates) = text_typeface_candidates_for_source(
                &self.font_mgr,
                &self.system_families,
                &self.custom_typefaces,
                &self.bundled_typefaces,
                &request.family,
                style,
                request.source_ttf,
            );
            candidates
                .into_iter()
                .filter_map(|selected| {
                    if selected.source == "legacy" {
                        return None;
                    }
                    let (bytes, index) =
                        if let Some((data, index)) = selected.typeface.to_existing_font_data() {
                            (data.as_bytes().to_vec(), index)
                        } else {
                            selected.typeface.to_font_bytes()?
                        };
                    if !matches!(
                        bytes.get(..4),
                        Some(b"OTTO" | b"\0\x01\0\0" | b"true" | b"ttcf")
                    ) || bytes.len() > 32 * 1024 * 1024
                        || (request.source_ttf
                            && u32::try_from(index).ok().is_none_or(|face| {
                                crate::renderer::native_pua_fonts::is_cff_program(&bytes, face)
                            }))
                    {
                        return None;
                    }
                    let names = super::super::font_lookup::supplied_family_names(
                        &bytes,
                        index,
                        &selected.typeface.family_name(),
                    );
                    // 원본 TTF의 전진을 구할 때 표시용 대체 face를 원본으로
                    // 취급하면, 원본에 없는 글리프의 조판 폭까지 바뀐다.
                    // HFT 표시 프로그램은 기존처럼 전체 후보를 허용한다.
                    let postscript_name = selected.typeface.post_script_name();
                    let family_matches = |family: &str| {
                        let family = family.trim();
                        names.iter().any(|name| family == name)
                            || postscript_name.as_deref() == Some(family)
                    };
                    let matches = if request.source_ttf {
                        request.family.split(',').next().is_some_and(family_matches)
                    } else {
                        request.family.split(',').any(family_matches)
                    };
                    matches.then_some((bytes, index))
                })
                .collect()
        })
    }

    pub fn prepare_native_pua_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        core.prepare_native_pua_fonts_with(|request| {
            let style = font_style_for_text(&TextStyle {
                font_family: request.family.clone(),
                bold: request.bold,
                italic: request.italic,
                ..Default::default()
            });
            let (_, candidates) = text_typeface_candidates_for_source(
                &self.font_mgr,
                &self.system_families,
                &self.custom_typefaces,
                &self.bundled_typefaces,
                &request.family,
                style,
                request.source_ttf,
            );
            let symbols: &[char] = if request.literal_figure_space {
                &['\u{2007}', '0']
            } else {
                &crate::renderer::native_pua_fonts::SUPPORTED_SYMBOLS
            };
            let selected = if request.literal_figure_space {
                // 글리프 fallback보다 앞선 실제 face의 숫자 폭을 먼저 사용한다.
                // 브라우저 공급자도 family별 프로그램 순서로 같은 계약을 적용한다.
                candidates.iter().find(|candidate| {
                    symbols
                        .iter()
                        .any(|ch| candidate.typeface.unichar_to_glyph(*ch as i32) != 0)
                })
            } else {
                symbols
                    .iter()
                    .find_map(|ch| select_typeface_for_character(&candidates, *ch))
            }
            .filter(|candidate| candidate.source != "legacy")?;
            if !symbols
                .iter()
                .any(|ch| selected.typeface.unichar_to_glyph(*ch as i32) != 0)
            {
                return None;
            }
            if let Some((data, index)) = selected.typeface.to_existing_font_data() {
                return (data.size() <= 32 * 1024 * 1024)
                    .then(|| (data.as_bytes().to_vec(), index));
            }
            // CoreText의 실제 system face는 재사용할 Data가 없을 수 있다.
            // 다른 face를 추정하지 않고 동일 Typeface의 SFNT stream을 읽는다.
            selected
                .typeface
                .to_font_bytes()
                .filter(|(bytes, _)| bytes.len() <= 32 * 1024 * 1024)
        })
    }
}
