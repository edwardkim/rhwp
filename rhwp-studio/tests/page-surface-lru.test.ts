import test from 'node:test';
import assert from 'node:assert/strict';

import { PageSurfaceLru } from '../src/view/page-surface-lru.ts';

type Entry = { key: string; lookupKey: string; pageIndex: number; pixelCount: number };

function entry(key: string, pageIndex: number, pixelCount: number, lookupKey = key): Entry {
  return { key, lookupKey, pageIndex, pixelCount };
}

test('정확한 key만 hit이며 take한 항목은 원장에서 active 소유권으로 빠진다', () => {
  const disposed: string[] = [];
  const lru = new PageSurfaceLru<Entry>(value => disposed.push(value.key), 100);
  lru.reconcile(100, 40);
  assert.equal(lru.put(entry('doc:a|page:0|scale:1', 0, 30)), true);

  assert.equal(lru.take('doc:a|page:0|scale:2'), null);
  assert.equal(lru.take('doc:a|page:0|scale:1')?.pageIndex, 0);
  assert.deepEqual(disposed, []);
  assert.deepEqual(lru.snapshot(), {
    pixelBudget: 100,
    reservedPixels: 40,
    cachedPixels: 0,
    totalAccountedPixels: 40,
    overBudgetMandatory: false,
    entryCount: 0,
    hits: 1,
    misses: 1,
    evictions: 0,
    rejected: 0,
    invalidations: 0,
  });
});

test('lookup key는 실제 surface 정수 치수가 든 exact entry를 O(1)로 찾는다', () => {
  const lru = new PageSurfaceLru<Entry>(() => undefined, 100);
  lru.put(entry('doc:a|scale:2|surfaces:main:2246x1588', 0, 40, 'doc:a|scale:2'));
  assert.equal(lru.hasLookup('doc:a|scale:2'), true);
  assert.equal(
    lru.takeLookup('doc:a|scale:2')?.key,
    'doc:a|scale:2|surfaces:main:2246x1588',
  );
  assert.equal(lru.hasLookup('doc:a|scale:2'), false);
});

test('touch 후 재삽입은 MRU가 되고 eviction은 가장 오래된 bundle부터 수행한다', () => {
  const disposed: string[] = [];
  const lru = new PageSurfaceLru<Entry>(value => disposed.push(value.key), 60);
  lru.put(entry('a', 0, 20));
  lru.put(entry('b', 1, 20));
  const a = lru.take('a');
  assert.ok(a);
  lru.put(a);
  lru.put(entry('c', 2, 30));

  assert.deepEqual(disposed, ['b']);
  assert.equal(lru.has('a'), true);
  assert.equal(lru.has('c'), true);
  assert.equal(lru.snapshot().cachedPixels, 50);
});

test('active와 pending 예약을 먼저 적용하고 mandatory 초과에서는 cache를 0으로 만든다', () => {
  const disposed: string[] = [];
  const lru = new PageSurfaceLru<Entry>(value => disposed.push(value.key), 100);
  lru.put(entry('a', 0, 35));
  lru.put(entry('b', 1, 35));
  lru.reconcile(100, 70);
  assert.deepEqual(disposed, ['a', 'b']);
  assert.equal(lru.snapshot().cachedPixels, 0);

  lru.put(entry('c', 2, 10));
  assert.deepEqual(disposed, ['a', 'b']);
  lru.reconcile(100, 120);
  assert.deepEqual(disposed, ['a', 'b', 'c']);
  assert.equal(lru.snapshot().overBudgetMandatory, true);
  assert.equal(lru.snapshot().totalAccountedPixels, 120);
});

test('page invalidation과 clear는 dispose를 정확히 한 번 호출하고 누적 원장을 0으로 만든다', () => {
  const disposed: string[] = [];
  const lru = new PageSurfaceLru<Entry>(value => disposed.push(value.key), 100);
  lru.put(entry('p0-low', 0, 10));
  lru.put(entry('p0-high', 0, 20));
  lru.put(entry('p1', 1, 30));

  lru.deletePage(0);
  assert.deepEqual(disposed, ['p0-low', 'p0-high']);
  assert.equal(lru.snapshot().cachedPixels, 30);
  lru.clear();
  assert.deepEqual(disposed, ['p0-low', 'p0-high', 'p1']);
  assert.equal(lru.snapshot().cachedPixels, 0);
  assert.equal(lru.snapshot().invalidations, 3);
});

test('0 pixel 항목은 보존하지 않고 즉시 폐기한다', () => {
  const disposed: string[] = [];
  const lru = new PageSurfaceLru<Entry>(value => disposed.push(value.key), 100);
  assert.equal(lru.put(entry('empty', 0, 0)), false);
  assert.deepEqual(disposed, ['empty']);
  assert.equal(lru.snapshot().rejected, 1);
});
