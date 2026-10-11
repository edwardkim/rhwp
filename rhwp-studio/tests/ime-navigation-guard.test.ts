import test from 'node:test';
import assert from 'node:assert/strict';
import { ImeNavigationGuard } from '../src/engine/ime-navigation-guard.ts';

const left = { key: 'ArrowLeft', code: 'ArrowLeft', shiftKey: false, ctrlKey: false, metaKey: false, altKey: false };

test('fcitx forwarded navigation is consumed once after replay', () => {
  const guard = new ImeNavigationGuard();
  guard.queue({ ...left, key: 'Process' });
  assert.equal(guard.consume(left), false, 'the editor replay must run');
  guard.markReplayed();
  assert.equal(guard.consume({ ...left, isComposing: true, keyCode: 229 }), false);
  assert.equal(guard.consume(left), true);
  assert.equal(guard.consume(left), false);
});

test('keyup and focus/document reset preserve the next physical press', () => {
  for (const beforeReplay of [false, true]) {
    const guard = new ImeNavigationGuard();
    guard.queue(left);
    if (beforeReplay) guard.release(left);
    guard.markReplayed();
    if (!beforeReplay) guard.release(left);
    assert.equal(guard.consume(left), false);
  }
  const guard = new ImeNavigationGuard();
  guard.queue(left);
  guard.markReplayed();
  guard.reset();
  assert.equal(guard.consume(left), false);
});

test('repeat, another key and changed modifiers remain independent inputs', () => {
  for (const next of [
    { ...left, repeat: true },
    { ...left, key: 'ArrowRight', code: 'ArrowRight' },
    { ...left, shiftKey: true },
    { ...left, ctrlKey: true },
    { ...left, altKey: true },
    { ...left, metaKey: true },
  ]) {
    const guard = new ImeNavigationGuard();
    guard.queue(left);
    guard.markReplayed();
    assert.equal(guard.consume(next), false);
    assert.equal(guard.consume(left), false);
  }
});
