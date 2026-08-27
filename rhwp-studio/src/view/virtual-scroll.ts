import type { PageInfo } from '@/core/types';
import {
  DEFAULT_PAGE_ARRANGEMENT,
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';
import type { PageMovementDirection } from './page-movement.ts';
import { resolvePageGap } from './page-gap.ts';

/** 그리드 모드 전환 줌 임계값 */
const GRID_ZOOM_THRESHOLD = 0.5;

/** [#3591] 가로 팬 여백 = clamp(창 폭 × 비율, 하한, 상한). 상한이 큰 화면에서의 증가를 끊는다. */
const PAN_SPACE_RATIO = 0.25;
const MIN_PAN_SPACE = 80;
const MAX_PAN_SPACE = 240;

export class VirtualScroll {
  private pageOffsets: number[] = [];
  private pageHeights: number[] = [];
  private pageWidths: number[] = [];
  private pageLefts: number[] = [];
  private pageRows: number[] = [];
  private pageColumns: number[] = [];
  private rowPages: number[][] = [];
  private maxPageWidth = 0;
  private totalHeight = 0;
  private totalWidth = 0;
  private columns = 1;
  private gridMode = false;
  private horizontalMode = false;
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
    this.pageGap = resolvePageGap(zoom, this.pageGapAt100Percent);
    this.pageHeights = pages.map((p) => p.height * zoom);
    this.pageWidths = pages.map((p) => p.width * zoom);
    this.maxPageWidth = Math.max(...this.pageWidths, 0);

    this.horizontalMode = movement === 'horizontal';
    if (this.horizontalMode) {
      this.gridMode = false;
      this.layoutHorizontalRow(viewportWidth, viewportHeight);
      return;
    }

    const normalized = normalizePageArrangement(arrangement);
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
        // 기존 자동 동작: 50% 이하에서만 뷰포트에 들어가는 최대 열 수를 쓴다.
        this.gridMode = zoom <= GRID_ZOOM_THRESHOLD && viewportWidth > 0 && pages.length > 1;
        if (this.gridMode) {
          const columns = Math.max(
            1,
            Math.floor((viewportWidth + this.pageGap) / (this.maxPageWidth + this.pageGap)),
          );
          this.layoutUniformGrid(viewportWidth, columns);
        } else {
          this.layoutSingleColumn();
        }
        break;
    }
    this.applyHorizontalPanSpace(viewportWidth);
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
    // 있다. 그리드 첫 진입(zoom 0.5)에서만 팬이 붙어 스크롤 여지가 생기고 문서가 중앙에서
    // 밀리는 현상이 그것이다. 그리드에는 팬을 주지 않는다.
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

  /** 뷰포트에 보이는 페이지 인덱스 목록을 반환한다 */
  getVisiblePages(
    scrollY: number,
    viewportHeight: number,
    scrollX = 0,
    viewportWidth = 0,
  ): number[] {
    const vpTop = scrollY;
    const vpBottom = scrollY + viewportHeight;
    const vpLeft = scrollX;
    const vpRight = viewportWidth > 0 ? scrollX + viewportWidth : Infinity;
    const visible: number[] = [];

    for (let i = 0; i < this.pageOffsets.length; i++) {
      const pageTop = this.pageOffsets[i];
      const pageBottom = pageTop + this.pageHeights[i];
      const pageLeft = this.getPageLeftResolved(i, this.totalWidth);
      const pageRight = pageLeft + this.pageWidths[i];

      if (
        pageTop < vpBottom
        && pageBottom > vpTop
        && pageLeft < vpRight
        && pageRight > vpLeft
      ) {
        visible.push(i);
      }
    }
    return visible;
  }

  /** 프리페치 대상 페이지 (visible 범위 ± 1행) */
  getPrefetchPages(
    scrollY: number,
    viewportHeight: number,
    scrollX = 0,
    viewportWidth = 0,
  ): number[] {
    const visible = this.getVisiblePages(scrollY, viewportHeight, scrollX, viewportWidth);
    if (visible.length === 0) return [];

    const prefetch = new Set(visible);

    if (this.horizontalMode) {
      const first = visible[0];
      const last = visible[visible.length - 1];
      if (first > 0) prefetch.add(first - 1);
      if (last + 1 < this.pageCount) prefetch.add(last + 1);
      return Array.from(prefetch).sort((a, b) => a - b);
    }

    const visibleRows = visible.map((pageIdx) => this.pageRows[pageIdx] ?? 0);
    const firstRow = Math.min(...visibleRows);
    const lastRow = Math.max(...visibleRows);
    for (const row of [firstRow - 1, lastRow + 1]) {
      for (const pageIdx of this.rowPages[row] ?? []) prefetch.add(pageIdx);
    }

    return Array.from(prefetch).sort((a, b) => a - b);
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
    for (let i = this.pageOffsets.length - 1; i >= 0; i--) {
      if (docY >= this.pageOffsets[i]) {
        return i;
      }
    }
    return 0;
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
    const rowOffset = this.pageOffsets[rowLastIdx];
    let rowFirst = rowLastIdx;
    while (rowFirst > 0 && this.pageOffsets[rowFirst - 1] === rowOffset) rowFirst--;
    return rowFirst;
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
      let bestIdx = 0;
      let bestDist = Infinity;
      for (let pageIdx = 0; pageIdx < this.pageLefts.length; pageIdx++) {
        const left = this.pageLefts[pageIdx] ?? 0;
        const right = left + (this.pageWidths[pageIdx] ?? 0);
        if (docX >= left && docX <= right) return pageIdx;
        const dist = docX < left ? left - docX : docX - right;
        if (dist < bestDist) {
          bestDist = dist;
          bestIdx = pageIdx;
        }
      }
      return bestIdx;
    }

    const rowLastIdx = this.getPageAtY(docY);
    if (!this.gridMode) return rowLastIdx;

    // 같은 row 의 페이지 범위 (rowLastIdx 부터 row 시작까지)
    const rowOffset = this.pageOffsets[rowLastIdx];
    let rowFirst = rowLastIdx;
    while (rowFirst > 0 && this.pageOffsets[rowFirst - 1] === rowOffset) rowFirst--;

    // X 가 페이지 안에 속하는 첫 번째 페이지 반환
    for (let i = rowFirst; i <= rowLastIdx; i++) {
      const left = this.pageLefts[i] ?? 0;
      const right = left + (this.pageWidths[i] ?? 0);
      if (docX >= left && docX <= right) return i;
    }

    // gap / margin 영역 — 가장 가까운 페이지로 fallback
    let bestIdx = rowFirst;
    let bestDist = Infinity;
    for (let i = rowFirst; i <= rowLastIdx; i++) {
      const left = this.pageLefts[i] ?? 0;
      const right = left + (this.pageWidths[i] ?? 0);
      const dist = docX < left ? left - docX : (docX > right ? docX - right : 0);
      if (dist < bestDist) { bestDist = dist; bestIdx = i; }
    }
    return bestIdx;
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
