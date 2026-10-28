import {
  resolvePageViewSettings,
  type PageMovementSettings,
} from './page-movement.ts';
import type { PageArrangement } from './page-arrangement.ts';
import {
  CENTER_ZOOM_ANCHOR,
  normalizeZoomAnchor,
  type ZoomAnchor,
} from './zoom-anchor.ts';
import { normalizeZoomFitMode, type ZoomFitMode } from './zoom-fit.ts';

export interface PageViewZoomCommit {
  value: number;
  fitMode: ZoomFitMode;
  anchor: ZoomAnchor;
}

/** 배치-only 기존 발행자와 배치+배율 transaction을 함께 받는 보기 이벤트 payload. */
export interface PageViewSettingsChange {
  arrangement?: unknown;
  pageMovement?: unknown;
  zoom?: Partial<PageViewZoomCommit> | null;
}

export interface ResolvedPageViewSettingsChange {
  arrangement: PageArrangement;
  pageMovement: PageMovementSettings;
  zoom: PageViewZoomCommit | null;
}

/** EventBus의 untyped payload를 정규화해 CanvasView commit 경계로 넘긴다. */
export function resolvePageViewSettingsChange(
  payload: unknown,
): ResolvedPageViewSettingsChange {
  const value = payload && typeof payload === 'object'
    ? payload as PageViewSettingsChange
    : {};
  const pageView = resolvePageViewSettings(value.arrangement, value.pageMovement);
  const requestedZoom = value.zoom;
  const zoom = requestedZoom
    && typeof requestedZoom === 'object'
    && typeof requestedZoom.value === 'number'
    && Number.isFinite(requestedZoom.value)
    ? {
        value: requestedZoom.value,
        fitMode: normalizeZoomFitMode(requestedZoom.fitMode),
        anchor: normalizeZoomAnchor(requestedZoom.anchor ?? CENTER_ZOOM_ANCHOR),
      }
    : null;
  return {
    arrangement: pageView.arrangement,
    pageMovement: pageView.movement,
    zoom,
  };
}
