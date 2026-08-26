import type { CursorRect } from '@/core/types';

export interface InitialCaretTarget {
  show(rect: CursorRect, zoom: number): void;
}

export interface InitialCaretFocusEventTarget {
  emit(
    event: 'cursor-rect-updated',
    payload: Pick<CursorRect, 'pageIndex' | 'x' | 'y'>,
  ): void;
}

/**
 * 문서 최초 캐럿을 표시하고 같은 물리 쪽을 편집 focus로 발행한다.
 *
 * 캐럿 DOM만 표시하면 CanvasView의 focus는 null로 남아 눈금자가 viewport fallback을
 * 사용한다. 줌으로 페이지 배치가 바뀌어도 저장된 캐럿 쪽을 유지하도록 두 동작을 한
 * 관문에서 수행한다.
 */
export function showInitialCaretAndPublishFocus(
  rect: CursorRect | null,
  zoom: number,
  caret: InitialCaretTarget,
  eventTarget: InitialCaretFocusEventTarget,
): boolean {
  if (!rect) return false;

  caret.show(rect, zoom);
  eventTarget.emit('cursor-rect-updated', {
    pageIndex: rect.pageIndex,
    x: rect.x,
    y: rect.y,
  });
  return true;
}
