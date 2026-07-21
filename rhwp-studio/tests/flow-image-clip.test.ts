import test from 'node:test';
import assert from 'node:assert/strict';

import {
  collectFlowImagePaintOps,
  composeImageFilter,
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

// composeImageFilter 효과별 CSS filter 문자열 pin (PR #2523 검토 후속 권고).
// web_canvas.rs::compose_image_filter 와 동일 산출 계약 — 문자열이 바뀌면
// WASM canvas 경로와 DOM flow-image 경로의 그림 효과가 갈라진다.

test('composeImageFilter pins effect-only outputs', () => {
  assert.equal(composeImageFilter({}), null);
  assert.equal(composeImageFilter({ effect: 'realPic' }), null);
  assert.equal(composeImageFilter({ effect: 'grayScale' }), 'grayscale(100%)');
  assert.equal(composeImageFilter({ effect: 'pattern8x8' }), 'grayscale(100%)');
  assert.equal(
    composeImageFilter({ effect: 'blackWhite' }),
    'grayscale(100%) contrast(1000%)',
  );
});

test('composeImageFilter pins brightness/contrast scaling', () => {
  assert.equal(composeImageFilter({ brightness: 20 }), 'brightness(1.2000)');
  assert.equal(composeImageFilter({ brightness: -20 }), 'brightness(0.8000)');
  assert.equal(composeImageFilter({ contrast: -30 }), 'contrast(0.7000)');
  assert.equal(composeImageFilter({ contrast: 50 }), 'contrast(1.5000)');
  assert.equal(
    composeImageFilter({ effect: 'grayScale', brightness: 10, contrast: -10 }),
    'grayscale(100%) brightness(1.1000) contrast(0.9000)',
  );
});

test('composeImageFilter ignores baked watermark and non-finite inputs', () => {
  // baked 픽셀은 효과가 이미 적용돼 있으므로 filter 를 걸지 않는다.
  assert.equal(
    composeImageFilter({ effect: 'blackWhite', brightness: 40, bakedWatermark: true }),
    null,
  );
  // 비유한/비문자 입력은 기본값(효과 없음, 0)으로 정규화.
  assert.equal(composeImageFilter({ brightness: Number.NaN, contrast: Infinity }), null);
  assert.equal(composeImageFilter({ effect: 123 }), null);
});
