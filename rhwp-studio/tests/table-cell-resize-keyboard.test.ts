import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildColumnResizeUpdates,
  buildLocalResizeUpdates,
  buildBoundaryResizeUpdates,
} from '../src/engine/table-resize-updates.ts';
import type { CellBbox } from '../src/core/types.ts';

// F5 셀 선택 후 키보드 셀 크기 조절 3모드(한컴 table(size).htm)의 update 구성 계약.
//
// - Ctrl/Cmd = 선택 칸(열)/줄(행) 전체에 같은 delta (표 전체 크기 변화)
// - Alt      = 선택 칸/줄 전체와 바로 오른쪽/아래 이웃을 반대로 조절 (표 크기 유지)
// - Shift    = 선택 끝 경계 이동, 이웃이 반대로 조절 (표 크기 유지)
//
// 렌더 괘선은 열별 max base grid 를 쓰므로 Ctrl 이 단일 셀에만 delta 를 보내면
// 다행 표에서 화면에 반영되지 않는다 — 칸/줄 전체 적용이 계약의 핵심이다.

/** rows×cols 균일 그리드 bbox 생성 (w/h 는 px, 75×로 HWPUNIT 환산됨). */
function grid(rows: number, cols: number, wPx = 40, hPx = 20): CellBbox[] {
  const cells: CellBbox[] = [];
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      cells.push({
        cellIdx: r * cols + c,
        row: r,
        col: c,
        rowSpan: 1,
        colSpan: 1,
        pageIndex: 0,
        x: c * wPx,
        y: r * hPx,
        w: wPx,
        h: hPx,
      });
    }
  }
  return cells;
}

const cellAt = (cells: CellBbox[], row: number, col: number) =>
  cells.find(b => b.row === row && b.col === col)!;

function widthSum(updates: ReturnType<typeof buildColumnResizeUpdates>): number {
  return updates.reduce((s, u) => s + (u.widthDelta ?? 0), 0);
}

function deltasByCell(updates: ReturnType<typeof buildColumnResizeUpdates>, axis: 'widthDelta' | 'heightDelta') {
  return new Map(updates.map(update => [update.cellIdx, update[axis] ?? 0]));
}

function mergeRight(cells: CellBbox[], row: number, col: number, span: number): CellBbox[] {
  const merged = cellAt(cells, row, col);
  merged.colSpan = span;
  merged.w *= span;
  return cells.filter(cell => cell.row !== row || cell.col <= col || cell.col >= col + span);
}

function mergeDown(cells: CellBbox[], row: number, col: number, span: number): CellBbox[] {
  const merged = cellAt(cells, row, col);
  merged.rowSpan = span;
  merged.h *= span;
  return cells.filter(cell => cell.col !== col || cell.row <= row || cell.row >= row + span);
}

// ─── Ctrl: 칸/줄 전체 ─────────────────────────────────────────────

test('Ctrl 가로: 선택 열의 모든 행 셀에 같은 widthDelta 가 붙는다', () => {
  const cells = grid(5, 3);
  const range = { startRow: 2, startCol: 1, endRow: 2, endCol: 1 };
  const updates = buildColumnResizeUpdates(cells, range, 'ArrowRight');

  assert.equal(updates.length, 5, '열의 다섯 행 전부가 대상이어야 한다');
  assert.ok(updates.every(u => u.widthDelta === 300));
  assert.ok(updates.every(u => cells[u.cellIdx].col === 1), '다른 열이 섞이면 안 된다');
});

test('Ctrl 세로: 선택 행의 모든 열 셀에 heightDelta, ↑ 는 음수', () => {
  const cells = grid(4, 6);
  const range = { startRow: 1, startCol: 3, endRow: 1, endCol: 3 };
  const updates = buildColumnResizeUpdates(cells, range, 'ArrowUp');

  assert.equal(updates.length, 6);
  assert.ok(updates.every(u => u.heightDelta === -300));
});

test('Ctrl: 병합 셀은 걸친 선택 칸 수만큼 delta 를 받아 저장 폭이 동기화된다', () => {
  const cells = grid(3, 3);
  cellAt(cells, 1, 1).colSpan = 2; // col1~2 에 걸친 병합 셀
  const range = { startRow: 0, startCol: 1, endRow: 0, endCol: 1 };
  const updates = buildColumnResizeUpdates(cells, range, 'ArrowRight');

  assert.equal(updates.length, 3, '세 행 전부 대상 (병합 셀 포함)');
  const merged = updates.find(u => u.cellIdx === cellAt(cells, 1, 1).cellIdx)!;
  assert.equal(merged.widthDelta, 300, '선택 열과 한 칸 겹치므로 1×delta');
});

test('Ctrl: 선택 범위가 병합 셀을 두 칸 걸치면 2×delta', () => {
  const cells = grid(2, 4);
  cellAt(cells, 0, 1).colSpan = 2; // col1~2
  const range = { startRow: 0, startCol: 1, endRow: 0, endCol: 2 };
  const updates = buildColumnResizeUpdates(cells, range, 'ArrowRight');

  const merged = updates.find(u => u.cellIdx === cellAt(cells, 0, 1).cellIdx)!;
  assert.equal(merged.widthDelta, 600, '두 칸에 걸치므로 2×delta');
  const single = updates.find(u => u.cellIdx === cellAt(cells, 1, 1).cellIdx)!;
  assert.equal(single.widthDelta, 300);
});

// ─── Alt: 칸/줄 전체와 바로 오른쪽/아래 이웃 ──────────────────────

test('Alt →: 선택 세로 칸 전체 +300, 바로 오른쪽 세로 칸 전체 −300', () => {
  const cells = grid(3, 3);
  const range = { startRow: 1, startCol: 1, endRow: 1, endCol: 1 };
  const updates = buildLocalResizeUpdates(cells, range, 'ArrowRight');

  assert.equal(updates.length, 6);
  assert.equal(widthSum(updates), 0, '늘어난 만큼 정확히 흡수돼야 표 폭이 유지된다');
  assert.deepEqual(deltasByCell(updates, 'widthDelta'), new Map([
    [1, 300], [4, 300], [7, 300],
    [2, -300], [5, -300], [8, -300],
  ]));
  assert.ok(updates.every(u => u.localResize === true && typeof u.renderWidth === 'number'));
});

test('Alt ↓: 선택 가로줄 전체 +300, 바로 아래 가로줄 전체 −300', () => {
  const cells = grid(3, 3);
  const range = { startRow: 1, startCol: 1, endRow: 1, endCol: 1 };
  const updates = buildLocalResizeUpdates(cells, range, 'ArrowDown');

  assert.equal(updates.length, 6);
  assert.deepEqual(deltasByCell(updates, 'heightDelta'), new Map([
    [3, 300], [4, 300], [5, 300],
    [6, -300], [7, -300], [8, -300],
  ]));
});

test('Alt: F5가 병합 시작 좌표만 선택해도 실제 병합 폭을 조절한다', () => {
  const cells = mergeRight(grid(2, 3), 0, 0, 2);
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  const updates = buildLocalResizeUpdates(cells, range, 'ArrowRight');

  assert.deepEqual(deltasByCell(updates, 'widthDelta'), new Map([
    [0, 600], [2, -600],
    [3, 300], [4, 300], [5, -600],
  ]));
  assert.equal(widthSum(updates), 0);
});

test('Alt: F5가 세로 병합 시작 좌표만 선택해도 실제 병합 높이를 조절한다', () => {
  const cells = mergeDown(grid(3, 2), 0, 0, 2);
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  const updates = buildLocalResizeUpdates(cells, range, 'ArrowDown');

  assert.deepEqual(deltasByCell(updates, 'heightDelta'), new Map([
    [0, 600], [1, 300], [3, 300], [4, -600], [5, -600],
  ]));
  assert.equal(updates.reduce((sum, update) => sum + (update.heightDelta ?? 0), 0), 0);
});

test('Alt: 바로 오른쪽 이웃이 최소 크기 미달이면 전체 조절을 건너뛴다', () => {
  const cells = grid(1, 2, 5); // 5px ≈ 375 HWPUNIT — 흡수하면 최소(300) 미달
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  const updates = buildLocalResizeUpdates(cells, range, 'ArrowRight');

  assert.equal(updates.length, 0);
});

test('Alt: 마지막 칸은 바로 오른쪽 이웃이 없어 조절하지 않는다', () => {
  const cells = grid(2, 2);
  const range = { startRow: 0, startCol: 1, endRow: 1, endCol: 1 };
  assert.equal(buildLocalResizeUpdates(cells, range, 'ArrowRight').length, 0);
});

// ─── Shift: 경계 이동 ─────────────────────────────────────────────

test('Shift →: 끝 경계 이동 — 대상 +300, 오른쪽 이웃 −300', () => {
  const cells = grid(3, 3);
  const range = { startRow: 1, startCol: 1, endRow: 1, endCol: 1 };
  const updates = buildBoundaryResizeUpdates(cells, range, 'ArrowRight');

  assert.equal(updates.length, 2);
  assert.equal(widthSum(updates), 0, '경계 이동은 표 폭을 바꾸지 않는다');
  const target = updates.find(u => u.cellIdx === cellAt(cells, 1, 1).cellIdx)!;
  const neighbor = updates.find(u => u.cellIdx === cellAt(cells, 1, 2).cellIdx)!;
  assert.equal(target.widthDelta, 300);
  assert.equal(neighbor.widthDelta, -300);
});

test('Shift ←: 같은 경계가 안쪽으로 — 대상 −300, 이웃 +300', () => {
  const cells = grid(1, 3);
  const range = { startRow: 0, startCol: 1, endRow: 0, endCol: 1 };
  const updates = buildBoundaryResizeUpdates(cells, range, 'ArrowLeft');

  const target = updates.find(u => u.cellIdx === cellAt(cells, 0, 1).cellIdx)!;
  const neighbor = updates.find(u => u.cellIdx === cellAt(cells, 0, 2).cellIdx)!;
  assert.equal(target.widthDelta, -300);
  assert.equal(neighbor.widthDelta, 300);
});

test('Shift: 마지막 칸은 이웃이 없어 no-op', () => {
  const cells = grid(2, 3);
  const range = { startRow: 0, startCol: 2, endRow: 1, endCol: 2 };
  assert.equal(buildBoundaryResizeUpdates(cells, range, 'ArrowRight').length, 0);
});

test('Shift ↓: 세로 경계 — 대상 행 +300, 아래 행 −300, 열마다 한 쌍', () => {
  const cells = grid(3, 2);
  const range = { startRow: 1, startCol: 0, endRow: 1, endCol: 1 };
  const updates = buildBoundaryResizeUpdates(cells, range, 'ArrowDown');

  assert.equal(updates.length, 4, '두 열 각각 대상+이웃 한 쌍');
  assert.equal(updates.reduce((s, u) => s + (u.heightDelta ?? 0), 0), 0);
});

test('Shift: 이웃이 최소 크기 미달이 되면 그 줄은 건너뛴다', () => {
  const cells = grid(1, 2);
  cellAt(cells, 0, 1).w = 5; // ≈375 HWPUNIT — −300 하면 최소 미달
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  assert.equal(buildBoundaryResizeUpdates(cells, range, 'ArrowRight').length, 0);
});

test('Shift →: 병합 셀은 실제 오른쪽 경계와 이웃 한 셀만 조절한다', () => {
  const cells = mergeRight(grid(1, 3), 0, 0, 2);
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  const updates = buildBoundaryResizeUpdates(cells, range, 'ArrowRight');

  assert.deepEqual(deltasByCell(updates, 'widthDelta'), new Map([[0, 300], [2, -300]]));
  assert.equal(updates.find(update => update.cellIdx === 0)?.renderWidth, 6300);
});

test('Shift ↓: 세로 병합 셀은 실제 아래 경계와 이웃 한 셀만 조절한다', () => {
  const cells = mergeDown(grid(3, 1), 0, 0, 2);
  const range = { startRow: 0, startCol: 0, endRow: 0, endCol: 0 };
  const updates = buildBoundaryResizeUpdates(cells, range, 'ArrowDown');

  assert.deepEqual(deltasByCell(updates, 'heightDelta'), new Map([[0, 300], [2, -300]]));
  assert.equal(updates.find(update => update.cellIdx === 0)?.renderHeight, 3300);
});
