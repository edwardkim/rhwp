import { EventBus } from '@/core/event-bus';
import { WasmBridge } from '@/core/wasm-bridge';
import type { ParaProperties } from '@/core/types';
import { VirtualScroll } from './virtual-scroll';
import { ViewportManager } from './viewport-manager';
import {
  horizontalPinCommit,
  pageMarginPinX,
  paraIndentPinX,
  pxToHwpunit,
  type HPinDropContext,
  type HPinKind,
  type RulerPinCommit,
} from './ruler-pin-geometry';

export type { RulerPinCommit };

/** 1mm = 96 / 25.4 px (at 96dpi, zoom=1) */
const PX_PER_MM = 96 / 25.4;

/** 눈금자 높이/너비 (CSS px) */
const RULER_SIZE = 20;

/** 문단 마커 크기 (CSS px) */
const MARKER_SIZE = 6;

/** 마커 히트테스트 허용 반경 (CSS px) */
const PIN_HIT_RADIUS = MARKER_SIZE + 2;

/** 핀을 끌 때 반대쪽 핀과의 사이에 남겨야 하는 본문 최소 크기 (mm). */
const MIN_BODY_MM = 10;


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
  private scrollContent: HTMLElement | null;
  private rafId = 0;
  private unsubscribers: (() => void)[] = [];

  /** 현재 커서 문단의 왼쪽 여백 (px, zoom=1 기준). ▽ 핀의 기준점 — 눈금자로 바꾸지는 않는다
   * (문단 여백은 문단 모양 대화상자 소유, 눈금자 △는 쪽 여백 소유). */
  private paraMarginLeftPx = 0;
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

  /** 히트테스트용 — 마지막 프레임에서 draw*가 기록한 핀 위치 (화면 px). y는 삼각형이
   * 실제로 그려진 세로 위치(▽=0, △=canvasH). */
  private hPins: { kind: HPinKind; x: number; y: number }[] = [];
  /** ▽ 핀이 놓인 영역의 좌우 (화면 px) — 셀 안이면 셀, 다단이면 현재 단, 아니면 본문 */
  private hRefLeft = 0;
  private hRefRight = 0;
  /** △ 핀이 커밋할 쪽 — 가로 눈금자는 보이는 첫 페이지를 기준으로 삼는다 */
  private hPageIdx = 0;
  /** 기준 페이지의 왼쪽 끝과 표시 너비 (화면 px) — △ 드롭 위치를 쪽 여백으로 되돌리는 데 쓴다 */
  private hPageLeft = 0;
  private hPageDisplayWidth = 0;
  /** 기준 쪽의 제본 여백과 맞쪽 뒤바꿈 여부 — 본문 경계를 PageDef 필드로 되돌릴 때 쓴다 */
  private hPageGutterPx = 0;
  private hBindingMirrored = false;
  private vPins: { kind: 'top' | 'bottom'; y: number; pageIdx: number }[] = [];

  /** 드래그 중 상태 — draw*가 이 값이 있으면 실제 문서값 대신 이걸 그린다 (라이브 프리뷰).
   *
   * 잡을 때 박는 건 "무엇을 잡았는가"(kind·pageIdx)와 시작 좌표뿐이다. 허용 범위는 매 이동
   * 때 다시 구한다 — 범위를 화면 px로 얼려 두면, 드래그 중 스크롤·확대로 기준 좌표가 바뀔 때
   * 클램프는 옛 화면을, 커밋은 새 화면을 보게 되어 상한이 통째로 무력해진다. */
  private hDrag: { kind: HPinKind; x: number; startX: number } | null = null;
  private vDrag: { kind: 'top' | 'bottom'; pageIdx: number; y: number; startY: number } | null = null;

  private onHPinDownBound: (e: MouseEvent) => void;
  private onHPinHoverBound: (e: MouseEvent) => void;
  private onVPinDownBound: (e: MouseEvent) => void;
  private onVPinHoverBound: (e: MouseEvent) => void;
  private onPinDragMoveBound: (e: MouseEvent) => void;
  private onPinDragUpBound: (e: MouseEvent) => void;

  /** 드래그 커밋 싱크 — 문단 서식/쪽 여백 두 종류를 한 콜백으로 묶는다. main.ts가 하나의
   * 핸들러로 연결한다 — 별개 콜백 두 개였을 때 한쪽만 executeOperation 커밋 경로를 타고
   * 다른 쪽은 wasm을 직접 호출하는 어긋남이 실제로 발생했다(쪽 여백 핀이 모델은 바꿨지만
   * CanvasView가 재플로우하지 않은 사례). */
  onCommitPin: ((commit: RulerPinCommit) => void) | null = null;

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
    this.scrollContent = container.querySelector('#scroll-content');

    this.onHPinDownBound = this.onHPinDown.bind(this);
    this.onHPinHoverBound = this.onHPinHover.bind(this);
    this.onVPinDownBound = this.onVPinDown.bind(this);
    this.onVPinHoverBound = this.onVPinHover.bind(this);
    this.onPinDragMoveBound = this.onPinDragMove.bind(this);
    this.onPinDragUpBound = this.onPinDragUp.bind(this);
    this.hCanvas.addEventListener('mousedown', this.onHPinDownBound);
    this.hCanvas.addEventListener('mousemove', this.onHPinHoverBound);
    this.vCanvas.addEventListener('mousedown', this.onVPinDownBound);
    this.vCanvas.addEventListener('mousemove', this.onVPinHoverBound);

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

  /** 페이지 좌측 화면 좌표를 계산한다 (scroll-container 뷰포트 기준). */
  private getPageScreenLeft(scrollX: number): number {
    // getPageLeftResolved 는 scroll-content 내부 좌표를 준다. 콘텐츠가 컨테이너보다
    // 좁으면 `margin: 0 auto` 가 콘텐츠를 중앙으로 밀어내므로(offsetLeft > 0),
    // 그 오프셋을 더해야 눈금자(컨테이너 기준)와 편집 용지가 일치한다.
    const contentLeft = this.scrollContent?.offsetLeft ?? 0;
    return contentLeft + this.virtualScroll.getPageLeftResolved(
      0,
      this.virtualScroll.getTotalWidth(),
    ) - scrollX;
  }

  /** 커서가 위치한 문단 속성이 변경되었을 때 호출 */
  private onParaChanged(props: ParaProperties): void {
    // WASM API는 ResolvedParaStyle 기반 — 이미 px (96dpi, zoom=1) 단위
    const ml = props.marginLeft ?? 0;
    const ind = props.indent ?? 0;
    if (this.hasParaInfo && ml === this.paraMarginLeftPx && ind === this.paraIndentPx) return;
    this.paraMarginLeftPx = ml;
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

  private onHPinDown(e: MouseEvent): void {
    // 왼쪽 버튼만. 오른쪽 버튼으로 누르면 드래그가 시작된 채 상황 메뉴가 열리고, 메뉴에서
    // 뗀 왼쪽 클릭이 그 자리를 여백으로 커밋한다.
    if (e.button !== 0) return;
    // hasParaInfo 게이트 없음 — ▽(paraIndent)는 hasParaInfo일 때만 drawHorizontal이
    // hPins에 넣으므로 이미 자연히 걸러진다. △(쪽 여백)는 문단 정보와 무관하게 항상
    // 존재해 여기서 막으면 안 된다.
    const rect = this.hCanvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const hit = this.hPins.find(
      (p) => Math.abs(p.x - x) <= PIN_HIT_RADIUS && Math.abs(p.y - y) <= PIN_HIT_RADIUS,
    );
    if (!hit) return;
    this.hDrag = { kind: hit.kind, x, startX: x };
    document.addEventListener('mousemove', this.onPinDragMoveBound);
    document.addEventListener('mouseup', this.onPinDragUpBound);
  }

  /** 마주보는 두 핀 사이에 남겨야 하는 간격 (화면 px).
   *
   * 본문 최소 크기를 화면 배율로 환산하되, 축소 상태에서도 두 핀이 히트테스트로 구분될
   * 만큼은 벌린다 — 10mm는 zoom 0.25에서 9.4px이라 반경 8px 두 개가 겹치고, 겹치면
   * hPins.find가 늘 먼저 넣은 왼쪽 핀을 집어 오른쪽 핀을 잡을 수 없다. */
  private minPinGap(): number {
    return Math.max(MIN_BODY_MM * PX_PER_MM * this.viewportManager.getZoom(), PIN_HIT_RADIUS * 2 + 1);
  }

  /** 잡은 가로 핀이 움직일 수 있는 범위 (화면 px) — 종이 안, 그리고 반대쪽 핀에서
   * 본문 최소 크기만큼 떨어진 곳까지. */
  private hDragRange(kind: HPinKind): { min: number; max: number } {
    const minBody = this.minPinGap();
    const pinX = (k: HPinKind) => this.hPins.find((p) => p.kind === k)?.x;
    const pageRight = this.hPageLeft + this.hPageDisplayWidth;
    if (kind === 'pageMarginLeft') {
      return { min: this.hPageLeft, max: (pinX('pageMarginRight') ?? pageRight) - minBody };
    }
    if (kind === 'pageMarginRight') {
      return { min: (pinX('pageMarginLeft') ?? this.hPageLeft) + minBody, max: pageRight };
    }
    // ▽ 첫 줄은 문단이 놓인 영역 안 — 왼쪽은 문단 여백(첫 줄은 그 왼쪽으로 못 간다),
    // 오른쪽은 영역 끝에서 본문 최소 폭만큼 앞.
    return {
      min: this.hRefLeft + this.paraMarginLeftPx * this.viewportManager.getZoom(),
      max: this.hRefRight - minBody,
    };
  }

  private onHPinHover(e: MouseEvent): void {
    if (this.hDrag) return;
    const rect = this.hCanvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    this.hCanvas.style.cursor =
      this.hPins.some((p) => Math.abs(p.x - x) <= PIN_HIT_RADIUS && Math.abs(p.y - y) <= PIN_HIT_RADIUS)
        ? 'ew-resize' : 'default';
  }

  private onVPinDown(e: MouseEvent): void {
    if (e.button !== 0) return;
    const y = e.clientY - this.vCanvas.getBoundingClientRect().top;
    const hit = this.vPins.find((p) => Math.abs(p.y - y) <= PIN_HIT_RADIUS);
    if (!hit) return;
    this.vDrag = { kind: hit.kind, pageIdx: hit.pageIdx, y, startY: y };
    document.addEventListener('mousemove', this.onPinDragMoveBound);
    document.addEventListener('mouseup', this.onPinDragUpBound);
  }

  /** 잡은 세로 핀이 움직일 수 있는 범위 (화면 px) — 머리말/꼬리말 안쪽부터 반대쪽 핀까지 */
  private vDragRange(kind: 'top' | 'bottom', pageIdx: number): { min: number; max: number } {
    const zoom = this.viewportManager.getZoom();
    const pageTop = this.virtualScroll.getPageOffset(pageIdx) - this.viewportManager.getScrollY();
    const info = this.wasm.getPageInfo(pageIdx);
    // 핀은 용지 끝까지 간다. 반대쪽 핀과의 사이에는 머리말·꼬리말이 들어앉으므로,
    // 본문 최소 크기를 남기려면 그 둘까지 함께 비워 둬야 한다.
    const minGap = this.minPinGap() + (info.marginHeader + info.marginFooter) * zoom;
    const pageBottom = pageTop + info.height * zoom;
    const sibling = this.vPins.find((p) => p.pageIdx === pageIdx && p.kind !== kind)?.y;
    return kind === 'top'
      ? { min: pageTop, max: (sibling ?? pageBottom) - minGap }
      : { min: (sibling ?? pageTop) + minGap, max: pageBottom };
  }

  private onVPinHover(e: MouseEvent): void {
    if (this.vDrag) return;
    const y = e.clientY - this.vCanvas.getBoundingClientRect().top;
    this.vCanvas.style.cursor =
      this.vPins.some((p) => Math.abs(p.y - y) <= PIN_HIT_RADIUS) ? 'ns-resize' : 'default';
  }

  /** 허용 범위 안으로 가둔다. 범위가 뒤집힐 만큼 좁은 용지면 min을 택한다(본문 0). */
  private static clampToRange(v: number, range: { min: number; max: number }): number {
    return Math.min(Math.max(v, range.min), Math.max(range.min, range.max));
  }

  private onPinDragMove(e: MouseEvent): void {
    if (this.hDrag) {
      const x = e.clientX - this.hCanvas.getBoundingClientRect().left;
      this.hDrag = { ...this.hDrag, x: Ruler.clampToRange(x, this.hDragRange(this.hDrag.kind)) };
      this.scheduleUpdate();
    } else if (this.vDrag) {
      const y = e.clientY - this.vCanvas.getBoundingClientRect().top;
      const range = this.vDragRange(this.vDrag.kind, this.vDrag.pageIdx);
      this.vDrag = { ...this.vDrag, y: Ruler.clampToRange(y, range) };
      this.scheduleUpdate();
    }
  }

  private onPinDragUp(): void {
    // 커밋보다 먼저 드래그 상태와 전역 리스너를 정리한다. 커밋이 던지면(드래그 중 쪽 수가
    // 줄어 getPageInfo 가 실패하는 경우 등) 정리 코드에 닿지 못해, 이후 화면 어디서 뗀
    // 마우스든 같은 커밋을 다시 쏘는 상태로 세션이 끝난다.
    const hDrag = this.hDrag;
    const vDrag = this.vDrag;
    this.hDrag = null;
    this.vDrag = null;
    document.removeEventListener('mousemove', this.onPinDragMoveBound);
    document.removeEventListener('mouseup', this.onPinDragUpBound);
    this.scheduleUpdate();

    // 움직이지 않았으면 커밋하지 않는다. 히트 반경이 8px이라 핀에서 몇 px 벗어난 지점을
    // 한 번 누르기만 해도 그 좌표가 그대로 새 여백이 되고, 스냅샷·되돌리기 항목이 쌓이며
    // 쪽 여백 변경은 구역 전체 재래핑까지 부른다.
    if (hDrag && hDrag.x !== hDrag.startX) this.commitHDrag(hDrag);
    if (vDrag && vDrag.y !== vDrag.startY) this.commitVDrag(vDrag);
  }

  /** 가로 핀 드롭 → 문서 변경. 소유 규칙(△=쪽 여백, ▽=문단 들여쓰기)과 좌표 역함수는
   * ruler-pin-geometry가 그리는 식과 짝으로 갖고 있다 — 여기서는 마지막 프레임의 기준
   * 좌표만 넘긴다. */
  private commitHDrag(drag: { kind: HPinKind; x: number }): void {
    this.onCommitPin?.(horizontalPinCommit(drag.kind, drag.x, this.hDropContext()));
  }

  private hDropContext(): HPinDropContext {
    return {
      zoom: this.viewportManager.getZoom(),
      pageIdx: this.hPageIdx,
      pageLeft: this.hPageLeft,
      pageDisplayWidth: this.hPageDisplayWidth,
      refLeft: this.hRefLeft,
      paraMarginLeftPx: this.paraMarginLeftPx,
      pageGutterPx: this.hPageGutterPx,
      bindingMirrored: this.hBindingMirrored,
    };
  }

  /** 세로 핀 드롭 위치 → 쪽 여백(HWPUNIT). marginTopPx/marginBottomPx 산식(drawVertical)의
   * 역함수 — 핀이 선 자리가 곧 그 여백이다. */
  private commitVDrag(drag: { kind: 'top' | 'bottom'; pageIdx: number; y: number }): void {
    const zoom = this.viewportManager.getZoom();
    const scrollY = this.viewportManager.getScrollY();
    const pageTop = this.virtualScroll.getPageOffset(drag.pageIdx) - scrollY;
    const info = this.wasm.getPageInfo(drag.pageIdx);

    const px = drag.kind === 'top'
      ? Math.max(0, (drag.y - pageTop) / zoom)
      : Math.max(0, (pageTop + info.height * zoom - drag.y) / zoom);
    this.onCommitPin?.({
      kind: 'pageMargin',
      pageIdx: drag.pageIdx,
      marginKind: drag.kind,
      hwpunit: pxToHwpunit(px),
    });
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

    // 문서를 닫아도 지난 문서의 핀이 배열에 남아 있으면 hover·드래그가 계속 걸리고,
    // 커밋이 getPageInfo 로 들어가 던진다.
    if (this.wasm.pageCount === 0) {
      this.hPins = [];
      ctx.restore();
      return;
    }

    const zoom = this.viewportManager.getZoom();
    const scrollX = this.viewportManager.getScrollX();
    // 기준 쪽은 "지금 보고 있는 쪽" — 구역마다 쪽 여백이 다를 수 있어 0번 쪽으로 고정하면
    // 다른 구역을 보는 동안 눈금이 용지와 어긋나고 △ 드래그가 엉뚱한 구역을 고친다.
    const visiblePages = this.virtualScroll.getVisiblePages(
      this.viewportManager.getScrollY(),
      this.container.clientHeight,
    );
    const pageIdx = visiblePages.length > 0 ? visiblePages[0] : 0;
    const pageInfo = this.wasm.getPageInfo(pageIdx);

    // 페이지 화면 좌표 (편집 용지와 정확히 일치)
    const pageScreenLeft = this.getPageScreenLeft(scrollX);
    const pageDisplayWidth = pageInfo.width * zoom;

    // 본문 영역 = 쪽 여백 안쪽. 드래그 중이면 잡은 △만 마우스 위치로 대체해 배경과 핀이
    // 함께 움직이게 한다 (라이브 프리뷰).
    // 핀은 해석된 본문 상자 경계에 선다 — 원본 여백으로 그리면 제본 여백이 있는 문서에서
    // 글의 시작보다 왼쪽에 서고, 끌어다 맞출수록 본문이 더 밀려 수렴하지 않는다 (#4971).
    const bodyLeftPx = pageMarginPinX('left', pageScreenLeft, pageDisplayWidth, pageInfo.bodyLeft, zoom);
    const bodyRightPx = pageMarginPinX(
      'right', pageScreenLeft, pageDisplayWidth, pageInfo.width - pageInfo.bodyRight, zoom);
    const pageLeftPinX = this.hDrag?.kind === 'pageMarginLeft' ? this.hDrag.x : bodyLeftPx;
    const pageRightPinX = this.hDrag?.kind === 'pageMarginRight' ? this.hDrag.x : bodyRightPx;

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
      ctx.fillRect(pageLeftPinX, 0, pageRightPinX - pageLeftPinX, canvasH);
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

    // 핀 — △ 좌/우는 쪽 여백(PageDef), ▽는 문단 첫 줄 들여쓰기(ParaShape.indent).
    // 여백의 주인은 쪽 하나다: 같은 축·같은 모양의 핀이 쪽 여백과 문단 여백 두 주인을
    // 나눠 가지면 어느 쪽이 움직였는지 눈금자만 봐서는 알 수 없다. 문단 좌우 여백은
    // 문단 모양 대화상자 전용으로 두고, 눈금자 △는 세로 눈금자의 쪽 위/아래 여백 핀과
    // 같은 소유·같은 커밋 경로(PageDef)를 쓴다.
    // 격자 보기에서는 핀을 두지 않는다. 가로 눈금자는 하나인데 한 행에 여러 쪽이 늘어서
    // 있어 어느 쪽의 여백인지 가리킬 수 없고(기준은 늘 행의 첫 쪽이 된다), 세로 핀은 같은
    // 행의 쪽마다 같은 y에 겹쳐 그려져 첫 쪽만 잡힌다. 쪽 여백은 편집 용지 대화상자에서.
    this.hPins = [];
    if (this.virtualScroll.isGridMode()) {
      ctx.restore();
      return;
    }
    ctx.fillStyle = palette.marker;

    this.hPageIdx = pageIdx;
    this.hPageLeft = pageScreenLeft;
    this.hPageDisplayWidth = pageDisplayWidth;
    this.hPageGutterPx = pageInfo.marginGutter ?? 0;
    this.hBindingMirrored = pageInfo.bindingMirrored ?? false;
    this.hPins.push(
      { kind: 'pageMarginLeft', x: pageLeftPinX, y: canvasH },
      { kind: 'pageMarginRight', x: pageRightPinX, y: canvasH },
    );
    this.drawTriangleUp(ctx, pageLeftPinX, canvasH, MARKER_SIZE);
    this.drawTriangleUp(ctx, pageRightPinX, canvasH, MARKER_SIZE);

    if (this.hasParaInfo) {
      // 문단이 놓인 영역 — 셀 안이면 셀 경계, 다단이면 현재 단 경계, 아니면 본문 영역.
      // ▽의 기준점이자 ▽가 벗어날 수 없는 범위다.
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

      let firstX = paraIndentPinX(refLeft, this.paraMarginLeftPx, this.paraIndentPx, zoom);
      if (this.hDrag?.kind === 'paraIndent') firstX = this.hDrag.x;

      this.hRefLeft = refLeft;
      this.hRefRight = refRight;
      this.hPins.push({ kind: 'paraIndent', x: firstX, y: 0 });
      this.drawTriangleDown(ctx, firstX, 0, MARKER_SIZE);
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
      this.vPins = [];
      ctx.restore();
      return;
    }

    const zoom = this.viewportManager.getZoom();
    const scrollY = this.viewportManager.getScrollY();
    const mmPx = PX_PER_MM * zoom;

    // 보이는 페이지 범위에서만 그리기
    const vpHeight = canvasH;
    const visiblePages = this.virtualScroll.getVisiblePages(scrollY, vpHeight);
    this.vPins = [];
    // 격자 보기: 같은 행의 쪽들이 같은 y를 공유해 핀이 겹친다 (가로 눈금자와 같은 이유).
    const gridMode = this.virtualScroll.isGridMode();

    for (const pageIdx of visiblePages) {
      // 페이지 상단의 화면 좌표 (scroll-container 뷰포트 기준)
      const pageScreenTop = this.virtualScroll.getPageOffset(pageIdx) - scrollY;
      const pageInfo = this.wasm.getPageInfo(pageIdx);

      // 핀은 자기가 쓰는 여백의 경계에 선다 — 용지 끝에서 marginTop/marginBottom 만큼.
      // 본문 위 끝(= marginTop + marginHeader)에 그리면 핀 위치와 커밋 값이 머리말만큼
      // 어긋나, marginTop 이 0 이 되는 자리에서 더 못 올라가고 그 위 머리말 폭은 눈금자로
      // 손댈 수단이 없었다.
      const marginTopPx = pageScreenTop + pageInfo.marginTop * zoom;
      const marginBottomPx = pageScreenTop + (pageInfo.height - pageInfo.marginBottom) * zoom;
      // 밝은 띠 = 쪽 여백 안쪽. 핀 두 개가 그 띠의 양 끝이다 — 가로 눈금자와 같은 읽는 법이다
      // (가로는 머리말이 없어 "여백 안쪽"이 곧 본문이다). 머리말/꼬리말을 세 번째 톤으로
      // 나누는 안은 버렸다: 어두운 테마의 여백↔본문 색차가 rgb 9 뿐이라 그 사이에 낀 톤은
      // 보이지 않고, 핀이 무엇을 가리키는지만 흐려진다.
      ctx.fillStyle = palette.bgBody;
      ctx.fillRect(0, marginTopPx, canvasW, marginBottomPx - marginTopPx);

      // 드래그 중이면 잡은 핀만 마우스 위치로 대체 (라이브 프리뷰)
      const topY = (this.vDrag?.kind === 'top' && this.vDrag.pageIdx === pageIdx) ? this.vDrag.y : marginTopPx;
      const bottomY = (this.vDrag?.kind === 'bottom' && this.vDrag.pageIdx === pageIdx) ? this.vDrag.y : marginBottomPx;
      if (!gridMode) {
        this.vPins.push({ kind: 'top', y: topY, pageIdx }, { kind: 'bottom', y: bottomY, pageIdx });
      }

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

      // 위/아래 여백 핀 — 본문 시작(▽)과 끝(△)을 표시 (가로 눈금자 마커와 동일 팔레트)
      ctx.fillStyle = palette.marker;
      this.drawTriangleDown(ctx, canvasW / 2, topY, MARKER_SIZE);
      this.drawTriangleUp(ctx, canvasW / 2, bottomY, MARKER_SIZE);
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

    this.hCanvas.removeEventListener('mousedown', this.onHPinDownBound);
    this.hCanvas.removeEventListener('mousemove', this.onHPinHoverBound);
    this.vCanvas.removeEventListener('mousedown', this.onVPinDownBound);
    this.vCanvas.removeEventListener('mousemove', this.onVPinHoverBound);
    document.removeEventListener('mousemove', this.onPinDragMoveBound);
    document.removeEventListener('mouseup', this.onPinDragUpBound);
  }
}
