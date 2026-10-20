//! 실제 원본 자식 컷의 바깥 프레임을 상위 예산과 부분 배치에 전달한다.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{CellUnit, LayoutEngine};

impl LayoutEngine {
    pub(crate) fn stored_hwpx_cut_owns_outer_frame(&self, host: &Paragraph, table: &Table) -> bool {
        let profile = self.profile.get();
        crate::renderer::float_placement::stored_hwpx_cut_owns_outer_frame(
            profile.hwpx_stored_layout()
                && !profile.hwp5_origin_hwpx()
                && !profile.session_edited(),
            host,
            table,
        )
    }

    /// 첫·마지막 원본 유닛의 여백은 중복 예약하지 않고 중간 컷이 여는 몫만 보충한다.
    pub(super) fn stored_nested_cut_outer_margin_extra(
        &self,
        cell: &Cell,
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
            if start < end
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
