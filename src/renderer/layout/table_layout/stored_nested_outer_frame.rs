//! 실제 원본 자식 컷의 바깥 프레임을 상위 예산과 부분 배치에 전달한다.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    pub(crate) fn stored_hwpx_cut_owns_outer_frame(&self, host: &Paragraph, table: &Table) -> bool {
        let profile = self.profile.get();
        crate::renderer::float_placement::stored_hwpx_cut_owns_outer_frame(
            (profile.hwp5_stored_pagination_layout()
                || profile.hwpx_stored_layout() && !profile.hwp5_origin_hwpx())
                && !profile.session_edited(),
            host,
            table,
        )
    }

    /// 첫·마지막 원본 유닛의 여백은 중복 예약하지 않고 중간 컷이 여는 몫만 보충한다.
    pub(super) fn stored_nested_cut_outer_margin_extra(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &[CellUnit],
        lo: usize,
        hi: usize,
    ) -> f64 {
        let mut extra = 0.0;
        let mut run_start = 0;
        while run_start < units.len() {
            let para_index = units[run_start].para_idx;
            let mut run_end = run_start + 1;
            while run_end < units.len() && units[run_end].para_idx == para_index {
                run_end += 1;
            }
            let start = lo.max(run_start);
            let end = hi.min(run_end);
            // 원본의 마지막 호스트 줄이 자식 종료를 인증하면 그 줄의 원점은
            // 종료 자식 상자와 위·아래 바깥여백을 이미 소유한다. 이어받기라는
            // 이유로 위 여백을 다시 예약하면 부모 끝과 뒤 본문만 아래로 밀린다.
            // 별도 후행 공백 줄은 같은 종료 프레임 인증이 아니다.
            let closes_stored_host_frame = end == run_end
                && para_index + 1 == cell.paragraphs.len()
                && units[run_end - 1].nested_host_closing_line_height > 0.0
                && !units[run_end - 1].host_trailing_spacing_charged;
            if start < end
                && !closes_stored_host_frame
                && !cell.dirty_flag
                && units[run_start..run_end]
                    .iter()
                    .all(|unit| unit.nested_row.is_some() || unit.mixed_nested_recursive)
            {
                if let Some((host, child)) = cell.paragraphs.get(para_index).and_then(|host| {
                    host.controls.iter().find_map(|control| match control {
                        Control::Table(child) => Some((host, child.as_ref())),
                        _ => None,
                    })
                }) {
                    if self.stored_hwpx_cut_owns_outer_frame(host, child) {
                        // 저장 자식 프레임의 흐름 높이는 이미 양쪽 바깥 여백을
                        // 포함한다. 같은 원본 컷에 여백을 다시 예약하지 않는다.
                        let saved_frame_owns_margin = (start > run_start || end < run_end)
                            && self
                                .mixed_nested_split_from_cut(
                                    cell, table, styles, lo, hi, para_index,
                                )
                                .and_then(|split| split.recursive_cut)
                                .filter(|cut| {
                                    cut.start_row == 0
                                        && cut.end_row == 1
                                        && !cut.is_block_split
                                        && !cut.start_cut_is_block
                                })
                                .and_then(|cut| {
                                    self.saved_nested_single_cell_frame_height(
                                        child,
                                        &cut.start_cut,
                                        &cut.end_cut,
                                        styles,
                                    )
                                })
                                .is_some();
                        if saved_frame_owns_margin {
                            run_start = run_end;
                            continue;
                        }
                        if start > run_start {
                            extra += hwpunit_to_px(child.outer_margin_top as i32, self.dpi);
                        }
                        if end < run_end {
                            extra += hwpunit_to_px(child.outer_margin_bottom as i32, self.dpi);
                        }
                    }
                }
            }
            run_start = run_end;
        }
        extra
    }
}
