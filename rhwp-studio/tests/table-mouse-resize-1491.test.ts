import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  buildCellSelectionColumnDragUpdates,
  cellOverlapsSelectionRange,
  findResizeCompensationNeighbors,
  shouldStartPersistentTableResize,
} from '../src/engine/table-resize-updates.ts';
import type { CellBbox } from '../src/core/types.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function cellSelectionMouseDownBlock(): string {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const start = mouse.indexOf('if (this.cursor.isInCellSelectionMode()) {');
  assert.notEqual(start, -1, 'cell selection mouse block not found');
  const end = mouse.indexOf('\n  // 우클릭 → 텍스트 선택 블록 유지', start);
  assert.notEqual(end, -1, 'cell selection mouse block end not found');
  return mouse.slice(start, end);
}

function resizeHoverBlock(): string {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const start = mouse.indexOf('export function handleResizeHover');
  assert.notEqual(start, -1, 'handleResizeHover not found');
  const end = mouse.indexOf('\nexport function onMouseUp', start);
  assert.notEqual(end, -1, 'handleResizeHover end not found');
  return mouse.slice(start, end);
}

function generalTableResizeMouseDownBlock(): string {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const start = mouse.indexOf('// 표 경계선 클릭 → 리사이즈 드래그 시작');
  assert.notEqual(start, -1, 'general table resize mousedown block not found');
  const end = mouse.indexOf('\n  // 머리말/꼬리말 편집 모드', start);
  assert.notEqual(end, -1, 'general table resize mousedown block end not found');
  return mouse.slice(start, end);
}

function resolveTableResizeHitBlock(): string {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const start = mouse.indexOf('function resolveTableResizeHit');
  assert.notEqual(start, -1, 'resolveTableResizeHit not found');
  const end = mouse.indexOf('\nfunction updateCellSelectionDrag', start);
  assert.notEqual(end, -1, 'resolveTableResizeHit end not found');
  return mouse.slice(start, end);
}

function hitTestBorderBlock(): string {
  const renderer = source('src/engine/table-resize-renderer.ts');
  const start = renderer.indexOf('hitTestBorder(');
  assert.notEqual(start, -1, 'hitTestBorder not found');
  const end = renderer.indexOf('\n  /** 경계선 위에 마커', start);
  assert.notEqual(end, -1, 'hitTestBorder end not found');
  return renderer.slice(start, end);
}

function inputHandlerTableSource(): string {
  return source('src/engine/input-handler-table.ts');
}

function updateResizeDragBlock(): string {
  const table = inputHandlerTableSource();
  const start = table.indexOf('export function updateResizeDrag');
  assert.notEqual(start, -1, 'updateResizeDrag not found');
  const end = table.indexOf('\nexport function finishResizeDrag', start);
  assert.notEqual(end, -1, 'updateResizeDrag end not found');
  return table.slice(start, end);
}

function finishResizeDragBlock(): string {
  const table = inputHandlerTableSource();
  const start = table.indexOf('export function finishResizeDrag');
  assert.notEqual(start, -1, 'finishResizeDrag not found');
  const end = table.indexOf('\nexport function cleanupResizeDrag', start);
  assert.notEqual(end, -1, 'finishResizeDrag end not found');
  return table.slice(start, end);
}

test('셀 선택 모드 Shift+경계선 클릭은 drag를 열지 않고 범위 선택에 남는다', () => {
  const block = cellSelectionMouseDownBlock();
  const resizeIdx = block.indexOf('shouldStartPersistentTableResize(e.button, e.shiftKey)');
  const shiftSelectIdx = block.indexOf('if (e.shiftKey || e.ctrlKey || e.metaKey)');

  assert.equal(shouldStartPersistentTableResize(0, true), false);
  assert.equal(shouldStartPersistentTableResize(0, false), true);
  assert.notEqual(resizeIdx, -1, 'persistent resize admission 필요');
  assert.notEqual(shiftSelectIdx, -1, 'Shift/Ctrl 셀 선택 경로 필요');
  assert.match(block, /shouldStartPersistentTableResize\(e\.button, e\.shiftKey\)[\s\S]*this\.startResizeDrag\(edge, pageX, pageY, pageBboxes\)/);
  assert.doesNotMatch(block, /startResizeDrag\([^\n]*e\.shiftKey/);
});

test('표 경계 hover는 hitTest 실패 시 직전 bbox 캐시로 경계선을 다시 판정한다', () => {
  const block = resizeHoverBlock();
  const fallbackIdx = block.indexOf('직전 표 bbox 캐시로 한 번 더 경계선을 확인');
  const clearCacheIdx = block.indexOf('this.cachedTableRef = null');

  assert.notEqual(fallbackIdx, -1, 'hitTest 실패 시 캐시 기반 hover fallback 필요');
  assert.notEqual(clearCacheIdx, -1, '표 밖에서는 캐시 정리 경로 유지 필요');
  assert.ok(fallbackIdx < clearCacheIdx, '캐시를 지우기 전에 경계선 fallback을 먼저 수행해야 함');
  assert.match(block, /this\.cachedCellBboxes\.filter/, 'fallback은 직전 bbox 캐시를 사용해야 함');
  assert.match(block, /hitTestBorder\(pageX,\s*pageY,\s*pageBboxes\)/, 'fallback도 경계선 hitTest를 사용해야 함');
});

test('표 경계 mousedown은 hover 캐시가 없으면 새 table bbox를 만들지 않는다', () => {
  const helper = resolveTableResizeHitBlock();
  const block = generalTableResizeMouseDownBlock();

  assert.match(helper, /function resolveTableResizeHit/, 'mousedown 전용 table resize hit helper 필요');
  assert.match(helper, /self\.cachedCellBboxes/, 'mousedown resize는 기존 bbox 캐시만 사용해야 함');
  assert.doesNotMatch(helper, /self\.wasm\.hitTest\(pageIdx,\s*pageX,\s*pageY\)/, '일반 mousedown에서 표 hitTest 기반 cold lookup 금지');
  assert.doesNotMatch(helper, /self\.wasm\.getTableCellBboxes/, '일반 mousedown에서 table bbox 생성 금지');
  assert.doesNotMatch(helper, /self\.wasm\.getPageControlLayout\(pageIdx\)/, '일반 mousedown에서 layout fallback 기반 cold lookup 금지');
  assert.match(block, /const resizeHit = resolveTableResizeHit\(this,\s*pageIdx,\s*pageX,\s*pageY\);/, '일반 mousedown resize는 helper를 사용해야 함');
});

// #4117: 셀 선택 클릭 전에는 아무도 bbox 캐시를 채우지 않아 표 경계 리사이즈가
// 시작되지 않았다. 이제 hover 가 단일 채움 지점(ensureTableCellBboxCache)으로
// 캐시를 채우되, task 2010 이 막은 "이동마다 표 전체 재계산"은 캐시·실패 메모가
// 계속 막는다 (행동 계약은 tests/table-resize-bbox-cache.test.ts).
test('표 경계 hover는 셀 선택 없이도 choke point로 캐시를 채운다 (#4117)', () => {
  const block = resizeHoverBlock();
  assert.match(
    block,
    /ensureTableCellBboxCache\(this,\s*tableRef,\s*pageIdx\)/,
    'hover 는 단일 채움 지점으로만 캐시를 채워야 함',
  );
  assert.doesNotMatch(
    block,
    /this\.wasm\.getTableCellBboxes/,
    'hover 에서 메모이제이션 없는 직접 엔진 조회 금지',
  );

  const cache = source('src/engine/table-bbox-cache.ts');
  assert.match(
    cache,
    /getTableCellBboxes\(tableRef\.sec,\s*tableRef\.ppi,\s*tableRef\.ci,\s*pageIdx\)/,
    '조회는 현재 페이지를 hint 로 전달해야 함 — 없으면 페이지 0부터 스윕',
  );
  assert.match(
    cache,
    /tableBboxFetchFailure/,
    '실패 메모 없이 이동마다 재시도하면 task 2010 이 막은 랙이 돌아온다',
  );

  const handler = source('src/engine/input-handler.ts');
  assert.match(
    handler,
    /this\.tableBboxFetchFailures\.clear\(\);/,
    '문서 변경(clearTableResizeRuntimeCache) 시 실패 메모도 함께 비워야 함',
  );
});

test('셀 선택 경로의 bbox 조회도 현재 페이지를 hint 로 전달한다 (#4117)', () => {
  const block = cellSelectionMouseDownBlock();
  assert.match(
    block,
    /getTableCellBboxes\(ctx\.sec,\s*ctx\.ppi,\s*ctx\.ci,\s*pageIdx\)/,
    'hint 없이 부르면 엔진이 페이지 0부터 렌더 트리를 훑는다',
  );
});

test('표 경계 hitTest는 교차점에서 행 경계 선반환으로 컬럼 resize를 막지 않는다', () => {
  const block = hitTestBorderBlock();

  assert.match(block, /const candidates/, '행/열 후보를 함께 모아야 함');
  assert.match(block, /type:\s*'col'[\s\S]*priority:\s*0/, '동률일 때 컬럼 후보를 우선해야 함');
  assert.match(block, /type:\s*'row'[\s\S]*priority:\s*1/, '행 후보는 컬럼 동률 우선순위 뒤에 있어야 함');
  assert.match(block, /candidates\.sort\(\(a,\s*b\) => a\.distance - b\.distance \|\| a\.priority - b\.priority\)/, '가장 가까운 경계를 고르고 동률은 컬럼 우선이어야 함');
});

test('drag 중 Shift 입력은 persistent resize를 local projection으로 승격하지 않는다', () => {
  const table = inputHandlerTableSource();

  assert.doesNotMatch(table, /promoteResizeDragToSingleCell/);
  assert.doesNotMatch(table, /state\.singleCellTarget = state\.resizeTarget/);
  assert.doesNotMatch(table, /resizeDragState\s*=\s*\{[\s\S]*singleCellTarget/);
});

test('Shift 단일 셀 resize target 판정은 hover와 같은 경계 허용폭을 사용한다', () => {
  const table = inputHandlerTableSource();

  assert.match(
    table,
    /function findSingleCellResizeTarget[\s\S]*const tolerance = 4\.0;/,
    'hover로 표시된 경계는 mousedown에서도 같은 허용폭으로 단일 셀 resize target을 잡아야 함',
  );
});

test('persistent drag marker와 finish는 Shift modifier로 projection owner를 바꾸지 않는다', () => {
  const update = updateResizeDragBlock();
  const finish = finishResizeDragBlock();

  assert.doesNotMatch(update, /singleCellTarget|markerBboxes|e\.shiftKey/);
  assert.doesNotMatch(finish, /promoteResizeDragToSingleCell|const singleCellTarget/);
  assert.match(finish, /if \(shouldSelectTable\)/, '작은 일반 외곽 drag는 종전 표 선택 흐름을 유지해야 함');
});

test('저장 불가능한 drag guard는 blocking UI 없이 WASM mutation 전에 종료한다', () => {
  const finish = finishResizeDragBlock();
  const legacyLocalGuardIdx = finish.indexOf('if (updates.some(update => update.localResize === true)) {');
  const wasmMutationIdx = finish.indexOf('wasm.resizeTableCells(');

  assert.notEqual(legacyLocalGuardIdx, -1, 'legacy localResize payload guard 필요');
  assert.notEqual(wasmMutationIdx, -1, '일반 persistent drag의 WASM mutation 경로는 유지해야 함');
  assert.ok(
    legacyLocalGuardIdx < wasmMutationIdx,
    'stale derived-geometry guard가 source mutation보다 먼저 실행돼야 함',
  );

  const legacyLocalGuard = finish.slice(legacyLocalGuardIdx, finish.indexOf('\n  // WASM 배치 API', legacyLocalGuardIdx));
  assert.match(legacyLocalGuard, /this\.cleanupResizeDrag\(\);\s*return;/, 'stale guard는 drag lifecycle을 닫아야 함');
  assert.doesNotMatch(legacyLocalGuard, /showToast/, 'stale guard는 편집 화면을 가리는 UI를 만들면 안 됨');
});

// 병합 셀이 섞인 표에서 마우스 드래그 리사이즈가 셀 선택 범위 내 병합 셀을
// 누락시키지 않는다. 이 파일의 다른 테스트와 달리 finishResizeDrag 가 실제로 쓰는
// cellOverlapsSelectionRange 를 직접 호출해 검증한다.

function mergedGridBbox(row: number, col: number, rowSpan: number, colSpan: number): CellBbox {
  return { cellIdx: row * 100 + col, row, col, rowSpan, colSpan, pageIndex: 0, x: 0, y: 0, w: 40, h: 20 };
}

test('세로 병합 셀은 시작 행이 선택 범위 밖이어도 하위 행 선택과 겹친다', () => {
  // row0~1 세로 병합 셀(col0). 사용자가 row1 만 셀 선택한 채 열 경계를 드래그하는 시나리오 —
  // 시작 좌표 비교(row >= startRow)라면 이 병합 셀이 통째로 빠진다.
  const merged = mergedGridBbox(0, 0, 2, 1);
  const range = { startRow: 1, startCol: 0, endRow: 1, endCol: 0 };
  assert.equal(cellOverlapsSelectionRange(merged, range), true);
});

test('가로 병합 셀은 시작 열이 선택 범위 밖이어도 하위 열 선택과 겹친다', () => {
  const merged = mergedGridBbox(0, 0, 1, 3);
  const range = { startRow: 0, startCol: 2, endRow: 0, endCol: 2 };
  assert.equal(cellOverlapsSelectionRange(merged, range), true);
});

test('선택 범위와 겹치지 않는 셀은 여전히 제외된다', () => {
  const plain = mergedGridBbox(2, 2, 1, 1);
  const range = { startRow: 0, startCol: 0, endRow: 1, endCol: 1 };
  assert.equal(cellOverlapsSelectionRange(plain, range), false);
});

test('finishResizeDrag 선택 셀 필터는 overlap 판정을 사용한다', () => {
  const finish = finishResizeDragBlock();
  assert.match(
    finish,
    /cellOverlapsSelectionRange\(b, range\)/,
    '마우스 경로 선택 필터는 시작 좌표 비교가 아니라 cellOverlapsSelectionRange 를 써야 함',
  );
  assert.doesNotMatch(
    finish,
    /b\.row >= range\.startRow/,
    '시작 좌표 직접 비교가 남아 있으면 병합 셀 누락이 재발한다',
  );
  assert.match(
    finish,
    /buildCellSelectionColumnDragUpdates\(selectedBboxes, state\.bboxes, deltaHwpUnit\)/,
    '셀 선택 열 드래그 update 구성은 추출된 순수 함수를 써야 함',
  );
});

test('병합 셀 열 드래그는 걸친 모든 행의 오른쪽 이웃을 보상한다', () => {
  // 3x2 표, col0 rows0-1 세로 병합. 경계 왼쪽 선택 셀 = 병합 셀 + (2,0).
  // 실측(2026-09-01, headless studio): 시작 행 이웃만 보상하면 row1 의 열 폭 합이
  // 어긋나 병합 셀이 최소폭으로 붕괴했다 (279.7px → 24px). 걸친 행 전부의 이웃
  // (0,1)·(1,1)·(2,1) 이 반대 delta 를 받아야 행별 합이 유지된다.
  const merged = mergedGridBbox(0, 0, 2, 1);
  const r2c0 = mergedGridBbox(2, 0, 1, 1);
  const r0c1 = mergedGridBbox(0, 1, 1, 1);
  const r1c1 = mergedGridBbox(1, 1, 1, 1);
  const r2c1 = mergedGridBbox(2, 1, 1, 1);
  const all = [merged, r0c1, r1c1, r2c0, r2c1];

  const updates = buildCellSelectionColumnDragUpdates([merged, r2c0], all, 100);
  const byCell = new Map(updates.map(u => [u.cellIdx, u.widthDelta]));

  assert.equal(byCell.get(merged.cellIdx), 100, '병합 셀은 +delta');
  assert.equal(byCell.get(r2c0.cellIdx), 100, '(2,0) 은 +delta');
  assert.equal(byCell.get(r0c1.cellIdx), -100, '(0,1) 은 -delta');
  assert.equal(byCell.get(r1c1.cellIdx), -100, '(1,1) 도 -delta — 시작 행만 보상하면 여기가 빠진다');
  assert.equal(byCell.get(r2c1.cellIdx), -100, '(2,1) 은 -delta');
  assert.equal(updates.length, 5, '표의 다섯 셀 전부가 정확히 한 번씩 update 를 받는다');
});

test('일반 모드 경계 드래그 보상 이웃은 병합 셀이 걸친 모든 줄을 쓴다', () => {
  // 실측(2026-09-01, headless studio): 2x3 표에서 row0 cols0-1 을 가로 병합하고
  // row0|row1 경계를 드래그하면, 아래 이웃 보상이 병합 셀의 시작 열 이웃만 찾아
  // (1,1) 이 -delta 를 받지 못했다. 드래그가 절반만 먹고 표 전체 높이가
  // 194.2px → 208.2px 로 불었다. 걸친 모든 열의 이웃이 보상을 받아야 한다.
  const hMerged = mergedGridBbox(0, 0, 1, 2); // row0, cols0-1 가로 병합
  const r02 = mergedGridBbox(0, 2, 1, 1);
  const r10 = mergedGridBbox(1, 0, 1, 1);
  const r11 = mergedGridBbox(1, 1, 1, 1);
  const r12 = mergedGridBbox(1, 2, 1, 1);
  const all = [hMerged, r02, r10, r11, r12];

  const below = findResizeCompensationNeighbors({ type: 'row', index: 1 } as any, hMerged, all);
  assert.deepEqual(
    below.map(b => b.cellIdx).sort((a, b) => a - b),
    [r10.cellIdx, r11.cellIdx].sort((a, b) => a - b),
    '가로 병합 셀의 행 경계 보상은 걸친 두 열의 아래 이웃 모두여야 한다',
  );

  // 세로 병합(rowSpan=2) 열 경계 대칭: 걸친 두 행의 오른쪽 이웃 모두
  const vMerged = mergedGridBbox(0, 0, 2, 1);
  const c01 = mergedGridBbox(0, 1, 1, 1);
  const c11 = mergedGridBbox(1, 1, 1, 1);
  const right = findResizeCompensationNeighbors({ type: 'col', index: 1 } as any, vMerged, [vMerged, c01, c11]);
  assert.deepEqual(
    right.map(b => b.cellIdx).sort((a, b) => a - b),
    [c01.cellIdx, c11.cellIdx].sort((a, b) => a - b),
    '세로 병합 셀의 열 경계 보상은 걸친 두 행의 오른쪽 이웃 모두여야 한다',
  );

  // 병합 없는 셀은 종전처럼 이웃 하나
  const single = findResizeCompensationNeighbors({ type: 'row', index: 1 } as any, r02, all);
  assert.deepEqual(single.map(b => b.cellIdx), [r12.cellIdx]);
});

test('병합 없는 표의 열 드래그 보상은 기존과 같다', () => {
  const r0c0 = mergedGridBbox(0, 0, 1, 1);
  const r1c0 = mergedGridBbox(1, 0, 1, 1);
  const r0c1 = mergedGridBbox(0, 1, 1, 1);
  const r1c1 = mergedGridBbox(1, 1, 1, 1);
  const all = [r0c0, r0c1, r1c0, r1c1];

  const updates = buildCellSelectionColumnDragUpdates([r1c0], all, 100);
  assert.deepEqual(
    updates,
    [
      { cellIdx: r1c0.cellIdx, widthDelta: 100 },
      { cellIdx: r1c1.cellIdx, widthDelta: -100 },
    ],
    '선택된 (1,0) 과 그 행의 오른쪽 이웃만 반대 delta 를 받는다',
  );
});
