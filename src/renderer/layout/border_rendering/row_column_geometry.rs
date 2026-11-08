//! 열 위치를 누적하기 전의 실제 폭을 셀 내용까지 함께 전달한다.

#[derive(Debug, Clone)]
pub(crate) struct RowColumnGeometry {
    positions: Vec<Vec<f64>>,
    widths: Vec<Vec<f64>>,
    spacing: f64,
    cell_widths: Vec<Vec<(usize, usize, f64)>>,
}

impl RowColumnGeometry {
    pub(super) fn new(positions: Vec<Vec<f64>>, widths: Vec<Vec<f64>>, spacing: f64) -> Self {
        Self {
            positions,
            widths,
            spacing,
            cell_widths: Vec::new(),
        }
    }

    /// 누적 끝점의 차이로 폭을 다시 구하면 동일한 원본 열도 서로 다른
    /// 정수 프레임으로 내려갈 수 있다. 그리드 생산자가 선택한 폭을 사용한다.
    pub(crate) fn cell_width(
        &self,
        row: usize,
        start: usize,
        end: usize,
        cell_spacing: f64,
    ) -> f64 {
        if let Some(width) = self.cell_widths.get(row).and_then(|cells| {
            cells
                .iter()
                .find_map(|&(left, right, width)| (left == start && right == end).then_some(width))
        }) {
            return width;
        }
        self.widths[row][start..end].iter().sum::<f64>()
            + (self.spacing * (end - start) as f64 - cell_spacing)
    }

    /// 부모 안쪽 폭에 맞추는 배율은 경계와 그 경계를 생산한 폭에 함께 적용한다.
    pub(crate) fn scale(&mut self, factor: f64) {
        for row in &mut self.positions {
            for position in row {
                *position *= factor;
            }
        }
        for row in &mut self.widths {
            for width in row {
                *width *= factor;
            }
        }
        self.spacing *= factor;
        for row in &mut self.cell_widths {
            for (_, _, width) in row {
                *width *= factor;
            }
        }
    }

    /// 같은 HU 프레임을 본문과 괘선에 전달한다. 실제 폭을 가상 열 합으로 되돌리지 않는다.
    pub(super) fn apply_closing_rows(
        &mut self,
        table: &crate::model::table::Table,
        dpi: f64,
        width_scale: f64,
    ) {
        self.cell_widths = vec![Vec::new(); self.positions.len()];
        let scale = dpi / 7200.0 * width_scale;
        let spacing = f64::from(table.cell_spacing) * scale;
        for (row, frames) in table.closing_row_frames().into_iter().enumerate() {
            let Some(frames) = frames else {
                continue;
            };
            for frame in frames {
                let cell = &table.cells[frame.cell_index];
                let start = usize::from(cell.col);
                let end = start + usize::from(cell.col_span);
                let left = frame.start_hu as f64 * scale;
                let width = frame.width_hu as f64 * scale;
                let track = (width + spacing) / f64::from(cell.col_span);
                for col in start..end {
                    self.positions[row][col] = left + track * (col - start) as f64;
                    self.widths[row][col] = track - spacing;
                }
                self.positions[row][end] = left + width + spacing;
                self.cell_widths[row].push((start, end, width));
            }
        }
    }
}

impl std::ops::Deref for RowColumnGeometry {
    type Target = [Vec<f64>];

    fn deref(&self) -> &Self::Target {
        &self.positions
    }
}
