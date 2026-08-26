export interface ObjectSelectionBox {
  pageIndex: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface ObjectSelectionSummary {
  /** 기존 다중 선택 오버레이가 사용하던 마지막 bbox의 페이지. */
  renderPageIndex: number;
  /** 선택 순서의 첫 유효 bbox를 편집 focus로 사용한다. */
  editingPageIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface EditingPageEventTarget {
  emit(event: 'editing-page-changed', pageIndex: number | null): void;
}

/** 다중 개체 선택의 기존 렌더 기준과 새 편집 focus 기준을 분리한다. */
export function summarizeObjectSelection(
  boxes: readonly ObjectSelectionBox[],
): ObjectSelectionSummary | null {
  if (boxes.length === 0) return null;

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const box of boxes) {
    minX = Math.min(minX, box.x);
    minY = Math.min(minY, box.y);
    maxX = Math.max(maxX, box.x + box.w);
    maxY = Math.max(maxY, box.y + box.h);
  }

  return {
    renderPageIndex: boxes[boxes.length - 1].pageIndex,
    editingPageIndex: boxes[0].pageIndex,
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

/** 선택 오버레이를 지운 소비처가 stale 편집 페이지도 함께 지우도록 한다. */
export function clearObjectEditingPage(eventTarget: EditingPageEventTarget): void {
  eventTarget.emit('editing-page-changed', null);
}
