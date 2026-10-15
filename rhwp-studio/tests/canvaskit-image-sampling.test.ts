import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';

let vite: ViteDevServer;
let rendererPrototype: Record<string, any>;
before(async () => {
  vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  rendererPrototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
});
after(async () => { await vite?.close(); });

function sampling(op: Record<string, unknown>, {
  sourceSize = 100, destinationSize = 20, profile = 'highQuality', renderMode = 'default',
  rasterDepth = 0, bitmapFiltering = undefined as 'nearest' | 'linear' | undefined,
} = {}) {
  const calls: unknown[][] = [];
  const renderer = Object.create(rendererPrototype);
  Object.assign(renderer, {
    canvasKit: { XYWHRect: (x: number, y: number, width: number, height: number) => [x, y, x + width, y + height] },
    croppedEffectImages: new WeakMap(), currentProfile: profile, renderMode, preferRasterDepth: rasterDepth,
    drawImageRect: (...args: unknown[]) => calls.push(args),
  });
  // drawImageOp consumes imageForOp's successfully prepared image. This checks
  // sampling dispatch, not a document's pixel layout or Hancom appearance.
  renderer.drawImageOp({}, { width: () => sourceSize, height: () => sourceSize }, {
    type: 'image', bbox: { x: 0, y: 0, width: destinationSize, height: destinationSize }, ...op,
  }, bitmapFiltering);
  assert.equal(calls.length, 1);
  return { filtering: calls[0][5], mipmaps: calls[0][6] };
}

test('grayscale and tone retain linear interpolation and high quality downsample mipmaps', () => {
  for (const op of [{ effect: 'grayScale' }, { brightness: 50 }, { contrast: -25 },
    { effect: 'grayScale', brightness: 50, contrast: 50 }]) {
    assert.deepEqual(sampling(op), { filtering: 'linear', mipmaps: true });
    assert.deepEqual(sampling(op, { sourceSize: 2, destinationSize: 8 }),
      { filtering: 'linear', mipmaps: false });
  }
});

test('only prepared binary effects force nearest without mipmaps', () => {
  for (const effect of ['blackWhite', 'pattern8x8']) {
    assert.deepEqual(sampling({ effect, brightness: 25 }), { filtering: 'nearest', mipmaps: false });
    assert.deepEqual(sampling({ effect, bakedWatermark: true }), { filtering: 'linear', mipmaps: true });
  }
});

test('tone retains profile and raster-group sampling constraints', () => {
  for (const options of [{ profile: 'fastPreview' }, { profile: 'interactive' }, { rasterDepth: 1 }]) {
    assert.deepEqual(sampling({ brightness: 50 }, options), { filtering: 'linear', mipmaps: false });
  }
  assert.deepEqual(sampling({ effect: 'grayScale' }, { profile: 'print' }), { filtering: 'linear', mipmaps: true });
  assert.deepEqual(sampling({ contrast: 50 }, { profile: 'interactive', renderMode: 'compat' }),
    { filtering: 'linear', mipmaps: true });
});

test('bitmap glyph filtering remains explicit and does not enable mipmaps', () => {
  for (const bitmapFiltering of ['nearest', 'linear'] as const) {
    assert.deepEqual(sampling({ effect: 'blackWhite' }, { bitmapFiltering }),
      { filtering: bitmapFiltering, mipmaps: false });
  }
});
