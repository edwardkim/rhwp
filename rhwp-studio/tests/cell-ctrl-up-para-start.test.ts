// [#2914] 표 셀 안 Ctrl+↑ 가 현재 문단 시작을 건너뛰는 문제 — 셀 분기에도
// 본문 분기와 동일한 "charOffset > 0 이면 현재 문단 시작 정지"(한컴 표준) 가드가 있어야 한다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function cellBranchBlock(): string {
  const cursor = readFileSync(join(rootDir, 'src/engine/cursor.ts'), 'utf8');
  const start = cursor.indexOf('moveToParagraphBoundary(direction: -1 | 1): void {');
  assert.notEqual(start, -1, 'moveToParagraphBoundary not found');
  const cellStart = cursor.indexOf('if (this.isInCell() && !this.isInTextBox()) {', start);
  assert.notEqual(cellStart, -1, 'cell branch not found');
  const cellEnd = cursor.indexOf('// ─── 단어 단위 이동', cellStart);
  assert.notEqual(cellEnd, -1, 'cell branch end not found');
  return cursor.slice(cellStart, cellEnd);
}

test('Ctrl+Up inside a table cell stops at current paragraph start when mid-paragraph', () => {
  const block = cellBranchBlock();
  // 본문 분기와 동일한 가드: 문단 중간이면 charOffset 만 0 으로 (셀 좌표 축은 스프레드로 보존)
  assert.match(
    block,
    /direction === -1 && pos\.charOffset > 0[\s\S]*?this\.position = \{ \.\.\.pos, charOffset: 0 \};[\s\S]*?this\.updateRect\(\);[\s\S]*?return;/,
    'cell branch must stop at current paragraph start before moving to the previous cell paragraph',
  );
  // 가드는 cpi/paraCount 계산(WASM 호출)보다 앞서야 한다 — cpi==0 무반응 케이스도 함께 해소.
  assert.ok(
    block.indexOf('direction === -1 && pos.charOffset > 0') < block.indexOf('getCellParagraphCount'),
    'guard must precede cell paragraph count queries',
  );
});
