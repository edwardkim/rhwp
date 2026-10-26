//! 저장할 때 내용이 넘친 표의 높이(sz)를 실제 행 높이 합으로 적는다.
//!
//! 한컴은 칸 높이(cellSz)는 선언값 그대로 두고 표 높이만 키워 저장한다. 다시 열 때 행 높이는
//! 내용으로 다시 재므로, 넘친 행만 커지고 다른 행은 선언 높이를 지킨다.
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use crate::renderer::px_to_hwpunit;

/// px 왕복 반올림 오차는 넘침으로 보지 않는다.
const ROUNDING_HU: u32 = 10;

fn content_height(measured: &MeasuredTable, dpi: f64) -> u32 {
    let rows = measured.cumulative_heights.last().copied().unwrap_or(0.0);
    px_to_hwpunit(rows, dpi).max(0) as u32
}

fn grown(declared: u32, content: u32) -> Option<u32> {
    (content > declared.saturating_add(ROUNDING_HU)).then_some(content)
}

impl DocumentCore {
    /// 모든 표를 깊이 우선 순서로 돌며, 키울 표의 새 높이를 같은 순서로 돌려준다.
    pub(super) fn grown_table_heights(&self) -> Vec<Option<u32>> {
        let mut out = Vec::new();
        for (section_idx, section) in self.document.sections.iter().enumerate() {
            let measured = self.measured_tables.get(section_idx);
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_idx, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    // 본문 표는 조판이 이미 잰 결과를 쓴다.
                    let content = measured
                        .and_then(|tables| {
                            tables.iter().find(|m| {
                                m.para_index == para_idx && m.control_index == control_idx
                            })
                        })
                        .map(|m| content_height(m, self.dpi));
                    out.push(content.and_then(|content| grown(table.common.height, content)));
                    self.push_nested_grown_heights(&table.cells, &mut out);
                }
            }
        }
        out
    }

    // ponytail: 저장마다 중첩 표를 다시 잰다. 중첩 표가 아주 많은 문서에서 저장이 느려지면
    // 조판의 중첩 측정을 캐시로 공유한다.
    fn push_nested_grown_heights(
        &self,
        cells: &[crate::model::table::Cell],
        out: &mut Vec<Option<u32>>,
    ) {
        for cell in cells {
            for paragraph in &cell.paragraphs {
                for control in &paragraph.controls {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    let measured =
                        HeightMeasurer::new(self.dpi).measure_table(table, 0, 0, &self.styles);
                    out.push(grown(
                        table.common.height,
                        content_height(&measured, self.dpi),
                    ));
                    self.push_nested_grown_heights(&table.cells, out);
                }
            }
        }
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
