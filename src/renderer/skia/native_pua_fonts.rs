//! 이미 적재한 실제 문자 fallback을 원 PUA의 조판/paint 공급자로 준비한다.

use super::super::font_lookup::{
    font_style_for_text, select_typeface_for_character, text_typeface_candidates,
};
use super::SkiaLayerRenderer;
use crate::{document_core::DocumentCore, renderer::TextStyle, HwpError};

impl SkiaLayerRenderer {
    pub fn prepare_native_pua_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        core.prepare_native_pua_fonts_with(|request| {
            let style = font_style_for_text(&TextStyle {
                font_family: request.family.clone(),
                bold: request.bold,
                italic: request.italic,
                ..Default::default()
            });
            let (_, candidates) = text_typeface_candidates(
                &self.font_mgr,
                &self.system_families,
                &self.custom_typefaces,
                &self.bundled_typefaces,
                &request.family,
                style,
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
