import test from 'node:test';
import assert from 'node:assert/strict';
import { VirtualScroll } from '../src/view/virtual-scroll.ts';
import { calculateAnchoredScroll } from '../src/view/zoom-anchor.ts';

const pages = [{ width: 800, height: 1000 }] as never;

test('horizontal pan coordinates center the page on both sides of overflow', () => {
  const viewportWidth = 900;
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 0.5, viewportWidth);
  const underflowLeft = scroll.getPageLeft(0);
  const underflowCenter = scroll.getCenteredScrollLeft(viewportWidth);
  assert.ok(underflowLeft >= 0);
  assert.equal(
    underflowLeft - underflowCenter,
    (viewportWidth - scroll.getPageWidth(0)) / 2,
  );

  scroll.setPageDimensions(pages, 1.25, viewportWidth);
  const overflowLeft = scroll.getPageLeft(0);
  const overflowCenter = scroll.getCenteredScrollLeft(viewportWidth);
  assert.equal(
    overflowLeft - overflowCenter,
    (viewportWidth - scroll.getPageWidth(0)) / 2,
  );
});

test('pointer anchor remains representable across the viewport-width boundary', () => {
  const viewportWidth = 900;
  const anchor = { x: 0.35, y: 0.75 };
  const scroll = new VirtualScroll();

  scroll.setPageDimensions(pages, 0.54, viewportWidth);
  const oldBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const viewport = {
    width: viewportWidth,
    height: 650,
    scrollLeft: scroll.getCenteredScrollLeft(viewportWidth),
    scrollTop: 0,
  };

  scroll.setPageDimensions(pages, 1.2, viewportWidth);
  const newBox = {
    left: scroll.getPageLeft(0),
    top: scroll.getPageOffset(0),
    width: scroll.getPageWidth(0),
    height: scroll.getPageHeight(0),
  };
  const forward = calculateAnchoredScroll(oldBox, newBox, viewport, anchor);
  assert.ok(forward.scrollLeft >= 0);
  assert.ok(forward.scrollLeft <= scroll.getTotalWidth() - viewportWidth);

  const reverse = calculateAnchoredScroll(
    newBox,
    oldBox,
    { ...viewport, ...forward },
    anchor,
  );
  assert.ok(Math.abs(reverse.scrollLeft - viewport.scrollLeft) < 1e-9);
});
