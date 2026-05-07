import type { CursorRect } from '@/core/types';
import { VirtualScroll } from '@/view/virtual-scroll';

/** Canvas 위에 깜박이는 캐럿을 렌더링한다 */
export class CaretRenderer {
  private caretEl: HTMLDivElement;
  private blinkTimer: number | null = null;
  private visible = false;
  private currentRect: CursorRect | null = null;

  // IME 조합 오버레이
  private compEl: HTMLDivElement;
  private isCompMode = false;

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
  ) {
    this.caretEl = document.createElement('div');
    this.caretEl.className = 'caret';
    this.caretEl.style.cssText =
      'position:absolute;width:2px;background:#000;pointer-events:none;z-index:10;display:none;';

    // IME 조합 오버레이 (블랙박스 + 흰색 글자)
    this.compEl = document.createElement('div');
    this.compEl.className = 'caret-composition';
    this.compEl.style.cssText =
      'position:absolute;background:#000;color:#fff;pointer-events:none;z-index:10;display:none;' +
      'line-height:1;overflow:hidden;white-space:pre;text-align:center;box-sizing:border-box;';

    // scroll-content 안에 배치 (스크롤과 함께 이동)
    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.caretEl);
      scrollContent.appendChild(this.compEl);
    } else {
      container.appendChild(this.caretEl);
      container.appendChild(this.compEl);
    }
  }

  /** 캐럿을 표시한다 */
  show(rect: CursorRect, zoom: number): void {
    this.ensureAttached();
    this.currentRect = rect;
    this.updatePosition(zoom);
    this.caretEl.style.display = 'block';
    this.startBlink();
  }

  /** 캐럿을 숨긴다 */
  hide(): void {
    this.stopBlink();
    this.caretEl.style.display = 'none';
    this.compEl.style.display = 'none';
    this.isCompMode = false;
    this.currentRect = null;
  }

  /** 줌/스크롤 변경 시 위치를 갱신한다 */
  updatePosition(zoom: number): void {
    if (!this.currentRect) return;
    const { pageIndex, x, y, height } = this.currentRect;
    const pageOffset = this.virtualScroll.getPageOffset(pageIndex);
    const pageLeft = this.calcPageLeft(pageIndex);

    this.caretEl.style.left = `${pageLeft + x * zoom}px`;
    this.caretEl.style.top = `${pageOffset + y * zoom}px`;
    this.caretEl.style.height = `${height * zoom}px`;
  }

  /** 새 CursorRect로 갱신한다 (깜박임 리셋) */
  update(rect: CursorRect, zoom: number): void {
    this.ensureAttached();
    this.currentRect = rect;
    this.updatePosition(zoom);
    // 조합 모드가 아닐 때만 일반 캐럿 표시
    if (!this.isCompMode) {
      this.caretEl.style.display = 'block';
      this.caretEl.style.opacity = '1';
      this.visible = true;
      this.startBlink();
    }
  }

  /** 드래그 중 캐럿 위치를 갱신한다 (기존 깜박임 타이머 유지) */
  updateLive(rect: CursorRect, zoom: number): void {
    this.ensureAttached();
    this.currentRect = rect;
    this.updatePosition(zoom);
    if (!this.isCompMode) {
      this.caretEl.style.display = 'block';
      this.caretEl.style.opacity = '1';
      this.visible = true;
      if (this.blinkTimer === null) {
        this.startBlink();
      }
    }
  }

  /** IME 조합 오버레이를 표시한다 */
  showComposition(startRect: CursorRect, charWidth: number, zoom: number, text: string, fontFamily: string): void {
    this.ensureAttached();
    this.isCompMode = true;

    // 일반 캐럿 숨기기
    this.caretEl.style.display = 'none';

    const { pageIndex, x, y, height } = startRect;
    const pageOffset = this.virtualScroll.getPageOffset(pageIndex);
    const pageLeft = this.calcPageLeft(pageIndex);

    // 블랙박스 위치/크기
    const w = Math.max(charWidth, height * 0.6) * zoom;
    const h = height * zoom;
    const left = pageLeft + x * zoom;
    const top = pageOffset + y * zoom;

    this.compEl.style.left = `${left}px`;
    this.compEl.style.top = `${top}px`;
    this.compEl.style.width = `${w}px`;
    this.compEl.style.height = `${h}px`;
    this.compEl.style.fontSize = `${height * 0.85 * zoom}px`;
    this.compEl.style.fontFamily = fontFamily || 'sans-serif';
    this.compEl.style.lineHeight = `${h}px`;
    this.compEl.textContent = text;
    this.compEl.style.display = 'block';
    this.compEl.style.opacity = '1';
    this.visible = true;
    this.startBlink();
  }

  /** IME 조합 오버레이를 숨기고 일반 캐럿으로 복귀한다 */
  hideComposition(): void {
    if (!this.isCompMode) return;
    this.isCompMode = false;
    this.compEl.style.display = 'none';
  }

  /** 페이지의 화면 X 좌표를 계산한다 (그리드/단일 열 공통) */
  private calcPageLeft(pageIndex: number): number {
    const gridLeft = this.virtualScroll.getPageLeft(pageIndex);
    if (gridLeft >= 0) return gridLeft;
    // 단일 열: CSS 중앙 정렬 보정
    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIndex);
    return (contentWidth - pageDisplayWidth) / 2;
  }

  /** 캐럿 엘리먼트가 DOM에 없으면 재부착한다 (loadDocument 후 컨테이너 교체 대응) */
  private ensureAttached(): void {
    const scrollContent = this.container.querySelector('#scroll-content');
    if (this.caretEl.parentElement && this.compEl.parentElement) return;
    if (scrollContent) {
      if (!this.caretEl.parentElement) scrollContent.appendChild(this.caretEl);
      if (!this.compEl.parentElement) scrollContent.appendChild(this.compEl);
    }
  }

  private startBlink(): void {
    this.stopBlink();
    this.visible = true;
    const target = this.isCompMode ? this.compEl : this.caretEl;
    target.style.opacity = '1';
    this.blinkTimer = window.setInterval(() => {
      this.visible = !this.visible;
      target.style.opacity = this.visible ? '1' : '0';
    }, 500);
  }

  private stopBlink(): void {
    if (this.blinkTimer !== null) {
      clearInterval(this.blinkTimer);
      this.blinkTimer = null;
    }
  }

  dispose(): void {
    this.stopBlink();
    this.caretEl.remove();
    this.compEl.remove();
  }
}
