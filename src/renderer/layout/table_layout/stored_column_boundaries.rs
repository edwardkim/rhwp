//! 저장 칸 폭으로 표 열 경계를 푼다.
//!
//! 칸 `(col, col_span, width)` 은 `경계[col + col_span] - 경계[col] = width + 칸 간격`
//! 이라는 차분 제약이다. 한컴은 행마다 칸 폭을 누적해 열 경계를 정하므로, 병합 격자에서도
//! 같은 경계는 모든 행에서 같은 x 에 놓인다(80168 「규제 개요」 표, 한컴 PDF 세로 괘선 실측).
//! 0번 경계에서 출발해 제약을 따라가면 칸이 쓰는 모든 경계가 하나로 정해진다.

use crate::model::table::Table;
use crate::renderer::hwpunit_to_px;
use std::collections::{BTreeSet, VecDeque};

/// 저장 HWPUNIT 은 정수라 행마다 누적하면 1~2HU 반올림 차가 날 수 있다.
const BOUNDARY_TOLERANCE_HU: i32 = 2;

/// 한 표의 저장 칸 폭 제약(px). `invalid_rows` 의 칸은 근거에서 뺀다.
pub(super) struct StoredCellSpans {
    col_count: usize,
    spacing: f64,
    tolerance: f64,
    /// (시작 경계, 끝 경계, 경계 간 길이 = 칸 폭 + 칸 간격)
    spans: Vec<(usize, usize, f64)>,
}

impl StoredCellSpans {
    pub(super) fn collect(
        table: &Table,
        col_count: usize,
        invalid_rows: &BTreeSet<u16>,
        dpi: f64,
        width_scale: f64,
    ) -> Option<Self> {
        if col_count == 0 {
            return None;
        }
        let spacing = hwpunit_to_px(i32::from(table.cell_spacing), dpi);
        let mut spans = Vec::new();
        for cell in &table.cells {
            if invalid_rows.contains(&cell.row) || cell.width == 0 || cell.col_span == 0 {
                continue;
            }
            let start = cell.col as usize;
            let end = start + cell.col_span as usize;
            if end > col_count {
                // 격자 밖으로 나가는 칸이 있으면 경계 해석의 전제가 깨진다.
                return None;
            }
            let width = hwpunit_to_px(cell.width.min(i32::MAX as u32) as i32, dpi) * width_scale;
            spans.push((start, end, width + spacing));
        }
        if spans.is_empty() {
            return None;
        }
        Some(Self {
            col_count,
            spacing,
            tolerance: hwpunit_to_px(BOUNDARY_TOLERANCE_HU, dpi) * width_scale.max(1.0),
            spans,
        })
    }

    /// 열 폭이 모든 저장 칸 폭 제약을 만족하는가.
    pub(super) fn satisfied_by(&self, col_widths: &[f64]) -> bool {
        if col_widths.len() != self.col_count {
            return false;
        }
        let mut boundary = Vec::with_capacity(self.col_count + 1);
        boundary.push(0.0);
        for width in col_widths {
            boundary.push(boundary.last().copied().unwrap_or(0.0) + width + self.spacing);
        }
        self.spans.iter().all(|&(start, end, length)| {
            (boundary[end] - boundary[start] - length).abs() <= self.tolerance
        })
    }

    /// 제약을 따라 열 경계를 풀고 열 폭을 돌려준다.
    ///
    /// 칸이 하나도 끝나거나 시작하지 않는 경계는 정할 근거가 없으므로 앞뒤 확정 경계
    /// 사이를 균등하게 나눈다. 다음 중 하나라도 해당하면 `None` 이다.
    /// - 같은 경계가 두 값으로 정해진다(행마다 칸 경계가 서로 모순되는 표).
    /// - 마지막 경계(`col_count`)가 0번 경계와 이어지지 않는다.
    /// - 폭이 0 이하인 열이 생긴다(경계가 단조 증가하지 않음).
    /// - 균등 분배 뒤에도 만족하지 못하는 칸이 남는다(0번 경계와 이어지지 않은 칸).
    pub(super) fn solve(&self) -> Option<Vec<f64>> {
        let mut adjacency: Vec<Vec<(usize, f64)>> = vec![Vec::new(); self.col_count + 1];
        for &(start, end, length) in &self.spans {
            adjacency[start].push((end, length));
            adjacency[end].push((start, -length));
        }
        let mut boundary: Vec<Option<f64>> = vec![None; self.col_count + 1];
        boundary[0] = Some(0.0);
        let mut queue = VecDeque::from([0usize]);
        while let Some(node) = queue.pop_front() {
            let base = boundary[node]?;
            for &(next, length) in &adjacency[node] {
                let value = base + length;
                match boundary[next] {
                    Some(existing) if (existing - value).abs() > self.tolerance => return None,
                    Some(_) => {}
                    None => {
                        boundary[next] = Some(value);
                        queue.push_back(next);
                    }
                }
            }
        }
        boundary[self.col_count]?;
        let mut resolved = vec![0.0f64; self.col_count + 1];
        let mut prev_known = 0usize;
        for (index, value) in boundary.iter().enumerate().skip(1) {
            let Some(value) = *value else {
                continue;
            };
            let start = resolved[prev_known];
            let gap = (index - prev_known) as f64;
            for step in 1..=index - prev_known {
                resolved[prev_known + step] = start + (value - start) * step as f64 / gap;
            }
            prev_known = index;
        }
        let col_widths: Vec<f64> = resolved
            .windows(2)
            .map(|pair| pair[1] - pair[0] - self.spacing)
            .collect();
        if col_widths.iter().any(|&width| width <= 0.0) || !self.satisfied_by(&col_widths) {
            return None;
        }
        Some(col_widths)
    }
}
