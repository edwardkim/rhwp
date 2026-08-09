import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = readFileSync(join(rootDir, 'src/engine/input-handler-keyboard.ts'), 'utf8');

test('#4272 중첩 셀 Ctrl+C는 plain text와 HTML을 모두 전체 path API로 복사한다', () => {
  const onCopyStart = source.indexOf('export function onCopy');
  const onCutStart = source.indexOf('export function onCut', onCopyStart);
  assert.notEqual(onCopyStart, -1, 'onCopy를 찾지 못함');
  assert.notEqual(onCutStart, -1, 'onCopy 끝을 찾지 못함');
  const onCopy = source.slice(onCopyStart, onCutStart);

  assert.match(onCopy, /if \(isNestedCellPosition\(start\)\)/);
  assert.match(onCopy, /copySelectionInCellByPath\([\s\S]*JSON\.stringify\(start\.cellPath\)/);
  assert.match(onCopy, /exportSelectionInCellHtmlByPath\([\s\S]*JSON\.stringify\(start\.cellPath\)/);
  assert.match(onCopy, /cellParaIndexOf\(start\)[\s\S]*cellParaIndexOf\(end\)/);
  assert.match(onCopy, /else if \(start\.parentParaIndex !== undefined\)[\s\S]*copySelectionInCell\(/);
});
