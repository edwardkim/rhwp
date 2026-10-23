//! 열 위치를 누적하기 전의 실제 폭을 셀 내용까지 함께 전달한다.

#[derive(Debug, Clone)]
pub(crate) struct RowColumnGeometry {
    positions: Vec<Vec<f64>>,
    widths: Vec<Vec<f64>>,
    spacing: f64,
}

impl RowColumnGeometry {
    pub(super) fn new(positions: Vec<Vec<f64>>, widths: Vec<Vec<f64>>, spacing: f64) -> Self {
        Self {
            positions,
            widths,
            spacing,
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
    }
}

impl std::ops::Deref for RowColumnGeometry {
    type Target = [Vec<f64>];

    fn deref(&self) -> &Self::Target {
        &self.positions
    }
}
