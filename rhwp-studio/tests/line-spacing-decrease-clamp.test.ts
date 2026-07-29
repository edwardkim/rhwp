import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// format:line-spacing-decrease(Alt+Shift+A)에 하한 clamp가 없어 연타 시 줄 간격이
// 0 이하/음수로 내려가는 문제(issue #3009)의 소스 가드. 짝인
// format:line-spacing-increase 는 이미 Math.min(500, ...)로 상한 clamp가 있었는데
// decrease 쪽에는 하한이 없었다. toolbar.ts ▼ 버튼의 Math.max(5, cur - 5)와 동일하게
// 5%로 하한을 맞춘다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const formatSrc = readFileSync(join(rootDir, 'src/command/commands/format.ts'), 'utf8');

test('format:line-spacing-decrease has a lower-bound clamp', () => {
  const idx = formatSrc.indexOf("id: 'format:line-spacing-decrease'");
  assert.ok(idx >= 0, 'format:line-spacing-decrease 커맨드를 찾을 수 없음');
  const block = formatSrc.slice(idx, idx + 500);
  assert.match(
    block,
    /Math\.max\(\s*5\s*,\s*current\s*-\s*10\s*\)/,
    'decrease 커맨드에 Math.max(5, current - 10) 하한 clamp가 없음',
  );
});
