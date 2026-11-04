//! 행마다 칸 경계가 엇갈리는 표의 열 경계(하한 해).
//!
//! 칸 `(col, col_span, width)` 은 `경계[col + col_span] - 경계[col] = width + 칸 간격`
//! 이다. 행마다 칸 폭 합이 다르면 이 등식이 서로 모순된다 — 짧은 행의 칸 끝이 다른
//! 행이 고정한 경계보다 앞에 놓인다. 한컴은 칸 폭을 하한으로 보고 경계를 단조 증가
//! 시킨다. 다른 행이 고정한 경계 뒤로 밀린 경계는 최소 열 폭만큼만 띄우고, 짧은 행의
//! 마지막 칸이 표 폭까지 남는 폭을 흡수한다(웹한글기안기 인쇄 실측).
//!
//! 칸 그리기·칸 문단 줄 나눔·행 높이 측정이 같은 경계를 쓰도록 HWPUNIT 으로 한 번 푼다.

use super::Table;

/// 저장 HWPUNIT 은 정수라 행마다 누적하면 1~2HU 반올림 차가 날 수 있다.
const BOUNDARY_TOLERANCE_HU: i64 = 2;

/// 모순으로 밀린 경계와 앞 경계 사이에 남기는 최소 열 폭. 웹한글기안기 인쇄(600dpi)에서
/// 고정 경계 뒤로 밀린 두 경계가 모두 8px(96HU) 떨어져 있었다.
const CONTRADICTION_MIN_COLUMN_HU: i64 = 96;

impl Table {
    /// 저장 칸 폭 등식이 서로 모순되는 표에서 하한 해로 푼 열 경계(HU, `col_count + 1` 개).
    ///
    /// 0번 경계는 0, 마지막 경계는 표 폭에서 바깥 간격을 뺀 값이다. 다음이면 `None`:
    /// - 등식이 모순되지 않는 표(종전 해석이 담당한다).
    /// - 폭 0 표, 격자 밖으로 나가는 칸.
    /// - 하한 경계가 표 폭을 넘는 표(칸 폭 합이 표보다 긴 행이 있음).
    pub(crate) fn contradictory_column_boundaries_hu(&self) -> Option<Vec<i64>> {
        let n = usize::from(self.col_count);
        if n == 0 || self.common.width == 0 || self.cell_spacing < 0 {
            return None;
        }
        let invalid_rows = self.invalid_declared_width_rows();
        let spacing = i64::from(self.cell_spacing);
        let mut spans = Vec::new();
        for cell in &self.cells {
            if invalid_rows.contains(&cell.row) || cell.width == 0 || cell.col_span == 0 {
                continue;
            }
            let start = usize::from(cell.col);
            let end = start + usize::from(cell.col_span);
            if end > n {
                return None;
            }
            spans.push((
                start,
                end,
                i64::from(self.physical_cell_width_hu(cell)) + spacing,
            ));
        }
        if spans.is_empty()
            || !exact_boundaries_contradict(n, &spans)
            || !self.has_short_declared_row(&invalid_rows)
        {
            return None;
        }

        let min_step = CONTRADICTION_MIN_COLUMN_HU + spacing;
        let target_end = i64::from(self.common.width) - spacing;
        let mut incoming: Vec<Vec<(usize, i64)>> = vec![Vec::new(); n + 1];
        let mut anchored = vec![false; n + 1];
        anchored[0] = true;
        anchored[n] = true;
        for &(start, end, length) in &spans {
            incoming[end].push((start, length));
            anchored[start] = true;
            anchored[end] = true;
        }
        // 경계마다 그 경계에서 끝나는 칸 폭의 최댓값(하한)을 쓴다. 그 값이 앞 경계보다
        // 앞이거나 같으면(다른 행이 앞 경계를 더 뒤에 고정함), 또는 끝나는 칸이 없으면
        // 앞 경계에서 최소 열 폭만큼 띄운다. 모순 없는 좁은 열은 그대로 둔다.
        let mut lower = vec![0i64; n + 1];
        for index in 1..=n {
            let declared = incoming[index]
                .iter()
                .map(|&(start, length)| lower[start] + length)
                .max();
            lower[index] = match declared {
                Some(value) if value > lower[index - 1] => value,
                _ => lower[index - 1] + min_step,
            };
        }
        if lower[n] > target_end + BOUNDARY_TOLERANCE_HU {
            return None;
        }
        lower[n] = lower[n].max(target_end);

        // 칸이 쓰지 않는 경계는 앞뒤 확정 경계 사이를 균등하게 나눈다.
        let mut resolved = lower.clone();
        let mut prev_known = 0usize;
        for index in 1..=n {
            if !anchored[index] {
                continue;
            }
            let start = resolved[prev_known];
            let gap = (index - prev_known) as i64;
            for step in 1..index - prev_known {
                resolved[prev_known + step] = start + (lower[index] - start) * step as i64 / gap;
            }
            prev_known = index;
        }
        if resolved
            .windows(2)
            .any(|pair| pair[1] - pair[0] - spacing <= 0)
        {
            return None;
        }
        Some(resolved)
    }

    /// 하한 해 경계로 본 각 칸의 물리 폭(HU). 모순 표가 아니면 `None`.
    pub(crate) fn contradictory_cell_widths_hu(&self) -> Option<Vec<i32>> {
        let boundaries = self.contradictory_column_boundaries_hu()?;
        let spacing = i64::from(self.cell_spacing);
        Some(
            self.cells
                .iter()
                .map(|cell| {
                    let start = usize::from(cell.col);
                    let end = start + usize::from(cell.col_span.max(1));
                    match (boundaries.get(start), boundaries.get(end)) {
                        (Some(&left), Some(&right)) => {
                            (right - left - spacing).clamp(0, i64::from(i32::MAX)) as i32
                        }
                        _ => cell.width.min(i32::MAX as u32) as i32,
                    }
                })
                .collect(),
        )
    }
}

impl Table {
    /// 칸 폭 합(앞 행에서 내려온 세로 병합 칸 포함)이 표 폭보다 짧은 행이 있는가.
    /// 남는 폭을 흡수할 짧은 행이 없으면 하한 해의 근거가 없다.
    fn has_short_declared_row(&self, invalid_rows: &std::collections::BTreeSet<u16>) -> bool {
        let spacing = i64::from(self.cell_spacing);
        let target = i64::from(self.common.width) - 2 * spacing;
        (0..self.row_count).any(|row| {
            if invalid_rows.contains(&row) {
                return false;
            }
            let mut total = 0i64;
            let mut count = 0usize;
            for cell in &self.cells {
                let start = cell.row;
                let covers = start <= row && row < start.saturating_add(cell.row_span.max(1));
                if covers && cell.width > 0 {
                    total += i64::from(self.physical_cell_width_hu(cell));
                    count += 1;
                }
            }
            let row_spacing = spacing * (count.saturating_sub(1) as i64);
            count > 0 && total + row_spacing < target - BOUNDARY_TOLERANCE_HU
        })
    }
}

/// 0번 경계에서 등식을 따라 경계를 정할 때, 같은 경계가 두 값으로 정해지거나 정해진
/// 경계가 단조 증가하지 않으면 참이다.
fn exact_boundaries_contradict(n: usize, spans: &[(usize, usize, i64)]) -> bool {
    let mut adjacency: Vec<Vec<(usize, i64)>> = vec![Vec::new(); n + 1];
    for &(start, end, length) in spans {
        adjacency[start].push((end, length));
        adjacency[end].push((start, -length));
    }
    let mut boundary: Vec<Option<i64>> = vec![None; n + 1];
    boundary[0] = Some(0);
    let mut queue = std::collections::VecDeque::from([0usize]);
    while let Some(node) = queue.pop_front() {
        let Some(base) = boundary[node] else {
            continue;
        };
        for &(next, length) in &adjacency[node] {
            let value = base + length;
            match boundary[next] {
                Some(existing) if (existing - value).abs() > BOUNDARY_TOLERANCE_HU => return true,
                Some(_) => {}
                None => {
                    boundary[next] = Some(value);
                    queue.push_back(next);
                }
            }
        }
    }
    let known: Vec<i64> = boundary.iter().flatten().copied().collect();
    known.windows(2).any(|pair| pair[1] <= pair[0])
}
