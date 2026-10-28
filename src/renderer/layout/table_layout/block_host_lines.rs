//! 수용한 표·뒤 공백 글줄의 동일한 유닛을 일반/부분 배치에서 소비한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::table::{Cell, Table};

impl LayoutEngine {
    /// 표 본체가 현재 밴드에 들어갈 때 뒤 공백 줄도 같은 밴드에서 수용한다.
    /// 본체부터 넘는 경우는 기존 내용 컷으로 나누며 페이지 크기로 원자화하지 않는다.
    pub(super) fn block_following_host_needs_fresh_band(
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        available: f64,
    ) -> bool {
        let Some(first) = units.get(start) else {
            return false;
        };
        if first.nested_row.is_none()
            || start
                .checked_sub(1)
                .is_some_and(|i| units[i].para_idx == first.para_idx)
            || !cell
                .paragraphs
                .get(first.para_idx)
                .is_some_and(crate::renderer::float_placement::whitespace_follows_block_table)
        {
            return false;
        }
        let owned = &units[start..];
        let end = owned
            .iter()
            .take_while(|u| u.para_idx == first.para_idx)
            .count();
        let Some(last) = owned.get(end.saturating_sub(1)) else {
            return false;
        };
        let closing = last.nested_host_closing_line_height;
        let height: f64 = owned[..end].iter().map(|u| u.height).sum();
        closing > 0.0 && height > available && height - closing <= available
    }

    /// 표와 뒤 공백 줄 전체를 수용한 컷은 그 유닛 원장을 그대로 소비한다.
    pub(in crate::renderer::layout) fn block_following_host_flow_height(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        para_index: usize,
        cut: Option<(usize, usize)>,
    ) -> Option<f64> {
        if !cell
            .paragraphs
            .get(para_index)
            .is_some_and(crate::renderer::float_placement::whitespace_follows_block_table)
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let first = units.partition_point(|unit| unit.para_idx < para_index);
        if units.get(first)?.para_idx != para_index {
            return None;
        }
        let end = first
            + units[first..]
                .iter()
                .take_while(|unit| unit.para_idx == para_index)
                .count();
        let (start_cut, end_cut) = cut.unwrap_or((0, units.len()));
        let owned = &units[first..end];
        (start_cut <= first
            && end <= end_cut
            && owned.iter().all(|unit| unit.nested_row.is_some())
            && owned.last()?.nested_host_closing_line_height > 0.0)
            .then(|| owned.iter().map(|unit| unit.height).sum())
    }

    /// 자식의 마지막 내용 컷과 함께 수용한 부모 공백 줄이다.
    /// 재귀 컷은 flow_height에 이미 포함하므로 scalar 컷의 소비자만 따로 읽는다.
    pub(in crate::renderer::layout) fn block_following_host_line_for_cut(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        para_index: usize,
        cut: Option<(usize, usize)>,
    ) -> f64 {
        if !cell
            .paragraphs
            .get(para_index)
            .is_some_and(crate::renderer::float_placement::whitespace_follows_block_table)
        {
            return 0.0;
        }
        let units = self.cell_units(cell, table, styles);
        let (start, end) = cut.unwrap_or((0, units.len()));
        units
            .iter()
            .take(end)
            .skip(start)
            .filter(|unit| unit.para_idx == para_index && unit.nested_row.is_some())
            .map(|unit| unit.nested_host_closing_line_height)
            .sum()
    }
}
