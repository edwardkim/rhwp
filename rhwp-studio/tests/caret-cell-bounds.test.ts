import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const caretRenderer = readFileSync(new URL('../src/engine/caret-renderer.ts', import.meta.url), 'utf8');
const inputHandler = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');

test('표 셀 IME 조합창은 Canvas clip과 별도로 cellBounds 안에 제한한다', () => {
  assert.match(caretRenderer, /private clampCompositionBox\(/);
  assert.match(caretRenderer, /const bounds = rect\.cellBounds;/);
  assert.match(caretRenderer, /w = Math\.min\(w, Math\.max\(0, bounds\.w\)\);/);
  assert.match(caretRenderer, /x = Math\.min\(Math\.max\(x, bounds\.x\), maxX\);/);
  assert.match(caretRenderer, /y = Math\.min\(Math\.max\(y, bounds\.y\), maxY\);/);
});

test('지연 셀 입력이 가시 높이를 넘으면 즉시 전체 페이지네이션을 수행한다', () => {
  assert.match(inputHandler, /if \(this\.flushDeferredPaginationForCellOverflow\(\)\) return;/);
  assert.match(inputHandler, /private flushDeferredPaginationForCellOverflow\(\): boolean/);
  assert.match(inputHandler, /if \(!this\.cursor\.getRect\(\)\?\.cellOverflowed\) return false;/);
  assert.match(inputHandler, /this\.wasm\.flushDeferredPagination\(\);/);
  assert.match(inputHandler, /this\.cursor\.moveTo\(this\.cursor\.getPosition\(\)\);/);
});
