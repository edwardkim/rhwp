//! 저장 중첩 래퍼의 선행 묶음은 같은 원본 물리 프레임에서만 유지한다.
//!
//! 현재 글줄로 재조판한 행은 새 CellUnit 컷을 소유한다. 저장 당시의
//! 원자적 prefix를 뒤에서 다시 적용해 실제 수용한 첫 줄을 이월하지 않는다.
use super::*;

impl LayoutEngine {
    fn stored_wrapper_prefix_is_source_owned(
        &self,
        table: &crate::model::table::Table,
        row: usize,
    ) -> bool {
        self.profile.get().hwp5_stored_pagination_layout()
            && !table.common.treat_as_char
            && matches!(table.layout_page_break(), TablePageBreak::RowBreak)
            && !self.row_uses_reflow_physical_frame(table, row)
    }

    /// 새 physical page에 온전히 들어갈 1×1 중첩 표 래퍼의 선행 문단 묶음을
    /// 현재 쪽 끝에 고립시키지 않아야 하는지 판정한다.
    ///
    /// Native HWP5 `RowBreak` 표에는 비어 있는 outer host → 1×1 child table →
    /// child 본문/내부 표라는 저장 형상이 있다. outer 행의 잔여 공간에 child 본문의
    /// 앞 몇 줄만 소비하면, scalar child renderer는 그 줄을 현재 쪽에 칠하지만 한컴은
    /// 그 다음 내부 표 앞의 묶음을 새 쪽에서 함께 시작한다. 86712의 r27이 그 사례다.
    ///
    /// 이 규칙은 일반 문단/중첩 표에 적용하지 않는다. 다음 내부 표 source paragraph
    /// 직전의 mixed-unit 경계를 provenance로 식별하고, 그 전 묶음이 새 본문의 대부분을
    /// 채울 때만 true다. 단순히 새 쪽에 들어간다는 이유로 중간 길이의 도입부까지
    /// 이월하면 59043 p35처럼 PDF가 허용한 분할을 망가뜨린다. 따라서 실제로 한
    /// 페이지보다 긴 child나 이미 continuation인 행, 새 본문의 일부만 쓰는 prefix는
    /// 종전 CellUnit 분할을 유지한다.
    pub(crate) fn should_defer_fresh_rowbreak_wrapper_prefix(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        fresh_body_height: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !start_cut.is_empty() || !self.stored_wrapper_prefix_is_source_owned(table, row) {
            return false;
        }

        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);

        for (wrapper_index, wrapper_cell) in row_cells.iter().enumerate() {
            let Some(host) = wrapper_cell
                .paragraphs
                .iter()
                .find(|paragraph| Self::paragraph_hosts_single_cell_nested_table(paragraph))
            else {
                continue;
            };
            let Some(child) = host.controls.iter().find_map(|control| match control {
                Control::Table(table) if table.row_count == 1 && table.col_count == 1 => {
                    Some(table.as_ref())
                }
                _ => None,
            }) else {
                continue;
            };
            let Some(child_cell) = child.cells.first() else {
                continue;
            };
            let Some(first_inner_table_para) = child_cell.paragraphs.iter().position(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Table(_)))
            }) else {
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} child_paras={} inner_table_para=none",
                        row,
                        wrapper_cell.col,
                        child_cell.paragraphs.len(),
                    );
                }
                continue;
            };

            let wrapper_units = self.cell_units(wrapper_cell, table, styles);
            let Some(inner_table_start) = wrapper_units.iter().position(|unit| {
                unit.mixed_nested_fragment
                    && !unit.mixed_nested_trailing
                    && unit.mixed_nested_source_para_idx == Some(first_inner_table_para)
            }) else {
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} child_inner_para={} units={} mixed_sources={:?}",
                        row,
                        wrapper_cell.col,
                        first_inner_table_para,
                        wrapper_units.len(),
                        wrapper_units
                            .iter()
                            .filter(|unit| unit.mixed_nested_fragment && !unit.mixed_nested_trailing)
                            .filter_map(|unit| unit.mixed_nested_source_para_idx)
                            .collect::<Vec<_>>(),
                    );
                }
                continue;
            };
            if inner_table_start == 0 {
                continue;
            }

            let partial_end = end_cut
                .get(wrapper_index)
                .copied()
                .unwrap_or(wrapper_units.len())
                .min(wrapper_units.len());
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} inner_start={} partial_end={} start_cut={:?} end_cut={:?}",
                    row, wrapper_cell.col, inner_table_start, partial_end, start_cut, end_cut,
                );
            }
            if partial_end == 0 || partial_end >= inner_table_start {
                continue;
            }

            // 형제 라벨 셀도 선행 묶음에 포함돼야 한다. 현재 조각에서 아직 남은
            // 형제 content가 있으면 이월이 행 구조를 바꾸므로 적용하지 않는다.
            if row_cells.iter().enumerate().any(|(index, cell)| {
                if index == wrapper_index {
                    return false;
                }
                let units = self.cell_units(cell, table, styles);
                end_cut.get(index).copied().unwrap_or(units.len()) < units.len()
            }) {
                continue;
            }

            let mut prefix_end = Vec::with_capacity(row_cells.len());
            for (index, cell) in row_cells.iter().enumerate() {
                if index == wrapper_index {
                    prefix_end.push(inner_table_start);
                } else {
                    prefix_end.push(self.cell_units(cell, table, styles).len());
                }
            }
            let prefix_height = self.row_cut_content_height(table, row, &[], &prefix_end, styles);
            // 이 helper는 scan 중 호출돼 `LayoutEngine::current_body_area`가 아직
            // 갱신되지 않은 경로가 있다. 현재 `TypesetState`가 보유한 fresh 본문
            // 높이를 호출자가 넘겨야 실제 다음 페이지 수용성을 판정할 수 있다.
            let body_height = fresh_body_height;
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN DEFER_WRAPPER_PREFIX_FIT r={} prefix={:.1} body={:.1} prefix_end={:?}",
                    row, prefix_height, body_height, prefix_end,
                );
            }
            // 86712 r27의 926.2 / 971.3px처럼 사실상 한 page를 이루는 prefix만
            // atomic start로 다룬다. 59043 r5의 569.1 / 971.3px 도입부는 PDF에서
            // 앞 page에 남아야 하므로 이 임계값 아래에서는 기존 분할을 보존한다.
            const NEAR_FULL_FRESH_BODY_RATIO: f64 = 0.80;
            if body_height > 0.0
                && prefix_height <= body_height + 0.5
                && prefix_height >= body_height * NEAR_FULL_FRESH_BODY_RATIO
            {
                return true;
            }
        }

        false
    }

    /// Fresh 1×1 wrapper fragment가 첫 내부 표를 paint하지 않는 마지막 RowCut을
    /// 반환한다.
    ///
    /// mixed projection에서 `end_cut`은 scalar child renderer에 inclusive owner로
    /// 전달된다. 따라서 첫 inner-table atom(예: unit 59) 바로 전 spacer(unit 58)를
    /// end로 넘겨도 표가 현재 fragment에 그려질 수 있다. 표 source와 그 선행 spacer
    /// 둘 다 다음 page로 넘기는 `table_atom - 2`가 안전한 경계다. 이 값은 source
    /// paragraph provenance로만 구하며 native HWP5 non-TAC RowBreak wrapper에만 적용한다.
    pub(crate) fn fresh_rowbreak_wrapper_safe_prefix_end_cut(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<RowCut> {
        if !start_cut.is_empty() || !self.stored_wrapper_prefix_is_source_owned(table, row) {
            return None;
        }
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);

        for (wrapper_index, wrapper_cell) in row_cells.iter().enumerate() {
            let Some(child) = wrapper_cell.paragraphs.iter().find_map(|paragraph| {
                if !Self::paragraph_hosts_single_cell_nested_table(paragraph) {
                    return None;
                }
                paragraph.controls.iter().find_map(|control| match control {
                    Control::Table(table) if table.row_count == 1 && table.col_count == 1 => {
                        Some(table.as_ref())
                    }
                    _ => None,
                })
            }) else {
                continue;
            };
            let Some(first_inner_table_para) = child.cells.first().and_then(|cell| {
                cell.paragraphs.iter().position(|paragraph| {
                    paragraph
                        .controls
                        .iter()
                        .any(|control| matches!(control, Control::Table(_)))
                })
            }) else {
                continue;
            };
            let units = self.cell_units(wrapper_cell, table, styles);
            let Some(first_table_atom) = units.iter().position(|unit| {
                unit.mixed_nested_fragment
                    && !unit.mixed_nested_trailing
                    && unit.mixed_nested_source_para_idx == Some(first_inner_table_para)
            }) else {
                continue;
            };
            let current_end = end_cut
                .get(wrapper_index)
                .copied()
                .unwrap_or(units.len())
                .min(units.len());
            if current_end != first_table_atom || first_table_atom < 2 {
                continue;
            }
            if row_cells.iter().enumerate().any(|(index, cell)| {
                index != wrapper_index
                    && end_cut
                        .get(index)
                        .copied()
                        .unwrap_or_else(|| self.cell_units(cell, table, styles).len())
                        < self.cell_units(cell, table, styles).len()
            }) {
                continue;
            }

            let mut safe_end_cut = end_cut.to_vec();
            safe_end_cut[wrapper_index] = first_table_atom - 2;
            return Some(safe_end_cut);
        }
        None
    }
}
