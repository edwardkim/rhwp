import assert from 'node:assert/strict';
import test from 'node:test';

import { encodedImageDimensions } from '../src/view/canvaskit/image-header.ts';

test('encodedImageDimensions reads bounded PNG, GIF, WebP, BMP, and JPEG headers', () => {
  const png = new Uint8Array(24);
  png.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0);
  png.set([0x49, 0x48, 0x44, 0x52], 12);
  new DataView(png.buffer).setUint32(16, 320, false);
  new DataView(png.buffer).setUint32(20, 240, false);

  const gif = new Uint8Array([0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x40, 0x01, 0xf0, 0x00]);
  const webp = new Uint8Array(30);
  webp.set(new TextEncoder().encode('RIFF'), 0);
  webp.set(new TextEncoder().encode('WEBPVP8X'), 8);
  webp.set([0x3f, 0x01, 0x00, 0xef, 0x00, 0x00], 24);
  const bmp = new Uint8Array(26);
  bmp.set([0x42, 0x4d], 0);
  new DataView(bmp.buffer).setInt32(18, 640, true);
  new DataView(bmp.buffer).setInt32(22, -480, true);
  const jpeg = new Uint8Array([
    0xff, 0xd8,
    0xff, 0xe0, 0x00, 0x04, 0x00, 0x00,
    0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0xc8, 0x01, 0x2c, 0x01, 0x01,
    0xff, 0xd9,
  ]);

  assert.deepEqual(encodedImageDimensions(png), { width: 320, height: 240 });
  assert.deepEqual(encodedImageDimensions(gif), { width: 320, height: 240 });
  assert.deepEqual(encodedImageDimensions(webp), { width: 320, height: 240 });
  assert.deepEqual(encodedImageDimensions(bmp), { width: 640, height: 480 });
  assert.deepEqual(encodedImageDimensions(jpeg), { width: 300, height: 200 });
});

test('encodedImageDimensions rejects malformed and zero-sized headers', () => {
  assert.equal(encodedImageDimensions(new Uint8Array([0xff, 0xd8, 0xff, 0xc0])), null);
  const png = new Uint8Array(24);
  png.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0);
  png.set([0x49, 0x48, 0x44, 0x52], 12);
  assert.equal(encodedImageDimensions(png), null);
});
