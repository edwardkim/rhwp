import test from 'node:test';
import assert from 'node:assert/strict';
import { CanvasMetricRecovery, isCanvasMetricDescriptorMismatch } from '../src/core/canvas-metric-recovery.ts';

const mismatch = 'Canvas metric descriptor changed; prepare font metrics again';
const tick = () => Promise.resolve();

function fixture() {
  const recovery = new CanvasMetricRecovery();
  const events: string[] = [];
  let current = true;
  const attempt = {
    key: 'document:1/fonts:1',
    isCurrent: () => current,
    invalidate: () => { events.push('invalidate'); },
    prepare: async (guard: () => boolean) => { assert.equal(guard(), true); events.push('prepare'); },
    repaint: () => { events.push('repaint'); },
    report: (error: unknown) => { events.push(`error:${String(error)}`); },
  };
  return { recovery, events, attempt, cancel: () => { current = false; } };
}

test('only the exact painter error (raw or WASM-prefixed) is recoverable', () => {
  for (const message of [mismatch, `렌더링 오류: ${mismatch}`]) {
    assert.equal(isCanvasMetricDescriptorMismatch(message), true);
    assert.equal(isCanvasMetricDescriptorMismatch(new Error(message)), true);
  }
  for (const error of [null, {}, 'resource failed', `${mismatch} extra`, `unrelated ${mismatch}`]) {
    assert.equal(isCanvasMetricDescriptorMismatch(error), false);
  }
});

test('paint cleanup precedes one invalidation/preparation/repaint, even for concurrent page failures', async () => {
  const f = fixture();
  const first = f.recovery.recover(mismatch, f.attempt);
  assert.deepEqual(f.events, []);
  const duplicate = f.recovery.recover(mismatch, f.attempt);
  f.events.push('paint-cleanup');
  assert.equal(await first, true);
  assert.equal(await duplicate, false);
  assert.deepEqual(f.events, ['paint-cleanup', 'invalidate', 'prepare', 'repaint']);
  assert.equal(await f.recovery.recover(mismatch, f.attempt), false);
});

test('persistent mismatch during repaint cannot schedule a second preparation', async () => {
  const f = fixture();
  let retry: Promise<boolean> | undefined;
  f.attempt.repaint = () => {
    f.events.push('repaint-failed');
    retry = f.recovery.recover(mismatch, f.attempt);
  };
  await f.recovery.recover(mismatch, f.attempt);
  assert.equal(await retry, false);
  assert.deepEqual(f.events, ['invalidate', 'prepare', 'repaint-failed']);
});

test('unrelated render errors do not invalidate metrics or consume the retry budget', async () => {
  const f = fixture();
  assert.equal(await f.recovery.recover(new Error('resource failed'), f.attempt), false);
  assert.deepEqual(f.events, []);
  assert.equal(await f.recovery.recover(mismatch, f.attempt), true);
});

test('document/view/backend invalidation before the microtask touches no resources', async () => {
  const f = fixture();
  const pending = f.recovery.recover(mismatch, f.attempt);
  f.cancel();
  assert.equal(await pending, false);
  assert.deepEqual(f.events, []);
});

test('document/view/backend invalidation while fonts load cannot repaint the new document', async () => {
  const f = fixture();
  let release!: () => void;
  const wait = new Promise<void>(resolve => { release = resolve; });
  let guard!: () => boolean;
  f.attempt.prepare = async g => { guard = g; f.events.push('waiting'); await wait; };
  const pending = f.recovery.recover(mismatch, f.attempt);
  await tick();
  f.cancel();
  assert.equal(guard(), false);
  release();
  assert.equal(await pending, false);
  assert.deepEqual(f.events, ['invalidate', 'waiting']);
});

test('preparation failure is diagnosed once and cannot loop', async () => {
  const f = fixture();
  f.attempt.prepare = async () => { throw new Error('font wait failed'); };
  assert.equal(await f.recovery.recover(mismatch, f.attempt), false);
  assert.equal(await f.recovery.recover(mismatch, f.attempt), false);
  assert.deepEqual(f.events, ['invalidate', 'error:Error: font wait failed']);
});

test('late rejection from an obsolete attempt is not attributed to the new view', async () => {
  const f = fixture();
  let reject!: (error: Error) => void;
  f.attempt.prepare = () => new Promise((_resolve, no) => { reject = no; });
  const pending = f.recovery.recover(mismatch, f.attempt);
  await tick();
  f.cancel();
  reject(new Error('old font failure'));
  assert.equal(await pending, false);
  assert.deepEqual(f.events, ['invalidate']);
});

test('external font generation or document change permits a fresh bounded attempt', async () => {
  const f = fixture();
  for (const key of ['document:1/fonts:1', 'document:1/fonts:2', 'document:2/fonts:2']) {
    assert.equal(await f.recovery.recover(mismatch, { ...f.attempt, key }), true);
    assert.equal(await f.recovery.recover(mismatch, { ...f.attempt, key }), false);
  }
  assert.equal(f.events.filter(e => e === 'prepare').length, 3);
});
