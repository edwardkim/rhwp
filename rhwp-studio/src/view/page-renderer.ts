import { WasmBridge } from '@/core/wasm-bridge';
import type { LayerRenderProfile } from '@/core/types';
import type { CanvasKitLayerRenderer } from './canvaskit-renderer';
import type { RenderBackend } from './render-backend';

/**
 * PageLayerTree JSON 의 PaintOp::Image 메타정보 (Task #516, Stage 5.2).
 * BehindText / InFrontOfText 그림 overlay 생성에 사용.
 */
export interface OverlayImageInfo {
  bbox: { x: number; y: number; width: number; height: number };
  mime: string;
  base64: string;
  effect: string;
  brightness: number;
  contrast: number;
  watermark?: { preset: 'hancom-watermark' | 'custom' };
  bakedWatermark?: boolean;
  wrap: 'behindText' | 'inFrontOfText';
  transform?: { rotation: number; horzFlip: boolean; vertFlip: boolean };
  /** 그림 자르기 (HWPUNIT, 75 HU = 1 px @ 96 DPI). `<img>` 의 source rect → bbox 매핑용. */
  crop?: { left: number; top: number; right: number; bottom: number };
}

interface OverlayImagesResult {
  behind: OverlayImageInfo[];
  front: OverlayImageInfo[];
  imageCount: number;
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
    displayScale: number,
    dpr: number,
  ): void {
    if (this.backend === 'canvaskit') {
      this.renderPageCanvasKit(pageIdx, canvas, renderScale);
      return;
    }

    // Task #516 Stage 5.2: 다층 layer 모드.
    // 1) 본문 Canvas 는 'flow' 필터로 BehindText/InFrontOfText 그림 제외
    // 2) overlay (BehindText / InFrontOfText) 는 같은 부모 컨테이너에 <img> 로 추가
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
    this.drawMarginGuides(pageIdx, canvas, renderScale);
    const overlays = this.applyOverlays(pageIdx, canvas, displayScale, dpr);
    this.scheduleReRender(pageIdx, canvas, renderScale, overlays.imageCount);
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
   * Canvas 의 부모 컨테이너에 BehindText / InFrontOfText 그림을 <img> overlay 로 추가.
   *
   * - BehindText: z-index 가 Canvas 뒤
   * - InFrontOfText: z-index 가 Canvas 앞
   * - mix-blend-mode 로 워터마크 효과 (multiply 등) 적용
   * - pointer-events: none — hit-test 는 Canvas (텍스트) 가 받음
   */
  private applyOverlays(
    pageIdx: number,
    canvas: HTMLCanvasElement,
    displayScale: number,
    dpr: number,
  ): OverlayImagesResult {
    const parent = canvas.parentElement;
    if (!parent) return { behind: [], front: [], imageCount: 0 };

    // 페이지 단위 overlay 컨테이너를 Canvas 의 sibling 으로 관리.
    // data-rhwp-overlay-page 속성으로 식별, 페이지 재렌더링 시 갱신.
    this.removePageLayers(parent, pageIdx);

    const overlays = this.getOverlayImages(pageIdx);
    const { behind, front } = overlays;
    if (behind.length === 0 && front.length === 0) {
      canvas.style.background = '';
      canvas.style.zIndex = '';
      return overlays;
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
    // 별도 페이지 배경 layer → BehindText → flow Canvas 순서로 합성한다.
    // Canvas 내부의 흰 배경은 WASM flow 렌더에서 생략된다.
    if (behind.length > 0) {
      canvas.style.background = 'transparent';
      canvas.style.zIndex = '2';

      const background = document.createElement('div');
      background.dataset.rhwpOverlay = `background-${pageIdx}`;
      background.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(background, top, left, transform, cssWidth, cssHeight);
      background.style.background = 'var(--color-surface)';
      background.style.zIndex = '0';
      parent.insertBefore(background, canvas);
    } else {
      canvas.style.background = '';
      canvas.style.zIndex = front.length > 0 ? '1' : '';
    }

    // BehindText overlay (Canvas 뒤)
    if (behind.length > 0) {
      const layer = this.createOverlayLayer(behind, displayScale);
      layer.dataset.rhwpOverlay = `behind-${pageIdx}`;
      layer.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = '1';
      // Canvas 보다 먼저 들어가도록 prepend
      parent.insertBefore(layer, canvas);
    }

    // InFrontOfText overlay (Canvas 앞)
    if (front.length > 0) {
      const layer = this.createOverlayLayer(front, displayScale);
      layer.dataset.rhwpOverlay = `front-${pageIdx}`;
      layer.dataset.rhwpOverlayPage = String(pageIdx);
      this.applyPageLayerBox(layer, top, left, transform, cssWidth, cssHeight);
      layer.style.zIndex = behind.length > 0 ? '3' : '2';  // Canvas 보다 앞
      parent.appendChild(layer);
    }
    return overlays;
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

  /** overlay 레이어 div 를 생성하고 그림 <img> 들을 추가 */
  private createOverlayLayer(
    images: OverlayImageInfo[],
    displayScale: number,
  ): HTMLDivElement {
    const layer = document.createElement('div');
    for (const img of images) {
      const el = document.createElement('img');
      el.src = `data:${img.mime};base64,${img.base64}`;
      el.style.pointerEvents = 'none';

      const filterParts: string[] = [];
      if (!img.bakedWatermark) {
        // CSS filter (그림 효과 + 밝기 + 대비)
        if (img.effect === 'grayScale' || img.effect === 'pattern8x8') {
          filterParts.push('grayscale(100%)');
        } else if (img.effect === 'blackWhite') {
          filterParts.push('grayscale(100%)');
          filterParts.push('contrast(1000%)');
        }
        if (img.brightness !== 0) {
          filterParts.push(`brightness(${(100 + img.brightness) / 100})`);
        }
        if (img.contrast !== 0) {
          filterParts.push(`contrast(${(100 + img.contrast) / 100})`);
        }
        if (filterParts.length > 0) {
          el.style.filter = filterParts.join(' ');
        }
      }
      // 워터마크는 multiply blend (흰색 배경 = 투명 효과, 텍스트 위 자연 합성).
      if (img.watermark && !img.bakedWatermark) {
        el.style.mixBlendMode = 'multiply';
        // WebCanvasRenderer 의 워터마크 alpha 정책과 동기화 (#677).
        el.style.opacity = '0.17';
      }
      // transform (회전/플립) — 작업 우선순위 낮음, 본 사이클은 미적용

      // bbox 는 zoom=1 페이지 좌표계이므로 화면 표시 배율을 적용한다.
      const dx = img.bbox.x * displayScale;
      const dy = img.bbox.y * displayScale;
      const dw = img.bbox.width * displayScale;
      const dh = img.bbox.height * displayScale;

      // crop (HWPUNIT, 75 HU = 1 px @ 96 DPI) 가 있고 자르기가 의미 있는 경우 source rect 처리.
      // <img> 는 직접 source rect 를 지정할 수 없으므로 wrapper div + overflow:hidden 으로
      // 시각 영역만 노출하고 내부 img 를 scale + offset 으로 배치한다 (Task #1154).
      const crop = img.crop;
      const cropSpanW = crop ? Math.max(0, crop.right - crop.left) : 0;
      const cropSpanH = crop ? Math.max(0, crop.bottom - crop.top) : 0;
      const hasCrop = !!crop && cropSpanW > 0 && cropSpanH > 0;
      if (hasCrop && crop) {
        // crop_hu / 75 → source rect (px) in image natural pixel space.
        const sxPx = crop.left / 75;
        const syPx = crop.top / 75;
        const swPx = cropSpanW / 75;
        const shPx = cropSpanH / 75;
        const scaleX = dw / swPx;
        const scaleY = dh / shPx;

        const wrapper = document.createElement('div');
        wrapper.style.position = 'absolute';
        wrapper.style.left = `${dx}px`;
        wrapper.style.top = `${dy}px`;
        wrapper.style.width = `${dw}px`;
        wrapper.style.height = `${dh}px`;
        wrapper.style.overflow = 'hidden';
        wrapper.style.pointerEvents = 'none';

        el.style.position = 'absolute';
        el.style.left = `${-sxPx * scaleX}px`;
        el.style.top = `${-syPx * scaleY}px`;
        // img.naturalWidth/Height 는 디코딩 완료 후에만 정확하므로 onload 시점에 적용.
        // 첫 렌더 즉시 비율로 width 만 지정 (naturalWidth 대신 swPx*scaleX = dw 같은
        // 자명한 단서가 없어, 로드 전에는 시각이 어색할 수 있다. scheduleReRender 가
        // 이미 비동기 디코드를 처리하므로 onload 콜백으로 최종 크기를 확정한다).
        const applyNaturalSize = () => {
          el.style.width = `${el.naturalWidth * scaleX}px`;
          el.style.height = `${el.naturalHeight * scaleY}px`;
        };
        if (el.complete && el.naturalWidth > 0) {
          applyNaturalSize();
        } else {
          el.addEventListener('load', applyNaturalSize, { once: true });
        }
        wrapper.appendChild(el);
        layer.appendChild(wrapper);
      } else {
        el.style.position = 'absolute';
        el.style.left = `${dx}px`;
        el.style.top = `${dy}px`;
        el.style.width = `${dw}px`;
        el.style.height = `${dh}px`;
        layer.appendChild(el);
      }
    }
    return layer;
  }

  /**
   * 페이지를 본문 layer (flow) 만 Canvas 에 렌더링한다 (Task #516, Stage 5.2).
   * BehindText / InFrontOfText 그림은 제외 — overlay 로 별도 표시.
   */
  renderPageFlow(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'flow');
    this.drawMarginGuides(pageIdx, canvas, scale);
    this.scheduleReRender(pageIdx, canvas, scale, 0);
  }

  /**
   * 페이지의 BehindText / InFrontOfText 그림 overlay 정보를 추출한다 (Task #516, Stage 5.2).
   * PageLayerTree JSON 을 파싱하여 wrap = behindText / inFrontOfText 인 image op 만 반환.
   */
  getOverlayImages(pageIdx: number): OverlayImagesResult {
    const overlayJson = this.wasm.getPageOverlayImages(pageIdx);
    if (overlayJson) {
      try {
        const parsed = JSON.parse(overlayJson);
        return {
          behind: Array.isArray(parsed?.behind) ? parsed.behind : [],
          front: Array.isArray(parsed?.front) ? parsed.front : [],
          imageCount: typeof parsed?.imageCount === 'number' ? parsed.imageCount : 0,
        };
      } catch (e) {
        console.warn('[PageRenderer] overlay image JSON parse 실패:', e);
      }
    }

    const json = this.wasm.getPageLayerTree(pageIdx);
    const behind: OverlayImageInfo[] = [];
    const front: OverlayImageInfo[] = [];
    const imageCount = (json.match(/"type":"image"/g) || []).length;
    try {
      const wrapper = JSON.parse(json);
      // PageLayerTree JSON 의 트리는 wrapper.root 안에 있음.
      // wrapper = { schemaVersion, pageWidth, pageHeight, root: { kind, ... } }
      const root = wrapper?.root;
      if (root) {
        collectOverlayImages(root, behind, front);
      }
    } catch (e) {
      console.warn('[PageRenderer] PageLayerTree JSON parse 실패:', e);
    }
    return { behind, front, imageCount };
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
          this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
          this.drawMarginGuides(pageIdx, canvas, renderScale);
        }
      }, delay);
      timers.push(timer);
    }

    // 안전망: 1500ms 시점에서도 큰 이미지가 디코드 안 끝났을 수 있으므로,
    // 페이지의 flow image base64 들을 자체 prefetch (Image.decode()) 한 후
    // 모두 완료되면 한 번 더 렌더링한다. setTimeout 과 별개로 동작.
    queueMicrotask(() => {
      this.prefetchFlowImages(pageIdx)
        .then(() => {
          if (canvas.parentElement) {
            this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, renderScale, 'flow');
            this.drawMarginGuides(pageIdx, canvas, renderScale);
          }
        })
        .catch(() => {});
    });

    this.reRenderTimers.set(pageIdx, timers);
  }

  /**
   * 페이지의 flow layer image (BehindText/InFrontOfText 제외) base64 데이터를
   * 자체 prefetch 하여 모든 이미지가 브라우저에 디코드 완료될 때까지 대기.
   * Task #1154 — IMAGE_CACHE 의 비동기 디코드 누락 안전망.
   */
  private async prefetchFlowImages(pageIdx: number): Promise<void> {
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
      const wrap = m[1]; // overlay wrap 은 prefetch 대상 아님 (별도 <img>)
      if (wrap === 'behindText' || wrap === 'inFrontOfText') continue;
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

/**
 * PageLayerTree JSON 트리를 재귀 순회하며 overlay 후보 image op 수집 (Task #516).
 * BehindText / InFrontOfText 그림만 분리. 본문 layer 의 image (어울림/위아래/None) 는 무시.
 */
function collectOverlayImages(
  node: any,
  behind: OverlayImageInfo[],
  front: OverlayImageInfo[],
): void {
  if (!node || typeof node !== 'object') return;
  // ops 배열 (Leaf 노드)
  if (Array.isArray(node.ops)) {
    for (const op of node.ops) {
      if (op?.type !== 'image') continue;
      if (op.wrap === 'behindText') {
        behind.push(toOverlayInfo(op, 'behindText'));
      } else if (op.wrap === 'inFrontOfText') {
        front.push(toOverlayInfo(op, 'inFrontOfText'));
      }
    }
  }
  // children (Group/ClipRect)
  if (Array.isArray(node.children)) {
    for (const child of node.children) {
      collectOverlayImages(child, behind, front);
    }
  }
  if (node.child) {
    collectOverlayImages(node.child, behind, front);
  }
}

function toOverlayInfo(op: any, wrap: 'behindText' | 'inFrontOfText'): OverlayImageInfo {
  return {
    bbox: op.bbox,
    mime: op.mime ?? 'application/octet-stream',
    base64: op.base64 ?? '',
    effect: op.effect ?? 'realPic',
    brightness: op.brightness ?? 0,
    contrast: op.contrast ?? 0,
    watermark: op.watermark,
    bakedWatermark: op.bakedWatermark === true,
    wrap,
    transform: op.transform,
    crop: op.crop,
  };
}
