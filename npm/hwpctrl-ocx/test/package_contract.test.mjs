import assert from 'node:assert/strict';
import test from 'node:test';

import { HwpCtrl, ParameterSet, createHwpCtrl } from '@rhwp/hwpctrl';

test('공개 패키지 진입점이 호환 층 생성자를 제공한다', () => {
  assert.equal(typeof createHwpCtrl, 'function');
  assert.equal(typeof HwpCtrl, 'function');
  assert.equal(typeof ParameterSet, 'function');

  const ctrl = createHwpCtrl();
  assert.ok(ctrl instanceof HwpCtrl);
});

test('GetTextFile이 TEXT와 UNICODE 코어 경로를 구분한다', () => {
  const calls = [];
  const ctrl = new HwpCtrl({
    doc: {
      getTextFileText() {
        calls.push('TEXT');
        return '"가&#9702;€"';
      },
      getTextFileUnicode() {
        calls.push('UNICODE');
        return '"가◦€"';
      },
    },
  });

  // TEXT 경로도 코어의 `&#N;` escape 를 웹 계약대로 원문으로 되돌려 준다(기안기 실측).
  assert.equal(ctrl.GetTextFile('TEXT', ''), '가◦€');
  assert.equal(ctrl.GetTextFile('unicode', ''), '가◦€');
  assert.deepEqual(calls, ['TEXT', 'UNICODE']);
});

test('ViewProperties는 정규화 가능한 보기 값만 제공한다', () => {
  const ctrl = new HwpCtrl();
  assert.equal(ctrl.ViewProperties.Count, 12);
  assert.equal(ctrl.ViewProperties.Item('ZoomType'), 5);
  assert.equal(ctrl.ViewProperties.Item('ZoomRatio'), 100);
  assert.equal(ctrl.ViewProperties.Item('OptionFlag'), undefined);
});
