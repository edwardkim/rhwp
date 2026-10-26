//! 실제 Native paint의 face 선택을 공통 텍스트 효과에 공급한다.

use super::super::font_lookup::{font_style_for_text, typeface_candidates_for_families};
use super::SkiaLayerRenderer;
use crate::{document_core::DocumentCore, renderer::TextStyle, HwpError};

impl SkiaLayerRenderer {
    pub fn prepare_text_shadow_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        let requests: Vec<crate::paint::metafile_fonts::FontRequest> =
            serde_json::from_str(&core.get_text_shadow_font_requests_native()?)
                .map_err(|e| HwpError::RenderError(e.to_string()))?;
        let mut changed = false;
        for request in requests {
            for family in &request.families {
                let names = crate::renderer::font_environment::program_names(family);
                let style = font_style_for_text(&TextStyle {
                    font_family: family.clone(),
                    bold: request.weight >= 600,
                    italic: request.italic,
                    ..Default::default()
                });
                let Some(selected) = typeface_candidates_for_families(
                    &self.font_mgr,
                    &self.system_families,
                    &self.custom_typefaces,
                    &self.bundled_typefaces,
                    &names,
                    &names,
                    style,
                )
                .into_iter()
                .find(|candidate| candidate.source != "legacy") else {
                    continue;
                };
                let Some((data, index)) = selected.typeface.to_existing_font_data() else {
                    continue;
                };
                // 이미 로드한 프로그램을 메모리로 공유한다. font file 복사는 하지 않는다.
                changed |= core.register_metafile_font_native(family, data.as_bytes(), index)?;
            }
        }
        Ok(changed)
    }
}
