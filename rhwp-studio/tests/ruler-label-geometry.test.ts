import test from 'node:test';
import assert from 'node:assert/strict';

import { isRulerLabelInsidePage } from '../src/view/ruler-label-geometry.ts';

const PX_PER_MM = 96 / 25.4;

test('A4 155%에서 20cm 라벨은 보이고 21cm 끝 라벨은 숨긴다', () => {
  const zoom = 1.55;
  const pageLeft = 248;
  const pageWidth = 210 * PX_PER_MM * zoom;
  const labelWidth = 11;

  assert.equal(
    isRulerLabelInsidePage(pageLeft + 200 * PX_PER_MM * zoom, labelWidth, pageLeft, pageWidth),
    true,
  );
  assert.equal(
    isRulerLabelInsidePage(pageLeft + 210 * PX_PER_MM * zoom, labelWidth, pageLeft, pageWidth),
    false,
  );
});

test('끝 tick 안쪽이라도 라벨 폭이 용지 경계를 넘으면 숨긴다', () => {
  assert.equal(isRulerLabelInsidePage(98, 6, 0, 100), false);
  assert.equal(isRulerLabelInsidePage(97, 6, 0, 100), true);
});

test('잘못된 좌표나 음수 크기는 표시하지 않는다', () => {
  assert.equal(isRulerLabelInsidePage(Number.NaN, 6, 0, 100), false);
  assert.equal(isRulerLabelInsidePage(50, -1, 0, 100), false);
  assert.equal(isRulerLabelInsidePage(50, 6, 0, -1), false);
});
