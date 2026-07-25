import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  calculateAnchoredScroll,
  type ZoomPageBox,
} from '../src/view/zoom-anchor.ts';

test('center anchor stays fixed while content crosses horizontal overflow', () => {
  const oldBox: ZoomPageBox = {
    left: 214.25,
    top: 10,
    width: 454.5,
    height: 643,
  };
  const newBox: ZoomPageBox = {
    left: 20,
    top: 10,
    width: 930.5,
    height: 1316.5,
  };
  const next = calculateAnchoredScroll(
    oldBox,
    newBox,
    {
      width: 883,
      height: 683,
      scrollLeft: 0,
      scrollTop: 0,
    },
    { x: 0.5, y: 0.5 },
  );

  assert.ok(Math.abs(next.scrollLeft - 43.75) < 0.01);
  assert.ok(Math.abs(next.scrollTop - 347.22433903576984) < 0.01);
});

test('off-center pointer anchor is reversible', () => {
  const fit: ZoomPageBox = {
    left: 214.25,
    top: 10,
    width: 454.5,
    height: 643,
  };
  const enlarged: ZoomPageBox = {
    left: 20,
    top: 10,
    width: 930.5,
    height: 1316.5,
  };
  const viewport = {
    width: 883,
    height: 683,
    scrollLeft: 0,
    scrollTop: 0,
  };
  const anchor = { x: 0.25, y: 0.75 };
  const forward = calculateAnchoredScroll(fit, enlarged, viewport, anchor);
  const reverse = calculateAnchoredScroll(
    enlarged,
    fit,
    { ...viewport, ...forward },
    anchor,
  );

  assert.ok(Math.abs(reverse.scrollLeft) < 1e-9);
  assert.ok(Math.abs(reverse.scrollTop) < 1e-9);
});

test('anchored scroll can preserve a point across viewport resize', () => {
  const oldBox = { left: 900, top: 10, width: 500, height: 700 };
  const newBox = { left: 700, top: 10, width: 500, height: 700 };
  const next = calculateAnchoredScroll(
    oldBox,
    newBox,
    {
      width: 900,
      height: 650,
      scrollLeft: 700,
      scrollTop: 200,
    },
    { x: 0.5, y: 0.5 },
    { width: 700, height: 550 },
  );

  assert.equal(next.scrollLeft, 600);
  assert.equal(next.scrollTop, 250);
});

test('CanvasView consumes the zoom anchor and corrects both scroll axes', () => {
  const source = readFileSync(
    new URL('../src/view/canvas-view.ts', import.meta.url),
    'utf8',
  );

  assert.match(source, /eventBus\.on\('zoom-changed', \(zoom, anchor\)/);
  assert.match(source, /calculateAnchoredScroll\(/);
  assert.match(source, /setScrollLeft\(nextScroll\.scrollLeft\)/);
  assert.match(source, /setScrollTop\(nextScroll\.scrollTop\)/);
});

test('CanvasView and ruler consume the stable horizontal coordinate', () => {
  const canvasSource = readFileSync(
    new URL('../src/view/canvas-view.ts', import.meta.url),
    'utf8',
  );
  const rulerSource = readFileSync(
    new URL('../src/view/ruler.ts', import.meta.url),
    'utf8',
  );

  assert.match(canvasSource, /getCenteredScrollLeft\(/);
  assert.match(
    rulerSource,
    /getPageLeftResolved\(\s*0,\s*this\.virtualScroll\.getTotalWidth\(\),?\s*\)/,
  );
  assert.doesNotMatch(rulerSource, /contentOffsetX/);
});
