import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function deleteRowColumnBlock(): string {
  const tableCmd = source('src/command/commands/table.ts');
  const start = tableCmd.indexOf('function applyTableDeleteRowColumn(');
  assert.notEqual(start, -1, 'applyTableDeleteRowColumn not found');
  const end = tableCmd.indexOf('\nexport const tableCommands', start);
  assert.notEqual(end, -1, 'tableCommands list after delete fn not found');
  return tableCmd.slice(start, end);
}

function clampHelperBlock(): string {
  const tableCmd = source('src/command/commands/table.ts');
  const start = tableCmd.indexOf('function clampedCellAfterDelete(');
  assert.notEqual(start, -1, 'clampedCellAfterDelete not found');
  const end = tableCmd.indexOf('function applyTableDeleteRowColumn(', start);
  assert.notEqual(end, -1, 'applyTableDeleteRowColumn after helper not found');
  return tableCmd.slice(start, end);
}

// #1483: 표 줄/칸 지우기 후 커서 cellIndex 보정 — 삭제로 줄어든 셀 범위 초과 방지.
test('줄/칸 지우기는 삭제 후 커서를 보정한다 (return pos 단독 금지)', () => {
  const block = deleteRowColumnBlock();

  // deleteTable* 반환값을 받아 ok/rowCount/colCount를 활용해야 한다.
  assert.match(block, /const res =/, 'deleteTable* 반환값을 받아야 함');
  assert.match(block, /res\.ok/, 'res.ok 분기 필요');
  assert.match(block, /res\.rowCount/, 'rowCount 사용 필요');
  assert.match(block, /res\.colCount/, 'colCount 사용 필요');
  // 보정 헬퍼를 호출해야 한다.
  assert.match(block, /clampedCellAfterDelete\(/, 'clampedCellAfterDelete 호출 필요');
  // 보정 없이 원래 위치만 반환하던 회귀(operation이 사실상 return pos만) 금지.
  // res.ok 실패 시의 폴백 return pos는 허용하되, 정상 경로는 보정 위치를 반환해야 한다.
  assert.match(block, /\.\.\.corrected/, '보정된 cellIndex를 커서에 반영해야 함');
  // 표 소멸 폴백.
  assert.match(block, /paragraphIndex:\s*pos\.parentParaIndex/, '표 소멸 시 본문 폴백 필요');
});

test('clampedCellAfterDelete는 범위 clamp + bbox 역조회 + 소멸 가드를 갖춘다', () => {
  const helper = clampHelperBlock();

  assert.match(helper, /rowCount <= 0 \|\| colCount <= 0/, '표 소멸 가드 필요');
  assert.match(helper, /Math\.min\(origRow,\s*rowCount - 1\)/, 'row clamp 필요');
  assert.match(helper, /Math\.min\(origCol,\s*colCount - 1\)/, 'col clamp 필요');
  assert.match(helper, /getTableCellBboxes\(/, 'cellIdx 역조회에 bbox 사용 필요');
  // 병합 셀 매칭 (rowSpan/colSpan 범위 포함).
  assert.match(helper, /b\.rowSpan/, '병합 셀 rowSpan 매칭 필요');
  assert.match(helper, /b\.colSpan/, '병합 셀 colSpan 매칭 필요');
});
