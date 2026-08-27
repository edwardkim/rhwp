import assert from 'node:assert/strict';
import test from 'node:test';
import {
  adjacentIconToolbarDividerTarget,
  clampIconToolbarScroll,
  hasIconToolbarOverflow,
  iconToolbarNavigationState,
  initIconToolbarScroller,
} from '../src/ui/icon-toolbar-scroller.ts';

test('overflow uses measured content with a one-pixel rounding tolerance', () => {
  assert.equal(hasIconToolbarOverflow(1219, 1280), false);
  assert.equal(hasIconToolbarOverflow(1219, 1218), false);
  assert.equal(hasIconToolbarOverflow(1219, 1217), true);
});

test('navigation resolves the next and previous visible divider boundaries', () => {
  const boundaries = [0, 182, 314, 402, 578, 710, 842];
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 0, 1, 700), 182);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 200, 1, 700), 314);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 500, -1, 700), 402);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 700, 1, 700), 700);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 0, -1, 700), 0);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 710, -1, 700), 578);
});

test('mode refresh preserves and clamps the current toolbar position', () => {
  assert.equal(clampIconToolbarScroll(240, 600, true), 240);
  assert.equal(clampIconToolbarScroll(720, 600, true), 600);
  assert.equal(clampIconToolbarScroll(-10, 600, true), 0);
  assert.equal(clampIconToolbarScroll(240, 0, false), 0);
});

test('navigation state resolves both overflow edges without disabling a focused button', () => {
  assert.deepEqual(iconToolbarNavigationState(0, 600, false), {
    atStart: true,
    atEnd: false,
  });
  assert.deepEqual(iconToolbarNavigationState(300, 600, false), {
    atStart: false,
    atEnd: false,
  });
  assert.deepEqual(iconToolbarNavigationState(600, 600, false), {
    atStart: false,
    atEnd: true,
  });
  assert.deepEqual(iconToolbarNavigationState(0, 0, true), {
    atStart: true,
    atEnd: true,
  });
});

test('optional chrome initialization is a no-op when the root is absent', () => {
  assert.equal(initIconToolbarScroller(null), null);
});
