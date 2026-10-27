//! CASES의 `&`는 고정 공백이 아니라 모든 행이 공유하는 칸 맞춤 경계다.
//! 한컴 2020 정상화·Print의 11pt/22pt 대조: 첫 경계 0.2em,
//! 연속 경계 사이의 빈 칸 0.5em. 두 번째 이후 경계는 0.7em을 전진한다.
//! 독립 Print job: 41e23654-131c-4973-bc92-71b8b8db1bef.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind, SpaceKind};

fn items(row: &EqNode) -> &[EqNode] {
    match row.ungrouped() {
        EqNode::Row(items) => items,
        _ => std::slice::from_ref(row),
    }
}

fn is_boundary(node: &EqNode) -> bool {
    matches!(node, EqNode::Space(SpaceKind::Tab))
}

impl EqLayout {
    pub(super) fn layout_cases_rows(&self, rows: &[EqNode], fs: f64) -> Vec<LayoutBox> {
        // 이 묶음의 직접적인 행 경계만 맞춘다. 중첩 EQALIGN 등의 경계는
        // 자식 배치가 소비하며 바깥 CASES의 열에 섞지 않는다.
        if !rows.iter().any(|row| items(row).iter().any(is_boundary)) {
            return rows
                .iter()
                .map(|row| self.matrix_row_box(self.layout_node(row, fs), fs))
                .collect();
        }

        let mut cells: Vec<Vec<LayoutBox>> = rows
            .iter()
            .map(|row| {
                items(row)
                    .split(is_boundary)
                    .map(|column| self.layout_row(column, fs))
                    .collect()
            })
            .collect();
        let column_count = cells.iter().map(Vec::len).max().unwrap_or(0);
        let mut widths = vec![0.0_f64; column_count];
        for row in &cells {
            for (column, cell) in row.iter().enumerate() {
                widths[column] = widths[column].max(cell.width);
            }
        }
        // `&&`의 가운데 칸도 칸 맞춤 구조에 참여한다. 앞 식의 너비와
        // 관계없이 모든 행이 같은 빈 칸 및 다음 열의 원점을 소비한다.
        for width in &mut widths {
            if *width == 0.0 {
                *width = fs * 0.5;
            }
        }
        let gap = fs * 0.2;
        let mut origins = Vec::with_capacity(column_count);
        let mut x = 0.0;
        for width in widths {
            origins.push(x);
            x += width + gap;
        }

        cells
            .iter_mut()
            .map(|row| {
                let frames: Vec<_> = row
                    .iter()
                    .map(|cell| self.matrix_cell_frame(cell, fs))
                    .collect();
                let ascent = frames
                    .iter()
                    .map(|(_, baseline)| *baseline)
                    .fold(0.0_f64, f64::max);
                let descent = frames
                    .iter()
                    .map(|(height, baseline)| height - baseline)
                    .fold(0.0_f64, f64::max);
                let mut width = 0.0_f64;
                for (column, cell) in row.iter_mut().enumerate() {
                    cell.x = origins[column];
                    cell.y = ascent - cell.baseline;
                    width = width.max(cell.x + cell.width);
                }
                LayoutBox {
                    font_size: fs,
                    x: 0.0,
                    y: 0.0,
                    width,
                    height: ascent + descent,
                    baseline: ascent,
                    kind: LayoutKind::Row(std::mem::take(row)),
                }
            })
            .collect()
    }
}
