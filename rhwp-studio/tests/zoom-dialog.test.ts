import test from 'node:test';
import assert from 'node:assert/strict';

import {
  clampCustomZoomPercent,
  detectZoomChoice,
  resolveZoomDialogZoom,
  ZOOM_PRESET_PERCENTAGES,
} from '../src/view/zoom-dialog-state.ts';

test('한컴 고정 비율 프리셋을 그대로 제공한다', () => {
  assert.deepEqual(ZOOM_PRESET_PERCENTAGES, [100, 125, 150, 200, 300, 500]);
});

test('현재 배율은 한컴 프리셋·맞춤·사용자 정의 순서로 복원된다', () => {
  assert.deepEqual(
    detectZoomChoice(1, { fitWidth: 0.8, fitPage: 0.6 }),
    { kind: 'preset', percent: 100 },
  );
  assert.deepEqual(
    detectZoomChoice(0.8, { fitWidth: 0.8, fitPage: 0.6 }),
    { kind: 'fitWidth' },
  );
  assert.deepEqual(
    detectZoomChoice(0.6, { fitWidth: 0.8, fitPage: 0.6 }),
    { kind: 'fitPage' },
  );
  assert.deepEqual(
    detectZoomChoice(1.37, { fitWidth: 0.8, fitPage: 0.6 }),
    { kind: 'custom', percent: 137 },
  );
});

test('사용자 정의 배율은 한컴 계약인 10~500%로 제한된다', () => {
  assert.equal(clampCustomZoomPercent(7), 10);
  assert.equal(clampCustomZoomPercent(137.4), 137);
  assert.equal(clampCustomZoomPercent(600), 500);
});

test('고정·맞춤 배율 선택을 실제 문서 배율로 계산한다', () => {
  const metrics = {
    viewportWidth: 1600,
    viewportHeight: 900,
    pageWidth: 800,
    pageHeight: 1000,
    pageGap: 10,
  };
  assert.equal(resolveZoomDialogZoom({
    zoomChoice: { kind: 'custom', percent: 240 },
    arrangement: { kind: 'single' },
    ...metrics,
  }), 2.4);
  assert.equal(resolveZoomDialogZoom({
    zoomChoice: { kind: 'fitPage' },
    arrangement: { kind: 'single' },
    ...metrics,
  }), 0.88);
});

test('여러 쪽은 별도 비율 선택보다 지정한 가로×세로 맞춤을 우선한다', () => {
  assert.equal(resolveZoomDialogZoom({
    zoomChoice: { kind: 'custom', percent: 240 },
    arrangement: { kind: 'multiple', columns: 2, rows: 2 },
    viewportWidth: 1600,
    viewportHeight: 900,
    pageWidth: 800,
    pageHeight: 1000,
    pageGap: 10,
  }), 0.435);
});
