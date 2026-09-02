import type { EventBus } from '@/core/event-bus';
import type { CursorState } from './cursor';

export type HeaderFooterEditingMode = 'header' | 'footer';

export interface HeaderFooterModeState {
  mode: HeaderFooterEditingMode;
  sectionIdx: number;
  applyTo: number;
  previewPage: number;
}

export type HeaderFooterModeChangedPayload = 'none' | HeaderFooterModeState;

export function headerFooterApplyToLabel(applyTo: number): string {
  if (applyTo === 1) return '짝수 쪽';
  if (applyTo === 2) return '홀수 쪽';
  return '양쪽';
}

export function headerFooterModeState(cursor: CursorState): HeaderFooterModeChangedPayload {
  if (!cursor.isInHeaderFooter()) return 'none';
  return {
    mode: cursor.headerFooterMode === 'header' ? 'header' : 'footer',
    sectionIdx: cursor.hfSectionIdx,
    applyTo: cursor.hfApplyTo,
    previewPage: cursor.hfPreviewPage,
  };
}

export function emitHeaderFooterModeChanged(eventBus: EventBus, cursor: CursorState): void {
  eventBus.emit('headerFooterModeChanged', headerFooterModeState(cursor));
}

export function parseHeaderFooterModeChanged(
  payload: unknown,
): HeaderFooterModeChangedPayload {
  if (payload === 'none') return 'none';
  if (payload === 'header' || payload === 'footer') {
    return {
      mode: payload,
      sectionIdx: 0,
      applyTo: 0,
      previewPage: 0,
    };
  }
  if (!payload || typeof payload !== 'object') return 'none';
  const value = payload as Partial<HeaderFooterModeState>;
  if (
    (value.mode !== 'header' && value.mode !== 'footer')
    || !Number.isSafeInteger(value.sectionIdx)
    || !Number.isSafeInteger(value.applyTo)
    || !Number.isSafeInteger(value.previewPage)
  ) {
    return 'none';
  }
  return value as HeaderFooterModeState;
}
