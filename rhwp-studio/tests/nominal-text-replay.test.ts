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
  for (const text of ['\u{1f53e}\ufe0f', '\u{1f53e}\u200d\u{1f53f}', '\u{1f53e}\u0301',
    '\u{f02fc}\ufe0f', '\u{f02fc}\u200d\u{f031c}', '\u{f02fc}\u0301', 'سلام']) {
    assert.equal(textRequiresComplexShaping(text), true);
  }
  assert.equal(textRequiresComplexShaping('ABC 가나 ∑'), false);
});

test('단독 그림문자와 뒤의 한글은 그리되 결합 이모지는 명목 재생으로 보내지 않는다', () => {
  for (const text of ['🦦나나끝', '앞😀뒤', '🦊', '🛠']) {
    assert.equal(textRequiresComplexShaping(text), false, text);
  }
  for (const text of ['🦦\ufe0f', '👩\u200d🔬', '👍🏽', '🇰🇷', '1\ufe0f\u20e3']) {
    assert.equal(textRequiresComplexShaping(text), true, text);
  }
});

test('supplementary private-use scalars use nominal glyphs, excluding noncharacters', () => {
  for (const text of ['\u{f0000}', '\u{f02fc}', '\u{ffffd}', '\u{100000}', '\u{10fffd}']) {
    assert.equal(textRequiresComplexShaping(text), false);
  }
  for (const text of ['\u{ffffe}', '\u{fffff}', '\u{10fffe}', '\u{10ffff}']) {
    assert.equal(textRequiresComplexShaping(text), true);
  }
});
