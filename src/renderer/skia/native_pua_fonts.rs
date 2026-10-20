//! 이미 적재한 주 face만 원 PUA의 조판/paint 공급자로 준비한다.

use super::super::font_lookup::{font_style_for_text, typeface_candidates_for_families};
use super::SkiaLayerRenderer;
use crate::{document_core::DocumentCore, renderer::TextStyle, HwpError};

impl SkiaLayerRenderer {
    pub fn prepare_native_pua_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        core.prepare_native_pua_fonts_with(|request| {
            let primary = crate::renderer::internal_font_family_members(&request.family)
                .into_iter()
                .next()?
                .to_owned();
            let mut families = vec![primary.clone()];
            families.extend(
                crate::renderer::installed_render_font_aliases(&primary)
                    .iter()
                    .map(|alias| (*alias).to_owned()),
            );
            let style = font_style_for_text(&TextStyle {
                font_family: request.family.clone(),
                bold: request.bold,
                italic: request.italic,
                ..Default::default()
            });
            let candidates = typeface_candidates_for_families(
                &self.font_mgr,
                &self.system_families,
                &self.custom_typefaces,
                &self.bundled_typefaces,
                &families,
                &families,
                style,
            );
            let selected = candidates
                .first()
                .filter(|candidate| candidate.source != "legacy")?;
            if !crate::renderer::native_pua_fonts::BOOK_BRACKETS
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
