import { WasmBridge } from '@/core/wasm-bridge';
import { EventBus } from '@/core/event-bus';
import type { PageInfo } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { CanvasPool } from './canvas-pool';
import { PageRenderer, type PageRenderContext, type PageRenderResult } from './page-renderer';
import { ViewportManager } from './viewport-manager';
import { CoordinateSystem } from './coordinate-system';
import type { CanvasKitLayerRenderer, CanvasKitRenderDiagnostics } from './canvaskit-renderer';
import { clampRenderScale, type RenderBackend } from './render-backend';
import type { LayerRenderProfile } from '@/core/types';
import { applyGridOverlayBox, createGridClipCornerOverlay, createGridOverlay } from './grid-overlay';
import { getGridViewSettings } from './grid-settings';

const TEXT_EDIT_STATIC_LAYER_VERIFY_DELAY_MS = 800;

type DeferredPrefetchTask =
  | { kind: 'idle'; id: number }
  | { kind: 'timeout'; id: number };

type IdleCallbackWindow = Window & {
  requestIdleCallback?: (callback: () => void, options?: { timeout: number }) => number;
  cancelIdleCallback?: (id: number) => void;
};

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
  private pendingTextEditRefreshes = new Map<number, PageRenderContext>();
  private textEditRefreshRafId: number | null = null;
  private textEditStaticLayerVerifyTimers = new Map<number, ReturnType<typeof setTimeout>>();
  private pendingPrefetchPages = new Set<number>();
  private deferredPrefetchTask: DeferredPrefetchTask | null = null;

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
      eventBus.on('document-page-invalidated', (payload) => this.refreshInvalidatedPage(payload)),
      eventBus.on('document-changed', () => this.refreshPages()),
      eventBus.on('document-view-changed', () => this.refreshPages()),
      eventBus.on('grid-view-changed', () => this.refreshGridOverlays()),
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
    const visibleSet = new Set(visiblePages);

    // 벗어난 페이지 해제
    const prefetchSet = new Set(prefetchPages);
    for (const pageIdx of this.canvasPool.activePages) {
      if (!prefetchSet.has(pageIdx)) {
        this.cancelPendingTextEditRefresh(pageIdx);
        this.cancelTextEditStaticLayerVerification(pageIdx);
        this.pageRenderer.cancelReRender(pageIdx);
        this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
        this.removeGridOverlay(pageIdx);
        this.canvasPool.release(pageIdx);
      }
    }

    // 현재 보이는 페이지는 즉시 렌더한다. 인접 페이지는 스크롤 입력 뒤에 처리한다.
    for (const pageIdx of visiblePages) {
      this.pendingPrefetchPages.delete(pageIdx);
      if (!this.canvasPool.has(pageIdx)) {
        this.renderPage(pageIdx);
      }
    }
    this.schedulePrefetchPages(prefetchPages.filter((pageIdx) => !visibleSet.has(pageIdx)));

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

  /** 스크롤 중에는 다음 페이지의 선렌더를 idle time으로 미룬다. */
  private schedulePrefetchPages(pageIndices: readonly number[]): void {
    const candidateSet = new Set(pageIndices);
    for (const pageIdx of this.pendingPrefetchPages) {
      if (!candidateSet.has(pageIdx)) this.pendingPrefetchPages.delete(pageIdx);
    }
    for (const pageIdx of pageIndices) {
      if (!this.canvasPool.has(pageIdx)) this.pendingPrefetchPages.add(pageIdx);
    }
    if (this.pendingPrefetchPages.size === 0 || this.deferredPrefetchTask !== null) return;

    const run = () => {
      this.deferredPrefetchTask = null;
      const pages = Array.from(this.pendingPrefetchPages);
      this.pendingPrefetchPages.clear();
      for (const pageIdx of pages) {
        if (!this.canvasPool.has(pageIdx)) this.renderPage(pageIdx);
      }
    };
    const idleWindow = window as IdleCallbackWindow;
    if (typeof idleWindow.requestIdleCallback === 'function') {
      this.deferredPrefetchTask = {
        kind: 'idle',
        id: idleWindow.requestIdleCallback(run, { timeout: 1000 }),
      };
      return;
    }
    this.deferredPrefetchTask = {
      kind: 'timeout',
      id: window.setTimeout(run, 250),
    };
  }

  private cancelPendingPrefetch(): void {
    const task = this.deferredPrefetchTask;
    this.deferredPrefetchTask = null;
    this.pendingPrefetchPages.clear();
    if (!task) return;

    if (task.kind === 'idle') {
      (window as IdleCallbackWindow).cancelIdleCallback?.(task.id);
    } else {
      clearTimeout(task.id);
    }
  }

  /** 단일 페이지를 렌더링한다 */
  private renderPage(pageIdx: number): void {
    const canvas = this.canvasPool.acquire(pageIdx);
    if (!canvas.parentElement) {
      this.scrollContent.appendChild(canvas);
    }
    if (!this.renderCanvas(pageIdx, canvas)) {
      this.canvasPool.release(pageIdx);
    }
  }

  /** 기존 canvas를 유지한 채 페이지 내용을 다시 그린다. */
  private renderCanvas(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderContext: PageRenderContext = {},
  ): boolean {
    const zoom = this.viewportManager.getZoom();
    const rawDpr = window.devicePixelRatio || 1;

    const pageInfo = this.pages[pageIdx];
    if (!pageInfo) {
      console.error(`[CanvasView] 페이지 ${pageIdx} 정보가 없습니다`);
      return false;
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

    // WASM이 Canvas 크기를 자동 설정한다 (물리 픽셀 = 페이지크기 × zoom × DPR)
    let renderResult: PageRenderResult = { needsTextEditStaticLayerVerification: false };
    let renderedCanvas = canvas;
    try {
      renderResult = this.pageRenderer.renderPage(pageIdx, canvas, renderScale, zoom, dpr, renderContext);
      if (renderResult.renderedCanvas && renderResult.renderedCanvas !== canvas) {
        renderedCanvas = renderResult.renderedCanvas;
        this.canvasPool.replace(pageIdx, canvas, renderedCanvas);
      }
    } catch (e) {
      console.error(`[CanvasView] 페이지 ${pageIdx} 렌더링 실패:`, e);
      this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
      this.removeGridOverlay(pageIdx);
      return false;
    }

    // CSS 표시 크기 = 물리 픽셀 / DPR (= 페이지크기 × zoom)
    renderedCanvas.style.width = `${renderedCanvas.width / dpr}px`;
    renderedCanvas.style.height = `${renderedCanvas.height / dpr}px`;
    this.renderGridOverlay(pageIdx, renderedCanvas);
    if (renderResult.needsTextEditStaticLayerVerification) {
      this.scheduleTextEditStaticLayerVerification(pageIdx);
    } else if (renderContext.reason !== 'text-edit') {
      this.cancelTextEditStaticLayerVerification(pageIdx);
    }
    return true;
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
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
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
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
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
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.releaseAllRenderedPages();
    this.pageRenderer.cancelAll();
    this.updateVisiblePages();
  }

  /** 텍스트 입력처럼 좁은 변경은 page info 재수집 없이 해당 페이지 canvas만 다시 그린다. */
  private refreshInvalidatedPage(payload: unknown): void {
    if (this.pages.length === 0) return;

    const pageIndex =
      typeof payload === 'object' && payload !== null && 'pageIndex' in payload
        ? Number((payload as { pageIndex?: unknown }).pageIndex)
        : Number(payload);
    const reason =
      typeof payload === 'object' && payload !== null && 'reason' in payload
        ? (payload as { reason?: unknown }).reason
        : undefined;
    const renderContext: PageRenderContext =
      reason === 'text-edit'
        ? { reason: 'text-edit', allowStaticOverlayReuse: true }
        : { reason: 'unknown', allowStaticOverlayReuse: false };

    if (!Number.isInteger(pageIndex) || pageIndex < 0) {
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.refreshPages();
      return;
    }

    const pageCount = this.wasm.pageCount;
    if (pageCount !== this.pages.length || pageIndex >= pageCount) {
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.refreshPages();
      return;
    }

    if (renderContext.reason === 'text-edit') {
      this.scheduleTextEditPageRefresh(pageIndex, renderContext);
      return;
    }

    this.cancelPendingTextEditRefresh(pageIndex);
    this.cancelTextEditStaticLayerVerification(pageIndex);
    this.refreshInvalidatedPageNow(pageIndex, renderContext);
  }

  private scheduleTextEditPageRefresh(pageIndex: number, renderContext: PageRenderContext): void {
    this.cancelTextEditStaticLayerVerification(pageIndex);
    this.pendingTextEditRefreshes.set(pageIndex, renderContext);
    if (this.textEditRefreshRafId !== null) return;

    this.textEditRefreshRafId = requestAnimationFrame(() => {
      this.textEditRefreshRafId = null;
      const pending = Array.from(this.pendingTextEditRefreshes.entries());
      this.pendingTextEditRefreshes.clear();
      for (const [pendingPageIndex, pendingContext] of pending) {
        this.refreshInvalidatedPageNow(pendingPageIndex, pendingContext);
      }
    });
  }

  private refreshInvalidatedPageNow(pageIndex: number, renderContext: PageRenderContext): void {
    if (this.pages.length === 0) return;

    const pageCount = this.wasm.pageCount;
    if (pageCount !== this.pages.length || pageIndex >= pageCount) {
      this.refreshPages();
      return;
    }

    const canvas = this.canvasPool.getCanvas(pageIndex);
    if (!canvas) {
      this.updateVisiblePages();
      return;
    }

    if (!this.renderCanvas(pageIndex, canvas, renderContext)) {
      this.canvasPool.release(pageIndex);
      this.updateVisiblePages();
    }
  }

  private cancelPendingTextEditRefresh(pageIndex?: number): void {
    if (typeof pageIndex === 'number') {
      this.pendingTextEditRefreshes.delete(pageIndex);
    } else {
      this.pendingTextEditRefreshes.clear();
    }
    if (this.pendingTextEditRefreshes.size > 0) return;
    if (this.textEditRefreshRafId !== null) {
      cancelAnimationFrame(this.textEditRefreshRafId);
      this.textEditRefreshRafId = null;
    }
  }

  private scheduleTextEditStaticLayerVerification(pageIndex: number): void {
    this.cancelTextEditStaticLayerVerification(pageIndex);
    const timer = setTimeout(() => {
      this.textEditStaticLayerVerifyTimers.delete(pageIndex);
      this.refreshInvalidatedPageNow(pageIndex, { reason: 'unknown', allowStaticOverlayReuse: false });
    }, TEXT_EDIT_STATIC_LAYER_VERIFY_DELAY_MS);
    this.textEditStaticLayerVerifyTimers.set(pageIndex, timer);
  }

  private cancelTextEditStaticLayerVerification(pageIndex?: number): void {
    if (typeof pageIndex === 'number') {
      const timer = this.textEditStaticLayerVerifyTimers.get(pageIndex);
      if (timer) clearTimeout(timer);
      this.textEditStaticLayerVerifyTimers.delete(pageIndex);
      return;
    }

    for (const timer of this.textEditStaticLayerVerifyTimers.values()) {
      clearTimeout(timer);
    }
    this.textEditStaticLayerVerifyTimers.clear();
  }

  /** 리소스를 정리한다 */
  private reset(): void {
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.cancelPendingPrefetch();
    this.pageRenderer.cancelAll();
    this.releaseAllRenderedPages();
    this.currentVisiblePages = [];
    this.pages = [];
    this.scrollContent.replaceChildren();
  }

  private releaseAllRenderedPages(): void {
    this.pageRenderer.resetImageRetryState();
    this.pageRenderer.removeAllPageLayers(this.scrollContent);
    this.removeAllGridOverlays();
    this.canvasPool.releaseAll();
  }

  private refreshGridOverlays(): void {
    this.removeAllGridOverlays();
    for (const pageIdx of this.canvasPool.activePages) {
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (canvas) this.renderGridOverlay(pageIdx, canvas);
    }
  }

  private renderGridOverlay(pageIdx: number, canvas: HTMLCanvasElement): void {
    this.removeGridOverlay(pageIdx);
    const settings = getGridViewSettings();
    if (!settings.visible) return;

    const pageInfo = this.pages[pageIdx];
    if (!pageInfo) return;

    const overlay = createGridOverlay(
      pageIdx,
      pageInfo,
      this.viewportManager.getZoom(),
      settings,
    );
    applyGridOverlayBox(overlay, canvas);
    this.scrollContent.appendChild(overlay);

    const clipCorners = createGridClipCornerOverlay(
      pageIdx,
      pageInfo,
      this.viewportManager.getZoom(),
      settings,
    );
    if (clipCorners) {
      applyGridOverlayBox(clipCorners, canvas);
      this.scrollContent.appendChild(clipCorners);
    }
  }

  private removeGridOverlay(pageIdx: number): void {
    this.scrollContent
      .querySelectorAll(`[data-rhwp-grid-page="${pageIdx}"]`)
      .forEach((el) => el.remove());
  }

  private removeAllGridOverlays(): void {
    this.scrollContent
      .querySelectorAll('[data-rhwp-grid-page]')
      .forEach((el) => el.remove());
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

  getCanvasKitRenderDiagnostics(pageIndex: number): CanvasKitRenderDiagnostics | null {
    return this.pageRenderer.getCanvasKitRenderDiagnostics(pageIndex);
  }

  getCoordinateSystem(): CoordinateSystem {
    return this.coordinateSystem;
  }
}
