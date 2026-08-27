import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  adjacentIconToolbarDividerTarget,
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

test('navigation resolves the next and previous visible divider boundaries', () => {
  const boundaries = [0, 182, 314, 402, 578, 710, 842];
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 0, 1, 700), 182);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 200, 1, 700), 314);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 500, -1, 700), 402);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 700, 1, 700), 700);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 0, -1, 700), 0);
  assert.equal(adjacentIconToolbarDividerTarget(boundaries, 710, -1, 700), 578);
});

test('controller owns resize, mode, focus, keyboard, and end-state synchronization', () => {
  assert.match(source, /new ResizeObserver/);
  assert.match(source, /new MutationObserver/);
  assert.match(source, /target\.parentElement === this\.track/);
  assert.match(source, /attributeFilter: \['style', 'hidden'\]/);
  assert.match(source, /availableWithoutNavigation/);
  assert.match(source, /this\.previousButton\.hidden = !overflowing/);
  assert.match(source, /querySelectorAll<HTMLElement>\(DIVIDER_SELECTOR\)/);
  assert.match(source, /rect\.right - trackLeft - targetBoundary/);
  assert.match(source, /rect\.left - trackLeft - targetBoundary/);
  assert.match(source, /classList\.toggle\('tb-scroll-nav-edge-hidden', atStart\)/);
  assert.match(source, /setAttribute\('aria-hidden', atEnd \? 'true' : 'false'\)/);
  assert.match(source, /event\.key === 'ArrowLeft'/);
  assert.match(source, /event\.key === 'End'/);
  assert.match(source, /commandRect\.right > visibleRight/);
  assert.match(source, /SCROLL_ANIMATION_DURATION_MS = 240/);
  assert.match(source, /classList\.add\(SCROLL_EXIT_CLASS\)/);
  assert.match(source, /requestAnimationFrame\(animate\)/);
  assert.match(source, /matchMedia\('\(prefers-reduced-motion: reduce\)'\)/);
  assert.match(source, /dispose\(\): void/);
});

test('edge navigation overlays the track and synchronizes its exit with scrolling', () => {
  assert.match(toolbarCss, /\.tb-scroll-nav\s*\{[^}]*position:\s*absolute;/s);
  assert.match(toolbarCss, /\.tb-scroll-nav\s*\{[^}]*background:\s*linear-gradient/s);
  assert.match(toolbarCss, /\.tb-scroll-nav\.tb-scroll-nav-edge-hidden\s*\{[^}]*visibility:\s*hidden;/s);
  assert.match(toolbarCss, /\.tb-scroll-nav\.tb-scroll-nav-edge-hidden\s*\{[^}]*opacity:\s*0;/s);
  assert.match(toolbarCss, /\.tb-scroll-nav\.tb-scroll-nav-edge-hidden\s*\{[^}]*pointer-events:\s*none;/s);
  assert.match(
    toolbarCss,
    /\.tb-scroll-nav\.tb-scroll-nav-transitioning-out\s*\{[^}]*--tb-scroll-exit-duration/s,
  );
  assert.doesNotMatch(toolbarCss, /\.tb-scroll-nav\.tb-scroll-nav-edge-hidden\s*\{[^}]*display:\s*none;/s);
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
