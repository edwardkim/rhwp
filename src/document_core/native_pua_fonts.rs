//! 원 PUA를 실제 표시할 글꼴 공급과 세션 메트릭의 소유 경계.

use crate::{error::HwpError, renderer::kerning::ExactFontSlot};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

use super::DocumentCore;

#[derive(Debug, Clone, Serialize)]
pub struct NativePuaFontRequest {
    pub family: String,
    pub bold: bool,
    pub italic: bool,
}

impl DocumentCore {
    /// 동기 공급자는 요청마다 준비한 한 프로그램만 빌려 주고, 배치는 한 번 재구성한다.
    pub fn prepare_native_pua_fonts_with(
        &mut self,
        mut provider: impl FnMut(&NativePuaFontRequest) -> Option<(Vec<u8>, u32)>,
    ) -> Result<bool, HwpError> {
        let mut fonts = (*self.native_pua_fonts).clone();
        let mut changed = false;
        for request in self.native_pua_font_requests() {
            if let Some((bytes, index)) = provider(&request) {
                changed |= fonts
                    .register(&request.family, request.bold, request.italic, &bytes, index)
                    .map_err(|message| HwpError::RenderError(message.into()))?;
            }
        }
        if changed {
            self.native_pua_fonts = std::sync::Arc::new(fonts);
            self.rebuild_derived_state();
        }
        Ok(changed)
    }
    /// 기존 bounded source traversal로 실제 PUA를 쓰는 슬롯만 조회한다.
    /// 뒤늦은 paint-time 검색이나 모든 글자 모양의 글꼴 선등록을 하지 않는다.
    pub fn native_pua_font_requests(&self) -> Vec<NativePuaFontRequest> {
        let mut slots = HashSet::<ExactFontSlot>::new();
        let mut work = 1_000_000;
        for section in &self.document.sections {
            super::queries::rendering::collect_exact_font_slots_from_paragraphs_filtered(
                &section.paragraphs,
                &mut slots,
                &mut work,
                0,
                true,
            );
            for master in &section.section_def.master_pages {
                super::queries::rendering::collect_exact_font_slots_from_paragraphs_filtered(
                    &master.paragraphs,
                    &mut slots,
                    &mut work,
                    0,
                    true,
                );
            }
        }
        let mut requests = BTreeMap::new();
        for slot in slots {
            if let Some(style) = self.styles.char_styles.get(slot.char_shape_id as usize) {
                let family = style.font_family_for_lang(slot.language_index).to_owned();
                requests
                    .entry((family.clone(), style.bold, style.italic))
                    .or_insert(NativePuaFontRequest {
                        family,
                        bold: style.bold,
                        italic: style.italic,
                    });
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
        if changed {
            self.native_pua_fonts = std::sync::Arc::new(fonts);
            self.rebuild_derived_state();
        }
        Ok(changed)
    }

    /// 호스트 글꼴 세대가 바뀌면 원 PUA 결정과 파생 배치를 함께 폐기한다.
    pub fn clear_native_pua_fonts(&mut self) {
        self.native_pua_fonts = Default::default();
        self.rebuild_derived_state();
    }
}
