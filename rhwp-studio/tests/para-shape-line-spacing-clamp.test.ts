import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('문단 모양 줄 간격 입력값은 collectMods에서 clampLineSpacing으로 min/max(0~9999)에 맞춰진다 (#2959)', () => {
  const dialog = source('src/ui/para-shape-dialog.ts');

  assert.match(
    dialog,
    /function clampLineSpacing\(value: number\): number \{\s*return Math\.max\(0, Math\.min\(9999, value\)\);\s*\}/,
    'clampLineSpacing 헬퍼가 0~9999 범위로 클램프해야 한다',
  );

  const collectStart = dialog.indexOf('private collectMods(');
  const collectEnd = dialog.indexOf('// 문단 간격', collectStart);
  assert.notEqual(collectStart, -1, 'collectMods 블록을 찾을 수 있어야 한다');
  const block = dialog.slice(collectStart, collectEnd);

  assert.match(block, /newLS = clampLineSpacing\(parseInt\(this\.lineSpacingInput\.value\) \|\| 160\)/);
  assert.match(block, /newLS = ptToRaw\(clampLineSpacing\(parseFloat\(this\.lineSpacingInput\.value\) \|\| 0\)\)/);
});
