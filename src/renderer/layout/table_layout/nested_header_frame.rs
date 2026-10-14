//! 중첩 표 이어받기의 반복 제목행을 같은 높이로 예약하고 배치한다.

use crate::model::control::Control;
use crate::model::table::Table;

use super::{LayoutEngine, NestedTableCut, NestedTableSplit, ResolvedStyleSet};

pub(in crate::renderer::layout) struct NestedHeaderFrame {
    pub(in crate::renderer::layout) rows: Vec<usize>,
    pub(in crate::renderer::layout) height: f64,
}

impl LayoutEngine {
    pub(in crate::renderer::layout) fn nested_header_frame(
        &self,
        table: &Table,
        start_row: usize,
        row_heights: &[f64],
    ) -> NestedHeaderFrame {
        let rows: Vec<_> = if table.repeat_header && start_row > 0 {
            table
                .leading_header_rows()
                .into_iter()
                .filter(|row| *row < start_row)
                .collect()
        } else {
            Vec::new()
        };
        let spacing = crate::renderer::hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let height = rows
            .iter()
            .map(|row| row_heights.get(*row).copied().unwrap_or(0.0) + spacing)
            .sum();
        NestedHeaderFrame { rows, height }
    }

    fn resolved_nested_header_height(
        &self,
        table: &Table,
        start_row: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        if !table.repeat_header || start_row == 0 || table.leading_header_rows().is_empty() {
            return 0.0;
        }
        let heights = self.resolve_row_heights(
            table,
            usize::from(table.col_count),
            usize::from(table.row_count),
            None,
            styles,
            true,
        );
        self.nested_header_frame(table, start_row, &heights).height
    }

    /// 행 원자로 투영한 자식도 새 조각의 제목행 공간을 소비한다.
    pub(super) fn atomic_nested_header_reservation(
        &self,
        cell: &crate::model::table::Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> f64 {
        let units = self.cell_units(cell, table, styles);
        let mut previous = None;
        let mut height = 0.0;
        for unit in units.iter().take(end).skip(start) {
            // 실제 자식 컷을 가진 유닛의 제목행은 continuation 높이와
            // 자식 RowCut 상자에서 이미 예약한다. 행 원자의 추가 예약과
            // 중복하면 내용 컷이 실제 프레임보다 먼저 끝난다.
            if unit.nested_table_fragment.is_some() {
                continue;
            }
            if previous == Some(unit.para_idx) {
                continue;
            }
            previous = Some(unit.para_idx);
            let Some(row) = unit.nested_row else {
                continue;
            };
            if let Some(child) = cell.paragraphs.get(unit.para_idx).and_then(|paragraph| {
                paragraph.controls.iter().find_map(|control| match control {
                    Control::Table(child) => Some(child.as_ref()),
                    _ => None,
                })
            }) {
                height += self.resolved_nested_header_height(child, row, styles);
            }
        }
        height
    }

    /// 자식의 예약 공간을 상위 흐름에 전달한다. 열은 병렬, 행과 문단은 순서대로 점유한다.
    fn nested_header_reservation_for_cut(
        &self,
        table: &Table,
        cut: &NestedTableCut,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut height = self.resolved_nested_header_height(table, cut.start_row, styles);
        // block cut은 셀 유닛 인덱스가 아니다. 해당 행의 제목행은 예약할 수 있지만,
        // 그 인덱스로 자식의 시작 위치를 추측하지 않는다.
        if cut.start_cut_is_block || cut.is_block_split {
            return height;
        }
        for row in cut.start_row..cut.end_row.min(usize::from(table.row_count)) {
            let cells = Self::row_cut_cell_order(table, row);
            let mut row_height = 0.0f64;
            for (index, cell_index) in cells.into_iter().enumerate() {
                let Some(cell) = table.cells.get(cell_index) else {
                    continue;
                };
                let units = self.cell_units(cell, table, styles);
                let start = if row == cut.start_row {
                    cut.start_cut.get(index).copied().unwrap_or(0)
                } else {
                    0
                }
                .min(units.len());
                let end = if row + 1 == cut.end_row {
                    cut.end_cut.get(index).copied().unwrap_or(units.len())
                } else {
                    units.len()
                }
                .clamp(start, units.len());
                let mut cell_height =
                    self.atomic_nested_header_reservation(cell, table, styles, start, end);
                let mut previous = None;
                for unit in units.iter().take(end).skip(start) {
                    if !unit.mixed_nested_recursive || previous == Some(unit.para_idx) {
                        continue;
                    }
                    previous = Some(unit.para_idx);
                    let child = cell.paragraphs.get(unit.para_idx).and_then(|paragraph| {
                        paragraph.controls.iter().find_map(|control| match control {
                            Control::Table(child) => Some(child.as_ref()),
                            _ => None,
                        })
                    });
                    if let Some((child, child_cut)) = child.zip(
                        self.mixed_nested_split_from_cut(
                            cell,
                            table,
                            styles,
                            start,
                            end,
                            unit.para_idx,
                        )
                        .and_then(|split| split.recursive_cut),
                    ) {
                        cell_height +=
                            self.nested_header_reservation_for_cut(child, &child_cut, styles);
                    }
                }
                row_height = row_height.max(cell_height);
            }
            height += row_height;
        }
        height
    }

    pub(in crate::renderer::layout) fn nested_split_header_reservation(
        &self,
        table: &Table,
        split: &NestedTableSplit,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        if let Some(cut) = split.recursive_cut.as_ref() {
            self.nested_header_reservation_for_cut(table, cut, styles)
        } else {
            self.resolved_nested_header_height(table, split.start_row, styles)
        }
    }

    pub(in crate::renderer::layout) fn with_nested_header_reservation(
        &self,
        table: &Table,
        mut split: NestedTableSplit,
        styles: &ResolvedStyleSet,
    ) -> NestedTableSplit {
        let height = self.nested_split_header_reservation(table, &split, styles);
        split.flow_height += height;
        if split.recursive_cut.is_none() {
            // 재귀 컷의 상자는 부분 표 경로가 직접 측정한다. scalar 행 창만 같은 높이로 늘린다.
            split.visible_height += height;
        }
        split
    }
}
