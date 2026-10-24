//! 이미 적재한 실제 SFNT 공급에서 원본 face 부재와 대체 face를 확인한다.

use super::super::font_lookup::{font_style_for_text, typeface_candidates_for_families};
use super::SkiaLayerRenderer;
use crate::{document_core::DocumentCore, renderer::TextStyle, HwpError};
use sha2::{Digest, Sha256};

impl SkiaLayerRenderer {
    pub fn prepare_declared_font_fallbacks(
        &self,
        core: &mut DocumentCore,
    ) -> Result<bool, HwpError> {
        core.prepare_declared_font_fallbacks_with(|request| {
            let lookup = |family: &str| {
                let families = crate::renderer::font_environment::program_names(family);
                let style = font_style_for_text(&TextStyle {
                    font_family: family.to_owned(),
                    bold: request.bold,
                    italic: request.italic,
                    ..Default::default()
                });
                typeface_candidates_for_families(
                    &self.font_mgr,
                    &self.system_families,
                    &self.custom_typefaces,
                    &self.bundled_typefaces,
                    &families,
                    &families,
                    style,
                )
                .into_iter()
                .find(|candidate| candidate.source != "legacy")
            };
            // 일반 fallback이 있다는 사실로 원 face가 설치됐다고 판정하지 않는다.
            // 반대로 원 face가 있으면 문자별 glyph fallback을 전체 face 치환으로 바꾸지 않는다.
            if lookup(&request.family).is_some() {
                return None;
            }
            let selected = lookup(&request.fallback)?;
            let (bytes, index) = selected.typeface.to_existing_font_data()?;
            ttf_parser::Face::parse(bytes.as_bytes(), index).ok()?;
            Some(crate::document_core::ObservedFontSource {
                family: selected.typeface.family_name(),
                sha256: Sha256::digest(bytes.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
                face_index: index,
            })
        })
    }
}
