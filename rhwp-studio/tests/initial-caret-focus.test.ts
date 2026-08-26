import test from 'node:test';
import assert from 'node:assert/strict';

import { showInitialCaretAndPublishFocus } from '../src/engine/initial-caret-focus.ts';

function runInitialFocus(pageIndex: number) {
  const events: Array<{ event: string; payload: unknown }> = [];
  const shown: Array<{ rect: unknown; zoom: number }> = [];
  const rect = { pageIndex, x: 42, y: 84, height: 18 };

  const published = showInitialCaretAndPublishFocus(
    rect,
    0.5,
    { show: (value, zoom) => shown.push({ rect: value, zoom }) },
    { emit: (event, payload) => events.push({ event, payload }) },
  );

  return { published, rect, shown, events };
}

test('문서 최초 캐럿은 저장된 물리 쪽을 편집 focus 이벤트로 발행한다', () => {
  const result = runInitialFocus(3);

  assert.equal(result.published, true);
  assert.deepEqual(result.shown, [{ rect: result.rect, zoom: 0.5 }]);
  assert.deepEqual(result.events, [{
    event: 'cursor-rect-updated',
    payload: { pageIndex: 3, x: 42, y: 84 },
  }]);
});

test('0번 쪽도 viewport fallback과 구별되는 명시적 편집 focus로 발행한다', () => {
  const result = runInitialFocus(0);

  assert.deepEqual(result.events, [{
    event: 'cursor-rect-updated',
    payload: { pageIndex: 0, x: 42, y: 84 },
  }]);
});

test('캐럿 좌표가 없으면 표시하거나 focus를 발행하지 않는다', () => {
  let showCount = 0;
  let emitCount = 0;

  const published = showInitialCaretAndPublishFocus(
    null,
    1,
    { show: () => { showCount += 1; } },
    { emit: () => { emitCount += 1; } },
  );

  assert.equal(published, false);
  assert.equal(showCount, 0);
  assert.equal(emitCount, 0);
});
