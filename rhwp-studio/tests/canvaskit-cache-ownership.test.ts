import test from 'node:test';
import assert from 'node:assert/strict';
import { BoundedNativeCache } from '../src/view/canvaskit/bounded-native-cache.ts';
import { CanvasKitSurfaceCache } from '../src/view/canvaskit/surface-cache.ts';
import { replayFingerprint } from '../src/view/canvaskit/replay-fingerprint.ts';

test('native LRU bounds count and byte ownership, touches hits, and rejects oversized entries without taking ownership', () => {
  const deleted: number[] = [];
  const cache = new BoundedNativeCache<number>(2, 10, value => deleted.push(value));
  assert.equal(cache.set('a', 1, 4), true); assert.equal(cache.set('b', 2, 4), true);
  assert.equal(cache.get('a'), 1); cache.set('c', 3, 4);
  assert.deepEqual(deleted, [2]); assert.equal(cache.bytes, 8); assert.equal(cache.evictions, 1);
  assert.equal(cache.set('oversized', 4, 11), false); assert.deepEqual(deleted, [2]);
  cache.clear(); assert.deepEqual(deleted, [2, 1, 3]); assert.equal(cache.bytes, 0);
});

test('surface cache reuses unchanged software targets but releases resized targets and never reuses WebGPU swapchain textures', () => {
  const cache = new CanvasKitSurfaceCache();
  const target: any = { width: 10, height: 10 };
  let deleted = 0;
  const create = () => ({ surface: { delete: () => { deleted++; } }, canvas: target, backend: 'software', fallbackReason: null }) as any;
  cache.get(target, create); cache.get(target, create);
  assert.equal(cache.created, 1); assert.equal(cache.reused, 1);
  target.width++;
  cache.get(target, create); assert.equal(deleted, 1);
  cache.clear(); assert.equal(deleted, 2);
  const webgpu = () => ({ ...create(), backend: 'webgpu' });
  cache.get(target, webgpu); cache.get(target, webgpu);
  assert.equal(cache.created, 4); assert.equal(deleted, 3);
  cache.clear();
});

test('strong replay fingerprint detects mutable byte/geometry changes and refuses cyclic or excessive work', () => {
  const bytes = Uint8Array.of(1, 2, 3);
  const input = { bytes, bounds: { x: 1, y: 2 }, clipEnabled: true };
  const original = replayFingerprint(input);
  assert.equal(original, replayFingerprint({ clipEnabled: true, bounds: { y: 2, x: 1 }, bytes }));
  bytes[1] = 9; assert.notEqual(original, replayFingerprint(input));
  const next = replayFingerprint(input); input.bounds.x++; assert.notEqual(next, replayFingerprint(input));
  const cyclic: any = {}; cyclic.self = cyclic; assert.equal(replayFingerprint(cyclic), null);
  assert.equal(replayFingerprint(new Array(100_001).fill(0)), null);
});
