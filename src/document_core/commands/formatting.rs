//! 글자모양/문단모양 조회·적용 관련 native 메서드

use super::super::helpers::{
    border_line_type_to_u8_val, build_tab_def_from_json, color_ref_to_css, json_has_border_keys,
    json_has_tab_keys, parse_char_shape_mods, parse_json_i16_array, parse_para_shape_mods,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::reflow_line_segs;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::{resolve_styles, ResolvedStyleSet};

fn char_shape_mods_affect_text_flow(mods: &crate::model::style::CharShapeMods) -> bool {
    mods.base_size.is_some()
        || mods.font_ids.is_some()
        || mods.ratios.is_some()
        || mods.spacings.is_some()
        || mods.relative_sizes.is_some()
        || mods.char_offsets.is_some()
}

fn body_available_width_for_para_shape(
    core: &DocumentCore,
    sec_idx: usize,
    para_shape_id: u16,
    styles: &ResolvedStyleSet,
) -> f64 {
    let Some(section) = core.document.sections.get(sec_idx) else {
        return 1.0;
    };
    let page_def = &section.section_def.page_def;
    let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
    let layout = PageLayoutInfo::from_page_def(page_def, &column_def, core.dpi);
    let col_width = layout
        .column_areas
        .first()
        .map(|a| a.width)
        .unwrap_or(layout.body_area.width);
    let para_style = styles.para_styles.get(para_shape_id as usize);
    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
    (col_width - margin_left - margin_right).max(1.0)
}

impl DocumentCore {
    pub fn get_char_properties_at_native(
        &self,
        sec_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(sec_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)))?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)))?;
        Ok(self.build_char_properties_json(para, char_offset))
    }

    /// 셀 내부 문단의 글자 속성 조회 (네이티브)
    pub fn get_cell_char_properties_at_native(
        &self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .get_cell_paragraph_ref(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
        Ok(self.build_char_properties_json(para, char_offset))
    }

    /// 캐럿 위치의 문단 속성 조회 (네이티브)
    pub fn get_para_properties_at_native(
        &self,
        sec_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::model::control::Control;
        use crate::model::style::HeadType;
        let section = self
            .document
            .sections
            .get(sec_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)))?;
        let Some(para) = section.paragraphs.get(para_idx) else {
            if let Some(src) = self.virtual_endnote_para_source(sec_idx, para_idx) {
                return self.get_para_properties_in_footnote_native(
                    src.section_index,
                    src.para_index,
                    src.control_index,
                    src.note_para_index,
                );
            }
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        };
        let mut json = self.build_para_properties_json(para.para_shape_id, sec_idx);

        // 번호 시작 방식 판별: numbering_id 패턴 기반
        let ps = self.styles.para_styles.get(para.para_shape_id as usize);
        let head_type = ps.map(|s| s.head_type).unwrap_or(HeadType::None);
        if head_type != HeadType::None {
            let cur_nid = ps.map(|s| s.numbering_id).unwrap_or(0);
            // NewNumber 컨트롤 체크
            let new_number = para.controls.iter().find_map(|c| {
                if let Control::NewNumber(nn) = c {
                    Some(nn.number)
                } else {
                    None
                }
            });
            let (mode, start_num) = if let Some(num) = new_number {
                (2, num as u32) // 새 번호 목록 시작 (NewNumber 컨트롤)
            } else {
                // 이전 번호 문단의 numbering_id를 역순 스캔
                let mut prev_nid: Option<u16> = None;
                let mut seen_before = false;
                for pi in (0..para_idx).rev() {
                    let pp = &section.paragraphs[pi];
                    let pps = self.styles.para_styles.get(pp.para_shape_id as usize);
                    let pht = pps.map(|s| s.head_type).unwrap_or(HeadType::None);
                    if pht == HeadType::None {
                        continue;
                    }
                    let pnid = pps.map(|s| s.numbering_id).unwrap_or(0);
                    if prev_nid.is_none() {
                        prev_nid = Some(pnid);
                    }
                    if pnid == cur_nid {
                        seen_before = true;
                        break;
                    }
                }
                match (prev_nid, seen_before) {
                    (Some(pid), _) if pid == cur_nid => (0, 1), // 앞 번호 이어
                    (_, true) => (1, 1),                        // 이전 번호 이어
                    _ => (2, 1),                                // 새 번호 시작
                }
            };
            json.pop(); // 마지막 '}' 제거
            json.push_str(&format!(
                ",\"numberingRestartMode\":{},\"numberingStartNum\":{}}}",
                mode, start_num
            ));
        }

        Ok(json)
    }

    fn virtual_endnote_para_source(
        &self,
        sec_idx: usize,
        para_idx: usize,
    ) -> Option<crate::renderer::pagination::EndnoteParaSource> {
        let body_len = self.document.sections.get(sec_idx)?.paragraphs.len();
        let local_idx = para_idx.checked_sub(body_len)?;
        self.pagination
            .get(sec_idx)?
            .endnote_para_sources
            .get(local_idx)
            .cloned()
    }

    /// 셀 내부 문단의 문단 속성 조회 (네이티브)
    pub fn get_cell_para_properties_at_native(
        &self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .get_cell_paragraph_ref(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
        Ok(self.build_para_properties_json(para.para_shape_id, sec_idx))
    }

    /// 글자 속성 JSON 생성 헬퍼
    pub(crate) fn build_char_properties_json(
        &self,
        para: &crate::model::paragraph::Paragraph,
        char_offset: usize,
    ) -> String {
        let char_shape_id = para.char_shape_id_at(char_offset).unwrap_or(0);
        let style = self.styles.char_styles.get(char_shape_id as usize);

        match style {
            Some(cs) => {
                use crate::model::style::UnderlineType;
                use crate::renderer::style_resolver::detect_lang_category;

                // 캐럿 위치 문자의 언어 카테고리를 판별하여 해당 폰트 반환
                let lang_index = para
                    .text
                    .chars()
                    .nth(char_offset)
                    .map(|ch| detect_lang_category(ch))
                    .unwrap_or(0);
                let font_family_raw = cs.font_family_for_lang(lang_index);
                let font_family =
                    crate::renderer::style_resolver::primary_font_name(&font_family_raw);

                let escaped_font = super::super::helpers::json_escape(font_family);
                let underline = !matches!(cs.underline, UnderlineType::None);
                let underline_type_str = match cs.underline {
                    UnderlineType::None => "None",
                    UnderlineType::Bottom => "Bottom",
                    UnderlineType::Top => "Top",
                };

                // raw CharShape에서 추가 속성 읽기
                let raw_cs = self
                    .document
                    .doc_info
                    .char_shapes
                    .get(char_shape_id as usize);
                let base_size = raw_cs.map(|s| s.base_size).unwrap_or(1000);

                // 언어별 글꼴 이름 배열 (원본 폰트명만, 폴백 제외)
                let font_families: Vec<String> = (0..7usize)
                    .map(|i| {
                        let name = cs.font_family_for_lang(i);
                        let primary = crate::renderer::style_resolver::primary_font_name(&name);
                        super::super::helpers::json_escape(primary)
                    })
                    .collect();
                let font_families_json = format!(
                    "[{}]",
                    font_families
                        .iter()
                        .map(|f| format!("\"{}\"", f))
                        .collect::<Vec<_>>()
                        .join(",")
                );

                // 언어별 수치 배열
                let (ratios, spacings, relative_sizes, char_offsets) = match raw_cs {
                    Some(s) => (s.ratios, s.spacings, s.relative_sizes, s.char_offsets),
                    None => ([100u8; 7], [0i8; 7], [100u8; 7], [0i8; 7]),
                };
                let ratios_json = format!(
                    "[{}]",
                    ratios
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let spacings_json = format!(
                    "[{}]",
                    spacings
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let relative_sizes_json = format!(
                    "[{}]",
                    relative_sizes
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let char_offsets_json = format!(
                    "[{}]",
                    char_offsets
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );

                let (
                    shadow_type,
                    shadow_color,
                    shadow_offset_x,
                    shadow_offset_y,
                    outline_type,
                    subscript,
                    superscript,
                    shade_color,
                    emboss,
                    engrave,
                    emphasis_dot,
                    underline_shape,
                    strike_shape,
                    kerning,
                ) = match raw_cs {
                    Some(s) => (
                        s.shadow_type,
                        s.shadow_color,
                        s.shadow_offset_x,
                        s.shadow_offset_y,
                        s.outline_type,
                        s.subscript,
                        s.superscript,
                        s.shade_color,
                        s.emboss,
                        s.engrave,
                        s.emphasis_dot,
                        s.underline_shape,
                        s.strike_shape,
                        s.kerning,
                    ),
                    None => (
                        0, 0xB2B2B2, 0i8, 0i8, 0, false, false, 0xFFFFFF, false, false, 0, 0, 0,
                        false,
                    ),
                };

                // 글자 테두리/배경 정보
                let border_fill_json = self.build_char_border_fill_json(raw_cs);

                format!(
                    concat!(
                        "{{\"fontFamily\":\"{}\",\"fontSize\":{},\"bold\":{},\"italic\":{},",
                        "\"underline\":{},\"underlineType\":\"{}\",\"underlineColor\":\"{}\",",
                        "\"strikethrough\":{},\"strikeColor\":\"{}\",",
                        "\"textColor\":\"{}\",\"shadeColor\":\"{}\",",
                        "\"shadowType\":{},\"shadowColor\":\"{}\",\"shadowOffsetX\":{},\"shadowOffsetY\":{},",
                        "\"outlineType\":{},",
                        "\"subscript\":{},\"superscript\":{},",
                        "\"emboss\":{},\"engrave\":{},",
                        "\"emphasisDot\":{},\"underlineShape\":{},\"strikeShape\":{},\"kerning\":{},",
                        "\"charShapeId\":{},",
                        "\"fontFamilies\":{},",
                        "\"ratios\":{},\"spacings\":{},\"relativeSizes\":{},\"charOffsets\":{},",
                        "{}",
                        "}}"
                    ),
                    escaped_font, base_size, cs.bold, cs.italic,
                    underline, underline_type_str, color_ref_to_css(cs.underline_color),
                    cs.strikethrough, color_ref_to_css(raw_cs.map(|s| s.strike_color).unwrap_or(0)),
                    color_ref_to_css(cs.text_color), color_ref_to_css(shade_color),
                    shadow_type, color_ref_to_css(shadow_color), shadow_offset_x, shadow_offset_y,
                    outline_type,
                    subscript, superscript,
                    emboss, engrave,
                    emphasis_dot, underline_shape, strike_shape, kerning,
                    char_shape_id,
                    font_families_json,
                    ratios_json, spacings_json, relative_sizes_json, char_offsets_json,
                    border_fill_json,
                )
            }
            None => {
                format!(
                    concat!(
                        "{{\"fontFamily\":\"sans-serif\",\"fontSize\":1000,\"bold\":false,\"italic\":false,",
                        "\"underline\":false,\"underlineType\":\"None\",\"underlineColor\":\"#000000\",",
                        "\"strikethrough\":false,\"strikeColor\":\"#000000\",",
                        "\"textColor\":\"#000000\",\"shadeColor\":\"#ffffff\",",
                        "\"shadowType\":0,\"shadowColor\":\"#b2b2b2\",\"shadowOffsetX\":0,\"shadowOffsetY\":0,",
                        "\"outlineType\":0,",
                        "\"subscript\":false,\"superscript\":false,",
                        "\"emboss\":false,\"engrave\":false,",
                        "\"emphasisDot\":0,\"underlineShape\":0,\"strikeShape\":0,\"kerning\":false,",
                        "\"charShapeId\":{},",
                        "\"fontFamilies\":[\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\"],",
                        "\"ratios\":[100,100,100,100,100,100,100],\"spacings\":[0,0,0,0,0,0,0],",
                        "\"relativeSizes\":[100,100,100,100,100,100,100],\"charOffsets\":[0,0,0,0,0,0,0],",
                        "\"borderFillId\":0,",
                        "\"borderLeft\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderRight\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderTop\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderBottom\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0",
                        "}}"
                    ),
                    char_shape_id
                )
            }
        }
    }

    /// charShapeId로 직접 글자 속성 JSON을 빌드 (스타일 상세 조회용)
    pub(crate) fn build_char_properties_json_by_id(&self, char_shape_id: u16) -> String {
        let style = self.styles.char_styles.get(char_shape_id as usize);
        match style {
            Some(cs) => {
                use crate::model::style::UnderlineType;
                // 한글(0) 언어를 기본으로 사용
                let font_family_raw = cs.font_family_for_lang(0);
                let font_family =
                    crate::renderer::style_resolver::primary_font_name(&font_family_raw);
                let escaped_font = super::super::helpers::json_escape(font_family);
                let underline = !matches!(cs.underline, UnderlineType::None);
                let underline_type_str = match cs.underline {
                    UnderlineType::None => "None",
                    UnderlineType::Bottom => "Bottom",
                    UnderlineType::Top => "Top",
                };
                let raw_cs = self
                    .document
                    .doc_info
                    .char_shapes
                    .get(char_shape_id as usize);
                let base_size = raw_cs.map(|s| s.base_size).unwrap_or(1000);
                let font_families: Vec<String> = (0..7usize)
                    .map(|i| {
                        let name = cs.font_family_for_lang(i);
                        let primary = crate::renderer::style_resolver::primary_font_name(&name);
                        super::super::helpers::json_escape(primary)
                    })
                    .collect();
                let font_families_json = format!(
                    "[{}]",
                    font_families
                        .iter()
                        .map(|f| format!("\"{}\"", f))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let (ratios, spacings, relative_sizes, char_offsets) = match raw_cs {
                    Some(s) => (s.ratios, s.spacings, s.relative_sizes, s.char_offsets),
                    None => ([100u8; 7], [0i8; 7], [100u8; 7], [0i8; 7]),
                };
                let ratios_json = format!(
                    "[{}]",
                    ratios
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let spacings_json = format!(
                    "[{}]",
                    spacings
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let relative_sizes_json = format!(
                    "[{}]",
                    relative_sizes
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let char_offsets_json = format!(
                    "[{}]",
                    char_offsets
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let (
                    shadow_type,
                    shadow_color,
                    shadow_offset_x,
                    shadow_offset_y,
                    outline_type,
                    subscript,
                    superscript,
                    shade_color,
                    emboss,
                    engrave,
                    emphasis_dot,
                    underline_shape,
                    strike_shape,
                    kerning,
                ) = match raw_cs {
                    Some(s) => (
                        s.shadow_type,
                        s.shadow_color,
                        s.shadow_offset_x,
                        s.shadow_offset_y,
                        s.outline_type,
                        s.subscript,
                        s.superscript,
                        s.shade_color,
                        s.emboss,
                        s.engrave,
                        s.emphasis_dot,
                        s.underline_shape,
                        s.strike_shape,
                        s.kerning,
                    ),
                    None => (
                        0, 0xB2B2B2, 0i8, 0i8, 0, false, false, 0xFFFFFF, false, false, 0, 0, 0,
                        false,
                    ),
                };
                let border_fill_json = self.build_char_border_fill_json(raw_cs);
                format!(
                    concat!(
                        "{{\"fontFamily\":\"{}\",\"fontSize\":{},\"bold\":{},\"italic\":{},",
                        "\"underline\":{},\"underlineType\":\"{}\",\"underlineColor\":\"{}\",",
                        "\"strikethrough\":{},\"strikeColor\":\"{}\",",
                        "\"textColor\":\"{}\",\"shadeColor\":\"{}\",",
                        "\"shadowType\":{},\"shadowColor\":\"{}\",\"shadowOffsetX\":{},\"shadowOffsetY\":{},",
                        "\"outlineType\":{},",
                        "\"subscript\":{},\"superscript\":{},",
                        "\"emboss\":{},\"engrave\":{},",
                        "\"emphasisDot\":{},\"underlineShape\":{},\"strikeShape\":{},\"kerning\":{},",
                        "\"charShapeId\":{},",
                        "\"fontFamilies\":{},",
                        "\"ratios\":{},\"spacings\":{},\"relativeSizes\":{},\"charOffsets\":{},",
                        "{}",
                        "}}"
                    ),
                    escaped_font, base_size, cs.bold, cs.italic,
                    underline, underline_type_str, color_ref_to_css(cs.underline_color),
                    cs.strikethrough, color_ref_to_css(raw_cs.map(|s| s.strike_color).unwrap_or(0)),
                    color_ref_to_css(cs.text_color), color_ref_to_css(shade_color),
                    shadow_type, color_ref_to_css(shadow_color), shadow_offset_x, shadow_offset_y,
                    outline_type,
                    subscript, superscript,
                    emboss, engrave,
                    emphasis_dot, underline_shape, strike_shape, kerning,
                    char_shape_id,
                    font_families_json,
                    ratios_json, spacings_json, relative_sizes_json, char_offsets_json,
                    border_fill_json,
                )
            }
            None => {
                format!(
                    concat!(
                        "{{\"fontFamily\":\"sans-serif\",\"fontSize\":1000,\"bold\":false,\"italic\":false,",
                        "\"underline\":false,\"underlineType\":\"None\",\"underlineColor\":\"#000000\",",
                        "\"strikethrough\":false,\"strikeColor\":\"#000000\",",
                        "\"textColor\":\"#000000\",\"shadeColor\":\"#ffffff\",",
                        "\"shadowType\":0,\"shadowColor\":\"#b2b2b2\",\"shadowOffsetX\":0,\"shadowOffsetY\":0,",
                        "\"outlineType\":0,",
                        "\"subscript\":false,\"superscript\":false,",
                        "\"emboss\":false,\"engrave\":false,",
                        "\"emphasisDot\":0,\"underlineShape\":0,\"strikeShape\":0,\"kerning\":false,",
                        "\"charShapeId\":{},",
                        "\"fontFamilies\":[\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\",\"sans-serif\"],",
                        "\"ratios\":[100,100,100,100,100,100,100],\"spacings\":[0,0,0,0,0,0,0],",
                        "\"relativeSizes\":[100,100,100,100,100,100,100],\"charOffsets\":[0,0,0,0,0,0,0],",
                        "\"borderFillId\":0,",
                        "\"borderLeft\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderRight\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderTop\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderBottom\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0",
                        "}}"
                    ),
                    char_shape_id
                )
            }
        }
    }

    /// 글자 테두리/배경 JSON 헬퍼 — CharShape의 border_fill_id를 참조하여 BorderFill 정보를 JSON 문자열로 반환
    pub(crate) fn build_char_border_fill_json(
        &self,
        raw_cs: Option<&crate::model::style::CharShape>,
    ) -> String {
        let bf_id = raw_cs.map(|s| s.border_fill_id).unwrap_or(0);
        if bf_id == 0 {
            return concat!(
                "\"borderFillId\":0,",
                "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
            ).to_string();
        }
        let bf = self
            .document
            .doc_info
            .border_fills
            .get((bf_id - 1) as usize);
        match bf {
            Some(bf) => {
                use crate::model::style::FillType;
                let dir_names = ["Left", "Right", "Top", "Bottom"];
                let borders_json: Vec<String> = bf.borders.iter().enumerate().map(|(i, b)| {
                    format!(
                        "\"border{}\":{{\"type\":{},\"width\":{},\"color\":\"{}\"}}",
                        dir_names[i],
                        border_line_type_to_u8_val(b.line_type),
                        b.width,
                        color_ref_to_css(b.color),
                    )
                }).collect();
                let (fill_type_str, fill_color, pat_color, pat_type) = match &bf.fill.solid {
                    Some(sf) if bf.fill.fill_type == FillType::Solid => {
                        ("solid", color_ref_to_css(sf.background_color),
                         color_ref_to_css(sf.pattern_color), sf.pattern_type)
                    }
                    _ => ("none", "#ffffff".to_string(), "#000000".to_string(), 0),
                };
                format!(
                    "\"borderFillId\":{},{},\"fillType\":\"{}\",\"fillColor\":\"{}\",\"patternColor\":\"{}\",\"patternType\":{}",
                    bf_id,
                    borders_json.join(","),
                    fill_type_str, fill_color, pat_color, pat_type,
                )
            }
            None => {
                concat!(
                    "\"borderFillId\":0,",
                    "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
                ).to_string()
            }
        }
    }

    /// 문단 속성 JSON 생성 헬퍼
    pub(crate) fn build_para_properties_json(&self, para_shape_id: u16, sec_idx: usize) -> String {
        use crate::model::style::{Alignment, FillType, HeadType};
        let ps = self.styles.para_styles.get(para_shape_id as usize);

        // 탭 정의 조회
        let raw_ps = self
            .document
            .doc_info
            .para_shapes
            .get(para_shape_id as usize);
        let tab_def_id = raw_ps.map(|p| p.tab_def_id).unwrap_or(0);
        let tab_def = self.document.doc_info.tab_defs.get(tab_def_id as usize);
        let tab_auto_left = tab_def.map(|td| td.auto_tab_left).unwrap_or(false);
        let tab_auto_right = tab_def.map(|td| td.auto_tab_right).unwrap_or(false);
        let tab_stops_json = tab_def
            .map(|td| {
                td.tabs
                    .iter()
                    .map(|t| {
                        format!(
                            "{{\"position\":{},\"type\":{},\"fill\":{}}}",
                            t.position, t.tab_type, t.fill_type
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();
        let default_tab_spacing = self
            .document
            .sections
            .get(sec_idx)
            .map(|s| s.section_def.default_tab_spacing)
            .unwrap_or(4000);

        // 테두리/배경 조회
        let bf_id = raw_ps.map(|p| p.border_fill_id).unwrap_or(0);
        let border_spacing = raw_ps.map(|p| p.border_spacing).unwrap_or([0; 4]);
        let border_fill_json = if bf_id > 0 {
            if let Some(bf) = self
                .document
                .doc_info
                .border_fills
                .get((bf_id - 1) as usize)
            {
                let dir_names = ["Left", "Right", "Top", "Bottom"];
                let borders: Vec<String> = bf
                    .borders
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        format!(
                            "\"border{}\":{{\"type\":{},\"width\":{},\"color\":\"{}\"}}",
                            dir_names[i],
                            border_line_type_to_u8_val(b.line_type),
                            b.width,
                            color_ref_to_css(b.color),
                        )
                    })
                    .collect();
                let (fill_type_str, fill_color, pat_color, pat_type) = match &bf.fill.solid {
                    Some(sf) if bf.fill.fill_type == FillType::Solid => (
                        "solid",
                        color_ref_to_css(sf.background_color),
                        color_ref_to_css(sf.pattern_color),
                        sf.pattern_type,
                    ),
                    _ => ("none", "#ffffff".to_string(), "#000000".to_string(), 0),
                };
                format!(
                    "\"borderFillId\":{},{},\"fillType\":\"{}\",\"fillColor\":\"{}\",\"patternColor\":\"{}\",\"patternType\":{}",
                    bf_id, borders.join(","), fill_type_str, fill_color, pat_color, pat_type,
                )
            } else {
                format!(
                    concat!(
                        "\"borderFillId\":0,",
                        "\"borderLeft\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderRight\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderTop\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderBottom\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
                    )
                )
            }
        } else {
            format!(
                concat!(
                    "\"borderFillId\":0,",
                    "\"borderLeft\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                    "\"borderRight\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                    "\"borderTop\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                    "\"borderBottom\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                    "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
                )
            )
        };

        // [Task #1037 + para-unit regression] dialog 표시 한컴 정합:
        // - margin/indent 는 raw_ps 직접 사용 (variant_div 미적용)
        // - HWP3 native 만 raw margin_left 가 continuation 라인 position 이므로
        //   한컴 dialog "왼쪽 여백" 을 effective first-line position 으로 보정한다.
        // - HWP5/HWPX 및 HWP3→HWP5 변환본은 raw margin_left 가 dialog 의 왼쪽 여백 의미다.
        let (raw_left_hu, raw_right_hu, raw_indent_hu) = raw_ps
            .map(|r| (r.margin_left, r.margin_right, r.indent))
            .unwrap_or((0, 0, 0));
        let is_hwp3_native =
            self.document.header.version.major == 3 && !self.document.is_hwp3_variant;
        let effective_left_hu = if is_hwp3_native {
            raw_left_hu + raw_indent_hu.min(0)
        } else {
            raw_left_hu
        };
        // [Issue #1172] ParaShape margin/indent 의 IR 값은 2× 스케일이다
        // (HWP5 바이너리 원본 스케일, HWPX 도 parser 의 val2x 로 통일 — header.rs).
        // 즉 1pt = 200 HWPUNIT. 한컴 편집기 정답: para-001 margin 2000 → 10.0pt.
        // dialog 표시(px→pt by frontend pxToPt)와 정합하려면 표준 hwpunit_to_px(7200/inch,
        // 1× 가정) 적용 전에 2× 스케일을 1× 로 환산(÷2)해야 한다. (종전: ÷2 누락 → 2배 표시)
        let dialog_margin_left_px = crate::renderer::hwpunit_to_px(effective_left_hu / 2, self.dpi);
        let dialog_margin_right_px = crate::renderer::hwpunit_to_px(raw_right_hu / 2, self.dpi);
        let dialog_indent_px = crate::renderer::hwpunit_to_px(raw_indent_hu / 2, self.dpi);

        match ps {
            Some(ps) => {
                let align_str = match ps.alignment {
                    Alignment::Justify => "justify",
                    Alignment::Left => "left",
                    Alignment::Right => "right",
                    Alignment::Center => "center",
                    Alignment::Distribute => "distribute",
                    Alignment::Split => "split",
                };
                let head_str = match ps.head_type {
                    HeadType::None => "None",
                    HeadType::Outline => "Outline",
                    HeadType::Number => "Number",
                    HeadType::Bullet => "Bullet",
                };
                // 원본 ParaShape에서 attr 비트 추출
                let (a1, a2) = raw_ps.map(|r| (r.attr1, r.attr2)).unwrap_or((0, 0));
                // 바이너리: attr1, HWPX: attr2 — OR 조합으로 양쪽 지원
                let widow_orphan = ((a1 >> 16) & 1 != 0) || ((a2 >> 5) & 1 != 0);
                let keep_with_next = ((a1 >> 17) & 1 != 0) || ((a2 >> 6) & 1 != 0);
                let keep_lines = ((a1 >> 18) & 1 != 0) || ((a2 >> 7) & 1 != 0);
                let page_break_before = ((a1 >> 19) & 1 != 0) || ((a2 >> 8) & 1 != 0);
                let font_line_height = (a1 >> 22) & 1 != 0;
                let single_line = (a2 & 0x03) != 0;
                let auto_space_kr_en = ((a2 >> 4) & 1 != 0) || ((a1 >> 20) & 1 != 0);
                let auto_space_kr_num = ((a2 >> 5) & 1 != 0) || ((a1 >> 21) & 1 != 0);
                // verticalAlign: attr1 bits 20-21 (autoSpacing과 충돌 시 0)
                let vertical_align = if !auto_space_kr_en && !auto_space_kr_num {
                    (a1 >> 20) & 0x03
                } else {
                    0
                };
                let english_break_unit = (a1 >> 5) & 0x03;
                let korean_break_unit = (a1 >> 7) & 0x01;
                let border_connect = (a1 >> 28) & 1 != 0;
                let border_ignore_margin = (a1 >> 29) & 1 != 0;
                format!(
                    concat!(
                        "{{\"alignment\":\"{}\",\"lineSpacing\":{:.1},\"lineSpacingType\":\"{:?}\",",
                        "\"marginLeft\":{:.1},\"marginRight\":{:.1},\"indent\":{:.1},",
                        "\"spacingBefore\":{:.1},\"spacingAfter\":{:.1},\"paraShapeId\":{},",
                        "\"headType\":\"{}\",\"paraLevel\":{},\"numberingId\":{},",
                        "\"widowOrphan\":{},\"keepWithNext\":{},\"keepLines\":{},\"pageBreakBefore\":{},",
                        "\"fontLineHeight\":{},\"singleLine\":{},",
                        "\"autoSpaceKrEn\":{},\"autoSpaceKrNum\":{},\"verticalAlign\":{},",
                        "\"englishBreakUnit\":{},\"koreanBreakUnit\":{},",
                        "\"tabAutoLeft\":{},\"tabAutoRight\":{},\"tabStops\":[{}],\"defaultTabSpacing\":{},",
                        "{},\"borderSpacing\":[{},{},{},{}],",
                        "\"borderConnect\":{},\"borderIgnoreMargin\":{}}}"
                    ),
                    align_str,
                    ps.line_spacing, ps.line_spacing_type,
                    dialog_margin_left_px, dialog_margin_right_px, dialog_indent_px,
                    // spacing_before/after는 원본 HWPUNIT → px (1x) 변환 (Task #9)
                    // ResolvedParaStyle은 /2.0이 적용되어 UI 표시에 부적합
                    raw_ps.map(|r| crate::renderer::hwpunit_to_px(r.spacing_before, self.dpi)).unwrap_or(ps.spacing_before),
                    raw_ps.map(|r| crate::renderer::hwpunit_to_px(r.spacing_after, self.dpi)).unwrap_or(ps.spacing_after),
                    para_shape_id,
                    head_str, ps.para_level, ps.numbering_id,
                    widow_orphan, keep_with_next, keep_lines, page_break_before,
                    font_line_height, single_line,
                    auto_space_kr_en, auto_space_kr_num, vertical_align,
                    english_break_unit, korean_break_unit,
                    tab_auto_left, tab_auto_right, tab_stops_json, default_tab_spacing,
                    border_fill_json,
                    border_spacing[0], border_spacing[1], border_spacing[2], border_spacing[3],
                    border_connect, border_ignore_margin,
                )
            }
            None => {
                format!(
                    concat!(
                        "{{\"alignment\":\"justify\",\"lineSpacing\":160.0,\"lineSpacingType\":\"Percent\",",
                        "\"marginLeft\":0.0,\"marginRight\":0.0,\"indent\":0.0,",
                        "\"spacingBefore\":0.0,\"spacingAfter\":0.0,\"paraShapeId\":{},",
                        "\"headType\":\"None\",\"paraLevel\":0,\"numberingId\":0,",
                        "\"widowOrphan\":false,\"keepWithNext\":false,\"keepLines\":false,\"pageBreakBefore\":false,",
                        "\"fontLineHeight\":false,\"singleLine\":false,",
                        "\"autoSpaceKrEn\":false,\"autoSpaceKrNum\":false,\"verticalAlign\":0,",
                        "\"englishBreakUnit\":0,\"koreanBreakUnit\":0,",
                        "\"tabAutoLeft\":false,\"tabAutoRight\":false,\"tabStops\":[],\"defaultTabSpacing\":{},",
                        "\"borderFillId\":0,",
                        "\"borderLeft\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderRight\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderTop\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"borderBottom\":{{\"type\":0,\"width\":0,\"color\":\"#000000\"}},",
                        "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0,",
                        "\"borderSpacing\":[0,0,0,0],",
                        "\"borderConnect\":false,\"borderIgnoreMargin\":false}}"
                    ),
                    para_shape_id, default_tab_spacing
                )
            }
        }
    }

    /// 글꼴 이름으로 font_id를 조회하거나 새로 생성한다 (네이티브).
    pub fn find_or_create_font_id_native(&mut self, name: &str) -> i32 {
        let font_faces = &self.document.doc_info.font_faces;

        // 한글(0번) 카테고리에서 검색
        if !font_faces.is_empty() {
            for (idx, font) in font_faces[0].iter().enumerate() {
                if font.name == name {
                    return idx as i32;
                }
            }
        }

        // 없으면 7개 전체 카테고리에 동일 이름으로 신규 등록
        let new_font = crate::model::style::Font {
            raw_data: None,
            name: name.to_string(),
            alt_type: 0,
            alt_name: None,
            type_info: None,
            default_name: None,
            subst_font: None,
        };

        let font_faces = &mut self.document.doc_info.font_faces;
        // font_faces가 7개 미만이면 확장
        while font_faces.len() < 7 {
            font_faces.push(Vec::new());
        }

        let new_id = font_faces[0].len();
        for lang in 0..7 {
            font_faces[lang].push(new_font.clone());
        }

        // raw_stream 보존: 7개 언어 카테고리에 FACE_NAME surgical insert
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let face_data = crate::serializer::doc_info::serialize_face_name(&new_font);
            let _ = crate::serializer::doc_info::surgical_insert_font_all_langs(raw, &face_data);
        }
        new_id as i32
    }

    /// 특정 언어 카테고리에서 글꼴 이름으로 ID를 찾거나, 없으면 해당 카테고리에만 등록한다.
    pub fn find_or_create_font_id_for_lang(&mut self, lang: usize, name: &str) -> i32 {
        if lang >= 7 {
            return -1;
        }
        let font_faces = &self.document.doc_info.font_faces;
        if font_faces.len() <= lang {
            return -1;
        }

        // 해당 언어 카테고리에서 검색
        for (idx, font) in font_faces[lang].iter().enumerate() {
            if font.name == name {
                return idx as i32;
            }
        }

        // 없으면 해당 카테고리에만 등록 (다른 언어 카테고리 font_faces 길이 맞추기)
        let new_font = crate::model::style::Font {
            raw_data: None,
            name: name.to_string(),
            alt_type: 0,
            alt_name: None,
            type_info: None,
            default_name: None,
            subst_font: None,
        };

        let font_faces = &mut self.document.doc_info.font_faces;
        while font_faces.len() < 7 {
            font_faces.push(Vec::new());
        }

        // 모든 카테고리의 길이를 맞추기 위해 전체에 등록
        let new_id = font_faces[lang].len();
        for l in 0..7 {
            if l == lang {
                font_faces[l].push(new_font.clone());
            } else {
                // 다른 카테고리에는 placeholder 등록 (길이 동기화)
                let placeholder = if !font_faces[l].is_empty() {
                    // 첫 번째 폰트를 복제 (기본 글꼴)
                    font_faces[l][0].clone()
                } else {
                    new_font.clone()
                };
                font_faces[l].push(placeholder);
            }
        }

        // raw_stream 보존
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let face_data = crate::serializer::doc_info::serialize_face_name(&new_font);
            let _ = crate::serializer::doc_info::surgical_insert_font_all_langs(raw, &face_data);
        }
        new_id as i32
    }

    /// 글자 서식 적용 (네이티브) — 본문 문단
    pub fn apply_char_format_native(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        if sec_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)));
        }
        if para_idx >= self.document.sections[sec_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        let mut mods = parse_char_shape_mods(props_json);
        // border/fill JSON이 있으면 BorderFill 생성/재사용하여 border_fill_id 설정
        if json_has_border_keys(props_json) {
            let bf_id = self.create_border_fill_from_json(props_json);
            mods.border_fill_id = Some(bf_id);
        }
        self.apply_char_mods_to_paragraph(sec_idx, para_idx, start_offset, end_offset, &mods);

        // 텍스트 폭/높이에 영향을 주는 글자 모양 변경 시 LineSeg 재계산.
        // 장평/자간은 글꼴 크기처럼 줄나눔과 페이지네이션을 바꾼다.
        if char_shape_mods_affect_text_flow(&mods) {
            let styles = resolve_styles(&self.document.doc_info, self.dpi);
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let para_shape_id = self.document.sections[sec_idx].paragraphs[para_idx].para_shape_id;
            let para_style = styles.para_styles.get(para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = (col_width - margin_left - margin_right).max(1.0);
            // 원본 LineSeg 무효화 → reflow가 max_font_size에서 새로 계산
            self.document.sections[sec_idx].paragraphs[para_idx]
                .line_segs
                .clear();
            reflow_line_segs(
                &mut self.document.sections[sec_idx].paragraphs[para_idx],
                available_width,
                &styles,
                self.dpi,
            );
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec_idx,
            para: para_idx,
            start: start_offset,
            end: end_offset,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 글자 서식 ID 직접 복원 (네이티브) — 본문 문단.
    ///
    /// Undo/Redo에서는 `CharProperties` JSON을 다시 적용하지 않고, 적용 전/후
    /// `char_shape_id`를 직접 복원한다. 조회 JSON은 UI 상태 표현용 값이 섞여
    /// 있으므로 history 복원 payload로 재해석하지 않는다.
    pub fn set_char_shape_id_native(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        char_shape_id: u32,
    ) -> Result<String, HwpError> {
        if sec_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)));
        }
        if para_idx >= self.document.sections[sec_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }
        if char_shape_id as usize >= self.document.doc_info.char_shapes.len() {
            return Err(HwpError::RenderError(format!(
                "글자 모양 ID {} 범위 초과 (총 {}개)",
                char_shape_id,
                self.document.doc_info.char_shapes.len()
            )));
        }

        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let available_width = {
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let para_shape_id = section.paragraphs[para_idx].para_shape_id;
            let para_style = styles.para_styles.get(para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            (col_width - margin_left - margin_right).max(1.0)
        };

        {
            let para = &mut self.document.sections[sec_idx].paragraphs[para_idx];
            para.apply_char_shape_range(start_offset, end_offset, char_shape_id);
            reflow_line_segs(para, available_width, &styles, self.dpi);
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec_idx,
            para: para_idx,
            start: start_offset,
            end: end_offset,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 글자 서식 적용 (네이티브) — 셀 내 문단
    pub fn apply_char_format_in_cell_native(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let mut mods = parse_char_shape_mods(props_json);
        if json_has_border_keys(props_json) {
            let bf_id = self.create_border_fill_from_json(props_json);
            mods.border_fill_id = Some(bf_id);
        }

        // 셀 내 문단의 기존 char_shape_id를 기반으로 새 ID 생성
        {
            let para = self
                .get_cell_paragraph_ref(
                    sec_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                )
                .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
            let base_id = para.char_shape_id_at(start_offset).unwrap_or(0);
            let new_id = self.document.find_or_create_char_shape(base_id, &mods);

            // 셀 문단에 범위 적용
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            cell_para.apply_char_shape_range(start_offset, end_offset, new_id);
        }

        // 텍스트 폭/높이에 영향을 주는 글자 모양 변경 시 셀 내 LineSeg 재계산.
        if char_shape_mods_affect_text_flow(&mods) {
            let dpi = self.dpi;
            let styles = resolve_styles(&self.document.doc_info, dpi);
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            let para_shape_id = cell_para.para_shape_id;
            let para_style = styles.para_styles.get(para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = (col_width - margin_left - margin_right).max(1.0);
            cell_para.line_segs.clear();
            reflow_line_segs(cell_para, available_width, &styles, dpi);

            // 표 dirty 마킹 — 셀 높이 재계산 필요
            if let Control::Table(ref mut t) =
                self.document.sections[sec_idx].paragraphs[parent_para_idx].controls[control_idx]
            {
                t.dirty = true;
            }
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec_idx,
            para: parent_para_idx,
            start: start_offset,
            end: end_offset,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 글자 서식 ID 직접 복원 (네이티브) — 셀 내 문단.
    pub fn set_char_shape_id_in_cell_native(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        char_shape_id: u32,
    ) -> Result<String, HwpError> {
        if char_shape_id as usize >= self.document.doc_info.char_shapes.len() {
            return Err(HwpError::RenderError(format!(
                "글자 모양 ID {} 범위 초과 (총 {}개)",
                char_shape_id,
                self.document.doc_info.char_shapes.len()
            )));
        }

        {
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            cell_para.apply_char_shape_range(start_offset, end_offset, char_shape_id);
        }

        self.reflow_cell_paragraph(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.mark_cell_control_dirty(sec_idx, parent_para_idx, control_idx);
        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec_idx,
            para: parent_para_idx,
            start: start_offset,
            end: end_offset,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 서식 적용 (네이티브) — 본문 문단
    pub fn apply_para_format_native(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        if sec_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)));
        }
        if para_idx >= self.document.sections[sec_idx].paragraphs.len() {
            if let Some(src) = self.virtual_endnote_para_source(sec_idx, para_idx) {
                return self.apply_para_format_in_footnote_native(
                    src.section_index,
                    src.para_index,
                    src.control_index,
                    src.note_para_index,
                    props_json,
                );
            }
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        let mut mods = parse_para_shape_mods(props_json);

        // 탭 설정 변경 처리: TabDef 생성 → tab_def_id 세팅
        if json_has_tab_keys(props_json) {
            let base_id = self.document.sections[sec_idx].paragraphs[para_idx].para_shape_id;
            let base_tab_def_id = self
                .document
                .doc_info
                .para_shapes
                .get(base_id as usize)
                .map(|ps| ps.tab_def_id)
                .unwrap_or(0);
            let new_td = build_tab_def_from_json(
                props_json,
                base_tab_def_id,
                &self.document.doc_info.tab_defs,
            );
            let new_tab_id = self.document.find_or_create_tab_def(new_td);
            mods.tab_def_id = Some(new_tab_id);
        }

        // 테두리/배경 변경 처리: BorderFill 생성 → border_fill_id 세팅
        if json_has_border_keys(props_json) {
            let bf_id = self.create_border_fill_from_json(props_json);
            mods.border_fill_id = Some(bf_id);
        }
        if let Some(arr) = parse_json_i16_array(props_json, "borderSpacing", 4) {
            mods.border_spacing = Some([arr[0], arr[1], arr[2], arr[3]]);
        }

        let base_id = self.document.sections[sec_idx].paragraphs[para_idx].para_shape_id;
        let new_id = self.document.find_or_create_para_shape(base_id, &mods);
        self.document.sections[sec_idx].paragraphs[para_idx].para_shape_id = new_id;

        // 줄간격 변경 시 LineSeg 재계산 (compose는 LineSeg 값을 그대로 사용하므로)
        if mods.line_spacing.is_some() || mods.line_spacing_type.is_some() {
            let styles = resolve_styles(&self.document.doc_info, self.dpi);
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let para_style = styles.para_styles.get(new_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = (col_width - margin_left - margin_right).max(1.0);
            reflow_line_segs(
                &mut self.document.sections[sec_idx].paragraphs[para_idx],
                available_width,
                &styles,
                self.dpi,
            );
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 서식 ID 직접 복원 (네이티브) — 본문 문단.
    ///
    /// Undo/Redo에서는 `ParaProperties` JSON을 다시 적용하지 않고, 적용 전/후
    /// `para_shape_id`를 직접 복원한다. 조회 JSON은 UI용 px 단위가 섞여 있어
    /// raw 값을 기대하는 apply parser에 재투입하면 단위가 깨질 수 있다.
    pub fn set_para_shape_id_native(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        para_shape_id: u16,
    ) -> Result<String, HwpError> {
        if sec_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", sec_idx)));
        }
        if para_idx >= self.document.sections[sec_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }
        if para_shape_id as usize >= self.document.doc_info.para_shapes.len() {
            return Err(HwpError::RenderError(format!(
                "문단 모양 ID {} 범위 초과 (총 {}개)",
                para_shape_id,
                self.document.doc_info.para_shapes.len()
            )));
        }

        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let available_width = {
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let para_style = styles.para_styles.get(para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            (col_width - margin_left - margin_right).max(1.0)
        };

        {
            let para = &mut self.document.sections[sec_idx].paragraphs[para_idx];
            para.para_shape_id = para_shape_id;
            reflow_line_segs(para, available_width, &styles, self.dpi);
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 서식 적용 (네이티브) — 셀 내 문단
    pub fn apply_para_format_in_cell_native(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let mut mods = parse_para_shape_mods(props_json);

        // 탭 설정 변경 처리: TabDef 생성 → tab_def_id 세팅
        if json_has_tab_keys(props_json) {
            let para = self
                .get_cell_paragraph_ref(
                    sec_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                )
                .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
            let base_tab_def_id = self
                .document
                .doc_info
                .para_shapes
                .get(para.para_shape_id as usize)
                .map(|ps| ps.tab_def_id)
                .unwrap_or(0);
            let new_td = build_tab_def_from_json(
                props_json,
                base_tab_def_id,
                &self.document.doc_info.tab_defs,
            );
            let new_tab_id = self.document.find_or_create_tab_def(new_td);
            mods.tab_def_id = Some(new_tab_id);
        }

        // 테두리/배경 변경 처리: BorderFill 생성 → border_fill_id 세팅
        if json_has_border_keys(props_json) {
            let bf_id = self.create_border_fill_from_json(props_json);
            mods.border_fill_id = Some(bf_id);
        }
        if let Some(arr) = parse_json_i16_array(props_json, "borderSpacing", 4) {
            mods.border_spacing = Some([arr[0], arr[1], arr[2], arr[3]]);
        }

        let new_id;
        {
            let para = self
                .get_cell_paragraph_ref(
                    sec_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                )
                .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
            let base_id = para.para_shape_id;
            new_id = self.document.find_or_create_para_shape(base_id, &mods);

            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            cell_para.para_shape_id = new_id;
        }

        // 줄간격 변경 시 셀 내 문단 LineSeg 재계산
        if mods.line_spacing.is_some() || mods.line_spacing_type.is_some() {
            let dpi = self.dpi;
            let styles = resolve_styles(&self.document.doc_info, dpi);
            let section = &self.document.sections[sec_idx];
            let page_def = &section.section_def.page_def;
            let column_def = DocumentCore::find_initial_column_def(&section.paragraphs);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_width = layout
                .column_areas
                .first()
                .map(|a| a.width)
                .unwrap_or(layout.body_area.width);
            let para_style = styles.para_styles.get(new_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = (col_width - margin_left - margin_right).max(1.0);
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            reflow_line_segs(cell_para, available_width, &styles, dpi);
        }

        // 표 dirty 마킹 — measure_section_incremental이 셀 높이를 재계산하도록
        {
            use crate::model::control::Control;
            if let Control::Table(ref mut t) =
                self.document.sections[sec_idx].paragraphs[parent_para_idx].controls[control_idx]
            {
                t.dirty = true;
            }
        }

        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: parent_para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 서식 ID 직접 복원 (네이티브) — 셀 내 문단.
    pub fn set_cell_para_shape_id_native(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        para_shape_id: u16,
    ) -> Result<String, HwpError> {
        if para_shape_id as usize >= self.document.doc_info.para_shapes.len() {
            return Err(HwpError::RenderError(format!(
                "문단 모양 ID {} 범위 초과 (총 {}개)",
                para_shape_id,
                self.document.doc_info.para_shapes.len()
            )));
        }

        {
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            cell_para.para_shape_id = para_shape_id;
        }

        self.reflow_cell_paragraph(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.mark_cell_control_dirty(sec_idx, parent_para_idx, control_idx);
        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: parent_para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 문서 내 동일 style_id를 사용하는 기존 문단의 para_shape_id를 찾는다.
    fn find_reference_para_shape_for_style(&self, style_id: usize) -> Option<u16> {
        use crate::model::control::Control;

        for section in &self.document.sections {
            for para in &section.paragraphs {
                if para.style_id as usize == style_id {
                    return Some(para.para_shape_id);
                }
                for ctrl in &para.controls {
                    if let Control::Table(t) = ctrl {
                        for cell in &t.cells {
                            for cp in &cell.paragraphs {
                                if cp.style_id as usize == style_id {
                                    return Some(cp.para_shape_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 문서의 ParaShape 풀에서 동일 numbering_id·head_type이면서 target level인 것을 찾는다.
    fn find_para_shape_with_nid_and_level(
        &self,
        nid: u16,
        head_type: crate::model::style::HeadType,
        level: u8,
    ) -> Option<u16> {
        for (i, ps) in self.document.doc_info.para_shapes.iter().enumerate() {
            if ps.numbering_id == nid && ps.head_type == head_type && ps.para_level == level {
                return Some(i as u16);
            }
        }
        None
    }

    /// 스타일 이름에서 개요 수준을 추출한다. "개요 N" → Some(N-1)
    fn parse_outline_level_from_style(&self, style_id: usize) -> Option<u8> {
        let style = self.document.doc_info.styles.get(style_id)?;
        let name = style.local_name.trim();
        let rest = name.strip_prefix("개요")?.trim();
        let level_num = rest.parse::<u8>().ok()?;
        if level_num >= 1 && level_num <= 10 {
            Some(level_num - 1)
        } else {
            None
        }
    }

    /// 스타일에 맞는 ParaShape ID를 결정한다.
    ///
    /// current_psid: 현재 문단의 ParaShape ID (번호 문맥 보존용)
    ///
    /// 번호가 있는 문단의 스타일을 변경할 때 numbering_id를 보존하여
    /// 후속 문단의 번호 연속성을 유지한다.
    fn resolve_style_para_shape_id(&mut self, style_id: usize, current_psid: u16) -> u16 {
        use crate::model::style::HeadType;

        let current_ps = self
            .document
            .doc_info
            .para_shapes
            .get(current_psid as usize)
            .cloned();
        let current_head = current_ps
            .as_ref()
            .map(|ps| ps.head_type)
            .unwrap_or(HeadType::None);
        let current_nid = current_ps.as_ref().map(|ps| ps.numbering_id).unwrap_or(0);

        // ── 현재 문단이 번호/개요를 가지고 있는 경우 ──
        // numbering_id와 head_type을 보존하고 para_level만 변경
        if current_head != HeadType::None {
            // 대상 스타일의 개요 수준 결정
            let target_level = self.parse_outline_level_from_style(style_id).or_else(|| {
                // 스타일 이름에서 못 찾으면 참조 문단에서 추출
                self.find_reference_para_shape_for_style(style_id)
                    .and_then(|psid| self.document.doc_info.para_shapes.get(psid as usize))
                    .filter(|ps| ps.head_type != HeadType::None)
                    .map(|ps| ps.para_level)
            });

            if let Some(level) = target_level {
                // 같은 numbering_id·head_type에서 target level인 ParaShape 검색
                if let Some(found) =
                    self.find_para_shape_with_nid_and_level(current_nid, current_head, level)
                {
                    return found;
                }

                // 없으면 현재 ParaShape 기반으로 level + 여백 변경하여 생성
                let current_level = current_ps.as_ref().map(|ps| ps.para_level).unwrap_or(0);
                let current_margin = current_ps.as_ref().map(|ps| ps.margin_left).unwrap_or(0);
                // 수준별 여백 증감: 수준 1단계당 2000 HWPUNIT
                let margin_delta = (level as i32 - current_level as i32) * 2000;
                let new_margin = (current_margin + margin_delta).max(0);
                let mods = crate::model::style::ParaShapeMods {
                    para_level: Some(level),
                    margin_left: Some(new_margin),
                    ..Default::default()
                };
                return self.document.find_or_create_para_shape(current_psid, &mods);
            }
        }

        // ── 현재 문단에 번호가 없는 경우 (바탕글 등) ──
        // 일반 스타일은 기존 문단의 실효 ParaShape가 아니라 스타일 정의값을 따른다.
        // 참조 문단을 우선하면 직접 서식이 섞인 문단 값이 스타일 적용값으로 번질 수 있다.
        let style = match self.document.doc_info.styles.get(style_id) {
            Some(s) => s.clone(),
            None => return 0,
        };
        let base_psid = style.para_shape_id;

        // 스타일 이름에서 "개요 N" 패턴 감지
        if let Some(level) = self.parse_outline_level_from_style(style_id) {
            // Outline 문단의 numbering_id는 0 (렌더링 시 구역의 outline_numbering_id로 해석)
            let mods = crate::model::style::ParaShapeMods {
                head_type: Some(HeadType::Outline),
                para_level: Some(level),
                numbering_id: Some(0),
                ..Default::default()
            };
            return self.document.find_or_create_para_shape(base_psid, &mods);
        }

        // 일반 스타일 → 기본 ParaShape 사용
        base_psid
    }

    /// 본문 문단의 LineSeg를 현재 CharShape/ParaShape 기준으로 다시 계산한다.
    pub(crate) fn reflow_body_paragraph(&mut self, sec_idx: usize, para_idx: usize) {
        let para_shape_id = match self
            .document
            .sections
            .get(sec_idx)
            .and_then(|s| s.paragraphs.get(para_idx))
        {
            Some(para) => para.para_shape_id,
            None => return,
        };
        let styles = resolve_styles(&self.document.doc_info, self.dpi);
        let available_width =
            body_available_width_for_para_shape(self, sec_idx, para_shape_id, &styles);
        if let Some(para) = self
            .document
            .sections
            .get_mut(sec_idx)
            .and_then(|s| s.paragraphs.get_mut(para_idx))
        {
            para.line_segs.clear();
            reflow_line_segs(para, available_width, &styles, self.dpi);
        }
    }

    /// 스타일 적용 (네이티브) — 본문 문단
    pub fn apply_style_native(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        style_id: usize,
    ) -> Result<String, HwpError> {
        let style = self
            .document
            .doc_info
            .styles
            .get(style_id)
            .cloned()
            .ok_or_else(|| HwpError::RenderError(format!("스타일 {} 범위 초과", style_id)))?;
        let new_char_shape_id = style.char_shape_id as u32;

        // 현재 문단의 기존 스타일/문단 모양을 먼저 읽어서 직접 서식 여부를 판단한다.
        let (current_style_id, current_psid) = self
            .document
            .sections
            .get(sec_idx)
            .and_then(|s| s.paragraphs.get(para_idx))
            .map(|p| (p.style_id, p.para_shape_id))
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 {}/{} 범위 초과", sec_idx, para_idx))
            })?;
        let old_style = self
            .document
            .doc_info
            .styles
            .get(current_style_id as usize)
            .cloned();

        if style.style_type == 1 {
            let text_len = {
                let para = self
                    .document
                    .sections
                    .get_mut(sec_idx)
                    .and_then(|s| s.paragraphs.get_mut(para_idx))
                    .ok_or_else(|| {
                        HwpError::RenderError(format!("문단 {}/{} 범위 초과", sec_idx, para_idx))
                    })?;
                para.apply_char_shape_to_entire_text(new_char_shape_id);
                para.text.chars().count()
            };

            self.reflow_body_paragraph(sec_idx, para_idx);
            self.document.sections[sec_idx].raw_stream = None;
            self.rebuild_section(sec_idx);
            self.event_log.push(DocumentEvent::CharFormatChanged {
                section: sec_idx,
                para: para_idx,
                start: 0,
                end: text_len,
            });
            return Ok("{\"ok\":true}".to_string());
        }

        let new_para_shape_id = match old_style.as_ref() {
            Some(old) if current_psid != old.para_shape_id => current_psid,
            _ => self.resolve_style_para_shape_id(style_id, current_psid),
        };

        let para = self
            .document
            .sections
            .get_mut(sec_idx)
            .and_then(|s| s.paragraphs.get_mut(para_idx))
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 {}/{} 범위 초과", sec_idx, para_idx))
            })?;

        para.style_id = style_id as u8;
        para.para_shape_id = new_para_shape_id;
        if let Some(old) = old_style {
            para.replace_style_char_shape_preserving_overrides(
                old.char_shape_id as u32,
                new_char_shape_id,
            );
        } else {
            para.set_single_char_shape(new_char_shape_id);
        }

        self.reflow_body_paragraph(sec_idx, para_idx);
        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 스타일 적용 (네이티브) — 셀 내 문단
    pub fn apply_cell_style_native(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        style_id: usize,
    ) -> Result<String, HwpError> {
        let style = self
            .document
            .doc_info
            .styles
            .get(style_id)
            .cloned()
            .ok_or_else(|| HwpError::RenderError(format!("스타일 {} 범위 초과", style_id)))?;
        let new_char_shape_id = style.char_shape_id as u32;

        // 현재 셀 문단의 기존 스타일/문단 모양을 먼저 읽어서 직접 서식 여부를 판단한다.
        let (current_style_id, current_psid) = self
            .get_cell_paragraph_ref(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .map(|p| (p.style_id, p.para_shape_id))
            .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
        let old_style = self
            .document
            .doc_info
            .styles
            .get(current_style_id as usize)
            .cloned();

        if style.style_type == 1 {
            let text_len = {
                let cell_para = self.get_cell_paragraph_mut(
                    sec_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                )?;
                cell_para.apply_char_shape_to_entire_text(new_char_shape_id);
                cell_para.text.chars().count()
            };

            self.reflow_cell_paragraph(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            );
            self.mark_cell_control_dirty(sec_idx, parent_para_idx, control_idx);
            self.document.sections[sec_idx].raw_stream = None;
            self.rebuild_section(sec_idx);
            self.event_log.push(DocumentEvent::CharFormatChanged {
                section: sec_idx,
                para: parent_para_idx,
                start: 0,
                end: text_len,
            });
            return Ok("{\"ok\":true}".to_string());
        }

        let new_para_shape_id = match old_style.as_ref() {
            Some(old) if current_psid != old.para_shape_id => current_psid,
            _ => self.resolve_style_para_shape_id(style_id, current_psid),
        };

        {
            let cell_para = self.get_cell_paragraph_mut(
                sec_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            cell_para.style_id = style_id as u8;
            cell_para.para_shape_id = new_para_shape_id;
            if let Some(old) = old_style {
                cell_para.replace_style_char_shape_preserving_overrides(
                    old.char_shape_id as u32,
                    new_char_shape_id,
                );
            } else {
                cell_para.set_single_char_shape(new_char_shape_id);
            }
        }

        self.reflow_cell_paragraph(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.mark_cell_control_dirty(sec_idx, parent_para_idx, control_idx);
        self.document.sections[sec_idx].raw_stream = None;
        self.rebuild_section(sec_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: sec_idx,
            para: parent_para_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 본문 문단에 글자 서식 적용 헬퍼
    pub(crate) fn apply_char_mods_to_paragraph(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        mods: &crate::model::style::CharShapeMods,
    ) {
        let base_id = self.document.sections[sec_idx].paragraphs[para_idx]
            .char_shape_id_at(start_offset)
            .unwrap_or(0);
        let new_id = self.document.find_or_create_char_shape(base_id, mods);
        self.document.sections[sec_idx].paragraphs[para_idx].apply_char_shape_range(
            start_offset,
            end_offset,
            new_id,
        );
    }

    /// 문단 번호 시작 방식을 설정한다.
    /// mode: 0 = 앞 번호 목록에 이어 (기본), 1 = 이전 번호 목록에 이어, 2 = 새 번호 목록 시작
    /// start_num: mode=2일 때 시작 번호
    pub fn set_numbering_restart_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        mode: u8,
        start_num: u32,
    ) -> Result<String, crate::error::HwpError> {
        use crate::model::paragraph::NumberingRestart;

        if section_idx >= self.document.sections.len() {
            return Err(crate::error::HwpError::RenderError(
                "구역 범위 초과".to_string(),
            ));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(crate::error::HwpError::RenderError(
                "문단 범위 초과".to_string(),
            ));
        }

        let restart = match mode {
            0 => None,
            1 => Some(NumberingRestart::ContinuePrevious),
            2 => Some(NumberingRestart::NewStart(start_num)),
            _ => None,
        };

        self.document.sections[section_idx].paragraphs[para_idx].numbering_restart = restart;
        self.document.sections[section_idx].raw_stream = None;

        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(crate::document_core::helpers::json_ok())
    }

    /// 감추기(PageHide) 컨트롤을 현재 문단에 삽입 또는 갱신한다.
    /// flags: { hideHeader, hideFooter, hideMasterPage, hideBorder, hideFill, hidePageNum }
    pub fn set_page_hide_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        hide_header: bool,
        hide_footer: bool,
        hide_master_page: bool,
        hide_border: bool,
        hide_fill: bool,
        hide_page_num: bool,
    ) -> Result<String, crate::error::HwpError> {
        use crate::model::control::{Control, PageHide};

        if section_idx >= self.document.sections.len() {
            return Err(crate::error::HwpError::RenderError(
                "구역 범위 초과".to_string(),
            ));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(crate::error::HwpError::RenderError(
                "문단 범위 초과".to_string(),
            ));
        }

        let all_false = !hide_header
            && !hide_footer
            && !hide_master_page
            && !hide_border
            && !hide_fill
            && !hide_page_num;

        let para = &mut self.document.sections[section_idx].paragraphs[para_idx];

        // 기존 PageHide 컨트롤 찾기
        let existing_idx = para
            .controls
            .iter()
            .position(|c| matches!(c, Control::PageHide(_)));

        if all_false {
            // 모두 false → 기존 PageHide 제거
            if let Some(idx) = existing_idx {
                para.controls.remove(idx);
                if idx < para.ctrl_data_records.len() {
                    para.ctrl_data_records.remove(idx);
                }
            }
        } else {
            let ph = PageHide {
                hide_header,
                hide_footer,
                hide_master_page,
                hide_border,
                hide_fill,
                hide_page_num,
            };
            if let Some(idx) = existing_idx {
                // 기존 컨트롤 갱신
                para.controls[idx] = Control::PageHide(ph);
            } else {
                // 새 컨트롤 삽입 (문단 맨 앞)
                para.controls.insert(0, Control::PageHide(ph));
                para.ctrl_data_records.insert(0, None);
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(crate::document_core::helpers::json_ok())
    }

    /// 현재 문단의 PageHide 상태를 조회한다.
    pub fn get_page_hide_native(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, crate::error::HwpError> {
        use crate::model::control::Control;

        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 범위 초과".to_string()))?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| crate::error::HwpError::RenderError("문단 범위 초과".to_string()))?;

        for ctrl in &para.controls {
            if let Control::PageHide(ph) = ctrl {
                return Ok(format!(
                    "{{\"ok\":true,\"exists\":true,\"hideHeader\":{},\"hideFooter\":{},\"hideMasterPage\":{},\"hideBorder\":{},\"hideFill\":{},\"hidePageNum\":{}}}",
                    ph.hide_header, ph.hide_footer, ph.hide_master_page,
                    ph.hide_border, ph.hide_fill, ph.hide_page_num
                ));
            }
        }
        Ok("{\"ok\":true,\"exists\":false}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::char_shape_mods_affect_text_flow;
    use crate::model::style::CharShapeMods;

    #[test]
    fn char_ratio_and_spacing_changes_require_text_reflow() {
        let mods = CharShapeMods {
            ratios: Some([99; 7]),
            ..Default::default()
        };
        assert!(char_shape_mods_affect_text_flow(&mods));

        let mods = CharShapeMods {
            spacings: Some([-1; 7]),
            ..Default::default()
        };
        assert!(char_shape_mods_affect_text_flow(&mods));
    }

    #[test]
    fn paint_only_char_shape_changes_do_not_require_text_reflow() {
        let mods = CharShapeMods {
            underline: Some(true),
            ..Default::default()
        };
        assert!(!char_shape_mods_affect_text_flow(&mods));
    }
}
