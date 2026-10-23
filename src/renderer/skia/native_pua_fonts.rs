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
            let selected = crate::renderer::native_pua_fonts::SUPPORTED_SYMBOLS
                .iter()
                .find_map(|ch| select_typeface_for_character(&candidates, *ch))
                .filter(|candidate| candidate.source != "legacy")?;
            if !crate::renderer::native_pua_fonts::SUPPORTED_SYMBOLS
                .iter()
                .any(|ch| selected.typeface.unichar_to_glyph(*ch as i32) != 0)
            {
                return None;
            }
            let (data, index) = selected.typeface.to_existing_font_data()?;
            (data.size() <= 32 * 1024 * 1024).then(|| (data.as_bytes().to_vec(), index))
        })
    }
}
