import test from 'node:test';
import assert from 'node:assert/strict';

import {
  collectFlowImagePaintOps,
  visibleFlowImageBbox,
} from '../src/view/flow-image-clip.ts';

const image = (bbox: { x: number; y: number; width: number; height: number }) => ({
  type: 'image',
  bbox,
  mime: 'image/png',
  base64: 'AA==',
});

test('flow image collector keeps the nested table-cell clip', () => {
  const tree = {
    kind: 'clipRect',
    clip: { x: 0, y: 0, width: 120, height: 80 },
    child: {
      kind: 'clipRect',
      clip: { x: 20, y: 10, width: 40, height: 30 },
      child: {
        kind: 'leaf',
        ops: [
          image({ x: 25, y: 15, width: 20, height: 10 }),
          image({ x: 70, y: 15, width: 20, height: 10 }),
        ],
      },
    },
  };

  const images = collectFlowImagePaintOps(tree, (op) => op.type === 'image');

  assert.equal(images.length, 2);
  assert.deepEqual(images[0].clip, { x: 20, y: 10, width: 40, height: 30 });
  assert.deepEqual(visibleFlowImageBbox(images[0]), { x: 25, y: 15, width: 20, height: 10 });
  assert.equal(visibleFlowImageBbox(images[1]), null);
});

test('flow image collector leaves unclipped images unchanged', () => {
  const images = collectFlowImagePaintOps(
    { kind: 'leaf', ops: [image({ x: 3, y: 4, width: 5, height: 6 })] },
    (op) => op.type === 'image',
  );

  assert.equal(images.length, 1);
  assert.equal(images[0].clip, null);
  assert.deepEqual(visibleFlowImageBbox(images[0]), { x: 3, y: 4, width: 5, height: 6 });
});
