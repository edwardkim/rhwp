import type { EventBus } from '@/core/event-bus';
import {
  CENTER_ZOOM_ANCHOR,
  normalizeZoomAnchor,
  type ZoomAnchor,
} from './zoom-anchor.ts';

const MIN_ZOOM = 0.25;
const MAX_ZOOM = 4.0;
const ZOOM_SETTLE_EPSILON = 0.001;
const ZOOM_SMOOTHING_TIME_MS = 16;
const WHEEL_ZOOM_SENSITIVITY = 0.00625;
const MAX_WHEEL_DELTA_PX = 120;

export class ViewportManager {
  private scrollY = 0;
  private scrollX = 0;
  private viewportWidth = 0;
  private viewportHeight = 0;
  private zoom = 1.0;
  private container: HTMLElement | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private resizeAnimationFrame: number | null = null;
  private scrollAnimationFrame: number | null = null;
  private zoomAnimationFrame: number | null = null;
  private zoomAnimationTimestamp: number | null = null;
  private zoomAnimating = false;
  private zoomTarget = 1.0;
  private zoomAnchor: ZoomAnchor = CENTER_ZOOM_ANCHOR;
  private onScrollBound: () => void;
  private onWheelBound: (e: WheelEvent) => void;
  private onZoomAnimationFrameBound: (timestamp: number) => void;
  private onResizeObserverErrorBound: (e: ErrorEvent) => void;
  private eventBus: EventBus;

  constructor(eventBus: EventBus) {
    this.eventBus = eventBus;
    this.onScrollBound = this.onScroll.bind(this);
    this.onWheelBound = this.onWheel.bind(this);
    this.onZoomAnimationFrameBound = this.onZoomAnimationFrame.bind(this);
    this.onResizeObserverErrorBound = this.onResizeObserverError.bind(this);
  }

  /** 스크롤 컨테이너에 연결한다 */
  attachTo(container: HTMLElement): void {
    this.detach();
    this.container = container;
    container.addEventListener('scroll', this.onScrollBound, { passive: true });
    container.addEventListener('wheel', this.onWheelBound, { passive: false });

    // 크로미움은 이 경고를 실제 스크립트 예외가 아니라 window `error` 이벤트로 합성
    // 보고한다. 아래 콜백이 동기 DOM 변이를 하지 않아도(매크로태스크로 미룸) 대형
    // 문서 초기 렌더처럼 같은 프레임에 다른 요소들이 대량으로 리사이즈되면 여전히
    // 뜰 수 있는, 기능에 영향 없는 잡음이라 uncaught error 로 새지 않게 막는다.
    window.addEventListener('error', this.onResizeObserverErrorBound);

    this.resizeObserver = new ResizeObserver(() => {
      if (this.resizeAnimationFrame !== null) return;

      // 리스너가 레이아웃을 다시 계산해 컨테이너 크기가 같은 프레임에 또 바뀌면
      // "ResizeObserver loop completed with undelivered notifications"가 발생한다.
      // rAF 는 **같은 렌더링 프레임 안**(레이아웃·RO 전달 이전)에 실행되므로 이 루프를
      // 벗어나지 못한다 — 실측으로 경고가 계속 발생했다. 매크로태스크(setTimeout 0)로
      // 프레임 경계를 넘겨 변이를 다음 프레임의 관찰 사이클로 미룬다.
      this.resizeAnimationFrame = window.setTimeout(() => {
        this.resizeAnimationFrame = null;
        if (!this.container) return;
        this.updateViewportSize();
        this.eventBus.emit('viewport-resize', this.viewportWidth, this.viewportHeight);
      }, 0);
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
    window.removeEventListener('error', this.onResizeObserverErrorBound);
    if (this.resizeAnimationFrame !== null) {
      clearTimeout(this.resizeAnimationFrame);
      this.resizeAnimationFrame = null;
    }
    if (this.scrollAnimationFrame !== null) {
      cancelAnimationFrame(this.scrollAnimationFrame);
      this.scrollAnimationFrame = null;
    }
    this.cancelZoomAnimation();
    this.container = null;
  }

  /** ResizeObserver 잡음 window error 를 uncaught 로 전파하지 않도록 막는다. */
  private onResizeObserverError(e: ErrorEvent): void {
    if (e.message?.includes('ResizeObserver loop')) {
      e.stopImmediatePropagation();
      e.preventDefault();
    }
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
    const deltaX = this.wheelDeltaPixels(e.deltaX, e.deltaMode);
    const deltaY = this.wheelDeltaPixels(e.deltaY, e.deltaMode);

    if (!e.ctrlKey && !e.metaKey) {
      if (
        this.container
        && !e.shiftKey
        && deltaY !== 0
        && Math.abs(deltaY) >= Math.abs(deltaX)
      ) {
        e.preventDefault();
        this.setScrollTop(this.container.scrollTop + deltaY);
      }
      return;
    }
    e.preventDefault();

    const boundedDelta = Math.max(
      -MAX_WHEEL_DELTA_PX,
      Math.min(MAX_WHEEL_DELTA_PX, deltaY),
    );
    if (boundedDelta === 0) return;

    const rect = this.container?.getBoundingClientRect();
    const anchor = rect && rect.width > 0 && rect.height > 0
      ? normalizeZoomAnchor({
        x: (e.clientX - rect.left) / rect.width,
        y: (e.clientY - rect.top) / rect.height,
      })
      : CENTER_ZOOM_ANCHOR;

    this.smoothZoomTo(
      this.zoomTarget * Math.exp(-boundedDelta * WHEEL_ZOOM_SENSITIVITY),
      anchor,
    );
  }

  private wheelDeltaPixels(delta: number, deltaMode: number): number {
    return deltaMode === 1
      ? delta * 16
      : deltaMode === 2
        ? delta * Math.max(this.viewportHeight, 1)
        : delta;
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

  setZoom(zoom: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
    this.cancelZoomAnimation();
    this.zoomAnchor = normalizeZoomAnchor(anchor);
    this.zoom = this.clampZoom(zoom);
    this.zoomTarget = this.zoom;
    this.eventBus.emit('zoom-changed', this.zoom, this.zoomAnchor);
  }

  smoothZoomBy(delta: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
    this.smoothZoomTo(this.zoomTarget + delta, anchor);
  }

  smoothZoomTo(zoom: number, anchor: ZoomAnchor = CENTER_ZOOM_ANCHOR): void {
    this.zoomAnchor = normalizeZoomAnchor(anchor);
    this.zoomTarget = this.clampZoom(zoom);
    if (Math.abs(this.zoomTarget - this.zoom) <= ZOOM_SETTLE_EPSILON) {
      this.setZoom(this.zoomTarget, this.zoomAnchor);
      return;
    }
    this.zoomAnimating = true;
    if (this.zoomAnimationFrame === null) {
      this.zoomAnimationFrame = requestAnimationFrame(this.onZoomAnimationFrameBound);
    }
  }

  isZoomAnimating(): boolean {
    return this.zoomAnimating;
  }

  private onZoomAnimationFrame(timestamp: number): void {
    this.zoomAnimationFrame = null;
    const elapsed = this.zoomAnimationTimestamp === null
      ? 16
      : Math.max(1, Math.min(timestamp - this.zoomAnimationTimestamp, 50));
    this.zoomAnimationTimestamp = timestamp;

    const progress = 1 - Math.exp(-elapsed / ZOOM_SMOOTHING_TIME_MS);
    const nextZoom = this.zoom + (this.zoomTarget - this.zoom) * progress;
    const settled = Math.abs(this.zoomTarget - nextZoom) <= ZOOM_SETTLE_EPSILON;
    this.zoom = settled ? this.zoomTarget : nextZoom;
    if (settled) {
      this.zoomAnimating = false;
      this.zoomAnimationTimestamp = null;
    }
    this.eventBus.emit('zoom-changed', this.zoom, this.zoomAnchor);

    if (!settled) {
      this.zoomAnimationFrame = requestAnimationFrame(this.onZoomAnimationFrameBound);
    }
  }

  private cancelZoomAnimation(): void {
    if (this.zoomAnimationFrame !== null) {
      cancelAnimationFrame(this.zoomAnimationFrame);
      this.zoomAnimationFrame = null;
    }
    this.zoomAnimationTimestamp = null;
    this.zoomAnimating = false;
    this.zoomTarget = this.zoom;
  }

  private clampZoom(zoom: number): number {
    return Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom));
  }

  setScrollTop(y: number): void {
    if (this.container) {
      this.container.scrollTop = y;
      this.scrollY = this.container.scrollTop;
    }
  }

  setScrollLeft(x: number): void {
    if (this.container) {
      this.container.scrollLeft = x;
      this.scrollX = this.container.scrollLeft;
    }
  }
}
