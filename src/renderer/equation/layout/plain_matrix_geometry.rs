//! 독립 한컴 Print에서 확인한 괄호 없는 행렬의 줄·열 배치.
//!
//! 일반 행렬은 열별 최대 폭을 쓰며 괄호 여백을 예약하지 않는다.
//! 줄 기준선과 전체 행렬 기준선은 나눗셈 사다리와 같은 글줄 메트릭을 소비한다.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind, MatrixStyle};

impl EqLayout {
    pub(super) fn layout_plain_matrix(&self, rows: &[Vec<EqNode>], fs: f64) -> LayoutBox {
        let mut cells: Vec<Vec<LayoutBox>> = rows
            .iter()
            .map(|row| {
                let mut boxes: Vec<_> = row.iter().map(|cell| self.layout_node(cell, fs)).collect();
                if self.hancom_math_face {
                    // 빈 템플릿/명시적 빈 칸도 반 em의 논리 슬롯을 소유한다.
                    if boxes.is_empty() {
                        boxes.push(self.layout_node(&EqNode::Empty, fs));
                    }
                    for cell in &mut boxes {
                        if matches!(cell.kind, LayoutKind::Empty) {
                            cell.width = fs * 0.5;
                        }
                    }
                }
                boxes
            })
            .collect();
        let columns = cells.iter().map(Vec::len).max().unwrap_or(0);
        if self.hancom_math_face && columns > 0 {
            // 한컴 정상 저장·Print의 불균등 행은 셀을 원문 순서대로 고정 열
            // 격자에 채운다. 선언한 행 수의 빈 예약 영역은 뒤에 보존한다.
            // 직사각형 행과 명시적 빈 칸의 소속은 이 연산으로 변하지 않는다.
            let row_count = cells.len();
            let mut ordered = cells.into_iter().flatten();
            cells = (0..row_count)
                .map(|_| ordered.by_ref().take(columns).collect())
                .collect();
        }
        let mut widths = vec![0.0_f64; columns];
        for row in &cells {
            for (ci, cell) in row.iter().enumerate() {
                widths[ci] = widths[ci].max(cell.width);
            }
        }
        let gap = fs * 0.2;
        let row_gap = fs * 0.15;
        let (line_height, line_baseline) =
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi).unwrap_or((fs, fs * 0.86));
        let row_count = cells.len();
        let mut y = 0.0;
        for (ri, row) in cells.iter_mut().enumerate() {
            let frames: Vec<(f64, f64)> = row
                .iter()
                .map(|cell| {
                    super::super::logical_font_frame::metrics_at_dpi(cell, fs, self.dpi)
                        .unwrap_or((cell.height, cell.baseline))
                })
                .collect();
            let ascent = frames
                .iter()
                .map(|(_, baseline)| *baseline)
                .fold(line_baseline, f64::max);
            let descent = frames
                .iter()
                .map(|(height, baseline)| height - baseline)
                .fold(line_height - line_baseline, f64::max);
            let mut height = if self.hancom_math_face {
                frames
                    .iter()
                    .map(|(height, _)| *height)
                    .fold(line_height, f64::max)
            } else {
                ascent + descent
            };
            if !self.hancom_math_face && ri + 1 == row_count {
                // 마지막 줄은 다음 줄의 피치를 예약하지 않고 표시 상자의 끝을 포함한다.
                height = row.iter().map(|cell| cell.height).fold(height, f64::max);
            }
            let mut x = 0.0;
            for (ci, cell) in row.iter_mut().enumerate() {
                cell.x = x + (widths[ci] - cell.width) / 2.0;
                cell.y = if self.hancom_math_face {
                    // 행렬 셀은 논리 상자의 가운데를 맞춘다. 본체·첨자 기준선은
                    // 각 셀 안에 보존하고 raw nominal em을 다시 예약하지 않는다.
                    let (cell_height, cell_baseline) = frames[ci];
                    y + (height - cell_height) / 2.0 + cell_baseline - cell.baseline
                } else {
                    y + ascent - cell.baseline
                };
                x += widths[ci] + gap;
            }
            y += height + if ri + 1 < row_count { row_gap } else { 0.0 };
        }
        let width = widths.iter().sum::<f64>() + gap * columns.saturating_sub(1) as f64;
        let baseline =
            super::ladder_geometry::centered_baseline(y, line_height, line_baseline, fs, self.dpi);
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: y,
            baseline,
            kind: LayoutKind::Matrix {
                cells,
                style: MatrixStyle::Plain,
            },
        }
    }
}
