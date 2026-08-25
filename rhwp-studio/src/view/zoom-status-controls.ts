import {
  formatShortcutLabel,
  type PlatformKind,
} from '../engine/navigation-keymap.ts';

export const ZOOM_SLIDER_MIN_PERCENT = 10;
export const ZOOM_SLIDER_NEUTRAL_PERCENT = 100;
export const ZOOM_SLIDER_MAX_PERCENT = 500;
export const ZOOM_SLIDER_MAX_POSITION = 1000;
export const ZOOM_SLIDER_NEUTRAL_POSITION = ZOOM_SLIDER_MAX_POSITION / 2;

/** 중앙 눈금 양쪽 약 3.6px(150px track 기준)에서 100%에 붙는다. */
const ZOOM_SLIDER_SNAP_POSITION = 24;

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return ZOOM_SLIDER_NEUTRAL_PERCENT;
  return Math.max(
    ZOOM_SLIDER_MIN_PERCENT,
    Math.min(ZOOM_SLIDER_MAX_PERCENT, value),
  );
}

function clampPosition(value: number): number {
  if (!Number.isFinite(value)) return ZOOM_SLIDER_NEUTRAL_POSITION;
  return Math.max(0, Math.min(ZOOM_SLIDER_MAX_POSITION, value));
}

/**
 * 10~100%와 100~500%를 각각 가로바 절반에 로그 비율로 배치한다.
 * 저배율 조작 공간을 보존하면서 한컴처럼 100%가 정확히 중앙에 온다.
 */
export function percentToZoomSliderPosition(percentValue: number): number {
  const percent = clampPercent(percentValue);
  if (percent === ZOOM_SLIDER_NEUTRAL_PERCENT) {
    return ZOOM_SLIDER_NEUTRAL_POSITION;
  }
  if (percent < ZOOM_SLIDER_NEUTRAL_PERCENT) {
    const ratio = Math.log(percent / ZOOM_SLIDER_MIN_PERCENT)
      / Math.log(ZOOM_SLIDER_NEUTRAL_PERCENT / ZOOM_SLIDER_MIN_PERCENT);
    return Math.round(ratio * ZOOM_SLIDER_NEUTRAL_POSITION);
  }
  const ratio = Math.log(percent / ZOOM_SLIDER_NEUTRAL_PERCENT)
    / Math.log(ZOOM_SLIDER_MAX_PERCENT / ZOOM_SLIDER_NEUTRAL_PERCENT);
  return Math.round(
    ZOOM_SLIDER_NEUTRAL_POSITION
    + ratio * (ZOOM_SLIDER_MAX_POSITION - ZOOM_SLIDER_NEUTRAL_POSITION),
  );
}

/** 가로바 위치를 실제 정수 배율로 바꾸며 중앙 근처는 100%에 즉시 스냅한다. */
export function zoomSliderPositionToPercent(
  positionValue: number,
  snap = true,
): number {
  const position = clampPosition(positionValue);
  if (
    snap
    && Math.abs(position - ZOOM_SLIDER_NEUTRAL_POSITION)
      <= ZOOM_SLIDER_SNAP_POSITION
  ) {
    return ZOOM_SLIDER_NEUTRAL_PERCENT;
  }
  if (position <= ZOOM_SLIDER_NEUTRAL_POSITION) {
    const ratio = position / ZOOM_SLIDER_NEUTRAL_POSITION;
    return Math.round(
      ZOOM_SLIDER_MIN_PERCENT
      * ((ZOOM_SLIDER_NEUTRAL_PERCENT / ZOOM_SLIDER_MIN_PERCENT) ** ratio),
    );
  }
  const ratio = (position - ZOOM_SLIDER_NEUTRAL_POSITION)
    / (ZOOM_SLIDER_MAX_POSITION - ZOOM_SLIDER_NEUTRAL_POSITION);
  return Math.round(
    ZOOM_SLIDER_NEUTRAL_PERCENT
    * ((ZOOM_SLIDER_MAX_PERCENT / ZOOM_SLIDER_NEUTRAL_PERCENT) ** ratio),
  );
}

/** 상태바 native tooltip용 플랫폼 단축키 문구. */
export function zoomPercentShortcutTitle(
  action: '확대' | '축소',
  shortcut: 'Ctrl++' | 'Ctrl+-',
  platform: PlatformKind,
): string {
  if (platform === 'mac') {
    const formatted = formatShortcutLabel(shortcut, platform).replace(/-$/, '−');
    return `${action} (${formatted})`;
  }
  const formatted = shortcut === 'Ctrl++' ? 'Ctrl + +' : 'Ctrl + -';
  return `${action} (${formatted})`;
}
