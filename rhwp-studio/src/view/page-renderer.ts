import { WasmBridge } from '@/core/wasm-bridge';
import type { LayerRenderProfile } from '@/core/types';
import type { CanvasKitLayerRenderer } from './canvaskit-renderer';
import type { RenderBackend } from './render-backend';

interface LayerPlaneSummary {
  hasBehind: boolean;
  hasFront: boolean;
  imageCount: number;
  rawSvgCount: number;  // OLE/차트 rawSvg op 수 — 비동기 디코드 재렌더 트리거용(image 와 의미 분리, #1456)
}

export class PageRenderer {
  private reRenderTimers = new Map<number, ReturnType<typeof setTimeout>[]>();
  private imageRetryCounts = new Map<number, number>();

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
  ): void {
    if (this.backend === 'canvaskit') {
      this.renderPageCanvasKit(pageIdx, canvas, renderScale);
      return;
    }

    // 다층 layer 모드.
    // 1) 본문 Canvas 는 'flow' 필터로 BehindText/InFrontOfText plane 제외
    // 2) behind/front plane 은 같은 부모 컨테이너에 별도 canvas layer 로 합성
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
    this.drawMarginGuides(pageIdx, canvas, renderScale);
    const overlays = this.applyOverlays(pageIdx, canvas, renderScale, dpr);
    // rawSvg(차트/OLE)도 web_canvas draw_image 비동기 디코드 경로를 타므로
    // image 와 함께 재렌더 트리거 카운트에 합산한다(#1456).
    this.scheduleReRender(pageIdx, canvas, renderScale, overlays.imageCount + overlays.rawSvgCount);
  }

  getBackend(): RenderBackend {
    return this.backend;
  }

  private renderPageCanvasKit(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
  ): void {
    if (!this.canvaskitRenderer) {
      throw new Error('CanvasKit renderer가 초기화되지 않았습니다');
    }

    const parent = canvas.parentElement;
    if (parent) {
      this.removePageLayers(parent, pageIdx);
    }

    const pageInfo = this.wasm.getPageInfo(pageIdx);
    canvas.width = Math.max(1, Math.floor(pageInfo.width * renderScale));
    canvas.height = Math.max(1, Math.floor(pageInfo.height * renderScale));

    const tree = this.wasm.getPageLayerTreeObject(pageIdx, this.renderProfile);
    try {
      this.canvaskitRenderer.renderPage(tree, canvas, renderScale, pageInfo);
    } catch (error) {
      this.canvaskitRenderer.recordRenderFailure(error);
      console.error(`[PageRenderer] CanvasKit 페이지 렌더링 실패 (page=${pageIdx}):`, error);
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return;
    }
    this.cancelReRender(pageIdx);
    this.imageRetryCounts.delete(pageIdx);
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
  ): LayerPlaneSummary {
    const parent = canvas.parentElement;
    if (!parent) return { hasBehind: false, hasFront: false, imageCount: 0, rawSvgCount: 0 };

    // 페이지 단위 overlay 컨테이너를 Canvas 의 sibling 으로 관리.
    // data-rhwp-overlay-page 속성으로 식별, 페이지 재렌더링 시 갱신.
    this.removePageLayers(parent, pageIdx);

    const layers = this.getLayerPlaneSummary(pageIdx);
    if (!layers.hasBehind && !layers.hasFront) {
      canvas.style.background = '';
      canvas.style.zIndex = '';
      return layers;
    }

    // 위치/크기 정합용 공통 정보. Canvas 물리 픽셀은 page × zoom × DPR 이므로
    // CSS 표시 크기는 실제 DPR 로만 나눈다.
    const safeDpr = dpr > 0 && Number.isFinite(dpr) ? dpr : 1;
    const cssWidth = canvas.width / safeDpr;
    const cssHeight = canvas.height / safeDpr;
    const top = canvas.style.top;
    const left = canvas.style.left;
    const transform = canvas.style.transform;

    // BehindText 가 있는 페이지는 flow Canvas 를 투명 배경으로 두고,
    // 실제 pageBackground layer → BehindText → flow Canvas 순서로 합성한다.
    // Canvas 내부의 흰 배경은 WASM flow 렌더에서 생략된다.
    if (layers.hasBehind) {
      canvas.style.background = 'transparent';
      canvas.style.zIndex = '2';

      const background = this.createFilteredCanvasLayer(pageIdx, canvas, renderScale, 'background');
      background.dataset.rhwpOverlay = `background-${pageIdx}`;
      background.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(background, top, left, transform, cssWidth, cssHeight);
      background.style.zIndex = '0';
      parent.insertBefore(background, canvas);
    } else {
      canvas.style.background = '';
      canvas.style.zIndex = layers.hasFront ? '1' : '';
    }

    // BehindText overlay (Canvas 뒤). 이미지뿐 아니라 표/도형 PaintOp도 포함한다.
    if (layers.hasBehind) {
      const layer = this.createFilteredCanvasLayer(pageIdx, canvas, renderScale, 'behind');
      layer.dataset.rhwpOverlay = `behind-${pageIdx}`;
      layer.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = '1';
      // Canvas 보다 먼저 들어가도록 prepend
      parent.insertBefore(layer, canvas);
    }

    // InFrontOfText overlay (Canvas 앞). 이미지뿐 아니라 글상자/도형 PaintOp도 포함한다.
    if (layers.hasFront) {
      const layer = this.createFilteredCanvasLayer(pageIdx, canvas, renderScale, 'front');
      layer.dataset.rhwpOverlay = `front-${pageIdx}`;
      layer.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = layers.hasBehind ? '3' : '2';  // Canvas 보다 앞
      parent.appendChild(layer);
    }
    return layers;
  }

  private createFilteredCanvasLayer(
    pageIdx: number,
    sourceCanvas: HTMLCanvasElement,
    renderScale: number,
    layerKind: 'background' | 'behind' | 'front',
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
    this.wasm.renderPageToCanvasFiltered(pageIdx, layer, renderScale, layerKind);
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
    parent.querySelectorAll(
      `[data-rhwp-overlay-page="${pageIdx}"],` +
      `[data-rhwp-overlay="background-${pageIdx}"],` +
      `[data-rhwp-overlay="behind-${pageIdx}"],` +
      `[data-rhwp-overlay="front-${pageIdx}"]`,
    ).forEach((el) => el.remove());
  }

  removeAllPageLayers(parent: HTMLElement): void {
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
    this.scheduleReRender(pageIdx, canvas, scale, 0);
  }

  private getLayerPlaneSummary(pageIdx: number): LayerPlaneSummary {
    const summary: LayerPlaneSummary = { hasBehind: false, hasFront: false, imageCount: 0, rawSvgCount: 0 };
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
   * 작은 이미지 (헤더 라벨 등) 는 200/600ms 안에 디코드되지만, 큰 PNG/JPEG
   * (수십 KB~수백 KB) 는 디코드가 1초 이상 걸릴 수 있어 한 번 더 시도하고
   * (Task #1154), 그래도 누락이면 마지막에 자체 prefetch로 강제 디코드한다.
   */
  private scheduleReRender(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    renderScale: number,
    imageCount: number,
  ): void {
    if (imageCount <= 0) {
      this.cancelReRender(pageIdx);
      this.imageRetryCounts.delete(pageIdx);
      return;
    }
    if (this.imageRetryCounts.get(pageIdx) === imageCount) return;

    this.cancelReRender(pageIdx);
    this.imageRetryCounts.set(pageIdx, imageCount);

    const delays = [200, 600, 1500];
    const timers: ReturnType<typeof setTimeout>[] = [];

    for (const delay of delays) {
      const timer = setTimeout(() => {
        if (canvas.parentElement) {
          this.reRenderPageCanvases(pageIdx, canvas, renderScale);
        }
      }, delay);
      timers.push(timer);
    }

    // 안전망: 1500ms 시점에서도 큰 이미지가 디코드 안 끝났을 수 있으므로,
    // 페이지의 image base64 들을 자체 prefetch (Image.decode()) 한 후
    // 모두 완료되면 한 번 더 렌더링한다. setTimeout 과 별개로 동작.
    queueMicrotask(() => {
      this.prefetchLayerImages(pageIdx)
        .then(() => {
          if (canvas.parentElement) {
            this.reRenderPageCanvases(pageIdx, canvas, renderScale);
          }
        })
        .catch(() => {});
    });

    this.reRenderTimers.set(pageIdx, timers);
  }

  private reRenderPageCanvases(
    pageIdx: number,
    flowCanvas: HTMLCanvasElement,
    renderScale: number,
  ): void {
    this.wasm.renderPageToCanvasFiltered(pageIdx, flowCanvas, renderScale, 'flow');
    this.drawMarginGuides(pageIdx, flowCanvas, renderScale);
    const parent = flowCanvas.parentElement;
    if (!parent) return;
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
  private async prefetchLayerImages(pageIdx: number): Promise<void> {
    let json: string;
    try {
      json = this.wasm.getPageLayerTree(pageIdx);
    } catch {
      return;
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
    await Promise.all(tasks);
  }

  /** 특정 페이지의 지연 재렌더링을 취소한다 */
  cancelReRender(pageIdx: number): void {
    const timers = this.reRenderTimers.get(pageIdx);
    if (timers) {
      for (const t of timers) clearTimeout(t);
      this.reRenderTimers.delete(pageIdx);
    }
  }

  /** 모든 지연 재렌더링을 취소한다 */
  cancelAll(): void {
    for (const timers of this.reRenderTimers.values()) {
      for (const t of timers) clearTimeout(t);
    }
    this.reRenderTimers.clear();
  }

  resetImageRetryState(): void {
    this.imageRetryCounts.clear();
  }

  dispose(): void {
    this.cancelAll();
    this.canvaskitRenderer?.dispose();
    this.canvaskitRenderer = null;
  }
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
      if (op.type === 'image') {
        summary.imageCount += 1;
      } else if (op.type === 'rawSvg') {
        // 차트/OLE 미리보기. web_canvas draw_image 비동기 디코드 경로를 타므로
        // image 와 동일하게 재렌더 트리거 대상에 포함한다(#1456).
        summary.rawSvgCount += 1;
      }
      const plane = layerReplayPlane(op, activeLayer);
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
