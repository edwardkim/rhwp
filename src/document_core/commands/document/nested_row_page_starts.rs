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

/// 저장 사본에서 쪽 원점으로 되감을 칸 줄: (구역, 본문 문단, 칸까지의 경로, 칸 문단, 줄).
/// 경로 마지막 항목의 칸 문단 번호는 쓰지 않는다.
pub(crate) type NestedRowPageStart = (usize, usize, Vec<(usize, usize, usize)>, usize, usize);

/// 칸 줄이 위에서 아래로만 이어지는가(되감김이 이미 있는 칸은 그 원점을 그대로 둔다).
fn continuous_lines(paragraphs: &[Paragraph]) -> bool {
    let mut previous = i32::MIN;
    paragraphs.iter().all(|paragraph| {
        !paragraph.line_segs.is_empty()
            && paragraph.line_segs.iter().all(|line| {
                let ok = line.vertical_pos >= previous;
                previous = line.vertical_pos;
                ok
            })
    })
}

impl DocumentCore {
    pub(crate) fn nested_row_page_starts(&self) -> Vec<NestedRowPageStart> {
        let mut starts = Vec::new();
        for (section_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    if let Control::Table(table) = control {
                        self.collect_nested_row_page_starts(
                            (section_idx, para_idx),
                            table,
                            &mut vec![(control_idx, 0, 0)],
                            &mut starts,
                        );
                    }
                }
            }
        }
        starts
    }

    fn collect_nested_row_page_starts(
        &self,
        host: (usize, usize),
        table: &Table,
        path: &mut Vec<(usize, usize, usize)>,
        starts: &mut Vec<NestedRowPageStart>,
    ) {
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            path.last_mut().expect("경로는 표마다 한 항목").1 = cell_idx;
            if path.len() > 1
                && !table.common.treat_as_char
                && table.layout_page_break() == TablePageBreak::RowBreak
                && cell.paragraphs.iter().any(|p| p.cell_lines_edited)
                && continuous_lines(&cell.paragraphs)
            {
                self.collect_cell_page_starts(host, path, &cell.paragraphs, starts);
            }
            for (cell_para_idx, paragraph) in cell.paragraphs.iter().enumerate() {
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    if let Control::Table(child) = control {
                        path.last_mut().expect("경로는 표마다 한 항목").2 = cell_para_idx;
                        path.push((control_idx, 0, 0));
                        self.collect_nested_row_page_starts(host, child, path, starts);
                        path.pop();
                    }
                }
            }
        }
    }

    /// 칸 줄마다 놓인 쪽을 캐럿 자리로 읽어, 앞 줄보다 뒤 쪽에서 시작하는 줄을 모은다.
    fn collect_cell_page_starts(
        &self,
        (section_idx, para_idx): (usize, usize),
        path: &[(usize, usize, usize)],
        paragraphs: &[Paragraph],
        starts: &mut Vec<NestedRowPageStart>,
    ) {
        let mut previous_page = None;
        for (cell_para_idx, paragraph) in paragraphs.iter().enumerate() {
            let json = path
                .iter()
                .enumerate()
                .map(|(depth, &(control, cell, cell_para))| {
                    let cell_para = if depth + 1 == path.len() {
                        cell_para_idx
                    } else {
                        cell_para
                    };
                    format!(
                        r#"{{"controlIndex":{control},"cellIndex":{cell},"cellParaIndex":{cell_para}}}"#
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let json = format!("[{json}]");
            for (line_idx, line) in paragraph.line_segs.iter().enumerate() {
                let char_offset = paragraph
                    .char_offsets
                    .iter()
                    .position(|&offset| offset >= line.text_start)
                    .unwrap_or(paragraph.char_offsets.len());
                let Some(page) = self
                    .get_cursor_rect_by_path_native(section_idx, para_idx, &json, char_offset)
                    .ok()
                    .and_then(|rect| serde_json::from_str::<serde_json::Value>(&rect).ok())
                    .and_then(|rect| rect["pageIndex"].as_u64())
                else {
                    return;
                };
                if previous_page.is_some_and(|previous| page > previous) {
                    starts.push((
                        section_idx,
                        para_idx,
                        path.to_vec(),
                        cell_para_idx,
                        line_idx,
                    ));
                }
                previous_page = Some(page);
            }
        }
    }

    /// 저장 사본에서 쪽마다 그 쪽 첫 칸 줄을 원점(0)으로 두고 다음 쪽 첫 줄 앞까지 같은 만큼 올린다.
    pub(crate) fn write_nested_row_page_starts(
        document: &mut Document,
        starts: &[NestedRowPageStart],
    ) {
        let mut index = 0;
        while index < starts.len() {
            let (section_idx, para_idx, path, _, _) = &starts[index];
            let same_cell = starts[index..]
                .iter()
                .take_while(|(s, p, other, _, _)| {
                    s == section_idx && p == para_idx && other == path
                })
                .count();
            let cell_starts: Vec<(usize, usize)> = starts[index..index + same_cell]
                .iter()
                .map(|&(_, _, _, cell_para, line)| (cell_para, line))
                .collect();
            if let Some(paragraphs) = cell_paragraphs_mut(document, *section_idx, *para_idx, path) {
                let mut delta = 0;
                for (cell_para_idx, paragraph) in paragraphs.iter_mut().enumerate() {
                    for (line_idx, line) in paragraph.line_segs.iter_mut().enumerate() {
                        if cell_starts.contains(&(cell_para_idx, line_idx)) {
                            delta = line.vertical_pos;
                        }
                        line.vertical_pos -= delta;
                    }
                }
            }
            index += same_cell;
        }
    }
}

fn cell_paragraphs_mut<'a>(
    document: &'a mut Document,
    section_idx: usize,
    para_idx: usize,
    path: &[(usize, usize, usize)],
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
