//! 실제 Native paint의 face 선택을 공통 텍스트 효과에 공급한다.

use super::super::font_lookup::{font_style_for_text, typeface_candidates_for_families};
use super::SkiaLayerRenderer;
use crate::{
    document_core::DocumentCore,
    paint::metafile_fonts::{FontRequest, MetafileFonts},
    renderer::TextStyle,
    HwpError,
};

impl SkiaLayerRenderer {
    pub fn prepare_text_shadow_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        let requests: Vec<crate::paint::metafile_fonts::FontRequest> =
            serde_json::from_str(&core.get_text_shadow_font_requests_native()?)
                .map_err(|e| HwpError::RenderError(e.to_string()))?;
        self.prepare_requested_paint_fonts(requests, |family, bytes, index| {
            core.register_metafile_font_native(family, bytes, index)
        })
    }

    /// 본문에서 사용하지 않은 WMF 내부 가족도 같은 renderer snapshot에서 공급한다.
    /// 요청·등록은 기존 공통 메타파일 자원 한도와 paint cache 무효화를 사용한다.
    pub fn prepare_metafile_fonts(&self, core: &mut DocumentCore) -> Result<bool, HwpError> {
        let requests: Vec<crate::paint::metafile_fonts::FontRequest> =
            serde_json::from_str(&core.get_metafile_font_requests_native()?)
                .map_err(|e| HwpError::RenderError(e.to_string()))?;
        self.prepare_requested_paint_fonts(requests, |family, bytes, index| {
            core.register_metafile_font_native(family, bytes, index)
        })
    }

    /// 한 쪽의 실제 글꼴만 추가한 독립 Print snapshot을 만든다.
    pub(crate) fn paint_fonts_for_page(
        &self,
        core: &DocumentCore,
        page: u32,
    ) -> Result<Option<MetafileFonts>, HwpError> {
        let requests = core.page_paint_font_requests(page)?;
        if requests.metafile.is_empty() && requests.text_effects.is_empty() {
            return Ok(None);
        }
        let mut fonts = core.metafile_fonts.clone();
        let changed = self.prepare_requested_paint_fonts(
            requests.text_effects.into_iter().chain(requests.metafile),
            |family, bytes, index| {
                fonts
                    .register(family, bytes, index)
                    .map_err(|message| HwpError::RenderError(message.into()))
            },
        )?;
        Ok(changed.then_some(fonts))
    }

    fn prepare_requested_paint_fonts(
        &self,
        requests: impl IntoIterator<Item = FontRequest>,
        mut register: impl FnMut(&str, &[u8], u32) -> Result<bool, HwpError>,
    ) -> Result<bool, HwpError> {
        let mut changed = false;
        for request in requests {
            let mut remaining = request.text.clone();
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
                let existing = selected.typeface.to_existing_font_data();
                let program = existing
                    .as_ref()
                    .map(|(data, index)| (data.as_bytes(), *index));
                // DirectWrite는 기존 스트림을 제공하지 않아도 원본 파일과 face index를 열 수 있다.
                #[cfg(target_os = "windows")]
                let reopened = if program.is_none() {
                    selected.typeface.to_font_bytes()
                } else {
                    None
                };
                #[cfg(target_os = "windows")]
                let program = program.or_else(|| {
                    reopened
                        .as_ref()
                        .map(|(bytes, index)| (bytes.as_slice(), *index))
                });
                let Some((bytes, index)) = program else {
                    continue;
                };
                if !request.text.is_empty() {
                    let Some(missing) = crate::renderer::font_symbol_coverage::font_missing_text(
                        bytes, index, &remaining,
                    ) else {
                        continue;
                    };
                    if missing == remaining {
                        continue;
                    }
                    remaining = missing;
                }
                changed |= register(family, bytes, index)?;
                if !request.text.is_empty() && remaining.is_empty() {
                    break;
                }
            }
        }
        Ok(changed)
    }
}
