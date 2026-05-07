//! document_core 헬퍼 함수 모음
//!
//! JSON 파싱, 색상 변환, HTML 처리, CSS 파싱 등 유틸리티 함수.

use crate::model::paragraph::Paragraph;
use crate::model::control::Control;
use crate::model::style::BorderLineType;
use crate::model::path::PathSegment;
use crate::error::HwpError;

/// 문단의 탐색 가능한 텍스트 길이를 반환한다.
///
/// CharOverlap(TCPS)은 inline 컨트롤이라 para.text에 포함되지 않지만,
/// 레이아웃에서 각 overlap이 char_offset 1개를 차지하므로 보정한다.
pub(crate) fn navigable_text_len(para: &Paragraph) -> usize {
    let text_len = para.text.chars().count();
    let char_overlap_count = para.controls.iter()
        .filter(|c| matches!(c, Control::CharOverlap(_)))
        .count();
    // 인라인 컨트롤의 최대 position을 구하여, text_len보다 클 경우 확장
    let positions = find_control_text_positions(para);
    let max_inline_pos = para.controls.iter().enumerate()
        .filter(|(_, c)| matches!(c,
            Control::Shape(_) | Control::Table(_) |
            Control::Picture(_) | Control::Equation(_) |
            Control::Footnote(_) | Control::Endnote(_)
        ))
        .filter_map(|(i, _)| positions.get(i).copied())
        .max()
        .map(|p| p + 1)  // position 뒤에 커서가 위치할 수 있으므로 +1
        .unwrap_or(0);
    text_len.max(max_inline_pos) + char_overlap_count
}

/// 문단 내 컨트롤의 텍스트 위치를 복원한다.
///
/// HWP 파서가 텍스트에서 컨트롤 문자(각 8 UTF-16 코드 유닛)를 제거한다.
/// char_offsets의 갭(연속된 위치 차이 > 문자 폭)으로 컨트롤 원래 위치를 복원한다.
///
/// 논리적 오프셋 → 텍스트 오프셋 변환.
/// 논리적 오프셋: 텍스트 문자 + 인라인 컨트롤을 각각 1로 세는 위치.
/// 반환: (텍스트 char_offset, 컨트롤 직후 여부)
pub(crate) fn logical_to_text_offset(para: &Paragraph, logical_offset: usize) -> (usize, bool) {
    let ctrl_positions = find_control_text_positions(para);
    if ctrl_positions.is_empty() {
        return (logical_offset, false);
    }

    // 논리적 위치에서 컨트롤 슬롯을 구성
    // 텍스트 "abc[ctrl]XYZ" → 논리적: a(0) b(1) c(2) [ctrl](3) X(4) Y(5) Z(6)
    // ctrl_positions = [3] (텍스트 인덱스 3에 컨트롤 삽입)
    // 정렬된 (텍스트위치, 컨트롤인덱스) 목록
    let mut sorted_ctrls: Vec<(usize, usize)> = ctrl_positions.iter().enumerate()
        .map(|(ci, &pos)| (pos, ci))
        .collect();
    sorted_ctrls.sort_by_key(|(pos, _)| *pos);

    let text_len = para.text.chars().count();
    let mut text_idx = 0usize;
    let mut logical_idx = 0usize;
    let mut ctrl_cursor = 0usize; // sorted_ctrls 내 현재 위치

    while logical_idx < logical_offset {
        // 현재 text_idx 위치에 컨트롤이 있는지 확인
        if ctrl_cursor < sorted_ctrls.len() && sorted_ctrls[ctrl_cursor].0 == text_idx {
            // 컨트롤 슬롯
            logical_idx += 1;
            ctrl_cursor += 1;
            if logical_idx == logical_offset {
                return (text_idx, true);
            }
        }
        // 텍스트 문자
        if text_idx < text_len {
            text_idx += 1;
            logical_idx += 1;
        } else {
            break;
        }
    }
    (text_idx, false)
}

/// 텍스트 오프셋 → 논리적 오프셋 변환.
/// text_offset 위치 앞에 있는 컨트롤 수만큼 논리적 위치가 밀림.
pub(crate) fn text_to_logical_offset(para: &Paragraph, text_offset: usize) -> usize {
    let ctrl_positions = find_control_text_positions(para);
    if ctrl_positions.is_empty() {
        return text_offset;
    }

    // text_offset 이전(미만)에 있는 컨트롤 수를 더함
    // pos < text_offset: 해당 컨트롤은 text_offset 앞에 위치
    // pos == text_offset: 컨트롤과 텍스트가 같은 위치 → 컨트롤이 먼저
    let before_count = ctrl_positions.iter().filter(|&&pos| pos < text_offset).count();
    text_offset + before_count
}

/// 논리적 문단 길이 (텍스트 문자 + 텍스트 흐름에 위치하는 컨트롤 수).
/// find_control_text_positions에 의해 텍스트 위치가 결정되는 컨트롤만 포함.
pub(crate) fn logical_paragraph_length(para: &Paragraph) -> usize {
    let ctrl_positions = find_control_text_positions(para);
    para.text.chars().count() + ctrl_positions.len()
}

/// 반환: positions[i] = para.controls[i]가 삽입되어야 할 텍스트 문자 인덱스
///
/// 알고리즘 본체는 [`Paragraph::control_text_positions`] 로 이동했으며 (#390),
/// 본 함수는 기존 호출 경로를 유지하기 위한 thin wrapper 다.
pub(crate) fn find_control_text_positions(para: &Paragraph) -> Vec<usize> {
    para.control_text_positions()
}

/// ShapeObject에서 TextBox를 추출하는 헬퍼
pub(crate) fn get_textbox_from_shape(shape: &crate::model::shape::ShapeObject) -> Option<&crate::model::shape::TextBox> {
    use crate::model::shape::ShapeObject;
    let drawing = match shape {
        ShapeObject::Rectangle(s) => &s.drawing,
        ShapeObject::Ellipse(s) => &s.drawing,
        ShapeObject::Polygon(s) => &s.drawing,
        ShapeObject::Curve(s) => &s.drawing,
        _ => return None,
    };
    drawing.text_box.as_ref()
}

/// ShapeObject에서 TextBox 가변 참조를 추출하는 헬퍼
pub(crate) fn get_textbox_from_shape_mut(shape: &mut crate::model::shape::ShapeObject) -> Option<&mut crate::model::shape::TextBox> {
    use crate::model::shape::ShapeObject;
    let drawing = match shape {
        ShapeObject::Rectangle(s) => &mut s.drawing,
        ShapeObject::Ellipse(s) => &mut s.drawing,
        ShapeObject::Polygon(s) => &mut s.drawing,
        ShapeObject::Curve(s) => &mut s.drawing,
        _ => return None,
    };
    drawing.text_box.as_mut()
}

/// 문단 목록에서 DocumentPath를 따라 중첩 표에 대한 가변 참조를 얻는다.
///
/// 경로 형식:
/// - 종단: `[Paragraph(pi), Control(ci)]` → 해당 표 반환
/// - 중첩: `[Paragraph(pi), Control(ci), Cell(r,c), ...rest]` → 셀 내 재귀
pub(crate) fn navigate_path_to_table<'a>(
    paragraphs: &'a mut Vec<Paragraph>,
    path: &[PathSegment],
) -> Result<&'a mut crate::model::table::Table, HwpError> {
    match path {
        [PathSegment::Paragraph(pi), PathSegment::Control(ci)] => {
            let para = paragraphs.get_mut(*pi).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", pi))
            })?;
            match para.controls.get_mut(*ci) {
                Some(Control::Table(t)) => Ok(t),
                Some(_) => Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                )),
                None => Err(HwpError::RenderError(format!(
                    "컨트롤 인덱스 {} 범위 초과", ci
                ))),
            }
        }
        [PathSegment::Paragraph(pi), PathSegment::Control(ci), PathSegment::Cell(row, col), rest @ ..] =>
        {
            let para = paragraphs.get_mut(*pi).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", pi))
            })?;
            match para.controls.get_mut(*ci) {
                Some(Control::Table(t)) => {
                    let cell = t.cell_at_mut(*row, *col).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "셀({},{}) 접근 실패", row, col
                        ))
                    })?;
                    navigate_path_to_table(&mut cell.paragraphs, rest)
                }
                Some(_) => Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                )),
                None => Err(HwpError::RenderError(format!(
                    "컨트롤 인덱스 {} 범위 초과", ci
                ))),
            }
        }
        _ => Err(HwpError::RenderError("잘못된 경로 형식".to_string())),
    }
}

/// UTF-16 위치를 char 인덱스로 변환한다.
pub(crate) fn utf16_pos_to_char_idx(char_offsets: &[u32], utf16_pos: u32) -> usize {
    char_offsets.iter().position(|&off| off >= utf16_pos).unwrap_or(char_offsets.len())
}

/// 줄 정보 결과 (구조체 반환용)
pub(crate) struct LineInfoResult {
    pub line_index: usize,
    pub line_count: usize,
    pub char_start: usize,
    pub char_end: usize,
}

/// 문단이 표 컨트롤을 포함하면 해당 control_idx를 반환한다.
pub(crate) fn has_table_control(para: &Paragraph) -> Option<usize> {
    para.controls.iter().position(|c| matches!(c, Control::Table(_)))
}

/// COLORREF (BGR) → CSS 색상 문자열 변환 (클립보드용).
pub(crate) fn clipboard_color_to_css(color: u32) -> String {
    let b = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let r = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// HTML 특수문자 이스케이프 (클립보드용).
pub(crate) fn clipboard_escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 이미지 MIME 타입 감지 (클립보드용).
pub(crate) fn detect_clipboard_image_mime(data: &[u8]) -> &'static str {
    if data.len() >= 8 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        "image/png"
    } else if data.len() >= 3 && data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.len() >= 6 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")) {
        "image/gif"
    } else {
        "application/octet-stream"
    }
}

/// JSON 문자열에서 CharShapeMods를 파싱한다 (간단한 키-값 파싱).
pub(crate) fn parse_char_shape_mods(json: &str) -> crate::model::style::CharShapeMods {
    use crate::model::style::{CharShapeMods, UnderlineType};
    let mut mods = CharShapeMods::default();

    if let Some(v) = json_bool(json, "bold") { mods.bold = Some(v); }
    if let Some(v) = json_bool(json, "italic") { mods.italic = Some(v); }
    if let Some(v) = json_bool(json, "underline") { mods.underline = Some(v); }
    if let Some(v) = json_bool(json, "strikethrough") { mods.strikethrough = Some(v); }
    if let Some(v) = json_i32(json, "fontSize") { mods.base_size = Some(v); }
    if let Some(v) = json_u16(json, "fontId") { mods.font_id = Some(v); }
    if let Some(v) = json_color(json, "textColor") { mods.text_color = Some(v); }
    if let Some(v) = json_color(json, "shadeColor") { mods.shade_color = Some(v); }
    // 확장 속성
    if let Some(v) = json_str(json, "underlineType") {
        mods.underline_type = Some(match v.as_str() {
            "Bottom" => UnderlineType::Bottom,
            "Top" => UnderlineType::Top,
            _ => UnderlineType::None,
        });
    }
    if let Some(v) = json_color(json, "underlineColor") { mods.underline_color = Some(v); }
    if let Some(v) = json_i32(json, "outlineType") { mods.outline_type = Some(v as u8); }
    if let Some(v) = json_i32(json, "shadowType") { mods.shadow_type = Some(v as u8); }
    if let Some(v) = json_color(json, "shadowColor") { mods.shadow_color = Some(v); }
    if let Some(v) = json_i32(json, "shadowOffsetX") { mods.shadow_offset_x = Some(v as i8); }
    if let Some(v) = json_i32(json, "shadowOffsetY") { mods.shadow_offset_y = Some(v as i8); }
    if let Some(v) = json_color(json, "strikeColor") { mods.strike_color = Some(v); }
    if let Some(v) = json_bool(json, "subscript") { mods.subscript = Some(v); }
    if let Some(v) = json_bool(json, "superscript") { mods.superscript = Some(v); }
    if let Some(v) = json_bool(json, "emboss") { mods.emboss = Some(v); }
    if let Some(v) = json_bool(json, "engrave") { mods.engrave = Some(v); }
    // 강조점/밑줄모양/취소선모양/커닝
    if let Some(v) = json_i32(json, "emphasisDot") { mods.emphasis_dot = Some(v as u8); }
    if let Some(v) = json_i32(json, "underlineShape") { mods.underline_shape = Some(v as u8); }
    if let Some(v) = json_i32(json, "strikeShape") { mods.strike_shape = Some(v as u8); }
    if let Some(v) = json_bool(json, "kerning") { mods.kerning = Some(v); }
    // 언어별 배열
    if let Some(arr) = json_u16_array(json, "fontIds") { mods.font_ids = Some(arr); }
    if let Some(arr) = json_u8_array(json, "ratios") { mods.ratios = Some(arr); }
    if let Some(arr) = json_i8_array(json, "spacings") { mods.spacings = Some(arr); }
    if let Some(arr) = json_u8_array(json, "relativeSizes") { mods.relative_sizes = Some(arr); }
    if let Some(arr) = json_i8_array(json, "charOffsets") { mods.char_offsets = Some(arr); }

    mods
}

/// JSON에서 [v0,v1,...,v6] 형태의 u8 배열 파싱 (7 요소)
pub(crate) fn json_u8_array(json: &str, key: &str) -> Option<[u8; 7]> {
    let pattern = format!("\"{}\":[", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let end = rest.find(']')?;
    let nums: Vec<u8> = rest[..end].split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() == 7 {
        Some([nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6]])
    } else {
        None
    }
}

/// JSON에서 [v0,v1,...,v6] 형태의 i8 배열 파싱 (7 요소)
pub(crate) fn json_i8_array(json: &str, key: &str) -> Option<[i8; 7]> {
    let pattern = format!("\"{}\":[", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let end = rest.find(']')?;
    let nums: Vec<i8> = rest[..end].split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() == 7 {
        Some([nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6]])
    } else {
        None
    }
}

/// JSON에서 [v0,v1,...,v6] 형태의 u16 배열 파싱 (7 요소)
pub(crate) fn json_u16_array(json: &str, key: &str) -> Option<[u16; 7]> {
    let pattern = format!("\"{}\":[", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let end = rest.find(']')?;
    let nums: Vec<u16> = rest[..end].split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if nums.len() == 7 {
        Some([nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6]])
    } else {
        None
    }
}

/// JSON에 border/fill 관련 키가 포함되어 있는지 확인한다.
pub(crate) fn json_has_border_keys(json: &str) -> bool {
    json.contains("\"borderLeft\"") || json.contains("\"borderRight\"")
        || json.contains("\"borderTop\"") || json.contains("\"borderBottom\"")
        || json.contains("\"fillType\"")
}

/// JSON에서 중첩 오브젝트를 문자열로 추출한다. (예: "borderLeft":{"type":1,"width":0,"color":"#000"})
pub(crate) fn json_object(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":{{", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len() - 1..]; // '{' 포함
    // 중괄호 매칭
    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if end > 0 { Some(rest[..end].to_string()) } else { None }
}

/// JSON 문자열에서 ParaShapeMods를 파싱한다.
pub(crate) fn parse_para_shape_mods(json: &str) -> crate::model::style::ParaShapeMods {
    use crate::model::style::{ParaShapeMods, Alignment, LineSpacingType, HeadType};
    let mut mods = ParaShapeMods::default();

    if let Some(v) = json_str(json, "alignment") {
        mods.alignment = Some(match v.as_str() {
            "left" => Alignment::Left,
            "right" => Alignment::Right,
            "center" => Alignment::Center,
            "justify" => Alignment::Justify,
            "distribute" => Alignment::Distribute,
            _ => Alignment::Justify,
        });
    }
    if let Some(v) = json_i32(json, "lineSpacing") { mods.line_spacing = Some(v); }
    if let Some(v) = json_str(json, "lineSpacingType") {
        mods.line_spacing_type = Some(match v.as_str() {
            "Fixed" => LineSpacingType::Fixed,
            "SpaceOnly" => LineSpacingType::SpaceOnly,
            "Minimum" => LineSpacingType::Minimum,
            _ => LineSpacingType::Percent,
        });
    }
    if let Some(v) = json_i32(json, "indent") { mods.indent = Some(v); }
    if let Some(v) = json_i32(json, "marginLeft") { mods.margin_left = Some(v); }
    if let Some(v) = json_i32(json, "marginRight") { mods.margin_right = Some(v); }
    if let Some(v) = json_i32(json, "spacingBefore") { mods.spacing_before = Some(v); }
    if let Some(v) = json_i32(json, "spacingAfter") { mods.spacing_after = Some(v); }
    // 확장 탭 속성
    if let Some(v) = json_str(json, "headType") {
        mods.head_type = Some(match v.as_str() {
            "Outline" => HeadType::Outline,
            "Number" => HeadType::Number,
            "Bullet" => HeadType::Bullet,
            _ => HeadType::None,
        });
    }
    if let Some(v) = json_i32(json, "paraLevel") { mods.para_level = Some(v as u8); }
    if let Some(v) = json_i32(json, "numberingId") { mods.numbering_id = Some(v as u16); }
    if let Some(v) = json_bool(json, "widowOrphan") { mods.widow_orphan = Some(v); }
    if let Some(v) = json_bool(json, "keepWithNext") { mods.keep_with_next = Some(v); }
    if let Some(v) = json_bool(json, "keepLines") { mods.keep_lines = Some(v); }
    if let Some(v) = json_bool(json, "pageBreakBefore") { mods.page_break_before = Some(v); }
    if let Some(v) = json_bool(json, "fontLineHeight") { mods.font_line_height = Some(v); }
    if let Some(v) = json_bool(json, "singleLine") { mods.single_line = Some(v); }
    if let Some(v) = json_bool(json, "autoSpaceKrEn") { mods.auto_space_kr_en = Some(v); }
    if let Some(v) = json_bool(json, "autoSpaceKrNum") { mods.auto_space_kr_num = Some(v); }
    if let Some(v) = json_i32(json, "verticalAlign") { mods.vertical_align = Some(v as u8); }
    if let Some(v) = json_i32(json, "englishBreakUnit") { mods.english_break_unit = Some(v as u8); }
    if let Some(v) = json_i32(json, "koreanBreakUnit") { mods.korean_break_unit = Some(v as u8); }

    mods
}

/// JSON에 탭 설정 관련 키가 포함되어 있는지 확인한다.
pub(crate) fn json_has_tab_keys(json: &str) -> bool {
    json.contains("\"tabStops\"") || json.contains("\"tabAutoLeft\"") || json.contains("\"tabAutoRight\"")
}

/// JSON에서 TabDef를 구성한다. 기존 TabDef를 기반으로 변경된 필드만 덮어쓴다.
pub(crate) fn build_tab_def_from_json(
    json: &str,
    base_tab_id: u16,
    tab_defs: &[crate::model::style::TabDef],
) -> crate::model::style::TabDef {
    use crate::model::style::TabDef;
    let base = tab_defs.get(base_tab_id as usize).cloned().unwrap_or_default();
    let auto_left = json_bool(json, "tabAutoLeft").unwrap_or(base.auto_tab_left);
    let auto_right = json_bool(json, "tabAutoRight").unwrap_or(base.auto_tab_right);
    let tabs = parse_tab_stops_json(json).unwrap_or(base.tabs);
    let attr = (if auto_left { 1u32 } else { 0 }) | (if auto_right { 2u32 } else { 0 });
    TabDef { raw_data: None, attr, tabs, auto_tab_left: auto_left, auto_tab_right: auto_right }
}

/// JSON "tabStops":[...] 배열에서 Vec<TabItem>을 파싱한다.
pub(crate) fn parse_tab_stops_json(json: &str) -> Option<Vec<crate::model::style::TabItem>> {
    use crate::model::style::TabItem;
    let key = "\"tabStops\":[";
    let start = json.find(key)?;
    let rest = &json[start + key.len()..];
    // ']' 까지의 내용을 추출 (중첩 대괄호 없으므로 단순 검색)
    let end = rest.find(']')?;
    let arr_str = &rest[..end];
    let mut tabs = Vec::new();
    let mut pos = 0;
    while pos < arr_str.len() {
        if let Some(obj_start) = arr_str[pos..].find('{') {
            let obj_rest = &arr_str[pos + obj_start..];
            if let Some(obj_end) = obj_rest.find('}') {
                let obj = &obj_rest[..=obj_end];
                let position = json_i32(obj, "position").unwrap_or(0) as u32;
                let tab_type = json_i32(obj, "type").unwrap_or(0) as u8;
                let fill_type = json_i32(obj, "fill").unwrap_or(0) as u8;
                tabs.push(TabItem { position, tab_type, fill_type });
                pos += obj_start + obj_end + 1;
            } else { break; }
        } else { break; }
    }
    Some(tabs)
}

/// JSON 배열에서 i16 값들을 파싱한다 (예: "borderSpacing":[0,0,0,0])
pub(crate) fn parse_json_i16_array(json: &str, key: &str, count: usize) -> Option<Vec<i16>> {
    let pattern = format!("\"{}\":[", key);
    let start = json.find(&pattern)?;
    let rest = &json[start + pattern.len()..];
    let end = rest.find(']')?;
    let arr_str = &rest[..end];
    let vals: Vec<i16> = arr_str.split(',')
        .filter_map(|s| s.trim().parse::<i16>().ok())
        .collect();
    if vals.len() == count { Some(vals) } else { None }
}

/// 간단한 JSON boolean 파싱
pub(crate) fn json_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let rest = rest.trim_start();
    if rest.starts_with("true") { Some(true) }
    else if rest.starts_with("false") { Some(false) }
    else { None }
}

/// 간단한 JSON i32 파싱
pub(crate) fn json_i32(json: &str, key: &str) -> Option<i32> {
    let pattern = format!("\"{}\":", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let rest = rest.trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// 간단한 JSON u16 파싱
pub(crate) fn json_u16(json: &str, key: &str) -> Option<u16> {
    json_i32(json, key).map(|v| v as u16)
}

/// 간단한 JSON 문자열 파싱 (이스케이프 시퀀스 디코딩 지원)
pub(crate) fn json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next() {
            None => return None,
            Some('"') => break,
            Some('\\') => match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(c) => { result.push('\\'); result.push(c); }
                None => return None,
            },
            Some(c) => result.push(c),
        }
    }
    Some(result)
}

/// CSS hex (#rrggbb) → HWP BGR (0x00BBGGRR) 변환
pub(crate) fn css_color_to_bgr(css: &str) -> Option<u32> {
    let hex = css.strip_prefix('#')?;
    if hex.len() != 6 { return None; }
    let r = u32::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u32::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u32::from_str_radix(&hex[4..6], 16).ok()?;
    Some(r | (g << 8) | (b << 16))
}

/// JSON에서 색상 값 파싱 (CSS hex → BGR)
pub(crate) fn json_color(json: &str, key: &str) -> Option<u32> {
    let css = json_str(json, key)?;
    css_color_to_bgr(&css)
}

/// 간단한 JSON u32 파싱
pub(crate) fn json_u32(json: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\":", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let rest = rest.trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// 간단한 JSON u8 파싱
pub(crate) fn json_u8(json: &str, key: &str) -> Option<u8> {
    json_u32(json, key).map(|v| v as u8)
}

/// 간단한 JSON i16 파싱
pub(crate) fn json_i16(json: &str, key: &str) -> Option<i16> {
    json_i32(json, key).map(|v| v as i16)
}

/// 간단한 JSON f64 파싱
pub(crate) fn json_f64(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let pos = json.find(&pattern)?;
    let rest = &json[pos + pattern.len()..];
    let num_str: String = rest.trim_start().chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    num_str.parse::<f64>().ok()
}

/// JSON 필수 필드 usize 파싱 (없으면 에러)
pub(crate) fn json_usize(json: &str, key: &str) -> Result<usize, HwpError> {
    let pattern = format!("\"{}\":", key);
    let pos = json.find(&pattern)
        .ok_or_else(|| HwpError::RenderError(format!("JSON 필드 '{}' 없음", key)))?;
    let rest = &json[pos + pattern.len()..];
    let num_str: String = rest.trim_start().chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    num_str.parse::<usize>()
        .map_err(|_| HwpError::RenderError(format!("JSON 필드 '{}' 값 파싱 실패", key)))
}

/// JSON 문자열 이스케이프
pub(crate) fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// JSON 성공 응답 생성: {"ok":true}
pub(crate) fn json_ok() -> String {
    r#"{"ok":true}"#.to_string()
}

/// JSON 성공 응답 생성: {"ok":true,...fields}
pub(crate) fn json_ok_with(fields: &str) -> String {
    format!("{{\"ok\":true,{}}}", fields)
}

/// HWP BGR 색상 (0x00BBGGRR)을 CSS hex (#RRGGBB)로 변환
/// 문자 위치 배열에서 x 좌표에 해당하는 문자 인덱스를 찾는다.
///
/// positions[i]는 i번째 문자의 왼쪽 끝 x좌표이다 (positions[0] = 0.0).
/// 각 문자의 중간점을 기준으로 좌/우를 판별한다.
pub(crate) fn find_char_at_x(positions: &[f64], x: f64) -> usize {
    if positions.len() <= 1 {
        return 0;
    }
    let char_count = positions.len() - 1;
    for i in 0..char_count {
        let mid = (positions[i] + positions[i + 1]) / 2.0;
        if x < mid {
            return i;
        }
    }
    char_count
}

pub(crate) fn color_ref_to_css(color: crate::model::ColorRef) -> String {
    let r = (color & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = ((color >> 16) & 0xFF) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

// === HTML 파싱 유틸리티 함수 ===

/// chars 배열에서 pos부터 target 문자를 찾아 인덱스를 반환한다.
pub(crate) fn find_char(chars: &[char], start: usize, target: char) -> usize {
    for i in start..chars.len() {
        if chars[i] == target { return i; }
    }
    chars.len()
}

/// HTML에서 닫는 태그의 다음 위치를 찾는다 (중첩 고려).
/// ASCII 대소문자 무시 바이트 비교
pub(crate) fn ascii_starts_with_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() { return false; }
    haystack.iter().zip(needle.iter())
        .all(|(h, n)| h.to_ascii_lowercase() == *n)
}

/// 바이트 인덱스 기반으로 닫는 태그를 찾는다 (parse_table_html 등 바이트 기반 파서 용).
/// start_pos는 바이트 인덱스이며, 반환값도 바이트 인덱스이다.
pub(crate) fn find_closing_tag(html: &str, start_pos: usize, tag_name: &str) -> usize {
    let bytes = html.as_bytes();
    let open_tag = format!("<{}", tag_name).to_lowercase().into_bytes();
    let close_tag = format!("</{}>", tag_name).to_lowercase().into_bytes();
    let len = bytes.len();

    let mut depth = 0;
    let mut pos = start_pos;

    while pos < len {
        if bytes[pos] == b'<' {
            // 닫는 태그 확인
            if ascii_starts_with_ci(&bytes[pos..], &close_tag) {
                depth -= 1;
                if depth <= 0 {
                    return pos + close_tag.len();
                }
                pos += close_tag.len();
                continue;
            }
            // 여는 태그 확인
            if ascii_starts_with_ci(&bytes[pos..], &open_tag) {
                depth += 1;
                pos += open_tag.len();
                continue;
            }
        }
        pos += 1;
    }

    len
}

/// char 인덱스 기반으로 닫는 태그를 찾는다 (parse_html_to_paragraphs 등 char 배열 기반 파서 용).
pub(crate) fn find_closing_tag_chars(chars: &[char], start_pos: usize, tag_name: &str) -> usize {
    let open_tag: Vec<char> = format!("<{}", tag_name).to_lowercase().chars().collect();
    let close_tag: Vec<char> = format!("</{}>", tag_name).to_lowercase().chars().collect();
    let len = chars.len();

    let mut depth = 0;
    let mut pos = start_pos;

    while pos < len {
        if chars[pos].to_lowercase().next() == Some('<') {
            // 닫는 태그 확인
            if pos + close_tag.len() <= len {
                let slice: String = chars[pos..pos + close_tag.len()].iter().collect();
                if slice.to_lowercase() == close_tag.iter().collect::<String>() {
                    depth -= 1;
                    if depth <= 0 {
                        return pos + close_tag.len();
                    }
                    pos += close_tag.len();
                    continue;
                }
            }
            // 여는 태그 확인
            if pos + open_tag.len() <= len {
                let slice: String = chars[pos..pos + open_tag.len()].iter().collect();
                if slice.to_lowercase() == open_tag.iter().collect::<String>() {
                    depth += 1;
                    pos += open_tag.len();
                    continue;
                }
            }
        }
        pos += 1;
    }

    len
}

/// HTML 태그의 style 속성에서 인라인 스타일 문자열을 추출한다.
pub(crate) fn parse_inline_style(tag: &str) -> String {
    let tag_lower = tag.to_lowercase();
    if let Some(style_start) = tag_lower.find("style=\"") {
        let after = &tag[style_start + 7..];
        if let Some(end) = after.find('"') {
            return after[..end].to_string();
        }
    }
    if let Some(style_start) = tag_lower.find("style='") {
        let after = &tag[style_start + 7..];
        if let Some(end) = after.find('\'') {
            return after[..end].to_string();
        }
    }
    String::new()
}

/// CSS 인라인 스타일에서 특정 속성의 값을 추출한다.
pub(crate) fn parse_css_value<'a>(css: &'a str, property: &str) -> Option<String> {
    let css = css.trim();
    // "property:" 또는 "property :" 패턴 검색
    for part in css.split(';') {
        let part = part.trim();
        if let Some(colon) = part.find(':') {
            let key = part[..colon].trim();
            if key == property {
                return Some(part[colon + 1..].trim().to_string());
            }
        }
    }
    None
}

/// pt/px 값 파싱 (예: "10.0pt", "12px", "14")
pub(crate) fn parse_pt_value(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with("pt") {
        s.trim_end_matches("pt").trim().parse().ok()
    } else if s.ends_with("px") {
        // px → pt (1px = 0.75pt at 96dpi)
        let px: f64 = s.trim_end_matches("px").trim().parse().ok()?;
        Some(px * 0.75)
    } else if s.ends_with("em") {
        // em → pt (1em ≈ 12pt 기본)
        let em: f64 = s.trim_end_matches("em").trim().parse().ok()?;
        Some(em * 12.0)
    } else {
        // 단위 없는 숫자 (pt로 간주)
        s.parse().ok()
    }
}

/// CSS 색상 문자열을 HWP BGR (0x00BBGGRR)로 변환한다.
pub(crate) fn css_color_to_hwp_bgr(css: &str) -> Option<u32> {
    let css = css.trim();
    if css.starts_with('#') {
        let hex = &css[1..];
        if hex.len() == 6 {
            let r = u32::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u32::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u32::from_str_radix(&hex[4..6], 16).ok()?;
            Some(r | (g << 8) | (b << 16))
        } else if hex.len() == 3 {
            let r = u32::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u32::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u32::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(r | (g << 8) | (b << 16))
        } else {
            None
        }
    } else if css.starts_with("rgb(") || css.starts_with("rgb (") {
        // rgb(r, g, b) 형식
        let inner = css.trim_start_matches("rgb").trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r: u32 = parts[0].trim().parse().ok()?;
            let g: u32 = parts[1].trim().parse().ok()?;
            let b: u32 = parts[2].trim().parse().ok()?;
            Some(r | (g << 8) | (b << 16))
        } else {
            None
        }
    } else {
        // 색상 이름 (기본적인 것만)
        match css.trim() {
            "black" => Some(0x000000),
            "white" => Some(0xFFFFFF),
            "red" => Some(0x0000FF),
            "green" => Some(0x008000),
            "blue" => Some(0xFF0000),
            "yellow" => Some(0x00FFFF),
            _ => None,
        }
    }
}

/// HTML 엔티티를 디코딩한다.
pub(crate) fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&#xA0;", " ")
}

/// HTML 태그를 제거하고 텍스트만 추출한다.
pub(crate) fn html_strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' { in_tag = true; continue; }
        if c == '>' { in_tag = false; continue; }
        if !in_tag { result.push(c); }
    }
    result
}

/// HTML을 플레인 텍스트로 변환한다 (태그 제거 + 엔티티 디코딩).
pub(crate) fn html_to_plain_text(html: &str) -> String {
    decode_html_entities(&html_strip_tags(html)).trim().to_string()
}

/// HTML 태그에서 숫자 속성값을 추출한다.
pub(crate) fn parse_html_attr_f64(tag: &str, attr: &str) -> Option<f64> {
    // width="200" 또는 width='200' 형식
    let patterns = [
        format!("{}=\"", attr),
        format!("{}='", attr),
    ];
    for pat in &patterns {
        if let Some(start) = tag.to_lowercase().find(&pat.to_lowercase()) {
            let after = &tag[start + pat.len()..];
            let delim = if pat.ends_with('"') { '"' } else { '\'' };
            if let Some(end) = after.find(delim) {
                let val_str = &after[..end];
                // "200px" → 200.0, "200" → 200.0
                let num_str = val_str.trim_end_matches("px").trim();
                return num_str.parse().ok();
            }
        }
    }
    None
}

/// HTML 태그에서 정수 속성값을 추출한다 (colspan="3" 등).
pub(crate) fn parse_html_attr_u16(tag: &str, attr: &str) -> Option<u16> {
    parse_html_attr_f64(tag, attr).map(|v| v as u16)
}

/// CSS dimension 값을 pt로 파싱한다 (width, height 등).
/// "38.50pt" → 38.5, "100px" → 75.0, "2cm" → 56.69
pub(crate) fn parse_css_dimension_pt(css: &str, property: &str) -> f64 {
    if let Some(val) = parse_css_value(css, property) {
        let val = val.trim();
        if val.ends_with("pt") {
            val.trim_end_matches("pt").trim().parse::<f64>().unwrap_or(0.0)
        } else if val.ends_with("px") {
            val.trim_end_matches("px").trim().parse::<f64>().unwrap_or(0.0) * 0.75
        } else if val.ends_with("cm") {
            val.trim_end_matches("cm").trim().parse::<f64>().unwrap_or(0.0) * 28.3465
        } else if val.ends_with("mm") {
            val.trim_end_matches("mm").trim().parse::<f64>().unwrap_or(0.0) * 2.83465
        } else if val.ends_with("in") {
            val.trim_end_matches("in").trim().parse::<f64>().unwrap_or(0.0) * 72.0
        } else if val.ends_with('%') {
            0.0 // 백분율은 무시
        } else {
            // 단위 없는 숫자 → pt로 간주
            val.parse::<f64>().unwrap_or(0.0)
        }
    } else {
        0.0
    }
}

/// CSS padding 축약형/개별 값을 파싱하여 [left, right, top, bottom] (pt)로 반환한다.
pub(crate) fn parse_css_padding_pt(css: &str) -> [f64; 4] {
    let mut result = [0.0f64; 4]; // left, right, top, bottom

    // 축약형 padding: "1.41pt 5.10pt" 또는 "5pt" 또는 "5pt 10pt 5pt 10pt"
    if let Some(val) = parse_css_value(css, "padding") {
        let parts: Vec<f64> = val.split_whitespace()
            .map(|p| parse_single_dimension_pt(p))
            .collect();
        match parts.len() {
            1 => { result = [parts[0]; 4]; },
            2 => {
                // top/bottom, left/right
                result = [parts[1], parts[1], parts[0], parts[0]];
            },
            3 => {
                // top, left/right, bottom
                result = [parts[1], parts[1], parts[0], parts[2]];
            },
            4 => {
                // top, right, bottom, left
                result = [parts[3], parts[1], parts[0], parts[2]];
            },
            _ => {},
        }
    }

    // 개별 방향 오버라이드
    if let Some(v) = parse_css_value(css, "padding-left") {
        result[0] = parse_single_dimension_pt(&v);
    }
    if let Some(v) = parse_css_value(css, "padding-right") {
        result[1] = parse_single_dimension_pt(&v);
    }
    if let Some(v) = parse_css_value(css, "padding-top") {
        result[2] = parse_single_dimension_pt(&v);
    }
    if let Some(v) = parse_css_value(css, "padding-bottom") {
        result[3] = parse_single_dimension_pt(&v);
    }

    result
}

/// 단일 CSS 치수 값을 pt로 변환한다.
pub(crate) fn parse_single_dimension_pt(s: &str) -> f64 {
    let s = s.trim();
    if s.ends_with("pt") {
        s.trim_end_matches("pt").trim().parse::<f64>().unwrap_or(0.0)
    } else if s.ends_with("px") {
        s.trim_end_matches("px").trim().parse::<f64>().unwrap_or(0.0) * 0.75
    } else if s.ends_with("cm") {
        s.trim_end_matches("cm").trim().parse::<f64>().unwrap_or(0.0) * 28.3465
    } else if s.ends_with("mm") {
        s.trim_end_matches("mm").trim().parse::<f64>().unwrap_or(0.0) * 2.83465
    } else if s.ends_with("in") {
        s.trim_end_matches("in").trim().parse::<f64>().unwrap_or(0.0) * 72.0
    } else {
        s.parse::<f64>().unwrap_or(0.0)
    }
}

/// CSS border 축약형 ("solid #000000 0.28pt" 등)을 파싱한다.
/// 반환값: (width_pt, color_bgr, style: 0=none,1=solid,2=dashed,3=dotted,4=double)
pub(crate) fn parse_css_border_shorthand(val: &str) -> (f64, u32, u8) {
    let val = val.trim();
    if val == "none" || val == "0" || val.is_empty() {
        return (0.0, 0, 0);
    }

    let parts: Vec<&str> = val.split_whitespace().collect();
    let mut width_pt = 0.0f64;
    let mut color: u32 = 0; // black
    let mut style: u8 = 1; // solid

    for part in &parts {
        let p = part.trim();
        // 스타일 키워드
        match p {
            "solid" => { style = 1; continue; },
            "dashed" => { style = 2; continue; },
            "dotted" => { style = 3; continue; },
            "double" => { style = 4; continue; },
            "none" => { style = 0; continue; },
            "hidden" => { style = 0; continue; },
            _ => {},
        }
        // 색상 (#hex 또는 rgb())
        if p.starts_with('#') || p.starts_with("rgb") {
            if let Some(c) = css_color_to_hwp_bgr(p) {
                color = c;
            }
            continue;
        }
        // 치수 값
        let dim = parse_single_dimension_pt(p);
        if dim > 0.0 {
            width_pt = dim;
        }
    }

    (width_pt, color, style)
}

/// CSS border 두께(pt)를 HWP border width 인덱스로 변환한다.
/// HWP 스펙: width 값이 선 굵기 인덱스 (0: 0.1mm, 1: 0.12mm, 2: 0.15mm, 3: 0.2mm, 4: 0.25mm, 5: 0.3mm, 6: 0.4mm, 7: 0.5mm)
pub(crate) fn css_border_width_to_hwp(pt: f64) -> u8 {
    let mm = pt * 0.3528; // 1pt ≈ 0.3528mm
    if mm < 0.11 { 0 }
    else if mm < 0.14 { 1 }
    else if mm < 0.18 { 2 }
    else if mm < 0.23 { 3 }
    else if mm < 0.28 { 4 }
    else if mm < 0.35 { 5 }
    else if mm < 0.45 { 6 }
    else { 7 }
}

/// BorderLineType을 u8 값으로 변환한다.
pub(crate) fn border_line_type_to_u8_val(lt: crate::model::style::BorderLineType) -> u8 {
    use crate::model::style::BorderLineType;
    match lt {
        BorderLineType::None => 0, BorderLineType::Solid => 1,
        BorderLineType::Dash => 2, BorderLineType::Dot => 3,
        BorderLineType::DashDot => 4, BorderLineType::DashDotDot => 5,
        BorderLineType::LongDash => 6, BorderLineType::Circle => 7,
        BorderLineType::Double => 8, BorderLineType::ThinThickDouble => 9,
        BorderLineType::ThickThinDouble => 10, BorderLineType::ThinThickThinTriple => 11,
        BorderLineType::Wave => 12, BorderLineType::DoubleWave => 13,
        BorderLineType::Thick3D => 14, BorderLineType::Thick3DReverse => 15,
        BorderLineType::Thin3D => 16, BorderLineType::Thin3DReverse => 17,
    }
}

/// u8 값을 BorderLineType으로 변환한다.
pub(crate) fn u8_to_border_line_type(v: u8) -> crate::model::style::BorderLineType {
    use crate::model::style::BorderLineType;
    match v {
        0 => BorderLineType::None, 1 => BorderLineType::Solid,
        2 => BorderLineType::Dash, 3 => BorderLineType::Dot,
        4 => BorderLineType::DashDot, 5 => BorderLineType::DashDotDot,
        6 => BorderLineType::LongDash, 7 => BorderLineType::Circle,
        8 => BorderLineType::Double, 9 => BorderLineType::ThinThickDouble,
        10 => BorderLineType::ThickThinDouble, 11 => BorderLineType::ThinThickThinTriple,
        12 => BorderLineType::Wave, 13 => BorderLineType::DoubleWave,
        14 => BorderLineType::Thick3D, 15 => BorderLineType::Thick3DReverse,
        16 => BorderLineType::Thin3D, 17 => BorderLineType::Thin3DReverse,
        _ => BorderLineType::None,
    }
}

/// 두 BorderFill이 동일한지 비교한다.
pub(crate) fn border_fills_equal(a: &crate::model::style::BorderFill, b: &crate::model::style::BorderFill) -> bool {
    if a.attr != b.attr { return false; }
    for i in 0..4 {
        if a.borders[i].line_type != b.borders[i].line_type { return false; }
        if a.borders[i].width != b.borders[i].width { return false; }
        if a.borders[i].color != b.borders[i].color { return false; }
    }
    // fill 비교 (fill_type + solid color)
    if a.fill.fill_type != b.fill.fill_type { return false; }
    match (&a.fill.solid, &b.fill.solid) {
        (Some(sa), Some(sb)) => sa.background_color == sb.background_color,
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::footnote::Footnote;

    #[test]
    fn navigable_text_len_counts_trailing_footnote_marker() {
        let para = Paragraph {
            text: "abc".to_string(),
            char_offsets: vec![0, 1, 2],
            controls: vec![Control::Footnote(Box::new(Footnote::default()))],
            ..Default::default()
        };

        assert_eq!(find_control_text_positions(&para), vec![3]);
        assert_eq!(navigable_text_len(&para), 4);
    }
}
