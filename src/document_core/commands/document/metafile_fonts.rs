//! WMF 이미지 내부 글꼴의 실제 공급과 paint 세대를 문서 측정 계약에서 분리한다.

use crate::{document_core::DocumentCore, error::HwpError};

impl DocumentCore {
    /// 공통 텍스트 효과의 실제 SFNT 공급이다. 기존 API 이름은 호환성을 위해 유지한다.
    /// 연속 그림자와 실제 탭의 점 리더를 포함하며 원본·조판 폭은 바꾸지 않는다.
    pub fn get_text_shadow_font_requests_native(&self) -> Result<String, HwpError> {
        if !self.hangul2024_compat {
            return Ok("[]".into());
        }
        use crate::document_core::queries::rendering::{
            collect_exact_font_slots_from_paragraphs_selected, ExactFontSlotSelection,
        };
        let mut tab_slots = std::collections::HashSet::new();
        let mut work = 1_000_000;
        for section in &self.document.sections {
            collect_exact_font_slots_from_paragraphs_selected(
                &section.paragraphs,
                &mut tab_slots,
                &mut work,
                0,
                ExactFontSlotSelection::Tab(&self.document.doc_info),
            );
            for master in &section.section_def.master_pages {
                collect_exact_font_slots_from_paragraphs_selected(
                    &master.paragraphs,
                    &mut tab_slots,
                    &mut work,
                    0,
                    ExactFontSlotSelection::Tab(&self.document.doc_info),
                );
            }
        }
        let mut seen: std::collections::HashMap<_, usize> = std::collections::HashMap::new();
        let mut requests: Vec<crate::paint::metafile_fonts::FontRequest> = Vec::new();
        for (index, resolved) in self.styles.char_styles.iter().enumerate() {
            let has_tab = tab_slots
                .iter()
                .any(|slot| slot.char_shape_id == index as u32);
            if resolved.shadow_type != 2 && !has_tab {
                continue;
            }
            for language in 0..7 {
                let tab = tab_slots.contains(&crate::renderer::kerning::ExactFontSlot::new(
                    index as u32,
                    language,
                ));
                if resolved.shadow_type != 2 && !tab {
                    continue;
                }
                let style = crate::renderer::TextStyle {
                    font_family: resolved.font_family_for_lang(language).to_owned(),
                    bold: resolved.bold,
                    italic: resolved.italic,
                    ..Default::default()
                };
                let families = crate::renderer::internal_font_family_members(&style.font_family)
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let weight = match style.css_font_weight() {
                    Some("bold") => 700,
                    Some(value) => value.parse().unwrap_or(400),
                    None => 400,
                };
                if families.is_empty() {
                    continue;
                }
                let key = (families.clone(), weight, style.italic);
                if let Some(&index) = seen.get(&key) {
                    // 먼저 수집한 점 요청 때문에 나중의 전체 그림자 공급을 줄이지 않는다.
                    if resolved.shadow_type == 2 {
                        requests[index].text.clear();
                    }
                } else {
                    seen.insert(key, requests.len());
                    requests.push(crate::paint::metafile_fonts::FontRequest {
                        families,
                        weight,
                        italic: style.italic,
                        text: if tab && resolved.shadow_type != 2 {
                            "·".into()
                        } else {
                            String::new()
                        },
                    });
                }
            }
        }
        serde_json::to_string(&requests).map_err(|e| HwpError::RenderError(e.to_string()))
    }

    /// 호스트가 선택한 실제 SFNT/TTC face를 메타파일 paint에 공급한다.
    pub fn register_metafile_font_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let changed = self
            .metafile_fonts
            .register(family, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))?;
        if changed {
            // 조판·원본·페이지네이션은 그대로이며 해당 글꼴을 소비한 paint만 다시 만든다.
            self.page_layer_tree_cache.borrow_mut().clear();
            self.layer_tree_json_cache.borrow_mut().clear();
        }
        Ok(changed)
    }

    /// 호스트 글꼴 세대가 교체되면 이전 문서의 paint font와 surface도 보존하지 않는다.
    pub fn clear_metafile_fonts_native(&mut self) {
        self.metafile_fonts = Default::default();
        self.page_layer_tree_cache.borrow_mut().clear();
        self.layer_tree_json_cache.borrow_mut().clear();
    }

    /// Canvas font 요청으로는 관측할 수 없는 SVG 이미지 내부의 family·문자 요구다.
    /// 조판 전에 원본 BinData를 읽으며, unsupported WMF를 성공한 변환으로 바꾸지 않는다.
    pub fn get_metafile_font_requests_native(&self) -> Result<String, HwpError> {
        if let Some(json) = self.metafile_fonts.cached_requests(self.bin_data_epoch) {
            return Ok(json);
        }
        let mut requests = Vec::new();
        {
            for item in &self.document.bin_data_content {
                let Some(data) = item
                    .data
                    .load_limited_shared(crate::model::bin_data::MAX_BIN_DATA_BYTES)
                else {
                    continue;
                };
                if crate::renderer::image_resolver::detect_image_mime_type(&data) != "image/x-wmf" {
                    continue;
                }
                let Some(svg) = crate::renderer::svg::convert_wmf_to_svg(&data) else {
                    continue;
                };
                if let Some(mut items) = crate::paint::metafile_fonts::requests(&svg) {
                    requests.append(&mut items);
                }
            }
        }
        let json =
            serde_json::to_string(&requests).map_err(|e| HwpError::RenderError(e.to_string()))?;
        self.metafile_fonts
            .cache_requests(self.bin_data_epoch, &json);
        Ok(json)
    }
}
