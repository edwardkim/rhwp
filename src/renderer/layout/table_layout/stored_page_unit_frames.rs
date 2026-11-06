//! 재조판 글줄 뒤의 원본 쪽 프레임 점유를 공통 유닛에 보존한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, VerticalAlign};
use crate::renderer::float_placement::{stored_text_page_frame, StoredTextPageFrame};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    /// 앞 행이 원본 줄에 맞춰 성장한 경우에도 개체의 첫 프레임과 정확히 닫히는
    /// 행 경계를 보존한다. 셀 유닛을 다시 만들지 않고 이미 확정한 물리 행을 읽는다.
    pub(super) fn stored_measured_object_frame_row_count(&self, table: &Table) -> Option<usize> {
        let profile = self.profile.get();
        let normalization = self.render_normalization.borrow();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.common.treat_as_char
            || table.common.height == 0
            || table.common.height > i32::MAX as u32
            || table.layout_page_break() != crate::model::table::TablePageBreak::RowBreak
            || !self.indexed_stored_row_boxes_exceed_object_frame(table)
            || normalization.table_frame_reflowed_without_cell_scope(table)
            || normalization.nested_table_has_reflowed_host(table)
        {
            return None;
        }
        drop(normalization);
        let rows = self.stored_frame_row_plans.borrow();
        let rows = rows.get(&(table as *const Table as usize))?;
        let mut prefix = 0.0;
        let declared = hwpunit_to_px(table.common.height as i32, self.dpi);
        let spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        for (index, height) in rows.iter().enumerate().take(rows.len().saturating_sub(1)) {
            // 개별 행을 정수로 절삭한 뒤 더하면 합성된 저장 줄 높이에서 1HU씩
            // 잃을 수 있다. 실제 물리 합과 선언을 같은 좌표계에서 정확히 비교한다.
            prefix += *height + if index > 0 { spacing } else { 0.0 };
            if prefix.to_bits() != declared.to_bits() {
                continue;
            }
            let boundary = index + 1;
            // 원시 선언으로 이미 닫혔거나 정규화로 축소한 상자는 기존 소비자가 소유한다.
            // 이 보완은 원본 글줄로 행이 성장하여 원시 합으로는 찾지 못한 경계만
            // 인증한다. 기존 경계에 새 강제 쪽 나눔 의미를 중복 부여하지 않는다.
            let raw_rows = table.get_raw_row_heights();
            if raw_rows.get(..boundary).is_some_and(|prefix| {
                prefix
                    .iter()
                    .all(|&height| height > 0 && height <= i32::MAX as u32)
                    && prefix.iter().map(|&height| i64::from(height)).sum::<i64>()
                        + i64::from(table.cell_spacing) * index as i64
                        >= i64::from(table.common.height)
            }) {
                return None;
            }
            // 첫 상자를 닫는 원본 행만 인증한다. 뒤에서 새로 조판한 내용이나
            // 경계를 가로지르는 병합 칸은 이 원점을 증명하지 못한다.
            let original = table
                .cells
                .iter()
                .filter(|cell| usize::from(cell.row) < boundary)
                .all(|cell| {
                    usize::from(cell.row) + usize::from(cell.row_span.max(1)) <= boundary
                        && self.stored_plain_text_cell_frame(table, cell)
                });
            return original.then_some(boundary);
        }
        None
    }

    /// 선언 첫 개체 프레임이 단일 저장 글줄 뒤의 빈 행 공간 안에서 닫히면
    /// 내용과 남은 물리 띠를 별도 커서로 보존한다. 줄을 복제하거나 새 줄로 만들지 않는다.
    /// 부모의 중첩 행 투영과 자식 부분 배치가 같은 두 유닛을 사용한다.
    pub(super) fn preserve_stored_blank_opening_frame(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &mut Vec<CellUnit>,
    ) {
        let Some((opening, remainder)) =
            self.stored_blank_opening_frame_heights(cell, table, styles, units)
        else {
            return;
        };
        let mut tail = units[0].clone();
        units[0].height = opening;
        units[0].following_line_advance = 0.0;
        tail.height = remainder;
        // 원본 글줄은 앞 유닛만 소유한다. 뒤 유닛은 저장 개체의 빈 물리 프레임이다.
        tail.vis_start = tail.vis_end;
        tail.empty_spacer = true;
        tail.following_line_advance = 0.0;
        tail.hard_break_before = true;
        tail.stored_frame_break_before = true;
        tail.page_frame_reset_before = true;
        units.push(tail);
    }

    fn stored_blank_opening_frame_heights(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &[CellUnit],
    ) -> Option<(f64, f64)> {
        let [unit] = units else { return None };
        let [paragraph] = cell.paragraphs.as_slice() else {
            return None;
        };
        let [line] = paragraph.line_segs.as_slice() else {
            return None;
        };
        let row = usize::from(cell.row);
        if !self.native_stored_table_has_fragment_frames(table)
            || table.common.height >= 0x8000_0000
            || cell.dirty_flag
            || cell.row_span != 1
            || cell.vertical_align != VerticalAlign::Top
            || cell.text_direction != 0
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.vertical_pos != 0
            || line.line_height <= 0
            || line.line_spacing < 0
            || unit.nested_row.is_some()
            || unit.vis_start != 0
            || unit.vis_end != 1
        {
            return None;
        }
        // 현재 물리 행 원점을 이미 계산했다면 그 계획을 읽는다. 아직 조회하지 않은
        // 순수 글줄 표는 같은 행 해석기로 준비한다. 중첩 개체 표는 위에서 제외한다.
        let cached_rows = {
            self.stored_frame_row_plans
                .borrow()
                .get(&(table as *const Table as usize))
                .cloned()
        };
        let has_independent_object = || {
            table.cells.iter().any(|other| {
                other.paragraphs.iter().any(|paragraph| {
                    paragraph.controls.iter().any(|control| {
                        !crate::renderer::composer::control_is_width_neutral_marker(control)
                    })
                })
            })
        };
        if cached_rows.is_none() && has_independent_object() {
            return None;
        }
        let rows = cached_rows.unwrap_or_else(|| {
            self.resolve_row_heights(
                table,
                usize::from(table.col_count),
                usize::from(table.row_count),
                None,
                styles,
                true,
            )
        });
        let height = *rows.get(row)?;
        let before = rows[..row].iter().sum::<f64>()
            + hwpunit_to_px(i32::from(table.cell_spacing), self.dpi) * row as f64;
        let opening = hwpunit_to_px(table.common.height as i32, self.dpi) - before;
        let epsilon = hwpunit_to_px(2, self.dpi);
        let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        let content_opening = opening - top - bottom;
        let remainder = height - opening;
        if !height.is_finite()
            || content_opening <= unit.height + epsilon
            || remainder <= epsilon
            || self
                .render_normalization
                .borrow()
                .row_frame_reflowed(table, row)
        {
            return None;
        }
        // 첫 프레임과 교차하지 않는 일반 행에서는 모든 셀을 순회하지 않는다.
        // 후보 행에서만 원본 개체와 병합 셀의 독립 소유 구조를 확인한다.
        if has_independent_object()
            || table.cells.iter().any(|other| {
                let first = usize::from(other.row);
                other.row_span > 1 && first <= row && row < first + usize::from(other.row_span)
            })
        {
            return None;
        }
        // 같은 행의 모든 칸이 첫 프레임 안에 잉크를 닫아야 빈 띠로 증명된다.
        // 여러 저장 글줄·가운데 정렬·별도 개체의 컷은 기존 생산자에 남긴다.
        for peer in table.cells.iter().filter(|peer| peer.row == cell.row) {
            let [p] = peer.paragraphs.as_slice() else {
                return None;
            };
            let [s] = p.line_segs.as_slice() else {
                return None;
            };
            let padding = peer.effective_padding(&table.padding);
            let end = hwpunit_to_px(
                s.line_height
                    .checked_add(i32::from(padding.top))?
                    .checked_add(i32::from(padding.bottom))?,
                self.dpi,
            );
            if peer.dirty_flag
                || peer.text_direction != 0
                || peer.vertical_align != VerticalAlign::Top
                || p.stored_text_partition_is_dirty()
                || p.cell_format_vpos_dirty
                || s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || s.vertical_pos != 0
                || s.line_height <= 0
                || end >= opening - epsilon
            {
                return None;
            }
        }
        Some((content_opening, remainder))
    }

    pub(super) fn stored_page_frame_for_unit(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        padding: f64,
    ) -> Option<StoredTextPageFrame> {
        if !self.stored_span_cut_preserves_line_slots(table, cell)
            || (hwpunit_to_px(cell.effective_vertical_padding_hu(&table.padding), self.dpi)
                - padding)
                .abs()
                > hwpunit_to_px(1, self.dpi)
        {
            return None;
        }
        let first = units.get(start)?;
        if first.vis_start != 0 || start > 0 && units[start - 1].para_idx == first.para_idx {
            return None;
        }
        stored_text_page_frame(cell, table, first.para_idx)
    }

    pub(super) fn preserve_source_page_frame_extents(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &mut [CellUnit],
    ) {
        if !self.stored_span_cut_preserves_line_slots(table, cell) {
            return;
        }
        let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        for start in 0..units.len() {
            let Some(frame) =
                self.stored_page_frame_for_unit(cell, table, units, start, top + bottom)
            else {
                continue;
            };
            let end = units[start..]
                .iter()
                .position(|unit| unit.para_idx >= frame.end_paragraph)
                .map_or(units.len(), |relative| start + relative);
            if end <= start || units[end - 1].para_idx + 1 != frame.end_paragraph {
                continue;
            }
            let advance = units[start..end]
                .iter()
                .map(|unit| unit.height)
                .sum::<f64>()
                - self.stored_span_line_tail_spacing(&units[end - 1]);
            let required = hwpunit_to_px(frame.content_height_hu, self.dpi);
            if required > advance {
                // 실제 마지막 글줄·가시 범위는 유지하고, 그 뒤 원본 빈 점유만 전진한다.
                // 일반 행과 블록 컷의 높이 walk가 같은 캐시된 유닛을 읽는다.
                units[end - 1].height += required - advance;
            }
        }
    }
}
