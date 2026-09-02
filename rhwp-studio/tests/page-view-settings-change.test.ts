import test from 'node:test';
import assert from 'node:assert/strict';

import { resolvePageViewSettingsChange } from '../src/view/page-view-settings-change.ts';

test('기존 배치-only payload는 zoom commit 없이 정규화된다', () => {
  assert.deepEqual(resolvePageViewSettingsChange({
    arrangement: { kind: 'double' },
    pageMovement: { direction: 'vertical', wheelHorizontal: true },
  }), {
    arrangement: { kind: 'double' },
    pageMovement: { direction: 'vertical', wheelHorizontal: true },
    zoom: null,
  });
});

test('배치·이동·배율 transaction은 한 snapshot으로 정규화된다', () => {
  assert.deepEqual(resolvePageViewSettingsChange({
    arrangement: { kind: 'double' },
    pageMovement: { direction: 'horizontal', wheelHorizontal: false },
    zoom: {
      value: 1.37,
      fitMode: 'fitWidth',
      anchor: { x: 0.25, y: 0.75 },
    },
  }), {
    arrangement: { kind: 'single' },
    pageMovement: { direction: 'horizontal', wheelHorizontal: false },
    zoom: {
      value: 1.37,
      fitMode: 'fitWidth',
      anchor: { x: 0.25, y: 0.75 },
    },
  });
});

test('유효하지 않은 zoom commit은 배치 변경을 막지 않고 제외된다', () => {
  assert.deepEqual(resolvePageViewSettingsChange({
    arrangement: { kind: 'facing' },
    pageMovement: { direction: 'vertical', wheelHorizontal: true },
    zoom: { value: Number.NaN, fitMode: 'unknown', anchor: { x: -1, y: 2 } },
  }), {
    arrangement: { kind: 'facing' },
    pageMovement: { direction: 'vertical', wheelHorizontal: true },
    zoom: null,
  });
});
