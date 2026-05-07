import type { PageInfo } from '@/core/types';

/** 그리드 모드 전환 줌 임계값 */
const GRID_ZOOM_THRESHOLD = 0.5;

export class VirtualScroll {
  private pageOffsets: number[] = [];
  private pageHeights: number[] = [];
  private pageWidths: number[] = [];
  private pageLefts: number[] = [];
  private maxPageWidth = 0;
  private totalHeight = 0;
  private totalWidth = 0;
  private columns = 1;
  private gridMode = false;
  private readonly pageGap: number;

  constructor(pageGap = 10) {
    this.pageGap = pageGap;
  }

  /** 페이지 크기 정보로 오프셋 배열을 구축한다 */
  setPageDimensions(pages: PageInfo[], zoom = 1.0, viewportWidth = 0): void {
    this.pageHeights = pages.map((p) => p.height * zoom);
    this.pageWidths = pages.map((p) => p.width * zoom);
    this.maxPageWidth = Math.max(...this.pageWidths, 0);

    // 그리드 모드 판정
    this.gridMode = zoom <= GRID_ZOOM_THRESHOLD && viewportWidth > 0 && pages.length > 1;

    if (this.gridMode) {
      this.layoutGrid(viewportWidth);
    } else {
      this.layoutSingleColumn();
    }
  }

  /** 단일 열 배치 (기존 동작) */
  private layoutSingleColumn(): void {
    this.columns = 1;
    this.pageOffsets = [];
    this.pageLefts = [];
    let offset = this.pageGap;
    for (let i = 0; i < this.pageHeights.length; i++) {
      this.pageOffsets.push(offset);
      this.pageLefts.push(-1); // -1 = CSS 중앙 정렬 사용
      offset += this.pageHeights[i] + this.pageGap;
    }
    this.totalHeight = offset;
    this.totalWidth = this.maxPageWidth + 40;
  }

  /** 그리드(다중 열) 배치 */
  private layoutGrid(viewportWidth: number): void {
    const gap = this.pageGap;
    const pw = this.maxPageWidth;

    // 열 수 계산: 뷰포트에 들어가는 최대 열 수
    this.columns = Math.max(1, Math.floor((viewportWidth + gap) / (pw + gap)));

    this.pageOffsets = [];
    this.pageLefts = [];

    // 그리드 전체 너비 = columns * pageWidth + (columns-1) * gap
    const gridWidth = this.columns * pw + (this.columns - 1) * gap;
    const marginLeft = Math.max(gap, (viewportWidth - gridWidth) / 2);

    let rowTop = gap;
    for (let i = 0; i < this.pageHeights.length; i++) {
      const col = i % this.columns;
      if (col === 0 && i > 0) {
        // 이전 행의 최대 높이만큼 이동
        const rowStart = i - this.columns;
        let maxH = 0;
        for (let j = rowStart; j < i && j < this.pageHeights.length; j++) {
          maxH = Math.max(maxH, this.pageHeights[j]);
        }
        rowTop += maxH + gap;
      }
      this.pageOffsets.push(rowTop);
      this.pageLefts.push(marginLeft + col * (pw + gap));
    }

    // 마지막 행 높이 추가
    const lastRowStart = Math.floor((this.pageHeights.length - 1) / this.columns) * this.columns;
    let lastRowMaxH = 0;
    for (let j = lastRowStart; j < this.pageHeights.length; j++) {
      lastRowMaxH = Math.max(lastRowMaxH, this.pageHeights[j]);
    }
    this.totalHeight = rowTop + lastRowMaxH + gap;
    this.totalWidth = Math.max(gridWidth + marginLeft * 2, viewportWidth);
  }

  /** 뷰포트에 보이는 페이지 인덱스 목록을 반환한다 */
  getVisiblePages(scrollY: number, viewportHeight: number): number[] {
    const vpTop = scrollY;
    const vpBottom = scrollY + viewportHeight;
    const visible: number[] = [];

    for (let i = 0; i < this.pageOffsets.length; i++) {
      const pageTop = this.pageOffsets[i];
      const pageBottom = pageTop + this.pageHeights[i];

      if (pageTop < vpBottom && pageBottom > vpTop) {
        visible.push(i);
      }
    }
    return visible;
  }

  /** 프리페치 대상 페이지 (visible 범위 ± 1행) */
  getPrefetchPages(scrollY: number, viewportHeight: number): number[] {
    const visible = this.getVisiblePages(scrollY, viewportHeight);
    if (visible.length === 0) return [];

    const first = visible[0];
    const last = visible[visible.length - 1];
    const prefetch = new Set(visible);

    // 이전/다음 행 추가
    const cols = this.columns;
    for (let c = 0; c < cols; c++) {
      const prev = first - cols + c;
      const next = last + 1 + c;
      if (prev >= 0) prefetch.add(prev);
      if (next < this.pageCount) prefetch.add(next);
    }

    return Array.from(prefetch).sort((a, b) => a - b);
  }

  /** 특정 문서 Y 좌표가 속하는 페이지 인덱스를 반환한다 */
  getPageAtY(docY: number): number {
    for (let i = this.pageOffsets.length - 1; i >= 0; i--) {
      if (docY >= this.pageOffsets[i]) {
        return i;
      }
    }
    return 0;
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

  getTotalHeight(): number {
    return this.totalHeight;
  }

  getTotalWidth(): number {
    return this.totalWidth;
  }

  isGridMode(): boolean {
    return this.gridMode;
  }

  getColumns(): number {
    return this.columns;
  }

  get pageCount(): number {
    return this.pageOffsets.length;
  }

  get gap(): number {
    return this.pageGap;
  }
}
