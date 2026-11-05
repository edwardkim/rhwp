//! 각주 모양 조회·적용 (주석 모양 대화상자의 각주 모양 탭).
//!
//! 미주 모양 API와 같은 필드를 주고받는다. 각주만의 것은 번호 매기기의 "쪽마다 새로 시작"과
//! 단 기준 배치(각 단 아래·전단·오른쪽 단 아래)다. 한/글 12.30은 각주 모양을 바꾸면 이미 있는
//! 각주의 번호 모양·장식 문자·내용 번호 위첨자(작게)를 함께 바꾸고, 새 각주도 그 모양을 따른다.
//! 번호는 각주마다 저장하며, 쪽마다 새로 시작이면 쪽마다 1부터 센 번호를 저장한다
//! (같은 쪽 두 각주와 다음 쪽 각주가 1·2·1).

use serde::Deserialize;

use crate::document_core::helpers::{clipboard_color_to_css, css_color_to_bgr, json_ok};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{AutoNumberType, Control};
use crate::model::footnote::{
    Footnote, FootnoteNumbering, FootnotePlacement, FootnoteShape, NumberFormat,
};
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::FootnoteSource;

/// `applyFootnoteShape` 입력. 빠진 필드는 그대로 둔다.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FootnoteShapeProps {
    number_format: Option<String>,
    user_char: Option<String>,
    prefix_char: Option<String>,
    suffix_char: Option<String>,
    start_number: Option<u16>,
    separator_enabled: Option<bool>,
    separator_length: Option<i32>,
    separator_margin_top: Option<i16>,
    separator_margin_bottom: Option<i16>,
    note_spacing: Option<u16>,
    separator_line_type: Option<u8>,
    separator_line_width: Option<u8>,
    separator_color: Option<String>,
    number_code_superscript: Option<bool>,
    print_inline_after_text: Option<bool>,
    numbering: Option<String>,
    placement: Option<String>,
}

fn invalid(message: String) -> HwpError {
    HwpError::RenderError(message)
}

fn placement_name(placement: FootnotePlacement) -> &'static str {
    match placement {
        FootnotePlacement::EachColumn => "eachColumn",
        FootnotePlacement::BelowText => "mergedColumn",
        FootnotePlacement::RightColumn => "rightColumn",
    }
}

fn placement_from_name(value: &str) -> Option<FootnotePlacement> {
    match value {
        "eachColumn" => Some(FootnotePlacement::EachColumn),
        "mergedColumn" => Some(FootnotePlacement::BelowText),
        "rightColumn" => Some(FootnotePlacement::RightColumn),
        _ => None,
    }
}

fn numbering_from_name(value: &str) -> Option<FootnoteNumbering> {
    match value {
        "continue" => Some(FootnoteNumbering::Continue),
        "restartSection" => Some(FootnoteNumbering::RestartSection),
        "restartPage" => Some(FootnoteNumbering::RestartPage),
        _ => None,
    }
}

fn note_char(value: &str) -> char {
    value.chars().next().unwrap_or('\0')
}

fn number_format_from_name(value: &str) -> Option<NumberFormat> {
    // 모르는 이름은 넘긴 기본값을 돌려주므로, 기본값을 바꿔도 같은 값일 때만 아는 이름이다.
    let digit = FootnoteShape::number_format_from_name(value, NumberFormat::Digit);
    (digit == FootnoteShape::number_format_from_name(value, NumberFormat::UserChar))
        .then_some(digit)
}

/// 실제로 쓰는 뒤 장식 문자. 원본이 정하지 않은 빈 값은 한/글 기본값 `)`다(HWPX 저장과 같다).
fn effective_suffix(shape: &FootnoteShape) -> char {
    if shape.suffix_char == '\0'
        && !shape.deco_chars_from_source
        && shape.number_format != NumberFormat::UserChar
    {
        ')'
    } else {
        shape.suffix_char
    }
}

/// 입력을 모두 확인한 뒤에만 모양을 바꾼다.
fn apply_props(shape: &mut FootnoteShape, props: FootnoteShapeProps) -> Result<(), HwpError> {
    let mut next = shape.clone();
    if let Some(v) = props.number_format {
        next.number_format = number_format_from_name(&v)
            .ok_or_else(|| invalid(format!("알 수 없는 번호 모양: {v}")))?;
    }
    // 장식 문자를 정하면 빈 값도 그대로 저장하도록 원본 값으로 둔다.
    if props.user_char.is_some() || props.prefix_char.is_some() || props.suffix_char.is_some() {
        next.suffix_char = effective_suffix(&next);
        next.deco_chars_from_source = true;
    }
    if let Some(v) = props.user_char {
        next.user_char = note_char(&v);
    }
    if let Some(v) = props.prefix_char {
        next.prefix_char = note_char(&v);
    }
    if let Some(v) = props.suffix_char {
        next.suffix_char = note_char(&v);
    }
    if let Some(v) = props.start_number {
        next.start_number = v.max(1);
    }
    if let Some(v) = props.separator_length {
        // -1~-4는 한/글의 길이 선택(5 cm·2 cm·단 너비의 1/3·단 너비)이다. 한/글 기본 각주 구분선이
        // -1(5 cm)이라 받은 값을 지킨다.
        next.separator_length = if v >= -4 { v } else { 0 };
    }
    if let Some(v) = props.separator_margin_top {
        // HWP5 저장본은 구분선 위 값을 두 번째 슬롯에 두기도 해 둘 다 바꾼다(미주 모양과 같다).
        next.separator_margin_top = v.max(0);
        next.separator_margin_bottom = v.max(0);
    }
    if let Some(v) = props.separator_margin_bottom {
        next.note_spacing = v.max(0);
    }
    if let Some(v) = props.note_spacing {
        next.raw_unknown = v;
    }
    if let Some(v) = props.separator_line_type {
        next.separator_line_type = v;
    }
    if let Some(v) = props.separator_line_width {
        next.separator_line_width = v;
    }
    if let Some(v) = props.separator_color {
        next.separator_color =
            css_color_to_bgr(&v).ok_or_else(|| invalid(format!("알 수 없는 색: {v}")))?;
    }
    if let Some(v) = props.number_code_superscript {
        next.number_code_superscript = v;
    }
    if let Some(v) = props.print_inline_after_text {
        next.print_inline_after_text = v;
    }
    if let Some(v) = props.numbering {
        next.numbering = numbering_from_name(&v)
            .ok_or_else(|| invalid(format!("알 수 없는 번호 매기기: {v}")))?;
    }
    if let Some(v) = props.placement {
        next.placement =
            placement_from_name(&v).ok_or_else(|| invalid(format!("알 수 없는 각주 위치: {v}")))?;
    }
    if props.separator_enabled == Some(false) {
        next.separator_length = 0;
        next.separator_line_type = 0;
        next.separator_line_width = 0;
    }
    next.attr = next.encode_attr();
    *shape = next;
    Ok(())
}

/// 문단들의 각주를 문서 순서(본문 → 표 셀·중첩 표 → 글상자)로 돈다.
fn for_each_footnote(paragraphs: &mut [Paragraph], f: &mut dyn FnMut(&mut Footnote)) {
    for para in paragraphs {
        for control in &mut para.controls {
            match control {
                Control::Footnote(note) => f(note),
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        for_each_footnote(&mut cell.paragraphs, f);
                    }
                }
                Control::Shape(shape) => {
                    if let Some(text_box) = shape.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                        for_each_footnote(&mut text_box.paragraphs, f);
                    }
                }
                _ => {}
            }
        }
    }
}

/// 쪽 배정이 가리키는 각주를 찾는다.
fn footnote_at<'a>(
    paragraphs: &'a mut [Paragraph],
    source: &FootnoteSource,
) -> Option<&'a mut Footnote> {
    fn note(control: &mut Control) -> Option<&mut Footnote> {
        match control {
            Control::Footnote(note) => Some(note),
            _ => None,
        }
    }
    match source {
        FootnoteSource::Body {
            para_index,
            control_index,
        } => note(
            paragraphs
                .get_mut(*para_index)?
                .controls
                .get_mut(*control_index)?,
        ),
        FootnoteSource::TableCell {
            para_index,
            table_control_index,
            cell_index,
            cell_para_index,
            cell_control_index,
        } => {
            let Control::Table(table) = paragraphs
                .get_mut(*para_index)?
                .controls
                .get_mut(*table_control_index)?
            else {
                return None;
            };
            note(
                table
                    .cells
                    .get_mut(*cell_index)?
                    .paragraphs
                    .get_mut(*cell_para_index)?
                    .controls
                    .get_mut(*cell_control_index)?,
            )
        }
        FootnoteSource::NestedTableCell {
            para_index,
            table_control_index,
            cell_path,
        } => {
            let Control::Table(table) = paragraphs
                .get_mut(*para_index)?
                .controls
                .get_mut(*table_control_index)?
            else {
                return None;
            };
            let mut table = &mut **table;
            let (last, steps) = cell_path.split_last()?;
            for step in steps {
                let Control::Table(inner) = table
                    .cells
                    .get_mut(step.cell_index)?
                    .paragraphs
                    .get_mut(step.cell_para_index)?
                    .controls
                    .get_mut(step.control_index)?
                else {
                    return None;
                };
                table = &mut **inner;
            }
            note(
                table
                    .cells
                    .get_mut(last.cell_index)?
                    .paragraphs
                    .get_mut(last.cell_para_index)?
                    .controls
                    .get_mut(last.control_index)?,
            )
        }
        FootnoteSource::ShapeTextBox {
            para_index,
            shape_control_index,
            tb_para_index,
            tb_control_index,
        } => {
            let Control::Shape(shape) = paragraphs
                .get_mut(*para_index)?
                .controls
                .get_mut(*shape_control_index)?
            else {
                return None;
            };
            let text_box = shape.drawing_mut()?.text_box.as_mut()?;
            note(
                text_box
                    .paragraphs
                    .get_mut(*tb_para_index)?
                    .controls
                    .get_mut(*tb_control_index)?,
            )
        }
    }
}

/// 각주 하나의 번호·모양을 맞춘다. 바뀐 것이 있으면 true.
fn sync_footnote(note: &mut Footnote, shape: &FootnoteShape, number: Option<u16>) -> bool {
    let before = (
        note.number,
        note.number_shape,
        note.before_decoration_letter,
        note.after_decoration_letter,
    );
    if let Some(number) = number {
        note.number = number;
    }
    note.number_shape = FootnoteShape::number_format_attr_code(shape.number_format);
    let suffix = effective_suffix(shape);
    note.before_decoration_letter = shape.prefix_char as u16;
    note.after_decoration_letter = suffix as u16;
    let mut changed = before
        != (
            note.number,
            note.number_shape,
            note.before_decoration_letter,
            note.after_decoration_letter,
        );
    let format = note.number_shape as u8;
    for para in &mut note.paragraphs {
        for control in &mut para.controls {
            if let Control::AutoNumber(auto) = control {
                if auto.number_type != AutoNumberType::Footnote {
                    continue;
                }
                let next = (
                    format,
                    shape.prefix_char,
                    suffix,
                    shape.user_char,
                    shape.number_code_superscript,
                    note.number,
                );
                if (
                    auto.format,
                    auto.prefix_char,
                    auto.suffix_char,
                    auto.user_symbol,
                    auto.superscript,
                    auto.assigned_number,
                ) != next
                {
                    (
                        auto.format,
                        auto.prefix_char,
                        auto.suffix_char,
                        auto.user_symbol,
                        auto.superscript,
                        auto.number,
                    ) = next;
                    auto.assigned_number = note.number;
                    changed = true;
                }
            }
        }
    }
    changed
}

impl DocumentCore {
    /// 구역의 각주 모양을 조회한다. 필드는 미주 모양(`getEndnoteShape`)과 같고, `numbering`은
    /// continue·restartSection·restartPage, `placement`는 eachColumn·mergedColumn·rightColumn이다.
    pub fn get_footnote_shape_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| invalid(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        let shape = &section.section_def.footnote_shape;
        let text = |ch: char| {
            if ch == '\0' {
                String::new()
            } else {
                ch.to_string()
            }
        };
        let numbering = match shape.numbering {
            FootnoteNumbering::Continue => "continue",
            FootnoteNumbering::RestartSection => "restartSection",
            FootnoteNumbering::RestartPage => "restartPage",
        };
        Ok(serde_json::json!({
            "ok": true,
            "numberFormat": Self::footnote_shape_number_format_name(shape.number_format),
            "userChar": text(shape.user_char),
            "prefixChar": text(shape.prefix_char),
            "suffixChar": text(effective_suffix(shape)),
            "startNumber": shape.start_number,
            "separatorEnabled": shape.separator_length != 0
                || shape.separator_line_type != 0
                || shape.separator_line_width != 0,
            "separatorLength": shape.separator_length,
            "separatorMarginTop": shape.separator_above_margin_hu(),
            "separatorMarginBottom": shape.separator_below_margin_hu(),
            "noteSpacing": shape.between_notes_margin_hu(),
            "separatorLineType": shape.separator_line_type,
            "separatorLineWidth": shape.separator_line_width,
            "separatorColor": clipboard_color_to_css(shape.separator_color),
            "numberCodeSuperscript": shape.number_code_superscript,
            "printInlineAfterText": shape.print_inline_after_text,
            "numbering": numbering,
            "placement": placement_name(shape.placement),
        })
        .to_string())
    }

    /// 구역의 각주 모양을 바꾸고, 그 구역(과 이어 세는 뒤 구역)의 각주 번호·모양을 맞춘다.
    pub fn apply_footnote_shape_native(
        &mut self,
        section_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let props: FootnoteShapeProps = serde_json::from_str(props_json)
            .map_err(|e| invalid(format!("각주 모양 JSON 오류: {e}")))?;
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| invalid(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        apply_props(&mut section.section_def.footnote_shape, props)?;
        section.raw_stream = None;
        self.sync_section_def_controls_from_section(section_idx);
        self.recompose_section(section_idx);
        self.refresh_footnote_numbers(section_idx);
        Ok(json_ok())
    }

    /// `from_section`부터 각주 번호와 모양을 각 구역의 각주 모양대로 맞춘다. 앞 구역에 이어
    /// 세는 뒤 구역까지 따라가고, 바뀐 구역은 다시 조판한다. 쪽마다 새로 시작하는 구역은 쪽
    /// 배정을 마친 뒤 번호를 매기고, 번호 폭이 쪽 배정을 바꿀 수 있어 안정될 때까지(최대 3번)
    /// 되풀이한다.
    pub(crate) fn refresh_footnote_numbers(&mut self, from_section: usize) {
        let mut carry = self.document.sections[..from_section]
            .iter_mut()
            .rev()
            .find_map(|section| {
                let mut last = None;
                for_each_footnote(&mut section.paragraphs, &mut |note| {
                    last = Some(note.number)
                });
                last
            })
            .unwrap_or(self.document.doc_properties.footnote_start_num.max(1) - 1);
        let mut per_page = Vec::new();
        for index in from_section..self.document.sections.len() {
            let section = &mut self.document.sections[index];
            let shape = section.section_def.footnote_shape.clone();
            if index > from_section && shape.numbering != FootnoteNumbering::Continue {
                break;
            }
            let mut next = match shape.numbering {
                FootnoteNumbering::Continue => carry.saturating_add(1),
                FootnoteNumbering::RestartSection => shape.start_number.max(1),
                // 쪽 배정 뒤 다시 센다. 지금은 모양만 맞춘다.
                FootnoteNumbering::RestartPage => 0,
            };
            let mut changed = false;
            for_each_footnote(&mut section.paragraphs, &mut |note| {
                let number = (next > 0).then_some(next);
                changed |= sync_footnote(note, &shape, number);
                if next > 0 {
                    next = next.saturating_add(1);
                }
                carry = note.number;
            });
            if shape.numbering == FootnoteNumbering::RestartPage {
                per_page.push(index);
            }
            if changed {
                section.raw_stream = None;
                self.recompose_section(index);
            }
        }
        self.paginate_if_needed();
        if !self.batch_mode {
            for index in per_page {
                for _ in 0..3 {
                    if !self.number_footnotes_per_page(index) {
                        break;
                    }
                    self.recompose_section(index);
                    self.paginate();
                }
            }
        }
        self.invalidate_page_tree_cache();
    }

    /// 쪽마다 각주를 1부터 센다. 번호가 바뀐 각주가 있으면 true.
    fn number_footnotes_per_page(&mut self, section_idx: usize) -> bool {
        let Some(pagination) = self.pagination.get(section_idx) else {
            return false;
        };
        let mut numbers = Vec::new();
        for page in &pagination.pages {
            let mut number = 0u16;
            for note in &page.footnotes {
                // 쪽을 넘어 이어지는 각주 조각은 번호를 다시 세지 않는다.
                if note.fragment.is_some_and(|fragment| !fragment.draw_number) {
                    continue;
                }
                number = number.saturating_add(1);
                numbers.push((note.source.clone(), number));
            }
        }
        let section = &mut self.document.sections[section_idx];
        let shape = section.section_def.footnote_shape.clone();
        let mut changed = false;
        for (source, number) in numbers {
            if let Some(note) = footnote_at(&mut section.paragraphs, &source) {
                changed |= sync_footnote(note, &shape, Some(number));
            }
        }
        if changed {
            section.raw_stream = None;
        }
        changed
    }
}
