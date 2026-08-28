import {
  MAX_DOCUMENT_ZOOM,
  MIN_DOCUMENT_ZOOM,
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';
import {
  resolveZoomFitZoom,
  type ZoomFitMode,
} from './zoom-fit.ts';
import type { PageMovementSettings } from './page-movement.ts';

export const ZOOM_PRESET_PERCENTAGES = [100, 125, 150, 200, 300, 500] as const;
export const MIN_CUSTOM_ZOOM_PERCENT = MIN_DOCUMENT_ZOOM * 100;
export const MAX_CUSTOM_ZOOM_PERCENT = MAX_DOCUMENT_ZOOM * 100;
const ZOOM_CHOICE_TOLERANCE = 0.005;

export type ZoomChoice =
  | { kind: 'preset'; percent: number }
  | { kind: 'fitWidth' }
  | { kind: 'fitPage' }
  | { kind: 'custom'; percent: number };

export interface ZoomDialogValue {
  zoomChoice: ZoomChoice;
  arrangement: PageArrangement;
  pageMovement: PageMovementSettings;
}

export interface ResolveZoomDialogInput
  extends Pick<ZoomDialogValue, 'zoomChoice' | 'arrangement'> {
  viewportWidth: number;
  viewportHeight: number;
  pageWidth: number;
  pageHeight: number;
  pageGap?: number;
}

export type CustomZoomValidationResult =
  | { valid: true; percent: number }
  | { valid: false; message: string };

/** 사용자 정의 배율 입력은 조용히 보정하지 않고 제출 가능한 정수 백분율만 받는다. */
export function validateCustomZoomPercent(rawValue: string): CustomZoomValidationResult {
  const value = rawValue.trim();
  if (value === '') {
    return { valid: false, message: '사용자 정의 배율을 입력하세요.' };
  }

  const percent = Number(value);
  if (!Number.isFinite(percent)) {
    return { valid: false, message: '사용자 정의 배율은 숫자로 입력하세요.' };
  }
  if (!Number.isInteger(percent)) {
    return { valid: false, message: '사용자 정의 배율은 정수로 입력하세요.' };
  }
  if (percent < MIN_CUSTOM_ZOOM_PERCENT || percent > MAX_CUSTOM_ZOOM_PERCENT) {
    return {
      valid: false,
      message: `${MIN_CUSTOM_ZOOM_PERCENT}~${MAX_CUSTOM_ZOOM_PERCENT}% 사이의 배율을 입력하세요.`,
    };
  }
  return { valid: true, percent };
}

export function clampCustomZoomPercent(value: number): number {
  if (!Number.isFinite(value)) return 100;
  return Math.max(
    MIN_CUSTOM_ZOOM_PERCENT,
    Math.min(MAX_CUSTOM_ZOOM_PERCENT, Math.round(value)),
  );
}

/** 현재 수치 배율을 다시 열리는 대화상자의 한컴형 비율 선택으로 복원한다. */
export function detectZoomChoice(
  currentZoom: number,
  fitZooms: { fitWidth: number; fitPage: number },
): ZoomChoice {
  for (const percent of ZOOM_PRESET_PERCENTAGES) {
    if (Math.abs(currentZoom - percent / 100) <= ZOOM_CHOICE_TOLERANCE) {
      return { kind: 'preset', percent };
    }
  }
  if (Math.abs(currentZoom - fitZooms.fitWidth) <= ZOOM_CHOICE_TOLERANCE) {
    return { kind: 'fitWidth' };
  }
  if (Math.abs(currentZoom - fitZooms.fitPage) <= ZOOM_CHOICE_TOLERANCE) {
    return { kind: 'fitPage' };
  }
  return { kind: 'custom', percent: clampCustomZoomPercent(currentZoom * 100) };
}

/** 대화상자 선택을 저장할 맞춤 배율로 바꾼다. 수치 선택은 맞춤이 아니다. */
export function zoomFitModeFromChoice(choice: ZoomChoice): ZoomFitMode {
  return choice.kind === 'fitWidth' || choice.kind === 'fitPage' ? choice.kind : 'none';
}

/** 여러 쪽은 비율 선택과 무관하게 지정 배열 전체의 쪽 맞춤 규칙을 저장한다. */
export function resolveZoomDialogFitMode(
  input: Pick<ZoomDialogValue, 'zoomChoice' | 'arrangement'>,
): ZoomFitMode {
  return normalizePageArrangement(input.arrangement).kind === 'multiple'
    ? 'fitPage'
    : zoomFitModeFromChoice(input.zoomChoice);
}

/** 대화상자 선택을 수치 배율로 바꾼다. 여러 쪽은 지정한 가로×세로 맞춤을 우선한다. */
export function resolveZoomDialogZoom(input: ResolveZoomDialogInput): number {
  const arrangement = normalizePageArrangement(input.arrangement);
  if (arrangement.kind === 'multiple') {
    return resolveZoomFitZoom('fitPage', {
      containerWidth: input.viewportWidth,
      containerHeight: input.viewportHeight,
      pageWidth: input.pageWidth,
      pageHeight: input.pageHeight,
      arrangement,
      pageGap: input.pageGap,
    }) ?? 1;
  }

  switch (input.zoomChoice.kind) {
    case 'fitWidth':
    case 'fitPage':
      return resolveZoomFitZoom(input.zoomChoice.kind, {
        containerWidth: input.viewportWidth,
        containerHeight: input.viewportHeight,
        pageWidth: input.pageWidth,
        pageHeight: input.pageHeight,
        arrangement,
        pageGap: input.pageGap,
      }) ?? 1;
    case 'preset':
    case 'custom':
      return clampCustomZoomPercent(input.zoomChoice.percent) / 100;
  }
}
