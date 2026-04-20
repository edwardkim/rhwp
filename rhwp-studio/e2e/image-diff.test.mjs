import assert from 'node:assert/strict';

import { PNG } from 'pngjs';

import { comparePngBuffers } from './helpers.mjs';

function makeWhitePng(width, height) {
  const image = new PNG({ width, height });
  for (let i = 0; i < image.data.length; i += 4) {
    image.data[i] = 255;
    image.data[i + 1] = 255;
    image.data[i + 2] = 255;
    image.data[i + 3] = 255;
  }
  return image;
}

function drawVerticalLine(image, x, y1, y2) {
  for (let y = y1; y <= y2; y++) {
    const base = (y * image.width + x) * 4;
    image.data[base] = 0;
    image.data[base + 1] = 0;
    image.data[base + 2] = 0;
    image.data[base + 3] = 255;
  }
}

async function main() {
  const base = makeWhitePng(8, 8);
  drawVerticalLine(base, 3, 1, 6);

  const shiftedByOne = PNG.sync.read(PNG.sync.write(base));
  shiftedByOne.data.fill(255);
  for (let i = 3; i < shiftedByOne.data.length; i += 4) {
    shiftedByOne.data[i] = 255;
  }
  drawVerticalLine(shiftedByOne, 4, 1, 6);

  const shiftedByTwo = PNG.sync.read(PNG.sync.write(base));
  shiftedByTwo.data.fill(255);
  for (let i = 3; i < shiftedByTwo.data.length; i += 4) {
    shiftedByTwo.data[i] = 255;
  }
  drawVerticalLine(shiftedByTwo, 5, 1, 6);

  const shiftedByOneDiff = await comparePngBuffers(PNG.sync.write(base), PNG.sync.write(shiftedByOne), {
    ignoreChannelDelta: 8,
    inkMaskWhiteDelta: 25,
    inkMaskAlphaThreshold: 8,
    inkMaskNeighborhoodRadius: 1,
    inkMaskMaxDiffPixels: 0,
  });
  assert.equal(shiftedByOneDiff.passed, true);
  assert.ok(shiftedByOneDiff.rawTolerantDiffPixels > 0);
  assert.equal(shiftedByOneDiff.rawInkMaskDiffPixels, 0);
  console.log('PASS: 1px neighborhood shift is tolerated by ink-mask diff');

  const shiftedByTwoDiff = await comparePngBuffers(PNG.sync.write(base), PNG.sync.write(shiftedByTwo), {
    ignoreChannelDelta: 8,
    inkMaskWhiteDelta: 25,
    inkMaskAlphaThreshold: 8,
    inkMaskNeighborhoodRadius: 1,
    inkMaskMaxDiffPixels: 0,
  });
  assert.equal(shiftedByTwoDiff.passed, false);
  assert.ok(shiftedByTwoDiff.rawInkMaskDiffPixels > 0);
  console.log('PASS: 2px shift still fails ink-mask diff');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
