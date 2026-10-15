import test from 'node:test';
import assert from 'node:assert/strict';
import { applyLayerImageEffectPixels } from '../src/view/image-effect-pixels.ts';

test('tone uses the native/SVG normalized additive brightness and centered contrast', () => {
  // Independent values from feComponentTransfer: slope = 1 + contrast/100,
  // intercept = (1 - slope)/2 + brightness/100, applied to normalized RGB.
  for (const [brightness, contrast, expected] of [
    [50, 0, 192], [-50, 0, 0], [0, 50, 32], [50, 50, 160],
    [0, -100, 128], [-25, -100, 64], [100, 100, 255],
  ]) {
    const pixels = Uint8Array.of(64, 64, 64, 37);
    assert.equal(applyLayerImageEffectPixels(pixels, 1, 'realPic', 0, 0, brightness, contrast), true);
    assert.deepEqual([...pixels], [expected, expected, expected, 37], `${brightness}/${contrast}`);
  }
});

test('grayscale retains fractional luminance until the final tone quantization', () => {
  const pixels = Uint8Array.of(0, 100, 0, 77);
  // 58.7 * 1.5 - 63.75 = 24.3; rounding the grayscale first would produce 25.
  applyLayerImageEffectPixels(pixels, 1, 'grayScale', 0, 0, 0, 50);
  assert.deepEqual([...pixels], [24, 24, 24, 77]);
  const grayOnly = Uint8ClampedArray.of(0, 100, 0, 77);
  applyLayerImageEffectPixels(grayOnly, 1, 'grayScale');
  assert.deepEqual([...grayOnly], [59, 59, 59, 77]);
});

test('binary effects precede tone and preserve alpha and source-pixel thresholding', () => {
  const pixels = Uint8Array.of(127, 127, 127, 0, 128, 128, 128, 255);
  applyLayerImageEffectPixels(pixels, 2, 'blackWhite', 0, 0, 25, 0);
  assert.deepEqual([...pixels], [64, 64, 64, 0, 255, 255, 255, 255]);
  const pattern = Uint8Array.of(128, 128, 128, 21, 128, 128, 128, 22);
  applyLayerImageEffectPixels(pattern, 2, 'pattern8x8');
  assert.deepEqual([...pattern], [255, 255, 255, 21, 0, 0, 0, 22]);
});

test('tone clamps its declared range and channels only after applying the full adjustment', () => {
  const pixels = Uint8Array.of(0, 64, 255, 129);
  applyLayerImageEffectPixels(pixels, 1, undefined, 0, 0, 200, -200);
  assert.deepEqual([...pixels], [255, 255, 255, 129]);
  const unchanged = Uint8Array.of(0, 64, 255, 129);
  assert.equal(applyLayerImageEffectPixels(unchanged, 1, 'realPic'), false);
  assert.deepEqual([...unchanged], [0, 64, 255, 129]);
});
