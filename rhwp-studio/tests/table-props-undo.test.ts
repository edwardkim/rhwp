import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 표/셀 속성·셀 테두리/배경 다이얼로그는 수십 개 속성을 한 번에 덮어쓰는
// 문서 mutation 이므로 반드시 편집 라우터(executeOperation)를 통과해 undo
// 스택에 기록되어야 한다 (#1320 계약, picture-props-dialog #2027 과 동일).

test('표/셀 속성 다이얼로그 onConfirm은 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/table-cell-props-dialog.ts');
  const start = dialog.indexOf('protected onConfirm');
  assert.notEqual(start, -1, 'onConfirm not found');
  const block = dialog.slice(start, dialog.indexOf('\n  private ', start + 1));

  assert.match(
    block,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'objectProps'/,
    '표/셀 속성 적용은 snapshot 명령으로 기록되어야 함',
  );
});

test('셀 테두리/배경 다이얼로그 onConfirm은 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/cell-border-bg-dialog.ts');
  const start = dialog.indexOf('protected onConfirm');
  assert.notEqual(start, -1, 'onConfirm not found');
  const block = dialog.slice(start, dialog.indexOf('\n  private ', start + 1));

  assert.match(
    block,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'objectProps'/,
    '셀 테두리/배경 적용은 snapshot 명령으로 기록되어야 함',
  );
});

test('두 다이얼로그 진입점 모두 CommandServices를 전달한다', () => {
  const table = source('src/command/commands/table.ts');
  const format = source('src/command/commands/format.ts');

  assert.match(
    table,
    /new TableCellPropsDialog\(services\.wasm, services\.eventBus, tableCtx, 0, 'table', services\)/,
    'table:cell-props(표 선택) 진입점에서 services 전달 필요',
  );
  assert.match(
    table,
    /new TableCellPropsDialog\(services\.wasm, services\.eventBus, tableCtx, pos\.cellIndex, 'cell', services\)/,
    'table:cell-props(셀 커서) 진입점에서 services 전달 필요',
  );
  assert.match(
    format,
    /new TableCellPropsDialog\(services\.wasm, services\.eventBus, tableCtx, pos\.cellIndex, 'table', services\)/,
    'format:object-properties 진입점에서 services 전달 필요',
  );
  const borderDialogCalls = table.match(/new CellBorderBgDialog\([\s\S]*?\);/g) ?? [];
  assert.equal(borderDialogCalls.length, 2, 'CellBorderBgDialog 진입점 2곳');
  for (const call of borderDialogCalls) {
    assert.match(call, /services,\s*\);$/, '셀 테두리/배경 진입점에서 services 전달 필요');
  }
});
