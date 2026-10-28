import test from 'node:test';
import assert from 'node:assert/strict';
import { CanvasSupplementalMetricProvider } from '../src/core/supplemental-text-metrics.ts';

const generation = { document: 1, fonts: 1 };
const requests = [{ key: 'style0:U1F600', cluster: '😀', font: '13.333px HCR' }];
const ready = async () => {};

function fixture(width = 18.3) {
  const provider = new CanvasSupplementalMetricProvider();
  provider.reset(generation);
  let calls = 0;
  let assigned = '';
  const context = {
    get font() { return assigned; },
    set font(value: string) {
      if (value === 'invalid') return;
      assigned = value.replace('HCR', 'resolved-HCR,fallback');
    },
    measureText() { calls += 1; return { width } as TextMetrics; },
  };
  return { provider, create: () => context, calls: () => calls };
}

test('waits for fonts and measures using the normal substituted descriptor', async () => {
  const f = fixture();
  let release!: () => void;
  const waiting = new Promise<void>(resolve => { release = resolve; });
  const pending = f.provider.prepare(generation, requests, () => waiting, f.create);
  assert.equal(f.calls(), 0);
  release();
  const result = await pending;
  assert.equal(result.status, 'ready');
  if (result.status !== 'ready') throw new Error('missing result');
  assert.equal(result.metrics[0].measuredAdvancePx, 18.3);
  assert.equal(result.metrics[0].resolvedFont, '13.333px resolved-HCR,fallback');
  assert.equal(result.metrics[0].evidence, 'backendMeasured');
  assert.equal(result.metrics[0].backend, 'canvas2d');
});

test('warm batches do not create a context or measure again', async () => {
  const f = fixture();
  const first = await f.provider.prepare(generation, requests, ready, f.create);
  if (first.status !== 'ready') throw new Error('missing result');
  first.metrics[0].measuredAdvancePx = 99; // returned data cannot corrupt cache
  const result = await f.provider.prepare(generation, requests, ready, () => {
    throw new Error('warm cache unexpectedly created Canvas');
  });
  assert.equal(result.status, 'ready');
  if (result.status !== 'ready') throw new Error('missing result');
  assert.equal(result.metrics[0].measuredAdvancePx, 18.3);
  assert.equal(result.measured, 0);
  assert.equal(result.cacheHits, 1);
  assert.equal(f.calls(), 1);
});

test('document/font changes discard responses waiting for fonts', async () => {
  for (const next of [{ document: 2, fonts: 1 }, { document: 1, fonts: 2 }]) {
    const f = fixture();
    let release!: () => void;
    const waiting = new Promise<void>(resolve => { release = resolve; });
    const pending = f.provider.prepare(generation, requests, () => waiting, f.create);
    f.provider.reset(next);
    release();
    assert.equal((await pending).status, 'stale');
    assert.equal(f.calls(), 0);
    assert.equal((await f.provider.prepare(generation, requests, ready, f.create)).status, 'stale');
    assert.equal((await f.provider.prepare(next, requests, ready, f.create)).status, 'ready');
  }
});

test('disposal and generation counter reuse cannot revive pending data', async () => {
  const f = fixture();
  let release!: () => void;
  const waiting = new Promise<void>(resolve => { release = resolve; });
  const pending = f.provider.prepare(generation, requests, () => waiting, f.create);
  f.provider.dispose();
  f.provider.reset(generation);
  release();
  assert.equal((await pending).status, 'stale');
});

test('a rejected batch does not cache its preceding successful measurements', async () => {
  const f = fixture();
  await assert.rejects(f.provider.prepare(generation, [requests[0], {
    key: 'invalid', cluster: '𝄞', font: 'invalid',
  }], ready, f.create), /rejected.*descriptor/);
  const result = await f.provider.prepare(generation, requests, ready, f.create);
  if (result.status !== 'ready') throw new Error('missing result');
  assert.equal(result.measured, 1);
  assert.equal(f.calls(), 2);
});

test('invalid advances and unsupported clusters are not successes', async () => {
  for (const width of [NaN, Infinity, -1]) {
    const f = fixture(width);
    await assert.rejects(f.provider.prepare(generation, requests, ready, f.create), /advance/);
  }
  for (const cluster of ['', 'ab', '😀\uFE0F', '😀\u200D😀', '\ud800', '\n', '\u0085']) {
    const f = fixture();
    await assert.rejects(f.provider.prepare(generation, [{ ...requests[0], cluster }], ready, f.create), /cluster/);
    assert.equal(f.calls(), 0);
  }
  const zero = fixture(0);
  const result = await zero.provider.prepare(generation, requests, ready, zero.create);
  if (result.status !== 'ready') throw new Error('missing result');
  assert.equal(result.metrics[0].measuredAdvancePx, 0);
});

test('font replacement clears the cache even for identical CSS inputs', async () => {
  const f = fixture();
  await f.provider.prepare(generation, requests, ready, f.create);
  const next = { ...generation, fonts: 2 };
  f.provider.reset(next);
  const result = await f.provider.prepare(next, requests, ready, f.create);
  if (result.status !== 'ready') throw new Error('missing result');
  assert.equal(result.measured, 1);
  assert.equal(f.calls(), 2);
});

test('fonts-ready rejection and unavailable Canvas do not poison a valid cache', async () => {
  const f = fixture();
  await f.provider.prepare(generation, requests, ready, f.create);
  const fresh = [{ ...requests[0], key: 'new', cluster: '𝄞' }];
  await assert.rejects(f.provider.prepare(generation, fresh, async () => {
    throw new Error('fonts unavailable');
  }, f.create), /fonts unavailable/);
  await assert.rejects(f.provider.prepare(generation, fresh, ready, () => null), /context unavailable/);
  const warm = await f.provider.prepare(generation, requests, ready, f.create);
  if (warm.status !== 'ready') throw new Error('missing result');
  assert.equal(warm.cacheHits, 1);
});

test('entry, duplicate and UTF-8 budgets are checked before Canvas work', async () => {
  const f = fixture();
  await assert.rejects(f.provider.prepare(generation, Array(4097).fill(requests[0]), ready, f.create), /limit/);
  await assert.rejects(f.provider.prepare(generation, [requests[0], requests[0]], ready, f.create), /Duplicate/);
  await assert.rejects(f.provider.prepare(generation, [{ ...requests[0], font: '가'.repeat(1_500_000) }], ready, f.create), /limit/);
  assert.equal(f.calls(), 0);
});

test('captures requests before awaiting and keys cache by the complete descriptor', async () => {
  const f = fixture();
  const mutable = [{ ...requests[0] }];
  let release!: () => void;
  const waiting = new Promise<void>(resolve => { release = resolve; });
  const pending = f.provider.prepare(generation, mutable, () => waiting, f.create);
  mutable[0].font = '99px HCR';
  release();
  const first = await pending;
  if (first.status !== 'ready') throw new Error('missing result');
  assert.equal(first.metrics[0].font, requests[0].font);
  const changed = await f.provider.prepare(generation, mutable, ready, f.create);
  if (changed.status !== 'ready') throw new Error('missing result');
  assert.equal(changed.measured, 1);
  assert.equal(f.calls(), 2);
});
