import assert from 'node:assert/strict';
import test from 'node:test';

import {
  canonicalJson,
  normalizePlatformSentinels,
  projectKerningOffRuns,
  sha256,
} from '../kerning_q1_baseline.mjs';

test('canonical JSON is key-order independent', () => {
  assert.equal(canonicalJson({ b: 2, a: 1 }), canonicalJson({ a: 1, b: 2 }));
  assert.equal(sha256(canonicalJson({ b: 2, a: 1 })).length, 64);
});

test('only target-width synthetic paragraph sentinels are normalized', () => {
  const differences = { count: 0 };
  const value = normalizePlatformSentinels({
    key64: 'section:0/para:18446744073709551615/char:3',
    key32: 'section:0/para:4294967295/char:3',
    ordinary: 'section:0/para:42/char:3',
  }, differences);
  assert.equal(value.key64, 'section:0/para:MAX/char:3');
  assert.equal(value.key32, 'section:0/para:MAX/char:3');
  assert.equal(value.ordinary, 'section:0/para:42/char:3');
  assert.equal(differences.count, 2);
});

test('off projection requires nine K0 runs and rejects an exposed kerning field', () => {
  const children = [];
  for (const ratio of [100, 90, 80]) {
    for (const spacing of [0, -5, -10]) {
      children.push({
        type: 'textRun',
        text: `BODY R${ratio} S${spacing} K0 | `,
        positions: [0, ratio + spacing],
        style: { ratio: ratio / 100 },
        paintStyle: { ratio: ratio / 100 },
      });
    }
  }
  assert.equal(projectKerningOffRuns({ children }).length, 9);
  children[0].style.kerning = false;
  assert.throws(() => projectKerningOffRuns({ children }), /unexpectedly exposes kerning/);
});
