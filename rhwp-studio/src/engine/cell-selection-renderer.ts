import type { CellBbox } from '@/core/types';
import { VirtualScroll } from '@/view/virtual-scroll';
import type { CellSelectionPhase, CellSelectionPoint } from './cell-selection-phase';

/** F5 셀 블록 선택 영역을 하이라이트 오버레이로 렌더링한다 */
export class CellSelectionRenderer {
  private layer: HTMLDivElement;
  private highlights: HTMLDivElement[] = [];

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
    private onPhaseChange: (phase: CellSelectionPhase | null) => void = () => {},
  ) {
    this.layer = document.createElement('div');
    this.layer.className = 'cell-selection-layer';
    this.layer.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:6;';
    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  /** 선택 범위 내 셀들을 하이라이트한다 */
  render(
    cellBboxes: CellBbox[],
    range: { startRow: number; startCol: number; endRow: number; endCol: number },
    zoom: number,
    excluded?: Set<string>,
    phase?: CellSelectionPhase,
    focus?: CellSelectionPoint,
  ): void {
    this.clearHighlights();
    this.ensureAttached();

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;

    for (const cell of cellBboxes) {
      // 셀이 선택 범위에 포함되는지 확인 (병합 셀 고려)
      const cellEndRow = cell.row + cell.rowSpan - 1;
      const cellEndCol = cell.col + cell.colSpan - 1;
      const overlaps =
        cell.row <= range.endRow && cellEndRow >= range.startRow &&
        cell.col <= range.endCol && cellEndCol >= range.startCol;
      if (!overlaps) continue;

      // Ctrl+클릭으로 제외된 셀인지 확인
      if (excluded && excluded.has(`${cell.row},${cell.col}`)) continue;

      const div = document.createElement('div');
      const pageOffset = this.virtualScroll.getPageOffset(cell.pageIndex);
      const pageDisplayWidth = this.virtualScroll.getPageWidth(cell.pageIndex);
      const pageLeft = (contentWidth - pageDisplayWidth) / 2;

      div.className = 'cell-selection-highlight';
      div.style.cssText =
        `position:absolute;` +
        `left:${pageLeft + cell.x * zoom}px;` +
        `top:${pageOffset + cell.y * zoom}px;` +
        `width:${cell.w * zoom}px;` +
        `height:${cell.h * zoom}px;`;
      this.layer.appendChild(div);
      this.highlights.push(div);
    }

    // 한컴과 같은 공간 근접 표시: 방향키가 움직이는 focus 셀의 중앙에만 단계 마커를 둔다.
    // 병합 셀에서는 focus 좌표를 포함하는 anchor bbox를 사용한다.
    if ((phase === 1 || phase === 2) && focus) {
      const focusCell = cellBboxes.find((cell) =>
        focus.row >= cell.row && focus.row < cell.row + cell.rowSpan &&
        focus.col >= cell.col && focus.col < cell.col + cell.colSpan
      );
      if (focusCell) {
        const marker = document.createElement('div');
        const pageOffset = this.virtualScroll.getPageOffset(focusCell.pageIndex);
        const pageDisplayWidth = this.virtualScroll.getPageWidth(focusCell.pageIndex);
        const pageLeft = (contentWidth - pageDisplayWidth) / 2;
        marker.className = phase === 1
          ? 'cell-selection-phase-marker cell-selection-phase-marker--single'
          : 'cell-selection-phase-marker cell-selection-phase-marker--range';
        marker.setAttribute('aria-hidden', 'true');
        marker.style.cssText =
          `left:${pageLeft + (focusCell.x + focusCell.w / 2) * zoom}px;` +
          `top:${pageOffset + (focusCell.y + focusCell.h / 2) * zoom}px;`;
        this.layer.appendChild(marker);
        this.highlights.push(marker);
      }
    }

    this.onPhaseChange(phase ?? null);
  }

  /** 모든 하이라이트를 제거한다 */
  clear(): void {
    this.clearHighlights();
    this.onPhaseChange(null);
  }

  private clearHighlights(): void {
    for (const div of this.highlights) {
      div.remove();
    }
    this.highlights = [];
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
