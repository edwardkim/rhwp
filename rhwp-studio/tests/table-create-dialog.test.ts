import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('표 만들기 상세 대화상자의 기본값은 글자처럼 취급 해제이다', () => {
  const dialog = source('src/ui/table-create-dialog.ts');
  const inputStart = dialog.indexOf("treatChk.id = 'tc-treat-as-char'");
  const applyStart = dialog.indexOf('const options: TableCreateOptions', inputStart);
  assert.notEqual(inputStart, -1, '글자처럼 취급 checkbox 초기화 블록을 찾을 수 있어야 한다');
  assert.notEqual(applyStart, -1, '표 만들기 적용 블록을 찾을 수 있어야 한다');

  const block = dialog.slice(inputStart, applyStart);
  assert.match(block, /treatChk\.checked\s*=\s*false/);
  assert.doesNotMatch(block, /treatChk\.checked\s*=\s*true/);
});
