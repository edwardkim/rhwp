import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  BASELINE_PATH,
  DEFECT_CERTAIN,
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
  const now = tallySweep([doc(), doc({ overlapCount: 1, hasSignal: true })].join('\n'));
  const { regressions } = compareCoverage(now, base);
  assert.ok(regressions.some((r) => r.what === 'CLEAN 문서'));
});

test('overflow 만 있는 문서는 CLEAN 이다 — 선언대로 그린 표가 대부분이다', () => {
  // 근거: 한/글 2022 가 파일에 넣어 둔 미리보기(PrvImage)를 재니 한/글도 표를 본문 여백
  // 밖으로 내보낸다 — 표-텍스트 721.4 / 셀보호2 688.5 대 본문 여백 680.3.
  const t = tallySweep([doc({ overflowCount: 3, hasSignal: true })].join('\n'));
  assert.equal(t.clean, 1);
  assert.equal(t.signals.overflow.documents, 1, 'CLEAN 이어도 신호는 계속 센다');
});

test('empty_page 도 CLEAN 판정에서 빠진다 — 도구가 가능성 신호로 정의한다', () => {
  assert.equal(tallySweep([doc({ emptyPageCount: 2 })].join('\n')).clean, 1);
});

test('결함 확실 신호 목록이 SIGNALS 의 부분집합이다', () => {
  const names = SIGNALS.map(([n]) => n);
  for (const n of DEFECT_CERTAIN) assert.ok(names.includes(n), `${n} 이 SIGNALS 에 없다`);
  assert.deepEqual(DEFECT_CERTAIN, ['off_canvas', 'overlap', 'text_overlap']);
});

test('CLEAN 이 늘면 개선으로 잡고 실패시키지 않는다', () => {
  const base = tallySweep([doc({ overlapCount: 1, hasSignal: true })].join('\n'));
  const now = tallySweep([doc()].join('\n'));
  const { regressions, improvements } = compareCoverage(now, base);
  assert.deepEqual(regressions, []);
  assert.ok(improvements.some((i) => i.what === 'CLEAN 문서'));
});

test('CLEAN 이 같아도 특정 신호 문서가 늘면 회귀다', () => {
  // 한 문서가 나아지고 다른 문서가 나빠져 총합이 같은 경우를 놓치지 않는다.
  const base = tallySweep([doc({ overlapCount: 2, hasSignal: true }), doc()].join('\n'));
  const now = tallySweep(
    [doc({ overlapCount: 1, hasSignal: true }), doc({ offCanvasCount: 1, hasSignal: true })].join('\n'),
  );
  const { regressions } = compareCoverage(now, base);
  assert.ok(regressions.some((r) => r.what === 'off_canvas 문서'));
});

test('파싱 오류가 늘면 회귀다 — 모수에서 빠져 커버리지가 착시로 오른다', () => {
  const base = tallySweep([doc({ overlapCount: 1, hasSignal: true })].join('\n'));
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
