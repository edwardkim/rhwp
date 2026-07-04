import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function commandBlock(commandId: string): string {
  const tableCmd = source('src/command/commands/table.ts');
  const start = tableCmd.indexOf(`id: '${commandId}'`);
  assert.notEqual(start, -1, `${commandId} command not found`);
  const end = tableCmd.indexOf('\n  {', start + 1);
  assert.notEqual(end, -1, `${commandId} command end not found`);
  return tableCmd.slice(start, end);
}

// #1491: 셀 너비 균등화는 현재 표시 bbox 폭을 기준으로 로컬 resize 힌트를 남겨야 한다.
test('셀 너비를 같게는 bbox 표시 폭 기준으로 평균을 계산한다', () => {
  const block = commandBlock('table:cell-width-equal');

  assert.match(block, /getTableCellBboxes\(sec,\s*ppi,\s*ci\)/, '현재 렌더 bbox를 읽어야 함');
  assert.match(block, /Math\.round\(bbox\.w \* 75\)/, 'bbox 폭을 HWPUNIT 표시 폭으로 변환해야 함');
  assert.match(block, /sum \+ cell\.renderWidth/, '평균 계산은 모델 폭이 아니라 표시 폭 기준이어야 함');
  assert.match(block, /totalWidth \/ cells\.length/, '선택된 실제 셀 개수를 기준으로 평균을 내야 함');
  assert.doesNotMatch(block, /sum \+ cell\.width/, '표시 폭 대신 모델 폭으로 평균을 내면 안 됨');
});

test('셀 너비를 같게는 선택 셀마다 localResize/renderWidth 힌트를 보낸다', () => {
  const block = commandBlock('table:cell-width-equal');

  assert.doesNotMatch(block, /info\.colSpan > 1/, '가로 병합 셀도 균등화 대상에 포함해야 함');
  assert.match(block, /localResize:\s*true/, '로컬 행 resize 의도를 전달해야 함');
  assert.match(block, /renderWidth:\s*avgWidth/, '목표 렌더 폭을 WASM에 전달해야 함');
  assert.match(block, /c\.renderWidth !== avgWidth/, 'delta가 0이어도 표시 폭이 다르면 실행해야 함');
  assert.doesNotMatch(
    block,
    /if \(delta !== 0\) updates\.push/,
    'delta가 있는 셀만 보내면 선택 행 힌트가 불완전해짐',
  );
});

test('셀 너비를 같게는 마우스 resize와 같은 operation 경로로 적용한다', () => {
  const block = commandBlock('table:cell-width-equal');

  assert.match(block, /ih\.executeOperation\({/, '직접 WASM 호출이 아니라 InputHandler operation 경로를 사용해야 함');
  assert.match(block, /operationType:\s*'equalizeTableCellWidths'/, '너비 균등화 전용 operationType을 기록해야 함');
  assert.match(block, /wasm\.resizeTableCells\(sec,\s*ppi,\s*ci,\s*updates\)/, 'snapshot operation 내부에서 resizeTableCells를 호출해야 함');
  assert.doesNotMatch(block, /services\.wasm\.resizeTableCells\(sec,\s*ppi,\s*ci,\s*updates\);/, '직접 호출하면 refresh/undo/cache 경로를 우회함');
  assert.match(block, /restoreEditorFocus\(ih\)/, '컨텍스트 메뉴 실행 후 편집 포커스를 복원해야 함');
});

test('셀 높이를 같게는 bbox 표시 높이 기준으로 평균을 계산한다', () => {
  const block = commandBlock('table:cell-height-equal');

  assert.match(block, /getTableCellBboxes\(sec,\s*ppi,\s*ci\)/, '현재 렌더 bbox를 읽어야 함');
  assert.match(block, /Math\.round\(bbox\.h \* 75\)/, 'bbox 높이를 HWPUNIT 표시 높이로 변환해야 함');
  assert.match(block, /sum \+ cell\.renderHeight/, '평균 계산은 모델 높이가 아니라 표시 높이 기준이어야 함');
  assert.match(block, /totalHeight \/ cells\.length/, '선택된 실제 셀 개수를 기준으로 평균을 내야 함');
  assert.doesNotMatch(block, /sum \+ c\.height/, '표시 높이 대신 모델 높이로 평균을 내면 안 됨');
});

test('셀 높이를 같게는 선택 셀마다 localResize/renderHeight 힌트를 보낸다', () => {
  const block = commandBlock('table:cell-height-equal');

  assert.match(block, /localResize:\s*true/, '로컬 세로 resize 의도를 전달해야 함');
  assert.match(block, /renderHeight:\s*avgHeight/, '목표 렌더 높이를 WASM에 전달해야 함');
  assert.match(block, /heightDelta:\s*0/, '셀 높이 균등화는 행 모델 높이를 직접 흔들면 안 됨');
  assert.match(block, /c\.renderHeight !== avgHeight/, 'delta가 0이어도 표시 높이가 다르면 실행해야 함');
});

test('셀 높이를 같게는 마우스 resize와 같은 operation 경로로 적용한다', () => {
  const block = commandBlock('table:cell-height-equal');

  assert.match(block, /ih\.executeOperation\({/, '직접 WASM 호출이 아니라 InputHandler operation 경로를 사용해야 함');
  assert.match(block, /operationType:\s*'equalizeTableCellHeights'/, '높이 균등화 전용 operationType을 기록해야 함');
  assert.match(block, /wasm\.resizeTableCells\(sec,\s*ppi,\s*ci,\s*updates\)/, 'snapshot operation 내부에서 resizeTableCells를 호출해야 함');
  assert.doesNotMatch(block, /services\.wasm\.resizeTableCells\(sec,\s*ppi,\s*ci,\s*updates\);/, '직접 호출하면 refresh/undo/cache 경로를 우회함');
  assert.match(block, /restoreEditorFocus\(ih\)/, '컨텍스트 메뉴 실행 후 편집 포커스를 복원해야 함');
});
