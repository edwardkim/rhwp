import test from 'node:test';
import assert from 'node:assert/strict';
import { scriptDrawMetrics } from '../src/view/canvaskit/script-metrics.ts';

test('첨자의 실제 producer 값을 다시 배율 계산하지 않고 소비한다', () => {
  const metrics = { fontSize: 9.25, baselineShift: -6.5 };
  assert.deepEqual(scriptDrawMetrics({ superscript: true, scriptMetrics: metrics }, 30), metrics);
  const sub = { fontSize: 7.5, baselineShift: 2.75 };
  assert.deepEqual(scriptDrawMetrics({ subscript: true, scriptMetrics: sub }, 12), sub);
});

test('이전 payload의 첨자·본문 계약을 보존한다', () => {
  assert.deepEqual(scriptDrawMetrics({ superscript: true }, 20), { fontSize: 14, baselineShift: -6 });
  assert.deepEqual(scriptDrawMetrics({ subscript: true }, 20), { fontSize: 14, baselineShift: 3 });
  assert.deepEqual(scriptDrawMetrics({}, 20), { fontSize: 20, baselineShift: 0 });
});

test('잘못된 producer 값을 기존 계약으로 숨기지 않는다', () => {
  for (const metrics of [
    { fontSize: NaN, baselineShift: 0 },
    { fontSize: -1, baselineShift: 0 },
    { fontSize: 12, baselineShift: Infinity },
  ]) {
    assert.throws(() => scriptDrawMetrics({ superscript: true, scriptMetrics: metrics }, 20));
  }
  assert.throws(() => scriptDrawMetrics({ scriptMetrics: { fontSize: 12, baselineShift: 0 } }, 20));
});
