import { VirtualScroll } from '@/view/virtual-scroll';
import type { CellBbox } from '@/core/types';

/** 경계선 종류 */
export type BorderEdgeType = 'row' | 'col';

/** 감지된 경계선 정보 */
export interface BorderEdge {
  type: BorderEdgeType;
  /** 경계선 인덱스 (행: 0=첫 행 상단, 열: 0=첫 열 좌측) */
  index: number;
  pageIndex: number;
}

interface RowLine { y: number; xStart: number; xEnd: number; index: number }
interface ColLine { x: number; yStart: number; yEnd: number; index: number }

/** 표 셀 경계선 위 hover 시 마커(하이라이트 라인)를 표시한다 */
export class TableResizeRenderer {
  private layer: HTMLDivElement;
  private marker: HTMLDivElement | null = null;
  private static readonly MARKER_COLOR = 'rgba(0, 120, 215, 0.5)';
  private static readonly MARKER_THICKNESS = 3;

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
  ) {
    this.layer = document.createElement('div');
    this.layer.className = 'table-resize-layer';
    this.layer.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:6;';
    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  /** 셀 bbox 배열에서 행/열 경계선 좌표를 계산한다 (페이지 좌표 기준) */
  computeBorderLines(bboxes: CellBbox[]): { rowLines: RowLine[]; colLines: ColLine[] } {
    if (bboxes.length === 0) return { rowLines: [], colLines: [] };

    // 표 전체 범위
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const b of bboxes) {
      minX = Math.min(minX, b.x);
      maxX = Math.max(maxX, b.x + b.w);
      minY = Math.min(minY, b.y);
      maxY = Math.max(maxY, b.y + b.h);
    }

    // 행 경계선 (수평): 셀 상단/하단 y 좌표 수집
    const rowYSet = new Map<number, number>(); // y(rounded) → index
    // 열 경계선 (수직): 셀 좌측/우측 x 좌표 수집
    const colXSet = new Map<number, number>(); // x(rounded) → index

    // 표 상단/하단, 좌측/우측 추가
    const ry = (v: number) => Math.round(v * 10) / 10; // 소수점 1자리 반올림

    // 모든 셀의 상/하단, 좌/우측 좌표 수집
    const rowYs = new Set<number>();
    const colXs = new Set<number>();
    for (const b of bboxes) {
      rowYs.add(ry(b.y));
      rowYs.add(ry(b.y + b.h));
      colXs.add(ry(b.x));
      colXs.add(ry(b.x + b.w));
    }

    // 정렬하여 인덱스 부여
    const sortedRowYs = [...rowYs].sort((a, b) => a - b);
    const sortedColXs = [...colXs].sort((a, b) => a - b);

    const rowLines: RowLine[] = sortedRowYs.map((y, i) => ({
      y, xStart: minX, xEnd: maxX, index: i,
    }));

    const colLines: ColLine[] = sortedColXs.map((x, i) => ({
      x, yStart: minY, yEnd: maxY, index: i,
    }));

    return { rowLines, colLines };
  }

  /** 마우스 좌표가 경계선 위인지 판별한다 (페이지 좌표 기준) */
  hitTestBorder(
    pageX: number, pageY: number,
    bboxes: CellBbox[],
    tolerance = 4,
  ): BorderEdge | null {
    if (bboxes.length === 0) return null;

    const { rowLines, colLines } = this.computeBorderLines(bboxes);
    const pageIndex = bboxes[0].pageIndex;
    const rounded = (v: number) => Math.round(v * 10) / 10;
    const rowIndexByY = new Map(rowLines.map(line => [rounded(line.y), line.index]));
    const colIndexByX = new Map(colLines.map(line => [rounded(line.x), line.index]));

    const candidates: Array<{ edge: BorderEdge; distance: number; priority: number }> = [];

    // 행 경계선 검사 (수평선): 실제 셀 segment 위에서만 잡는다.
    for (const b of bboxes) {
      if (pageX < b.x - tolerance || pageX > b.x + b.w + tolerance) continue;

      const topIndex = rowIndexByY.get(rounded(b.y));
      if (topIndex !== undefined && Math.abs(pageY - b.y) <= tolerance) {
        candidates.push({
          edge: { type: 'row', index: topIndex, pageIndex },
          distance: Math.abs(pageY - b.y),
          priority: 1,
        });
      }

      const bottomY = b.y + b.h;
      const bottomIndex = rowIndexByY.get(rounded(bottomY));
      if (bottomIndex !== undefined && Math.abs(pageY - bottomY) <= tolerance) {
        candidates.push({
          edge: { type: 'row', index: bottomIndex, pageIndex },
          distance: Math.abs(pageY - bottomY),
          priority: 1,
        });
      }
    }

    // 열 경계선 검사 (수직선): 실제 셀 segment 위에서만 잡는다.
    for (const b of bboxes) {
      if (pageY < b.y - tolerance || pageY > b.y + b.h + tolerance) continue;

      const leftIndex = colIndexByX.get(rounded(b.x));
      if (leftIndex !== undefined && Math.abs(pageX - b.x) <= tolerance) {
        candidates.push({
          edge: { type: 'col', index: leftIndex, pageIndex },
          distance: Math.abs(pageX - b.x),
          priority: 0,
        });
      }

      const rightX = b.x + b.w;
      const rightIndex = colIndexByX.get(rounded(rightX));
      if (rightIndex !== undefined && Math.abs(pageX - rightX) <= tolerance) {
        candidates.push({
          edge: { type: 'col', index: rightIndex, pageIndex },
          distance: Math.abs(pageX - rightX),
          priority: 0,
        });
      }
    }

    if (candidates.length === 0) return null;
    candidates.sort((a, b) => a.distance - b.distance || a.priority - b.priority);
    return candidates[0].edge;
  }

  /** 경계선 위에 마커(하이라이트 라인)를 표시한다 */
  showMarker(
    edge: BorderEdge,
    bboxes: CellBbox[],
    zoom: number,
  ): void {
    this.clear();
    this.ensureAttached();
    if (bboxes.length === 0) return;

    const { rowLines, colLines } = this.computeBorderLines(bboxes);
    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageOffset = this.virtualScroll.getPageOffset(edge.pageIndex);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(edge.pageIndex);
    const pageLeft = (contentWidth - pageDisplayWidth) / 2;

    const t = TableResizeRenderer.MARKER_THICKNESS;
    const el = document.createElement('div');

    if (edge.type === 'row') {
      const line = rowLines.find(l => l.index === edge.index);
      if (!line) return;
      const left = pageLeft + line.xStart * zoom;
      const top = pageOffset + line.y * zoom - t / 2;
      const width = (line.xEnd - line.xStart) * zoom;
      el.style.cssText =
        `position:absolute;` +
        `left:${left}px;top:${top}px;` +
        `width:${width}px;height:${t}px;` +
        `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
    } else {
      const line = colLines.find(l => l.index === edge.index);
      if (!line) return;
      const left = pageLeft + line.x * zoom - t / 2;
      const top = pageOffset + line.yStart * zoom;
      const height = (line.yEnd - line.yStart) * zoom;
      el.style.cssText =
        `position:absolute;` +
        `left:${left}px;top:${top}px;` +
        `width:${t}px;height:${height}px;` +
        `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
    }

    this.layer.appendChild(el);
    this.marker = el;
  }

  /** 드래그 중 마커를 지정된 위치에 표시한다 (원래 경계선이 아닌 마우스 위치) */
  showDragMarker(
    type: BorderEdgeType,
    position: number, // row: pageY, col: pageX
    pageIndex: number,
    bboxes: CellBbox[],
    zoom: number,
    markerBboxes?: CellBbox[],
  ): void {
    this.clear();
    this.ensureAttached();
    if (bboxes.length === 0) return;
    const markerRange = markerBboxes && markerBboxes.length > 0 ? markerBboxes : bboxes;

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageOffset = this.virtualScroll.getPageOffset(pageIndex);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIndex);
    const pageLeft = (contentWidth - pageDisplayWidth) / 2;

    const t = TableResizeRenderer.MARKER_THICKNESS;
    const el = document.createElement('div');

    if (type === 'row') {
      const minX = Math.min(...markerRange.map(b => b.x));
      const maxX = Math.max(...markerRange.map(b => b.x + b.w));
      const left = pageLeft + minX * zoom;
      const top = pageOffset + position * zoom - t / 2;
      const width = (maxX - minX) * zoom;
      el.style.cssText =
        `position:absolute;left:${left}px;top:${top}px;` +
        `width:${width}px;height:${t}px;` +
        `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
    } else {
      const minY = Math.min(...markerRange.map(b => b.y));
      const maxY = Math.max(...markerRange.map(b => b.y + b.h));
      const left = pageLeft + position * zoom - t / 2;
      const top = pageOffset + minY * zoom;
      const height = (maxY - minY) * zoom;
      el.style.cssText =
        `position:absolute;left:${left}px;top:${top}px;` +
        `width:${t}px;height:${height}px;` +
        `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
    }

    this.layer.appendChild(el);
    this.marker = el;
  }

  /** 마커를 제거한다 */
  clear(): void {
    if (this.marker) {
      this.marker.remove();
      this.marker = null;
    }
  }

  /** 레이어가 DOM에 없으면 재부착한다 */
  private ensureAttached(): void {
    if (this.layer.parentElement) return;
    const scrollContent = this.container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  dispose(): void {
    this.clear();
    this.layer.remove();
  }
}
