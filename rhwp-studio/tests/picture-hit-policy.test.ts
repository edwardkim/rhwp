import test from 'node:test';
import assert from 'node:assert/strict';

import {
  isNestedCellDescendantOfControl,
  orderedControlLayoutPages,
} from '../src/engine/picture-hit-policy.ts';

test('독립 전경 도형은 같은 문단의 표 셀 그림 조상이 아니다', () => {
  // #7333 8쪽: p141/c0 사각 주석 위 클릭은 p141/c2 표 셀의 스크린샷을 선택하면 안 된다.
  const foregroundShape = { secIdx: 0, paraIdx: 141, controlIdx: 0 };
  const screenshotInTable = {
    secIdx: 0,
    paraIdx: 141,
    cellPath: [{ controlIndex: 2, cellIndex: 0, cellParaIndex: 0 }],
  };

  assert.equal(isNestedCellDescendantOfControl(foregroundShape, screenshotInTable), false);
});

test('글상자 control은 그 안의 cellPath 그림의 조상으로 유지한다', () => {
  const textBox = { secIdx: 0, paraIdx: 141, controlIdx: 3 };
  const pictureInTextBox = {
    secIdx: 0,
    paraIdx: 141,
    cellPath: [{ controlIndex: 3, cellIndex: 0, cellParaIndex: 0 }],
  };

  assert.equal(isNestedCellDescendantOfControl(textBox, pictureInTextBox), true);
});

test('마우스로 고른 개체는 해당 쪽을 먼저 다시 찾는다', () => {
  assert.deepEqual(orderedControlLayoutPages(4, 2), [2, 0, 1, 3]);
  assert.deepEqual(orderedControlLayoutPages(4), [0, 1, 2, 3]);
});
