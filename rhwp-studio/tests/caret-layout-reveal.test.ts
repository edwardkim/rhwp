import test from 'node:test';
import assert from 'node:assert/strict';

import { CaretLayoutReveal } from '../src/engine/caret-layout-reveal.ts';

test('쪽/단 나누기와 본문 Enter는 다음 layout 완료에 한 번만 캐럿 reveal을 예약한다', () => {
  for (const operationType of ['pageBreak', 'columnBreak', 'splitParagraph', 'snapshot:pageBreak', 'snapshot:columnBreak', 'snapshot:splitParagraph']) {
    const reveal = new CaretLayoutReveal();
    reveal.requestFor(operationType);
    assert.equal(reveal.consume(), true, `${operationType}: layout 완료 뒤 reveal`);
    assert.equal(reveal.consume(), false, `${operationType}: 같은 완료 이벤트에서 재사용하지 않음`);
  }
});

test('일반 전체 편집은 지연 reveal을 예약하지 않는다', () => {
  const reveal = new CaretLayoutReveal();
  reveal.requestFor('insertText');
  reveal.requestFor('snapshot:pasteInternal');
  reveal.requestFor('splitParagraphInCell');
  reveal.requestFor('splitParagraphInHeaderFooter');
  reveal.requestFor('splitParagraphInFootnote');
  assert.equal(reveal.consume(), false);
});

test('여러 본문 Enter의 예약은 한 완료 경계에서 합쳐 소비한다', () => {
  const reveal = new CaretLayoutReveal();
  reveal.requestFor('splitParagraph');
  reveal.requestFor('splitParagraph');
  assert.equal(reveal.consume(), true);
  assert.equal(reveal.consume(), false);
});

test('경계 명령 뒤의 일반 명령은 아직 도착하지 않은 layout reveal 예약을 지우지 않는다', () => {
  const reveal = new CaretLayoutReveal();
  reveal.requestFor('pageBreak');
  reveal.requestFor('insertText');
  assert.equal(reveal.consume(), true);
  assert.equal(reveal.consume(), false);
});

test('문서 전환 경계에서는 아직 도착하지 않은 layout reveal 예약을 폐기한다', () => {
  const reveal = new CaretLayoutReveal();
  reveal.requestFor('pageBreak');
  reveal.clear();
  assert.equal(reveal.consume(), false);
});
