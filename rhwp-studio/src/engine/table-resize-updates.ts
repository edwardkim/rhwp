// F5 셀 선택 후 키보드 셀 크기 조절 3모드(한컴 table(size).htm)의
// resizeTableCells update 구성. WASM/DOM 의존이 없는 순수 로직이라
// 단위 테스트가 직접 검증한다 (tests/table-cell-resize-keyboard.test.ts).

import type { CellBbox } from '../core/types';

export type ResizeArrowKey = 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight';

export type CellSelectionRange = {
  startRow: number;
  startCol: number;
  endRow: number;
  endCol: number;
};

export type LocalResizeUpdate = {
  cellIdx: number;
  widthDelta?: number;
  heightDelta?: number;
  localResize?: boolean;
  renderWidth?: number;
  renderHeight?: number;
};

/** 키보드 셀 크기 조절: 1 키스트로크 당 이동량 (HWPUNIT, 약 1mm). */
const KEYBOARD_RESIZE_DELTA_HWP = 300;
/** px(96dpi) → HWPUNIT 환산 배율 (7200/96). getCellDisplaySize 와 동일 계약. */
const RESIZE_HWPUNIT_PER_PX = 75;
/** 셀 최소 크기 (HWPUNIT) — Rust MIN_CELL_SIZE(200)보다 보수적으로 300. */
const RESIZE_MIN_CELL_HWP = 300;

function resizeAxis(key: ResizeArrowKey): { isHoriz: boolean; delta: number } {
  const isHoriz = key === 'ArrowLeft' || key === 'ArrowRight';
  const delta =
    key === 'ArrowRight' || key === 'ArrowDown'
      ? KEYBOARD_RESIZE_DELTA_HWP
      : -KEYBOARD_RESIZE_DELTA_HWP;
  return { isHoriz, delta };
}

/** 페이지 fragment 중복을 제거한다. 병합 셀도 실제 경계를 찾아 처리해야 한다. */
function collectUniqueCells(bboxes: CellBbox[]): CellBbox[] {
  const seen = new Set<number>();
  const out: CellBbox[] = [];
  for (const b of bboxes) {
    if (seen.has(b.cellIdx)) continue;
    seen.add(b.cellIdx);
    out.push(b);
  }
  return out;
}

function cellSizeHwp(b: CellBbox, isHoriz: boolean): number {
  return Math.round((isHoriz ? b.w : b.h) * RESIZE_HWPUNIT_PER_PX);
}

function axisStart(b: CellBbox, isHoriz: boolean): number {
  return isHoriz ? b.col : b.row;
}

function axisEnd(b: CellBbox, isHoriz: boolean): number {
  return axisStart(b, isHoriz) + (isHoriz ? b.colSpan : b.rowSpan) - 1;
}

function crossContains(b: CellBbox, isHoriz: boolean, position: number): boolean {
  const start = isHoriz ? b.row : b.col;
  const span = isHoriz ? b.rowSpan : b.colSpan;
  return start <= position && position < start + span;
}

function overlapCount(b: CellBbox, isHoriz: boolean, start: number, end: number): number {
  return Math.max(0, Math.min(axisEnd(b, isHoriz), end) - Math.max(axisStart(b, isHoriz), start) + 1);
}

/** F5는 병합 셀의 시작 좌표만 보관하므로 실제 병합 범위까지 선택 축을 확장한다. */
function selectedAxisRange(
  cells: CellBbox[],
  range: CellSelectionRange,
  isHoriz: boolean,
): { start: number; end: number } | null {
  const selected = cells.filter(cell =>
    overlapCount(cell, true, range.startCol, range.endCol) > 0
      && overlapCount(cell, false, range.startRow, range.endRow) > 0);
  if (selected.length === 0) return null;
  return {
    start: Math.min(...selected.map(cell => axisStart(cell, isHoriz))),
    end: Math.max(...selected.map(cell => axisEnd(cell, isHoriz))),
  };
}

function sizeUpdate(
  cellIdx: number,
  isHoriz: boolean,
  delta: number,
  renderSize: number,
): LocalResizeUpdate {
  return isHoriz
    ? { cellIdx, widthDelta: delta, localResize: true, renderWidth: renderSize }
    : { cellIdx, heightDelta: delta, localResize: true, renderHeight: renderSize };
}

/**
 * Ctrl/Cmd+방향키: 선택 칸(열)/줄(행) 전체에 같은 delta — 표 전체 크기가 변한다.
 *
 * 렌더 괘선은 열별 max 로 만드는 base grid 를 쓰므로, 셀 하나만 조절하면
 * 다행 표에서 열 max 가 그대로라 화면에 반영되지 않는다. 병합 셀은 걸친
 * 선택 칸/줄 수만큼 delta 를 곱해 저장 폭/높이를 동기화한다 — 빼놓으면
 * 저장·재열기 후 병합 셀 폭이 열 폭 합과 어긋난다.
 */
export function buildColumnResizeUpdates(
  bboxes: CellBbox[],
  range: CellSelectionRange,
  key: ResizeArrowKey,
): LocalResizeUpdate[] {
  const { isHoriz, delta } = resizeAxis(key);
  const updates: LocalResizeUpdate[] = [];
  for (const b of collectUniqueCells(bboxes)) {
    // 셀이 걸친 선택 칸(열)/줄(행) 수 — span==1 이면 0 또는 1.
    const [selLo, selHi] = isHoriz ? [range.startCol, range.endCol] : [range.startRow, range.endRow];
    const overlap = overlapCount(b, isHoriz, selLo, selHi);
    if (overlap <= 0) continue;
    const d = delta * overlap;
    updates.push(isHoriz ? { cellIdx: b.cellIdx, widthDelta: d } : { cellIdx: b.cellIdx, heightDelta: d });
  }
  return updates;
}

/**
 * Alt+방향키: 선택 세로 칸/가로줄 전체와 바로 오른쪽/아래 이웃을 반대로
 * 조절해 표 크기를 유지한다. 병합 셀은 걸친 선택 칸/줄 수만큼 delta를
 * 적용한다. 선택의 바깥 경계가 없는 경우에는 표 크기를 바꾸지 않는다.
 */
export function buildLocalResizeUpdates(
  bboxes: CellBbox[],
  range: CellSelectionRange,
  key: ResizeArrowKey,
): LocalResizeUpdate[] {
  const { isHoriz, delta } = resizeAxis(key);
  const cells = collectUniqueCells(bboxes);
  const selectedRange = selectedAxisRange(cells, range, isHoriz);
  if (!selectedRange) return [];
  const { start: selectionStart, end: selectionEnd } = selectedRange;
  const selectionSpan = selectionEnd - selectionStart + 1;
  const neighborAxis = selectionEnd + 1;
  if (!cells.some(b => axisStart(b, isHoriz) <= neighborAxis && neighborAxis <= axisEnd(b, isHoriz))) {
    return [];
  }

  const deltas = new Map<number, { cell: CellBbox; delta: number }>();
  const addDelta = (cell: CellBbox, amount: number) => {
    const entry = deltas.get(cell.cellIdx);
    if (entry) entry.delta += amount;
    else deltas.set(cell.cellIdx, { cell, delta: amount });
  };
  for (const cell of cells) {
    const selectedCount = overlapCount(cell, isHoriz, selectionStart, selectionEnd);
    if (selectedCount > 0) addDelta(cell, delta * selectedCount);
    if (axisStart(cell, isHoriz) <= neighborAxis && neighborAxis <= axisEnd(cell, isHoriz)) {
      addDelta(cell, -delta * selectionSpan);
    }
  }
  const changed = [...deltas.values()].filter(entry => entry.delta !== 0);
  if (changed.some(entry => cellSizeHwp(entry.cell, isHoriz) + entry.delta < RESIZE_MIN_CELL_HWP)) {
    return [];
  }
  return changed.map(entry =>
    sizeUpdate(
      entry.cell.cellIdx,
      isHoriz,
      entry.delta,
      cellSizeHwp(entry.cell, isHoriz) + entry.delta,
    ));
}

/**
 * Shift+방향키: 경계 이동 — 셀이 커진 만큼 이웃이 작아진다. 표 전체 크기 유지.
 *
 * 정책: 항상 선택 끝(오른쪽/아래) 경계를 움직인다. →/↓ 는 바깥으로(셀 +,
 * 이웃 −), ←/↑ 는 안으로(셀 −, 이웃 +). 마지막 칸/줄은 이웃이 없어 no-op.
 */
export function buildBoundaryResizeUpdates(
  bboxes: CellBbox[],
  range: CellSelectionRange,
  key: ResizeArrowKey,
): LocalResizeUpdate[] {
  const { isHoriz, delta } = resizeAxis(key);
  const cells = collectUniqueCells(bboxes);

  const updates: LocalResizeUpdate[] = [];
  const processedTargets = new Set<number>();
  const processedNeighbors = new Set<number>();
  const laneStart = isHoriz ? range.startRow : range.startCol;
  const laneEnd = isHoriz ? range.endRow : range.endCol;
  const edge = isHoriz ? range.endCol : range.endRow;
  for (let lane = laneStart; lane <= laneEnd; lane++) {
    const target = cells.find(b =>
      axisStart(b, isHoriz) <= edge && edge <= axisEnd(b, isHoriz) && crossContains(b, isHoriz, lane));
    if (!target || processedTargets.has(target.cellIdx)) continue;
    const neighborAxis = axisEnd(target, isHoriz) + 1;
    const neighbor = cells.find(b => axisStart(b, isHoriz) === neighborAxis && crossContains(b, isHoriz, lane));
    if (!target || !neighbor) continue; // 마지막 칸/줄 — 이웃 없음
    if (cellSizeHwp(target, isHoriz) + delta < RESIZE_MIN_CELL_HWP) continue;
    if (cellSizeHwp(neighbor, isHoriz) - delta < RESIZE_MIN_CELL_HWP) continue;
    updates.push(sizeUpdate(target.cellIdx, isHoriz, delta, cellSizeHwp(target, isHoriz) + delta));
    processedTargets.add(target.cellIdx);
    if (!processedNeighbors.has(neighbor.cellIdx)) {
      updates.push(sizeUpdate(neighbor.cellIdx, isHoriz, -delta, cellSizeHwp(neighbor, isHoriz) - delta));
      processedNeighbors.add(neighbor.cellIdx);
    }
  }
  return updates;
}
