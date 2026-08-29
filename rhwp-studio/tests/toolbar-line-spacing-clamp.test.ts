import test from 'node:test';
import { codeOnly } from './support/source-guard.ts';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const toolbar = readFileSync(new URL('../src/ui/toolbar.ts', import.meta.url), 'utf8');

test('툴바 줄 간격 직접입력/증가버튼은 format:line-spacing-increase 커맨드와 동일하게 500%로 clamp한다', () => {
  assert.match(codeOnly(toolbar), /const clamped = Math\.min\(500, num\);/);
  assert.match(codeOnly(toolbar), /const next = Math\.min\(500, cur \+ 5\);/);
});
