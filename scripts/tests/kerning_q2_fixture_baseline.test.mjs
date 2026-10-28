import assert from 'node:assert/strict';
import test from 'node:test';

import {
  assertCurrentOnOffIdentity,
  projectBodyRuns,
} from '../kerning_q2_fixture_baseline.mjs';

function run(ratio, spacing, kerning, lane, positions) {
  return {
    type: 'textRun',
    text: `BODY R${ratio} S${spacing} K${kerning ? 1 : 0} L${lane} | AV To WA HH 가나다`,
    positions,
    style: { fontFamily: 'Noto Sans KR', fontSize: 13.333 },
  };
}

test('body projection preserves nine paired on/off axes', () => {
  const children = [];
  for (const ratio of [100, 90, 80]) {
    for (const spacing of [0, -5, -10]) {
      const lane = spacing === -5 ? 'fresh' : 'stored';
      const text = `BODY R${ratio} S${spacing} K0 L${lane} | AV To WA HH 가나다`;
      const positions = Array.from({ length: Array.from(text).length + 1 }, (_, index) => index);
      children.push(run(ratio, spacing, false, lane, positions));
      children.push(run(ratio, spacing, true, lane, positions));
    }
  }
  const rows = projectBodyRuns({ children });
  assert.equal(rows.length, 18);
  assert.equal(assertCurrentOnOffIdentity(rows).length, 9);
});

test('current baseline rejects an unplanned on/off difference', () => {
  const rows = [
    { ratio: 100, spacing: 0, lane: 'stored-line-lane', kerning: false, samplePositions: [0, 1] },
    { ratio: 100, spacing: 0, lane: 'stored-line-lane', kerning: true, samplePositions: [0, 0.9] },
  ];
  assert.throws(() => assertCurrentOnOffIdentity(rows), /expected 9|unexpectedly changes/);
});
