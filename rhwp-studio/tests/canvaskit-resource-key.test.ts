import test from 'node:test';
import assert from 'node:assert/strict';

import { layerResourceKeyMatches } from '../src/view/canvaskit/resource-key.ts';

const bytes = new Uint8Array([1, 2]);
const digest = 'b7d770040f780e9deff6bc038abea66e108b88d098d16d24cd7486eb671060b2';

test('CanvasKit strict resource key verifies kind, length, and BLAKE3 digest', () => {
  assert.equal(layerResourceKeyMatches('img', `img:blake3:2:${digest}`, bytes), true);
  assert.equal(layerResourceKeyMatches('svg', `img:blake3:2:${digest}`, bytes), false);
  assert.equal(layerResourceKeyMatches('img', `img:blake3:3:${digest}`, bytes), false);
  assert.equal(layerResourceKeyMatches('img', `img:blake3:2:${'0'.repeat(64)}`, bytes), false);
  assert.equal(layerResourceKeyMatches('img', `img:blake3:02:${digest}`, bytes), false);
});
