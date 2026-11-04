//! 같은 외부 호스트에 붙은 부모 프레임과 자손 개체의 전량 점유.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::table::Table;
use crate::renderer::float_placement::ReflowExternalTableFlow;

impl LayoutEngine {
    /// 측정기가 발행한 물리 행과 공통 자손 집계를 분할·전체 배치에 전달한다.
    pub(in crate::renderer) fn reflow_unrestricted_table_flow(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
    ) -> Option<ReflowExternalTableFlow> {
        if !crate::renderer::float_placement::supports_reflow_external_table_frame(table) {
            return None;
        }
        let measured = self.measured_reflow_block_table(table, styles)?;
        let frames = measured.row_frame_heights.as_ref()?;
        let parent_height =
            frames.iter().sum::<f64>() + measured.cell_spacing * (frames.len() + 1) as f64;
        crate::renderer::float_placement::reflow_external_table_flow(
            table,
            self.dpi,
            parent_height,
            |cell| self.reflow_block_cell_flow_in_cell(cell, table, styles),
        )
    }
}
