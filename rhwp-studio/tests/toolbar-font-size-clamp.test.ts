import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const toolbar = readFileSync(new URL('../src/ui/toolbar.ts', import.meta.url), 'utf8');

test('툴바 글자 크기 입력은 char-shape-dialog와 동일하게 1~4096pt로 clamp한다', () => {
  assert.match(toolbar, /const clampedPt = Math\.min\(4096, Math\.max\(1, pt\)\);/);
  assert.match(toolbar, /const newPt = Math\.min\(4096, pt \+ 1\);/);
});
