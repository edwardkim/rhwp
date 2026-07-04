import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

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

// #1491 후속: Shift+경계선 드래그는 셀 선택 확장보다 resize 판정이 우선해야 한다.
test('셀 선택 모드 Shift+경계선 클릭은 확장 선택보다 리사이즈를 먼저 시도한다', () => {
  const block = cellSelectionMouseDownBlock();
  const resizeIdx = block.indexOf('this.startResizeDrag(edge, pageX, pageY, pageBboxes, e.shiftKey)');
  const shiftSelectIdx = block.indexOf('if (e.shiftKey || e.ctrlKey || e.metaKey)');

  assert.notEqual(resizeIdx, -1, '경계선 resize 시작 경로 필요');
  assert.notEqual(shiftSelectIdx, -1, 'Shift/Ctrl 셀 선택 경로 필요');
  assert.ok(
    resizeIdx < shiftSelectIdx,
    '경계선 위 Shift+마우스는 셀 선택 확장이 아니라 단일 셀 resize로 들어가야 함',
  );
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

test('표 경계 mousedown은 hover 캐시가 없어도 현재 좌표에서 table bbox를 복구한다', () => {
  const mouse = source('src/engine/input-handler-mouse.ts');
  const block = generalTableResizeMouseDownBlock();

  assert.match(mouse, /function resolveTableResizeHit/, 'mousedown 전용 table resize hit 복구 helper 필요');
  assert.match(mouse, /self\.wasm\.hitTest\(pageIdx,\s*pageX,\s*pageY\)/, '셀 내부 hitTest로 table ref를 복구해야 함');
  assert.match(mouse, /self\.wasm\.getPageControlLayout\(pageIdx\)/, '경계선 위 hitTest 실패 시 layout fallback이 필요');
  assert.match(block, /const resizeHit = resolveTableResizeHit\(this,\s*pageIdx,\s*pageX,\s*pageY\);/, '일반 mousedown resize는 helper를 사용해야 함');
  assert.doesNotMatch(
    block,
    /this\.tableResizeRenderer && this\.cachedCellBboxes && this\.cachedTableRef/,
    'hover 캐시가 없다는 이유로 mousedown resize를 포기하면 안 됨',
  );
});

test('표 경계 hitTest는 교차점에서 행 경계 선반환으로 컬럼 resize를 막지 않는다', () => {
  const block = hitTestBorderBlock();

  assert.match(block, /const candidates/, '행/열 후보를 함께 모아야 함');
  assert.match(block, /type:\s*'col'[\s\S]*priority:\s*0/, '동률일 때 컬럼 후보를 우선해야 함');
  assert.match(block, /type:\s*'row'[\s\S]*priority:\s*1/, '행 후보는 컬럼 동률 우선순위 뒤에 있어야 함');
  assert.match(block, /candidates\.sort\(\(a,\s*b\) => a\.distance - b\.distance \|\| a\.priority - b\.priority\)/, '가장 가까운 경계를 고르고 동률은 컬럼 우선이어야 함');
});

test('Shift가 drag 중 확인되어도 시작 시 계산한 단일 셀 후보를 resize 대상으로 승격한다', () => {
  const table = inputHandlerTableSource();

  assert.match(table, /resizeTarget,/, 'drag state에 시작 시 계산한 단일 셀 후보를 보존해야 함');
  assert.match(table, /function promoteResizeDragToSingleCell/, '동적 Shift 승격 헬퍼가 필요');
  assert.doesNotMatch(table, /if \(state\.edge\?\.type !== 'col'\) return null;/, '세로 경계도 가로와 같은 Shift 단일 셀 resize 승격 대상이어야 함');
  assert.match(table, /if \(!shiftKey \|\| !state\.resizeTarget\) return null;/, 'Shift가 없으면 일반 resize 흐름을 유지해야 함');
  assert.match(table, /state\.singleCellTarget = state\.resizeTarget;/, 'Shift 확인 시 후보를 단일 셀 대상으로 승격해야 함');
  assert.match(table, /state\.shiftResize = true;/, '승격된 resize는 Shift 단일 셀 resize로 기록해야 함');
  assert.match(table, /state\.minResizePos = resizeBounds\.min;/, '승격 후 단일 셀 bounds를 다시 적용해야 함');
  assert.match(table, /state\.maxResizePos = resizeBounds\.max;/, '승격 후 단일 셀 bounds를 다시 적용해야 함');
});

test('Shift 단일 셀 resize target 판정은 hover와 같은 경계 허용폭을 사용한다', () => {
  const table = inputHandlerTableSource();

  assert.match(
    table,
    /function findSingleCellResizeTarget[\s\S]*const tolerance = 4\.0;/,
    'hover로 표시된 경계는 mousedown에서도 같은 허용폭으로 단일 셀 resize target을 잡아야 함',
  );
});

test('Shift drag marker와 finish 적용은 같은 단일 셀 승격 대상을 사용한다', () => {
  const update = updateResizeDragBlock();
  const finish = finishResizeDragBlock();

  assert.match(update, /const singleCellTarget = promoteResizeDragToSingleCell\(this,\s*this\.resizeDragState,\s*e\.shiftKey\);/, 'marker 표시 전에 Shift 단일 셀 후보를 승격해야 함');
  assert.match(update, /const markerBboxes = singleCellTarget/, 'marker는 승격된 단일 셀 후보로 제한해야 함');
  assert.match(finish, /const singleCellTarget = promoteResizeDragToSingleCell\(this,\s*state,\s*e\.shiftKey\);/, 'finish 적용 전에 Shift 단일 셀 후보를 승격해야 함');
  assert.match(finish, /if \(shouldSelectTable && !singleCellTarget\)/, '승격된 단일 셀 resize는 작은 드래그에서 표 선택으로 바뀌면 안 됨');
});

test('Shift 세로 resize는 가로처럼 단일 셀 local height 경로를 사용한다', () => {
  const table = inputHandlerTableSource();
  const finish = finishResizeDragBlock();

  assert.match(table, /const shouldResizeSingleCell = shiftResize \|\|/, 'Shift 단일 셀 resize는 가로와 세로 경계 모두에 적용해야 함');
  assert.match(table, /shiftResize: shouldResizeSingleCell,/, '세로 Shift drag state도 local single-cell resize로 기록해야 함');
  assert.match(
    finish,
    /else if \(state\.edge\.type === 'col' && inCellSel && range\)/,
    'Shift 없는 세로 경계는 셀 선택 모드여도 선택 셀 전용 보상이 아니라 행 전체 resize로 가야 함',
  );
  assert.match(
    finish,
    /if \(box\.col !== targetBox\.col\) continue;[\s\S]*pushLocalResizeHeightHint\(updates, box\.cellIdx, getCellDisplaySize\(box, state\.edge\)\);/,
    '세로 Shift 단일 셀 resize는 같은 열의 나머지 셀 현재 높이를 보존 힌트로 유지해야 함',
  );
  assert.match(finish, /heightDelta:\s*0,[\s\S]*renderHeight: targetDesiredSize,/, '세로 Shift는 모델 행 높이가 아니라 target renderHeight만 바꿔야 함');
  assert.match(finish, /heightDelta:\s*0,[\s\S]*renderHeight: neighborDesiredSize,/, '세로 Shift 보상 셀도 모델 행 높이가 아니라 renderHeight만 바꿔야 함');
});
