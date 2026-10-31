//! 원본 소유의 예약 높이와 완료 셀의 물리 테두리 높이를 함께 발행한다.

use super::{HeightMeasurer, MeasuredCell, MeasuredTable, ResolvedStyleSet, Table};
use crate::renderer::{float_placement::ReflowExternalTableFlow, hwpunit_to_px};

impl HeightMeasurer {
    pub(super) fn reflow_cell_frame_rows(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
        depth: usize,
        source_rows: &[f64],
        measured_cells: &[MeasuredCell],
    ) -> Option<Vec<f64>> {
        match self.reflow_cell_frame_rows_with_storage(
            table,
            styles,
            depth,
            source_rows,
            measured_cells,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(super) fn reflow_cell_frame_rows_with_storage<S: crate::storage::Storage>(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
        depth: usize,
        source_rows: &[f64],
        measured_cells: &[MeasuredCell],
        storage: &S,
    ) -> Result<Option<Vec<f64>>, S::Error> {
        crate::storage::with_optional(|failure| {
            if !table.cells.iter().any(|cell| {
                crate::renderer::float_placement::has_reflow_block_cell_host(&cell.paragraphs)
            }) {
                return None;
            }
            let mut frames = crate::storage::optional(storage.copy_slice(source_rows), failure)?;
            let mut resolved = false;
            for (row, height) in frames.iter_mut().enumerate() {
                // 걸침 칸의 물리 frame은 그 칸의 실제 조각 소유가 결정한다.
                if table.cells.iter().any(|cell| {
                    let first = usize::from(cell.row);
                    first <= row && row < first + usize::from(cell.row_span) && cell.row_span > 1
                }) {
                    continue;
                }
                let cells: Vec<_> = crate::storage::optional(
                    storage.collect(
                        table
                            .cells
                            .iter()
                            .filter(|cell| usize::from(cell.row) == row),
                    ),
                    failure,
                )?;
                if cells.is_empty() {
                    continue;
                }
                let mut has_reflow_host = false;
                crate::storage::optional(
                    storage.bytes(cells.len() * std::mem::size_of::<f64>()),
                    failure,
                )?;
                let known_frames: Option<Vec<_>> = cells
                    .into_iter()
                    .map(|cell| {
                        let measured = measured_cells.iter().find(|measured| {
                            measured.row == usize::from(cell.row)
                                && measured.col == usize::from(cell.col)
                        })?;
                        if let Some(flow) = crate::storage::optional(
                            self.cell_reflow_block_flow_with_storage(
                                cell,
                                styles,
                                depth,
                                measured.content_width,
                                storage,
                            ),
                            failure,
                        )? {
                            has_reflow_host = true;
                            return flow.parent_frame_height(cell, table, self.dpi);
                        }
                        // 실제로 측정한 일반 텍스트 형제 칸도 같은 부모 행을 소유한다.
                        // 외부 개체·세로쓰기·별도 자식의 높이를 plain 몫으로 추측하지 않는다.
                        if cell.text_direction != 0
                            || !cell
                                .paragraphs
                                .iter()
                                .all(crate::renderer::cell_paragraph_has_only_text_metadata)
                        {
                            return None;
                        }
                        Some(
                            hwpunit_to_px(i32::try_from(cell.height).ok()?, self.dpi).max(
                                measured.total_content_height
                                    + measured.padding_top
                                    + measured.padding_bottom,
                            ),
                        )
                    })
                    .collect();
                if let Some(known) = known_frames.filter(|_| has_reflow_host) {
                    *height = known.into_iter().fold(0.0_f64, f64::max);
                    resolved = true;
                }
            }
            resolved.then_some(frames)
        })
    }

    /// 자식의 물리 행과 그 밖으로 이어지는 자손의 점유를 재귀해서 구분한다.
    /// 배치도 같은 집계 함수를 사용하며 부모 prefix를 자식 끝에 합산하지 않는다.
    pub(super) fn measured_reflow_external_flow(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
        depth: usize,
        measured: &MeasuredTable,
    ) -> Option<ReflowExternalTableFlow> {
        match self.measured_reflow_external_flow_with_storage(
            table,
            styles,
            depth,
            measured,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(super) fn measured_reflow_external_flow_with_storage<S: crate::storage::Storage>(
        &self,
        table: &Table,
        styles: &ResolvedStyleSet,
        depth: usize,
        measured: &MeasuredTable,
        storage: &S,
    ) -> Result<Option<ReflowExternalTableFlow>, S::Error> {
        crate::storage::with_optional(|failure| {
            let frames = measured.row_frame_heights.as_ref()?;
            let parent_height =
                frames.iter().sum::<f64>() + measured.cell_spacing * (frames.len() + 1) as f64;
            crate::storage::optional(
                crate::renderer::float_placement::reflow_external_table_flow_with_storage(
                    table,
                    self.dpi,
                    parent_height,
                    |cell| {
                        crate::storage::with_optional(|failure| {
                            let width = measured
                                .cells
                                .iter()
                                .find(|measured| {
                                    measured.row == usize::from(cell.row)
                                        && measured.col == usize::from(cell.col)
                                })?
                                .content_width;
                            crate::storage::optional(
                                self.cell_reflow_block_flow_with_storage(
                                    cell, styles, depth, width, storage,
                                ),
                                failure,
                            )?
                        })
                    },
                    storage,
                ),
                failure,
            )?
        })
    }
}
