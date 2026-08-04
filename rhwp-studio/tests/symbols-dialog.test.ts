import test from 'node:test';
import assert from 'node:assert/strict';

import { isCodePointInBlock } from '../src/ui/unicode-block.ts';

// [문자표] 최근 사용 문자는 현재 표시된 블록과 다른 블록에 속할 수 있다.
// selectChar()가 codePoint - block.start 로 그리드 인덱스를 계산하므로,
// 문자가 현재 블록에 속하지 않으면 엉뚱한 셀을 하이라이트하게 되는 버그가 있었다.
test('현재 블록에 속하지 않는 코드포인트는 false', () => {
  const block = { name: '기본 라틴 문자', start: 0x0020, end: 0x007F };
  assert.equal(isCodePointInBlock(0x0041, block), true); // 'A' — 블록 내부
  assert.equal(isCodePointInBlock(0xAC00, block), false); // '가' — 다른 블록(한글 음절)
});
