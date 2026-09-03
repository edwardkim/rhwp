import test from 'node:test';
import assert from 'node:assert/strict';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import {
  buildColumnResizeUpdates,
} from '../src/engine/table-resize-updates.ts';
import type { CellBbox } from '../src/core/types.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

// F5 셀 선택 후 지속 가능한 Ctrl/Cmd 셀 크기 조절의 update 구성 계약.
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

test('Alt/Shift+Arrow는 빈 resize handler에 삼켜지지 않고 셀 탐색을 수행한다', async () => {
  const vite = await createServer({
    root: rootDir,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    const { onKeyDown } = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    let phase = 1;
    const calls: Array<[string, number, number]> = [];
    let selectionUpdates = 0;
    let caretHides = 0;
    let prevented = 0;
    const cursor = {
      isInHeaderFooter: () => false,
      isInFootnote: () => false,
      isInPictureObjectSelection: () => false,
      isInTableObjectSelection: () => false,
      isInBlockSelectionMode: () => false,
      isInCellSelectionMode: () => true,
      getCellSelectionPhase: () => phase,
      moveCellSelection: (dr: number, dc: number) => calls.push(['move', dr, dc]),
      expandCellSelection: (dr: number, dc: number) => calls.push(['expand', dr, dc]),
    };
    const handler = {
      active: true,
      cursor,
      flushDeferredPaginationIfNeeded: () => {},
      updateCellSelection: () => { selectionUpdates += 1; },
      caret: { hide: () => { caretHides += 1; } },
    };
    const arrow = (key: string, modifiers: Record<string, boolean>) => ({
      key,
      code: key,
      shiftKey: false,
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      preventDefault: () => { prevented += 1; },
      ...modifiers,
    });

    onKeyDown.call(handler, arrow('ArrowRight', { altKey: true }));
    phase = 2;
    onKeyDown.call(handler, arrow('ArrowDown', { shiftKey: true }));

    assert.deepEqual(calls, [['move', 0, 1], ['expand', 1, 0]]);
    assert.equal(prevented, 2, '두 이벤트 모두 실제 셀 탐색 owner가 소비해야 함');
    assert.equal(selectionUpdates, 2);
    assert.equal(caretHides, 1, 'phase 1 이동만 숨은 caret 상태를 갱신한다');
  } finally {
    await vite.close();
  }
});
