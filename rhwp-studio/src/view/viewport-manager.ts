import { EventBus } from '@/core/event-bus';

export class ViewportManager {
  private scrollY = 0;
  private scrollX = 0;
  private viewportWidth = 0;
  private viewportHeight = 0;
  private zoom = 1.0;
  private container: HTMLElement | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private scrollAnimationFrame: number | null = null;
  private onScrollBound: () => void;
  private onWheelBound: (e: WheelEvent) => void;

  constructor(private eventBus: EventBus) {
    this.onScrollBound = this.onScroll.bind(this);
    this.onWheelBound = this.onWheel.bind(this);
  }

  /** 스크롤 컨테이너에 연결한다 */
  attachTo(container: HTMLElement): void {
    this.detach();
    this.container = container;
    container.addEventListener('scroll', this.onScrollBound, { passive: true });
    container.addEventListener('wheel', this.onWheelBound, { passive: false });

    this.resizeObserver = new ResizeObserver(() => {
      this.updateViewportSize();
      this.eventBus.emit('viewport-resize', this.viewportWidth, this.viewportHeight);
    });
    this.resizeObserver.observe(container);
    this.updateViewportSize();
  }

  /** 연결을 해제한다 */
  detach(): void {
    if (this.container) {
      this.container.removeEventListener('scroll', this.onScrollBound);
      this.container.removeEventListener('wheel', this.onWheelBound);
    }
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    if (this.scrollAnimationFrame !== null) {
      cancelAnimationFrame(this.scrollAnimationFrame);
      this.scrollAnimationFrame = null;
    }
    this.container = null;
  }

  private onScroll(): void {
    if (!this.container) return;
    this.scrollY = this.container.scrollTop;
    this.scrollX = this.container.scrollLeft;
    if (this.scrollAnimationFrame !== null) return;

    // 스크롤 중에는 최신 좌표만 다음 프레임에 반영해 렌더가 입력 이벤트를 막지 않게 한다.
    this.scrollAnimationFrame = requestAnimationFrame(() => {
      this.scrollAnimationFrame = null;
      this.eventBus.emit('viewport-scroll', this.scrollY, this.scrollX);
    });
  }

  /** Ctrl+휠: 브라우저 줌 대신 문서 줌 */
  private onWheel(e: WheelEvent): void {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();

    // deltaY < 0 → 확대, deltaY > 0 → 축소
    const step = 0.1;
    const direction = e.deltaY < 0 ? 1 : -1;
    this.setZoom(this.zoom + step * direction);
  }

  private updateViewportSize(): void {
    if (!this.container) return;
    this.viewportWidth = this.container.clientWidth;
    this.viewportHeight = this.container.clientHeight;
  }

  getScrollY(): number {
    return this.scrollY;
  }

  getScrollX(): number {
    return this.scrollX;
  }

  getViewportSize(): { width: number; height: number } {
    return { width: this.viewportWidth, height: this.viewportHeight };
  }

  getZoom(): number {
    return this.zoom;
  }

  setZoom(zoom: number): void {
    this.zoom = Math.max(0.25, Math.min(4.0, zoom));
    this.eventBus.emit('zoom-changed', this.zoom);
  }

  setScrollTop(y: number): void {
    if (this.container) {
      this.container.scrollTop = y;
      this.scrollY = this.container.scrollTop;
    }
  }
}
