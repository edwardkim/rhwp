import test from 'node:test';
import assert from 'node:assert/strict';
import { textRequiresComplexShaping } from '../src/view/canvaskit/nominal-text-replay.ts';

test('plain shadowed circle symbols use nominal SFNT glyphs', () => {
  // Unicode의 두 그림자 원형 기호와 실제 Noto cmap/hmtx 프로그램 증거를 따른다.
  for (const text of ['\u{1f53e}', '\u{1f53f}', '\u{1f53e}\u{1f53f}']) {
    assert.equal(textRequiresComplexShaping(text), false);
  }
});

test('variation selectors, joining sequences and combining marks still need shaping', () => {
  for (const text of ['\u{1f53e}\ufe0f', '\u{1f53e}\u200d\u{1f53f}', '\u{1f53e}\u0301', 'سلام']) {
    assert.equal(textRequiresComplexShaping(text), true);
  }
  assert.equal(textRequiresComplexShaping('ABC 가나 ∑'), false);
});
