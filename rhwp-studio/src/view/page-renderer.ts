import { WasmBridge } from '@/core/wasm-bridge';
import type { LayerRenderProfile } from '@/core/types';
import type { CanvasKitLayerRenderer, CanvasKitRenderDiagnostics } from './canvaskit-renderer';
import {
  collectFlowImagePaintOps,
  visibleFlowImageBbox,
  type FlowImagePaintOp,
} from './flow-image-clip';
import type { RenderBackend } from './render-backend';

interface LayerPlaneSummary {
  hasBehind: boolean;
  hasFront: boolean;
  imageCount: number;
  rawSvgCount: number;  // OLE/차트 rawSvg op 수 — 비동기 디코드 재렌더 트리거용(image 와 의미 분리, #1456)
  flowImageCount: number;
  flowRawSvgCount: number;
  flowStaticCount: number;
  signature: string;
}

export interface PageRenderContext {
  reason?: 'text-edit' | 'unknown';
  allowStaticOverlayReuse?: boolean;
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
}

interface LayerSummaryCacheEntry {
  key: string;
  summary: LayerPlaneSummary;
}

interface ReRenderJob {
  fallbackTimer: ReturnType<typeof setTimeout>;
  completed: boolean;
}

const IMAGE_RE_RENDER_FALLBACK_DELAY_MS = 1500;
const HWP_UNITS_PER_CSS_PIXEL = 75;

export class PageRenderer {
  private reRenderJobs = new Map<number, ReRenderJob>();
  private imageRetryCounts = new Map<number, string>();
  private layerSummaryCache = new Map<number, LayerSummaryCacheEntry>();
  private canvaskitDiagnosticsByPage = new Map<number, CanvasKitRenderDiagnostics>();
  private flowSplitSupported: boolean | null = null;

  constructor(
    private wasm: WasmBridge,
    private backend: RenderBackend = 'canvas2d',
    private renderProfile: LayerRenderProfile = 'screen',
    private canvaskitRenderer: CanvasKitLayerRenderer | null = null,
  ) {}

  /** 페이지를 Canvas에 렌더링한다 (renderScale = zoom × DPR) */
  renderPage(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    _displayScale: number,
    dpr: number,
    context: PageRenderContext = {},
  ): PageRenderResult {
    if (this.backend === 'canvaskit') {
      this.layerSummaryCache.delete(pageIdx);
      const renderedCanvas = this.renderPageCanvasKit(pageIdx, canvas, renderScale);
      return { needsTextEditStaticLayerVerification: false, renderedCanvas };
    }

    const layers = this.getLayerPlaneSummary(pageIdx, canvas, renderScale, context);
    const preferStaticFlow = this.shouldSplitStaticFlow(layers);
    let reuseStaticFlow = this.renderFlowCanvas(pageIdx, canvas, renderScale, preferStaticFlow);
    const flowImages = reuseStaticFlow && layers.flowImageCount > 0
      ? this.getFlowImagePaintOps(pageIdx)
      : [];
    const usesDomFlowImages =
      reuseStaticFlow &&
      layers.flowRawSvgCount === 0 &&
      flowImages.length === layers.flowImageCount &&
      flowImages.length > 0;

    // 다층 layer 모드.
    // 1) 본문 Canvas 는 'flow' 필터로 BehindText/InFrontOfText plane 제외
    // 2) behind/front plane 은 같은 부모 컨테이너에 별도 canvas layer 로 합성
    this.drawMarginGuides(pageIdx, canvas, renderScale);
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
      canvas.parentElement && this.removeOverlayLayer(canvas.parentElement, pageIdx, 'flow-static');
      reuseStaticFlow = false;
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
      this.drawMarginGuides(pageIdx, canvas, renderScale);
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
      {
        retrySignature: overlays.signature,
        reuseStaticFlow,
        reuseStaticOverlay: context.reason === 'text-edit' && context.allowStaticOverlayReuse === true,
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

  getCanvasKitRenderDiagnostics(pageIdx: number): CanvasKitRenderDiagnostics | null {
    const diagnostics = this.canvaskitDiagnosticsByPage.get(pageIdx);
    if (!diagnostics) return null;
    return {
      ...diagnostics,
      lastUnsupportedOps: [...diagnostics.lastUnsupportedOps],
      lastExpectedUnsupportedOps: [...diagnostics.lastExpectedUnsupportedOps],
      lastUnexpectedUnsupportedOps: [...diagnostics.lastUnexpectedUnsupportedOps],
      readinessBlockers: [...diagnostics.readinessBlockers],
    };
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
      canvas.width = Math.max(1, Math.floor(pageInfo.width * renderScale));
      canvas.height = Math.max(1, Math.floor(pageInfo.height * renderScale));
      const tree = this.wasm.getPageLayerTreeObject(pageIdx, this.renderProfile);
      renderStarted = true;
      const renderedCanvas = this.canvaskitRenderer.renderPage(tree, canvas, renderScale, pageInfo);
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
        const flowStatic = this.createOrReuseFilteredCanvasLayer(
          pageIdx,
          canvas,
          renderScale,
          'flow-static',
          layers,
          allowReuse,
          false,
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
    layer.style.background = 'var(--doc-paper)';

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

      const frame = document.createElement('div');
      frame.style.position = 'absolute';
      frame.style.left = `${(image.bbox.x - (needsClipWrapper ? visibleBbox.x : 0)) * displayScale}px`;
      frame.style.top = `${(image.bbox.y - (needsClipWrapper ? visibleBbox.y : 0)) * displayScale}px`;
      frame.style.width = `${image.bbox.width * displayScale}px`;
      frame.style.height = `${image.bbox.height * displayScale}px`;
      frame.style.overflow = 'hidden';
      frame.style.pointerEvents = 'none';
      const scaleX = image.horzFlip ? -1 : 1;
      const scaleY = image.vertFlip ? -1 : 1;
      frame.style.transform = `rotate(${image.rotation}deg) scale(${scaleX}, ${scaleY})`;
      frame.style.transformOrigin = 'center';

      const element = new Image();
      element.src = `data:${image.mime};base64,${image.base64}`;
      element.style.position = 'absolute';
      element.style.pointerEvents = 'none';
      const applyCrop = () => applyFlowImageCrop(element, image, displayScale);
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
      this.wasm.renderPageToCanvasFiltered(pageIdx, layer, renderScale, layerKind);
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
  renderPageFlow(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'flow');
    this.drawMarginGuides(pageIdx, canvas, scale);
    this.scheduleReRender(pageIdx, canvas, scale, 0, {
      retrySignature: 'flow-only',
      reuseStaticFlow: false,
      reuseStaticOverlay: false,
    });
  }

  private shouldSplitStaticFlow(layers: LayerPlaneSummary): boolean {
    return (
      !layers.hasBehind &&
      layers.flowStaticCount > 0 &&
      this.flowSplitSupported !== false
    );
  }

  private getFlowImagePaintOps(pageIdx: number): FlowImagePaintOp[] {
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
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
      return false;
    }
    try {
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow-dynamic');
      this.flowSplitSupported = true;
      return true;
    } catch (error) {
      this.flowSplitSupported = false;
      console.warn('[PageRenderer] flow-dynamic 렌더 미지원, 기존 flow 렌더로 fallback:', error);
      this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
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
      return {
        hasBehind: wrapper.hasBehind,
        hasFront: wrapper.hasFront,
        imageCount,
        rawSvgCount,
        flowImageCount,
        flowRawSvgCount,
        flowStaticCount,
        signature: `overlay:${wrapper.hasBehind ? 1 : 0}:${wrapper.hasFront ? 1 : 0}:${imageCount}:${rawSvgCount}:${flowImageCount}:${flowRawSvgCount}:${json.length}`,
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
        collectLayerPlaneSummary(root, summary, null);
        summary.flowStaticCount = summary.flowImageCount + summary.flowRawSvgCount;
        summary.signature = `tree:${summary.hasBehind ? 1 : 0}:${summary.hasFront ? 1 : 0}:${summary.imageCount}:${summary.rawSvgCount}:${summary.flowImageCount}:${summary.flowRawSvgCount}`;
      }
    } catch (e) {
      console.warn('[PageRenderer] PageLayerTree JSON parse 실패:', e);
    }
    return summary;
  }

  /** 편집 용지 여백 가이드라인을 캔버스에 그린다 (4모서리 L자 표시) */
  private drawMarginGuides(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    const pageInfo = this.wasm.getPageInfo(pageIdx);
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const { width, height, marginLeft, marginRight, marginTop, marginBottom, marginHeader, marginFooter } = pageInfo;
    const left = marginLeft;
    // 한컴 HWP 기준: 본문 시작 = marginHeader + marginTop
    const top = marginHeader + marginTop;
    const right = width - marginRight;
    // 한컴 HWP 기준: 본문 끝 = height - marginFooter - marginBottom
    const bottom = height - marginFooter - marginBottom;
    const L = 15;

    ctx.save();
    // WASM 렌더링 후 ctx transform 상태가 불확실하므로 명시적으로 설정
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    ctx.strokeStyle = '#C0C0C0';
    ctx.lineWidth = 0.3;
    ctx.beginPath();

    // 좌상 코너
    ctx.moveTo(left, top - L);
    ctx.lineTo(left, top);
    ctx.lineTo(left - L, top);

    // 우상 코너
    ctx.moveTo(right + L, top);
    ctx.lineTo(right, top);
    ctx.lineTo(right, top - L);

    // 좌하 코너
    ctx.moveTo(left - L, bottom);
    ctx.lineTo(left, bottom);
    ctx.lineTo(left, bottom + L);

    // 우하 코너
    ctx.moveTo(right, bottom + L);
    ctx.lineTo(right, bottom);
    ctx.lineTo(right + L, bottom);

    ctx.stroke();
    ctx.restore();
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
    policy: ReRenderPolicy,
  ): void {
    if (imageCount <= 0) {
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return;
    }
    const retryKey = `${imageCount}:${policy.retrySignature}`;
    if (this.imageRetryCounts.get(pageIdx) === retryKey) return;

    this.cancelReRender(pageIdx);
    this.imageRetryCounts.set(pageIdx, retryKey);

    const job: ReRenderJob = {
      fallbackTimer: 0 as unknown as ReturnType<typeof setTimeout>,
      completed: false,
    };
    const finish = () => {
      if (job.completed || this.reRenderJobs.get(pageIdx) !== job) return;
      job.completed = true;
      clearTimeout(job.fallbackTimer);
      this.reRenderJobs.delete(pageIdx);
      if (canvas.parentElement) {
        this.reRenderPageCanvases(pageIdx, canvas, renderScale, policy);
      }
    };
    job.fallbackTimer = setTimeout(finish, IMAGE_RE_RENDER_FALLBACK_DELAY_MS);
    this.reRenderJobs.set(pageIdx, job);

    // 자체 prefetch로 실제 decode를 마친 경우에만 fallback보다 먼저 다시 그린다.
    queueMicrotask(() => {
      this.prefetchLayerImages(pageIdx)
        .then((decoded) => {
          if (decoded) finish();
        })
        .catch(() => {});
    });
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
          this.wasm.renderPageToCanvasFiltered(pageIdx, flowStatic, renderScale, 'flow-static');
          renderedStaticFlow = true;
        } catch (error) {
          this.flowSplitSupported = false;
          flowStatic.remove();
          console.warn('[PageRenderer] flow-static 지연 재렌더 실패, 기존 flow 재렌더로 fallback:', error);
        }
      }
    }

    if (!renderedStaticFlow) {
      this.wasm.renderPageToCanvasFiltered(pageIdx, flowCanvas, renderScale, 'flow');
      this.drawMarginGuides(pageIdx, flowCanvas, renderScale);
    }

    if (policy.reuseStaticOverlay) return;

    parent.querySelectorAll<HTMLCanvasElement>(
      `[data-rhwp-overlay-page="${pageIdx}"][data-rhwp-layer-kind]`,
    ).forEach((layerCanvas) => {
      const kind = layerCanvas.dataset.rhwpLayerKind;
      if (kind === 'background' || kind === 'behind' || kind === 'front') {
        layerCanvas.width = flowCanvas.width;
        layerCanvas.height = flowCanvas.height;
        this.wasm.renderPageToCanvasFiltered(pageIdx, layerCanvas, renderScale, kind);
      }
    });
  }

  /**
   * 페이지의 image base64 데이터를
   * 자체 prefetch 하여 모든 이미지가 브라우저에 디코드 완료될 때까지 대기.
   * Task #1154 — IMAGE_CACHE 의 비동기 디코드 누락 안전망.
   */
  private async prefetchLayerImages(pageIdx: number): Promise<boolean> {
    let json: string;
    try {
      json = this.wasm.getPageLayerTree(pageIdx);
    } catch {
      return false;
    }
    const tasks: Promise<unknown>[] = [];
    const seen = new Set<string>();
    const enqueue = (dataUrl: string) => {
      if (seen.has(dataUrl)) return;
      seen.add(dataUrl);
      tasks.push(
        new Promise<void>((resolve) => {
          const img = new Image();
          img.onload = () => resolve();
          img.onerror = () => resolve();
          img.src = dataUrl;
          // decode() 이 더 정확하지만 일부 브라우저 미지원
          if (typeof img.decode === 'function') {
            img.decode().then(() => resolve()).catch(() => resolve());
          }
        }),
      );
    };
    // image 항목들의 mime + base64 추출 (간단한 정규식)
    const re = /"type":"image"[^}]*?(?:"wrap":"(behindText|inFrontOfText)")?[^}]*?"mime":"([^"]+)","base64":"([^"]+)"/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(json)) !== null) {
      enqueue(`data:${m[2]};base64,${m[3]}`);
    }
    // rawSvg 항목 (OLE/차트 미리보기) 의 embedded data URL 추출.
    // svg 필드는 JSON 인코딩 문자열이며 내부에 data:image/MIME;base64,... 가 등장한다.
    // rawSvg 의 wrap 은 항상 flow 이므로 overlay 필터링 불필요.
    const dataUrlRe = /data:(image\/[A-Za-z0-9.+-]+);base64,([A-Za-z0-9+/=]+)/g;
    let d: RegExpExecArray | null;
    while ((d = dataUrlRe.exec(json)) !== null) {
      enqueue(`data:${d[1]};base64,${d[2]}`);
    }
    if (tasks.length === 0) return false;
    await Promise.all(tasks);
    return true;
  }

  /** 특정 페이지의 지연 재렌더링을 취소한다 */
  cancelReRender(pageIdx: number): void {
    const job = this.reRenderJobs.get(pageIdx);
    if (job) {
      job.completed = true;
      clearTimeout(job.fallbackTimer);
      this.reRenderJobs.delete(pageIdx);
    }
  }

  /** 모든 지연 재렌더링을 취소한다 */
  cancelAll(): void {
    for (const job of this.reRenderJobs.values()) {
      job.completed = true;
      clearTimeout(job.fallbackTimer);
    }
    this.reRenderJobs.clear();
  }

  resetImageRetryState(): void {
    this.imageRetryCounts.clear();
    this.layerSummaryCache.clear();
    this.canvaskitDiagnosticsByPage.clear();
  }

  dispose(): void {
    this.cancelAll();
    this.layerSummaryCache.clear();
    this.canvaskitDiagnosticsByPage.clear();
    this.canvaskitRenderer?.dispose();
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
    signature: 'empty',
  };
}

function finiteCount(value: unknown): number {
  const n = Number(value);
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}

function collectLayerPlaneSummary(
  node: any,
  summary: LayerPlaneSummary,
  inheritedLayer: any,
): void {
  if (!node || typeof node !== 'object') return;
  const activeLayer = node.layer ?? inheritedLayer;
  if (Array.isArray(node.ops)) {
    for (const op of node.ops) {
      if (!op || typeof op !== 'object') continue;
      const plane = layerReplayPlane(op, activeLayer);
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
      if (plane === 'behindText') {
        summary.hasBehind = true;
      } else if (plane === 'inFrontOfText') {
        summary.hasFront = true;
      }
    }
  }
  if (Array.isArray(node.children)) {
    for (const child of node.children) {
      collectLayerPlaneSummary(child, summary, activeLayer);
    }
  }
  if (node.child) {
    collectLayerPlaneSummary(node.child, summary, activeLayer);
  }
}

function layerReplayPlane(op: any, layer: any): 'background' | 'behindText' | 'flow' | 'inFrontOfText' {
  if (op?.type === 'pageBackground') {
    return 'background';
  }
  if (layer?.textWrap === 'behindText') {
    return 'behindText';
  }
  if (layer?.textWrap === 'inFrontOfText') {
    return 'inFrontOfText';
  }
  if (op?.type === 'image') {
    if (op.wrap === 'behindText') return 'behindText';
    if (op.wrap === 'inFrontOfText') return 'inFrontOfText';
  }
  return 'flow';
}

function applyFlowImageCrop(
  element: HTMLImageElement,
  image: FlowImagePaintOp,
  displayScale: number,
): void {
  const crop = image.crop;
  if (!crop || element.naturalWidth <= 0 || element.naturalHeight <= 0) {
    element.style.left = '0';
    element.style.top = '0';
    element.style.width = '100%';
    element.style.height = '100%';
    return;
  }

  const sourceLeft = crop.left / HWP_UNITS_PER_CSS_PIXEL;
  const sourceTop = crop.top / HWP_UNITS_PER_CSS_PIXEL;
  const sourceWidth = (crop.right - crop.left) / HWP_UNITS_PER_CSS_PIXEL;
  const sourceHeight = (crop.bottom - crop.top) / HWP_UNITS_PER_CSS_PIXEL;
  if (sourceWidth <= 0 || sourceHeight <= 0) return;

  const scaleX = (image.bbox.width * displayScale) / sourceWidth;
  const scaleY = (image.bbox.height * displayScale) / sourceHeight;
  element.style.left = `${-sourceLeft * scaleX}px`;
  element.style.top = `${-sourceTop * scaleY}px`;
  element.style.width = `${element.naturalWidth * scaleX}px`;
  element.style.height = `${element.naturalHeight * scaleY}px`;
}
