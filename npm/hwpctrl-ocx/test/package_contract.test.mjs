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
