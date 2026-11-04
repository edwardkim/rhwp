//! 제한된 자식의 실제 소유 컷에서 호스트 내용 프레임을 재구성한다.

use super::{row_physical_band::CutHeight, CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    table::{Cell, Table},
};
use crate::renderer::hwpunit_to_px;

/// 문단의 실제 소스 유닛 범위와 선택한 컷이 함께 결정하는 여백 소유.
pub(in crate::renderer::layout) struct ReflowHostCutOwnership {
    pub(in crate::renderer::layout) owns_start: bool,
    pub(in crate::renderer::layout) owns_end: bool,
}

impl ReflowHostCutOwnership {
    pub(super) fn new(first: usize, last: usize, start: usize, end: usize) -> Option<Self> {
        (start < last && end > first).then_some(Self {
            owns_start: start <= first,
            owns_end: end >= last,
        })
    }
}

impl LayoutEngine {
    /// 셀 내부의 재귀 컷은 새 물리 viewport를 연다. 행 사이 컷과 블록 슬롯은
    /// 이 계약이 아니므로 부모 호스트의 첫·끝 여백 소유를 그대로 사용한다.
    pub(in crate::renderer::layout) fn reflow_cut_opens_outer_frame(
        cut: &super::NestedTableCut,
    ) -> bool {
        !cut.is_block_split
            && !cut.start_cut_is_block
            && cut
                .start_cut
                .iter()
                .chain(&cut.end_cut)
                .any(|&unit| unit > 0)
    }

    /// 행 범위로 접힌 컷도 원본 유닛의 셀 내부 viewport 소유를 보존한다.
    /// 중간 행을 온전히 소유한다는 사실만으로 이어받기 여백을 잃지 않는다.
    pub(in crate::renderer::layout) fn reflow_units_open_outer_frame(
        units: &[CellUnit],
        cut: Option<&super::NestedTableCut>,
    ) -> bool {
        cut.is_some_and(Self::reflow_cut_opens_outer_frame)
            || units.first().into_iter().chain(units.last()).any(|unit| {
                unit.nested_table_fragment.as_ref().is_some_and(|fragment| {
                    fragment
                        .start_cut
                        .iter()
                        .chain(&fragment.end_cut)
                        .any(|&consumed| consumed > 0)
                })
            })
    }

    /// 실제 배치도 측정과 같은 문단 유닛 범위로 처음·끝 여백의 소유를 정한다.
    pub(in crate::renderer::layout) fn reflow_host_cut_ownership(
        units: &[CellUnit],
        paragraph_index: usize,
        start: usize,
        end: usize,
    ) -> Option<ReflowHostCutOwnership> {
        let first = units
            .iter()
            .position(|unit| unit.para_idx == paragraph_index)?;
        let last = units
            .iter()
            .rposition(|unit| unit.para_idx == paragraph_index)?
            + 1;
        ReflowHostCutOwnership::new(first, last, start, end)
    }

    /// 실제 배치와 정렬은 수용한 같은 컷의 내용 프레임을 소비한다.
    pub(in crate::renderer::layout) fn reflow_partial_cell_content_height(
        &self,
        cell: &Cell,
        table: &Table,
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        // 여러 행을 소유한 칸의 프레임은 걸침 조각 원장이 결정한다.
        if cell.row_span != 1 {
            return None;
        }
        if self
            .reflow_unrestricted_table_flow(table, styles)
            .is_some_and(|flow| {
                self.current_body_area.get().3 > 0.0
                    && flow.parent_frame_height <= self.current_body_area.get().3
            })
        {
            return Some(if start == 0 {
                self.reflow_block_cell_flow_in_cell(cell, table, styles)?
                    .bottom
            } else {
                0.0
            });
        }
        let units = self.cell_units(cell, table, styles);
        let start = start.min(units.len());
        let end = end.clamp(start, units.len());
        self.reflow_cut_content_height(cell, table, &units, start, end, styles, CutHeight::Occupied)
    }

    /// 모든 칸의 같은 컷이 닫은 행 프레임은 paint의 꼬리 접기 대상이 아니다.
    pub(in crate::renderer::layout) fn reflow_partial_row_frame_height(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let cells = self.indexed_row_cut_cell_order(table, row);
        if cells.is_empty() {
            return None;
        }
        let mut height = 0.0_f64;
        for (slot, index) in cells.into_iter().enumerate() {
            let cell = &table.cells[index];
            let content = self.reflow_partial_cell_content_height(
                cell,
                table,
                start.get(slot).copied().unwrap_or(0),
                end.get(slot).copied().unwrap_or(usize::MAX),
                styles,
            )?;
            let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
            height = height.max(content + top + bottom);
        }
        Some(height)
    }

    /// 소스 유닛 주소는 유지하고 각 조각이 여는 여백과 닫는 여백을 함께 센다.
    /// 제한 없는 자식은 부모 밖의 독립 프레임이므로 포함 높이로 환산하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflow_cut_content_height(
        &self,
        cell: &Cell,
        table: &Table,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
        purpose: CutHeight,
    ) -> Option<f64> {
        if cell.text_direction != 0 {
            return None;
        }
        // 혼합 스트림은 자체 소스 투영과 물리 예약을 함께 발행한다.
        // mixed_nested_recursive도 그 투영의 속성이며 nested_row의 행 컷과
        // 같은 주소가 아니다. 기존 공통 컷 장부가 점유와 수용량을 소비하게
        // 하고 이 행 프레임으로 다시 해석하지 않는다.
        if units
            .get(start..end)?
            .iter()
            .any(|unit| unit.mixed_nested_fragment)
        {
            return None;
        }
        let flow = self.reflow_block_cell_flow_in_cell(cell, table, styles)?;
        if flow
            .origins
            .iter()
            .flatten()
            .flatten()
            .any(|o| o.host_relative)
        {
            return None;
        }
        if start == 0 && end == units.len() {
            return Some(flow.bottom);
        }
        let selected = units.get(start..end)?;
        if selected.is_empty() {
            return Some(0.0);
        }
        // 문단마다 전체 유닛을 다시 훑지 않고 원본 소유 범위를 한 번만 색인한다.
        let mut bounds = vec![None; cell.paragraphs.len()];
        for (index, unit) in units.iter().enumerate() {
            let slot = bounds.get_mut(unit.para_idx)?;
            let (first, _) = slot.unwrap_or((index, index));
            *slot = Some((first, index + 1));
        }
        let mut height = 0.0;
        for (pi, para) in cell.paragraphs.iter().enumerate() {
            let source = bounds[pi];
            let own = source.map_or(&units[0..0], |(first, last)| {
                let lo = first.max(start).min(last);
                let hi = last.min(end).max(lo);
                &units[lo..hi]
            });
            if own.is_empty() {
                // 현재 컷이 소비하지 않은 다음 호스트의 줄은 예약하지 않는다.
                // 그 줄은 호스트의 첫 소유 유닛과 함께 다음 조각에서 한 번 소비한다.
                continue;
            }
            let children: Vec<_> = para
                .controls
                .iter()
                .filter_map(|c| {
                    if let Control::Table(t) = c {
                        Some(t.as_ref())
                    } else {
                        None
                    }
                })
                .collect();
            if children.is_empty() {
                height += own.iter().map(|u| u.height).sum::<f64>();
                continue;
            }
            // 한 소스 유닛에 여러 자식이 섞이면 기존 원장이 개체 소유를 결정한다.
            let [child] = children.as_slice() else {
                return None;
            };
            let (first, last) = source?;
            let ReflowHostCutOwnership {
                owns_start,
                owns_end,
            } = ReflowHostCutOwnership::new(first, last, start, end)?;
            let recursive_cut = self
                .nested_table_split_from_cut_units(cell, table, styles, start, end, pi)
                .and_then(|s| s.recursive_cut);
            let repeats_outer_frame =
                Self::reflow_units_open_outer_frame(own, recursive_cut.as_ref());
            let body = if let Some(cut) = recursive_cut.as_ref() {
                self.nested_cut_height(child, cut, styles, purpose)
            } else if let Some((start_row, end_row)) =
                self.nested_row_range_from_cut_units(cell, table, styles, start, end, pi)
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
            let offset = if owns_start {
                hwpunit_to_px(child.common.vertical_offset as i32, self.dpi)
            } else {
                0.0
            };
            let object = offset
                + body
                + hwpunit_to_px(
                    if owns_start || repeats_outer_frame {
                        i32::from(child.outer_margin_top)
                    } else {
                        0
                    } + if owns_end || repeats_outer_frame {
                        i32::from(child.outer_margin_bottom)
                    } else {
                        0
                    },
                    self.dpi,
                );
            let last_para = pi + 1 == cell.paragraphs.len();
            let line = crate::renderer::float_placement::reflow_host_line_height(
                para, styles, self.dpi, last_para,
            )?;
            let style = styles.para_styles.get(para.para_shape_id as usize);
            let before = if owns_start {
                crate::renderer::cell_paragraph_spacing::paragraph_before(
                    para,
                    pi,
                    style.map_or(0.0, |s| s.spacing_before),
                    self.dpi,
                )
                .occupied_before()
            } else {
                0.0
            };
            let after = if owns_end && !last_para {
                style.map_or(0.0, |s| s.spacing_after)
            } else {
                0.0
            };
            height += before + line.max(object) + after;
        }
        Some(height)
    }
}
