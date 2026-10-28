//! 원 PUA를 실제 표시할 글꼴 공급과 세션 메트릭의 소유 경계.

use crate::{error::HwpError, renderer::kerning::ExactFontSlot};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

use super::DocumentCore;

#[derive(Debug, Clone, Serialize)]
pub struct NativePuaFontRequest {
    pub family: String,
    pub paint_families: Vec<String>,
    pub bold: bool,
    pub italic: bool,
    /// 리터럴 공백의 프로그램은 PUA 기호 fallback과 별도로 선택한다.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub literal_figure_space: bool,
}

impl DocumentCore {
    /// 동기 공급자는 요청마다 준비한 한 프로그램만 빌려 주고, 배치는 한 번 재구성한다.
    pub fn prepare_native_pua_fonts_with(
        &mut self,
        mut provider: impl FnMut(&NativePuaFontRequest) -> Option<(Vec<u8>, u32)>,
    ) -> Result<bool, HwpError> {
        let mut fonts = crate::renderer::native_pua_fonts::NativePuaFonts::default();
        for request in self.native_pua_font_requests() {
            if let Some((bytes, index)) = provider(&request) {
                if request.literal_figure_space {
                    fonts.register_literal_space(
                        &request.family,
                        request.bold,
                        request.italic,
                        &bytes,
                        index,
                    )
                } else {
                    fonts.register(&request.family, request.bold, request.italic, &bytes, index)
                }
                .map_err(|message| HwpError::RenderError(message.into()))?;
            }
        }
        let changed = fonts != *self.native_pua_fonts;
        if changed {
            self.native_pua_fonts = std::sync::Arc::new(fonts);
            self.refresh_native_pua_font_layout();
        }
        Ok(changed)
    }
    /// 기존 bounded source traversal로 실제 PUA를 쓰는 슬롯만 조회한다.
    /// 뒤늦은 paint-time 검색이나 모든 글자 모양의 글꼴 선등록을 하지 않는다.
    pub fn native_pua_font_requests(&self) -> Vec<NativePuaFontRequest> {
        use super::queries::rendering::{
            collect_exact_font_slots_from_paragraphs_filtered, ExactFontSlotFilter,
        };
        let mut requests = BTreeMap::new();
        let mut work = 1_000_000;
        for (filter, literal_figure_space) in [
            (ExactFontSlotFilter::Pua, false),
            (ExactFontSlotFilter::LiteralFigureSpace, true),
        ] {
            let mut slots = HashSet::<ExactFontSlot>::new();
            for section in &self.document.sections {
                collect_exact_font_slots_from_paragraphs_filtered(
                    &section.paragraphs,
                    &mut slots,
                    &mut work,
                    0,
                    filter,
                );
                for master in &section.section_def.master_pages {
                    collect_exact_font_slots_from_paragraphs_filtered(
                        &master.paragraphs,
                        &mut slots,
                        &mut work,
                        0,
                        filter,
                    );
                }
            }
            for slot in slots {
                if self
                    .styles
                    .has_exact_font_source(slot.char_shape_id, slot.language_index)
                {
                    continue;
                }
                if let Some(style) = self.styles.char_styles.get(slot.char_shape_id as usize) {
                    let family = style.font_family_for_lang(slot.language_index).to_owned();
                    requests
                        .entry((
                            family.clone(),
                            style.bold,
                            style.italic,
                            literal_figure_space,
                        ))
                        .or_insert(NativePuaFontRequest {
                            paint_families: crate::renderer::render_font_family_names_for_weight(
                                &family, style.bold,
                            ),
                            family,
                            bold: style.bold,
                            italic: style.italic,
                            literal_figure_space,
                        });
                }
            }
        }
        requests.into_values().collect()
    }

    /// 공급자는 이 family/style을 paint에 준비한 실제 SFNT bytes를 전달한다.
    /// bytes는 읽기만 하며 저장·문서 직렬화·다른 family로의 복사를 하지 않는다.
    pub fn register_native_pua_font(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let mut fonts = (*self.native_pua_fonts).clone();
        let changed = fonts
            .register(family, bold, italic, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))?;
        let supported = fonts.get(family, bold, italic).is_some();
        if changed {
            self.native_pua_fonts = std::sync::Arc::new(fonts);
            self.refresh_native_pua_font_layout();
        }
        Ok(supported)
    }

    /// 실제 리터럴 공백에 선택한 SFNT만 별도 세션 메트릭으로 공급한다.
    pub fn register_literal_figure_space_font(
        &mut self,
        family: &str,
        bold: bool,
        italic: bool,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let mut fonts = (*self.native_pua_fonts).clone();
        let changed = fonts
            .register_literal_space(family, bold, italic, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))?;
        let supported = fonts.literal_space_em(family, bold, italic).is_some();
        if changed {
            self.native_pua_fonts = std::sync::Arc::new(fonts);
            self.refresh_native_pua_font_layout();
        }
        Ok(supported)
    }

    /// 호스트 글꼴 세대가 바뀌면 원 PUA 결정과 파생 배치를 함께 폐기한다.
    pub fn clear_native_pua_fonts(&mut self) {
        self.native_pua_fonts = Default::default();
        self.refresh_native_pua_font_layout();
    }

    /// 글꼴 공급은 원본/명시 exact registry를 다시 읽거나 지우는 문서 교체가 아니다.
    fn refresh_native_pua_font_layout(&mut self) {
        self.rebuild_resolved_styles();
        self.refresh_exact_font_measurement_contexts();
        self.recompose_all_with_horizontal_shaping();
        self.mark_all_sections_dirty();
        self.measured_tables.clear();
        self.measured_sections.clear();
        self.dirty_paragraphs.clear();
        self.para_column_map.clear();
        self.invalidate_page_tree_cache();
        self.overflow_links_cache.borrow_mut().clear();
        self.paginate_if_needed();
    }
}
