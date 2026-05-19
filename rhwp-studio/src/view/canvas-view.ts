import { WasmBridge } from '@/core/wasm-bridge';
import { EventBus } from '@/core/event-bus';
import type { PageInfo } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { CanvasPool } from './canvas-pool';
import { PageRenderer } from './page-renderer';
import { ViewportManager } from './viewport-manager';
import { CoordinateSystem } from './coordinate-system';
import type { CanvasKitLayerRenderer } from './canvaskit-renderer';
import { clampRenderScale, type RenderBackend } from './render-backend';
import type { LayerRenderProfile } from '@/core/types';

export class CanvasView {
  private virtualScroll: VirtualScroll;
  private canvasPool: CanvasPool;
  private pageRenderer: PageRenderer;
  private viewportManager: ViewportManager;
  private coordinateSystem: CoordinateSystem;

  private scrollContent: HTMLElement;
  private pages: PageInfo[] = [];
  private currentVisiblePages: number[] = [];
  private unsubscribers: (() => void)[] = [];

  constructor(
    private container: HTMLElement,
    private wasm: WasmBridge,
    private eventBus: EventBus,
    renderBackend: RenderBackend = 'canvas2d',
    renderProfile: LayerRenderProfile = 'screen',
    canvaskitRenderer: CanvasKitLayerRenderer | null = null,
  ) {
    this.virtualScroll = new VirtualScroll();
    this.canvasPool = new CanvasPool();
    this.pageRenderer = new PageRenderer(wasm, renderBackend, renderProfile, canvaskitRenderer);
    this.viewportManager = new ViewportManager(eventBus);
    this.coordinateSystem = new CoordinateSystem(this.virtualScroll);

    this.scrollContent = container.querySelector('#scroll-content')!;
    this.viewportManager.attachTo(container);

    this.unsubscribers.push(
      eventBus.on('viewport-scroll', () => this.updateVisiblePages()),
      eventBus.on('viewport-resize', () => this.onViewportResize()),
      eventBus.on('zoom-changed', (zoom) => this.onZoomChanged(zoom as number)),
      eventBus.on('document-changed', () => this.refreshPages()),
      eventBus.on('document-view-changed', () => this.refreshPages()),
    );
  }

  /** 문서 로드 후 호출 — 페이지 정보 수집 및 가상 스크롤 초기화 */
  loadDocument(): void {
    this.reset();

    const pageCount = this.wasm.pageCount;
    this.pages = [];
    for (let i = 0; i < pageCount; i++) {
      try {
        this.pages.push(this.wasm.getPageInfo(i));
      } catch (e) {
        console.error(`[CanvasView] 페이지 ${i} 정보 조회 실패:`, e);
      }
    }

    if (this.pages.length === 0) {
      console.error('[CanvasView] 로드된 페이지가 없습니다');
      return;
    }

    // 모바일: 문서 로드 시 폭 맞춤 줌 자동 적용
    if (window.innerWidth < 1024 && this.pages.length > 0) {
      const containerWidth = this.container.clientWidth - 20;
      const pageWidth = this.pages[0].width;
      if (pageWidth > 0 && containerWidth > 0) {
        const fitZoom = containerWidth / pageWidth;
        this.viewportManager.setZoom(Math.max(0.1, Math.min(fitZoom, 4.0)));
      }
    }

    this.recalcLayout();

    this.container.scrollTop = 0;
    this.updateVisiblePages();

    console.log(`[CanvasView] ${this.pages.length}/${pageCount}페이지 로드, 총 높이: ${this.virtualScroll.getTotalHeight()}px`);
  }

  /** 레이아웃을 재계산한다 (줌/리사이즈 공통) */
  private recalcLayout(): void {
    const zoom = this.viewportManager.getZoom();
    const { width: vpWidth } = this.viewportManager.getViewportSize();
    this.virtualScroll.setPageDimensions(this.pages, zoom, vpWidth);
    this.scrollContent.style.height = `${this.virtualScroll.getTotalHeight()}px`;
    this.scrollContent.style.width = `${this.virtualScroll.getTotalWidth()}px`;

    // 그리드 모드 CSS 클래스 토글
    this.scrollContent.classList.toggle('grid-mode', this.virtualScroll.isGridMode());
  }

  /** 스크롤/리사이즈 시 보이는 페이지를 갱신한다 */
  private updateVisiblePages(): void {
    const scrollY = this.viewportManager.getScrollY();
    const { height: vpHeight } = this.viewportManager.getViewportSize();

    const prefetchPages = this.virtualScroll.getPrefetchPages(scrollY, vpHeight);
    const visiblePages = this.virtualScroll.getVisiblePages(scrollY, vpHeight);

    // 벗어난 페이지 해제
    const prefetchSet = new Set(prefetchPages);
    for (const pageIdx of this.canvasPool.activePages) {
      if (!prefetchSet.has(pageIdx)) {
        this.pageRenderer.cancelReRender(pageIdx);
        this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
        this.canvasPool.release(pageIdx);
      }
    }

    // 새로 보이는 페이지 렌더링
    for (const pageIdx of prefetchPages) {
      if (!this.canvasPool.has(pageIdx)) {
        this.renderPage(pageIdx);
      }
    }

    // 현재 페이지 번호 갱신
    if (visiblePages.length > 0) {
      const vpCenter = scrollY + vpHeight / 2;
      const currentPage = this.virtualScroll.getPageAtY(vpCenter);
      this.eventBus.emit(
        'current-page-changed',
        currentPage,
        this.virtualScroll.pageCount,
      );
    }

    this.currentVisiblePages = visiblePages;
  }

  /** 단일 페이지를 렌더링한다 */
  private renderPage(pageIdx: number): void {
    const canvas = this.canvasPool.acquire(pageIdx);
    const zoom = this.viewportManager.getZoom();
    const rawDpr = window.devicePixelRatio || 1;

    const pageInfo = this.pages[pageIdx];
    if (!pageInfo) {
      console.error(`[CanvasView] 페이지 ${pageIdx} 정보가 없습니다`);
      this.canvasPool.release(pageIdx);
      return;
    }
    // iOS/WebKit과 GPU surface가 감당하기 어려운 물리 픽셀 수를 중앙 정책으로 제한한다.
    const renderScale = clampRenderScale(pageInfo, zoom * rawDpr);
    const dpr = renderScale / (zoom > 0 ? zoom : 1);

    // Canvas를 DOM에 추가하고 위치를 설정한다
    canvas.style.top = `${this.virtualScroll.getPageOffset(pageIdx)}px`;

    // 그리드 모드: 고정 left 좌표, 단일 열: CSS 중앙 정렬
    const pageLeft = this.virtualScroll.getPageLeft(pageIdx);
    if (pageLeft >= 0) {
      canvas.style.left = `${pageLeft}px`;
      canvas.style.transform = 'none';
    } else {
      canvas.style.left = '50%';
      canvas.style.transform = 'translateX(-50%)';
    }

    this.scrollContent.appendChild(canvas);

    // WASM이 Canvas 크기를 자동 설정한다 (물리 픽셀 = 페이지크기 × zoom × DPR)
    try {
      this.pageRenderer.renderPage(pageIdx, canvas, renderScale, zoom, dpr);
    } catch (e) {
      console.error(`[CanvasView] 페이지 ${pageIdx} 렌더링 실패:`, e);
      this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
      this.canvasPool.release(pageIdx);
      return;
    }

    // CSS 표시 크기 = 물리 픽셀 / DPR (= 페이지크기 × zoom)
    canvas.style.width = `${canvas.width / dpr}px`;
    canvas.style.height = `${canvas.height / dpr}px`;
  }

  /** 뷰포트 리사이즈 처리 */
  private onViewportResize(): void {
    if (this.pages.length === 0) {
      this.updateVisiblePages();
      return;
    }

    // 그리드 모드에서 열 수가 바뀔 수 있으므로 레이아웃 재계산
    const wasGrid = this.virtualScroll.isGridMode();
    this.recalcLayout();
    const isGrid = this.virtualScroll.isGridMode();

    if (wasGrid || isGrid) {
      // 그리드 관련 변경 시 전체 재렌더링
      this.releaseAllRenderedPages();
      this.pageRenderer.cancelAll();
    }
    this.updateVisiblePages();
  }

  /** 줌 변경 처리 */
  private onZoomChanged(zoom: number): void {
    if (this.pages.length === 0) return;

    // 현재 보이는 페이지 기억
    const scrollY = this.viewportManager.getScrollY();
    const { height: vpHeight } = this.viewportManager.getViewportSize();
    const vpCenter = scrollY + vpHeight / 2;
    const focusPage = this.virtualScroll.getPageAtY(vpCenter);
    const oldOffset = this.virtualScroll.getPageOffset(focusPage);
    const relativeY = vpCenter - oldOffset;
    const oldHeight = this.virtualScroll.getPageHeight(focusPage);
    const ratio = oldHeight > 0 ? relativeY / oldHeight : 0;

    // 페이지 크기 재계산
    this.recalcLayout();

    // 스크롤 위치 보정
    const newOffset = this.virtualScroll.getPageOffset(focusPage);
    const newHeight = this.virtualScroll.getPageHeight(focusPage);
    const newCenter = newOffset + newHeight * ratio;
    this.viewportManager.setScrollTop(newCenter - vpHeight / 2);

    // 모든 Canvas 재렌더링
    this.releaseAllRenderedPages();
    this.pageRenderer.cancelAll();
    this.updateVisiblePages();

    this.eventBus.emit('zoom-level-display', zoom);
  }

  /** 편집 후 보이는 페이지를 재렌더링한다 */
  refreshPages(): void {
    if (this.pages.length === 0) return;

    // 페이지 정보 재수집 (페이지 수/크기가 변경될 수 있음)
    const pageCount = this.wasm.pageCount;
    this.pages = [];
    for (let i = 0; i < pageCount; i++) {
      try {
        this.pages.push(this.wasm.getPageInfo(i));
      } catch (e) {
        console.error(`[CanvasView] 페이지 ${i} 정보 조회 실패:`, e);
      }
    }

    this.recalcLayout();

    // 보이는 페이지 재렌더링
    this.releaseAllRenderedPages();
    this.pageRenderer.cancelAll();
    this.updateVisiblePages();
  }

  /** 리소스를 정리한다 */
  private reset(): void {
    this.pageRenderer.cancelAll();
    this.releaseAllRenderedPages();
    this.currentVisiblePages = [];
    this.pages = [];
    this.scrollContent.replaceChildren();
  }

  private releaseAllRenderedPages(): void {
    this.pageRenderer.resetImageRetryState();
    this.pageRenderer.removeAllPageLayers(this.scrollContent);
    this.canvasPool.releaseAll();
  }

  /** 전체 정리 */
  dispose(): void {
    this.reset();
    this.pageRenderer.dispose();
    this.viewportManager.detach();
    for (const unsub of this.unsubscribers) {
      unsub();
    }
    this.unsubscribers = [];
  }

  getVirtualScroll(): VirtualScroll {
    return this.virtualScroll;
  }

  getViewportManager(): ViewportManager {
    return this.viewportManager;
  }

  getRenderBackend(): RenderBackend {
    return this.pageRenderer.getBackend();
  }

  getCoordinateSystem(): CoordinateSystem {
    return this.coordinateSystem;
  }
}
