import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  adjacentIconToolbarGroupTarget,
  hasIconToolbarOverflow,
} from '../src/ui/icon-toolbar-scroller.ts';

const source = readFileSync(new URL('../src/ui/icon-toolbar-scroller.ts', import.meta.url), 'utf8');
const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
const toolbarCss = readFileSync(new URL('../src/styles/toolbar.css', import.meta.url), 'utf8');

test('overflow uses measured content with a one-pixel rounding tolerance', () => {
  assert.equal(hasIconToolbarOverflow(1219, 1280), false);
  assert.equal(hasIconToolbarOverflow(1219, 1218), false);
  assert.equal(hasIconToolbarOverflow(1219, 1217), true);
});

test('navigation resolves the next and previous visible group boundaries', () => {
  const boundaries = [0, 182, 314, 402, 578, 710, 842];
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 0, 1, 700), 182);
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 200, 1, 700), 314);
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 500, -1, 700), 402);
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 700, 1, 700), 700);
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 0, -1, 700), 0);
  assert.equal(adjacentIconToolbarGroupTarget(boundaries, 710, -1, 700), 578);
});

test('controller owns resize, mode, focus, keyboard, and end-state synchronization', () => {
  assert.match(source, /new ResizeObserver/);
  assert.match(source, /new MutationObserver/);
  assert.match(source, /target\.parentElement === this\.track/);
  assert.match(source, /attributeFilter: \['style', 'hidden'\]/);
  assert.match(source, /availableWithoutNavigation/);
  assert.match(source, /this\.previousButton\.hidden = !overflowing/);
  assert.match(source, /this\.previousButton\.disabled = navigationHidden \|\| current <= SCROLL_EPSILON/);
  assert.match(source, /event\.key === 'ArrowLeft'/);
  assert.match(source, /event\.key === 'End'/);
  assert.match(source, /commandRect\.right > viewportRect\.right/);
  assert.match(source, /dispose\(\): void/);
});

test('main initializes the controller and mode switching still targets the single track', () => {
  assert.match(main, /initIconToolbarScroller\(document\.getElementById\('icon-toolbar'\)!\)/);
  assert.match(main, /#icon-toolbar \.tb-scroll-track > \.tb-group/);
});

test('existing split menu escapes the horizontal clip without changing its command DOM', () => {
  assert.match(toolbarCss, /\.tb-split-menu\s*\{[^}]*position:\s*fixed;/s);
  assert.match(main, /const setToolbarSplitOpen = \(split: Element, open: boolean\)/);
  assert.match(main, /menu\.style\.left = `\$\{left\}px`/);
  assert.match(main, /menu\.style\.top = `\$\{top\}px`/);
  assert.match(main, /arrow\.setAttribute\('aria-haspopup', 'menu'\)/);
  assert.match(source, /\.tb-split-arrow'\)\?\.setAttribute\('aria-expanded', 'false'\)/);
});
