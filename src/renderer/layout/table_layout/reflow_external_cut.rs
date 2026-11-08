//! 독립 자식의 소스 컷과 부모 물리 프레임의 서로 다른 소유.

use super::{row_physical_band::CutHeight, LayoutEngine, ResolvedStyleSet, RowCutResult};
use crate::model::{control::Control, table::Table};
use crate::renderer::{float_placement::signed_hwpunit, hwpunit_to_px};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct ReflowExternalCutFrame {
    pub(in crate::renderer) parent_owned: bool,
    pub(in crate::renderer) external_content_owned: bool,
    pub(in crate::renderer) parent_height: f64,
    /// 호출자의 host 바깥여백을 제외한 현재 조각의 예약 높이.
    pub(in crate::renderer) occupied_height: f64,
}

impl LayoutEngine {
    pub(in crate::renderer) fn reflow_unrestricted_fragment_frame(
        &self,
        table: &Table,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<ReflowExternalCutFrame> {
        self.reflow_external_cut_frame(table, start, end, styles, CutHeight::Occupied)
    }

    pub(super) fn reflow_external_cut_frame(
        &self,
        table: &Table,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
        purpose: CutHeight,
    ) -> Option<ReflowExternalCutFrame> {
        let full = self.reflow_unrestricted_table_flow(table, styles)?;
        let body_height = self.current_body_area.get().3;
        // 부모 자체가 여러 쪽을 소유하면 그 기존 physical row 원장을 사용한다.
        if body_height <= 0.0 || full.parent_frame_height > body_height {
            return None;
        }
        let parent_owned = start.iter().all(|&cut| cut == 0);
        let outside = hwpunit_to_px(
            i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
            self.dpi,
        );
        let offset = if parent_owned {
            hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), self.dpi)
        } else {
            0.0
        };
        let parent_height = if parent_owned {
            full.parent_frame_height
        } else {
            0.0
        };
        let host_outside = if parent_owned { outside } else { 0.0 };
        let mut root_bottom = parent_height + host_outside + offset;
        let mut external_content_owned = false;
        for (slot, index) in self
            .indexed_row_cut_cell_order(table, 0)
            .into_iter()
            .enumerate()
        {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            let first = start.get(slot).copied().unwrap_or(0).min(units.len());
            let last = end
                .get(slot)
                .copied()
                .unwrap_or(units.len())
                .clamp(first, units.len());
            if first == last {
                continue;
            }
            for (pi, para) in cell.paragraphs.iter().enumerate() {
                let Some(Control::Table(child)) = para
                    .controls
                    .iter()
                    .find(|control| matches!(control, Control::Table(_)))
                else {
                    continue;
                };
                if !units[first..last].iter().any(|unit| unit.para_idx == pi) {
                    continue;
                }
                external_content_owned = true;
                let body = if let Some(cut) = self
                    .nested_table_split_from_cut_units(cell, table, styles, first, last, pi)
                    .and_then(|split| split.recursive_cut)
                {
                    self.nested_cut_height(child, &cut, styles, purpose)
                } else if let Some((start_row, end_row)) =
                    self.nested_row_range_from_cut_units(cell, table, styles, first, last, pi)
                {
                    self.nested_cut_height(
                        child,
                        &super::NestedTableCut {
                            start_row,
                            end_row,
                            start_cut: Vec::new(),
                            end_cut: Vec::new(),
                            is_block_split: false,
                            start_cut_is_block: false,
                        },
                        styles,
                        purpose,
                    )
                } else {
                    self.nested_table_body_height(child, styles)
                };
                let child_offset = if parent_owned {
                    hwpunit_to_px(signed_hwpunit(child.common.vertical_offset), self.dpi)
                } else {
                    0.0
                };
                let child_bottom = child_offset
                    + body
                    + hwpunit_to_px(
                        i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
                        self.dpi,
                    );
                root_bottom = root_bottom.max(child_bottom);
            }
        }
        Some(ReflowExternalCutFrame {
            parent_owned,
            external_content_owned,
            parent_height,
            occupied_height: root_bottom - host_outside - offset,
        })
    }

    /// 같은 컷 query로 수용을 결정한다. 부모 줄 prefix를 독립 자식 높이에 더하지 않는다.
    pub(super) fn advance_reflow_external_row_cut(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<RowCutResult> {
        if row != 0
            || !self
                .reflow_unrestricted_table_flow(table, styles)?
                .child_splittable
        {
            return None;
        }
        let cells = self.indexed_row_cut_cell_order(table, row);
        let mut end: Vec<_> = cells
            .iter()
            .enumerate()
            .map(|(slot, &index)| {
                start
                    .get(slot)
                    .copied()
                    .unwrap_or(0)
                    .min(self.cell_units(&table.cells[index], table, styles).len())
            })
            .collect();
        self.reflow_unrestricted_fragment_frame(table, start, &end, styles)?;
        for (slot, &index) in cells.iter().enumerate() {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            // 저장된 강제 경계와 혼합 개체의 소유는 기존 일반 스캐너가 유지한다.
            if units.iter().any(|unit| {
                unit.hard_break_before
                    || unit.stored_frame_break_before
                    || unit.page_frame_reset_before
                    || unit.mixed_nested_fragment
            }) {
                return None;
            }
            let first = end[slot];
            while end[slot] < units.len() {
                let before = end[slot];
                end[slot] += 1;
                let required = self
                    .reflow_external_cut_frame(table, start, &end, styles, CutHeight::Capacity)?
                    .occupied_height;
                // 첫 source 유닛의 진행 보장은 일반 컷 경로와 같다.
                if before > first && required > available + super::ROW_CUT_CAPACITY_FP_EPSILON_PX {
                    end[slot] = before;
                    break;
                }
            }
        }
        let frame = self.reflow_unrestricted_fragment_frame(table, start, &end, styles)?;
        let fully_consumed = cells.iter().enumerate().all(|(slot, &index)| {
            end[slot] == self.cell_units(&table.cells[index], table, styles).len()
        });
        Some(RowCutResult {
            end_cut: end,
            hit_hard_break: false,
            fully_consumed,
            consumed_height: frame.occupied_height,
            physical_height: Some(frame.occupied_height),
        })
    }
}
