import { WasmBridge } from '@/core/wasm-bridge';
import { EventBus } from '@/core/event-bus';
import type { PageInfo } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { CanvasPool } from './canvas-pool';
import { PageRenderer, type PageRenderContext, type PageRenderResult } from './page-renderer';
import { ViewportManager } from './viewport-manager';
import { CoordinateSystem } from './coordinate-system';
import type { CanvasKitRenderDiagnostics } from './canvaskit-renderer';
import { clampRenderScale, type RenderBackend } from './render-backend';
import {
  RendererSession,
  type RendererSessionDiagnostics,
  type RendererSessionSelection,
} from './renderer-session';
import { applyGridOverlayBox, createGridClipCornerOverlay, createGridOverlay } from './grid-overlay';
import { getGridViewSettings } from './grid-settings';
import {
  calculateAnchoredScroll,
  CENTER_ZOOM_ANCHOR,
  normalizeZoomAnchor,
  type ZoomAnchor,
  type ZoomPageBox,
} from './zoom-anchor.ts';
import { SubsecondRevisionWatcher } from '@/core/subsecond-runtime';

const TEXT_EDIT_STATIC_LAYER_VERIFY_DELAY_MS = 800;
const AUTO_RENDERER_RESELECTION_DELAY_MS = 300;

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
  private subsecondRevisionWatcher: SubsecondRevisionWatcher;

  private scrollContent: HTMLElement;
  private pages: PageInfo[] = [];
  private currentVisiblePages: number[] = [];
  private unsubscribers: (() => void)[] = [];
  private pendingTextEditRefreshes = new Map<number, PageRenderContext>();
  private textEditRefreshRafId: number | null = null;
  private textEditStaticLayerVerifyTimers = new Map<number, ReturnType<typeof setTimeout>>();
  private pendingPrefetchPages = new Set<number>();
  private deferredPrefetchTask: DeferredPrefetchTask | null = null;
  private rendererSelectionEpoch = 0;
  private rendererFallbackScheduled = false;
  private activeRendererDecisionKey: string | null = null;
  private autoRendererReselectionTimer: ReturnType<typeof setTimeout> | null = null;
  private documentLoadPrepared = false;
  private layoutViewportSize = { width: 0, height: 0 };
  private disposed = false;

  constructor(
    private container: HTMLElement,
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private rendererSession: RendererSession,
  ) {
    this.virtualScroll = new VirtualScroll();
    this.canvasPool = new CanvasPool();
    this.pageRenderer = new PageRenderer(wasm);
    this.viewportManager = new ViewportManager(eventBus);
    this.coordinateSystem = new CoordinateSystem(this.virtualScroll);
    this.subsecondRevisionWatcher = new SubsecondRevisionWatcher(
      wasm,
      () => eventBus.emit('document-view-changed', 'subsecond-renderer'),
    );
    this.subsecondRevisionWatcher.start();

    this.scrollContent = container.querySelector('#scroll-content')!;
    this.viewportManager.attachTo(container);

    this.unsubscribers.push(
      eventBus.on('viewport-scroll', () => {
        if (!this.viewportManager.isZoomAnimating()) this.updateVisiblePages();
      }),
      eventBus.on('viewport-resize', () => this.onViewportResize()),
      eventBus.on('zoom-changed', (zoom, anchor) => {
        this.onZoomChanged(
          zoom as number,
          normalizeZoomAnchor(anchor as Partial<ZoomAnchor> | undefined),
        );
      }),
      eventBus.on('document-page-invalidated', (payload) => {
        void this.refreshInvalidatedPageForMutation(payload);
      }),
      eventBus.on('document-changed', () => {
        void this.refreshPagesForMutation();
      }),
      eventBus.on('document-view-changed', (source) => {
        if (source === 'subsecond-renderer') {
          this.refreshPages();
          return;
        }
        void this.refreshPagesForRevision();
      }),
      eventBus.on('grid-view-changed', () => this.refreshGridOverlays()),
    );
  }

  /** 문서 로드 후 호출 — 페이지 정보 수집 및 가상 스크롤 초기화 */
  async loadDocument(): Promise<void> {
    if (this.disposed) return;
    if (!this.documentLoadPrepared) this.prepareDocumentLoad();
    const epoch = this.rendererSelectionEpoch;
    this.documentLoadPrepared = false;
    if (this.disposed) return;
    const selection = await this.rendererSession.resolve(this.wasm);
    if (
      this.disposed
      || epoch !== this.rendererSelectionEpoch
      || !this.rendererSession.isCurrent(selection)
    ) return;
    this.applyRendererSelection(selection);

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
    this.viewportManager.setScrollLeft(
      this.virtualScroll.getCenteredScrollLeft(this.layoutViewportSize.width),
    );

    this.container.scrollTop = 0;
    this.updateVisiblePages();
    // 초기 replay가 예약한 document fallback을 load 완료 전에 확정한다.
    await Promise.resolve();

    console.log(`[CanvasView] ${this.pages.length}/${pageCount}페이지 로드, 총 높이: ${this.virtualScroll.getTotalHeight()}px`);
  }

  /** WASM 문서 교체 직후 호출하여 이전 문서의 renderer와 canvas를 동기적으로 분리한다. */
  prepareDocumentLoad(): void {
    if (this.disposed) return;
    this.rendererSelectionEpoch += 1;
    this.documentLoadPrepared = true;
    this.cancelAutoRendererReselection();
    this.rendererFallbackScheduled = false;
    this.rendererSession.beginDocument(this.wasm.documentDigest);
    // [#3315] 문서 범위 object URL 도 같은 경계에서 넘긴다 — 새 문서가 flow 그림을 조회하지
    // 않으면 옛 문서의 URL 을 거둘 기회가 다시 오지 않는다.
    this.pageRenderer.beginDocument();
    this.activeRendererDecisionKey = null;
    this.reset();
  }

  resetRendererDiagnostics(): void {
    this.pageRenderer.releaseAllPageDiagnostics();
  }

  private async refreshPagesForRevision(): Promise<void> {
    const selected = await this.selectNextDocumentRevision(false);
    if (!selected) return;
    this.refreshPages();
  }

  private async refreshPagesForMutation(): Promise<void> {
    const selected = await this.selectMutationRevision();
    if (!selected || !this.rendererSession.isCurrent(selected.selection)) return;
    this.refreshPages();
  }

  private async refreshInvalidatedPageForMutation(payload: unknown): Promise<void> {
    const selected = await this.selectMutationRevision();
    if (!selected || !this.rendererSession.isCurrent(selected.selection)) return;
    if (selected.backendChanged) {
      this.refreshPages();
      return;
    }
    this.refreshInvalidatedPage(payload);
  }

  private async selectMutationRevision(): Promise<{
    selection: RendererSessionSelection;
    backendChanged: boolean;
  } | null> {
    if (this.disposed) return null;
    const pinned = this.rendererSession.pinAutoMutationRevision();
    if (!pinned) return this.selectNextDocumentRevision();

    this.rendererSelectionEpoch += 1;
    const selected = {
      selection: pinned,
      backendChanged: this.applyRendererSelection(pinned),
    };
    this.scheduleAutoRendererReselection();
    return selected;
  }

  private scheduleAutoRendererReselection(): void {
    this.cancelAutoRendererReselection();
    this.autoRendererReselectionTimer = setTimeout(() => {
      this.autoRendererReselectionTimer = null;
      void this.selectNextDocumentRevision().then((selected) => {
        if (!selected || this.disposed) return;
        if (selected.backendChanged) this.refreshPages();
      });
    }, AUTO_RENDERER_RESELECTION_DELAY_MS);
  }

  private cancelAutoRendererReselection(): void {
    if (this.autoRendererReselectionTimer === null) return;
    clearTimeout(this.autoRendererReselectionTimer);
    this.autoRendererReselectionTimer = null;
  }

  private async selectNextDocumentRevision(resetResources = true): Promise<{
    selection: RendererSessionSelection;
    backendChanged: boolean;
  } | null> {
    if (this.disposed) return null;
    const epoch = ++this.rendererSelectionEpoch;
    this.rendererSession.invalidateDocument({ resetResources });
    await Promise.resolve();
    if (this.disposed || epoch !== this.rendererSelectionEpoch) return null;

    const selection = await this.rendererSession.resolve(this.wasm);
    if (
      this.disposed
      || epoch !== this.rendererSelectionEpoch
      || !this.rendererSession.isCurrent(selection)
    ) return null;
    return {
      selection,
      backendChanged: this.applyRendererSelection(selection),
    };
  }

  private applyRendererSelection(selection: RendererSessionSelection): boolean {
    const decisionChanged = this.activeRendererDecisionKey !== selection.diagnostics.decisionKey;
    const changed = this.pageRenderer.configure(
      selection.backend,
      selection.diagnostics.renderProfile,
      selection.canvaskitRenderer,
      selection.backend === 'canvas2d'
        && (
          selection.diagnostics.fallbackReason === 'canvaskitResourcePreparationFailed'
          || selection.diagnostics.fallbackReason === 'canvaskitRuntimeFailed'
        ),
    );
    if (decisionChanged && !changed) this.pageRenderer.invalidateDocumentRevision();
    this.activeRendererDecisionKey = selection.diagnostics.decisionKey;
    this.eventBus.emit('renderer-selection-changed', selection.diagnostics);
    return changed;
  }

  /** DEV baseline이 pool 소유권을 바꾸지 않고 현재 페이지를 즉시 다시 그린다. */
  rerenderPageForDiagnostics(pageIdx: number): boolean {
    const canvas = this.canvasPool.getCanvas(pageIdx);
    return canvas ? this.renderCanvas(pageIdx, canvas) : false;
  }

  /** 레이아웃을 재계산한다 (줌/리사이즈 공통) */
  private recalcLayout(): void {
    const zoom = this.viewportManager.getZoom();
    const viewport = this.viewportManager.getViewportSize();
    this.virtualScroll.setPageDimensions(this.pages, zoom, viewport.width);
    this.scrollContent.style.height = `${this.virtualScroll.getTotalHeight()}px`;
    this.scrollContent.style.width = `${this.virtualScroll.getTotalWidth()}px`;
    this.layoutViewportSize = viewport;

    // 그리드 모드 CSS 클래스 토글
    this.scrollContent.classList.toggle('grid-mode', this.virtualScroll.isGridMode());

    // [#3377] 좌표계가 바뀌어도 기렌더 캔버스·오버레이는 renderCanvas 밖에서 재배치되지
    // 않아, 첫 로딩 중 스크롤바 등장(clientWidth −15px) 같은 재계산 뒤에 신·구 좌표계가
    // 공존했다. 활성 페이지 전체에 현재 좌표를 재적용한다. 줌 애니메이션 중에는 preview
    // 변환(scale 동반)이 위치를 관리하므로 그 경로를 재사용한다.
    if (this.viewportManager.isZoomAnimating()) {
      this.updateRenderedPageZoomPreview();
    } else {
      this.repositionActivePages();
    }
  }

  /** 페이지 요소(캔버스·오버레이)에 현재 레이아웃 좌표를 적용하는 단일 관문. */
  private positionPageElement(element: HTMLElement, pageIdx: number): void {
    element.style.top = `${this.virtualScroll.getPageOffset(pageIdx)}px`;

    // 그리드/광폭 팬: 고정 left 좌표, 단일 열: CSS 중앙 정렬
    const pageLeft = this.virtualScroll.getPageLeft(pageIdx);
    if (pageLeft >= 0) {
      element.style.left = `${pageLeft}px`;
      element.style.transform = 'none';
    } else {
      element.style.left = '50%';
      element.style.transform = 'translateX(-50%)';
    }
    element.style.transformOrigin = '';
  }

  /** [#3377] 이미 렌더된 페이지의 캔버스와 오버레이를 현재 레이아웃 좌표로 재배치한다. */
  private repositionActivePages(): void {
    for (const pageIdx of this.canvasPool.activePages) {
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (canvas) this.positionPageElement(canvas, pageIdx);
      this.scrollContent.querySelectorAll<HTMLElement>(
        `[data-rhwp-overlay-page="${pageIdx}"], [data-rhwp-grid-page="${pageIdx}"]`,
      ).forEach((element) => this.positionPageElement(element, pageIdx));
    }
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
        this.pageRenderer.releasePageDiagnostics(pageIdx);
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
      // [#2560] 그리드 모드에서 getPageAtY 는 행의 마지막 쪽을 준다. 상태바가
      // 3열이면 첫 행에서 "3 / N" 으로 표시되고 1·2쪽은 현재 쪽이 될 수 없었다.
      const currentPage = this.virtualScroll.getRowFirstPageAtY(vpCenter);
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
    this.positionPageElement(canvas, pageIdx);

    // WASM이 Canvas 크기를 자동 설정한다 (물리 픽셀 = 페이지크기 × zoom × DPR)
    let renderResult: PageRenderResult = { needsTextEditStaticLayerVerification: false };
    let renderedCanvas = canvas;
    const rendererDecisionKey = this.activeRendererDecisionKey;
    try {
      renderResult = this.pageRenderer.renderPage(pageIdx, canvas, renderScale, zoom, dpr, renderContext);
      if (renderResult.renderedCanvas && renderResult.renderedCanvas !== canvas) {
        renderedCanvas = renderResult.renderedCanvas;
        this.canvasPool.replace(pageIdx, canvas, renderedCanvas);
      }
      const canvaskitDiagnostics = this.pageRenderer.getBackend() === 'canvaskit'
        ? this.pageRenderer.getCanvasKitRenderDiagnostics(pageIdx)
        : null;
      if (
        canvaskitDiagnostics
        && !canvaskitDiagnostics.passesRuntimeReadinessGate
        && rendererDecisionKey
        && this.rendererSession.isAutoRequest()
      ) {
        const details = [
          `blockers=${canvaskitDiagnostics.readinessBlockers.join(',') || 'unknown'}`,
          canvaskitDiagnostics.lastRenderError
            ? `error=${canvaskitDiagnostics.lastRenderError}`
            : null,
          canvaskitDiagnostics.lastUnexpectedUnsupportedOps.length > 0
            ? `unexpectedOps=${canvaskitDiagnostics.lastUnexpectedUnsupportedOps.join(',')}`
            : null,
        ].filter((detail): detail is string => detail !== null).join('; ');
        this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
        this.removeGridOverlay(pageIdx);
        this.scheduleCanvasKitFallback(
          new Error(`CanvasKit runtime readiness gate failed (${details})`),
          rendererDecisionKey,
          'runtime',
        );
        return false;
      }
    } catch (e) {
      console.error(`[CanvasView] 페이지 ${pageIdx} 렌더링 실패:`, e);
      this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
      this.removeGridOverlay(pageIdx);
      if (this.pageRenderer.getBackend() === 'canvaskit' && rendererDecisionKey) {
        this.scheduleCanvasKitFallback(e, rendererDecisionKey, 'resource');
      }
      return false;
    }

    // CSS 표시 크기 = 물리 픽셀 / DPR (= 페이지크기 × zoom)
    renderedCanvas.style.width = `${renderedCanvas.width / dpr}px`;
    renderedCanvas.style.height = `${renderedCanvas.height / dpr}px`;
    renderedCanvas.style.transformOrigin = '';
    renderedCanvas.dataset.rhwpRenderedZoom = String(zoom);
    this.renderGridOverlay(pageIdx, renderedCanvas);
    if (renderResult.needsTextEditStaticLayerVerification) {
      this.scheduleTextEditStaticLayerVerification(pageIdx);
    } else if (renderContext.reason !== 'text-edit') {
      this.cancelTextEditStaticLayerVerification(pageIdx);
    }
    return true;
  }

  private scheduleCanvasKitFallback(
    error: unknown,
    expectedDecisionKey: string,
    kind: 'resource' | 'runtime',
  ): void {
    if (this.rendererFallbackScheduled) return;
    const selection = kind === 'resource'
      ? this.rendererSession.fallbackFromResourceFailure(error, expectedDecisionKey)
      : this.rendererSession.fallbackFromRuntimeFailure(error, expectedDecisionKey);
    if (!selection) return;
    this.rendererFallbackScheduled = true;
    queueMicrotask(() => {
      this.rendererFallbackScheduled = false;
      if (this.disposed || !this.rendererSession.isCurrent(selection)) return;
      this.applyRendererSelection(selection);
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.releaseAllRenderedPages();
      this.pageRenderer.cancelAll();
      this.updateVisiblePages();
    });
  }

  /** 뷰포트 리사이즈 처리 */
  private onViewportResize(): void {
    const nextViewport = this.viewportManager.getViewportSize();
    if (this.pages.length === 0) {
      this.layoutViewportSize = nextViewport;
      this.updateVisiblePages();
      return;
    }

    const previousViewport = this.layoutViewportSize;
    const canPreserveCenter = previousViewport.width > 0 && previousViewport.height > 0;
    const scrollLeft = this.viewportManager.getScrollX();
    const scrollTop = this.viewportManager.getScrollY();
    const focusPage = canPreserveCenter
      ? this.virtualScroll.getPageAtPoint(
        scrollLeft + previousViewport.width / 2,
        scrollTop + previousViewport.height / 2,
      )
      : 0;
    const oldBox = canPreserveCenter
      ? this.getZoomPageBox(focusPage, previousViewport.width)
      : null;

    // 그리드 모드에서 열 수가 바뀔 수 있으므로 레이아웃 재계산
    const wasGrid = this.virtualScroll.isGridMode();
    this.recalcLayout();
    const isGrid = this.virtualScroll.isGridMode();

    if (oldBox) {
      const newBox = this.getZoomPageBox(focusPage, nextViewport.width);
      const nextScroll = calculateAnchoredScroll(
        oldBox,
        newBox,
        {
          width: previousViewport.width,
          height: previousViewport.height,
          scrollLeft,
          scrollTop,
        },
        CENTER_ZOOM_ANCHOR,
        nextViewport,
      );
      this.viewportManager.setScrollLeft(this.clampScrollLeft(nextScroll.scrollLeft));
      this.viewportManager.setScrollTop(nextScroll.scrollTop);
    } else {
      this.viewportManager.setScrollLeft(
        this.virtualScroll.getCenteredScrollLeft(nextViewport.width),
      );
    }

    if (wasGrid || isGrid) {
      // 그리드 관련 변경 시 전체 재렌더링
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.releaseAllRenderedPages();
      this.pageRenderer.cancelAll();
    }
    this.updateVisiblePages();
  }

  /**
   * [#3591] 앵커 계산 결과를 실제 스크롤 가능 범위로 가둔다.
   *
   * 팬 여백이 창 폭 100% 이던 시절에는 오버슈트를 여백이 흡수했지만, 여백이
   * 얇아지면 계산값이 범위를 넘을 수 있다(브라우저는 대입 시 클램프하므로 화면은
   * 안전하나, 이후 계산이 어긋난 값을 근거로 삼는 것을 막는다). 콘텐츠가 창보다
   * 좁으면 스크롤 여지가 없으므로 0 이다.
   */
  private clampScrollLeft(value: number): number {
    const viewportWidth = this.viewportManager.getViewportSize().width;
    const maxScroll = Math.max(0, this.virtualScroll.getTotalWidth() - viewportWidth);
    return Math.max(0, Math.min(value, maxScroll));
  }

  private getZoomPageBox(pageIdx: number, viewportWidth: number): ZoomPageBox {
    const layoutWidth = Math.max(viewportWidth, this.virtualScroll.getTotalWidth());
    return {
      left: this.virtualScroll.getPageLeftResolved(pageIdx, layoutWidth),
      top: this.virtualScroll.getPageOffset(pageIdx),
      width: this.virtualScroll.getPageWidth(pageIdx),
      height: this.virtualScroll.getPageHeight(pageIdx),
    };
  }

  /** 줌 변경 처리 */
  private onZoomChanged(zoom: number, anchor: ZoomAnchor): void {
    if (this.pages.length === 0) return;

    const scrollTop = this.viewportManager.getScrollY();
    const scrollLeft = this.viewportManager.getScrollX();
    const { width: vpWidth, height: vpHeight } = this.viewportManager.getViewportSize();
    const anchorDocumentX = scrollLeft + vpWidth * anchor.x;
    const anchorDocumentY = scrollTop + vpHeight * anchor.y;
    const focusPage = this.virtualScroll.getPageAtPoint(anchorDocumentX, anchorDocumentY);
    const oldBox = this.getZoomPageBox(focusPage, vpWidth);

    this.recalcLayout();

    const newBox = this.getZoomPageBox(focusPage, vpWidth);
    const nextScroll = calculateAnchoredScroll(
      oldBox,
      newBox,
      {
        width: vpWidth,
        height: vpHeight,
        scrollLeft,
        scrollTop,
      },
      anchor,
    );
    this.viewportManager.setScrollLeft(this.clampScrollLeft(nextScroll.scrollLeft));
    this.viewportManager.setScrollTop(nextScroll.scrollTop);

    this.eventBus.emit('zoom-level-display', zoom);

    if (this.viewportManager.isZoomAnimating()) {
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.cancelPendingPrefetch();
      this.updateRenderedPageZoomPreview();
      return;
    }

    // 모든 Canvas 재렌더링
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.releaseAllRenderedPages();
    this.pageRenderer.cancelAll();
    this.updateVisiblePages();
  }

  private updateRenderedPageZoomPreview(): void {
    const zoom = this.viewportManager.getZoom();
    for (const pageIdx of this.canvasPool.activePages) {
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (!canvas) continue;
      const renderedZoom = Number(canvas.dataset.rhwpRenderedZoom);
      const scale = Number.isFinite(renderedZoom) && renderedZoom > 0
        ? zoom / renderedZoom
        : 1;
      this.applyZoomPreviewBox(canvas, pageIdx, scale);
      this.scrollContent.querySelectorAll<HTMLElement>(
        `[data-rhwp-overlay-page="${pageIdx}"], [data-rhwp-grid-page="${pageIdx}"]`,
      ).forEach((element) => this.applyZoomPreviewBox(element, pageIdx, scale));
    }
  }

  private applyZoomPreviewBox(element: HTMLElement, pageIdx: number, scale: number): void {
    element.style.top = `${this.virtualScroll.getPageOffset(pageIdx)}px`;
    const pageLeft = this.virtualScroll.getPageLeft(pageIdx);
    if (pageLeft >= 0) {
      element.style.left = `${pageLeft}px`;
      element.style.transform = `scale(${scale})`;
      element.style.transformOrigin = 'top left';
    } else {
      element.style.left = '50%';
      element.style.transform = `translateX(-50%) scale(${scale})`;
      element.style.transformOrigin = 'top center';
    }
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
    if (this.disposed) return;
    this.disposed = true;
    this.subsecondRevisionWatcher.stop();
    this.rendererSelectionEpoch += 1;
    this.documentLoadPrepared = false;
    this.cancelAutoRendererReselection();
    this.reset();
    this.pageRenderer.dispose();
    this.rendererSession.dispose();
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

  getRendererSessionDiagnostics(): RendererSessionDiagnostics | null {
    return this.rendererSession.diagnostics();
  }

  getCanvasKitRenderDiagnostics(pageIndex: number): CanvasKitRenderDiagnostics | null {
    return this.pageRenderer.getCanvasKitRenderDiagnostics(pageIndex);
  }

  getCurrentCanvasKitRenderDiagnostics(): CanvasKitRenderDiagnostics | null {
    return this.pageRenderer.getCurrentCanvasKitRenderDiagnostics();
  }

  getCoordinateSystem(): CoordinateSystem {
    return this.coordinateSystem;
  }
}
