import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  BASELINE_PATH,
  SIGNALS,
  compareCoverage,
  coveragePercent,
  tallySweep,
} from '../layout-coverage-sweep.mjs';

function doc(overrides = {}) {
  return JSON.stringify({
    offCanvasCount: 0,
    overflowCount: 0,
    overlapCount: 0,
    textOverlapCount: 0,
    emptyPageCount: 0,
    hasSignal: false,
    ...overrides,
  });
}

test('신호 없는 문서를 CLEAN 으로 센다', () => {
  const t = tallySweep([doc(), doc()].join('\n'));
  assert.equal(t.documents, 2);
  assert.equal(t.clean, 2);
  assert.equal(coveragePercent(t), 100);
});

test('신호별 문서 수와 노드 수를 따로 센다', () => {
  const t = tallySweep(
    [
      doc({ offCanvasCount: 3, hasSignal: true }),
      doc({ offCanvasCount: 1, overflowCount: 5, hasSignal: true }),
    ].join('\n'),
  );
  assert.equal(t.signals.off_canvas.documents, 2);
  assert.equal(t.signals.off_canvas.nodes, 4);
  assert.equal(t.signals.overflow.documents, 1);
  assert.equal(t.signals.overflow.nodes, 5);
  assert.equal(t.clean, 0);
});

test('파싱 오류 레코드는 문서 수에서 빼고 따로 센다', () => {
  // 오류를 CLEAN 으로 세면 파싱이 깨질수록 커버리지가 올라간다.
  const t = tallySweep([doc(), '{"error":"broken","source":"x.hwp"}'].join('\n'));
  assert.equal(t.documents, 1);
  assert.equal(t.errors, 1);
  assert.equal(t.clean, 1);
  assert.equal(coveragePercent(t), 100);
});

test('JSON 이 아닌 줄과 깨진 JSON 은 건너뛴다', () => {
  const t = tallySweep(['진행 로그', '', '{not json', doc()].join('\n'));
  assert.equal(t.documents, 1);
});

test('문서가 0 이면 커버리지는 0 이다 — NaN 을 만들지 않는다', () => {
  assert.equal(coveragePercent(tallySweep('')), 0);
});

test('CLEAN 이 줄면 회귀다', () => {
  const base = tallySweep([doc(), doc()].join('\n'));
  const now = tallySweep([doc(), doc({ overflowCount: 1, hasSignal: true })].join('\n'));
  const { regressions } = compareCoverage(now, base);
  assert.ok(regressions.some((r) => r.what === 'CLEAN 문서'));
});

test('CLEAN 이 늘면 개선으로 잡고 실패시키지 않는다', () => {
  const base = tallySweep([doc({ overflowCount: 1, hasSignal: true })].join('\n'));
  const now = tallySweep([doc()].join('\n'));
  const { regressions, improvements } = compareCoverage(now, base);
  assert.deepEqual(regressions, []);
  assert.ok(improvements.some((i) => i.what === 'CLEAN 문서'));
});

test('CLEAN 이 같아도 특정 신호 문서가 늘면 회귀다', () => {
  // 한 문서가 나아지고 다른 문서가 나빠져 총합이 같은 경우를 놓치지 않는다.
  const base = tallySweep([doc({ overflowCount: 2, hasSignal: true }), doc()].join('\n'));
  const now = tallySweep(
    [doc({ overflowCount: 1, hasSignal: true }), doc({ offCanvasCount: 1, hasSignal: true })].join('\n'),
  );
  const { regressions } = compareCoverage(now, base);
  assert.ok(regressions.some((r) => r.what === 'off_canvas 문서'));
});

test('파싱 오류가 늘면 회귀다 — 모수에서 빠져 커버리지가 착시로 오른다', () => {
  const base = tallySweep([doc({ overflowCount: 1, hasSignal: true })].join('\n'));
  const now = tallySweep(['{"error":"broken"}'].join('\n'));
  const { regressions } = compareCoverage(now, base);
  assert.ok(regressions.some((r) => r.what === '파싱 오류'));
});

test('기준선 파일이 신호 다섯 종을 모두 담는다', () => {
  const baseline = JSON.parse(readFileSync(BASELINE_PATH, 'utf8'));
  assert.equal(Object.keys(baseline.signals).length, SIGNALS.length);
  for (const [name] of SIGNALS) {
    assert.ok(baseline.signals[name], `${name} 이 기준선에 없다`);
  }
  assert.ok(baseline.documents > 0);
  assert.equal(baseline.clean + 0, baseline.clean, 'clean 은 수여야 한다');
});
