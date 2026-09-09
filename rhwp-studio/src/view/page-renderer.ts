import { WasmBridge, type DeferredFocusedPagePatch } from '@/core/wasm-bridge';
import type { LayerRenderProfile } from '@/core/types';
import type { FontDecisionTraceRecordV1 } from '@/core/font-decision-trace';
import { layerPaintOpReplayPlane } from './canvaskit/replay-plane';
import type {
  CanvasKitFontDecisionEvidence,
  CanvasKitLayerRenderer,
  CanvasKitRenderDiagnostics,
} from './canvaskit-renderer';
import {
  cacheableImageKeySignature,
  collectImagePrefetchDataUrls,
  completeImagePrefetch,
  shouldSkipImagePrefetch,
  type PrefetchSignature,
} from './image-prefetch-signature.ts';
import { collectVectorRawSvgDataUrls } from './raw-svg-prefetch';
import {
  collectFlowImagePaintOps,
  flowImageOpsFromNarrowQuery,
  visibleFlowImageBbox,
  type FlowImagePaintOp,
} from './flow-image-clip';
import { FlowImageUrlCache } from './flow-image-url-cache';
import {
  drawPageMarginGuides,
  type PageMarginGuideEdges,
  type PageSpaceRect,
} from './page-margin-guides';
import type { RenderBackend } from './render-backend';
import { isSameRenderDocument, type RenderDocumentIdentity } from './render-document-identity.ts';

interface LayerPlaneSummary {
  hasBehind: boolean;
  hasFront: boolean;
  imageCount: number;
  rawSvgCount: number;  // OLE/차트 rawSvg op 수 — 비동기 디코드 재렌더 트리거용(image 와 의미 분리, #1456)
  flowImageCount: number;
  flowRawSvgCount: number;
  flowStaticCount: number;
  // [#5763] flow 그림 밑에 불투명 채우기(그림을 담은 표 칸의 흰 배경 등)가 깔린 페이지.
  // flow-static 분리는 그림만 canvas 아래 평면으로 내리고 그 채우기는 canvas 에 남기므로,
  // 분리하면 채우기가 그림을 덮어 그림이 통째로 사라진다. 그런 페이지는 분리하지 않는다.
  flowStaticOccluded: boolean;
  // [#5780] 쪽 배경. DOM flow 그림 갈래(DIV)는 Background plane 을 실을 자리가 없어
  // 단색 배경은 DIV background 로 실어 주고, 그라데이션/이미지 배경은 DOM 갈래를
  // 포기하고 flow-static canvas(Background 포함 필터)로 폴백한다.
  pageBackgroundCss: string | null;
  pageBackgroundComplex: boolean;
  signature: string;
}

export interface PageRenderContext {
  reason?: 'text-edit' | 'unknown';
  allowStaticOverlayReuse?: boolean;
  focusedPagePatch?: DeferredFocusedPagePatch;
}

export interface PageRenderResult {
  needsTextEditStaticLayerVerification: boolean;
  renderedCanvas?: HTMLCanvasElement;
}

type OverlayLayerKind = 'background' | 'behind' | 'front';
type StaticCanvasLayerKind = OverlayLayerKind | 'flow-static';

interface ReRenderPolicy {
  retrySignature: string;
  reuseStaticFlow: boolean;
  reuseStaticOverlay: boolean;
  displayScale: number;
}

interface LayerSummaryCacheEntry {
  key: string;
  summary: LayerPlaneSummary;
}

interface ReRenderJob {
  fallbackTimer: ReturnType<typeof setTimeout>;
  earlyRawSvgTimers: ReturnType<typeof setTimeout>[];
  completed: boolean;
}

const IMAGE_RE_RENDER_FALLBACK_DELAY_MS = 1500;
// 순수 SVG 차트/OLE는 prefetch 대상 data URL이 없을 수 있다. 첫 paint가 시작한
// 이미지 decode를 빠르게 반영하되, 일반 이미지처럼 전역 반복 재렌더는 피한다.
const RAW_SVG_EARLY_RE_RENDER_DELAYS_MS = [0, 32, 96, 240] as const;
const HWP_UNITS_PER_CSS_PIXEL = 75;

export class PageRenderer {
  private reRenderJobs = new Map<number, ReRenderJob>();
  private imageRetryCounts = new Map<number, string>();
  private layerSummaryCache = new Map<number, LayerSummaryCacheEntry>();
  /** zoom과 무관한 페이지별 Canvas surface 상한. 문서/revision 경계에서만 무효화한다. */
  private surfaceLayerCountCache = new Map<number, number>();
  private canvaskitDiagnosticsByPage = new Map<number, CanvasKitRenderDiagnostics>();
  /**
   * prefetch 를 끝낸 페이지의 그림 서명 (Task #3315).
   *
   * 내용에서 유도된 키라 스스로 무효화된다 — 편집 때 비우지 않는다. 비우면 서명을 두는
   * 의미가 사라진다. 서명은 자기가 어느 문서의 것인지(`documentDigest`) 함께 들고 다니므로
   * 옛 문서의 항목이 새 문서에서 잘못 맞아떨어지지 않는다. 다만 **맞지 않을 뿐 사라지지도
   * 않으므로**, 수명은 `beginDocument` 가 문서 경계에서 거둔다.
   */
  private prefetchedImageSignatures = new Map<number, PrefetchSignature>();
  /**
   * DOM flow 그림의 신원 키별 object URL (Task #3315).
   *
   * 키가 내용에서 유도되므로 스스로 무효화된다 — 편집 때 비우지 않는다. 문서 경계는
   * `beginDocument` 가 가른다.
   */
  private flowImageUrls = new FlowImageUrlCache();
  /**
   * 위 페이지 단위 파생 상태가 어느 문서의 것인지 (Task #3315).
   *
   * `beginDocument` 가 이 값과 현재 문서를 견줘 거둘지 말지 정한다. 항목마다 신원이 박혀 있는
   * 것과 별개로 필요하다 — 항목의 신원은 "새 문서에서 잘못 맞지 않게" 하고, 이 값은 "옛 문서의
   * 항목을 언제 버릴지"를 정한다.
   */
  private documentScope: RenderDocumentIdentity | null = null;
  private prefetchRequestTokens = new Map<number, number>();
  private nextPrefetchRequestToken = 0;
  private flowSplitSupported: boolean | null = null;
  private pageMarginGuideEdges: PageMarginGuideEdges = 'both';

  constructor(
    private wasm: WasmBridge,
    private backend: RenderBackend = 'canvas2d',
    private renderProfile: LayerRenderProfile = 'screen',
    private canvaskitRenderer: CanvasKitLayerRenderer | null = null,
  ) {}

  configure(
    backend: RenderBackend,
    renderProfile: LayerRenderProfile,
    canvaskitRenderer: CanvasKitLayerRenderer | null,
    preserveCanvasKitDiagnostics = false,
  ): boolean {
    const changed =
      this.backend !== backend
      || this.renderProfile !== renderProfile
      || this.canvaskitRenderer !== canvaskitRenderer;
    if (!changed) return false;

    this.cancelAll();
    if (!preserveCanvasKitDiagnostics) this.releaseAllPageDiagnostics();
    this.layerSummaryCache.clear();
    this.surfaceLayerCountCache.clear();
    this.backend = backend;
    this.renderProfile = renderProfile;
    this.canvaskitRenderer = canvaskitRenderer;
    return true;
  }

  setPageMarginGuideEdges(edges: PageMarginGuideEdges): boolean {
    if (this.pageMarginGuideEdges === edges) return false;
    this.pageMarginGuideEdges = edges;
    return true;
  }

  /**
   * 문서 (재)로드 경계 — `CanvasView.prepareDocumentLoad` 가 부른다 (Task #3315).
   *
   * **문서 범위 파생 상태의 수명을 정하는 유일한 자리다.** `PageRenderer` 는 문서보다 오래
   * 살므로, 여기서 거두지 않으면 세션이 끝날 때까지 남는다 — `dispose()` 는 문서 닫기·뷰 교체
   * 기능이 생길 때를 위한 자리라 지금은 호출부가 없다(`CanvasView.dispose`).
   *
   * 거두는 것은 셋이다.
   *
   * - flow 그림 object URL — 브라우저가 명시적 회수까지 붙들고 있다. 조회 시점으로 미루면 새
   *   문서가 flow 그림을 한 장도 조회하지 않을 때(그림 없는 문서·CanvasKit 경로) 옛 문서의
   *   URL 이 그대로 남는다.
   * - 재시도 키(`imageRetryCounts`)·prefetch 서명(`prefetchedImageSignatures`) — 둘 다 키에
   *   문서 신원이 박혀 있어 새 문서에서 **잘못 맞아떨어지지는 않는다.** 그래서 이건 정확성이
   *   아니라 수명 문제다. 다시 읽히지 않을 항목이 문서를 열 때마다 페이지 수만큼 쌓인다.
   *
   * 편집(문서 revision 변화)으로는 거두지 않는다 — 그 경계는 `resetImageRetryState` 이고,
   * 거기서 재시도 키를 비우면 페이지마다 재렌더가 한 번 더 돈다(#3672). 페이지가 풀에서
   * 빠질 때도 거두지 않는다 — 같은 이유로 페이지를 다시 볼 때마다 재렌더가 한 번 더 돈다.
   *
   * 같은 문서를 다시 로드한 경우에는 신원이 같으므로 그대로 둔다.
   */
  beginDocument(): void {
    const identity: RenderDocumentIdentity = {
      digest: this.wasm.documentDigest,
      generation: this.wasm.documentGeneration,
    };
    this.flowImageUrls.beginDocument(identity);
    if (isSameRenderDocument(this.documentScope, identity)) return;

    this.imageRetryCounts.clear();
    this.prefetchedImageSignatures.clear();
    this.surfaceLayerCountCache.clear();
    // 신원을 모르면(`digest === null`) 항목이 어느 문서 것인지 표시할 수 없다. 그 상태에서는
    // `buildImageRetryKey` 도 서명 기록도 멈추므로 지킬 것이 없다 — 범위를 비워 둔다.
    this.documentScope = identity.digest === null ? null : identity;
  }

  invalidateDocumentRevision(): void {
    this.cancelAll();
    this.releaseAllPageDiagnostics();
    this.layerSummaryCache.clear();
    this.surfaceLayerCountCache.clear();
    // [#3315] object URL 캐시는 여기서 비우지 않는다. 이 메서드는 renderer decision key 에
    // 묶여 있어 같은 문서를 편집할 때마다 불리므로, 여기서 비우면 캐시가 매 키 입력에 수 MB 를
    // 다시 읽는다 — 캐시가 없는 것과 같아진다. 문서 경계는 `beginDocument` 가 가른다.
  }

  /** 페이지를 Canvas에 렌더링한다 (renderScale = zoom × DPR) */
  renderPage(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    displayScale: number,
    dpr: number,
    context: PageRenderContext = {},
  ): PageRenderResult {
    if (this.backend === 'canvaskit') {
      this.layerSummaryCache.delete(pageIdx);
      this.surfaceLayerCountCache.set(pageIdx, 1);
      const renderedCanvas = this.renderPageCanvasKit(pageIdx, canvas, renderScale);
      return { needsTextEditStaticLayerVerification: false, renderedCanvas };
    }

    if (
      context.reason === 'text-edit'
      && context.focusedPagePatch?.pageIndex === pageIdx
      && this.renderFocusedPagePatch(pageIdx, canvas, renderScale, displayScale, context)
    ) {
      return { needsTextEditStaticLayerVerification: false };
    }

    const layers = this.getLayerPlaneSummary(pageIdx, canvas, renderScale, context);
    this.surfaceLayerCountCache.set(pageIdx, this.canvasSurfaceLayerCount(layers));
    const preferStaticFlow = this.shouldSplitStaticFlow(layers);
    let reuseStaticFlow = this.renderFlowCanvas(pageIdx, canvas, renderScale, preferStaticFlow);
    const flowImages = reuseStaticFlow && layers.flowImageCount > 0
      ? this.getFlowImagePaintOps(pageIdx)
      : [];
    const usesDomFlowImages =
      reuseStaticFlow &&
      layers.flowRawSvgCount === 0 &&
      flowImages.length === layers.flowImageCount &&
      flowImages.length > 0 &&
      // [#5780] 그라데이션/이미지 쪽 배경은 DIV 로 못 싣는다 — Background plane 을
      // 포함하는 flow-static canvas 갈래로 폴백한다.
      !layers.pageBackgroundComplex;

    // 다층 layer 모드.
    // 1) 본문 Canvas 는 'flow' 필터로 BehindText/InFrontOfText plane 제외
    // 2) behind/front plane 은 같은 부모 컨테이너에 별도 canvas layer 로 합성
    this.drawMarginGuides(pageIdx, canvas, renderScale, undefined, displayScale);
    let overlays: LayerPlaneSummary;
    try {
      overlays = this.applyOverlays(
        pageIdx,
        canvas,
        renderScale,
        dpr,
        context,
        layers,
        reuseStaticFlow,
        usesDomFlowImages ? flowImages : [],
      );
    } catch (error) {
      if (!reuseStaticFlow) throw error;
      this.flowSplitSupported = false;
      this.surfaceLayerCountCache.clear();
      canvas.parentElement && this.removeOverlayLayer(canvas.parentElement, pageIdx, 'flow-static');
      reuseStaticFlow = false;
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow', this.renderProfile);
      this.drawMarginGuides(pageIdx, canvas, renderScale, undefined, displayScale);
      overlays = this.applyOverlays(pageIdx, canvas, renderScale, dpr, context, layers, false, []);
    }
    this.rememberLayerPlaneSummary(pageIdx, canvas, renderScale, layers);
    // rawSvg(차트/OLE)도 web_canvas draw_image 비동기 디코드 경로를 타므로
    // image 와 함께 재렌더 트리거 카운트에 합산한다(#1456).
    this.scheduleReRender(
      pageIdx,
      canvas,
      renderScale,
      usesDomFlowImages ? overlays.rawSvgCount : overlays.imageCount + overlays.rawSvgCount,
      overlays.rawSvgCount,
      {
        retrySignature: overlays.signature,
        reuseStaticFlow,
        reuseStaticOverlay: context.reason === 'text-edit' && context.allowStaticOverlayReuse === true,
        displayScale,
      },
    );
    return {
      needsTextEditStaticLayerVerification:
        context.reason === 'text-edit' &&
        context.allowStaticOverlayReuse === true &&
        ((reuseStaticFlow && !usesDomFlowImages) || layers.hasBehind || layers.hasFront),
    };
  }

  getBackend(): RenderBackend {
    return this.backend;
  }

  getRenderProfile(): LayerRenderProfile {
    return this.renderProfile;
  }

  /**
   * 해당 페이지가 만들 수 있는 Canvas surface 수를 콘텐츠 plane 구성에서 계산한다.
   * 아직 렌더하지 않은 retained 페이지도 전역 예산에 넣을 수 있도록 zoom과 독립적으로 캐시한다.
   */
  getCanvasSurfaceLayerCount(pageIdx: number): number {
    if (this.backend === 'canvaskit') return 1;
    const cached = this.surfaceLayerCountCache.get(pageIdx);
    if (cached !== undefined) return cached;

    const layers = this.getLayerPlaneSummaryFromOverlayImages(pageIdx)
      ?? this.getLayerPlaneSummaryFromTree(pageIdx);
    const layerCount = this.canvasSurfaceLayerCount(layers);
    this.surfaceLayerCountCache.set(pageIdx, layerCount);
    return layerCount;
  }

  private canvasSurfaceLayerCount(layers: LayerPlaneSummary): number {
    return 1
      + (this.shouldSplitStaticFlow(layers) ? 1 : 0)
      + (layers.hasBehind ? 2 : 0)
      + (layers.hasFront ? 1 : 0);
  }

  getCanvasKitRenderDiagnostics(pageIdx: number): CanvasKitRenderDiagnostics | null {
    const diagnostics = this.canvaskitDiagnosticsByPage.get(pageIdx);
    if (!diagnostics) return null;
    return {
      ...diagnostics,
      lastUnsupportedOps: [...diagnostics.lastUnsupportedOps],
      lastExpectedUnsupportedOps: [...diagnostics.lastExpectedUnsupportedOps],
      lastUnexpectedUnsupportedOps: [...diagnostics.lastUnexpectedUnsupportedOps],
      readinessBlockers: [...diagnostics.readinessBlockers],
      replayFeatureCounts: { ...diagnostics.replayFeatureCounts },
    };
  }

  /** DEV baseline용 renderer-global counter의 최신 snapshot을 반환한다. */
  getCurrentCanvasKitRenderDiagnostics(): CanvasKitRenderDiagnostics | null {
    if (!this.canvaskitRenderer) return null;
    const diagnostics = this.canvaskitRenderer.diagnostics();
    return {
      ...diagnostics,
      lastUnsupportedOps: [...diagnostics.lastUnsupportedOps],
      lastExpectedUnsupportedOps: [...diagnostics.lastExpectedUnsupportedOps],
      lastUnexpectedUnsupportedOps: [...diagnostics.lastUnexpectedUnsupportedOps],
      readinessBlockers: [...diagnostics.readinessBlockers],
      replayFeatureCounts: { ...diagnostics.replayFeatureCounts },
    };
  }

  getCanvasKitFontDecisionEvidence(
    pageIndex: number,
    record: FontDecisionTraceRecordV1,
  ): CanvasKitFontDecisionEvidence | null {
    const diagnostics = this.canvaskitDiagnosticsByPage.get(pageIndex);
    if (!diagnostics) return null;
    return this.canvaskitRenderer?.fontDecisionEvidence(
      record,
      diagnostics.replayFeatureCounts.glyphRuns > 0,
    ) ?? null;
  }

  releasePageDiagnostics(pageIdx: number): void {
    this.canvaskitDiagnosticsByPage.delete(pageIdx);
  }

  releaseAllPageDiagnostics(): void {
    this.canvaskitDiagnosticsByPage.clear();
  }

  /** 비동기 이미지/RawSvg 보정까지 끝난 surface만 완성 cache entry로 승격한다. */
  isPageSurfaceComplete(parent: HTMLElement, pageIdx: number): boolean {
    if (this.reRenderJobs.has(pageIdx)) return false;
    return Array.from(
      parent.querySelectorAll<HTMLImageElement>(
        `[data-rhwp-overlay-page="${pageIdx}"] img`,
      ),
    ).every(image => image.complete && image.naturalWidth > 0);
  }

  /** main Canvas와 해당 page overlay를 현재 DOM 순서 그대로 분리한다. */
  detachPageSurfaceElements(
    parent: HTMLElement,
    pageIdx: number,
    mainCanvas: HTMLCanvasElement,
  ): HTMLElement[] {
    const page = String(pageIdx);
    const elements = Array.from(parent.children).filter((element): element is HTMLElement => (
      element === mainCanvas
      || (element instanceof HTMLElement && element.dataset.rhwpOverlayPage === page)
    ));
    for (const element of elements) element.remove();
    return elements;
  }

  attachPageSurfaceElements(parent: HTMLElement, elements: readonly HTMLElement[]): void {
    for (const element of elements) parent.appendChild(element);
  }

  private renderPageCanvasKit(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
  ): HTMLCanvasElement {
    this.canvaskitDiagnosticsByPage.delete(pageIdx);
    if (!this.canvaskitRenderer) {
      throw new Error('CanvasKit renderer가 초기화되지 않았습니다');
    }

    const parent = canvas.parentElement;
    const canvasChildIndex = parent
      ? Array.prototype.indexOf.call(parent.children, canvas)
      : -1;
    if (parent) {
      this.removePageLayers(parent, pageIdx);
    }

    let renderStarted = false;
    try {
      const pageInfo = this.wasm.getPageInfo(pageIdx);
      // WASM Canvas2D 경로와 같은 bitmap 경계 규칙을 쓴다. A4처럼 분수 CSS px인
      // 페이지를 절사하면 CanvasKit 쪽만 우·하단 한 줄이 빠져 backend parity가
      // 깨진다. 콘텐츠 좌표의 scale은 유지하고 bitmap 크기만 올림한다.
      canvas.width = Math.max(1, Math.ceil(pageInfo.width * renderScale));
      canvas.height = Math.max(1, Math.ceil(pageInfo.height * renderScale));
      const tree = this.wasm.getPageLayerTreeObject(pageIdx, this.renderProfile);
      renderStarted = true;
      const renderedCanvas = this.canvaskitRenderer.renderPage(
        tree,
        canvas,
        renderScale,
        pageInfo,
        key => this.wasm.getSourceFontBytes(key),
        this.wasm.documentGeneration,
      );
      this.canvaskitDiagnosticsByPage.set(pageIdx, this.canvaskitRenderer.diagnostics());
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return renderedCanvas;
    } catch (error) {
      this.canvaskitRenderer.recordRenderFailure(error, !renderStarted);
      this.canvaskitDiagnosticsByPage.set(pageIdx, this.canvaskitRenderer.diagnostics());
      console.error(`[PageRenderer] CanvasKit 페이지 렌더링 실패 (page=${pageIdx}):`, error);
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      if (!renderStarted) throw error;
      const replacement = parent && canvasChildIndex >= 0
        ? parent.children.item(canvasChildIndex)
        : null;
      if (canvas.parentElement !== parent && replacement instanceof HTMLCanvasElement) {
        return replacement;
      }
      return canvas;
    }
  }

  /**
   * Canvas 의 부모 컨테이너에 BehindText / InFrontOfText plane canvas 를 추가.
   *
   * - BehindText: flow Canvas 뒤
   * - InFrontOfText: flow Canvas 앞
   * - image/table/shape PaintOp 를 같은 PageLayerTree layer metadata 로 분류
   * - pointer-events: none — hit-test 는 flow Canvas 가 받음
   */
  private applyOverlays(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    dpr: number,
    context: PageRenderContext,
    layers: LayerPlaneSummary,
    reuseStaticFlow: boolean,
    flowImages: readonly FlowImagePaintOp[],
  ): LayerPlaneSummary {
    const parent = canvas.parentElement;
    if (!parent) return emptyLayerPlaneSummary();

    const allowReuse =
      context.reason === 'text-edit' && context.allowStaticOverlayReuse === true;

    if (!allowReuse) {
      // 페이지 단위 overlay 컨테이너를 Canvas 의 sibling 으로 관리.
      // data-rhwp-overlay-page 속성으로 식별, 페이지 재렌더링 시 갱신.
      this.removePageLayers(parent, pageIdx);
    }

    const safeDpr = dpr > 0 && Number.isFinite(dpr) ? dpr : 1;
    const cssWidth = canvas.width / safeDpr;
    const cssHeight = canvas.height / safeDpr;
    const top = canvas.style.top;
    const left = canvas.style.left;
    const transform = canvas.style.transform;

    if (reuseStaticFlow) {
      if (flowImages.length > 0) {
        const flowImageLayer = this.createOrReuseFlowImageLayer(
          pageIdx,
          canvas,
          renderScale / safeDpr,
          layers,
          allowReuse,
          flowImages,
        );
        this.applyPageLayerBox(flowImageLayer, top, left, transform, cssWidth, cssHeight);
        flowImageLayer.style.zIndex = '0';
        parent.insertBefore(flowImageLayer, canvas);
      } else {
        // RawSvg 차트/OLE는 첫 Canvas2D 렌더가 이미지 디코드를 시작해야 지연 재렌더에서 보인다.
        const flowStatic = this.createOrReuseFilteredCanvasLayer(
          pageIdx,
          canvas,
          renderScale,
          'flow-static',
          layers,
          allowReuse,
        );
        this.applyPageLayerBox(flowStatic, top, left, transform, cssWidth, cssHeight);
        flowStatic.style.zIndex = '0';
        flowStatic.style.background = 'var(--doc-paper)';
        parent.insertBefore(flowStatic, canvas);
      }
      canvas.style.background = 'transparent';
      canvas.style.zIndex = layers.hasFront ? '1' : '1';
    } else {
      this.removeOverlayLayer(parent, pageIdx, 'flow-static');
    }

    if (!layers.hasBehind && !layers.hasFront) {
      if (reuseStaticFlow) return layers;
      this.removePageLayers(parent, pageIdx);
      canvas.style.background = '';
      canvas.style.zIndex = '';
      return layers;
    }

    // BehindText 가 있는 페이지는 flow Canvas 를 투명 배경으로 두고,
    // 실제 pageBackground layer → BehindText → flow Canvas 순서로 합성한다.
    // Canvas 내부의 흰 배경은 WASM flow 렌더에서 생략된다.
    if (layers.hasBehind) {
      canvas.style.background = 'transparent';
      canvas.style.zIndex = '2';

      const background = this.createOrReuseFilteredCanvasLayer(
        pageIdx,
        canvas,
        renderScale,
        'background',
        layers,
        allowReuse,
      );
      this.applyPageLayerBox(background, top, left, transform, cssWidth, cssHeight);
      background.style.zIndex = '0';
      parent.insertBefore(background, canvas);
    } else {
      this.removeOverlayLayer(parent, pageIdx, 'background');
      this.removeOverlayLayer(parent, pageIdx, 'behind');
      canvas.style.background = reuseStaticFlow ? 'transparent' : '';
      canvas.style.zIndex = layers.hasFront || reuseStaticFlow ? '1' : '';
    }

    // BehindText overlay (Canvas 뒤). 이미지뿐 아니라 표/도형 PaintOp도 포함한다.
    if (layers.hasBehind) {
      const layer = this.createOrReuseFilteredCanvasLayer(
        pageIdx,
        canvas,
        renderScale,
        'behind',
        layers,
        allowReuse,
      );
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = '1';
      // Canvas 보다 먼저 들어가도록 prepend
      parent.insertBefore(layer, canvas);
    }

    // InFrontOfText overlay (Canvas 앞). 이미지뿐 아니라 글상자/도형 PaintOp도 포함한다.
    if (layers.hasFront) {
      const layer = this.createOrReuseFilteredCanvasLayer(
        pageIdx,
        canvas,
        renderScale,
        'front',
        layers,
        allowReuse,
      );
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = layers.hasBehind ? '3' : '2';  // Canvas 보다 앞
      parent.appendChild(layer);
    } else {
      this.removeOverlayLayer(parent, pageIdx, 'front');
    }
    return layers;
  }

  private createOrReuseFlowImageLayer(
    pageIdx: number,
    sourceCanvas: HTMLCanvasElement,
    displayScale: number,
    summary: LayerPlaneSummary,
    allowReuse: boolean,
    images: readonly FlowImagePaintOp[],
  ): HTMLElement {
    const key = this.buildStaticOverlayKey(pageIdx, sourceCanvas, displayScale, 'flow-static', summary);
    const selector = `[data-rhwp-flow-image-page="${pageIdx}"]`;
    const existing = sourceCanvas.parentElement?.querySelector<HTMLElement>(selector) ?? null;
    if (allowReuse && existing?.dataset.rhwpStaticOverlayKey === key) return existing;

    existing?.remove();
    const layer = document.createElement('div');
    layer.dataset.rhwpOverlay = `flow-images-${pageIdx}`;
    layer.dataset.rhwpOverlayPage = String(pageIdx);
    layer.dataset.rhwpFlowImagePage = String(pageIdx);
    layer.dataset.rhwpStaticOverlayKey = key;
    layer.style.pointerEvents = 'none';
    // [#5780] 쪽 배경색이 선언된 쪽은 종이색이 아니라 그 색을 실어야 한다 — DIV 갈래는
    // Background plane 을 canvas 로 싣지 못하므로 여기서 단색 배경을 대신 진다
    // (그라데이션/이미지 배경은 usesDomFlowImages 게이트가 canvas 갈래로 폴백).
    layer.style.background = summary.pageBackgroundCss ?? 'var(--doc-paper)';

    for (const image of images) {
      const visibleBbox = visibleFlowImageBbox(image);
      if (!visibleBbox) continue;

      // clip이 실제 그림보다 작을 때만 별도 wrapper를 둔다. 일반 그림은 기존 DOM
      // 경로를 그대로 사용해 정적 이미지 분리의 비용 이점을 유지한다.
      const needsClipWrapper = image.clip !== null && (
        visibleBbox.x !== image.bbox.x ||
        visibleBbox.y !== image.bbox.y ||
        visibleBbox.width !== image.bbox.width ||
        visibleBbox.height !== image.bbox.height ||
        image.rotation !== 0
      );
      const clipHost = needsClipWrapper ? document.createElement('div') : layer;
      if (needsClipWrapper) {
        clipHost.style.position = 'absolute';
        clipHost.style.left = `${visibleBbox.x * displayScale}px`;
        clipHost.style.top = `${visibleBbox.y * displayScale}px`;
        clipHost.style.width = `${visibleBbox.width * displayScale}px`;
        clipHost.style.height = `${visibleBbox.height * displayScale}px`;
        clipHost.style.overflow = 'hidden';
        clipHost.style.pointerEvents = 'none';
      }

      // [#6099] image.bbox 는 **회전 후 외접 상자**다. 프레임을 그 크기로 만들고
      // rotate 를 다시 걸면 이중 회전이 되어 90° 그림이 세로로 선 채 clip 에
      // 잘린다(2197981 스캔 서식: 한글 712×506 vs 503×452 정사각형). SVG/캔버스
      // 페인터(`effective_image_bbox`)와 같은 규약 — 90/270° 는 프레임을 회전 **전**
      // 치수(가로세로 swap)로 만들고 같은 중심에서 rotate 해 외접 상자를 복원한다.
      const quarterTurned = ((image.rotation % 180) + 180) % 180 === 90;
      const frameWidth = quarterTurned ? image.bbox.height : image.bbox.width;
      const frameHeight = quarterTurned ? image.bbox.width : image.bbox.height;
      const frameX = image.bbox.x + (image.bbox.width - frameWidth) / 2;
      const frameY = image.bbox.y + (image.bbox.height - frameHeight) / 2;
      const frame = document.createElement('div');
      frame.style.position = 'absolute';
      frame.style.left = `${(frameX - (needsClipWrapper ? visibleBbox.x : 0)) * displayScale}px`;
      frame.style.top = `${(frameY - (needsClipWrapper ? visibleBbox.y : 0)) * displayScale}px`;
      frame.style.width = `${frameWidth * displayScale}px`;
      frame.style.height = `${frameHeight * displayScale}px`;
      frame.style.overflow = 'hidden';
      frame.style.pointerEvents = 'none';
      const scaleX = image.horzFlip ? -1 : 1;
      const scaleY = image.vertFlip ? -1 : 1;
      frame.style.transform = `rotate(${image.rotation}deg) scale(${scaleX}, ${scaleY})`;
      frame.style.transformOrigin = 'center';

      const element = new Image();
      element.alt = '';
      // data URL(전체 트리 경로) 또는 신원 키별 object URL(좁은 질의 경로) — #3315.
      element.src = image.src;
      element.style.position = 'absolute';
      element.style.pointerEvents = 'none';
      // 그림 효과(회색조/흑백/밝기/명암) — WASM canvas 경로(render_image)와 달리
      // DOM flow-image 경로는 필터가 누락돼 원본 컬러로 렌더되던 문제를 고친다.
      if (image.filter) element.style.filter = image.filter;
      const applyCrop = () =>
        applyFlowImageCrop(element, image, displayScale, frameWidth, frameHeight);
      element.addEventListener('load', applyCrop, { once: true });
      applyCrop();
      frame.appendChild(element);
      clipHost.appendChild(frame);
      if (needsClipWrapper) layer.appendChild(clipHost);
    }
    return layer;
  }

  private createOrReuseFilteredCanvasLayer(
    pageIdx: number,
    sourceCanvas: HTMLCanvasElement,
    renderScale: number,
    layerKind: StaticCanvasLayerKind,
    summary: LayerPlaneSummary,
    allowReuse: boolean,
    renderImmediately = true,
  ): HTMLCanvasElement {
    const key = this.buildStaticOverlayKey(pageIdx, sourceCanvas, renderScale, layerKind, summary);
    const reusableLayer = this.findOverlayLayer(sourceCanvas.parentElement, pageIdx, layerKind);
    if (
      allowReuse &&
      reusableLayer?.dataset.rhwpStaticOverlayKey === key &&
      reusableLayer.width === sourceCanvas.width &&
      reusableLayer.height === sourceCanvas.height
    ) {
      return reusableLayer;
    }

    reusableLayer?.remove();
    const layer = this.createFilteredCanvasLayer(
      pageIdx,
      sourceCanvas,
      renderScale,
      layerKind,
      renderImmediately,
    );
    layer.dataset.rhwpOverlay = `${layerKind}-${pageIdx}`;
    layer.dataset.rhwpOverlayPage = String(pageIdx);
    layer.dataset.rhwpStaticOverlayKey = key;
    return layer;
  }

  private createFilteredCanvasLayer(
    pageIdx: number,
    sourceCanvas: HTMLCanvasElement,
    renderScale: number,
    layerKind: StaticCanvasLayerKind,
    renderImmediately = true,
  ): HTMLCanvasElement {
    const layer = document.createElement('canvas');
    layer.width = sourceCanvas.width;
    layer.height = sourceCanvas.height;
    layer.dataset.rhwpLayerKind = layerKind;
    layer.style.pointerEvents = 'none';
    // Overlay canvas elements inherit #scroll-content canvas background unless
    // this is explicit. A front layer with an opaque page background hides all
    // lower background/behind layers.
    layer.style.background = 'transparent';
    if (renderImmediately) {
      this.wasm.renderPageToCanvasFiltered(
        pageIdx,
        layer,
        renderScale,
        layerKind,
        this.renderProfile,
      );
    }
    return layer;
  }

  private applyPageLayerBox(
    layer: HTMLElement,
    top: string,
    left: string,
    transform: string,
    cssWidth: number,
    cssHeight: number,
  ): void {
    layer.style.position = 'absolute';
    layer.style.top = top;
    layer.style.left = left;
    layer.style.transform = transform;
    layer.style.width = `${cssWidth}px`;
    layer.style.height = `${cssHeight}px`;
    layer.style.overflow = 'hidden';
    layer.style.pointerEvents = 'none';
  }

  removePageLayers(parent: HTMLElement, pageIdx: number): void {
    this.layerSummaryCache.delete(pageIdx);
    parent.querySelectorAll(
      `[data-rhwp-overlay-page="${pageIdx}"],` +
      `[data-rhwp-overlay="background-${pageIdx}"],` +
      `[data-rhwp-overlay="behind-${pageIdx}"],` +
      `[data-rhwp-overlay="front-${pageIdx}"]`,
    ).forEach((el) => el.remove());
  }

  private findOverlayLayer(
    parent: HTMLElement | null,
    pageIdx: number,
    layerKind: StaticCanvasLayerKind,
  ): HTMLCanvasElement | null {
    return parent?.querySelector<HTMLCanvasElement>(
      `[data-rhwp-overlay-page="${pageIdx}"][data-rhwp-layer-kind="${layerKind}"]`,
    ) ?? null;
  }

  private removeOverlayLayer(parent: HTMLElement, pageIdx: number, layerKind: StaticCanvasLayerKind): void {
    this.findOverlayLayer(parent, pageIdx, layerKind)?.remove();
  }

  private buildStaticOverlayKey(
    pageIdx: number,
    sourceCanvas: HTMLCanvasElement,
    renderScale: number,
    layerKind: StaticCanvasLayerKind,
    summary: LayerPlaneSummary,
  ): string {
    return [
      `page=${pageIdx}`,
      `scale=${renderScale}`,
      `width=${sourceCanvas.width}`,
      `height=${sourceCanvas.height}`,
      `layer=${layerKind}`,
      `profile=${this.renderProfile}`,
      `backend=${this.backend}`,
      `summary=${summary.signature}`,
    ].join('|');
  }

  removeAllPageLayers(parent: HTMLElement): void {
    this.layerSummaryCache.clear();
    parent.querySelectorAll(
      '[data-rhwp-overlay-page],' +
      '[data-rhwp-overlay^="background-"],' +
      '[data-rhwp-overlay^="behind-"],' +
      '[data-rhwp-overlay^="front-"]',
    ).forEach((el) => el.remove());
  }

  /**
   * 페이지를 본문 layer (flow) 만 Canvas 에 렌더링한다 (Task #516, Stage 5.2).
   * BehindText / InFrontOfText plane 은 제외 — overlay canvas 로 별도 표시.
   */
  renderPageFlow(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    scale: number,
    displayScale = scale,
  ): void {
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'flow', this.renderProfile);
    this.drawMarginGuides(pageIdx, canvas, scale, undefined, displayScale);
    this.scheduleReRender(pageIdx, canvas, scale, 0, 0, {
      retrySignature: 'flow-only',
      reuseStaticFlow: false,
      reuseStaticOverlay: false,
      displayScale,
    });
  }

  private shouldSplitStaticFlow(layers: LayerPlaneSummary): boolean {
    return (
      !layers.hasBehind &&
      // [#5763] 그림 밑에 깔린 불투명 채우기는 canvas(flow-dynamic) 에 남아 아래 평면의
      // 그림을 덮는다. 그런 페이지는 한 평면에 순서대로 그린다 — 분리 이득보다 그림
      // 소실이 크다 (156550355 문서 3·4·11쪽이 빈 흰 상자로 보이던 원인).
      !layers.flowStaticOccluded &&
      layers.flowStaticCount > 0 &&
      this.flowSplitSupported !== false
    );
  }

  /**
   * 본문 그림의 DOM 배치 정보.
   *
   * [#3315] 좁은 질의를 먼저 쓴다. 종전에는 여기서 전체 레이어 트리 JSON 을 받았는데, 그림
   * 1장이 4.77MB 면 그 JSON 이 6.6MB 라 편집마다 경계 복사와 `JSON.parse` 로 20 ms 가 나갔다.
   * 좁은 질의는 같은 문서에서 309 bytes 다.
   *
   * 캐시로는 풀 수 없다 — 본문이 흐르면 그림이 그대로여도 bbox 가 움직인다. 그래서 질의를
   * 좁히고, **바이트만** 신원 키별 object URL 로 캐시한다.
   *
   * 좁은 질의를 못 쓰는 경우(구형 WASM·합성 그림이 섞인 페이지·낡은 키)에는 종전 경로로
   * 되돌아간다. 조용히 그림을 빠뜨리는 것보다 느린 게 낫다.
   */
  private getFlowImagePaintOps(pageIdx: number): FlowImagePaintOp[] {
    const narrowJson = this.wasm.getPageFlowImageOps(pageIdx);
    if (narrowJson !== null) {
      const images = flowImageOpsFromNarrowQuery(narrowJson, (key, mime) =>
        this.flowImageUrls.urlFor(key, mime, (k) => this.wasm.getSourceImageBytes(k)),
      );
      if (images !== null) return images;
    }

    let json: string;
    try {
      json = this.wasm.getPageLayerTree(pageIdx);
    } catch {
      return [];
    }
    try {
      const root = JSON.parse(json)?.root;
      return collectFlowImagePaintOps(
        root,
        (op, layer) => op.type === 'image' && layerReplayPlane(op, layer) === 'flow',
      );
    } catch {
      return [];
    }
  }

  private renderFlowCanvas(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    preferStaticFlow: boolean,
  ): boolean {
    if (!preferStaticFlow) {
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow', this.renderProfile);
      return false;
    }
    try {
      this.wasm.renderPageToCanvasFiltered(
        pageIdx,
        canvas,
        renderScale,
        'flow-dynamic',
        this.renderProfile,
      );
      this.flowSplitSupported = true;
      return true;
    } catch (error) {
      this.flowSplitSupported = false;
      this.surfaceLayerCountCache.clear();
      console.warn('[PageRenderer] flow-dynamic 렌더 미지원, 기존 flow 렌더로 fallback:', error);
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow', this.renderProfile);
      return false;
    }
  }

  /**
   * [#3137 Stage 4] stable same-line edit가 제공한 좁은 dirty rect만 다시 재생한다.
   *
   * 이미지/RawSvg는 비동기 decode와 별도 static layer 계약이 있으므로 보수적으로
   * full repaint에 남긴다. 실패하면 caller가 기존 renderPage 경로를 그대로 수행한다.
   */
  private renderFocusedPagePatch(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    displayScale: number,
    context: PageRenderContext,
  ): boolean {
    const patch = context.focusedPagePatch;
    if (!patch || !canvas.parentElement) return false;

    const layers = this.getLayerPlaneSummary(pageIdx, canvas, renderScale, context);
    if (layers.imageCount > 0 || layers.rawSvgCount > 0) return false;

    try {
      this.wasm.renderPagePatchToCanvasFiltered(
        pageIdx,
        canvas,
        renderScale,
        'flow',
        patch,
        this.renderProfile,
      );
      this.drawMarginGuides(pageIdx, canvas, renderScale, patch, displayScale);
      this.rememberLayerPlaneSummary(pageIdx, canvas, renderScale, layers);
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return true;
    } catch (error) {
      console.warn('[PageRenderer] focused page patch 실패, 전체 repaint로 fallback:', error);
      return false;
    }
  }

  private getLayerPlaneSummary(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    context: PageRenderContext,
  ): LayerPlaneSummary {
    const cacheKey = this.buildLayerSummaryCacheKey(pageIdx, canvas, renderScale);
    if (context.reason === 'text-edit' && context.allowStaticOverlayReuse === true) {
      const cached = this.layerSummaryCache.get(pageIdx);
      if (cached?.key === cacheKey) return { ...cached.summary };
    }

    const overlaySummary = this.getLayerPlaneSummaryFromOverlayImages(pageIdx);
    if (overlaySummary) {
      this.layerSummaryCache.set(pageIdx, { key: cacheKey, summary: overlaySummary });
      return overlaySummary;
    }
    const treeSummary = this.getLayerPlaneSummaryFromTree(pageIdx);
    this.layerSummaryCache.set(pageIdx, { key: cacheKey, summary: treeSummary });
    return treeSummary;
  }

  private rememberLayerPlaneSummary(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    summary: LayerPlaneSummary,
  ): void {
    this.layerSummaryCache.set(pageIdx, {
      key: this.buildLayerSummaryCacheKey(pageIdx, canvas, renderScale),
      summary: { ...summary },
    });
  }

  private buildLayerSummaryCacheKey(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
  ): string {
    return [
      `page=${pageIdx}`,
      `scale=${renderScale}`,
      `width=${canvas.width}`,
      `height=${canvas.height}`,
      `profile=${this.renderProfile}`,
      `backend=${this.backend}`,
    ].join('|');
  }

  private getLayerPlaneSummaryFromOverlayImages(pageIdx: number): LayerPlaneSummary | null {
    let json: string;
    try {
      json = this.wasm.getPageOverlayImages(pageIdx);
    } catch {
      return null;
    }
    if (!json || json.trim()[0] !== '{') return null;
    try {
      const wrapper = JSON.parse(json);
      if (typeof wrapper?.hasBehind !== 'boolean' || typeof wrapper?.hasFront !== 'boolean') {
        return null;
      }
      const behind = Array.isArray(wrapper.behind) ? wrapper.behind : [];
      const front = Array.isArray(wrapper.front) ? wrapper.front : [];
      const imageCount = finiteCount(wrapper.imageCount);
      const rawSvgCount = finiteCount(wrapper.rawSvgCount);
      const flowImageCount =
        wrapper.flowImageCount === undefined
          ? Math.max(0, imageCount - behind.length - front.length)
          : finiteCount(wrapper.flowImageCount);
      const flowRawSvgCount =
        wrapper.flowRawSvgCount === undefined
          ? rawSvgCount
          : finiteCount(wrapper.flowRawSvgCount);
      const flowStaticCount = flowImageCount + flowRawSvgCount;
      // [#5763] 구형 WASM 은 이 필드를 안 낸다 — 그때는 종전대로 분리를 허용한다.
      const flowStaticOccluded = wrapper.flowStaticOccluded === true;
      // [#5780] 쪽 배경 요약 — 구형 WASM 은 필드가 없다(null/false 폴백).
      const pageBackgroundCss =
        typeof wrapper.pageBackgroundCss === 'string' ? wrapper.pageBackgroundCss : null;
      const pageBackgroundComplex = wrapper.pageBackgroundComplex === true;
      return {
        hasBehind: wrapper.hasBehind,
        hasFront: wrapper.hasFront,
        imageCount,
        rawSvgCount,
        flowImageCount,
        flowRawSvgCount,
        flowStaticCount,
        flowStaticOccluded,
        pageBackgroundCss,
        pageBackgroundComplex,
        signature: `overlay:${wrapper.hasBehind ? 1 : 0}:${wrapper.hasFront ? 1 : 0}:${imageCount}:${rawSvgCount}:${flowImageCount}:${flowRawSvgCount}:${flowStaticOccluded ? 1 : 0}:${pageBackgroundCss ?? ''}:${pageBackgroundComplex ? 1 : 0}:${json.length}`,
      };
    } catch (e) {
      console.warn('[PageRenderer] OverlayImageSummary JSON parse 실패:', e);
      return null;
    }
  }

  private getLayerPlaneSummaryFromTree(pageIdx: number): LayerPlaneSummary {
    const summary: LayerPlaneSummary = emptyLayerPlaneSummary();
    let json: string;
    try {
      json = this.wasm.getPageLayerTree(pageIdx);
    } catch (e) {
      console.warn('[PageRenderer] PageLayerTree JSON 조회 실패:', e);
      return summary;
    }
    try {
      const wrapper = JSON.parse(json);
      const root = wrapper?.root;
      if (root) {
        collectLayerPlaneSummary(root, summary, null, { opaqueFlowFills: [] });
        summary.flowStaticCount = summary.flowImageCount + summary.flowRawSvgCount;
        summary.signature = `tree:${summary.hasBehind ? 1 : 0}:${summary.hasFront ? 1 : 0}:${summary.imageCount}:${summary.rawSvgCount}:${summary.flowImageCount}:${summary.flowRawSvgCount}:${summary.flowStaticOccluded ? 1 : 0}:${summary.pageBackgroundCss ?? ''}:${summary.pageBackgroundComplex ? 1 : 0}`;
      }
    } catch (e) {
      console.warn('[PageRenderer] PageLayerTree JSON parse 실패:', e);
    }
    return summary;
  }

  private drawMarginGuides(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    scale: number,
    clip?: PageSpaceRect,
    displayScale = 1,
  ): void {
    drawPageMarginGuides(
      this.wasm.getPageInfo(pageIdx),
      canvas,
      scale,
      clip,
      this.pageMarginGuideEdges,
      displayScale,
    );
  }

  /**
   * 비동기 이미지 로드 대응: data URL 이미지가 첫 렌더링 시
   * 아직 디코딩되지 않았을 수 있으므로 점진적 재렌더링한다.
   *
   * decode 완료 후 한 번 다시 그린다. base64를 직접 추출할 수 없는 경우에는
   * fallback 시점에 한 번만 다시 그려 이미지 누락 안전망을 유지한다.
   */
  private scheduleReRender(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    imageCount: number,
    rawSvgCount: number,
    policy: ReRenderPolicy,
  ): void {
    if (imageCount <= 0) {
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return;
    }
    const retryKey = this.buildImageRetryKey(pageIdx, imageCount, rawSvgCount, policy);
    if (retryKey !== null && this.imageRetryCounts.get(pageIdx) === retryKey) return;

    this.cancelReRender(pageIdx);
    if (retryKey === null) this.imageRetryCounts.delete(pageIdx);
    else this.imageRetryCounts.set(pageIdx, retryKey);
    const prefetchRequestToken = ++this.nextPrefetchRequestToken;
    this.prefetchRequestTokens.set(pageIdx, prefetchRequestToken);

    const job: ReRenderJob = {
      fallbackTimer: 0 as unknown as ReturnType<typeof setTimeout>,
      earlyRawSvgTimers: [],
      completed: false,
    };
    const finish = () => {
      if (job.completed || this.reRenderJobs.get(pageIdx) !== job) return;
      job.completed = true;
      clearTimeout(job.fallbackTimer);
      for (const timer of job.earlyRawSvgTimers) clearTimeout(timer);
      this.reRenderJobs.delete(pageIdx);
      if (canvas.parentElement) {
        this.reRenderPageCanvases(pageIdx, canvas, renderScale, policy);
      }
    };
    job.fallbackTimer = setTimeout(finish, IMAGE_RE_RENDER_FALLBACK_DELAY_MS);
    this.reRenderJobs.set(pageIdx, job);

    if (rawSvgCount > 0) {
      for (const delay of RAW_SVG_EARLY_RE_RENDER_DELAYS_MS) {
        const timer = setTimeout(() => {
          if (job.completed || this.reRenderJobs.get(pageIdx) !== job) return;
          if (canvas.parentElement) {
            this.reRenderPageCanvases(pageIdx, canvas, renderScale, policy);
          }
        }, delay);
        job.earlyRawSvgTimers.push(timer);
      }
    }

    // 자체 prefetch로 실제 decode를 마친 경우에만 fallback보다 먼저 다시 그린다.
    queueMicrotask(() => {
      this.prefetchLayerImages(pageIdx, rawSvgCount, prefetchRequestToken)
        .then((decoded) => {
          if (decoded) finish();
        })
        .catch(() => {});
    });
  }

  /**
   * 재시도 상태를 재사용할지 판정하는 키 (Task #3315).
   *
   * 종전 키는 `imageCount:rawSvgCount:overlaySignature` 였다. 그 재료는 **그림 내용을 보지
   * 못한다** — 밝기/대비를 켜면 워터마크 bake 경로로 바이트가 바뀌는데 개수는 그대로라 키가
   * 불변이고, 그걸 "변화 없음"으로 읽으면 첫 draw 에서 디코드가 안 끝난 그림이 빈 채로 남는다.
   *
   * 그 위험을 막으려고 `refreshPages` 가 매 편집에 `resetImageRetryState()` 로 전부 비웠는데,
   * 그러면 **페이지마다 재렌더가 한 번 더 돈다** — prefetch 가 서명으로 건너뛰어 `finish()` 가
   * 즉시 불리고 `reRenderPageCanvases` 가 다시 그린다.
   *
   * 비우는 대신 키가 봐야 할 것을 직접 들게 한다.
   *
   * - `getPageSourceImageKeys` — `bin:{epoch}:{id}:{variant}` 목록. 밝기/대비로 bake 여부가
   *   바뀌면 variant 가 `src` ↔ `wmpng` 로 갈리므로 **내용 변화가 키에 나타난다.**
   * - 문서 신원(digest·generation) — 키의 세대는 문서마다 0 에서 시작해 서로 충돌한다
   *   (`bin:0:1:src`). 문서 경계를 키가 들지 않으면 새 문서가 옛 재시도 상태를 재사용한다.
   *   `prefetchedImageSignatures`·`FlowImageUrlCache` 와 같은 이유·같은 방식이다.
   * - RawSvg — compact 그림 키에는 포함되지 않고, 브라우저의 SVG decode 캐시는 별도로
   *   비워질 수 있다. 개수만으로는 재렌더 준비 상태를 증명할 수 없으므로 이 페이지는
   *   재사용하지 않는다.
   *
   * 판정 재료가 없으면(`null`) **재사용하지 않는다** — 구형 WASM, 키를 낼 수 없는 합성 그림이
   * 섞인 페이지(`cacheable:false`), 문서 신원 미상. 안전망을 없애는 쪽이 아니라 이미 끝난 일을
   * 되풀이하지 않는 쪽으로만 작동해야 한다.
   */
  private buildImageRetryKey(
    pageIdx: number,
    imageCount: number,
    rawSvgCount: number,
    policy: ReRenderPolicy,
  ): string | null {
    // RawSvg 차트/OLE는 첫 paint 뒤 비동기 decode가 끝나야 다시 그려진다. source-image key는
    // Image 노드만 대상으로 하고 브라우저 IMAGE_CACHE의 eviction도 관찰하지 못하므로, 같은
    // 개수라는 이유로 timer/fallback을 건너뛰면 공백이 고착될 수 있다 (#1456).
    if (rawSvgCount > 0) return null;
    const imageKeys = cacheableImageKeySignature(this.wasm.getPageSourceImageKeys(pageIdx));
    const documentDigest = this.wasm.documentDigest;
    if (imageKeys === null || documentDigest === null) return null;
    return [
      documentDigest,
      this.wasm.documentGeneration,
      imageKeys,
      imageCount,
      rawSvgCount,
      policy.retrySignature,
    ].join('|');
  }

  private reRenderPageCanvases(
    pageIdx: number,
    flowCanvas: HTMLCanvasElement,
    renderScale: number,
    policy: ReRenderPolicy,
  ): void {
    const parent = flowCanvas.parentElement;
    if (!parent) return;

    let renderedStaticFlow = false;
    if (policy.reuseStaticFlow) {
      const flowStatic = this.findOverlayLayer(parent, pageIdx, 'flow-static');
      if (flowStatic) {
        flowStatic.width = flowCanvas.width;
        flowStatic.height = flowCanvas.height;
        try {
          this.wasm.renderPageToCanvasFiltered(
            pageIdx,
            flowStatic,
            renderScale,
            'flow-static',
            this.renderProfile,
          );
          renderedStaticFlow = true;
        } catch (error) {
          this.flowSplitSupported = false;
          this.surfaceLayerCountCache.clear();
          flowStatic.remove();
          console.warn('[PageRenderer] flow-static 지연 재렌더 실패, 기존 flow 재렌더로 fallback:', error);
        }
      }
    }

    if (!renderedStaticFlow) {
      this.wasm.renderPageToCanvasFiltered(
        pageIdx,
        flowCanvas,
        renderScale,
        'flow',
        this.renderProfile,
      );
      this.drawMarginGuides(pageIdx, flowCanvas, renderScale, undefined, policy.displayScale);
    }

    if (policy.reuseStaticOverlay) return;

    parent.querySelectorAll<HTMLCanvasElement>(
      `[data-rhwp-overlay-page="${pageIdx}"][data-rhwp-layer-kind]`,
    ).forEach((layerCanvas) => {
      const kind = layerCanvas.dataset.rhwpLayerKind;
      if (kind === 'background' || kind === 'behind' || kind === 'front') {
        layerCanvas.width = flowCanvas.width;
        layerCanvas.height = flowCanvas.height;
        this.wasm.renderPageToCanvasFiltered(
          pageIdx,
          layerCanvas,
          renderScale,
          kind,
          this.renderProfile,
        );
      }
    });
  }

  /**
   * 페이지의 image base64 데이터를
   * 자체 prefetch 하여 모든 이미지가 브라우저에 디코드 완료될 때까지 대기.
   * Task #1154 — IMAGE_CACHE 의 비동기 디코드 누락 안전망.
   */
  private async prefetchLayerImages(
    pageIdx: number,
    rawSvgCount: number,
    prefetchRequestToken: number,
  ): Promise<boolean> {
    // [#3315] 그림이 그대로면 다시 디코드시킬 것이 없다. 서명 조회는 수백 바이트인데
    // 아래의 전체 레이어 트리 조회는 그림 1장에 수 MB 라, 편집마다 되풀이할 값이 아니다.
    const imageKeys = cacheableImageKeySignature(this.wasm.getPageSourceImageKeys(pageIdx));
    const documentDigest = this.wasm.documentDigest;
    const documentGeneration = this.wasm.documentGeneration;
    if (
      shouldSkipImagePrefetch(
        this.prefetchedImageSignatures.get(pageIdx),
        imageKeys,
        documentDigest,
        documentGeneration,
        rawSvgCount,
      )
    ) {
      return true;
    }

    let json: string;
    try {
      json = this.wasm.getPageLayerTree(pageIdx);
    } catch {
      return false;
    }
    const tasks: Promise<boolean>[] = [];
    const seen = new Set<string>();
    const enqueue = (dataUrl: string) => {
      if (seen.has(dataUrl)) return;
      seen.add(dataUrl);
      tasks.push(
        new Promise<boolean>((resolve) => {
          const img = new Image();
          let settled = false;
          const finish = (ok: boolean) => {
            if (settled) return;
            settled = true;
            resolve(ok);
          };
          const supportsDecode = typeof img.decode === 'function';
          img.onload = () => {
            if (!supportsDecode) finish(true);
          };
          img.onerror = () => finish(false);
          img.src = dataUrl;
          // decode() 이 더 정확하지만 일부 브라우저 미지원
          if (supportsDecode) {
            try {
              img.decode().then(() => finish(true)).catch(() => finish(false));
            } catch {
              finish(false);
            }
          }
        }),
      );
    };
    let layerTree: unknown = null;
    try {
      layerTree = JSON.parse(json);
      const imageDataUrls: string[] = [];
      collectImagePrefetchDataUrls(layerTree, imageDataUrls);
      for (const dataUrl of imageDataUrls) enqueue(dataUrl);
    } catch {
      // 유효한 PageLayerTree JSON이 아니면 완료 서명을 기록하지 않고 다음 렌더에서 재시도한다.
    }
    // rawSvg 항목 (OLE/차트 미리보기) 의 embedded data URL 추출.
    // svg 필드는 JSON 인코딩 문자열이며 내부에 data:image/MIME;base64,... 가 등장한다.
    // rawSvg 의 wrap 은 항상 flow 이므로 overlay 필터링 불필요.
    const dataUrlRe = /data:(image\/[A-Za-z0-9.+-]+);base64,([A-Za-z0-9+/=]+)/g;
    let d: RegExpExecArray | null;
    while ((d = dataUrlRe.exec(json)) !== null) {
      enqueue(`data:${d[1]};base64,${d[2]}`);
    }
    // 벡터 rawSvg(차트/OLE 미리보기)는 내부 raster data URL 이 없어 위 정규식으로
    // 잡히지 않는다. web_canvas.render_raw_svg 와 동일하게 조각을 wrap 한 SVG data URL
    // 을 프리페치해야 비동기 로드 완료 신호를 얻어 지연 재렌더(finish)가 발동하고,
    // WASM 캐시의 SVG 이미지가 로드 완료 상태로 flow-static overlay 에 그려진다.
    const hadRawSvg = json.includes('"type":"rawSvg"');
    if (hadRawSvg && layerTree !== null) {
      const vectorRawSvgUrls: string[] = [];
      collectVectorRawSvgDataUrls(layerTree, vectorRawSvgUrls);
      for (const dataUrl of vectorRawSvgUrls) enqueue(dataUrl);
    }
    return completeImagePrefetch(tasks, () => (
      this.prefetchRequestTokens.get(pageIdx) === prefetchRequestToken
      && this.wasm.documentGeneration === documentGeneration
    ), () => {
      if (imageKeys !== null && documentDigest !== null) {
        this.prefetchedImageSignatures.set(pageIdx, {
          documentDigest,
          documentGeneration,
          imageKeys,
          hadRawSvg,
        });
      }
    });
  }

  /** 특정 페이지의 지연 재렌더링을 취소한다 */
  cancelReRender(pageIdx: number): void {
    this.prefetchRequestTokens.delete(pageIdx);
    const job = this.reRenderJobs.get(pageIdx);
    if (job) {
      job.completed = true;
      clearTimeout(job.fallbackTimer);
      for (const timer of job.earlyRawSvgTimers) clearTimeout(timer);
      this.reRenderJobs.delete(pageIdx);
    }
  }

  /** 모든 지연 재렌더링을 취소한다 */
  cancelAll(): void {
    for (const job of this.reRenderJobs.values()) {
      job.completed = true;
      clearTimeout(job.fallbackTimer);
      for (const timer of job.earlyRawSvgTimers) clearTimeout(timer);
    }
    this.reRenderJobs.clear();
    this.prefetchRequestTokens.clear();
  }

  /**
   * 렌더된 페이지를 통째로 버릴 때 파생 상태를 정리한다.
   *
   * [#3315] `imageRetryCounts` 는 **여기서 비우지 않는다.** 이 메서드는 편집마다
   * (`refreshPages` → `releaseAllRenderedPages`) 불리므로, 비우면 페이지마다 재렌더가 한 번 더
   * 돈다 — prefetch 가 서명으로 건너뛰어 `finish()` 가 즉시 불리고 다시 그린다.
   *
   * 문서 경계와 그림 내용 변화는 재시도 키가 직접 든다(`buildImageRetryKey`). 그래서 **편집
   * 시점에** 비워 줄 필요가 없다 — 맞춰야 하는 계약이 #3648·P1 에서 깨진 그 계약이다.
   *
   * 다만 "잘못 맞지 않는다"와 "사라진다"는 다르다. 다시 읽히지 않을 항목을 거두는 자리는
   * 문서 경계인 `beginDocument` 다.
   */
  resetImageRetryState(): void {
    this.prefetchRequestTokens.clear();
    this.layerSummaryCache.clear();
    this.surfaceLayerCountCache.clear();
    this.canvaskitDiagnosticsByPage.clear();
  }

  dispose(): void {
    this.cancelAll();
    this.imageRetryCounts.clear();
    this.prefetchedImageSignatures.clear();
    this.prefetchRequestTokens.clear();
    this.layerSummaryCache.clear();
    this.surfaceLayerCountCache.clear();
    this.canvaskitDiagnosticsByPage.clear();
    this.flowImageUrls.releaseAll();
    this.documentScope = null;
    this.canvaskitRenderer = null;
  }
}

function emptyLayerPlaneSummary(): LayerPlaneSummary {
  return {
    hasBehind: false,
    hasFront: false,
    imageCount: 0,
    rawSvgCount: 0,
    flowImageCount: 0,
    flowRawSvgCount: 0,
    flowStaticCount: 0,
    flowStaticOccluded: false,
    pageBackgroundCss: null,
    pageBackgroundComplex: false,
    signature: 'empty',
  };
}

function finiteCount(value: unknown): number {
  const n = Number(value);
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}

/**
 * [#5763] paint 순서대로 훑으며 "그림 밑에 깔린 불투명 flow 채우기" 를 모은다.
 *
 * Rust `FlowStaticOcclusion` 과 같은 규칙이다 — 이 경로는 좁은 질의(overlay summary)를 못 쓰는
 * 구형 WASM·예외 상황의 폴백이라 판정이 갈리면 안 된다.
 */
interface FlowOcclusionScan {
  opaqueFlowFills: Array<{ x: number; y: number; width: number; height: number }>;
}

function opaqueFlowFillBbox(op: any): FlowOcclusionScan['opaqueFlowFills'][number] | null {
  if (op.type !== 'rectangle' && op.type !== 'ellipse' && op.type !== 'path') return null;
  const style = op.style;
  if (!style || typeof style !== 'object') return null;
  if (typeof style.opacity === 'number' && style.opacity < 1) return null;
  const filled = style.fillColor != null || style.pattern != null || op.gradient != null;
  if (!filled) return null;
  const b = op.bbox;
  if (!b || typeof b.x !== 'number' || typeof b.width !== 'number') return null;
  return { x: b.x, y: b.y, width: b.width, height: b.height };
}

function bboxIntersects(a: { x: number; y: number; width: number; height: number }, b: any): boolean {
  if (!b || typeof b.x !== 'number' || typeof b.width !== 'number') return false;
  return a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;
}

function collectLayerPlaneSummary(
  node: any,
  summary: LayerPlaneSummary,
  inheritedLayer: any,
  scan: FlowOcclusionScan,
): void {
  if (!node || typeof node !== 'object') return;
  const activeLayer = node.layer ?? inheritedLayer;
  if (Array.isArray(node.ops)) {
    for (const op of node.ops) {
      if (!op || typeof op !== 'object') continue;
      const plane = layerReplayPlane(op, activeLayer);
      if (op.type === 'pageBackground') {
        // [#5780] Rust overlay 요약과 같은 규칙 — 첫 pageBackground 만 취한다.
        if (summary.pageBackgroundCss === null && !summary.pageBackgroundComplex) {
          summary.pageBackgroundCss =
            typeof op.backgroundColor === 'string' ? op.backgroundColor : null;
          summary.pageBackgroundComplex = op.gradient != null || op.image != null;
        }
      }
      if (op.type === 'image') {
        summary.imageCount += 1;
        if (plane === 'flow') {
          summary.flowImageCount += 1;
        }
      } else if (op.type === 'rawSvg') {
        // 차트/OLE 미리보기. web_canvas draw_image 비동기 디코드 경로를 타므로
        // image 와 동일하게 재렌더 트리거 대상에 포함한다(#1456).
        summary.rawSvgCount += 1;
        if (plane === 'flow') {
          summary.flowRawSvgCount += 1;
        }
      }
      if (plane === 'flow') {
        if (op.type === 'image' || op.type === 'rawSvg') {
          if (scan.opaqueFlowFills.some((fill) => bboxIntersects(fill, op.bbox))) {
            summary.flowStaticOccluded = true;
          }
        } else {
          const fill = opaqueFlowFillBbox(op);
          if (fill) scan.opaqueFlowFills.push(fill);
        }
      }
      if (plane === 'behindText') {
        summary.hasBehind = true;
      } else if (plane === 'inFrontOfText') {
        summary.hasFront = true;
      }
    }
  }
  if (Array.isArray(node.children)) {
    for (const child of node.children) {
      collectLayerPlaneSummary(child, summary, activeLayer, scan);
    }
  }
  if (node.child) {
    collectLayerPlaneSummary(node.child, summary, activeLayer, scan);
  }
}

// #2318: 로컬 중복 구현을 제거하고 공유 분류기(replay-plane.ts)로 통일 —
// masterPage provenance cap 을 포함한 단일 진실 원천.
function layerReplayPlane(op: any, layer: any): 'background' | 'behindText' | 'flow' | 'inFrontOfText' {
  return layerPaintOpReplayPlane(op, layer);
}

function applyFlowImageCrop(
  element: HTMLImageElement,
  image: FlowImagePaintOp,
  displayScale: number,
  // [#6099] 90/270° 프레임은 회전 전 치수(swap)로 만들어지므로, crop 사영도
  // bbox(회전 후 외접)가 아니라 실제 프레임 치수를 기준으로 해야 한다.
  frameWidth: number = image.bbox.width,
  frameHeight: number = image.bbox.height,
): void {
  const crop = image.crop;
  if (!crop || element.naturalWidth <= 0 || element.naturalHeight <= 0) {
    element.style.left = '0';
    element.style.top = '0';
    element.style.width = '100%';
    element.style.height = '100%';
    return;
  }

  const scaleXHu = image.originalSizeHu
    ? image.originalSizeHu[0] / element.naturalWidth
    : HWP_UNITS_PER_CSS_PIXEL;
  const scaleYHu = image.originalSizeHu
    ? image.originalSizeHu[1] / element.naturalHeight
    : HWP_UNITS_PER_CSS_PIXEL;
  const sourceLeft = crop.left / scaleXHu;
  const sourceTop = crop.top / scaleYHu;
  const sourceWidth = (crop.right - crop.left) / scaleXHu;
  const sourceHeight = (crop.bottom - crop.top) / scaleYHu;
  if (sourceWidth <= 0 || sourceHeight <= 0) return;

  const scaleX = (frameWidth * displayScale) / sourceWidth;
  const scaleY = (frameHeight * displayScale) / sourceHeight;
  element.style.left = `${-sourceLeft * scaleX}px`;
  element.style.top = `${-sourceTop * scaleY}px`;
  element.style.width = `${element.naturalWidth * scaleX}px`;
  element.style.height = `${element.naturalHeight * scaleY}px`;
}
