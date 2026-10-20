//! 부모의 저장 쪽 경계로 확인한 작은 중첩 글자 표의 물리 프레임.

use crate::model::{
    control::Control,
    document::Document,
    paragraph::{LineSeg, Paragraph},
    shape::{TextWrap, VertRelTo},
    table::{Cell, Table, TablePageBreak},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct StoredTextFrameUnit {
    pub paragraph: usize,
    pub line: usize,
    pub physical_height_hu: i32,
    pub starts_frame: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StoredTextFrame {
    pub start: usize,
    pub end: usize,
    pub body_height_hu: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredNestedTextFrames {
    pub units: Vec<StoredTextFrameUnit>,
    pub frames: Vec<StoredTextFrame>,
}

impl StoredNestedTextFrames {
    pub fn frame_for_cut(&self, start: usize, end: usize) -> Option<StoredTextFrame> {
        self.frames
            .iter()
            .copied()
            .find(|frame| frame.start == start && frame.end == end.min(self.units.len()))
    }
}

fn original(paragraph: &Paragraph) -> bool {
    !paragraph.stored_text_partition_is_dirty()
        && !paragraph.cell_format_vpos_dirty
        && !paragraph.line_segs.is_empty()
        && paragraph.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos >= 0
                && line.line_height > 0
                && line.line_spacing >= 0
                && line.segment_width > 0
        })
}

fn prove(
    document: &Document,
    owner: &Table,
    parent: &Cell,
    host_index: usize,
    child: &Table,
) -> Option<StoredNestedTextFrames> {
    let host = parent.paragraphs.get(host_index)?;
    let successor = parent.paragraphs.get(host_index + 1)?;
    if owner.page_break != TablePageBreak::RowBreak
        || owner.common.treat_as_char
        || parent.dirty_flag
        || host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || successor.stored_text_partition_is_dirty()
        || successor.cell_format_vpos_dirty
        || !host.text.trim().is_empty()
        || host.controls.len() != 1
        || host.line_segs.len() != 1
        || successor.line_segs.is_empty()
        || host.line_segs[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || successor.line_segs[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        // HWPX의 CELL은 내부 RowBreak다. serializer의 문자열과 내부 enum을 혼동하지 않는다.
        || child.page_break != TablePageBreak::RowBreak
        || child.common.treat_as_char
        || !child.common.flow_with_text
        || child.common.text_wrap != TextWrap::TopAndBottom
        || child.common.vert_rel_to != VertRelTo::Para
        || child.common.vertical_offset != 0
        || child.caption.is_some()
        || child.row_count != 1
        || child.col_count != 1
        || child.cells.len() != 1
        || child.common.height == 0
        || child.common.height > i32::MAX as u32
        || child.outer_margin_top < 0
        || child.outer_margin_bottom < 0
    {
        return None;
    }
    let cell = &child.cells[0];
    if cell.dirty_flag
        || cell.row_span != 1
        || cell.col_span != 1
        || cell.height <= child.common.height
        || cell.paragraphs.iter().any(|paragraph| {
            !original(paragraph)
                || !paragraph.controls.is_empty()
                || paragraph.text.trim().is_empty()
        })
    {
        return None;
    }
    let spacing = |index: usize| {
        let paragraph = cell.paragraphs.get(index)?;
        let shape = document
            .doc_info
            .para_shapes
            .get(usize::from(paragraph.para_shape_id))?;
        Some((shape.spacing_before / 2, shape.spacing_after / 2))
    };
    let lines: Vec<_> = cell
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(pi, paragraph)| {
            paragraph
                .line_segs
                .iter()
                .enumerate()
                .map(move |(li, line)| (pi, li, line))
        })
        .collect();
    let first = lines.first()?;
    if first.2.vertical_pos != spacing(first.0)?.0 {
        return None;
    }
    let mut restart = None;
    for (index, pair) in lines.windows(2).enumerate() {
        let (pi, _, previous) = pair[0];
        let (next_pi, next_li, next) = pair[1];
        if next.vertical_pos < previous.vertical_pos {
            if restart.is_some()
                || next_li != 0
                || pi == next_pi
                || next.vertical_pos != spacing(next_pi)?.0
            {
                return None;
            }
            restart = Some(index + 1);
        } else {
            let advance = i64::from(previous.line_height)
                + i64::from(previous.line_spacing)
                + if pi == next_pi {
                    0
                } else {
                    i64::from(spacing(pi)?.1) + i64::from(spacing(next_pi)?.0)
                };
            if i64::from(next.vertical_pos) - i64::from(previous.vertical_pos) != advance {
                return None;
            }
        }
    }
    let restart = restart?;
    let padding = cell.effective_padding(&child.padding);
    let pad_top = i32::from(padding.top);
    let pad_bottom = i32::from(padding.bottom);
    if pad_top < 0 || pad_bottom < 0 {
        return None;
    }
    let opening_ink_end = lines[restart - 1]
        .2
        .vertical_pos
        .checked_add(lines[restart - 1].2.line_height)?
        .checked_add(pad_top)?
        .checked_add(pad_bottom)?;
    let opening = i32::try_from(child.common.height).ok()?;
    if opening < opening_ink_end {
        return None;
    }
    let last = lines.last()?.2;
    let tail = last
        .vertical_pos
        .checked_add(last.line_height)?
        .checked_add(pad_top)?
        .checked_add(pad_bottom)?;
    let outer_top = i32::from(child.outer_margin_top);
    let outer_bottom = i32::from(child.outer_margin_bottom);
    // 부모 뒤 줄의 새 원점이 자식 마지막 상자와 사방 여백을 정확히 닫는다.
    // 작은 로컬 되감김이나 선언 높이만으로 쪽 경계를 승격하지 않는다.
    if successor.line_segs[0].vertical_pos
        != tail.checked_add(outer_top)?.checked_add(outer_bottom)?
        || successor.line_segs[0].vertical_pos >= host.line_segs[0].vertical_pos
    {
        return None;
    }
    let frames = vec![
        StoredTextFrame {
            start: 0,
            end: restart,
            body_height_hu: opening,
        },
        StoredTextFrame {
            start: restart,
            end: lines.len(),
            body_height_hu: tail,
        },
    ];
    for frame in &frames {
        frame
            .body_height_hu
            .checked_add(outer_top)?
            .checked_add(outer_bottom)?;
    }
    let mut units = Vec::with_capacity(lines.len());
    for frame in &frames {
        for index in frame.start..frame.end {
            let (paragraph, line, source) = lines[index];
            let end = index + 1 == frame.end;
            let mut height = if end {
                source.line_height
            } else {
                lines[index + 1].2.vertical_pos - source.vertical_pos
            };
            if index == frame.start {
                height += source.vertical_pos + pad_top + outer_top;
            }
            if end {
                height += pad_bottom + outer_bottom;
                if frame.start == 0 {
                    height += opening - opening_ink_end;
                }
            }
            units.push(StoredTextFrameUnit {
                paragraph,
                line,
                physical_height_hu: height,
                starts_frame: index == frame.start,
            });
        }
    }
    Some(StoredNestedTextFrames { units, frames })
}

pub(super) fn collect(document: &Document) -> HashMap<usize, Arc<StoredNestedTextFrames>> {
    fn visit(
        document: &Document,
        table: &Table,
        result: &mut HashMap<usize, Arc<StoredNestedTextFrames>>,
    ) {
        for cell in &table.cells {
            for (pi, paragraph) in cell.paragraphs.iter().enumerate() {
                for control in &paragraph.controls {
                    if let Control::Table(child) = control {
                        if let Some(frames) = prove(document, table, cell, pi, child) {
                            result
                                .insert(child.as_ref() as *const Table as usize, Arc::new(frames));
                        }
                        visit(document, child, result);
                    }
                }
            }
        }
    }
    let mut result = HashMap::new();
    let profile = document.layout_profile();
    if profile.hwp5_stored_pagination_layout() && !profile.session_edited() {
        for section in &document.sections {
            for paragraph in &section.paragraphs {
                for control in &paragraph.controls {
                    if let Control::Table(table) = control {
                        visit(document, table, &mut result);
                    }
                }
            }
        }
    }
    result
}
