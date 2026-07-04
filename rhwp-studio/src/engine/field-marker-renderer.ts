import type { CursorRect } from '@/core/types';
import { VirtualScroll } from '@/view/virtual-scroll';

/**
 * 누름틀(ClickHere) 필드 경계에 낫표 마커(「 」)를 렌더링한다.
 *
 * 커서가 필드 내부에 있을 때만 표시되며,
 * 필드 시작 위치에 「, 끝 위치에 」를 빨간색으로 표시한다.
 */
export class FieldMarkerRenderer {
  private startEl: HTMLDivElement;
  private endEl: HTMLDivElement;
  private visible = false;

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
  ) {
    const style =
      'position:absolute;pointer-events:none;z-index:9;display:none;' +
      'color:#ff0000;font-weight:bold;line-height:1;user-select:none;';

    this.startEl = document.createElement('div');
    this.startEl.className = 'field-marker field-marker-start';
    this.startEl.style.cssText = style;
    this.startEl.textContent = '「';

    this.endEl = document.createElement('div');
    this.endEl.className = 'field-marker field-marker-end';
    this.endEl.style.cssText = style;
    this.endEl.textContent = '」';

    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.startEl);
      scrollContent.appendChild(this.endEl);
    } else {
      container.appendChild(this.startEl);
      container.appendChild(this.endEl);
    }
  }

  /** 필드 시작/끝 위치에 낫표 마커를 표시한다 */
  show(startRect: CursorRect, endRect: CursorRect, zoom: number): void {
    this.ensureAttached();
    this.visible = true;

    const fontSize = startRect.height * zoom;

    // 시작 마커 「 — 필드 시작 경계에 맞춘다.
    const startPageOffset = this.virtualScroll.getPageOffset(startRect.pageIndex);
    const startPageLeft = this.calcPageLeft(startRect.pageIndex);
    this.startEl.style.fontSize = `${fontSize}px`;
    this.startEl.style.left = `${startPageLeft + startRect.x * zoom}px`;
    this.startEl.style.top = `${startPageOffset + startRect.y * zoom}px`;
    this.startEl.style.height = `${startRect.height * zoom}px`;
    this.startEl.style.display = 'block';

    // 끝 마커 」 — 필드 끝점 오른쪽에 배치
    const endPageOffset = this.virtualScroll.getPageOffset(endRect.pageIndex);
    const endPageLeft = this.calcPageLeft(endRect.pageIndex);
    this.endEl.style.fontSize = `${fontSize}px`;
    this.endEl.style.left = `${endPageLeft + endRect.x * zoom}px`;
    this.endEl.style.top = `${endPageOffset + endRect.y * zoom}px`;
    this.endEl.style.height = `${endRect.height * zoom}px`;
    this.endEl.style.display = 'block';
  }

  /** 마커를 숨긴다 */
  hide(): void {
    this.visible = false;
    this.startEl.style.display = 'none';
    this.endEl.style.display = 'none';
  }

  /** 줌/스크롤 변경 시 위치 갱신용 — show()를 다시 호출하는 방식으로 처리 */
  get isVisible(): boolean {
    return this.visible;
  }

  /** 페이지의 화면 X 좌표를 계산한다 */
  private calcPageLeft(pageIndex: number): number {
    const gridLeft = this.virtualScroll.getPageLeft(pageIndex);
    if (gridLeft >= 0) return gridLeft;
    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIndex);
    return (contentWidth - pageDisplayWidth) / 2;
  }

  /** DOM에서 분리된 경우 재부착 */
  private ensureAttached(): void {
    const scrollContent = this.container.querySelector('#scroll-content');
    if (this.startEl.parentElement && this.endEl.parentElement) return;
    if (scrollContent) {
      if (!this.startEl.parentElement) scrollContent.appendChild(this.startEl);
      if (!this.endEl.parentElement) scrollContent.appendChild(this.endEl);
    }
  }

  dispose(): void {
    this.startEl.remove();
    this.endEl.remove();
  }
}
