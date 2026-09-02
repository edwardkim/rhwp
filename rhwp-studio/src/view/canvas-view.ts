import { WasmBridge } from '@/core/wasm-bridge';
import { EventBus } from '@/core/event-bus';
import type { PageInfo } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { CanvasPool } from './canvas-pool';
import { PageRenderer, type PageRenderContext, type PageRenderResult } from './page-renderer';
import { ViewportManager } from './viewport-manager';
import { CoordinateSystem } from './coordinate-system';
import { scrollByPageStep, type PageScrollDirection } from './page-scroll';
import type { CanvasKitRenderDiagnostics } from './canvaskit-renderer';
import type { FontDecisionTraceRecordV1 } from '@/core/font-decision-trace';
import { clampRenderScale, type RenderBackend } from './render-backend';
import {
  RendererSession,
  type RendererSessionDiagnostics,
  type RendererSessionSelection,
} from './renderer-session';
import { applyGridOverlayBox, createGridClipCornerOverlay, createGridOverlay } from './grid-overlay';
import { getGridViewSettings } from './grid-settings';
import { userSettings } from '@/core/user-settings';
import {
  pageArrangementsEqual,
  type PageArrangement,
} from './page-arrangement.ts';
import {
  resolvePageViewSettings,
  type PageMovementSettings,
} from './page-movement.ts';
import { resolvePageViewSettingsChange } from './page-view-settings-change.ts';
import {
  calculateAnchoredScroll,
  CENTER_ZOOM_ANCHOR,
  normalizeZoomAnchor,
  type ZoomAnchor,
  type ZoomPageBox,
} from './zoom-anchor.ts';
import {
  resolveActivePage,
  type ActivePageSnapshot,
} from './active-page.ts';
import {
  headerFooterApplyToLabel,
  parseHeaderFooterModeChanged,
  type HeaderFooterModeState,
} from '@/engine/header-footer-mode.ts';
import {
  headerFooterClipPath,
  resolveHeaderFooterBadgeMetrics,
  resolveHeaderFooterBandBox,
} from './header-footer-edit-overlay.ts';
import {
  drawPageMarginGuideCorners,
  type PageMarginGuideEdges,
} from './page-margin-guides.ts';
import {
  DEFAULT_CANVAS2D_LAYER_COUNT,
  DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET,
  planRenderSurfaceBudget,
  type RenderSurfaceBudgetPlan,
  type RenderSurfaceDecision,
} from './render-surface-budget.ts';
import {
  PageSurfaceLru,
  type PageSurfaceCacheEntry,
  type PageSurfaceLruSnapshot,
} from './page-surface-lru.ts';
import {
  createBrowserPageRenderSchedulerHost,
  PageRenderScheduler,
  type PageRenderSchedulerSnapshot,
  type PageRenderWork,
} from './page-render-scheduler.ts';

/** 문서 교체 중 보여줄 빈 쪽 기본 크기(A4, zoom 1 기준 CSS px). 이전 문서 쪽 크기를 모를 때만 쓴다. */
const BLANK_PAGE_FALLBACK_SIZE = { width: 794, height: 1123 };

const TEXT_EDIT_STATIC_LAYER_VERIFY_DELAY_MS = 800;
const AUTO_RENDERER_RESELECTION_DELAY_MS = 300;

type VisibilityUpdateReason =
  | 'scroll'
  | 'initial'
  | 'zoom-settled'
  | 'resize'
  | 'mutation'
  | 'strict';

interface ScrollMotion {
  direction: -1 | 0 | 1;
  speed: number;
}

interface PageSurfaceBundle extends PageSurfaceCacheEntry {
  mainCanvas: HTMLCanvasElement;
  elements: HTMLElement[];
  leaseId: number;
}

export class CanvasView {
  private virtualScroll: VirtualScroll;
  private canvasPool: CanvasPool;
  private pageRenderer: PageRenderer;
  private viewportManager: ViewportManager;
  private applyingPageViewSettingsTransaction = false;
  private coordinateSystem: CoordinateSystem;

  private scrollContent: HTMLElement;
  private pageArrangement: PageArrangement;
  private pageMovement: PageMovementSettings;
  private pages: PageInfo[] = [];
  private currentVisiblePages: number[] = [];
  private currentRetainedPages: number[] = [];
  private editingPageIndex: number | null = null;
  private headerFooterEditState: HeaderFooterModeState | null = null;
  private activePageSnapshot: ActivePageSnapshot | null = null;
  private renderSurfacePlan: RenderSurfaceBudgetPlan | null = null;
  private renderSurfaceDecisions = new Map<number, RenderSurfaceDecision>();
  private pageSurfaceLru: PageSurfaceLru<PageSurfaceBundle>;
  private pageRenderScheduler: PageRenderScheduler;
  private nextSurfaceLeaseId = 0;
  private renderWorkGeneration = 0;
  private lastScrollSample: { x: number; y: number; at: number } | null = null;
  private previousEffectiveDpr = new Map<number, number>();
  private renderSurfaceEnvironmentKey: string | null = null;
  private unsubscribers: (() => void)[] = [];
  private pendingTextEditRefreshes = new Map<number, PageRenderContext>();
  private textEditRefreshRafId: number | null = null;
  private textEditStaticLayerVerifyTimers = new Map<number, ReturnType<typeof setTimeout>>();
  private rendererSelectionEpoch = 0;
  private rendererFallbackScheduled = false;
  private activeRendererDecisionKey: string | null = null;
  private autoRendererReselectionTimer: ReturnType<typeof setTimeout> | null = null;
  private documentLoadPrepared = false;
  private layoutViewportSize = { width: 0, height: 0 };
  private blankPagePlaceholder: HTMLElement | null = null;
  private lastPageSize: { width: number; height: number } | null = null;
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
    this.pageSurfaceLru = new PageSurfaceLru(
      bundle => this.disposeCachedPageSurface(bundle),
      DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET,
    );
    this.pageRenderScheduler = new PageRenderScheduler(
      createBrowserPageRenderSchedulerHost(window),
    );
    this.viewportManager = new ViewportManager(eventBus);
    this.coordinateSystem = new CoordinateSystem(this.virtualScroll);
    const viewSettings = userSettings.getViewSettings();
    const pageView = resolvePageViewSettings(
      viewSettings.pageArrangement,
      viewSettings.pageMovement,
    );
    this.pageArrangement = pageView.arrangement;
    this.pageMovement = pageView.movement;
    this.viewportManager.setPageMovement(this.pageMovement);

    this.scrollContent = container.querySelector('#scroll-content')!;
    this.viewportManager.attachTo(container);

    this.unsubscribers.push(
      eventBus.on('viewport-scroll', () => {
        if (!this.viewportManager.isZoomAnimating()) this.updateVisiblePages('scroll');
      }),
      eventBus.on('viewport-resize', () => this.onViewportResize()),
      eventBus.on('zoom-changed', (zoom, anchor) => {
        if (this.applyingPageViewSettingsTransaction) return;
        this.onZoomChanged(
          zoom as number,
          normalizeZoomAnchor(anchor as Partial<ZoomAnchor> | undefined),
        );
      }),
      eventBus.on('page-view-settings-changed', (payload) => {
        this.setPageViewSettings(payload);
      }),
      eventBus.on('headerFooterModeChanged', (payload) => {
        this.handleHeaderFooterModeChanged(payload);
      }),
      eventBus.on('document-page-invalidated', (payload) => {
        void this.refreshInvalidatedPageForMutation(payload);
      }),
      eventBus.on('document-changed', (reason) => {
        // document-agent는 host 응답 전 strict render를 이미 완료한다. 나머지 observer에는
        // commit event를 전달하되 CanvasView만 같은 revision을 두 번 그리지 않는다.
        if (reason === 'document-agent-rendered') return;
        void this.refreshPagesForMutation();
      }),
      eventBus.on('document-view-changed', () => {
        void this.refreshPagesForRevision();
      }),
      eventBus.on('grid-view-changed', () => this.refreshGridOverlays()),
      eventBus.on('cursor-rect-updated', (payload) => {
        const pageIndex = this.pageIndexFromPayload(payload);
        if (pageIndex !== null) this.setEditingPageIndex(pageIndex);
      }),
      eventBus.on('editing-page-changed', (payload) => {
        this.setEditingPageIndex(this.pageIndexFromPayload(payload));
      }),
      eventBus.on('picture-object-selection-changed', (selected) => {
        if (selected === false) this.setEditingPageIndex(null);
      }),
      eventBus.on('table-object-selection-changed', (selected) => {
        if (selected === false) this.setEditingPageIndex(null);
      }),
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
    this.lastPageSize = { width: this.pages[0].width, height: this.pages[0].height };
    this.updateVisiblePages('initial');
    this.clearBlankPagePlaceholder();
    // 초기 replay가 예약한 document fallback을 load 완료 전에 확정한다.
    await Promise.resolve();

    console.log(`[CanvasView] ${this.pages.length}/${pageCount}페이지 로드, 총 높이: ${this.virtualScroll.getTotalHeight()}px`);

    // 문서 화면이 새로 섰다 — 쪽 정보를 스스로 그리는 바깥 소비자(눈금자)에게 알린다.
    // 지금까지 눈금자는 캐럿·스크롤·확대 이벤트에 얹혀 갱신됐다. 그 셋은 값이 그대로면
    // 오지 않는다(문단 여백이 같은 문서를 잇달아 열기, 이미 맨 위인 문서의 scrollTop=0,
    // 배율 그대로) — 그때 눈금자는 빈 쪽 단계에서 그린 눈금 없는 회색 띠로 남았다.
    this.eventBus.emit('document-view-loaded');
  }

  /**
   * PgUp/PgDn — 화면을 쪽 단위로 옮긴다. 문서는 떠 있지만 편집기가 활성이 아닐 때
   * (InputHandler 가 키를 소유하지 않는 상태) 전역 폴백이 쓰는 진입점이다.
   * 편집기가 활성이면 캐럿까지 함께 옮기는 `InputHandler.scrollByPageKey` 가 처리한다.
   */
  scrollByPage(direction: PageScrollDirection): boolean {
    if (this.disposed) return false;
    return scrollByPageStep(this.virtualScroll, this.viewportManager, direction).moved;
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
    this.showBlankPagePlaceholder();
  }

  /**
   * 문서 열기를 시작할 때 현재 뷰를 비우고 빈 쪽 상태로 만든다. 파싱이 끝날 때까지
   * 이전 문서를 붙잡고 있다가 한 번에 갈아치우면 화면이 튀어 보인다.
   */
  showBlankPage(): void {
    if (this.disposed) return;
    this.reset();
    this.showBlankPagePlaceholder();
  }

  /**
   * 새 문서의 첫 쪽이 그려질 때까지 빈 흰 쪽을 대신 놓는다. 자리표시자가 없으면 회색 작업
   * 영역이 그대로 드러나 문서를 열 때마다 화면이 깜빡이는 것처럼 보인다.
   */
  private showBlankPagePlaceholder(): void {
    if (this.disposed) return;
    const zoom = this.viewportManager.getZoom();
    const size = this.lastPageSize ?? BLANK_PAGE_FALLBACK_SIZE;
    const gap = this.virtualScroll.getPageGap();
    const width = size.width * zoom;
    const height = size.height * zoom;

    const placeholder = document.createElement('div');
    placeholder.className = 'page-placeholder';
    placeholder.setAttribute('aria-hidden', 'true');
    placeholder.style.top = `${gap}px`;
    placeholder.style.width = `${width}px`;
    placeholder.style.height = `${height}px`;

    // 자리표시자만 있는 동안에도 스크롤 영역이 쪽 하나 크기를 유지해야 가운데 정렬이 흔들리지 않는다.
    this.scrollContent.style.width = `${width + 40}px`;
    this.scrollContent.style.height = `${height + gap * 2}px`;
    this.scrollContent.appendChild(placeholder);
    this.blankPagePlaceholder = placeholder;
  }

  private clearBlankPagePlaceholder(): void {
    this.blankPagePlaceholder?.remove();
    this.blankPagePlaceholder = null;
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
    // InputHandler의 mutation 직후 caret 갱신보다 VirtualScroll 재계산이 늦다.
    // 새 page offset을 소비할 수 있는 완료 경계를 별도 이벤트로 알린다.
    this.eventBus.emit('document-layout-refreshed', { source: 'mutation' });
  }

  /** document-agent RPC 응답 전에 현재 visible page가 실제 canvas로 그려졌는지 확인한다. */
  async refreshDocumentAgentMutation(): Promise<void> {
    const selected = await this.selectMutationRevision();
    if (!selected || !this.rendererSession.isCurrent(selected.selection)) {
      throw new Error('document-agent renderer revision을 선택하지 못했습니다.');
    }
    this.refreshPages();
    const scrollY = this.viewportManager.getScrollY();
    const scrollX = this.viewportManager.getScrollX();
    const viewport = this.viewportManager.getViewportSize();
    const visiblePages = this.virtualScroll.getVisiblePages(
      scrollY,
      viewport.height,
      scrollX,
      viewport.width,
    );
    const failed = visiblePages.filter(pageIndex => !this.canvasPool.has(pageIndex));
    if (visiblePages.length === 0 || failed.length > 0) {
      throw new Error(`document-agent visible page render 실패: ${failed.join(',') || 'none'}`);
    }
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
    if (decisionChanged) this.pageSurfaceLru?.clear();
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
    this.virtualScroll.setPageDimensions(
      this.pages,
      zoom,
      viewport.width,
      this.pageArrangement,
      this.pageMovement.direction,
      viewport.height,
    );
    this.scrollContent.style.height = `${this.virtualScroll.getTotalHeight()}px`;
    this.scrollContent.style.width = `${this.virtualScroll.getTotalWidth()}px`;
    this.layoutViewportSize = viewport;

    // 그리드 모드 CSS 클래스 토글
    this.scrollContent.classList.toggle('grid-mode', this.virtualScroll.isGridMode());
    this.scrollContent.classList.toggle(
      'horizontal-page-movement',
      this.virtualScroll.isHorizontalMode(),
    );

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
        `[data-rhwp-overlay-page="${pageIdx}"], [data-rhwp-grid-page="${pageIdx}"], [data-rhwp-hf-edit-page="${pageIdx}"]`,
      ).forEach((element) => this.positionPageElement(element, pageIdx));
    }
  }

  /** 호출 이유별 동기 계약을 보존하며 보이는 페이지를 갱신한다. */
  private updateVisiblePages(reason: VisibilityUpdateReason = 'strict'): void {
    const scrollY = this.viewportManager.getScrollY();
    const scrollX = this.viewportManager.getScrollX();
    const { width: vpWidth, height: vpHeight } = this.viewportManager.getViewportSize();
    const isScroll = reason === 'scroll';
    if (!isScroll) {
      this.pageRenderScheduler.cancelAll();
      this.lastScrollSample = null;
    }
    const generation = ++this.renderWorkGeneration;
    const motion = isScroll
      ? this.sampleScrollMotion(scrollX, scrollY)
      : { direction: 0 as const, speed: 0 };

    const visibility = this.virtualScroll.getVisibilitySnapshot(
      scrollY,
      vpHeight,
      scrollX,
      vpWidth,
    );
    const visiblePages = [...visibility.visiblePages];
    const prefetchPages = [...visibility.prefetchPages];
    const visibleSet = new Set(visiblePages);

    // 벗어난 완성 surface는 잠시 분리한다. 새 working set 예약을 확정한 뒤 남는 headroom에만 넣는다.
    const prefetchSet = new Set(prefetchPages);
    const detachedBundles: PageSurfaceBundle[] = [];
    for (const pageIdx of this.canvasPool.activePages) {
      if (!prefetchSet.has(pageIdx)) {
        this.cancelPendingTextEditRefresh(pageIdx);
        this.cancelTextEditStaticLayerVerification(pageIdx);
        const bundle = this.detachCompletedPageSurface(pageIdx);
        if (bundle) detachedBundles.push(bundle);
        else this.discardActivePageSurface(pageIdx);
      }
    }

    // 새 가시·보존 집합을 먼저 확정해야 새로 그리는 쪽도 현재 예산으로 판정된다.
    this.currentVisiblePages = visiblePages;
    this.currentRetainedPages = prefetchPages;
    this.updateActivePageSnapshot();
    this.refreshRenderSurfacePlan(!isScroll);
    this.reconcilePageSurfaceBudget();
    for (const bundle of detachedBundles) this.pageSurfaceLru.put(bundle);
    this.reconcilePageSurfaceBudget();

    if (!isScroll) {
      // 초기/줌/resize/편집/strict는 기존처럼 visible 전체를 응답 전에 동기 렌더한다.
      for (const pageIdx of visiblePages) {
        if (!this.canvasPool.has(pageIdx)) this.renderPage(pageIdx);
      }
    } else {
      // exact cache hit는 raster queue를 거치지 않고 현재 retained working set에 즉시
      // 재부착한다. LRU 안에만 남겨 두면 queue는 비었어도 retained-complete가 될 수 없다.
      for (const pageIdx of prefetchPages) this.restoreCachedSurface(pageIdx);
    }

    const centerPage = visiblePages.length > 0
      ? this.virtualScroll.getPageAtPoint(
          scrollX + vpWidth / 2,
          scrollY + vpHeight / 2,
        )
      : null;
    const visibleWork = isScroll
      ? this.buildVisibleRenderWork(visiblePages, centerPage, generation)
      : [];
    const adjacentPages = prefetchPages.filter((pageIdx) => !visibleSet.has(pageIdx));
    const prefetchWork = this.buildPrefetchRenderWork(
      adjacentPages,
      visiblePages,
      motion,
      generation,
    );
    this.pageRenderScheduler.setDesiredWork(
      generation,
      visibleWork,
      prefetchWork,
      isScroll,
    );
    this.renderHeaderFooterEditOverlays();
  }

  private sampleScrollMotion(scrollX: number, scrollY: number): ScrollMotion {
    const at = performance.now();
    const previous = this.lastScrollSample;
    this.lastScrollSample = { x: scrollX, y: scrollY, at };
    if (!previous) return { direction: 0, speed: 0 };
    const delta = this.pageMovement.direction === 'horizontal'
      ? scrollX - previous.x
      : scrollY - previous.y;
    const elapsed = Math.max(1, at - previous.at);
    return {
      direction: delta === 0 ? 0 : delta > 0 ? 1 : -1,
      speed: Math.abs(delta) / elapsed,
    };
  }

  private restoreCachedSurface(pageIdx: number): void {
    if (this.canvasPool.has(pageIdx)) return;
    const descriptor = this.pageSurfaceDescriptor(pageIdx);
    if (!descriptor || !this.pageSurfaceLru.hasLookup(descriptor.lookupKey)) return;
    this.renderPage(pageIdx);
  }

  private buildVisibleRenderWork(
    visiblePages: readonly number[],
    centerPage: number | null,
    generation: number,
  ): PageRenderWork[] {
    const focusedVisible = this.editingPageIndex !== null
      && visiblePages.includes(this.editingPageIndex)
      ? this.editingPageIndex
      : null;
    return visiblePages.flatMap((pageIdx) => {
      const descriptor = this.pageSurfaceDescriptor(pageIdx);
      if (!descriptor) return [];
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (canvas?.dataset.rhwpSurfaceCacheLookupKey === descriptor.lookupKey) return [];
      const focusPriority = pageIdx === focusedVisible
        ? 0
        : pageIdx === centerPage
          ? 1
          : 2 + Math.abs(pageIdx - (centerPage ?? pageIdx)) / 1000;
      return [this.createPageRenderWork(
        pageIdx,
        descriptor.lookupKey,
        focusPriority,
        'visible',
        generation,
      )];
    });
  }

  private buildPrefetchRenderWork(
    adjacentPages: readonly number[],
    visiblePages: readonly number[],
    motion: ScrollMotion,
    generation: number,
  ): PageRenderWork[] {
    const firstVisible = visiblePages[0] ?? 0;
    const lastVisible = visiblePages[visiblePages.length - 1] ?? firstVisible;
    const ordered = [...adjacentPages].sort((a, b) => {
      if (motion.direction > 0) {
        const aAhead = a > lastVisible ? 0 : 1;
        const bAhead = b > lastVisible ? 0 : 1;
        if (aAhead !== bAhead) return aAhead - bAhead;
        return a - b;
      }
      if (motion.direction < 0) {
        const aAhead = a < firstVisible ? 0 : 1;
        const bAhead = b < firstVisible ? 0 : 1;
        if (aAhead !== bAhead) return aAhead - bAhead;
        return b - a;
      }
      const center = (firstVisible + lastVisible) / 2;
      return Math.abs(a - center) - Math.abs(b - center);
    });

    return ordered.flatMap((pageIdx, order) => {
      const descriptor = this.pageSurfaceDescriptor(pageIdx);
      if (!descriptor) return [];
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (
        canvas?.dataset.rhwpSurfaceCacheLookupKey === descriptor.lookupKey
        || (!canvas && this.pageSurfaceLru.hasLookup(descriptor.lookupKey))
      ) return [];
      // 속도는 overscan을 늘리지 않고 같은 기존 ±1행 후보 중 진행 반대편의 priority만
      // 낮춘다. 빠르게 이동할수록 다음 행을 먼저 채우되 후보 집합 자체는 바뀌지 않는다.
      const isBehind = motion.direction > 0
        ? pageIdx < firstVisible
        : motion.direction < 0
          ? pageIdx > lastVisible
          : false;
      const reverseDirectionPenalty = isBehind
        ? 1 + Math.min(4, motion.speed / 2)
        : 0;
      return [this.createPageRenderWork(
        pageIdx,
        descriptor.lookupKey,
        order + reverseDirectionPenalty,
        'prefetch',
        generation,
      )];
    });
  }

  private createPageRenderWork(
    pageIdx: number,
    rasterKey: string,
    priority: number,
    workClass: 'visible' | 'prefetch',
    generation: number,
  ): PageRenderWork {
    return {
      pageIndex: pageIdx,
      priority,
      rasterKey,
      isValid: () => {
        if (this.disposed || generation !== this.renderWorkGeneration) return false;
        const desired = workClass === 'visible'
          ? this.currentVisiblePages.includes(pageIdx)
          : this.currentRetainedPages.includes(pageIdx);
        return desired && this.pageSurfaceDescriptor(pageIdx)?.lookupKey === rasterKey;
      },
      run: () => this.renderScheduledPage(pageIdx, rasterKey),
    };
  }

  private renderScheduledPage(pageIdx: number, rasterKey: string): void {
    if (this.pageSurfaceDescriptor(pageIdx)?.lookupKey !== rasterKey) return;
    const canvas = this.canvasPool.getCanvas(pageIdx);
    if (canvas) {
      if (canvas.dataset.rhwpSurfaceCacheLookupKey === rasterKey) {
        this.applySurfaceDecisionDiagnostics(pageIdx, canvas);
      } else if (!this.renderCanvas(pageIdx, canvas)) {
        this.discardActivePageSurface(pageIdx);
      }
    } else {
      this.renderPage(pageIdx);
    }
    if (this.headerFooterEditState && this.currentVisiblePages.includes(pageIdx)) {
      this.renderHeaderFooterEditOverlays();
    }
  }

  /** HF 타겟을 구역 첫 페이지에 가상 투영하고 실제 적용 쪽을 함께 표시한다. */
  private handleHeaderFooterModeChanged(payload: unknown): void {
    // 여백 안내 꺾쇠는 main raster에 포함된다. mode가 바뀌면 offscreen bundle도 보수적으로 버린다.
    this.pageSurfaceLru.clear();
    const state = parseHeaderFooterModeChanged(payload);
    if (state === 'none') {
      this.headerFooterEditState = null;
      this.setPageMarginGuideEdges('both');
      this.removeHeaderFooterEditOverlays();
      return;
    }

    this.headerFooterEditState = state;
    // HF와 본문이 공유하는 경계에서는 본문 기준 꺾쇠 방향이 반대다.
    // 머리말은 본문 위쪽, 꼬리말은 본문 아래쪽 꺾쇠를 잠시 숨긴다.
    this.setPageMarginGuideEdges(state.mode === 'header' ? 'bottom' : 'top');
    if (!this.currentVisiblePages.includes(state.previewPage)) {
      const pageTop = this.virtualScroll.getPageOffset(state.previewPage);
      this.viewportManager.setScrollTop(Math.max(0, pageTop - this.virtualScroll.getPageGap()));
      this.updateVisiblePages('strict');
      return;
    }
    this.renderHeaderFooterEditOverlays();
  }

  private setPageMarginGuideEdges(edges: PageMarginGuideEdges): void {
    if (!this.pageRenderer.setPageMarginGuideEdges(edges)) return;
    for (const pageIdx of Array.from(this.canvasPool.activePages)) {
      const canvas = this.canvasPool.getCanvas(pageIdx);
      if (!canvas) continue;
      if (!this.renderCanvas(pageIdx, canvas)) this.canvasPool.release(pageIdx);
    }
  }

  private renderHeaderFooterEditOverlays(force = false): void {
    const state = this.headerFooterEditState;
    if (!state || this.pages.length === 0) {
      this.removeHeaderFooterEditOverlays();
      return;
    }

    const desiredPages = new Set<number>();
    for (const pageIdx of this.canvasPool.activePages) {
      const page = this.pages[pageIdx];
      if (!page) continue;
      const isPreview = pageIdx === state.previewPage;
      let isAppliedPage = false;
      try {
        const target = this.wasm.getHeaderFooterEditTarget(pageIdx, state.mode === 'header');
        isAppliedPage = target.sectionIndex === state.sectionIdx && target.applyTo === state.applyTo;
      } catch {
        // 현재 렌더된 HF가 없는 쪽은 연관 표시 대상에서 뺀다.
      }
      if (!isPreview && !isAppliedPage) continue;
      desiredPages.add(pageIdx);

      const zoom = this.viewportManager.getZoom();
      const overlayKey = [
        state.mode,
        state.sectionIdx,
        state.applyTo,
        isPreview ? 'representative' : 'related',
        zoom,
      ].join(':');
      const selector = `[data-rhwp-hf-edit-page="${pageIdx}"]`;
      const existing = this.scrollContent.querySelector<HTMLElement>(selector);
      if (!force && existing?.dataset.hfOverlayKey === overlayKey) {
        this.positionPageElement(existing, pageIdx);
        continue;
      }
      existing?.remove();

      const layer = document.createElement('div');
      layer.className = `hf-edit-surface-layer ${isPreview ? 'is-representative' : 'is-related'}`;
      layer.dataset.rhwpHfEditPage = String(pageIdx);
      layer.dataset.hfApplyTo = String(state.applyTo);
      layer.dataset.hfMode = state.mode;
      layer.dataset.hfOverlayKey = overlayKey;
      layer.setAttribute('aria-hidden', 'true');
      layer.style.width = `${page.width * zoom}px`;
      layer.style.height = `${page.height * zoom}px`;
      this.positionPageElement(layer, pageIdx);

      const band = resolveHeaderFooterBandBox(page, state.mode === 'header');
      const rawDpr = window.devicePixelRatio || 1;
      const renderScale = clampRenderScale(page, zoom * rawDpr);
      const dpr = renderScale / (zoom > 0 ? zoom : 1);
      if (isPreview) {
        const previewCanvas = document.createElement('canvas');
        previewCanvas.className = 'hf-edit-preview-canvas';
        try {
          this.wasm.renderHeaderFooterEditPreviewToCanvas(
            pageIdx,
            state.sectionIdx,
            state.mode === 'header',
            state.applyTo,
            previewCanvas,
            renderScale,
          );
          previewCanvas.style.width = `${previewCanvas.width / dpr}px`;
          previewCanvas.style.height = `${previewCanvas.height / dpr}px`;
          previewCanvas.style.clipPath = headerFooterClipPath(page, band, zoom);
          layer.appendChild(previewCanvas);
        } catch (error) {
          console.error('[CanvasView] HF 대표 편집 preview 렌더링 실패:', error);
        }
      }

      // 일반 페이지 본문 여백과 같은 Canvas 꺾쇠 렌더러를 그대로 사용한다.
      // 별도 CSS border를 쓰면 색·두께·확대 배율이 기존 페이지 가이드와 달라진다.
      const guideCanvas = document.createElement('canvas');
      guideCanvas.className = 'hf-edit-guide-canvas';
      guideCanvas.width = Math.max(1, Math.round(page.width * renderScale));
      guideCanvas.height = Math.max(1, Math.round(page.height * renderScale));
      guideCanvas.style.width = `${guideCanvas.width / dpr}px`;
      guideCanvas.style.height = `${guideCanvas.height / dpr}px`;
      // 공유 경계의 반대 방향 본문 꺾쇠는 숨겼으므로 HF 기준 네 모서리를 그린다.
      drawPageMarginGuideCorners(band, guideCanvas, renderScale, 'both', undefined, zoom);
      layer.appendChild(guideCanvas);

      const region = document.createElement('div');
      region.className = `hf-edit-region ${isPreview ? 'is-representative' : 'is-related'}`;
      region.style.left = `${band.x * zoom}px`;
      region.style.top = `${band.y * zoom}px`;
      region.style.width = `${band.width * zoom}px`;
      region.style.height = `${band.height * zoom}px`;
      layer.appendChild(region);

      if (isPreview) {
        const kind = state.mode === 'header' ? '머리말' : '꼬리말';
        const badgeMetrics = resolveHeaderFooterBadgeMetrics(zoom);
        const badge = document.createElement('span');
        badge.className = 'hf-edit-badge';
        badge.textContent = `${kind}(${headerFooterApplyToLabel(state.applyTo)})`;
        badge.style.left = `${band.x * zoom}px`;
        badge.style.top = `${band.y * zoom}px`;
        badge.style.fontSize = `${badgeMetrics.fontSizePx}px`;
        badge.style.setProperty('--hf-edit-badge-gap', `${badgeMetrics.gapPx}px`);
        layer.appendChild(badge);
      }

      this.scrollContent.appendChild(layer);
    }
    this.scrollContent.querySelectorAll<HTMLElement>('[data-rhwp-hf-edit-page]')
      .forEach((element) => {
        const pageIdx = Number(element.dataset.rhwpHfEditPage);
        if (!desiredPages.has(pageIdx)) element.remove();
      });
  }

  private removeHeaderFooterEditOverlays(): void {
    this.scrollContent.querySelectorAll('[data-rhwp-hf-edit-page]').forEach((element) => element.remove());
  }

  private pageIndexFromPayload(payload: unknown): number | null {
    const value = typeof payload === 'object' && payload !== null && 'pageIndex' in payload
      ? (payload as { pageIndex?: unknown }).pageIndex
      : payload;
    if (typeof value !== 'number' || !Number.isInteger(value) || value < 0) return null;
    return value;
  }

  private setEditingPageIndex(pageIndex: number | null): void {
    if (this.editingPageIndex === pageIndex) return;
    this.editingPageIndex = pageIndex;
    this.updateActivePageSnapshot();
    // 눈금자는 순수 스크롤의 viewport fallback이 아니라 마지막 편집 focus를 따른다.
    // current-page-changed와 렌더 가시성은 위 active snapshot 계약을 계속 사용한다.
    this.eventBus.emit('focused-page-changed', pageIndex);
    this.refreshRenderSurfacePlan(true);
  }

  /** 캐럿·개체 선택과 스크롤이 공유하는 활성 페이지 판정·발행 관문. */
  private updateActivePageSnapshot(): void {
    const viewport = this.viewportManager.getViewportSize();
    const viewportCenterX = this.viewportManager.getScrollX() + viewport.width / 2;
    const viewportCenterY = this.viewportManager.getScrollY() + viewport.height / 2;
    const viewportPageIndex = this.currentVisiblePages.length > 0
      ? this.virtualScroll.getPageAtPoint(viewportCenterX, viewportCenterY)
      : null;
    const next = resolveActivePage({
      pageCount: this.virtualScroll.pageCount,
      visiblePages: this.currentVisiblePages,
      editingPageIndex: this.editingPageIndex,
      viewportPageIndex,
    });
    const snapshotChanged = !(
      next?.pageIndex === this.activePageSnapshot?.pageIndex
      && next?.source === this.activePageSnapshot?.source
    );

    this.activePageSnapshot = next;
    if (snapshotChanged) this.eventBus.emit('active-page-changed', next);
    // 전체 쪽 수·구역 쪽번호가 pagination으로 바뀔 수 있으므로 snapshot이 같아도
    // 기존 상태 표시줄 이벤트는 매 visible-page 갱신마다 유지한다.
    if (next) {
      this.eventBus.emit(
        'current-page-changed',
        next.pageIndex,
        this.virtualScroll.pageCount,
      );
    }
  }

  private cancelPendingPrefetch(): void {
    this.renderWorkGeneration += 1;
    // 일부 prototype 단위 테스트와 문서 교체 초기화는 constructor가 만든 scheduler보다
    // 먼저 정리 경계를 재현한다. production 인스턴스에는 항상 존재하지만 정리 자체는
    // 멱등이어야 하므로 없는 경우도 안전하게 허용한다.
    this.pageRenderScheduler?.cancelAll();
  }

  private refreshRenderSurfacePlan(rerenderChangedPages: boolean): void {
    if (this.pages.length === 0 || this.currentRetainedPages.length === 0) {
      this.renderSurfacePlan = null;
      this.renderSurfaceDecisions.clear();
      return;
    }

    const previousDecisions = this.renderSurfaceDecisions;
    const visibleSet = new Set(this.currentVisiblePages);
    const focusPage = this.editingPageIndex
      ?? this.activePageSnapshot?.pageIndex
      ?? this.currentVisiblePages[0]
      ?? 0;
    const layerCount = this.pageRenderer.getBackend() === 'canvaskit'
      ? 1
      : DEFAULT_CANVAS2D_LAYER_COUNT;
    const rawDpr = window.devicePixelRatio || 1;
    const renderProfile = this.pageRenderer.getRenderProfile();
    const environmentKey = `${rawDpr}:${layerCount}:${renderProfile}`;
    if (environmentKey !== this.renderSurfaceEnvironmentKey) {
      // 모니터 DPR/backend/profile이 바뀌면 이전 bucket은 새 환경의 hysteresis 근거가 아니다.
      this.previousEffectiveDpr.clear();
      this.renderSurfaceEnvironmentKey = environmentKey;
    }
    const plan = planRenderSurfaceBudget({
      pages: this.currentRetainedPages.flatMap((pageIndex) => {
        const page = this.pages[pageIndex];
        if (!page) return [];
        return [{
          pageIndex,
          width: page.width,
          height: page.height,
          layerCount: this.pageRenderer.getCanvasSurfaceLayerCount(pageIndex),
          visible: visibleSet.has(pageIndex),
          focused: this.editingPageIndex === pageIndex,
          distanceFromFocus: Math.abs(pageIndex - focusPage),
        }];
      }),
      zoom: this.viewportManager.getZoom(),
      rawDpr,
      layerCount,
      renderProfile,
      previousEffectiveDpr: this.previousEffectiveDpr,
    });
    const nextDecisions = new Map(
      plan.decisions.map(decision => [decision.pageIndex, decision]),
    );
    this.renderSurfacePlan = plan;
    this.renderSurfaceDecisions = nextDecisions;
    for (const decision of plan.decisions) {
      this.previousEffectiveDpr.set(decision.pageIndex, decision.effectiveDpr);
    }

    if (!rerenderChangedPages) {
      this.reconcilePageSurfaceBudget();
      for (const pageIndex of this.canvasPool.activePages) {
        const canvas = this.canvasPool.getCanvas(pageIndex);
        const descriptor = this.pageSurfaceDescriptor(pageIndex);
        if (!canvas || !descriptor) continue;
        if (canvas.dataset.rhwpSurfaceCacheLookupKey === descriptor.lookupKey) {
          this.applySurfaceDecisionDiagnostics(pageIndex, canvas);
        }
      }
      return;
    }
    for (const pageIndex of this.canvasPool.activePages) {
      const before = previousDecisions.get(pageIndex);
      const after = nextDecisions.get(pageIndex);
      if (!after) continue;
      const page = this.pages[pageIndex];
      const zoom = this.viewportManager.getZoom();
      const beforeScale = before && page
        ? clampRenderScale(page, zoom * before.effectiveDpr)
        : null;
      const afterScale = page
        ? clampRenderScale(page, zoom * after.effectiveDpr)
        : null;
      // tier label만 바뀌고 실제 bitmap scale이 같으면 raster 결과도 같으므로 다시 그리지 않는다.
      const changed = beforeScale === null
        || afterScale === null
        || Math.abs(beforeScale - afterScale) > 0.001;
      const canvas = this.canvasPool.getCanvas(pageIndex);
      if (!canvas) continue;
      if (changed) {
        this.renderCanvas(pageIndex, canvas);
      } else {
        // 가시성만 바뀐 페이지는 다시 raster하지 않고 진단값만 현재 plan에 맞춘다.
        this.applySurfaceDecisionDiagnostics(pageIndex, canvas);
      }
    }
    this.reconcilePageSurfaceBudget();
  }

  private applySurfaceDecisionDiagnostics(pageIdx: number, canvas: HTMLCanvasElement): void {
    const decision = this.renderSurfaceDecisions.get(pageIdx);
    const plan = this.renderSurfacePlan;
    canvas.dataset.rhwpSurfaceVisible = decision?.visible ? 'true' : 'false';
    canvas.dataset.rhwpSurfaceLayerCount = String(decision?.layerCount ?? 1);
    canvas.dataset.rhwpEstimatedSurfaceBytes = String(decision?.surfaceBytes ?? 0);
    canvas.dataset.rhwpEstimatedVisibleSurfaceBytes = String(
      (plan?.visibleSurfacePixels ?? 0) * 4,
    );
    canvas.dataset.rhwpEstimatedRetainedSurfaceBytes = String(
      (plan?.retainedSurfacePixels ?? 0) * 4,
    );
    canvas.dataset.rhwpSurfaceBudgetState = plan?.withinBudget === false ? 'exceeded' : 'within';
  }

  private pageSurfaceDescriptor(pageIdx: number): { lookupKey: string; estimatedPixelCount: number } | null {
    const page = this.pages[pageIdx];
    const decision = this.renderSurfaceDecisions.get(pageIdx);
    if (!page || !decision) return null;
    const zoom = this.viewportManager.getZoom();
    const renderScale = clampRenderScale(page, zoom * decision.effectiveDpr);
    // PageInfo와 실제 layer tree의 소수 정밀도가 다를 수 있으므로 생성 전 예약은 각 축 1px을 더 잡는다.
    // 생성 뒤에는 실제 Canvas 정수 치수 합으로 즉시 reconcile한다.
    const estimatedWidth = Math.max(1, Math.ceil(page.width * renderScale) + 1);
    const estimatedHeight = Math.max(1, Math.ceil(page.height * renderScale) + 1);
    const identity = this.activeRendererDecisionKey
      ?? `${this.wasm.documentDigest ?? 'document:none'}|generation:${this.wasm.documentGeneration}`;
    const lookupKey = [
      identity,
      `page:${pageIdx}`,
      `geometry:${page.width}x${page.height}`,
      `backend:${this.pageRenderer.getBackend()}`,
      `profile:${this.pageRenderer.getRenderProfile()}`,
      `scale:${renderScale}`,
      `layers:${decision.layerCount}`,
    ].join('|');
    return {
      lookupKey,
      estimatedPixelCount: estimatedWidth * estimatedHeight * decision.layerCount,
    };
  }

  private pageSurfaceElements(pageIdx: number, mainCanvas: HTMLCanvasElement): HTMLElement[] {
    const page = String(pageIdx);
    return Array.from(this.scrollContent.children).filter((element): element is HTMLElement => (
      element === mainCanvas
      || (element instanceof HTMLElement && element.dataset.rhwpOverlayPage === page)
    ));
  }

  private pageSurfaceShape(elements: readonly HTMLElement[], mainCanvas: HTMLCanvasElement): string {
    return elements.map((element) => {
      const kind = element === mainCanvas
        ? 'main'
        : element.dataset.rhwpLayerKind ?? element.dataset.rhwpOverlay ?? element.tagName.toLowerCase();
      return element instanceof HTMLCanvasElement
        ? `${kind}:${element.width}x${element.height}`
        : `${kind}:dom`;
    }).join(',');
  }

  private actualPageSurfacePixels(elements: readonly HTMLElement[]): number {
    return elements.reduce((sum, element) => (
      element instanceof HTMLCanvasElement ? sum + element.width * element.height : sum
    ), 0);
  }

  private detachCompletedPageSurface(pageIdx: number): PageSurfaceBundle | null {
    const mainCanvas = this.canvasPool.getCanvas(pageIdx);
    const descriptor = this.pageSurfaceDescriptor(pageIdx);
    if (
      !mainCanvas
      || !descriptor
      || mainCanvas.dataset.rhwpSurfaceCacheLookupKey !== descriptor.lookupKey
      || !this.pageRenderer.isPageSurfaceComplete(this.scrollContent, pageIdx)
    ) return null;

    this.removeGridOverlay(pageIdx);
    this.scrollContent.querySelector(`[data-rhwp-hf-edit-page="${pageIdx}"]`)?.remove();
    const elements = this.pageRenderer.detachPageSurfaceElements(
      this.scrollContent,
      pageIdx,
      mainCanvas,
    );
    const detachedMain = this.canvasPool.detach(pageIdx);
    if (detachedMain !== mainCanvas) {
      this.pageRenderer.attachPageSurfaceElements(this.scrollContent, elements);
      return null;
    }
    const pixelCount = this.actualPageSurfacePixels(elements);
    const key = `${descriptor.lookupKey}|surfaces:${this.pageSurfaceShape(elements, mainCanvas)}`;
    return {
      key,
      lookupKey: descriptor.lookupKey,
      pageIndex: pageIdx,
      pixelCount,
      mainCanvas,
      elements,
      leaseId: ++this.nextSurfaceLeaseId,
    };
  }

  private discardActivePageSurface(pageIdx: number): void {
    this.pageRenderer.cancelReRender(pageIdx);
    this.pageRenderer.removePageLayers(this.scrollContent, pageIdx);
    this.pageRenderer.releasePageDiagnostics(pageIdx);
    this.removeGridOverlay(pageIdx);
    this.scrollContent.querySelector(`[data-rhwp-hf-edit-page="${pageIdx}"]`)?.remove();
    this.canvasPool.release(pageIdx);
  }

  private disposeCachedPageSurface(bundle: PageSurfaceBundle): void {
    this.pageRenderer.cancelReRender(bundle.pageIndex);
    this.pageRenderer.removePageLayers(this.scrollContent, bundle.pageIndex);
    this.pageRenderer.releasePageDiagnostics(bundle.pageIndex);
    for (const element of bundle.elements) {
      element.remove();
      if (element !== bundle.mainCanvas && element instanceof HTMLCanvasElement) {
        element.width = 0;
        element.height = 0;
      }
    }
    this.canvasPool.releaseDetached(bundle.mainCanvas);
  }

  private restoreCachedPageSurface(bundle: PageSurfaceBundle): void {
    this.canvasPool.adopt(bundle.pageIndex, bundle.mainCanvas);
    this.pageRenderer.attachPageSurfaceElements(this.scrollContent, bundle.elements);
    for (const element of bundle.elements) this.positionPageElement(element, bundle.pageIndex);
    bundle.mainCanvas.dataset.rhwpSurfaceCacheLease = String(bundle.leaseId);
    this.applySurfaceDecisionDiagnostics(bundle.pageIndex, bundle.mainCanvas);
    this.renderGridOverlay(bundle.pageIndex, bundle.mainCanvas);
  }

  private reconcilePageSurfaceBudget(): void {
    const lru = this.pageSurfaceLru;
    if (!lru) return;
    const plan = this.renderSurfacePlan;
    if (!plan) {
      lru.reconcile(DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET, 0);
      return;
    }
    let reservedPixels = 0;
    for (const decision of plan.decisions) {
      const descriptor = this.pageSurfaceDescriptor(decision.pageIndex);
      if (!descriptor) continue;
      const activeCanvas = this.canvasPool.getCanvas(decision.pageIndex);
      if (activeCanvas) {
        const actualPixels = Number(activeCanvas.dataset.rhwpActualSurfacePixels);
        reservedPixels += Number.isFinite(actualPixels) && actualPixels > 0
          ? actualPixels
          : descriptor.estimatedPixelCount;
      } else if (!lru.hasLookup(descriptor.lookupKey)) {
        reservedPixels += descriptor.estimatedPixelCount;
      }
    }
    lru.reconcile(plan.retainedPixelBudget, reservedPixels);
  }

  getPageSurfaceCacheDiagnostics(): PageSurfaceLruSnapshot {
    return this.pageSurfaceLru.snapshot();
  }

  getPageRenderSchedulerDiagnostics(): PageRenderSchedulerSnapshot {
    return this.pageRenderScheduler.snapshot();
  }

  /** 단일 페이지를 렌더링하거나 정확한 detached bundle을 재부착한다. */
  private renderPage(pageIdx: number): void {
    const descriptor = this.pageSurfaceDescriptor(pageIdx);
    if (descriptor) {
      const cached = this.pageSurfaceLru.takeLookup(descriptor.lookupKey);
      if (cached) {
        this.restoreCachedPageSurface(cached);
        this.reconcilePageSurfaceBudget();
        return;
      }
      // 같은 page의 다른 DPR/backend/revision surface는 정확한 hit가 아니며 headroom을 점유하지 않는다.
      this.pageSurfaceLru.deletePage(pageIdx);
    }

    const canvas = this.canvasPool.acquire(pageIdx);
    if (!canvas.parentElement) this.scrollContent.appendChild(canvas);
    if (!this.renderCanvas(pageIdx, canvas)) {
      this.canvasPool.release(pageIdx);
    }
    this.reconcilePageSurfaceBudget();
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
    if (!this.renderSurfaceDecisions.has(pageIdx)) {
      this.refreshRenderSurfacePlan(false);
    }
    const surfaceDecision = this.renderSurfaceDecisions.get(pageIdx);
    const requestedDpr = surfaceDecision?.effectiveDpr ?? rawDpr;
    // iOS/WebKit과 GPU surface가 감당하기 어려운 물리 픽셀 수를 중앙 정책으로 제한한다.
    const renderScale = clampRenderScale(pageInfo, zoom * requestedDpr);
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
      renderedCanvas.classList.add('document-page-canvas');
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
    renderedCanvas.dataset.rhwpRenderTier = surfaceDecision?.tier ?? 'screen';
    renderedCanvas.dataset.rhwpRenderBucket = String(dpr);
    renderedCanvas.dataset.rhwpRenderScale = String(renderScale);
    renderedCanvas.dataset.rhwpRawDpr = String(rawDpr);
    renderedCanvas.dataset.rhwpEffectiveDpr = String(dpr);
    this.applySurfaceDecisionDiagnostics(pageIdx, renderedCanvas);
    const surfaceDescriptor = this.pageSurfaceDescriptor(pageIdx);
    const surfaceElements = this.pageSurfaceElements(pageIdx, renderedCanvas);
    const actualSurfacePixels = this.actualPageSurfacePixels(surfaceElements);
    renderedCanvas.dataset.rhwpActualSurfacePixels = String(actualSurfacePixels);
    if (surfaceDescriptor) {
      renderedCanvas.dataset.rhwpSurfaceCacheLookupKey = surfaceDescriptor.lookupKey;
      renderedCanvas.dataset.rhwpSurfaceCacheKey = `${surfaceDescriptor.lookupKey}|surfaces:${this.pageSurfaceShape(surfaceElements, renderedCanvas)}`;
    } else {
      delete renderedCanvas.dataset.rhwpSurfaceCacheLookupKey;
      delete renderedCanvas.dataset.rhwpSurfaceCacheKey;
    }
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
      this.updateVisiblePages('strict');
    });
  }

  /** 뷰포트 리사이즈 처리 */
  private onViewportResize(): void {
    const nextViewport = this.viewportManager.getViewportSize();
    if (this.pages.length === 0) {
      this.layoutViewportSize = nextViewport;
      this.updateVisiblePages('resize');
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
    this.updateVisiblePages('resize');
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
    this.updateVisiblePages('zoom-settled');
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
        `[data-rhwp-overlay-page="${pageIdx}"], [data-rhwp-grid-page="${pageIdx}"], [data-rhwp-hf-edit-page="${pageIdx}"]`,
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
    this.updateVisiblePages('mutation');
  }

  /** 텍스트 입력처럼 좁은 변경은 page info 재수집 없이 해당 페이지 canvas만 다시 그린다. */
  private refreshInvalidatedPage(payload: unknown): void {
    if (this.pages.length === 0) return;
    // 부분 revision 계약이 없으므로 편집/undo/redo는 detached bundle 전체를 무효화한다.
    this.pageSurfaceLru.clear();

    const pageIndex =
      typeof payload === 'object' && payload !== null && 'pageIndex' in payload
        ? Number((payload as { pageIndex?: unknown }).pageIndex)
        : Number(payload);
    const reason =
      typeof payload === 'object' && payload !== null && 'reason' in payload
        ? (payload as { reason?: unknown }).reason
        : undefined;
    const focusedPagePatch =
      typeof payload === 'object' && payload !== null && 'focusedPagePatch' in payload
        ? (payload as { focusedPagePatch?: unknown }).focusedPagePatch
        : undefined;
    const validFocusedPagePatch = (() => {
      if (!focusedPagePatch || typeof focusedPagePatch !== 'object') return undefined;
      const candidate = focusedPagePatch as {
        pageIndex?: unknown;
        x?: unknown;
        y?: unknown;
        width?: unknown;
        height?: unknown;
      };
      const numbers = [candidate.x, candidate.y, candidate.width, candidate.height];
      if (
        !Number.isSafeInteger(candidate.pageIndex)
        || candidate.pageIndex !== pageIndex
        || !numbers.every((value) => typeof value === 'number' && Number.isFinite(value))
        || (candidate.width as number) <= 0
        || (candidate.height as number) <= 0
      ) {
        return undefined;
      }
      return {
        pageIndex: candidate.pageIndex as number,
        x: candidate.x as number,
        y: candidate.y as number,
        width: candidate.width as number,
        height: candidate.height as number,
      };
    })();
    const renderContext: PageRenderContext =
      reason === 'text-edit'
        ? {
            reason: 'text-edit',
            allowStaticOverlayReuse: true,
            ...(validFocusedPagePatch ? { focusedPagePatch: validFocusedPagePatch } : {}),
          }
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
      this.updateVisiblePages('mutation');
      return;
    }

    if (!this.renderCanvas(pageIndex, canvas, renderContext)) {
      this.canvasPool.release(pageIndex);
      this.updateVisiblePages('mutation');
      return;
    }
    this.renderHeaderFooterEditOverlays(true);
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
    const hadActivePage = this.activePageSnapshot !== null;
    const hadFocusedPage = this.editingPageIndex !== null;
    this.cancelPendingTextEditRefresh();
    this.cancelTextEditStaticLayerVerification();
    this.cancelPendingPrefetch();
    this.pageRenderer.cancelAll();
    this.releaseAllRenderedPages();
    this.currentVisiblePages = [];
    this.currentRetainedPages = [];
    this.editingPageIndex = null;
    this.headerFooterEditState = null;
    this.pageRenderer.setPageMarginGuideEdges('both');
    this.activePageSnapshot = null;
    this.virtualScroll.reset();
    this.previousEffectiveDpr.clear();
    this.renderSurfaceEnvironmentKey = null;
    if (hadActivePage) this.eventBus.emit('active-page-changed', null);
    if (hadFocusedPage) this.eventBus.emit('focused-page-changed', null);
    this.pages = [];
    this.scrollContent.replaceChildren();
    this.blankPagePlaceholder = null;
  }

  private releaseAllRenderedPages(): void {
    this.cancelPendingPrefetch();
    this.pageSurfaceLru?.clear();
    this.pageRenderer.resetImageRetryState();
    this.pageRenderer.removeAllPageLayers(this.scrollContent);
    this.removeHeaderFooterEditOverlays();
    this.removeAllGridOverlays();
    this.canvasPool.releaseAll();
    this.renderSurfacePlan = null;
    this.renderSurfaceDecisions.clear();
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

  /**
   * 뷰가 쥔 것을 전부 놓는다 — 감시자, 렌더러 세션, 뷰포트 청취, 이벤트 구독.
   *
   * **호출부가 없는 것이 지금의 계약이다** (#4592). `canvasView` 는 `main.ts` 모듈 바인딩이고
   * 한 번 만들어진 뒤 교체되지 않는다. 스튜디오에는 문서 닫기도 뷰 교체도 없으므로 이 뷰의
   * 수명은 realm 과 같고, 탭이 닫히면 감시자·구독·wasm 핸들이 함께 사라진다. 그래서 지금
   * 누수는 없다 — 없는 것은 해체 **경로**이지 해체 **구현**이 아니다.
   *
   * `pagehide`/`beforeunload` 에 걸지 않는다. **적극적으로 해롭다** — bfcache 로 복원되는
   * 페이지에서 폐기된 뷰가 되살아나고, 어차피 realm 이 사라지는 시점의 해제는 의식일 뿐이다.
   * #4579 가 같은 이유로 배선을 거절했다.
   *
   * 이 메서드와 `disposed` 가드들이 살아나는 시점은 하나뿐이다: 문서 닫기나 뷰 교체 기능이
   * 생길 때. 개발용 렌더 런타임은 `main.ts`가 realm 단위로 소유하므로, 그 새 수명 경로에서
   * 반환된 해제 함수를 함께 부른다. 그 전까지 호출부를 지어내지 않는다.
   */
  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
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

  /**
   * 문서 내용과 무관한 페이지 화면 배치를 바꾼다.
   *
   * 중심 쪽을 전환 전후 같은 뷰포트 앵커에 놓고, 실제 행·열 슬롯 토폴로지가 달라진 경우에만
   * Canvas 내용을 버린다. 좌표만 달라지면 recalcLayout()의 reposition 경로로 기존 Canvas를 쓴다.
   */
  private setPageViewSettings(changeValue: unknown): boolean {
    if (this.disposed) return false;
    const next = resolvePageViewSettingsChange(changeValue);
    const movementUnchanged = this.pageMovement.direction === next.pageMovement.direction
      && this.pageMovement.wheelHorizontal === next.pageMovement.wheelHorizontal;
    const viewChanged = !pageArrangementsEqual(this.pageArrangement, next.arrangement)
      || !movementUnchanged;
    if (!viewChanged && !next.zoom) return false;

    if (this.pages.length === 0) {
      this.pageArrangement = next.arrangement;
      this.pageMovement = next.pageMovement;
      this.viewportManager.setPageMovement(this.pageMovement);
      if (next.zoom) {
        this.applyingPageViewSettingsTransaction = true;
        try {
          this.viewportManager.setZoom(
            next.zoom.value,
            next.zoom.anchor,
            next.zoom.fitMode,
          );
        } finally {
          this.applyingPageViewSettingsTransaction = false;
        }
        this.eventBus.emit('zoom-level-display', this.viewportManager.getZoom());
      }
      return true;
    }

    const scrollTop = this.viewportManager.getScrollY();
    const scrollLeft = this.viewportManager.getScrollX();
    const { width: viewportWidth, height: viewportHeight } = this.viewportManager.getViewportSize();
    const anchor = next.zoom?.anchor ?? CENTER_ZOOM_ANCHOR;
    const focusPage = this.virtualScroll.getPageAtPoint(
      scrollLeft + viewportWidth * anchor.x,
      scrollTop + viewportHeight * anchor.y,
    );
    const oldBox = this.getZoomPageBox(focusPage, viewportWidth);
    const previousTopology = this.virtualScroll.getLayoutTopologyKey();
    const previousZoom = this.viewportManager.getZoom();

    this.pageArrangement = next.arrangement;
    this.pageMovement = next.pageMovement;
    this.viewportManager.setPageMovement(this.pageMovement);
    let zoomEventFailure: { error: unknown } | null = null;
    if (next.zoom) {
      this.applyingPageViewSettingsTransaction = true;
      try {
        this.viewportManager.setZoom(
          next.zoom.value,
          next.zoom.anchor,
          next.zoom.fitMode,
        );
      } catch (error) {
        // EventBus는 모든 구독자에게 배달한 뒤 첫 오류를 되던진다. 최종 레이아웃 commit까지
        // 끝낸 다음 같은 오류를 다시 던져 transaction guard나 화면 상태가 반쪽으로 남지 않게 한다.
        zoomEventFailure = { error };
      } finally {
        this.applyingPageViewSettingsTransaction = false;
      }
    }
    const zoomChanged = this.viewportManager.getZoom() !== previousZoom;
    const layoutChanged = viewChanged || zoomChanged;

    if (layoutChanged) this.recalcLayout();

    const nextTopology = this.virtualScroll.getLayoutTopologyKey();
    if (layoutChanged) {
      const newBox = this.getZoomPageBox(focusPage, viewportWidth);
      const nextScroll = calculateAnchoredScroll(
        oldBox,
        newBox,
        {
          width: viewportWidth,
          height: viewportHeight,
          scrollLeft,
          scrollTop,
        },
        anchor,
      );
      this.viewportManager.setScrollLeft(this.clampScrollLeft(nextScroll.scrollLeft));
      this.viewportManager.setScrollTop(nextScroll.scrollTop);
    }

    if (next.zoom) {
      this.eventBus.emit('zoom-level-display', this.viewportManager.getZoom());
    }

    if (layoutChanged && (zoomChanged || previousTopology !== nextTopology)) {
      this.cancelPendingTextEditRefresh();
      this.cancelTextEditStaticLayerVerification();
      this.cancelPendingPrefetch();
      this.releaseAllRenderedPages();
      this.pageRenderer.cancelAll();
    }
    if (layoutChanged) this.updateVisiblePages('zoom-settled');
    if (zoomEventFailure) throw zoomEventFailure.error;
    return true;
  }

  getViewportManager(): ViewportManager {
    return this.viewportManager;
  }

  /** 전역 쪽 번호를 뷰포트 상단으로 이동한다. */
  gotoPage(pageIndex: number): boolean {
    if (!Number.isInteger(pageIndex) || pageIndex < 0 || pageIndex >= this.virtualScroll.pageCount) {
      return false;
    }
    if (this.virtualScroll.isHorizontalMode()) {
      this.viewportManager.setScrollLeft(this.clampScrollLeft(
        this.virtualScroll.getPageLeft(pageIndex) - this.virtualScroll.getPageGap(),
      ));
    } else {
      this.viewportManager.setScrollTop(this.virtualScroll.getPageOffset(pageIndex));
    }
    return true;
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

  getCanvasKitFontDecisionEvidence(pageIndex: number, record: FontDecisionTraceRecordV1) {
    return this.pageRenderer.getCanvasKitFontDecisionEvidence(pageIndex, record);
  }

  getCoordinateSystem(): CoordinateSystem {
    return this.coordinateSystem;
  }
}
