//! 원본 첫 개체 상자가 본문 아래를 닫으면 후속 저장 쪽 컷도 같은 물리 프레임을 소유한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{control::Control, paragraph::Paragraph, shape::VertAlign, table::Table};

impl LayoutEngine {
    /// 실제 병합 종료 컷이 다음 저장 쪽을 열면 앞 조각도 독립된 정렬 상자다.
    /// 본문 프레임 소유 확인은 호출자가 공유하며, 여기서는 같은 실제 유닛을 읽는다.
    pub(in crate::renderer::layout) fn original_restart_span_cut_owns_alignment(
        &self,
        table: &Table,
        cell: &crate::model::table::Cell,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let units = self.cell_units(cell, table, styles);
        cell.row_span > 1
            && end > 0
            && end < units.len()
            && self
                .native_multirow_saved_reset_frame_cut(table, cell, &units, end, styles)
                .is_some()
    }

    /// 컷 인덱스가 모든 일반 소유자의 끝에 있는가. 남은 내용이 큰 예산에
    /// 들어가는 것과 이미 모든 유닛을 소비한 것은 서로 다른 상태다.
    pub(crate) fn row_cut_consumes_all_ordinary_units(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let owners = Self::row_cut_cell_order(table, row);
        !owners.is_empty()
            && owners.len() == cut.len()
            && owners.iter().zip(cut).all(|(&index, &end)| {
                end == self.cell_units(&table.cells[index], table, styles).len()
            })
    }

    /// 실제 일반 컷과 걸침 컷이 소유하는 원본 쪽 재시작을 대조한다.
    /// 처음부터0인 줄 하나로 쪽을 추측하지 않으며, 완료 셀의 빈 높이와
    /// 다음 쪽에 남은 병합 셀의 유닛을 별도로 확인한다.
    pub(crate) fn row_cut_closes_original_restart_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        row_heights: &[f64],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let owners = Self::row_cut_cell_order(table, row);
        if owners.is_empty() || owners.len() != cut.len() || row >= row_heights.len() {
            return false;
        }
        let mut unfinished = 0;
        let mut advances_after_restart = false;
        for (index, &cell_index) in owners.iter().enumerate() {
            let cell = &table.cells[cell_index];
            let units = self.cell_units(cell, table, styles);
            let end = cut[index];
            if end > units.len() {
                return false;
            }
            if end == units.len() {
                continue;
            }
            unfinished += 1;
            // 첫 줄의 공동0→0 슬롯만 이 경로가 소유한다. 이후 양수
            // 되감김은 기존 공동 프레임 계약에서 판정한다.
            let lines: Vec<_> = cell
                .paragraphs
                .iter()
                .flat_map(|para| para.line_segs.iter().map(move |line| (para, line)))
                .collect();
            if end != 1
                || units[0].para_idx != 0
                || units[0].vis_start != 0
                || units[0].vis_end != 1
                || lines.len() < 2
                || lines[..2].iter().any(|(para, line)| {
                    !para.controls.is_empty()
                        || line.vertical_pos != 0
                        || line.line_height <= 0
                        || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                            != 0
                })
            {
                return false;
            }
            // 다음 실제 유닛이 바로 두 번째 저장 줄을 소유해야 한다.
            let next = &units[end];
            let Some(next_line) = cell
                .paragraphs
                .get(next.para_idx)
                .and_then(|para| para.line_segs.get(next.vis_start))
            else {
                return false;
            };
            if !std::ptr::eq(next_line, lines[1].1) {
                return false;
            }
            advances_after_restart |= lines.get(2).is_some_and(|(para, line)| {
                para.controls.is_empty()
                    && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && i64::from(line.vertical_pos)
                        == i64::from(next_line.line_height)
                            + i64::from(next_line.line_spacing.max(0))
            });
        }
        let mut spanning_restart = false;
        for cell in table.cells.iter().filter(|cell| {
            let first = usize::from(cell.row);
            first <= row && row + 1 < first + usize::from(cell.row_span)
        }) {
            let first = usize::from(cell.row);
            let physical = row_heights[first..=row].iter().sum::<f64>()
                + crate::renderer::hwpunit_to_px(table.cell_spacing as i32, self.dpi)
                    * (row - first) as f64;
            let (_, end) = self.rowbreak_straddle_cut_units(
                table,
                cell,
                0,
                row + 1,
                &[],
                false,
                None,
                cut,
                false,
                physical,
                row_heights,
                &[],
                styles,
            );
            let units = self.cell_units(cell, table, styles);
            if end == units.len() {
                continue;
            }
            if self
                .native_multirow_saved_reset_frame_cut(table, cell, &units, end, styles)
                .is_none()
            {
                return false;
            }
            spanning_restart = true;
        }
        // 미소비 소유자가 하나여도 같은 셀의0→0 뒤 정상 사다리가
        // 이어지면 실제 저장 쪽 재시작이다. 완료 소유자의 수는 쪽
        // 프레임을 줄이지 않으며, 양수 reset의 공동 계약과 구분한다.
        (unfinished > 0 && advances_after_restart) || (unfinished == 0 && spanning_restart)
    }

    /// 저장 호스트·개체 높이로 첫 본문 프레임의 종료를 독립적으로 확인한다.
    /// 내용 높이 또는 현재 출력 좌표를 증거로 쓰지 않는다. 한컴의 정수 HU
    /// 저장 반올림 두 단위만 허용하며, 편집·재조판한 원본에는 적용하지 않는다.
    pub(crate) fn original_body_closed_table_frame(
        &self,
        host: &Paragraph,
        table: &Table,
        body_height: f64,
    ) -> bool {
        let profile = self.profile.get();
        let [line] = host.line_segs.as_slice() else {
            return false;
        };
        if !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || super::super::para_has_non_whitespace_text(host)
            || !matches!(host.controls.as_slice(), [Control::Table(_)])
            || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.vertical_pos < 0
            || line.line_height <= 0
            || !body_height.is_finite()
            || body_height <= 0.0
            || table.common.treat_as_char
            || !crate::renderer::float_placement::is_para_topbottom_float(&table.common)
            || !table.common.flow_with_text
            || table.common.allow_overlap
            || table.common.vert_align != VertAlign::Top
            || (table.common.vertical_offset as i32) < 0
            || table.common.height == 0
            || table.common.height > i32::MAX as u32
            || table.row_count <= 1
            || table.page_break != crate::model::table::TablePageBreak::RowBreak
            || table.caption.is_some()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.cells.iter().any(|cell| {
                cell.dirty_flag
                    || cell.paragraphs.iter().any(|para| {
                        para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty
                    })
            })
        {
            return false;
        }
        let stored_bottom = i64::from(line.vertical_pos)
            + i64::from(table.common.vertical_offset as i32)
            + i64::from(table.outer_margin_top)
            + i64::from(table.common.height);
        let frame_bottom = crate::renderer::float_placement::single_cell_page_fragment_bottom(
            table,
            body_height,
            self.dpi,
        );
        let frame_bottom_hu = (frame_bottom * 7200.0 / self.dpi).round() as i64;
        (stored_bottom - frame_bottom_hu).abs() <= 2
    }

    /// 미소비 셀 모두가 같은 원본 쪽 경계에서 끝나는지 확인한다.
    /// 완료 셀과 미소비 셀을 구분하며 한 셀의 로컬 reset으로 전체 행을 늘리지 않는다.
    pub(crate) fn row_cut_closes_shared_original_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let owners = Self::row_cut_cell_order(table, row);
        // 일반 행 컷은 row_span == 1 소유자만 표현한다. 이 행을 가로지르는
        // 병합 셀이 있으면 블록 컷의 별도 소유 관계가 필요하므로 추정하지 않는다.
        if owners.is_empty()
            || owners.len() != cut.len()
            || table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && usize::from(cell.row) <= row
                    && row < usize::from(cell.row) + usize::from(cell.row_span)
            })
        {
            return false;
        }
        let mut unfinished = 0;
        for (index, cell_index) in owners.into_iter().enumerate() {
            let cell = &table.cells[cell_index];
            let units = self.cell_units(cell, table, styles);
            let end = cut[index];
            if end > units.len() {
                return false;
            }
            if end == units.len() {
                continue;
            }
            unfinished += 1;
            let certified = self
                .native_multirow_saved_reset_frame_cut(table, cell, &units, end, styles)
                .is_some();
            if certified {
                continue;
            }
            let Some(previous) = end.checked_sub(1).and_then(|i| units.get(i)) else {
                return false;
            };
            let next = &units[end];
            let Some(before_para) = cell.paragraphs.get(previous.para_idx) else {
                return false;
            };
            let Some(after_para) = cell.paragraphs.get(next.para_idx) else {
                return false;
            };
            let before = if previous.empty_spacer {
                before_para.line_segs.last()
            } else {
                previous
                    .vis_end
                    .checked_sub(1)
                    .and_then(|i| before_para.line_segs.get(i))
            };
            let after = if next.empty_spacer {
                after_para.line_segs.first()
            } else {
                after_para.line_segs.get(next.vis_start)
            };
            let (Some(before), Some(after)) = (before, after) else {
                return false;
            };
            // 종료 간격의 개별 술어는 같은 높이의 줄을 요구할 수 있다. 본문
            // 프레임의 공동 종료는 각 셀의 실제 컷을 대조하며 글꼴 높이를 같다고
            // 가정하지 않는다. 개체 문단의 로컬 재시작은 이 증거에 포함하지 않는다.
            if !before_para.controls.is_empty()
                || !after_para.controls.is_empty()
                || (before.tag | after.tag)
                    & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                    != 0
                || before.vertical_pos <= 0
                || after.vertical_pos != 0
                || next.para_idx < previous.para_idx
                || (next.para_idx == previous.para_idx && previous.vis_end != next.vis_start)
                || (next.para_idx != previous.para_idx
                    && !previous.empty_spacer
                    && previous.vis_end != before_para.line_segs.len())
            {
                return false;
            }
        }
        // 이미 완료한 셀은 다음 쪽의 공동 소유자가 아니다. 한 셀의 정상 쪽
        // 재시작은 내용 종료를 증명해도 빈 띠를 포함한 행 전체 종료를 증명하지
        // 않는다. 실제 미소비 소유자 둘 이상이 같은 종료 컷을 공유해야 한다.
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "SOURCE_BODY_FRAME_OWNERS: row={} cut={:?} unfinished={}",
                row, cut, unfinished
            );
        }
        unfinished > 1
    }
}
