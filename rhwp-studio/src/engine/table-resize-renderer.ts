import { VirtualScroll } from '@/view/virtual-scroll';
import type { CellBbox } from '@/core/types';
import { computeBorderSpans, mergeBorderCoords, type BorderSpan } from './table-border-lines';

/** 경계선 종류 */
export type BorderEdgeType = 'row' | 'col';

/** 감지된 경계선 정보 */
export interface BorderEdge {
  type: BorderEdgeType;
  /** 경계선 인덱스 (행: 0=첫 행 상단, 열: 0=첫 열 좌측) */
  index: number;
  pageIndex: number;
}

/**
 * 경계선이 **실제로 존재하는** 구간. 병합 칸이 있으면 한 열/행 경계가 여러 토막으로 끊긴다.
 *
 * [#7191] 종전에는 선마다 표 전체 범위(`minX..maxX` / `minY..maxY`) 하나만 들고 있었다.
 * 그런데 적중 판정(`hitTestBorder`)은 칸 상자를 훑으므로, 그리는 범위와 잡는 범위가
 * 서로 다른 출처였다 — 경계가 두 칸에만 있는데 선은 표 높이 828px 를 가로질러 그려지고,
 * 그 구간에 마우스를 올리면 잡히지 않았다(3147199 1쪽 `x=374.0`·`x=446.5`).
 * 이제 둘 다 칸 상자에서 나온다.
 */
interface RowLine { y: number; spans: BorderSpan[]; index: number }
interface ColLine { x: number; spans: BorderSpan[]; index: number }

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

  /**
   * 셀 bbox 배열에서 행/열 경계선 좌표를 계산한다 (페이지 좌표 기준).
   *
   * rowIndexByY/colIndexByX 는 병합 **전** 반올림 좌표에서 대표 괘선 인덱스를 찾는 맵이다.
   * 셀 좌표로 인덱스를 되찾는 쪽(hitTestBorder)은 반드시 이 맵을 써야 한다 — 괘선 목록의
   * 대표 좌표만으로 맵을 다시 만들면, 병합돼 사라진 좌표를 가진 셀의 경계를 못 찾는다.
   */
  computeBorderLines(bboxes: CellBbox[]): {
    rowLines: RowLine[];
    colLines: ColLine[];
    rowIndexByY: Map<number, number>;
    colIndexByX: Map<number, number>;
  } {
    if (bboxes.length === 0) {
      return { rowLines: [], colLines: [], rowIndexByY: new Map(), colIndexByX: new Map() };
    }

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

    // 반올림 경계에 걸쳐 갈라진 같은 경계를 하나로 묶는다.
    const rows = mergeBorderCoords(rowYs);
    const cols = mergeBorderCoords(colXs);

    // [#7191] 각 경계선이 실제로 존재하는 구간을 칸 상자에서 모은다 — 적중 판정과 같은
    // 출처다. 아래 두 루프의 인덱스 조회는 `hitTestBorder` 의 것과 한 글자도 다르지 않다.
    const { rowSpans, colSpans } = computeBorderSpans(
      bboxes, rows.indexByCoord, cols.indexByCoord, ry,
    );

    const rowLines: RowLine[] = rows.positions.map((y, i) => ({
      y, spans: rowSpans.get(i) ?? [], index: i,
    }));

    const colLines: ColLine[] = cols.positions.map((x, i) => ({
      x, spans: colSpans.get(i) ?? [], index: i,
    }));

    return {
      rowLines,
      colLines,
      rowIndexByY: rows.indexByCoord,
      colIndexByX: cols.indexByCoord,
    };
  }

  /** 마우스 좌표가 경계선 위인지 판별한다 (페이지 좌표 기준) */
  hitTestBorder(
    pageX: number, pageY: number,
    bboxes: CellBbox[],
    tolerance = 4,
  ): BorderEdge | null {
    if (bboxes.length === 0) return null;

    // 인덱스 맵은 반드시 computeBorderLines 가 준 것을 쓴다. 괘선 목록의 대표 좌표로
    // 다시 만들면 병합돼 사라진 좌표를 가진 셀의 경계가 잡히지 않는다.
    const { rowIndexByY, colIndexByX } = this.computeBorderLines(bboxes);
    const pageIndex = bboxes[0].pageIndex;
    const rounded = (v: number) => Math.round(v * 10) / 10;

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
    const pageLeft = this.virtualScroll.getPageLeftResolved(edge.pageIndex, contentWidth);

    const t = TableResizeRenderer.MARKER_THICKNESS;
    const line = edge.type === 'row'
      ? rowLines.find(l => l.index === edge.index)
      : colLines.find(l => l.index === edge.index);
    if (!line || line.spans.length === 0) return;

    // [#7191] 경계가 실제로 있는 구간마다 하나씩 그린다. 병합 칸 때문에 한 경계가 여러
    // 토막으로 끊기면 그 토막들만 보이고, 잡히지 않는 구간에는 선도 없다.
    const el = document.createElement('div');
    el.style.cssText = 'position:absolute;left:0;top:0;pointer-events:none;';
    for (const span of line.spans) {
      const seg = document.createElement('div');
      if (edge.type === 'row') {
        const left = pageLeft + span.start * zoom;
        const top = pageOffset + (line as RowLine).y * zoom - t / 2;
        const width = (span.end - span.start) * zoom;
        seg.style.cssText =
          `position:absolute;` +
          `left:${left}px;top:${top}px;` +
          `width:${width}px;height:${t}px;` +
          `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
      } else {
        const left = pageLeft + (line as ColLine).x * zoom - t / 2;
        const top = pageOffset + span.start * zoom;
        const height = (span.end - span.start) * zoom;
        seg.style.cssText =
          `position:absolute;` +
          `left:${left}px;top:${top}px;` +
          `width:${t}px;height:${height}px;` +
          `background:${TableResizeRenderer.MARKER_COLOR};pointer-events:none;`;
      }
      el.appendChild(seg);
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
    const pageLeft = this.virtualScroll.getPageLeftResolved(pageIndex, contentWidth);

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
