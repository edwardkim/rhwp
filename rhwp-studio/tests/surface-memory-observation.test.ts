import test from 'node:test';
import assert from 'node:assert/strict';
import { SurfaceMemoryObservation } from '../src/dev/surface-memory-observation.ts';

const sample = (pixels: number) => ({ at: 0, boundary: 'raster', zoom: 1,
  activePixels: pixels, idlePixels: 0, cachedPixels: 20,
  active: [{ page: 0, visible: true, focused: true, pixels }] });

test('memory observation keeps peak and released last independently without aliasing', () => {
  const o = new SurfaceMemoryObservation();
  const s = sample(100);
  o.observe(s); s.active[0].pixels = 999;
  o.observe(sample(10));
  assert.equal(o.snapshot().peak?.totalPixels, 120);
  assert.equal(o.snapshot().peak?.active[0].pixels, 100);
  assert.equal(o.snapshot().last?.totalPixels, 30);
  const saved = o.snapshot(); saved.peak!.active[0].pixels = 500;
  assert.equal(o.snapshot().peak?.active[0].pixels, 100);
  o.clear(); assert.equal(o.snapshot().count, 0); assert.equal(o.snapshot().peak, null);
});

test('invalid memory sample cannot replace peak or corrupt counts', () => {
  const o = new SurfaceMemoryObservation();
  for (const n of [-1, Infinity, NaN]) o.observe(sample(n));
  assert.equal(o.snapshot().count, 0);
  o.observe(sample(0)); assert.equal(o.snapshot().peak?.totalPixels, 20);
});
