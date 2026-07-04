import { EventBus } from '@/core/event-bus';
import { WasmBridge } from '@/core/wasm-bridge';
import type { ParaProperties } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { ViewportManager } from './viewport-manager';

/** 1mm = 96 / 25.4 px (at 96dpi, zoom=1) */
const PX_PER_MM = 96 / 25.4;

/** 눈금자 높이/너비 (CSS px) */
const RULER_SIZE = 20;

/** 문단 마커 크기 (CSS px) */
const MARKER_SIZE = 6;

interface RulerPalette {
  bgMargin: string;
  bgBody: string;
  tick: string;
  text: string;
  marker: string;
}

function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

export class Ruler {
  private hCtx: CanvasRenderingContext2D | null;
  private vCtx: CanvasRenderingContext2D | null;
  private rafId = 0;
  private unsubscribers: (() => void)[] = [];

  /** 현재 커서 문단의 왼쪽 여백 (px, zoom=1 기준) */
  private paraMarginLeftPx = 0;
  /** 현재 커서 문단의 오른쪽 여백 (px, zoom=1 기준) */
  private paraMarginRightPx = 0;
  /** 현재 커서 문단의 첫 줄 들여쓰기 (px, zoom=1 기준, 음수 = 내어쓰기) */
  private paraIndentPx = 0;
  /** 문단 정보가 유효한지 여부 */
  private hasParaInfo = false;

  /** 셀 내부 여부 및 셀 좌표 (px, zoom=1, 페이지 좌표 기준) */
  private inCell = false;
  private cellX = 0;
  private cellWidth = 0;

  /** 커서의 x 좌표 (px, zoom=1, 페이지 좌표 기준) — 다단에서 현재 단 결정용 */
  private cursorColumnX = 0;

  constructor(
    private hCanvas: HTMLCanvasElement,
    private vCanvas: HTMLCanvasElement,
    private container: HTMLElement,
    private eventBus: EventBus,
    private wasm: WasmBridge,
    private virtualScroll: VirtualScroll,
    private viewportManager: ViewportManager,
  ) {
    this.hCtx = hCanvas.getContext('2d');
    this.vCtx = vCanvas.getContext('2d');

    this.unsubscribers.push(
      eventBus.on('viewport-scroll', () => this.scheduleUpdate()),
      eventBus.on('zoom-changed', () => this.scheduleUpdate()),
      eventBus.on('viewport-resize', () => { this.resize(); this.scheduleUpdate(); }),
      eventBus.on('document-changed', () => this.scheduleUpdate()),
      eventBus.on('document-view-changed', () => this.scheduleUpdate()),
      eventBus.on('theme-changed', () => this.scheduleUpdate()),
      eventBus.on('cursor-para-changed', (props) => this.onParaChanged(props as ParaProperties)),
      eventBus.on('cursor-cell-changed', (data) => this.onCellChanged(data as { inCell: boolean; cellX?: number; cellWidth?: number })),
      eventBus.on('cursor-rect-updated', (rect: any) => {
        if (rect && typeof rect.x === 'number') {
          this.cursorColumnX = rect.x;
          this.scheduleUpdate();
        }
      }),
    );

    this.resize();
  }

  private palette(): RulerPalette {
    return {
      bgMargin: cssVar('--ruler-bg', '#d0d0d0'),
      bgBody: cssVar('--ruler-body', '#ffffff'),
      tick: cssVar('--ruler-tick', '#555555'),
      text: cssVar('--ruler-text', '#333333'),
      marker: cssVar('--ruler-marker', '#4080c0'),
    };
  }

  /** Canvas 물리 크기를 컨테이너에 맞춰 설정 */
  resize(): void {
    const dpr = window.devicePixelRatio || 1;

    // 가로 눈금자: 너비 = scroll-container 너비, 높이 = RULER_SIZE
    const hW = this.container.clientWidth;
    this.hCanvas.width = Math.round(hW * dpr);
    this.hCanvas.height = Math.round(RULER_SIZE * dpr);
    this.hCanvas.style.width = `${hW}px`;
    this.hCanvas.style.height = `${RULER_SIZE}px`;

    // 세로 눈금자: 너비 = RULER_SIZE, 높이 = scroll-container 높이
    const vH = this.container.clientHeight;
    this.vCanvas.width = Math.round(RULER_SIZE * dpr);
    this.vCanvas.height = Math.round(vH * dpr);
    this.vCanvas.style.width = `${RULER_SIZE}px`;
    this.vCanvas.style.height = `${vH}px`;
  }

  /** requestAnimationFrame으로 스로틀링하여 그리기 예약 */
  private scheduleUpdate(): void {
    if (this.rafId) return;
    this.rafId = requestAnimationFrame(() => {
      this.rafId = 0;
      this.update();
    });
  }

  /** 가로/세로 눈금자를 모두 다시 그린다 */
  update(): void {
    this.drawHorizontal();
    this.drawVertical();
  }

  /**
   * 페이지 좌측 화면 좌표를 계산한다 (scroll-container 뷰포트 기준).
   * scroll-content는 margin: 0 auto로 가운데 정렬되므로,
   * 컨테이너가 scroll-content보다 넓으면 auto-margin 오프셋을 반영한다.
   */
  private getPageScreenLeft(pageInfo: { width: number }, zoom: number, scrollX: number): number {
    const scrollContentWidth = this.virtualScroll.getMaxPageWidth() + 40;
    const containerWidth = this.container.clientWidth;
    // margin: 0 auto에 의한 오프셋
    const contentOffsetX = Math.max(0, (containerWidth - scrollContentWidth) / 2);
    const pageDisplayWidth = pageInfo.width * zoom;
    // scroll-content 내에서 페이지 가운데 정렬 (left: 50%; transform: translateX(-50%))
    const pageLeftInContent = (scrollContentWidth - pageDisplayWidth) / 2;
    return contentOffsetX + pageLeftInContent - scrollX;
  }

  /** 커서가 위치한 문단 속성이 변경되었을 때 호출 */
  private onParaChanged(props: ParaProperties): void {
    // WASM API는 ResolvedParaStyle 기반 — 이미 px (96dpi, zoom=1) 단위
    const ml = props.marginLeft ?? 0;
    const mr = props.marginRight ?? 0;
    const ind = props.indent ?? 0;
    if (this.hasParaInfo
      && ml === this.paraMarginLeftPx
      && mr === this.paraMarginRightPx
      && ind === this.paraIndentPx) return;
    this.paraMarginLeftPx = ml;
    this.paraMarginRightPx = mr;
    this.paraIndentPx = ind;
    this.hasParaInfo = true;
    this.scheduleUpdate();
  }

  /** 커서가 셀 안/밖으로 이동했을 때 호출 */
  private onCellChanged(data: { inCell: boolean; cellX?: number; cellWidth?: number }): void {
    if (data.inCell && data.cellX !== undefined && data.cellWidth !== undefined) {
      if (this.inCell && data.cellX === this.cellX && data.cellWidth === this.cellWidth) return;
      this.inCell = true;
      this.cellX = data.cellX;
      this.cellWidth = data.cellWidth;
    } else {
      if (!this.inCell) return; // 셀 밖→셀 밖: 변경 없음
      this.inCell = false;
    }
    this.scheduleUpdate();
  }

  /** 아래쪽을 가리키는 삼각형 ▽ (첫 줄 시작 위치 마커) */
  private drawTriangleDown(ctx: CanvasRenderingContext2D, cx: number, top: number, size: number): void {
    ctx.beginPath();
    ctx.moveTo(cx - size / 2, top);
    ctx.lineTo(cx + size / 2, top);
    ctx.lineTo(cx, top + size);
    ctx.closePath();
    ctx.fill();
  }

  /** 위쪽을 가리키는 삼각형 △ (나머지 줄 시작 위치 마커) */
  private drawTriangleUp(ctx: CanvasRenderingContext2D, cx: number, bottom: number, size: number): void {
    ctx.beginPath();
    ctx.moveTo(cx - size / 2, bottom);
    ctx.lineTo(cx + size / 2, bottom);
    ctx.lineTo(cx, bottom - size);
    ctx.closePath();
    ctx.fill();
  }

  /** 가로 눈금자 그리기 */
  private drawHorizontal(): void {
    const ctx = this.hCtx;
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const canvasW = this.hCanvas.width / dpr;
    const canvasH = RULER_SIZE;

    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const palette = this.palette();

    // 전체 배경 (여백색)
    ctx.fillStyle = palette.bgMargin;
    ctx.fillRect(0, 0, canvasW, canvasH);

    if (this.wasm.pageCount === 0) {
      ctx.restore();
      return;
    }

    const zoom = this.viewportManager.getZoom();
    const scrollX = this.viewportManager.getScrollX();
    const pageInfo = this.wasm.getPageInfo(0);

    // 페이지 화면 좌표 (편집 용지와 정확히 일치)
    const pageScreenLeft = this.getPageScreenLeft(pageInfo, zoom, scrollX);
    const pageDisplayWidth = pageInfo.width * zoom;

    // 본문 영역 배경
    const bodyLeftPx = pageScreenLeft + pageInfo.marginLeft * zoom;
    const bodyRightPx = pageScreenLeft + pageDisplayWidth - pageInfo.marginRight * zoom;

    if (this.inCell) {
      // 셀 모드: 셀 영역만 본문 톤, 나머지는 여백 톤
      const cellLeftPx = pageScreenLeft + this.cellX * zoom;
      const cellRightPx = pageScreenLeft + (this.cellX + this.cellWidth) * zoom;
      ctx.fillStyle = palette.bgBody;
      ctx.fillRect(cellLeftPx, 0, cellRightPx - cellLeftPx, canvasH);
    } else if (pageInfo.columns && pageInfo.columns.length > 1) {
      // 다단 모드: 현재 커서가 위치한 단만 본문 톤으로 표시
      const cursorX = this.cursorColumnX;
      let activeCol = 0;
      for (let i = 0; i < pageInfo.columns.length; i++) {
        const col = pageInfo.columns[i];
        if (cursorX >= col.x && cursorX < col.x + col.width) {
          activeCol = i;
          break;
        }
      }
      const col = pageInfo.columns[activeCol];
      const colLeft = pageScreenLeft + col.x * zoom;
      const colRight = pageScreenLeft + (col.x + col.width) * zoom;
      ctx.fillStyle = palette.bgBody;
      ctx.fillRect(colLeft, 0, colRight - colLeft, canvasH);
    } else {
      ctx.fillStyle = palette.bgBody;
      ctx.fillRect(bodyLeftPx, 0, bodyRightPx - bodyLeftPx, canvasH);
    }

    // mm 눈금 그리기
    const mmPx = PX_PER_MM * zoom;
    const pageWidthMm = Math.ceil(pageInfo.width / PX_PER_MM);

    ctx.strokeStyle = palette.tick;
    ctx.fillStyle = palette.text;
    ctx.lineWidth = 0.5;
    ctx.font = '9px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';

    for (let mm = 0; mm <= pageWidthMm; mm++) {
      const x = pageScreenLeft + mm * mmPx;

      // 화면 밖 스킵
      if (x < -10 || x > canvasW + 10) continue;

      let tickH: number;
      if (mm % 10 === 0) {
        tickH = 10;
        // 10mm 단위 숫자 (cm 단위로 표시)
        const cm = mm / 10;
        if (cm > 0) {
          ctx.fillText(`${cm}`, x, 1);
        }
      } else if (mm % 5 === 0) {
        tickH = 6;
      } else {
        tickH = 3;
      }

      ctx.beginPath();
      ctx.moveTo(x, canvasH);
      ctx.lineTo(x, canvasH - tickH);
      ctx.stroke();
    }

    // 문단 들여쓰기 마커 (▽ 첫 줄, △ 나머지 줄, △ 오른쪽)
    if (this.hasParaInfo) {
      ctx.fillStyle = palette.marker;

      // 셀 안이면 셀 경계, 다단이면 현재 단 경계, 아니면 본문 영역 기준
      let refLeft: number;
      let refRight: number;
      if (this.inCell) {
        refLeft = pageScreenLeft + this.cellX * zoom;
        refRight = pageScreenLeft + (this.cellX + this.cellWidth) * zoom;
      } else if (pageInfo.columns && pageInfo.columns.length > 1) {
        let activeCol = 0;
        for (let i = 0; i < pageInfo.columns.length; i++) {
          const col = pageInfo.columns[i];
          if (this.cursorColumnX >= col.x && this.cursorColumnX < col.x + col.width) {
            activeCol = i;
            break;
          }
        }
        const col = pageInfo.columns[activeCol];
        refLeft = pageScreenLeft + col.x * zoom;
        refRight = pageScreenLeft + (col.x + col.width) * zoom;
      } else {
        refLeft = bodyLeftPx;
        refRight = bodyRightPx;
      }

      let firstX: number;
      let remainX: number;

      if (this.paraIndentPx >= 0) {
        // 들여쓰기: 나머지 줄은 marginLeft, 첫 줄은 marginLeft + indent
        remainX = refLeft + this.paraMarginLeftPx * zoom;
        firstX = refLeft + (this.paraMarginLeftPx + this.paraIndentPx) * zoom;
      } else {
        // 내어쓰기: 첫 줄은 marginLeft, 나머지 줄은 marginLeft + |indent|
        firstX = refLeft + this.paraMarginLeftPx * zoom;
        remainX = refLeft + (this.paraMarginLeftPx - this.paraIndentPx) * zoom;
      }

      this.drawTriangleDown(ctx, firstX, 0, MARKER_SIZE);
      this.drawTriangleUp(ctx, remainX, canvasH, MARKER_SIZE);

      // 오른쪽 여백 마커 △
      const rightX = refRight - this.paraMarginRightPx * zoom;
      this.drawTriangleUp(ctx, rightX, canvasH, MARKER_SIZE);
    }

    ctx.restore();
  }

  /** 세로 눈금자 그리기 — 보이는 모든 페이지의 눈금을 각각 표시 */
  private drawVertical(): void {
    const ctx = this.vCtx;
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const canvasW = RULER_SIZE;
    const canvasH = this.vCanvas.height / dpr;

    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const palette = this.palette();

    // 전체 배경 (여백색)
    ctx.fillStyle = palette.bgMargin;
    ctx.fillRect(0, 0, canvasW, canvasH);

    if (this.wasm.pageCount === 0) {
      ctx.restore();
      return;
    }

    const zoom = this.viewportManager.getZoom();
    const scrollY = this.viewportManager.getScrollY();
    const mmPx = PX_PER_MM * zoom;

    // 보이는 페이지 범위에서만 그리기
    const vpHeight = canvasH;
    const visiblePages = this.virtualScroll.getVisiblePages(scrollY, vpHeight);

    for (const pageIdx of visiblePages) {
      // 페이지 상단의 화면 좌표 (scroll-container 뷰포트 기준)
      const pageScreenTop = this.virtualScroll.getPageOffset(pageIdx) - scrollY;
      const pageInfo = this.wasm.getPageInfo(pageIdx);

      // 본문 영역 배경
      const bodyTopPx = pageScreenTop + (pageInfo.marginHeader + pageInfo.marginTop) * zoom;
      const bodyBottomPx = pageScreenTop + pageInfo.height * zoom - (pageInfo.marginFooter + pageInfo.marginBottom) * zoom;
      ctx.fillStyle = palette.bgBody;
      ctx.fillRect(0, bodyTopPx, canvasW, bodyBottomPx - bodyTopPx);

      // mm 눈금 그리기
      const pageHeightMm = Math.ceil(pageInfo.height / PX_PER_MM);

      ctx.strokeStyle = palette.tick;
      ctx.fillStyle = palette.text;
      ctx.lineWidth = 0.5;
      ctx.font = '9px sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      for (let mm = 0; mm <= pageHeightMm; mm++) {
        const y = pageScreenTop + mm * mmPx;

        // 화면 밖 스킵
        if (y < -10 || y > canvasH + 10) continue;

        let tickW: number;
        if (mm % 10 === 0) {
          tickW = 10;
          // 10mm 단위 숫자 (cm 단위, 세로 텍스트)
          const cm = mm / 10;
          if (cm > 0) {
            ctx.save();
            ctx.translate(canvasW / 2 - 2, y);
            ctx.rotate(-Math.PI / 2);
            ctx.fillText(`${cm}`, 0, 0);
            ctx.restore();
          }
        } else if (mm % 5 === 0) {
          tickW = 6;
        } else {
          tickW = 3;
        }

        ctx.beginPath();
        ctx.moveTo(canvasW, y);
        ctx.lineTo(canvasW - tickW, y);
        ctx.stroke();
      }
    }

    ctx.restore();
  }

  /** 리소스 정리 */
  dispose(): void {
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = 0;
    }
    for (const unsub of this.unsubscribers) {
      unsub();
    }
    this.unsubscribers = [];
  }
}
