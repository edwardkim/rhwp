//! 분할 편집이 만든 누적 셀 캐시를 현재 줄 구성으로 복원한다.

use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::renderer::composer::{reflow_line_segs_after_cell_split, ParagraphBox};
use crate::renderer::hwpunit_to_px;
use crate::renderer::page_layout::PageLayoutInfo;

impl DocumentCore {
    /// native 저장을 거친 1×2 분할의 clone 계보와 누적 좌표를 함께 확인한다.
    /// 한컴의 물리 쪽 reset 또는 일반 2열 셀의 저장 줄은 대상이 아니다.
    /// 재래핑과 세로 사다리는 편집 경로의 공통 producer를 그대로 사용한다.
    pub(super) fn repair_reparsed_split_cell_cache(&mut self) {
        if self.source_format != crate::parser::FileFormat::Hwp {
            return;
        }
        // 원본 프로그램 등록 뒤의 유효 스타일이다. 편집 producer도
        // resolve_render_styles를 사용하므로 초기 self.styles로 재추정하지 않는다.
        let styles = self.resolve_render_styles();
        let mut jobs = Vec::new();
        for (section_index, section) in self.document.sections.iter().enumerate() {
            let columns = Self::find_initial_column_def(&section.paragraphs);
            let layout =
                PageLayoutInfo::from_page_def(&section.section_def.page_def, &columns, self.dpi);
            let body_height = crate::renderer::px_to_hwpunit(layout.body_area.height, self.dpi);
            for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_index, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    // clone 판별은 나머지 행이 모두 전폭인 경우만 성립한다.
                    // 일반 거대 2열 표의 모든 행에서 같은 검사를 반복하지 않는다.
                    if table.col_count != 2 {
                        continue;
                    }
                    let mut right_cells = table.cells.iter().filter(|cell| cell.col == 1);
                    let Some(right) = right_cells.next() else {
                        continue;
                    };
                    if right_cells.next().is_some() {
                        continue;
                    }
                    {
                        let row = usize::from(right.row);
                        if !crate::renderer::typeset::is_reparsed_single_column_cell_split_row(
                            table, row,
                        ) {
                            continue;
                        }
                        let Some((cell_index, cell)) = table
                            .cells
                            .iter()
                            .enumerate()
                            .find(|(_, cell)| cell.row as usize == row && cell.col == 0)
                        else {
                            continue;
                        };
                        let mut previous = None;
                        let mut cumulative = false;
                        let mut reset = false;
                        for line in cell.paragraphs.iter().flat_map(|p| &p.line_segs) {
                            reset |= previous.is_some_and(|v| line.vertical_pos < v);
                            cumulative |= line.vertical_pos > body_height;
                            previous = Some(line.vertical_pos);
                        }
                        if cumulative && !reset {
                            jobs.push((
                                section_index,
                                paragraph_index,
                                control_index,
                                cell_index,
                                cell.paragraphs.len(),
                                Self::table_cell_reflow_metrics(table),
                            ));
                        }
                    }
                }
            }
        }
        for (section, paragraph, control, cell, _, metrics) in jobs {
            let Some(&(width, left, right)) = metrics.get(cell) else {
                continue;
            };
            let inner = width
                .saturating_sub(left as i32)
                .saturating_sub(right as i32);
            let content = ParagraphBox::content_width_px(hwpunit_to_px(inner, self.dpi), self.dpi);
            let mut repaired = false;
            if let Control::Table(table) =
                &mut self.document.sections[section].paragraphs[paragraph].controls[control]
            {
                for source in &mut table.cells[cell].paragraphs {
                    // 표만 담은 문단과 폭 0 개체 앵커는 텍스트 가용 상자를 소유하지 않는다.
                    // TAC 표도 자체 폭/정렬을 소비하므로 그 저장 앵커를 텍스트 프레임으로
                    // 바꾸지 않는다. 글자가 섞인 문단과 수식 줄은 실제 줄 구성 대상이다.
                    let table_only_host = source.text.chars().all(char::is_whitespace)
                        && !source.controls.is_empty()
                        && source
                            .controls
                            .iter()
                            .all(|control| matches!(control, Control::Table(_)));
                    if table_only_host
                        || source.line_segs.is_empty()
                        || source.line_segs.iter().all(|line| line.segment_width == 0)
                    {
                        continue;
                    }
                    let frame = ParagraphBox::content_for_style(
                        content.width_px(self.dpi),
                        styles.para_styles.get(source.para_shape_id as usize),
                        self.dpi,
                    );
                    let horizontal = frame.effective();
                    if !frame.is_usable()
                        || source.line_segs.iter().all(|line| {
                            line.column_start == horizontal.start
                                && line.segment_width == frame.width_hwp()
                        })
                    {
                        continue;
                    }
                    // 분할 저장 캐시의 원점·폭이 실제 셀/문단 여백과 다를 때만
                    // 편집 producer를 사용한다. 유효한 저장 줄의 컷은 재추정하지 않는다.
                    reflow_line_segs_after_cell_split(source, frame, &styles, self.dpi);
                    repaired = true;
                }
            }
            if repaired {
                self.rebuild_table_cell_vpos_ladder_native(section, paragraph, control, cell);
            }
        }
    }
}
