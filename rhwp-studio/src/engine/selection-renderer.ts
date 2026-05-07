import type { SelectionRect } from '@/core/types';
import { VirtualScroll } from '@/view/virtual-scroll';

/** 선택 영역을 파란색 반투명 사각형으로 렌더링한다 */
export class SelectionRenderer {
  private layer: HTMLDivElement;
  private highlights: HTMLDivElement[] = [];
  private activeCount = 0;
  private lastSignature = '';

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
  ) {
    this.layer = document.createElement('div');
    this.layer.className = 'selection-layer';
    this.layer.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:5;';
    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  /** 선택 사각형을 렌더링한다 */
  render(rects: SelectionRect[], zoom: number): void {
    this.ensureAttached();

    const scrollContent = this.container.querySelector('#scroll-content');
    if (!scrollContent || rects.length === 0) {
      this.clear();
      return;
    }

    const contentWidth = scrollContent?.clientWidth ?? 0;
    const layouts: string[] = [];

    for (const rect of rects) {
      const pageOffset = this.virtualScroll.getPageOffset(rect.pageIndex);
      const pageDisplayWidth = this.virtualScroll.getPageWidth(rect.pageIndex);
      const pageLeft = (contentWidth - pageDisplayWidth) / 2;
      layouts.push([
        pageLeft + rect.x * zoom,
        pageOffset + rect.y * zoom,
        rect.width * zoom,
        rect.height * zoom,
      ].map(v => v.toFixed(2)).join(','));
    }

    const signature = layouts.join('|');
    if (signature === this.lastSignature) return;

    for (let i = 0; i < layouts.length; i++) {
      const div = this.ensureHighlight(i);
      const [left, top, width, height] = layouts[i].split(',');

      div.style.left = `${left}px`;
      div.style.top = `${top}px`;
      div.style.width = `${width}px`;
      div.style.height = `${height}px`;
      div.style.display = 'block';
    }

    for (let i = layouts.length; i < this.activeCount; i++) {
      this.highlights[i].style.display = 'none';
    }
    this.activeCount = layouts.length;
    this.lastSignature = signature;
  }

  /** 모든 하이라이트를 제거한다 */
  clear(): void {
    if (this.activeCount === 0 && this.lastSignature === '') return;
    for (let i = 0; i < this.activeCount; i++) {
      this.highlights[i].style.display = 'none';
    }
    this.activeCount = 0;
    this.lastSignature = '';
  }

  /** 레이어가 DOM에 없으면 재부착한다 (loadDocument 후 컨테이너 교체 대응) */
  private ensureAttached(): void {
    if (this.layer.parentElement) return;
    const scrollContent = this.container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  private ensureHighlight(index: number): HTMLDivElement {
    let div = this.highlights[index];
    if (!div) {
      div = document.createElement('div');
      div.style.cssText =
        'position:absolute;background:rgba(51,144,255,0.35);pointer-events:none;display:none;';
      this.layer.appendChild(div);
      this.highlights[index] = div;
    }
    return div;
  }

  dispose(): void {
    this.clear();
    this.layer.remove();
  }
}
