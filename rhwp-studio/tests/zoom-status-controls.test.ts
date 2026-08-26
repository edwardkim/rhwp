import test from 'node:test';
import assert from 'node:assert/strict';

import {
  ZOOM_SLIDER_MAX_POSITION,
  ZOOM_SLIDER_NEUTRAL_POSITION,
  percentToZoomSliderPosition,
  zoomPercentShortcutTitle,
  zoomSliderPositionToPercent,
} from '../src/view/zoom-status-controls.ts';

test('배율 가로바는 10%·100%·500%를 시작·중앙·끝에 배치한다', () => {
  assert.equal(percentToZoomSliderPosition(10), 0);
  assert.equal(percentToZoomSliderPosition(100), ZOOM_SLIDER_NEUTRAL_POSITION);
  assert.equal(percentToZoomSliderPosition(500), ZOOM_SLIDER_MAX_POSITION);
  assert.equal(zoomSliderPositionToPercent(0), 10);
  assert.equal(zoomSliderPositionToPercent(ZOOM_SLIDER_NEUTRAL_POSITION), 100);
  assert.equal(zoomSliderPositionToPercent(ZOOM_SLIDER_MAX_POSITION), 500);
});

test('10~500% 배율 변환은 구간별 로그 비율로 왕복한다', () => {
  for (const percent of [10, 25, 50, 75, 100, 125, 200, 300, 400, 500]) {
    const position = percentToZoomSliderPosition(percent);
    assert.ok(
      Math.abs(zoomSliderPositionToPercent(position, false) - percent) <= 1,
      `${percent}%가 가로바 위치를 왕복해야 한다`,
    );
  }
});

test('중앙 근처를 드래그하면 100%에 즉시 스냅하고 범위를 벗어나면 풀린다', () => {
  assert.equal(zoomSliderPositionToPercent(478), 100);
  assert.equal(zoomSliderPositionToPercent(522), 100);
  assert.notEqual(zoomSliderPositionToPercent(470), 100);
  assert.notEqual(zoomSliderPositionToPercent(530), 100);
});

test('확대·축소 호버 단축키는 플랫폼 표기를 사용한다', () => {
  assert.equal(zoomPercentShortcutTitle('확대', 'Ctrl++', 'mac'), '확대 (⌘+)');
  assert.equal(zoomPercentShortcutTitle('축소', 'Ctrl+-', 'mac'), '축소 (⌘−)');
  assert.equal(zoomPercentShortcutTitle('확대', 'Ctrl++', 'other'), '확대 (Ctrl + +)');
  assert.equal(zoomPercentShortcutTitle('축소', 'Ctrl+-', 'other'), '축소 (Ctrl + -)');
});
