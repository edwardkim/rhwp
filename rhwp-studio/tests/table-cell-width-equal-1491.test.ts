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
  test(`${commandId}은 저장 불가능한 projection을 만들지 않고 사용자에게 이유를 보인다`, () => {
    const block = commandBlock(commandId);
    assert.match(block, /showToast\(\{ message: LOCAL_TABLE_RESIZE_UNSUPPORTED_MESSAGE \}\)/);
    assert.doesNotMatch(block, /localResize|renderWidth|renderHeight|resizeTableCells/);
  });
}

test('균등화 안내는 포맷 제약을 구체적으로 설명한다', () => {
  const tableCmd = source('src/command/commands/table.ts');
  assert.match(tableCmd, /LOCAL_TABLE_RESIZE_UNSUPPORTED_MESSAGE/);
  const policy = source('src/engine/table-resize-updates.ts');
  assert.match(policy, /HWP\/HWPX.*저장할 수 없어 지원하지 않습니다/);
});
