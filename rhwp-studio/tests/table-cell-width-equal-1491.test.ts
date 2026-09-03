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

for (const commandId of ['table:cell-width-equal', 'table:cell-height-equal']) {
  test(`${commandId}은 저장 불가능한 projection과 blocking UI를 만들지 않는다`, () => {
    const block = commandBlock(commandId);
    assert.match(block, /canExecute:\s*localTableGeometryCanPersist/);
    assert.doesNotMatch(block, /showToast|localResize|renderWidth|renderHeight|resizeTableCells/);
  });
}

test('균등화 command routing은 포맷이 저장할 수 없는 geometry를 비활성화한다', () => {
  const tableCmd = source('src/command/commands/table.ts');
  assert.match(tableCmd, /const localTableGeometryCanPersist = \(\) => false/);
});
