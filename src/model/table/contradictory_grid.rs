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
use crate::storage::Storage;

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
        match self.contradictory_column_boundaries_hu_with_storage(&crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(crate) fn contradictory_column_boundaries_hu_with_storage<S: Storage>(
        &self,
        storage: &S,
    ) -> Result<Option<Vec<i64>>, S::Error> {
        let n = usize::from(self.col_count);
        if n == 0 || self.common.width == 0 || self.cell_spacing < 0 {
            return Ok(None);
        }
        let invalid_rows = self.invalid_declared_width_rows_with_storage(storage)?;
        let spacing = i64::from(self.cell_spacing);
        let mut spans = storage.vec(self.cells.len())?;
        for cell in &self.cells {
            if invalid_rows.contains(&cell.row) || cell.width == 0 || cell.col_span == 0 {
                continue;
            }
            let start = usize::from(cell.col);
            let end = start + usize::from(cell.col_span);
            if end > n {
                return Ok(None);
            }
            storage.push(
                &mut spans,
                (
                    start,
                    end,
                    i64::from(self.physical_cell_width_hu(cell)) + spacing,
                ),
            )?;
        }
        if spans.is_empty()
            || !exact_boundaries_contradict(n, &spans, storage)?
            || !self.has_short_declared_row(&invalid_rows, storage)?
        {
            return Ok(None);
        }

        let min_step = CONTRADICTION_MIN_COLUMN_HU + spacing;
        let target_end = i64::from(self.common.width) - spacing;
        let mut incoming: Vec<Vec<(usize, i64)>> = storage.vec(n + 1)?;
        incoming.resize_with(n + 1, Vec::new);
        let mut anchored = storage.filled(n + 1, false)?;
        anchored[0] = true;
        anchored[n] = true;
        for &(start, end, length) in &spans {
            storage.push(&mut incoming[end], (start, length))?;
            anchored[start] = true;
            anchored[end] = true;
        }
        // 경계마다 그 경계에서 끝나는 칸 폭의 최댓값(하한)을 쓴다. 그 값이 앞 경계보다
        // 앞이거나 같으면(다른 행이 앞 경계를 더 뒤에 고정함), 또는 끝나는 칸이 없으면
        // 앞 경계에서 최소 열 폭만큼 띄운다. 모순 없는 좁은 열은 그대로 둔다.
        let mut lower = storage.filled(n + 1, 0i64)?;
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
            return Ok(None);
        }
        lower[n] = lower[n].max(target_end);

        // 칸이 쓰지 않는 경계는 앞뒤 확정 경계 사이를 균등하게 나눈다.
        let mut resolved = lower;
        let mut prev_known = 0usize;
        for index in 1..=n {
            if !anchored[index] {
                continue;
            }
            let start = resolved[prev_known];
            let end = resolved[index];
            let gap = (index - prev_known) as i64;
            for step in 1..index - prev_known {
                resolved[prev_known + step] = start + (end - start) * step as i64 / gap;
            }
            prev_known = index;
        }
        if resolved
            .windows(2)
            .any(|pair| pair[1] - pair[0] - spacing <= 0)
        {
            return Ok(None);
        }
        Ok(Some(resolved))
    }

    /// 하한 해 경계로 본 각 칸의 물리 폭(HU). 모순 표가 아니면 `None`.
    pub(crate) fn contradictory_cell_widths_hu_with_storage<S: Storage>(
        &self,
        storage: &S,
    ) -> Result<Option<Vec<i32>>, S::Error> {
        let Some(boundaries) = self.contradictory_column_boundaries_hu_with_storage(storage)?
        else {
            return Ok(None);
        };
        let spacing = i64::from(self.cell_spacing);
        Ok(Some(storage.collect(self.cells.iter().map(|cell| {
            let start = usize::from(cell.col);
            let end = start + usize::from(cell.col_span.max(1));
            match (boundaries.get(start), boundaries.get(end)) {
                (Some(&left), Some(&right)) => {
                    (right - left - spacing).clamp(0, i64::from(i32::MAX)) as i32
                }
                _ => cell.width.min(i32::MAX as u32) as i32,
            }
        }))?))
    }
}

impl Table {
    /// 칸 폭 합(앞 행에서 내려온 세로 병합 칸 포함)이 표 폭보다 짧은 행이 있는가.
    /// 남는 폭을 흡수할 짧은 행이 없으면 하한 해의 근거가 없다.
    fn has_short_declared_row<S: Storage>(
        &self,
        invalid_rows: &std::collections::BTreeSet<u16>,
        storage: &S,
    ) -> Result<bool, S::Error> {
        let spacing = i64::from(self.cell_spacing);
        let target = i64::from(self.common.width) - 2 * spacing;
        // 세로 병합 칸은 시작/끝 행에서 한 번씩 더하고 뺀다. 행마다 모든 칸을
        // 다시 순회하지 않으며, 기존 행 소유·간격 판정은 그대로 유지한다.
        let rows = usize::from(self.row_count);
        let mut changes = storage.filled(rows + 1, (0i64, 0i64))?;
        for cell in &self.cells {
            let start = usize::from(cell.row).min(rows);
            let end = usize::from(cell.row.saturating_add(cell.row_span.max(1))).min(rows);
            if start < end && cell.width > 0 {
                let width = i64::from(self.physical_cell_width_hu(cell));
                changes[start].0 += width;
                changes[end].0 -= width;
                changes[start].1 += 1;
                changes[end].1 -= 1;
            }
        }
        let (mut total, mut count) = (0i64, 0i64);
        for (row, &(width_delta, count_delta)) in changes[..rows].iter().enumerate() {
            total += width_delta;
            count += count_delta;
            if !invalid_rows.contains(&(row as u16))
                && count > 0
                && total + spacing * (count - 1) < target - BOUNDARY_TOLERANCE_HU
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// 0번 경계에서 등식을 따라 경계를 정할 때, 같은 경계가 두 값으로 정해지거나 정해진
/// 경계가 단조 증가하지 않으면 참이다.
fn exact_boundaries_contradict<S: Storage>(
    n: usize,
    spans: &[(usize, usize, i64)],
    storage: &S,
) -> Result<bool, S::Error> {
    let mut adjacency: Vec<Vec<(usize, i64)>> = storage.vec(n + 1)?;
    adjacency.resize_with(n + 1, Vec::new);
    for &(start, end, length) in spans {
        storage.push(&mut adjacency[start], (end, length))?;
        storage.push(&mut adjacency[end], (start, -length))?;
    }
    let mut boundary = storage.filled(n + 1, None)?;
    boundary[0] = Some(0);
    let mut queue = storage.vec(n + 1)?;
    storage.push(&mut queue, 0usize)?;
    let mut cursor = 0;
    while cursor < queue.len() {
        let node = queue[cursor];
        cursor += 1;
        let Some(base) = boundary[node] else {
            continue;
        };
        for &(next, length) in &adjacency[node] {
            let value = base + length;
            match boundary[next] {
                Some(existing) if (existing - value).abs() > BOUNDARY_TOLERANCE_HU => {
                    return Ok(true)
                }
                Some(_) => {}
                None => {
                    boundary[next] = Some(value);
                    storage.push(&mut queue, next)?;
                }
            }
        }
    }
    let mut previous = None;
    for &value in boundary.iter().flatten() {
        if previous.is_some_and(|old| value <= old) {
            return Ok(true);
        }
        previous = Some(value);
    }
    Ok(false)
}
