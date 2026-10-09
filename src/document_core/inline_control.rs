//! 글자처럼 취급하는 개체의 슬롯과 문단 소유권을 함께 옮긴다.

use super::{header_footer_apply_from_u8, helpers::get_textbox_from_shape_mut, DocumentCore};
use crate::{
    error::HwpError,
    model::{
        control::Control,
        document::Section,
        paragraph::{CharShapeRef, MarkpenMark, Paragraph, RangeTag},
    },
    serializer::body_text::{control_stream_slots, field_stream_ends},
};
use serde::{Deserialize, Serialize};

/// 기존 편집 API의 본문/셀, 머리말·꼬리말, 각주·미주 주소.
/// cellPath의 각 항목은 (컨트롤, 셀, 내부 문단)이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum InlineControlOwner {
    Body {
        section_index: usize,
        paragraph_index: usize,
        #[serde(default)]
        cell_path: Vec<(usize, usize, usize)>,
    },
    HeaderFooter {
        section_index: usize,
        is_header: bool,
        apply_to: u8,
        paragraph_index: usize,
        #[serde(default)]
        cell_path: Vec<(usize, usize, usize)>,
    },
    Note {
        section_index: usize,
        parent_paragraph_index: usize,
        note_control_index: usize,
        paragraph_index: usize,
        #[serde(default)]
        cell_path: Vec<(usize, usize, usize)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InlineControlAddress {
    pub owner: InlineControlOwner,
    pub control_index: usize,
}

/// charOffset은 Unicode scalar와 인라인 개체를 각각 한 칸으로 센 캐럿 위치다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InlineControlCaret {
    pub owner: InlineControlOwner,
    pub char_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineControlMoveResult {
    pub changed: bool,
    pub address: InlineControlAddress,
    pub char_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Address {
    section: usize,
    root: usize,
    path: Vec<(usize, usize, usize)>,
}

fn invalid(message: &str) -> HwpError {
    HwpError::RenderError(message.into())
}

impl InlineControlOwner {
    fn resolve(&self, core: &DocumentCore) -> Result<Address, HwpError> {
        let (section, root, mut path, tail) = match self {
            Self::Body {
                section_index,
                paragraph_index,
                cell_path,
            } => (*section_index, *paragraph_index, vec![], cell_path),
            Self::HeaderFooter {
                section_index,
                is_header,
                apply_to,
                paragraph_index,
                cell_path,
            } => {
                if *apply_to > 2 {
                    return Err(invalid("머리말·꼬리말 적용 범위가 잘못되었습니다"));
                }
                let (root, ctrl) = core
                    .find_header_footer_control(
                        *section_index,
                        *is_header,
                        header_footer_apply_from_u8(*apply_to),
                    )
                    .ok_or_else(|| invalid("머리말·꼬리말을 찾을 수 없습니다"))?;
                (
                    *section_index,
                    root,
                    vec![(ctrl, 0, *paragraph_index)],
                    cell_path,
                )
            }
            Self::Note {
                section_index,
                parent_paragraph_index,
                note_control_index,
                paragraph_index,
                cell_path,
            } => {
                let control = core
                    .document
                    .sections
                    .get(*section_index)
                    .and_then(|s| s.paragraphs.get(*parent_paragraph_index))
                    .and_then(|p| p.controls.get(*note_control_index));
                if !matches!(control, Some(Control::Footnote(_) | Control::Endnote(_))) {
                    return Err(invalid("각주·미주 주소가 잘못되었습니다"));
                }
                (
                    *section_index,
                    *parent_paragraph_index,
                    vec![(*note_control_index, 0, *paragraph_index)],
                    cell_path,
                )
            }
        };
        path.extend(tail);
        Ok(Address {
            section,
            root,
            path,
        })
    }
    fn with_address(&self, address: &Address) -> Self {
        match self {
            Self::Body { .. } => Self::Body {
                section_index: address.section,
                paragraph_index: address.root,
                cell_path: address.path.clone(),
            },
            Self::HeaderFooter {
                is_header,
                apply_to,
                ..
            } => Self::HeaderFooter {
                section_index: address.section,
                is_header: *is_header,
                apply_to: *apply_to,
                paragraph_index: address.path[0].2,
                cell_path: address.path[1..].to_vec(),
            },
            Self::Note { .. } => Self::Note {
                section_index: address.section,
                parent_paragraph_index: address.root,
                note_control_index: address.path[0].0,
                paragraph_index: address.path[0].2,
                cell_path: address.path[1..].to_vec(),
            },
        }
    }
}

fn locked(control: &Control) -> bool {
    match control {
        Control::Table(table) => table.common.locked,
        Control::Shape(shape) => shape.common().locked,
        Control::Picture(picture) => picture.lock || picture.common.locked,
        Control::Equation(equation) => equation.common.locked,
        _ => false,
    }
}

fn paragraph_mut<'a>(
    section: &'a mut Section,
    address: &Address,
    protect: bool,
) -> Result<&'a mut Paragraph, HwpError> {
    let mut para = section
        .paragraphs
        .get_mut(address.root)
        .ok_or_else(|| invalid("문단 주소가 잘못되었습니다"))?;
    for &(ctrl, cell, child) in &address.path {
        let control = para
            .controls
            .get_mut(ctrl)
            .ok_or_else(|| invalid("상위 개체 주소가 잘못되었습니다"))?;
        if protect && locked(control) {
            return Err(invalid("잠긴 개체 내부의 개체를 옮길 수 없습니다"));
        }
        para = match control {
            Control::Table(t) => {
                let cell = t
                    .cells
                    .get_mut(cell)
                    .ok_or_else(|| invalid("셀 주소가 잘못되었습니다"))?;
                if protect && cell.cell_protect() {
                    return Err(invalid("보호된 셀의 개체를 옮길 수 없습니다"));
                }
                cell.paragraphs.get_mut(child)
            }
            Control::Shape(s) if cell == 0 => {
                get_textbox_from_shape_mut(s).and_then(|t| t.paragraphs.get_mut(child))
            }
            Control::Picture(p) if cell == 0 => {
                p.caption.as_mut().and_then(|c| c.paragraphs.get_mut(child))
            }
            Control::Header(h) if cell == 0 => h.paragraphs.get_mut(child),
            Control::Footer(f) if cell == 0 => f.paragraphs.get_mut(child),
            Control::Footnote(n) if cell == 0 => n.paragraphs.get_mut(child),
            Control::Endnote(n) if cell == 0 => n.paragraphs.get_mut(child),
            _ => None,
        }
        .ok_or_else(|| invalid("내부 문단 주소가 잘못되었습니다"))?;
    }
    Ok(para)
}

fn adjust_path_after_remove(
    target: &mut Address,
    source: &Address,
    control: usize,
) -> Result<(), HwpError> {
    if target.section == source.section
        && target.root == source.root
        && target.path.starts_with(&source.path)
        && target.path.len() > source.path.len()
    {
        let step = &mut target.path[source.path.len()];
        if step.0 == control {
            return Err(invalid("개체를 자기 내부로 옮길 수 없습니다"));
        }
        if step.0 > control {
            step.0 -= 1;
        }
    }
    Ok(())
}

/// 캐럿의 정확한 스트림 경계와 컨트롤 삽입 순서를 함께 구한다.
fn caret_slot(para: &Paragraph, logical: usize) -> Result<(u32, usize), HwpError> {
    let len = para.text.chars().count();
    if para.ctrl_data_records.len() > para.controls.len() {
        return Err(invalid("문단의 부가 데이터가 컨트롤과 일치하지 않습니다"));
    }
    if para.field_ranges.iter().any(|range| {
        range.start_char_idx > range.end_char_idx
            || range.end_char_idx > len
            || range.control_idx >= para.controls.len()
            || range
                .control_idx
                .checked_add(range.inner_slot_count)
                .is_none_or(|end| end >= para.controls.len())
    }) {
        return Err(invalid("필드 범위가 문단과 일치하지 않습니다"));
    }

    let text_len = para.text.chars().count();
    if para.char_offsets.len() != text_len {
        return Err(invalid("문단의 문자 좌표가 잘못되었습니다"));
    }
    let mut previous_end = 0;
    for (offset, ch) in para.char_offsets.iter().zip(para.text.chars()) {
        if *offset < previous_end {
            return Err(invalid("문단의 문자 좌표 순서가 잘못되었습니다"));
        }
        previous_end = offset
            .checked_add(Paragraph::char_stream_len(ch))
            .ok_or_else(|| invalid("문단의 문자 좌표가 너무 큽니다"))?;
    }
    let slots = control_stream_slots(para);
    let mut inline = 0;
    for (ci, control) in para.controls.iter().enumerate() {
        if !control.is_logical_inline() {
            continue;
        }
        let (raw, text) = slots[ci].ok_or_else(|| invalid("개체 슬롯을 찾을 수 없습니다"))?;
        let position = text + inline;
        if logical == position {
            return Ok((raw, ci));
        }
        if logical == position + 1 {
            return Ok((raw + 8, ci + 1));
        }
        if logical < position {
            break;
        }
        inline += 1;
    }
    let before = para
        .controls
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_logical_inline())
        .filter(|(ci, _)| {
            slots[*ci].is_some_and(|(_, text)| {
                text + para.controls[..*ci]
                    .iter()
                    .filter(|c| c.is_logical_inline())
                    .count()
                    < logical
            })
        })
        .count();
    let text = logical
        .checked_sub(before)
        .ok_or_else(|| invalid("캐럿 주소가 잘못되었습니다"))?;
    if text > text_len {
        return Err(invalid("캐럿이 문단 범위를 벗어났습니다"));
    }
    let raw = para.char_offsets.get(text).copied().unwrap_or_else(|| {
        para.char_offsets
            .last()
            .zip(para.text.chars().last())
            .map_or_else(
                || para.char_count.saturating_sub(1),
                |(offset, ch)| offset + Paragraph::char_stream_len(ch),
            )
    });
    let ci = slots
        .iter()
        .position(|slot| slot.is_some_and(|(pos, _)| pos >= raw))
        .unwrap_or(para.controls.len());
    Ok((raw, ci))
}

fn active_shape(para: &Paragraph, pos: u32) -> u32 {
    para.char_shapes
        .iter()
        .rfind(|run| run.start_pos <= pos)
        .map_or(0, |run| run.char_shape_id)
}
fn normalize_shapes(para: &mut Paragraph) {
    para.char_shapes.sort_by_key(|run| run.start_pos);
    let mut runs: Vec<CharShapeRef> = vec![];
    for run in para.char_shapes.drain(..) {
        if runs
            .last()
            .is_some_and(|last| last.start_pos == run.start_pos)
        {
            runs.pop();
        }
        if runs
            .last()
            .is_none_or(|last| last.char_shape_id != run.char_shape_id)
        {
            runs.push(run);
        }
    }
    para.char_shapes = runs;
}

// HWPX 표지와 HWP 범위를 하나의 축으로 편집한다. 짝 없는/알 수 없는 표지는
// 범위로 변환하지 않고 그대로 남겨 기존 부가 정보를 버리지 않는다.
fn prepare_markpen_ranges(para: &mut Paragraph) -> Vec<MarkpenMark> {
    let mut paired = vec![false; para.markpen_marks.len()];
    let mut open = Vec::new();
    for (index, mark) in para.markpen_marks.iter().enumerate() {
        if let Some(color) = &mark.color {
            let valid = color
                .strip_prefix('#')
                .is_some_and(|value| value.len() == 6 && u32::from_str_radix(value, 16).is_ok());
            open.push((index, valid));
        } else if let Some((begin, valid)) = open.pop() {
            if valid
                && mark.stream_position(para) >= para.markpen_marks[begin].stream_position(para)
            {
                paired[begin] = true;
                paired[index] = true;
            }
        }
    }
    let extras = para
        .markpen_marks
        .iter()
        .enumerate()
        .filter(|(index, _)| !paired[*index])
        .map(|(_, mark)| MarkpenMark {
            utf16_pos: Some(mark.stream_position(para)),
            ..mark.clone()
        })
        .collect();
    para.range_tags = para.effective_markpen_range_tags();
    extras
}

fn sync_markpen_marks(para: &mut Paragraph, extras: Vec<MarkpenMark>) {
    // 같은 시작점의 중첩 표지는 바깥 색부터 연다. 닫힌 순서로 수집한 범위를
    // 그대로 import하면 같은 슬롯을 덮은 두 색의 순서가 뒤집힌다.
    let ranges = std::mem::take(&mut para.range_tags);
    para.range_tags = ranges
        .iter()
        .filter(|tag| tag.tag >> 24 == 2)
        .cloned()
        .collect();
    para.range_tags
        .sort_by_key(|tag| (tag.start, std::cmp::Reverse(tag.end)));
    para.import_markpen_range_tags();
    para.range_tags = ranges;
    para.markpen_marks.extend(extras);
    for mark in &mut para.markpen_marks {
        if let Some(pos) = mark.utf16_pos {
            mark.char_idx = para.char_offsets.partition_point(|&offset| offset < pos);
        }
    }
    para.markpen_marks.sort_by_key(|mark| mark.utf16_pos);
}

struct Slot {
    control: Control,
    data: Option<Vec<u8>>,
    shapes: Vec<CharShapeRef>,
    tags: Vec<RangeTag>,
}
fn remove_slot(para: &mut Paragraph, ci: usize) -> Result<(Slot, u32), HwpError> {
    let raw = control_stream_slots(para)
        .get(ci)
        .copied()
        .flatten()
        .ok_or_else(|| invalid("개체 슬롯을 찾을 수 없습니다"))?
        .0;
    let end = raw
        .checked_add(8)
        .ok_or_else(|| invalid("개체 좌표가 너무 큽니다"))?;
    if para
        .char_offsets
        .iter()
        .any(|offset| raw <= *offset && *offset < end)
    {
        return Err(invalid(
            "가시 자리표시자 개체는 텍스트를 보존하며 옮길 수 없습니다",
        ));
    }
    let mut shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: active_shape(para, raw),
    }];
    shapes.extend(
        para.char_shapes
            .iter()
            .filter(|run| raw < run.start_pos && run.start_pos < end)
            .map(|run| CharShapeRef {
                start_pos: run.start_pos - raw,
                char_shape_id: run.char_shape_id,
            }),
    );
    let after = active_shape(para, end);
    para.char_shapes
        .retain(|run| run.start_pos < raw || run.start_pos >= end);
    for run in &mut para.char_shapes {
        if run.start_pos >= end {
            run.start_pos -= 8;
        }
    }
    para.char_shapes.push(CharShapeRef {
        start_pos: raw,
        char_shape_id: after,
    });
    normalize_shapes(para);
    let collapse = |pos: u32| {
        if pos <= raw {
            pos
        } else {
            pos.saturating_sub(8).max(raw)
        }
    };
    let mut extras = prepare_markpen_ranges(para);
    let mut tags = vec![];
    let mut survivors = vec![];
    for mut tag in para.range_tags.drain(..) {
        let overlap = tag.start < end && tag.end > raw;
        let point = tag.start == tag.end && raw <= tag.start && tag.start < end;
        if overlap || point {
            tags.push(RangeTag {
                start: tag.start.max(raw) - raw,
                end: tag.end.min(end) - raw,
                tag: tag.tag,
            });
        }
        let nonempty = tag.start < tag.end;
        tag.start = collapse(tag.start);
        tag.end = collapse(tag.end);
        if !point && !(overlap && nonempty && tag.start == tag.end) {
            survivors.push(tag);
        }
    }
    para.range_tags = survivors;
    for offset in &mut para.char_offsets {
        *offset = collapse(*offset);
    }
    for mark in &mut extras {
        if let Some(pos) = &mut mark.utf16_pos {
            *pos = collapse(*pos);
        }
    }
    sync_markpen_marks(para, extras);
    for range in &mut para.field_ranges {
        if range.control_idx < ci && ci <= range.control_idx.saturating_add(range.inner_slot_count)
        {
            range.inner_slot_count -= 1;
        }
        if range.control_idx > ci {
            range.control_idx -= 1;
        }
    }
    para.align_ctrl_data_records();
    let slot = Slot {
        control: para.controls.remove(ci),
        data: para.ctrl_data_records.remove(ci),
        shapes,
        tags,
    };
    para.char_count = para
        .char_count
        .checked_sub(8)
        .ok_or_else(|| invalid("문단 슬롯 수가 잘못되었습니다"))?;
    para.invalidate_layout_inputs();
    Ok((slot, raw))
}

fn insert_slot(para: &mut Paragraph, raw: u32, ci: usize, slot: Slot) -> Result<(), HwpError> {
    let count = para
        .char_count
        .checked_add(8)
        .ok_or_else(|| invalid("문단이 너무 큽니다"))?;
    if para
        .char_offsets
        .iter()
        .any(|offset| *offset >= raw && offset.checked_add(8).is_none())
    {
        return Err(invalid("문자 좌표가 너무 큽니다"));
    }
    let field_ends = field_stream_ends(para);
    let mut extras = prepare_markpen_ranges(para);
    let restore = active_shape(para, raw);
    for run in &mut para.char_shapes {
        if run.start_pos >= raw {
            run.start_pos = run
                .start_pos
                .checked_add(8)
                .ok_or_else(|| invalid("글자 모양 좌표가 너무 큽니다"))?;
        }
    }
    para.char_shapes
        .extend(slot.shapes.into_iter().map(|run| CharShapeRef {
            start_pos: raw + run.start_pos,
            char_shape_id: run.char_shape_id,
        }));
    para.char_shapes.push(CharShapeRef {
        start_pos: raw + 8,
        char_shape_id: restore,
    });
    normalize_shapes(para);
    for offset in &mut para.char_offsets {
        if *offset >= raw {
            *offset += 8;
        }
    }
    for tag in &mut para.range_tags {
        if tag.start >= raw {
            tag.start = tag
                .start
                .checked_add(8)
                .ok_or_else(|| invalid("영역 좌표가 너무 큽니다"))?;
        }
        if tag.end >= raw {
            tag.end = tag
                .end
                .checked_add(8)
                .ok_or_else(|| invalid("영역 좌표가 너무 큽니다"))?;
        }
    }
    para.range_tags
        .extend(slot.tags.into_iter().map(|tag| RangeTag {
            start: raw + tag.start,
            end: raw + tag.end,
            tag: tag.tag,
        }));
    for mark in &mut extras {
        if let Some(pos) = &mut mark.utf16_pos {
            if *pos >= raw {
                *pos = pos
                    .checked_add(8)
                    .ok_or_else(|| invalid("형광펜 좌표가 너무 큽니다"))?;
            }
        }
    }
    sync_markpen_marks(para, extras);
    for (index, range) in para.field_ranges.iter_mut().enumerate() {
        if range.control_idx < ci && field_ends[index].is_some_and(|end| raw <= end) {
            range.inner_slot_count += 1;
        }
        if range.control_idx >= ci {
            range.control_idx += 1;
        }
    }
    para.align_ctrl_data_records();
    para.controls.insert(ci, slot.control);
    para.ctrl_data_records.insert(ci, slot.data);
    para.char_count = count;
    para.has_para_text = true;
    para.invalidate_layout_inputs();
    Ok(())
}

impl DocumentCore {
    fn reflow_inline_control_owner(&mut self, address: &Address) {
        let Some(&(ctrl, cell, child)) = address.path.last() else {
            self.reflow_paragraph(address.section, address.root);
            return;
        };
        let parent = Address {
            path: address.path[..address.path.len() - 1].to_vec(),
            ..address.clone()
        };
        let control = &paragraph_mut(&mut self.document.sections[address.section], &parent, false)
            .expect("검증한 상위 문단")
            .controls[ctrl];
        if address.path.len() == 1 {
            match control {
                Control::Header(h) => {
                    let apply = super::header_footer_apply_to_u8(h.apply_to);
                    self.reflow_hf_paragraph(address.section, true, apply, child);
                    return;
                }
                Control::Footer(f) => {
                    let apply = super::header_footer_apply_to_u8(f.apply_to);
                    self.reflow_hf_paragraph(address.section, false, apply, child);
                    return;
                }
                Control::Footnote(_) | Control::Endnote(_) => {
                    self.reflow_footnote_paragraph(address.section, address.root, ctrl, child);
                    return;
                }
                _ => {}
            }
        }
        let Some((width, left, right)) = Self::cell_metrics_for_control(control, cell) else {
            return;
        };
        let para = paragraph_mut(&mut self.document.sections[address.section], address, false)
            .expect("검증한 내부 문단");
        let style = self.styles.para_styles.get(para.para_shape_id as usize);
        let available = crate::renderer::hwpunit_to_px(width, self.dpi)
            - crate::renderer::hwpunit_to_px(left as i32 + right as i32, self.dpi)
            - style.map_or(0.0, |style| style.margin_left + style.margin_right);
        crate::renderer::composer::reflow_line_segs(
            para,
            crate::renderer::composer::ParagraphBox::content_width_px(available.max(0.0), self.dpi),
            &self.styles,
            self.dpi,
        );
    }

    /// 복제본에서 모든 주소·슬롯 변경을 검증한 뒤 원본 개체를 옮긴다.
    pub fn move_inline_control_native(
        &mut self,
        source: &InlineControlAddress,
        destination: &InlineControlCaret,
    ) -> Result<InlineControlMoveResult, HwpError> {
        if self.document.header.distribution {
            return Err(invalid("배포용 문서의 개체를 옮길 수 없습니다"));
        }
        let source_address = source.owner.resolve(self)?;
        let mut destination_address = destination.owner.resolve(self)?;
        let source_section = self
            .document
            .sections
            .get(source_address.section)
            .ok_or_else(|| invalid("구역 주소가 잘못되었습니다"))?;
        let destination_section = self
            .document
            .sections
            .get(destination_address.section)
            .ok_or_else(|| invalid("구역 주소가 잘못되었습니다"))?;
        let mut sections = vec![(source_address.section, source_section.clone())];
        if source_address.section != destination_address.section {
            sections.push((destination_address.section, destination_section.clone()));
        }
        let source_para = paragraph_mut(&mut sections[0].1, &source_address, true)?;
        let control = source_para
            .controls
            .get(source.control_index)
            .ok_or_else(|| invalid("개체 주소가 잘못되었습니다"))?;
        if locked(control) {
            return Err(invalid("잠긴 개체를 옮길 수 없습니다"));
        }
        if !control.is_treat_as_char_object() {
            return Err(invalid(
                "글자처럼 취급하는 표·그림·도형·수식만 옮길 수 있습니다",
            ));
        }
        caret_slot(source_para, 0)?;
        let source_text = control_stream_slots(source_para)[source.control_index]
            .ok_or_else(|| invalid("원본 개체 슬롯을 찾을 수 없습니다"))?
            .1;
        let source_position = source_text
            + source_para.controls[..source.control_index]
                .iter()
                .filter(|control| control.is_logical_inline())
                .count();
        let same_paragraph = source_address == destination_address;
        let dest_section_index = usize::from(source_address.section != destination_address.section);
        let destination_para = paragraph_mut(
            &mut sections[dest_section_index].1,
            &destination_address,
            true,
        )?;
        let (mut raw, mut ci) = caret_slot(destination_para, destination.char_offset)?;
        adjust_path_after_remove(
            &mut destination_address,
            &source_address,
            source.control_index,
        )?;
        if same_paragraph
            && (destination.char_offset == source_position
                || destination.char_offset == source_position + 1)
        {
            return Ok(InlineControlMoveResult {
                changed: false,
                address: source.clone(),
                char_offset: source_position,
            });
        }
        let source_para = paragraph_mut(&mut sections[0].1, &source_address, false)?;
        let (slot, old_raw) = remove_slot(source_para, source.control_index)?;
        let char_offset = if same_paragraph && destination.char_offset > source_position {
            destination.char_offset - 1
        } else {
            destination.char_offset
        };
        if same_paragraph {
            if raw > old_raw {
                raw -= 8;
            }
            if ci > source.control_index {
                ci -= 1;
            }
        }
        let destination_para = paragraph_mut(
            &mut sections[dest_section_index].1,
            &destination_address,
            false,
        )?;
        insert_slot(destination_para, raw, ci, slot)?;
        let actual = control_stream_slots(destination_para)[ci]
            .ok_or_else(|| invalid("이동한 개체 슬롯을 검증할 수 없습니다"))?
            .0;
        if actual != raw {
            return Err(invalid("대상 캐럿에 개체 슬롯을 보존할 수 없습니다"));
        }
        let mut source_reflow_address = source_address.clone();
        if source_reflow_address.section == destination_address.section
            && source_reflow_address.root == destination_address.root
            && source_reflow_address
                .path
                .starts_with(&destination_address.path)
            && source_reflow_address.path.len() > destination_address.path.len()
        {
            let step = &mut source_reflow_address.path[destination_address.path.len()];
            if step.0 >= ci {
                step.0 += 1;
            }
        }
        // 복제본과 같은 입력을 다시 적용해 살아 있는 Box 식별자와 조판 캐시를 보존한다.
        let source_para = paragraph_mut(
            &mut self.document.sections[source_address.section],
            &source_address,
            false,
        )
        .expect("검증한 원본 주소");
        let (slot, _) = remove_slot(source_para, source.control_index).expect("검증한 원본 슬롯");
        let destination_para = paragraph_mut(
            &mut self.document.sections[destination_address.section],
            &destination_address,
            false,
        )
        .expect("검증한 대상 주소");
        insert_slot(destination_para, raw, ci, slot).expect("검증한 대상 슬롯");
        for address in [&source_reflow_address, &destination_address] {
            self.document.sections[address.section].raw_stream = None;
            self.reflow_inline_control_owner(address);
        }
        for (section_index, _) in sections {
            self.recompose_section(section_index);
        }
        self.active_field = None;
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        let address = InlineControlAddress {
            owner: destination.owner.with_address(&destination_address),
            control_index: ci,
        };
        self.event_log
            .push(crate::model::event::DocumentEvent::InlineControlMoved {
                source: source.clone(),
                destination: address.clone(),
            });
        Ok(InlineControlMoveResult {
            changed: true,
            address,
            char_offset,
        })
    }
}
