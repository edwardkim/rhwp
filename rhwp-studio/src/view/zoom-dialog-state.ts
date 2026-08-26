import {
  calculateMultiplePagesZoom,
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';
import {
  calculateArrangementFitWidthZoom,
  calculateFitPageZoom,
} from './zoom-fit.ts';
import type { PageMovementSettings } from './page-movement.ts';

export const ZOOM_PRESET_PERCENTAGES = [100, 125, 150, 200, 300, 500] as const;
export const MIN_CUSTOM_ZOOM_PERCENT = 10;
export const MAX_CUSTOM_ZOOM_PERCENT = 500;
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

/** 대화상자 선택을 수치 배율로 바꾼다. 여러 쪽은 지정한 가로×세로 맞춤을 우선한다. */
export function resolveZoomDialogZoom(input: ResolveZoomDialogInput): number {
  const arrangement = normalizePageArrangement(input.arrangement);
  if (arrangement.kind === 'multiple') {
    return calculateMultiplePagesZoom({
      viewportWidth: input.viewportWidth,
      viewportHeight: input.viewportHeight,
      pageWidth: input.pageWidth,
      pageHeight: input.pageHeight,
      columns: arrangement.columns,
      rows: arrangement.rows,
      pageGap: input.pageGap,
    });
  }

  switch (input.zoomChoice.kind) {
    case 'fitWidth':
      return calculateArrangementFitWidthZoom({
        containerWidth: input.viewportWidth,
        pageWidth: input.pageWidth,
        arrangement,
        pageGap: input.pageGap,
      });
    case 'fitPage':
      return calculateFitPageZoom(
        input.viewportWidth,
        input.viewportHeight,
        input.pageWidth,
        input.pageHeight,
      );
    case 'preset':
    case 'custom':
      return clampCustomZoomPercent(input.zoomChoice.percent) / 100;
  }
}
