//! 편집이 지은 중첩 표 칸 글줄이 쪽에서 나뉜 자리를 저장 사본에 한/글처럼 적는다.
//!
//! 한/글 12.30은 쪽 경계에서 "나눔"인 중첩 표의 행을 쪽 바닥의 글줄 사이에서 나누면 다음 쪽
//! 첫 줄의 vpos를 0으로 되감아 저장하고, 그 뒤 줄을 그 원점에서 잇는다. 편집 중인 칸 줄은
//! 이어 둔 채라 다음 편집도 쪽을 새로 나눈다. 저장본에 되감김이 없으면 다시 열 때 그 행이
//! 한/글이 한 쪽에 둔 저장 프레임처럼 보여 통째로 다음 쪽에 넘어간다.
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::table::{Table, TablePageBreak};

use crate::storage::{Storage, Unbounded};

type CellPathStep = (usize, usize, usize);

pub(crate) struct NestedRowPageStart {
    host: (usize, usize),
    path: Vec<CellPathStep>,
    lines: Vec<(usize, usize)>,
}

/// 저장 파일에 이미 들어 있는 쪽 원점은 그대로 보존한다.
fn continuous_lines(paragraphs: &[Paragraph]) -> bool {
    let mut previous = i32::MIN;
    paragraphs.iter().all(|paragraph| {
        !paragraph.line_segs.is_empty()
            && paragraph.line_segs.iter().all(|line| {
                let ordered = line.vertical_pos >= previous;
                previous = line.vertical_pos;
                ordered
            })
    })
}

impl DocumentCore {
    pub(crate) fn nested_row_page_starts(&self) -> Vec<NestedRowPageStart> {
        match self.nested_row_page_starts_with_storage(&Unbounded) {
            Ok(starts) => starts,
            Err(never) => match never {},
        }
    }

    pub(crate) fn nested_row_page_starts_with_storage<S: Storage>(
        &self,
        storage: &S,
    ) -> Result<Vec<NestedRowPageStart>, S::Error> {
        let mut starts = Vec::new();
        if !self.effective_layout_profile().session_edited() {
            return Ok(starts);
        }
        for (section_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    if let Control::Table(table) = control {
                        let mut path = storage.collect([(control_idx, 0, 0)])?;
                        self.collect_nested_row_page_starts(
                            (section_idx, para_idx),
                            table,
                            &mut path,
                            &mut starts,
                            storage,
                        )?;
                    }
                }
            }
        }
        Ok(starts)
    }

    fn collect_nested_row_page_starts<S: Storage>(
        &self,
        host: (usize, usize),
        table: &Table,
        path: &mut Vec<CellPathStep>,
        starts: &mut Vec<NestedRowPageStart>,
        storage: &S,
    ) -> Result<(), S::Error> {
        let depth = path.len() - 1;
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            path[depth].1 = cell_idx;
            if depth > 0
                && !table.common.treat_as_char
                && table.layout_page_break() == TablePageBreak::RowBreak
                && cell.paragraphs.iter().any(|p| p.cell_lines_edited)
                && continuous_lines(&cell.paragraphs)
            {
                let lines = self.edited_cell_page_starts(host, path, &cell.paragraphs, storage)?;
                if !lines.is_empty() {
                    storage.push(
                        starts,
                        NestedRowPageStart {
                            host,
                            path: storage.collect(path.iter().copied())?,
                            lines,
                        },
                    )?;
                }
            }
            for (cell_para_idx, paragraph) in cell.paragraphs.iter().enumerate() {
                path[depth].2 = cell_para_idx;
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    if let Control::Table(child) = control {
                        storage.push(path, (control_idx, 0, 0))?;
                        self.collect_nested_row_page_starts(host, child, path, starts, storage)?;
                        path.pop();
                    }
                }
            }
        }
        Ok(())
    }

    fn edited_cell_page_starts<S: Storage>(
        &self,
        host: (usize, usize),
        path: &mut [CellPathStep],
        paragraphs: &[Paragraph],
        storage: &S,
    ) -> Result<Vec<(usize, usize)>, S::Error> {
        let lines = storage.collect(
            paragraphs
                .iter()
                .enumerate()
                .flat_map(|(p, para)| (0..para.line_segs.len()).map(move |line| (p, line))),
        )?;
        let mut starts = Vec::new();
        if lines.len() < 2 {
            return Ok(starts);
        }
        let mut page_at = |index: usize| {
            let (p, line) = lines[index];
            let para = &paragraphs[p];
            path.last_mut()?.2 = p;
            let offset = para
                .char_offsets
                .partition_point(|&offset| offset < para.line_segs[line].text_start);
            self.locate_cell_cursor(host.0, host.1, path, offset, None, None)
                .ok()
                .map(|(page, _)| page)
        };
        let last = lines.len() - 1;
        let (Some(mut page), Some(last_page)) = (page_at(0), page_at(last)) else {
            return Ok(starts);
        };
        let mut first = 0;
        // 연속된 칸 글줄의 쪽 소속은 순서대로 전진한다. 각 다음 쪽의 첫 줄만 찾는다.
        while page < last_page {
            let (mut left, mut right) = (first + 1, last);
            while left < right {
                let middle = left + (right - left) / 2;
                let Some(found) = page_at(middle) else {
                    return Ok(Vec::new());
                };
                if found > page {
                    right = middle;
                } else {
                    left = middle + 1;
                }
            }
            let Some(next) = page_at(left) else {
                return Ok(Vec::new());
            };
            if next <= page || next > last_page {
                return Ok(Vec::new());
            }
            storage.push(&mut starts, lines[left])?;
            first = left;
            page = next;
        }
        Ok(starts)
    }

    pub(crate) fn write_nested_row_page_starts(
        document: &mut Document,
        starts: &[NestedRowPageStart],
    ) {
        for start in starts {
            let Some(paragraphs) =
                cell_paragraphs_mut(document, start.host.0, start.host.1, &start.path)
            else {
                continue;
            };
            // 음수 원점도 형식에서 허용한다. 차이가 저장 필드에 맞을 때만 칸 전체에 적용한다.
            let mut delta = 0;
            let mut resets = start.lines.iter().copied().peekable();
            let representable = paragraphs.iter().enumerate().all(|(p, para)| {
                para.line_segs.iter().enumerate().all(|(i, line)| {
                    if resets.next_if_eq(&(p, i)).is_some() {
                        delta = line.vertical_pos;
                    }
                    line.vertical_pos.checked_sub(delta).is_some()
                })
            });
            if !representable || resets.peek().is_some() {
                continue;
            }
            let mut delta = 0;
            let mut resets = start.lines.iter().copied().peekable();
            for (p, para) in paragraphs.iter_mut().enumerate() {
                for (i, line) in para.line_segs.iter_mut().enumerate() {
                    if resets.next_if_eq(&(p, i)).is_some() {
                        delta = line.vertical_pos;
                    }
                    line.vertical_pos -= delta;
                }
            }
        }
    }
}

fn cell_paragraphs_mut<'a>(
    document: &'a mut Document,
    section_idx: usize,
    para_idx: usize,
    path: &[CellPathStep],
) -> Option<&'a mut Vec<Paragraph>> {
    let mut paragraph = document
        .sections
        .get_mut(section_idx)?
        .paragraphs
        .get_mut(para_idx)?;
    let (last, outer) = path.split_last()?;
    for &(control, cell, cell_para) in outer {
        let Control::Table(table) = paragraph.controls.get_mut(control)? else {
            return None;
        };
        paragraph = table.cells.get_mut(cell)?.paragraphs.get_mut(cell_para)?;
    }
    let Control::Table(table) = paragraph.controls.get_mut(last.0)? else {
        return None;
    };
    Some(&mut table.cells.get_mut(last.1)?.paragraphs)
}
