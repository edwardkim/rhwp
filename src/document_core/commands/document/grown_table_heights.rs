//! 저장할 때 내용이 넘친 표의 높이(sz)를 실제 행 높이 합으로 적는다.
//!
//! 한컴은 칸 높이(cellSz)는 선언값 그대로 두고 표 높이만 키워 저장한다. 다시 열 때 행 높이는
//! 내용으로 다시 재므로, 넘친 행만 커지고 다른 행은 선언 높이를 지킨다.
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::table::Table;
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use crate::renderer::px_to_hwpunit;

fn content_height(measured: &MeasuredTable, dpi: f64) -> u32 {
    let rows = measured.cumulative_heights.last().copied().unwrap_or(0.0);
    px_to_hwpunit(rows, dpi).max(0) as u32
}

fn grown(declared: u32, content: u32) -> Option<u32> {
    (content > declared).then_some(content)
}

/// rhwp가 다시 짠 칸 문단이 있는 표인가. 한컴이 저장한 줄을 그대로 가진 표는 그 높이(sz)가
/// 한컴 조판값이므로 rhwp 측정으로 덮어쓰지 않는다. 본문 표는 이번 편집의 재조판 표시도 본다.
fn has_reflowed_cell_text(table: &Table) -> bool {
    table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .any(|paragraph| {
            // 재조판은 저장 태그를 보존하면서 새 줄·공백 규칙을 함께 발행할 수도 있다.
            // 그 발행 출처를 소비해야 중첩 셀의 정상 저장 태그를 미편집으로 오인하지 않는다.
            paragraph
                .layout_space_metrics
                .iter()
                .zip(&paragraph.line_segs)
                .any(|((start, _), line)| *start == line.text_start)
                || paragraph.line_segs.is_empty()
                || paragraph
                    .line_segs
                    .iter()
                    .any(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        })
}

impl DocumentCore {
    pub(super) fn grown_table_heights(&self) -> Vec<Option<u32>> {
        match self.grown_table_heights_with_storage(&crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    /// 본문은 현재 측정 결과를 재사용한다. 자식 표도 같은 정책으로 재며, 편집한
    /// 내용이 없는 표의 원본 높이를 다시 쓰지 않는다. 셀의 선언 높이는 보존한다.
    pub(super) fn grown_table_heights_with_storage<S: crate::storage::Storage>(
        &self,
        storage: &S,
    ) -> Result<Vec<Option<u32>>, S::Error> {
        let mut out = storage.vec(0)?;
        let profile = self.effective_layout_profile();
        let measurer = HeightMeasurer::new(self.dpi)
            .with_hwp3_variant(profile.hwp3_layout())
            .with_stored_geometry_policy(profile.stored_geometry_policy())
            .with_native_hwp5(profile.hwp5_stored_pagination_layout())
            .with_hwpx_stored_layout(profile.hwpx_stored_layout())
            .with_hwp5_origin_hwpx(profile.hwp5_origin_hwpx())
            .with_session_edited(profile.session_edited())
            .with_render_normalization(std::sync::Arc::clone(&self.render_normalization.overlay));
        for (section_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    let content = self
                        .measured_tables
                        .get(section_idx)
                        .and_then(|tables| {
                            tables.iter().find(|m| {
                                m.para_index == para_idx && m.control_index == control_idx
                            })
                        })
                        .filter(|_| {
                            has_reflowed_cell_text(table)
                                || self.table_text_reflowed_path_exists(
                                    section_idx,
                                    para_idx,
                                    control_idx,
                                )
                        })
                        .map(|m| content_height(m, self.dpi));
                    storage.push(
                        &mut out,
                        content.and_then(|value| grown(table.common.height, value)),
                    )?;
                    self.push_nested_grown_heights_with_storage(
                        &table.cells,
                        &measurer,
                        &mut out,
                        storage,
                    )?;
                }
            }
        }
        Ok(out)
    }

    fn push_nested_grown_heights_with_storage<S: crate::storage::Storage>(
        &self,
        cells: &[crate::model::table::Cell],
        measurer: &HeightMeasurer,
        out: &mut Vec<Option<u32>>,
        storage: &S,
    ) -> Result<(), S::Error> {
        // 형제 표의 원본 순서와 apply 순서를 보존하는 깊이 우선 순회다.
        for cell in cells {
            for paragraph in &cell.paragraphs {
                for control in &paragraph.controls {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    storage.bytes(0)?;
                    let height = if has_reflowed_cell_text(table) {
                        let measured = measurer.measure_table_with_storage(
                            table,
                            0,
                            0,
                            &self.styles,
                            storage,
                        )?;
                        grown(table.common.height, content_height(&measured, self.dpi))
                    } else {
                        None
                    };
                    storage.push(out, height)?;
                    self.push_nested_grown_heights_with_storage(
                        &table.cells,
                        measurer,
                        out,
                        storage,
                    )?;
                }
            }
        }
        Ok(())
    }
}

/// `grown_table_heights`와 같은 순서로 저장 사본의 표 높이를 키운다.
pub(super) fn apply_grown_table_heights(document: &mut Document, heights: &[Option<u32>]) {
    fn walk(paragraphs: &mut [Paragraph], heights: &mut std::slice::Iter<'_, Option<u32>>) {
        for paragraph in paragraphs {
            for control in &mut paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                if let Some(Some(height)) = heights.next() {
                    table.sync_ctrl_height(*height);
                }
                for cell in &mut table.cells {
                    walk(&mut cell.paragraphs, heights);
                }
            }
        }
    }
    let mut heights = heights.iter();
    for section in &mut document.sections {
        walk(&mut section.paragraphs, &mut heights);
    }
}
