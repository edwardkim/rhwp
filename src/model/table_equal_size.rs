//! 셀 블록의 너비·높이를 같게 맞춘다 (한컴 [표 - 셀 너비를 같게 / 셀 높이를 같게]).
//!
//! 한컴 한글 12.30은 블록의 셀만 같은 크기로 바꾸고, 블록 밖 셀은 제자리에 둔 채
//! 표 격자를 다시 나눈다. 그래서 블록 밖 셀은 여러 줄에 걸치는 병합 셀이 된다.

use super::Table;

/// 크기를 맞출 방향.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellSizeAxis {
    Width,
    Height,
}

/// 여러 줄에 걸친 병합 셀이 줄마다 다른 자리를 받으면 맞추지 않는다.
const CROSSING_LINES: &str =
    "여러 줄에 걸친 병합 셀의 경계가 줄마다 달라 크기를 같게 맞출 수 없습니다";

impl Table {
    /// 칸 경계 위치(HWPUNIT, 길이 = 열 수 + 1)를 셀 너비에서 구한다.
    ///
    /// 행마다 셀 너비를 왼쪽부터 더해 각 열 경계를 정한다. 비거나 겹친 행, 또는
    /// 행끼리 2HU보다 크게 어긋난 경계가 있으면 하나의 격자로 볼 수 없어 `None`이다.
    pub fn column_bounds(&self) -> Option<Vec<i64>> {
        let cols = usize::from(self.col_count);
        let mut bounds: Vec<Option<i64>> = vec![None; cols + 1];
        for row in 0..self.row_count {
            let mut cells: Vec<_> = self
                .cells
                .iter()
                .filter(|c| c.row <= row && row < c.row.saturating_add(c.row_span))
                .collect();
            cells.sort_by_key(|c| c.col);
            let (mut col, mut x) = (0usize, 0i64);
            for cell in cells {
                let end = usize::from(cell.col) + usize::from(cell.col_span);
                if usize::from(cell.col) != col || end > cols {
                    return None;
                }
                for (at, pos) in [(col, x), (end, x + i64::from(cell.width))] {
                    match bounds[at] {
                        Some(known) if (known - pos).abs() > 2 => return None,
                        Some(_) => {}
                        None => bounds[at] = Some(pos),
                    }
                }
                x += i64::from(cell.width);
                col = end;
            }
            if col != cols {
                return None;
            }
        }
        bounds.into_iter().collect()
    }

    /// 블록(행·열 포함 범위)의 셀을 `axis` 방향으로 같은 크기로 맞추고 격자를 다시 나눈다.
    ///
    /// `bounds`는 그 방향 격자선 위치(HWPUNIT, 길이 = 줄 수 + 1)다. 너비는 칸 경계,
    /// 높이는 화면에 그린 행 경계를 넘긴다. 블록의 줄(높이면 열, 너비면 행)마다 블록 길이를
    /// 그 줄의 셀 수로 나눠 반올림한 값이 그 줄 셀의 크기가 된다. 여러 줄에 걸친 셀은
    /// 줄마다 같은 자리를 받아야 한다. 반올림으로 생긴 차이만큼 블록 끝 뒤 경계와 표 크기를
    /// 함께 민다. 블록 밖 셀의
    /// 높이 선언은 그대로 두고, 너비는 새 경계 사이 거리로 맞춘다.
    ///
    /// 반환: (옛 셀 번호마다 정렬한 뒤의 새 번호, 너비가 바뀐 셀의 새 번호).
    pub fn equalize_cell_block(
        &mut self,
        axis: CellSizeAxis,
        (start_row, start_col, end_row, end_col): (u16, u16, u16, u16),
        bounds: &[i64],
    ) -> Result<(Vec<usize>, Vec<usize>), String> {
        let height = axis == CellSizeAxis::Height;
        let along = |c: &super::Cell| {
            let (s, n) = if height {
                (c.row, c.row_span)
            } else {
                (c.col, c.col_span)
            };
            (usize::from(s), usize::from(s) + usize::from(n))
        };
        let cross = |c: &super::Cell| {
            let (s, n) = if height {
                (c.col, c.col_span)
            } else {
                (c.row, c.row_span)
            };
            (usize::from(s), usize::from(s) + usize::from(n))
        };
        let (a0, a1, b0, b1, lines, others) = if height {
            (
                start_row,
                end_row,
                start_col,
                end_col,
                self.row_count,
                self.col_count,
            )
        } else {
            (
                start_col,
                end_col,
                start_row,
                end_row,
                self.col_count,
                self.row_count,
            )
        };
        let (a0, a1, b0, b1) = (
            usize::from(a0),
            usize::from(a1) + 1,
            usize::from(b0),
            usize::from(b1) + 1,
        );
        if a0 >= a1 || b0 >= b1 || a1 > usize::from(lines) || b1 > usize::from(others) {
            return Err("셀 블록이 표 밖에 있습니다".to_string());
        }
        if bounds.len() != usize::from(lines) + 1 || bounds.windows(2).any(|w| w[0] > w[1]) {
            return Err("표 격자 경계가 올바르지 않습니다".to_string());
        }

        // 블록 셀을 줄마다 순서대로 모은다. 블록 경계에 걸친 셀은 받지 않는다.
        let mut per_line: Vec<Vec<(usize, usize)>> = vec![Vec::new(); b1 - b0];
        let mut in_block = vec![false; self.cells.len()];
        for (i, cell) in self.cells.iter().enumerate() {
            let ((s, e), (t, f)) = (along(cell), cross(cell));
            if e > usize::from(lines) || f > usize::from(others) {
                return Err("표 격자가 올바르지 않습니다".to_string());
            }
            if s >= a1 || e <= a0 || t >= b1 || f <= b0 {
                continue;
            }
            if s < a0 || e > a1 || t < b0 || f > b1 {
                return Err(
                    "블록 경계에 걸친 병합 셀이 있어 크기를 같게 맞출 수 없습니다".to_string(),
                );
            }
            in_block[i] = true;
            for line in &mut per_line[t - b0..f - b0] {
                line.push((s, i));
            }
        }
        let start = bounds[a0];
        let extent = bounds[a1] - start;
        // 줄마다 그 줄의 칸 수로 블록 길이를 나눈다(한컴 12.30 실측: 두 칸 줄 6750HU, 세 칸 줄 4500HU).
        let mut sizes = Vec::with_capacity(per_line.len());
        for line in &mut per_line {
            line.sort_unstable();
            let n = line.len() as i64;
            let size = (extent + n / 2) / n;
            if n == 0 || size <= 0 || size > i64::from(u32::MAX) {
                return Err("셀 블록 크기가 올바르지 않습니다".to_string());
            }
            sizes.push(size);
        }
        // 반올림한 줄 길이 가운데 가장 긴 것이 새 블록 끝이다. 줄마다 마지막 칸이 그 끝까지 닿는다.
        // ponytail: 줄마다 반올림 차이가 다르면 마지막 칸이 몇 HU 다르다. 한컴 저장본은 나머지가 없는 경우만 확인했다.
        let end = per_line
            .iter()
            .zip(&sizes)
            .map(|(line, size)| start + size * line.len() as i64)
            .max()
            .unwrap_or(start + extent);
        let shift = end - (start + extent);
        let mut block_edges: Vec<Option<(i64, i64)>> = vec![None; self.cells.len()];
        for (line, &size) in per_line.iter().zip(&sizes) {
            let mut next = a0;
            for (k, &(s, i)) in line.iter().enumerate() {
                let k = k as i64;
                let edge = (
                    start + k * size,
                    if k + 1 == line.len() as i64 {
                        end
                    } else {
                        start + (k + 1) * size
                    },
                );
                if s != next {
                    return Err("표 격자가 올바르지 않습니다".to_string());
                }
                // 여러 줄에 걸친 칸은 줄마다 같은 자리에 와야 한다.
                if block_edges[i].is_some_and(|known| known != edge) {
                    return Err(CROSSING_LINES.to_string());
                }
                block_edges[i] = Some(edge);
                next = along(&self.cells[i]).1;
            }
            if next != a1 {
                return Err("표 격자가 올바르지 않습니다".to_string());
            }
        }
        let moved = |at: usize| bounds[at] + if at >= a1 { shift } else { 0 };
        let edges: Vec<(i64, i64)> = self
            .cells
            .iter()
            .zip(&block_edges)
            .map(|(cell, edge)| {
                edge.unwrap_or_else(|| {
                    let (s, e) = along(cell);
                    (moved(s), moved(e))
                })
            })
            .collect();
        let mut grid: Vec<i64> = edges.iter().flat_map(|&(s, e)| [s, e]).collect();
        grid.sort_unstable();
        grid.dedup();
        let new_lines =
            u16::try_from(grid.len() - 1).map_err(|_| "표 줄 수가 너무 많습니다".to_string())?;
        let index = |pos: i64| grid.binary_search(&pos).unwrap_or_else(|at| at) as u16;

        // 셀 영역(zone)은 옮긴 경계를 덮는 가장 가까운 새 격자선으로 넓힌다.
        for zone in &mut self.zones {
            let (zs, ze) = if height {
                (&mut zone.start_row, &mut zone.end_row)
            } else {
                (&mut zone.start_col, &mut zone.end_col)
            };
            let (s, e) = (usize::from(*zs), usize::from(*ze) + 1);
            if e > usize::from(lines) || s >= e {
                continue;
            }
            let first = grid.partition_point(|&g| g <= moved(s)).saturating_sub(1);
            *zs = first as u16;
            *ze = index(moved(e)).max(first as u16 + 1) - 1;
        }

        let mut width_changed = vec![false; self.cells.len()];
        for (i, cell) in self.cells.iter_mut().enumerate() {
            let (s, e) = edges[i];
            let (at, span) = (index(s), index(e) - index(s));
            if height {
                cell.row = at;
                cell.row_span = span;
                if in_block[i] {
                    cell.height = (e - s) as u32;
                }
            } else {
                cell.col = at;
                cell.col_span = span;
                let width = (e - s) as u32;
                if cell.width != width {
                    cell.width = width;
                    if cell.raw_list_extra.len() >= 4 {
                        cell.raw_list_extra[0..4].copy_from_slice(&width.to_le_bytes());
                    }
                    width_changed[i] = true;
                }
            }
        }

        let (mut total_width, mut total_height) = (self.common.width, self.common.height);
        let grown = |total: u32| (i64::from(total) + shift).clamp(0, i64::from(u32::MAX)) as u32;
        if height {
            self.row_count = new_lines;
            total_height = grown(total_height);
        } else {
            self.col_count = new_lines;
            total_width = grown(total_width);
        }
        let mut order: Vec<usize> = (0..self.cells.len()).collect();
        order.sort_by_key(|&i| (self.cells[i].row, self.cells[i].col));
        let mut cells: Vec<Option<super::Cell>> = std::mem::take(&mut self.cells)
            .into_iter()
            .map(Some)
            .collect();
        self.cells = order.iter().filter_map(|&i| cells[i].take()).collect();
        self.rebuild_row_sizes();
        self.sync_ctrl_dimensions(total_width, total_height);
        self.rebuild_grid();
        let mut moved_to = vec![0; order.len()];
        let mut widened = Vec::new();
        for (new, &old) in order.iter().enumerate() {
            moved_to[old] = new;
            if width_changed[old] {
                widened.push(new);
            }
        }
        Ok((moved_to, widened))
    }
}
