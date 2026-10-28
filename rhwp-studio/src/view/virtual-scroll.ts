import type { PageInfo } from '@/core/types';
import {
  DEFAULT_PAGE_ARRANGEMENT,
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';
import type { PageMovementDirection } from './page-movement.ts';
import { resolvePageGap } from './page-gap.ts';

/** 자동 열 경계에서 resize/zoom 미세 입력이 왕복하지 않게 하는 CSS px 여유. */
const AUTO_COLUMN_HYSTERESIS_PX = 8;

/** [#3591] 가로 팬 여백 = clamp(창 폭 × 비율, 하한, 상한). 상한이 큰 화면에서의 증가를 끊는다. */
const PAN_SPACE_RATIO = 0.25;
const MIN_PAN_SPACE = 80;
const MAX_PAN_SPACE = 240;

export interface AutoPageColumnMetrics {
  pageCount: number;
  viewportWidth: number;
  displayedPageWidth: number;
  pageGap: number;
  committedColumns?: number;
  hysteresisPx?: number;
}

export interface VisibilityQueryStats {
  /** Y/X 인덱스가 실제 후보로 조사한 논리 행 수. */
  readonly rowsExamined: number;
  /** 기존 strict AABB predicate를 실제 적용한 페이지 수. */
  readonly pagesExamined: number;
}

export interface VisibilitySnapshot {
  readonly geometryRevision: number;
  readonly scrollX: number;
  readonly scrollY: number;
  readonly viewportWidth: number;
  readonly viewportHeight: number;
  readonly visiblePages: readonly number[];
  readonly visibleRows: readonly number[];
  readonly firstVisibleRow: number | null;
  readonly lastVisibleRow: number | null;
  /** visible 바깥에서 기존 ±1행/±1쪽 규칙으로 더한 후보. */
  readonly adjacentPages: readonly number[];
  readonly prefetchPages: readonly number[];
  readonly queryStats: VisibilityQueryStats;
}

interface VisibilityCacheKey {
  geometryRevision: number;
  scrollX: number;
  scrollY: number;
  viewportWidth: number;
  viewportHeight: number;
}

/**
 * 자동 쪽 배치의 가로 열 수를 표시 geometry만으로 결정한다.
 *
 * `committedColumns`를 생략하면 현재 폭에 들어가는 열 수를 바로 반환한다. 이전 commit을
 * 전달하면 인접 열 경계 양쪽에 작은 dead band를 두어 resize/zoom의 sub-pixel 흔들림으로
 * 열 수가 왕복하지 않게 한다.
 */
export function resolveAutoPageColumns({
  pageCount,
  viewportWidth,
  displayedPageWidth,
  pageGap,
  committedColumns,
  hysteresisPx = AUTO_COLUMN_HYSTERESIS_PX,
}: AutoPageColumnMetrics): number {
  const safePageCount = Number.isFinite(pageCount)
    ? Math.max(1, Math.floor(pageCount))
    : 1;
  const safeViewportWidth = Number.isFinite(viewportWidth) && viewportWidth > 0
    ? viewportWidth
    : 0;
  const safePageWidth = Number.isFinite(displayedPageWidth) && displayedPageWidth > 0
    ? displayedPageWidth
    : 0;
  const safeGap = Number.isFinite(pageGap) && pageGap >= 0 ? pageGap : 0;

  if (safeViewportWidth === 0 || safePageWidth === 0) return 1;

  const fitColumns = Math.max(
    1,
    Math.floor((safeViewportWidth + safeGap) / (safePageWidth + safeGap)),
  );
  const candidate = Math.min(safePageCount, fitColumns);
  if (committedColumns === undefined || !Number.isFinite(committedColumns)) return candidate;

  const committed = Math.min(
    safePageCount,
    Math.max(1, Math.floor(committedColumns)),
  );
  if (candidate === committed) return committed;

  const hysteresis = Number.isFinite(hysteresisPx)
    ? Math.max(0, hysteresisPx)
    : AUTO_COLUMN_HYSTERESIS_PX;
  if (candidate > committed) {
    const nextBoundary = (committed + 1) * safePageWidth + committed * safeGap;
    return safeViewportWidth >= nextBoundary + hysteresis ? candidate : committed;
  }

  const currentBoundary = committed * safePageWidth + (committed - 1) * safeGap;
  return safeViewportWidth < currentBoundary - hysteresis ? candidate : committed;
}

export class VirtualScroll {
  private pageOffsets: number[] = [];
  private pageHeights: number[] = [];
  private pageWidths: number[] = [];
  private pageLefts: number[] = [];
  private pageRows: number[] = [];
  private pageColumns: number[] = [];
  private rowPages: number[][] = [];
  private rowTops: number[] = [];
  private rowBottoms: number[] = [];
  private geometryRevision = 0;
  private visibilityCache: {
    key: VisibilityCacheKey;
    snapshot: VisibilitySnapshot;
  } | null = null;
  private maxPageWidth = 0;
  private totalHeight = 0;
  private totalWidth = 0;
  private columns = 1;
  private gridMode = false;
  private horizontalMode = false;
  private committedAutoColumns: number | undefined;
  private readonly pageGapAt100Percent: number;
  private pageGap: number;

  constructor(pageGapAt100Percent = 10) {
    this.pageGapAt100Percent = pageGapAt100Percent;
    this.pageGap = resolvePageGap(1, pageGapAt100Percent);
  }

  /** 페이지 크기 정보로 오프셋 배열을 구축한다 */
  setPageDimensions(
    pages: PageInfo[],
    zoom = 1.0,
    viewportWidth = 0,
    arrangement: PageArrangement = DEFAULT_PAGE_ARRANGEMENT,
    movement: PageMovementDirection = 'vertical',
    viewportHeight = 0,
  ): void {
    this.geometryRevision += 1;
    this.visibilityCache = null;
    this.pageGap = resolvePageGap(zoom, this.pageGapAt100Percent);
    this.pageHeights = pages.map((p) => p.height * zoom);
    this.pageWidths = pages.map((p) => p.width * zoom);
    this.maxPageWidth = Math.max(...this.pageWidths, 0);

    this.horizontalMode = movement === 'horizontal';
    if (this.horizontalMode) {
      this.committedAutoColumns = undefined;
      this.gridMode = false;
      this.layoutHorizontalRow(viewportWidth, viewportHeight);
      this.rebuildVisibilityIndex();
      return;
    }

    const normalized = normalizePageArrangement(arrangement);
    if (normalized.kind !== 'auto') this.committedAutoColumns = undefined;
    switch (normalized.kind) {
      case 'single':
        this.gridMode = false;
        this.layoutSingleColumn();
        break;
      case 'double':
        this.gridMode = true;
        this.layoutUniformGrid(viewportWidth, 2);
        break;
      case 'facing':
        this.gridMode = true;
        this.layoutFacingPages(viewportWidth);
        break;
      case 'multiple':
        this.gridMode = true;
        this.layoutUniformGrid(viewportWidth, normalized.columns);
        break;
      case 'auto':
      default:
        // 자동은 절대 배율 gate가 아니라 실제 표시 폭과 뷰포트 폭으로 열 수를 고른다.
        // pageCount cap은 존재하지 않는 빈 열이 중앙 정렬 폭에 포함되는 것도 막는다.
        {
          const columns = resolveAutoPageColumns({
            pageCount: pages.length,
            viewportWidth,
            displayedPageWidth: this.maxPageWidth,
            pageGap: this.pageGap,
            committedColumns: this.committedAutoColumns,
          });
          this.committedAutoColumns = columns;
          this.gridMode = columns > 1;
          if (this.gridMode) {
            this.layoutUniformGrid(viewportWidth, columns);
          } else {
            this.layoutSingleColumn();
          }
        }
        break;
    }
    this.applyHorizontalPanSpace(viewportWidth);
    this.rebuildVisibilityIndex();
  }

  /** 문서 교체 시 이전 문서의 자동 열 commit이 새 문서 경계 판단에 섞이지 않게 한다. */
  resetAutoColumnCommit(): void {
    this.committedAutoColumns = undefined;
  }

  /** 문서 수명 전환 중 늦은 viewport 갱신이 이전 문서 geometry를 보지 못하게 한다. */
  reset(): void {
    this.geometryRevision += 1;
    this.visibilityCache = null;
    this.pageOffsets = [];
    this.pageHeights = [];
    this.pageWidths = [];
    this.pageLefts = [];
    this.pageRows = [];
    this.pageColumns = [];
    this.rowPages = [];
    this.rowTops = [];
    this.rowBottoms = [];
    this.maxPageWidth = 0;
    this.totalHeight = 0;
    this.totalWidth = 0;
    this.columns = 1;
    this.gridMode = false;
    this.horizontalMode = false;
    this.committedAutoColumns = undefined;
  }

  /** 한컴 가로 쪽 이동: 한 쪽 배치의 모든 페이지를 왼쪽에서 오른쪽으로 잇는다. */
  private layoutHorizontalRow(viewportWidth: number, viewportHeight: number): void {
    this.columns = 1;
    this.pageOffsets = new Array(this.pageHeights.length).fill(0);
    this.pageLefts = new Array(this.pageHeights.length).fill(0);
    this.pageRows = new Array(this.pageHeights.length).fill(0);
    this.pageColumns = this.pageHeights.map((_, pageIdx) => pageIdx);
    this.rowPages = this.pageHeights.length > 0
      ? [this.pageHeights.map((_, pageIdx) => pageIdx)]
      : [];

    const innerWidth = this.pageWidths.reduce((sum, width) => sum + width, 0)
      + this.pageGap * Math.max(0, this.pageWidths.length - 1);
    const marginLeft = Math.max(this.pageGap, (viewportWidth - innerWidth) / 2);
    const maxPageHeight = Math.max(...this.pageHeights, 0);
    this.totalHeight = Math.max(viewportHeight, maxPageHeight + this.pageGap * 2);

    let left = marginLeft;
    for (let pageIdx = 0; pageIdx < this.pageWidths.length; pageIdx++) {
      this.pageLefts[pageIdx] = left;
      this.pageOffsets[pageIdx] = Math.max(
        this.pageGap,
        (this.totalHeight - this.pageHeights[pageIdx]) / 2,
      );
      left += this.pageWidths[pageIdx] + this.pageGap;
    }
    this.totalWidth = Math.max(viewportWidth, innerWidth + marginLeft * 2);
  }

  /** 단일 열 배치 (기존 동작) */
  private layoutSingleColumn(): void {
    this.columns = 1;
    this.pageOffsets = [];
    this.pageLefts = [];
    this.pageRows = [];
    this.pageColumns = [];
    this.rowPages = [];
    let offset = this.pageGap;
    for (let i = 0; i < this.pageHeights.length; i++) {
      this.pageOffsets.push(offset);
      this.pageLefts.push(-1); // -1 = CSS 중앙 정렬 사용
      this.pageRows.push(i);
      this.pageColumns.push(0);
      this.rowPages.push([i]);
      offset += this.pageHeights[i] + this.pageGap;
    }
    this.totalHeight = offset;
    this.totalWidth = this.maxPageWidth + 40;
  }

  /** 연속 페이지를 고정 열 수로 배치한다. */
  private layoutUniformGrid(viewportWidth: number, columns: number): void {
    const safeColumns = Math.max(1, Math.floor(columns));
    const slots = this.pageHeights.map((_, pageIdx) => ({
      pageIdx,
      row: Math.floor(pageIdx / safeColumns),
      col: pageIdx % safeColumns,
    }));
    this.layoutPageSlots(viewportWidth, safeColumns, slots);
  }

  /** 첫 홀수 쪽을 오른쪽에 둔 뒤 짝수/홀수 맞쪽을 구성한다. */
  private layoutFacingPages(viewportWidth: number): void {
    const slots = this.pageHeights.map((_, pageIdx) => ({
      pageIdx,
      row: Math.floor((pageIdx + 1) / 2),
      col: pageIdx % 2 === 0 ? 1 : 0,
    }));
    this.layoutPageSlots(viewportWidth, 2, slots);
  }

  /** 실제 페이지 인덱스와 행/열 슬롯의 대응을 공통 좌표 배열로 변환한다. */
  private layoutPageSlots(
    viewportWidth: number,
    columns: number,
    slots: { pageIdx: number; row: number; col: number }[],
  ): void {
    const gap = this.pageGap;
    const pw = this.maxPageWidth;
    this.columns = columns;
    this.pageOffsets = new Array(this.pageHeights.length).fill(0);
    this.pageLefts = new Array(this.pageHeights.length).fill(0);
    this.pageRows = new Array(this.pageHeights.length).fill(0);
    this.pageColumns = new Array(this.pageHeights.length).fill(0);
    this.rowPages = [];

    if (slots.length === 0) {
      this.totalHeight = 0;
      this.totalWidth = Math.max(0, viewportWidth);
      return;
    }

    // 그리드 전체 너비 = columns * pageWidth + (columns-1) * gap
    const gridWidth = this.columns * pw + (this.columns - 1) * gap;
    const marginLeft = Math.max(gap, (viewportWidth - gridWidth) / 2);

    const rowCount = Math.max(...slots.map((slot) => slot.row)) + 1;
    const rowHeights = new Array(rowCount).fill(0);
    for (const { pageIdx, row } of slots) {
      this.pageRows[pageIdx] = row;
      (this.rowPages[row] ??= []).push(pageIdx);
      rowHeights[row] = Math.max(rowHeights[row], this.pageHeights[pageIdx] ?? 0);
    }

    const rowTops = new Array(rowCount).fill(gap);
    for (let row = 1; row < rowCount; row++) {
      rowTops[row] = rowTops[row - 1] + rowHeights[row - 1] + gap;
    }

    for (const { pageIdx, row, col } of slots) {
      this.pageColumns[pageIdx] = col;
      this.pageOffsets[pageIdx] = rowTops[row];
      this.pageLefts[pageIdx] = marginLeft
        + col * (pw + gap)
        + (pw - (this.pageWidths[pageIdx] ?? 0)) / 2;
    }

    const lastRow = rowCount - 1;
    this.totalHeight = rowTops[lastRow] + rowHeights[lastRow] + gap;
    this.totalWidth = Math.max(gridWidth + marginLeft * 2, viewportWidth);
  }

  /**
   * [#3591] 가로 팬 여백을 계산한다.
   *
   * 종전에는 편측 여백이 창 폭 100% 라, 스크롤 영역의 대부분이 빈 공간이었고
   * 창이 커질수록(4K 최대화 등) 함께 커졌다 — 화면이 클수록 문서는 작아 보이는데
   * 빈 스크롤만 길어지는 반대 동작이었다.
   *
   * 정책: 콘텐츠가 창 안에 들어가면 팬이 필요 없으므로 0(브라우저 자연 중앙 정렬
   * 회복). 창보다 넓은 광폭 문서에만 창 폭의 일부를 여유로 주되, 상한이 화면 크기
   * 증가를 끊는다.
   */
  private horizontalPanSpace(viewportWidth: number, contentWidth: number): number {
    // 그리드는 layoutGrid 의 marginLeft 가 이미 중앙을 잡고, base 가 항상 창 폭 이상
    // (`max(gridWidth + marginLeft*2, viewportWidth)`)이라 팬 조건이 경계에서 참이 될 수
    // 있다. 자동 열 수가 1→다중 열로 바뀌는 순간 팬이 붙으면 스크롤 여지가 생기고 문서가
    // 중앙에서 밀린다. 그리드에는 팬을 주지 않는다.
    if (this.gridMode) return 0;
    if (contentWidth <= viewportWidth) return 0;
    const ratio = viewportWidth * PAN_SPACE_RATIO;
    return Math.min(Math.max(ratio, MIN_PAN_SPACE), MAX_PAN_SPACE);
  }

  private applyHorizontalPanSpace(viewportWidth: number): void {
    if (viewportWidth <= 0) return;
    const baseWidth = this.totalWidth;
    const pan = this.horizontalPanSpace(viewportWidth, baseWidth);
    if (pan <= 0) {
      // 팬 없음: 단일 열은 CSS 중앙 정렬(-1)을 그대로 두고, 그리드는 자체
      // marginLeft 가 이미 중앙을 잡는다. totalWidth 도 baseWidth 그대로다.
      return;
    }
    this.pageLefts = this.pageLefts.map((left, pageIdx) => {
      const resolved = left >= 0
        ? left
        : (baseWidth - (this.pageWidths[pageIdx] ?? 0)) / 2;
      return resolved + pan;
    });
    this.totalWidth = baseWidth + pan * 2;
  }

  /** layout 결과만 읽는 파생 인덱스. 기존 page 좌표의 권위는 바꾸지 않는다. */
  private rebuildVisibilityIndex(): void {
    this.rowTops = new Array(this.rowPages.length).fill(0);
    this.rowBottoms = new Array(this.rowPages.length).fill(0);
    for (let row = 0; row < this.rowPages.length; row++) {
      const pages = this.rowPages[row] ?? [];
      let top = Infinity;
      let bottom = -Infinity;
      for (const pageIdx of pages) {
        const pageTop = this.pageOffsets[pageIdx] ?? 0;
        top = Math.min(top, pageTop);
        bottom = Math.max(bottom, pageTop + (this.pageHeights[pageIdx] ?? 0));
      }
      this.rowTops[row] = top === Infinity ? 0 : top;
      this.rowBottoms[row] = bottom === -Infinity ? 0 : bottom;
    }
  }

  private sameVisibilityKey(a: VisibilityCacheKey, b: VisibilityCacheKey): boolean {
    return a.geometryRevision === b.geometryRevision
      && Object.is(a.scrollX, b.scrollX)
      && Object.is(a.scrollY, b.scrollY)
      && Object.is(a.viewportWidth, b.viewportWidth)
      && Object.is(a.viewportHeight, b.viewportHeight);
  }

  private firstIndexWhere(length: number, predicate: (index: number) => boolean): number {
    let low = 0;
    let high = length;
    while (low < high) {
      const mid = low + Math.floor((high - low) / 2);
      if (predicate(mid)) high = mid;
      else low = mid + 1;
    }
    return low;
  }

  private candidateRows(vpTop: number, vpBottom: number): number[] {
    const rowCount = this.rowPages.length;
    if (rowCount === 0) return [];
    if (Number.isNaN(vpTop) || Number.isNaN(vpBottom) || vpBottom < vpTop) {
      return Array.from({ length: rowCount }, (_, row) => row);
    }
    const first = this.firstIndexWhere(rowCount, row => this.rowBottoms[row] > vpTop);
    const end = this.firstIndexWhere(rowCount, row => this.rowTops[row] >= vpBottom);
    if (first >= end) return [];
    return Array.from({ length: end - first }, (_, offset) => first + offset);
  }

  private horizontalCandidatePages(vpLeft: number, vpRight: number): number[] {
    const count = this.pageCount;
    if (count === 0) return [];
    if (Number.isNaN(vpLeft) || Number.isNaN(vpRight) || vpRight < vpLeft) {
      return Array.from({ length: count }, (_, pageIdx) => pageIdx);
    }
    const first = this.firstIndexWhere(
      count,
      pageIdx => (this.pageLefts[pageIdx] ?? 0) + (this.pageWidths[pageIdx] ?? 0) > vpLeft,
    );
    const end = vpRight === Infinity
      ? count
      : this.firstIndexWhere(count, pageIdx => (this.pageLefts[pageIdx] ?? 0) >= vpRight);
    if (first >= end) return [];
    return Array.from({ length: end - first }, (_, offset) => first + offset);
  }

  /**
   * 같은 geometry/viewport 입력의 visible·prefetch를 한 번만 계산한 불변 결과.
   * 기존 wrapper는 이 snapshot을 복사해 반환하므로 호출자가 캐시 배열을 바꿀 수 없다.
   */
  getVisibilitySnapshot(
    scrollY: number,
    viewportHeight: number,
    scrollX = 0,
    viewportWidth = 0,
  ): VisibilitySnapshot {
    const key: VisibilityCacheKey = {
      geometryRevision: this.geometryRevision,
      scrollX,
      scrollY,
      viewportWidth,
      viewportHeight,
    };
    if (this.visibilityCache && this.sameVisibilityKey(this.visibilityCache.key, key)) {
      return this.visibilityCache.snapshot;
    }

    const vpTop = scrollY;
    const vpBottom = scrollY + viewportHeight;
    const vpLeft = scrollX;
    const vpRight = viewportWidth > 0 ? scrollX + viewportWidth : Infinity;
    const candidateRows = this.horizontalMode
      ? (this.rowPages.length > 0 ? [0] : [])
      : this.candidateRows(vpTop, vpBottom);
    const candidates = this.horizontalMode
      ? this.horizontalCandidatePages(vpLeft, vpRight)
      : candidateRows.flatMap(row => this.rowPages[row] ?? []);
    const visible: number[] = [];
    for (const pageIdx of candidates) {
      const pageTop = this.pageOffsets[pageIdx];
      const pageBottom = pageTop + this.pageHeights[pageIdx];
      const pageLeft = this.getPageLeftResolved(pageIdx, this.totalWidth);
      const pageRight = pageLeft + this.pageWidths[pageIdx];
      if (
        pageTop < vpBottom
        && pageBottom > vpTop
        && pageLeft < vpRight
        && pageRight > vpLeft
      ) visible.push(pageIdx);
    }
    visible.sort((a, b) => a - b);

    const visibleRows = [...new Set(visible.map(pageIdx => this.pageRows[pageIdx] ?? 0))]
      .sort((a, b) => a - b);
    const firstVisibleRow = visibleRows[0] ?? null;
    const lastVisibleRow = visibleRows[visibleRows.length - 1] ?? null;
    const prefetch = new Set(visible);
    if (visible.length > 0) {
      if (this.horizontalMode) {
        const first = visible[0];
        const last = visible[visible.length - 1];
        if (first > 0) prefetch.add(first - 1);
        if (last + 1 < this.pageCount) prefetch.add(last + 1);
      } else if (firstVisibleRow !== null && lastVisibleRow !== null) {
        for (const row of [firstVisibleRow - 1, lastVisibleRow + 1]) {
          for (const pageIdx of this.rowPages[row] ?? []) prefetch.add(pageIdx);
        }
      }
    }
    const prefetchPages = [...prefetch].sort((a, b) => a - b);
    const visibleSet = new Set(visible);
    const adjacentPages = prefetchPages.filter(pageIdx => !visibleSet.has(pageIdx));
    const snapshot: VisibilitySnapshot = Object.freeze({
      geometryRevision: this.geometryRevision,
      scrollX,
      scrollY,
      viewportWidth,
      viewportHeight,
      visiblePages: Object.freeze(visible),
      visibleRows: Object.freeze(visibleRows),
      firstVisibleRow,
      lastVisibleRow,
      adjacentPages: Object.freeze(adjacentPages),
      prefetchPages: Object.freeze(prefetchPages),
      queryStats: Object.freeze({
        rowsExamined: candidateRows.length,
        pagesExamined: candidates.length,
      }),
    });
    this.visibilityCache = { key, snapshot };
    return snapshot;
  }

  /** 뷰포트에 보이는 페이지 인덱스 목록을 반환한다 */
  getVisiblePages(
    scrollY: number,
    viewportHeight: number,
    scrollX = 0,
    viewportWidth = 0,
  ): number[] {
    return [...this.getVisibilitySnapshot(
      scrollY,
      viewportHeight,
      scrollX,
      viewportWidth,
    ).visiblePages];
  }

  /** 프리페치 대상 페이지 (visible 범위 ± 1행) */
  getPrefetchPages(
    scrollY: number,
    viewportHeight: number,
    scrollX = 0,
    viewportWidth = 0,
  ): number[] {
    return [...this.getVisibilitySnapshot(
      scrollY,
      viewportHeight,
      scrollX,
      viewportWidth,
    ).prefetchPages];
  }

  /** 특정 문서 Y 좌표가 속하는 페이지 인덱스를 반환한다 */
  /**
   * Y 가 속한 행의 **마지막** 쪽 인덱스.
   *
   * 그리드 모드에서 한 행의 모든 쪽은 같은 offset 을 가지므로(layoutGrid),
   * 뒤에서부터 스캔하는 이 함수는 그 행의 최대 인덱스를 돌려준다.
   * `getPageAtPoint` 가 X 로 좁히기 위한 스캔 끝점으로 쓴다.
   *
   * "현재 쪽" 이 필요하면 [`getRowFirstPageAtY`] 를 쓸 것 — [#2560].
   */
  getPageAtY(docY: number): number {
    if (this.horizontalMode) return 0;
    if (this.rowPages.length === 0 || Number.isNaN(docY)) return 0;
    if (docY < this.rowTops[0]) return 0;
    const firstAfter = this.firstIndexWhere(
      this.rowTops.length,
      row => this.rowTops[row] > docY,
    );
    const row = Math.max(0, firstAfter - 1);
    const pages = this.rowPages[row] ?? [];
    return pages[pages.length - 1] ?? 0;
  }

  /**
   * Y 가 속한 행의 **첫** 쪽 인덱스 — 사람이 말하는 "현재 쪽".
   *
   * 단일 컬럼 모드에서는 `getPageAtY` 와 동치다.
   */
  getRowFirstPageAtY(docY: number): number {
    if (this.horizontalMode) return 0;
    const rowLastIdx = this.getPageAtY(docY);
    if (!this.gridMode) return rowLastIdx;
    const row = this.pageRows[rowLastIdx] ?? 0;
    return this.rowPages[row]?.[0] ?? rowLastIdx;
  }

  private pageAtX(pageIndices: readonly number[], docX: number): number {
    if (pageIndices.length === 0) return 0;
    if (!Number.isFinite(docX)) return pageIndices[0];
    const firstNotLeft = this.firstIndexWhere(
      pageIndices.length,
      index => {
        const pageIdx = pageIndices[index];
        const right = (this.pageLefts[pageIdx] ?? 0) + (this.pageWidths[pageIdx] ?? 0);
        return right >= docX;
      },
    );
    if (firstNotLeft >= pageIndices.length) return pageIndices[pageIndices.length - 1];
    const nextPage = pageIndices[firstNotLeft];
    const nextLeft = this.pageLefts[nextPage] ?? 0;
    if (docX >= nextLeft) return nextPage;
    if (firstNotLeft === 0) return nextPage;

    const previousPage = pageIndices[firstNotLeft - 1];
    const previousRight = (this.pageLefts[previousPage] ?? 0)
      + (this.pageWidths[previousPage] ?? 0);
    return docX - previousRight <= nextLeft - docX ? previousPage : nextPage;
  }

  /** 한 행에 놓이는 쪽 수. 단일 컬럼 모드는 1. */
  get pagesPerRow(): number {
    return this.gridMode ? this.columns : 1;
  }

  /**
   * 문서 좌표 (X, Y) 가 속하는 페이지 인덱스를 반환한다.
   * 단일 컬럼 모드: getPageAtY 와 동치 (X 무관).
   * 그리드 모드: row(Y) 결정 후 같은 row 안에서 X 가 속하는 페이지 반환.
   *              gap 영역(페이지 사이 빈 공간) click 은 가장 가까운 페이지로 fallback.
   */
  getPageAtPoint(docX: number, docY: number): number {
    if (this.horizontalMode) {
      return this.pageAtX(this.rowPages[0] ?? [], docX);
    }
    if (this.rowPages.length === 0 || Number.isNaN(docY) || docY < this.rowTops[0]) return 0;

    const rowLastIdx = this.getPageAtY(docY);
    if (!this.gridMode) return rowLastIdx;
    const row = this.pageRows[rowLastIdx] ?? 0;
    return this.pageAtX(this.rowPages[row] ?? [], docX);
  }

  getPageOffset(pageIdx: number): number {
    return this.pageOffsets[pageIdx] ?? 0;
  }

  getPageHeight(pageIdx: number): number {
    return this.pageHeights[pageIdx] ?? 0;
  }

  getPageWidth(pageIdx: number): number {
    return this.pageWidths[pageIdx] ?? 0;
  }

  /** 페이지의 X 좌표를 반환한다 (-1이면 CSS 중앙 정렬 사용) */
  getPageLeft(pageIdx: number): number {
    return this.pageLefts[pageIdx] ?? -1;
  }

  /**
   * 페이지의 X 좌표를 그리드/단일 컬럼 모드 통합으로 반환.
   * 그리드 모드: pageLefts[i] 그대로.
   * 단일 컬럼 모드(sentinel −1): (containerWidth - pageWidth) / 2 fallback.
   */
  getPageLeftResolved(pageIdx: number, containerWidth: number): number {
    const pl = this.pageLefts[pageIdx] ?? -1;
    if (pl >= 0) return pl;
    const pw = this.pageWidths[pageIdx] ?? 0;
    return (containerWidth - pw) / 2;
  }

  getMaxPageWidth(): number {
    return this.maxPageWidth;
  }

  /** 페이지 사이/위아래 여백(px). 빈 쪽 자리표시자도 같은 여백을 쓴다. */
  getPageGap(): number {
    return this.pageGap;
  }

  getTotalHeight(): number {
    return this.totalHeight;
  }

  getTotalWidth(): number {
    return this.totalWidth;
  }

  getCenteredScrollLeft(viewportWidth: number): number {
    if (this.horizontalMode) return 0;
    return Math.max(0, (this.totalWidth - viewportWidth) / 2);
  }

  isGridMode(): boolean {
    return this.gridMode;
  }

  isHorizontalMode(): boolean {
    return this.horizontalMode;
  }

  getColumns(): number {
    return this.columns;
  }

  /** Canvas 내용 재사용 여부를 판정하는 행·열 슬롯 토폴로지 키. 좌표·배율은 포함하지 않는다. */
  getLayoutTopologyKey(): string {
    const direction = this.horizontalMode ? 'horizontal' : 'vertical';
    return `${direction}|${this.columns}|${this.pageRows.join(',')}|${this.pageColumns.join(',')}`;
  }

  /** 위에서 아래 순서의 실제 행 시작 페이지. 맞쪽의 빈 슬롯은 목록에 들어가지 않는다. */
  getRowStartPages(): number[] {
    return this.rowPages.flatMap((pages) => pages.length > 0 ? [pages[0]] : []);
  }

  get pageCount(): number {
    return this.pageOffsets.length;
  }

  get gap(): number {
    return this.pageGap;
  }
}
