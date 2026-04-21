import { WasmBridge } from '@/core/wasm-bridge';
import type { PageInfo, PageLayerTree } from '@/core/types';
import { CanvasKitLayerRenderer } from './canvaskit-renderer';
import { clampRenderScale, type RenderBackend } from './render-backend';

export class PageRenderer {
  private reRenderTimers = new Map<number, ReturnType<typeof setTimeout>[]>();
  private layerTreeCache = new Map<number, PageLayerTree>();

  constructor(
    private wasm: WasmBridge,
    private backend: RenderBackend,
    private canvaskitRenderer: CanvasKitLayerRenderer | null,
  ) {}

  /** 페이지를 Canvas에 렌더링한다 (scale = zoom × DPR) */
  renderPage(pageIdx: number, pageInfo: PageInfo, canvas: HTMLCanvasElement, scale: number): void {
    const appliedScale = this.renderContent(pageIdx, pageInfo, canvas, scale);
    this.drawMarginGuides(pageInfo, canvas, appliedScale);
    this.scheduleReRender(pageIdx, pageInfo, canvas, scale);
  }

  getBackend(): RenderBackend {
    return this.backend;
  }

  private renderContent(
    pageIdx: number,
    pageInfo: PageInfo,
    canvas: HTMLCanvasElement,
    scale: number,
  ): number {
    if (this.backend !== 'canvaskit') {
      this.wasm.renderPageToCanvas(pageIdx, canvas, scale);
      return scale;
    }

    if (!this.canvaskitRenderer) {
      throw new Error('CanvasKit renderer가 초기화되지 않았습니다');
    }

    const appliedScale = clampRenderScale(pageInfo, scale);

    canvas.width = Math.max(1, Math.floor(pageInfo.width * appliedScale));
    canvas.height = Math.max(1, Math.floor(pageInfo.height * appliedScale));
    let layerTree = this.layerTreeCache.get(pageIdx);
    if (!layerTree) {
      layerTree = this.wasm.getPageLayerTree(pageIdx);
      this.layerTreeCache.set(pageIdx, layerTree);
    }
    this.canvaskitRenderer.renderPage(layerTree, canvas, appliedScale);
    return appliedScale;
  }

  /** 편집 용지 여백 가이드라인을 캔버스에 그린다 (4모서리 L자 표시) */
  private drawMarginGuides(pageInfo: PageInfo, canvas: HTMLCanvasElement, scale: number): void {
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
  private scheduleReRender(
    pageIdx: number,
    pageInfo: PageInfo,
    canvas: HTMLCanvasElement,
    scale: number,
  ): void {
    this.cancelReRender(pageIdx);

    const delays = [200, 600];
    const timers: ReturnType<typeof setTimeout>[] = [];

    for (const delay of delays) {
      const timer = setTimeout(() => {
        if (canvas.parentElement) {
          const appliedScale = this.renderContent(pageIdx, pageInfo, canvas, scale);
          this.drawMarginGuides(pageInfo, canvas, appliedScale);
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

  clearLayerTreeCache(): void {
    this.layerTreeCache.clear();
  }
}
