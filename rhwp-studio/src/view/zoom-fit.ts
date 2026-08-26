import {
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';

const MIN_REQUESTED_ZOOM = 0.1;
const MAX_REQUESTED_ZOOM = 4;
const HORIZONTAL_FRAME_PADDING = 40;
const VERTICAL_FRAME_PADDING = 20;
const DEFAULT_PAGE_GAP = 10;

function clampRequestedZoom(zoom: number): number {
  return Math.max(MIN_REQUESTED_ZOOM, Math.min(MAX_REQUESTED_ZOOM, zoom));
}

export function calculateFitWidthZoom(
  containerWidth: number,
  pageWidth: number,
): number {
  if (pageWidth <= 0) return 1;
  return clampRequestedZoom(
    (containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth,
  );
}

export interface ArrangementFitWidthInput {
  containerWidth: number;
  pageWidth: number;
  arrangement: PageArrangement;
  pageGap?: number;
}

/** 현재 쪽 배치의 한 행 전체가 문서 창 너비 안에 들어오도록 배율을 계산한다. */
export function calculateArrangementFitWidthZoom(
  input: ArrangementFitWidthInput,
): number {
  if (input.pageWidth <= 0) return 1;
  const arrangement = normalizePageArrangement(input.arrangement);
  const columns = arrangement.kind === 'multiple'
    ? arrangement.columns
    : arrangement.kind === 'double' || arrangement.kind === 'facing'
      ? 2
      : 1;
  const pageGap = Number.isFinite(input.pageGap)
    ? Math.max(0, input.pageGap ?? 0)
    : DEFAULT_PAGE_GAP;
  const availableWidth = input.containerWidth
    - HORIZONTAL_FRAME_PADDING
    - pageGap * (columns - 1);
  return clampRequestedZoom(availableWidth / (input.pageWidth * columns));
}

export function calculateFitPageZoom(
  containerWidth: number,
  containerHeight: number,
  pageWidth: number,
  pageHeight: number,
): number {
  if (pageWidth <= 0 || pageHeight <= 0) return 1;
  return clampRequestedZoom(Math.min(
    (containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth,
    (containerHeight - VERTICAL_FRAME_PADDING) / pageHeight,
  ));
}

/** 사용자가 마지막으로 고른 맞춤 배율. 'none' 은 수치 배율(사용자 지정)이다. */
export type ZoomFitMode = 'none' | 'fitWidth' | 'fitPage';

export function normalizeZoomFitMode(value: unknown): ZoomFitMode {
  return value === 'fitWidth' || value === 'fitPage' ? value : 'none';
}

export interface ZoomFitMetrics {
  containerWidth: number;
  containerHeight: number;
  pageWidth: number;
  pageHeight: number;
  arrangement: PageArrangement;
  pageGap?: number;
}

/**
 * 저장된 맞춤 배율을 지금의 창·쪽 크기로 다시 계산한다.
 *
 * 맞춤은 수치가 아니라 규칙이므로 문서마다(쪽 크기가 다르므로) 다시 계산해야 한다.
 * 수치 배율('none')이면 되돌릴 배율이 없다는 뜻으로 null 을 준다.
 */
export function resolveZoomFitZoom(
  mode: ZoomFitMode,
  metrics: ZoomFitMetrics,
): number | null {
  switch (mode) {
    case 'fitWidth':
      return calculateArrangementFitWidthZoom({
        containerWidth: metrics.containerWidth,
        pageWidth: metrics.pageWidth,
        arrangement: metrics.arrangement,
        pageGap: metrics.pageGap,
      });
    case 'fitPage':
      return calculateFitPageZoom(
        metrics.containerWidth,
        metrics.containerHeight,
        metrics.pageWidth,
        metrics.pageHeight,
      );
    case 'none':
      return null;
  }
}
