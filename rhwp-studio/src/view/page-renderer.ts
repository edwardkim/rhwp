import { WasmBridge } from '@/core/wasm-bridge';

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
  wrap: 'behindText' | 'inFrontOfText';
  transform?: { rotation: number; horzFlip: boolean; vertFlip: boolean };
}

export class PageRenderer {
  private reRenderTimers = new Map<number, ReturnType<typeof setTimeout>[]>();

  constructor(private wasm: WasmBridge) {}

  /** 페이지를 Canvas에 렌더링한다 (scale = zoom × DPR) */
  renderPage(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    // Task #516 Stage 5.2: 다층 layer 모드.
    // 1) 본문 Canvas 는 'flow' 필터로 BehindText/InFrontOfText 그림 제외
    // 2) overlay (BehindText / InFrontOfText) 는 같은 부모 컨테이너에 <img> 로 추가
    this.wasm.renderPageToCanvasFiltered(pageIdx, canvas, scale, 'flow');
    this.drawMarginGuides(pageIdx, canvas, scale);
    this.applyOverlays(pageIdx, canvas, scale);
    this.scheduleReRender(pageIdx, canvas, scale);
  }

  /**
   * Canvas 의 부모 컨테이너에 BehindText / InFrontOfText 그림을 <img> overlay 로 추가.
   *
   * - BehindText: z-index 가 Canvas 뒤
   * - InFrontOfText: z-index 가 Canvas 앞
   * - mix-blend-mode 로 워터마크 효과 (multiply 등) 적용
   * - pointer-events: none — hit-test 는 Canvas (텍스트) 가 받음
   */
  private applyOverlays(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    const parent = canvas.parentElement;
    if (!parent) return;

    // 페이지 단위 overlay 컨테이너를 Canvas 의 sibling 으로 관리.
    // data-rhwp-overlay-page 속성으로 식별, 페이지 재렌더링 시 갱신.
    const existingBehind = parent.querySelector(
      `[data-rhwp-overlay="behind-${pageIdx}"]`,
    ) as HTMLElement | null;
    const existingFront = parent.querySelector(
      `[data-rhwp-overlay="front-${pageIdx}"]`,
    ) as HTMLElement | null;
    if (existingBehind) existingBehind.remove();
    if (existingFront) existingFront.remove();

    const { behind, front } = this.getOverlayImages(pageIdx);
    // Task #516 Stage 5.2 진단 로그 — 시각 판정 통과 후 제거
    console.log(`[Task#516] applyOverlays page=${pageIdx} behind=${behind.length} front=${front.length}`);
    if (behind.length === 0 && front.length === 0) return;

    // 위치/크기 정합용 공통 정보
    const dpr = scale; // scale = zoom × DPR. CSS 표시 크기 = canvas / dpr
    const cssWidth = canvas.width / dpr;
    const cssHeight = canvas.height / dpr;
    const top = canvas.style.top;
    const left = canvas.style.left;
    const transform = canvas.style.transform;

    // BehindText overlay (Canvas 뒤)
    if (behind.length > 0) {
      const layer = this.createOverlayLayer(behind, cssWidth, cssHeight);
      layer.dataset.rhwpOverlay = `behind-${pageIdx}`;
      layer.style.position = 'absolute';
      layer.style.top = top;
      layer.style.left = left;
      layer.style.transform = transform;
      layer.style.width = `${cssWidth}px`;
      layer.style.height = `${cssHeight}px`;
      layer.style.pointerEvents = 'none';
      layer.style.zIndex = '0';  // Canvas (z=auto) 보다 뒤
      // Canvas 보다 먼저 들어가도록 prepend
      parent.insertBefore(layer, canvas);
    }

    // InFrontOfText overlay (Canvas 앞)
    if (front.length > 0) {
      const layer = this.createOverlayLayer(front, cssWidth, cssHeight);
      layer.dataset.rhwpOverlay = `front-${pageIdx}`;
      layer.style.position = 'absolute';
      layer.style.top = top;
      layer.style.left = left;
      layer.style.transform = transform;
      layer.style.width = `${cssWidth}px`;
      layer.style.height = `${cssHeight}px`;
      layer.style.pointerEvents = 'none';
      layer.style.zIndex = '2';  // Canvas (z=auto) 보다 앞
      parent.appendChild(layer);
    }
  }

  /** overlay 레이어 div 를 생성하고 그림 <img> 들을 추가 */
  private createOverlayLayer(
    images: OverlayImageInfo[],
    cssWidth: number,
    cssHeight: number,
  ): HTMLDivElement {
    const layer = document.createElement('div');
    for (const img of images) {
      const el = document.createElement('img');
      el.src = `data:${img.mime};base64,${img.base64}`;
      el.style.position = 'absolute';
      // bbox 는 페이지 좌표계 (CSS px 기준), Canvas 와 동일 좌표계.
      el.style.left = `${img.bbox.x}px`;
      el.style.top = `${img.bbox.y}px`;
      el.style.width = `${img.bbox.width}px`;
      el.style.height = `${img.bbox.height}px`;
      el.style.pointerEvents = 'none';
      // CSS filter (그림 효과 + 밝기 + 대비)
      const filterParts: string[] = [];
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
      // 워터마크는 multiply blend (흰색 배경 = 투명 효과, 텍스트 위 자연 합성).
      // 회색조 처리 + 투명도 조절의 정합한 시각은 별도 task 로 분리 처리.
      if (img.watermark) {
        el.style.mixBlendMode = 'multiply';
      }
      // transform (회전/플립) — 작업 우선순위 낮음, 본 사이클은 미적용
      void cssWidth; void cssHeight;
      layer.appendChild(el);
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
    this.scheduleReRender(pageIdx, canvas, scale);
  }

  /**
   * 페이지의 BehindText / InFrontOfText 그림 overlay 정보를 추출한다 (Task #516, Stage 5.2).
   * PageLayerTree JSON 을 파싱하여 wrap = behindText / inFrontOfText 인 image op 만 반환.
   */
  getOverlayImages(pageIdx: number): { behind: OverlayImageInfo[]; front: OverlayImageInfo[] } {
    const json = this.wasm.getPageLayerTree(pageIdx);
    const behind: OverlayImageInfo[] = [];
    const front: OverlayImageInfo[] = [];
    // Task #516 진단 로그 (시각 판정 통과 후 제거)
    const imageOpCount = (json.match(/"type":"image"/g) || []).length;
    const wrapBehindCount = (json.match(/"wrap":"behindText"/g) || []).length;
    console.log(`[Task#516] JSON image ops=${imageOpCount}, wrap=behindText=${wrapBehindCount}`);
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
    console.log(`[Task#516] collected behind=${behind.length} front=${front.length}`);
    return { behind, front };
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
   * 200ms, 600ms 두 번 재시도하여 대부분의 이미지 로드를 커버한다.
   */
  private scheduleReRender(pageIdx: number, canvas: HTMLCanvasElement, scale: number): void {
    this.cancelReRender(pageIdx);

    const delays = [200, 600];
    const timers: ReturnType<typeof setTimeout>[] = [];

    for (const delay of delays) {
      const timer = setTimeout(() => {
        if (canvas.parentElement) {
          this.wasm.renderPageToCanvas(pageIdx, canvas, scale);
          this.drawMarginGuides(pageIdx, canvas, scale);
        }
      }, delay);
      timers.push(timer);
    }
    this.reRenderTimers.set(pageIdx, timers);
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
    wrap,
    transform: op.transform,
  };
}
