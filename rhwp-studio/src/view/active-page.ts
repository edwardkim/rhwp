export type ActivePageSource = 'editing' | 'viewport';

export interface ActivePageSnapshot {
  pageIndex: number;
  source: ActivePageSource;
}

export interface ActivePageCandidates {
  pageCount: number;
  visiblePages: readonly number[];
  editingPageIndex: number | null;
  viewportPageIndex: number | null;
}

export interface RulerPageCandidates {
  documentPageCount: number;
  layoutPageCount: number;
  focusedPageIndex: number | null;
  activePageIndex: number | null;
}

function isValidPageIndex(pageIndex: number | null, pageCount: number): pageIndex is number {
  return pageIndex !== null
    && Number.isInteger(pageIndex)
    && pageIndex >= 0
    && pageIndex < pageCount;
}

/**
 * 편집 focus와 뷰포트가 서로 다른 페이지를 가리킬 때 눈금자·상태 표시줄이 공유할
 * 활성 페이지를 고른다.
 *
 * 편집 페이지는 실제로 보일 때만 우선한다. 사용자가 캐럿을 그대로 둔 채 문서를
 * 스크롤해 그 페이지가 화면 밖으로 나가면, 눈금자와 현재 쪽 표시까지 화면 밖의
 * 편집 문맥에 붙잡히지 않고 뷰포트 페이지로 넘어간다.
 */
export function resolveActivePage({
  pageCount,
  visiblePages,
  editingPageIndex,
  viewportPageIndex,
}: ActivePageCandidates): ActivePageSnapshot | null {
  if (!Number.isInteger(pageCount) || pageCount <= 0) return null;

  const visible = visiblePages.filter(
    (pageIndex) => isValidPageIndex(pageIndex, pageCount),
  );
  if (visible.length === 0) return null;

  const visibleSet = new Set(visible);
  if (
    isValidPageIndex(editingPageIndex, pageCount)
    && visibleSet.has(editingPageIndex)
  ) {
    return { pageIndex: editingPageIndex, source: 'editing' };
  }

  if (
    isValidPageIndex(viewportPageIndex, pageCount)
    && visibleSet.has(viewportPageIndex)
  ) {
    return { pageIndex: viewportPageIndex, source: 'viewport' };
  }

  return { pageIndex: visible[0], source: 'viewport' };
}

/**
 * 눈금자는 뷰포트 표시기가 아니라 마지막 편집 focus의 조작 표면이다.
 * 순수 스크롤로 활성(가시) 페이지가 바뀌어도 focus가 유효하면 그 페이지를 유지하고,
 * 아직 편집 focus가 없을 때만 뷰포트 활성 페이지를 초기 fallback으로 쓴다.
 */
export function resolveRulerPageIndex({
  documentPageCount,
  layoutPageCount,
  focusedPageIndex,
  activePageIndex,
}: RulerPageCandidates): number | null {
  const pageCount = Math.min(documentPageCount, layoutPageCount);
  if (!Number.isInteger(pageCount) || pageCount <= 0) return null;
  if (isValidPageIndex(focusedPageIndex, pageCount)) return focusedPageIndex;
  if (isValidPageIndex(activePageIndex, pageCount)) return activePageIndex;
  return null;
}

/** 눈금자의 문단·셀 편집 문맥은 viewport 활성 쪽이 아니라 마지막 편집 focus에 속한다. */
export function hasRulerEditingContext(
  pageIndex: number,
  focusedPageIndex: number | null,
): boolean {
  return Number.isInteger(pageIndex)
    && pageIndex >= 0
    && pageIndex === focusedPageIndex;
}
