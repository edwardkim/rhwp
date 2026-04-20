import { VirtualScroll } from '@/view/virtual-scroll';

/** 핸들 방향 */
export type HandleDirection = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w' | 'rotate';

interface HandleInfo {
  dir: HandleDirection;
  el: HTMLDivElement;
  /** 화면 좌표 기준 중심 (render 시 계산) */
  cx: number;
  cy: number;
}

/** 표/그림 객체 선택 시 외곽선 + 8개 리사이즈 핸들(+회전 핸들)을 표시한다 */
export class TableObjectRenderer {
  private layer: HTMLDivElement;
  private borders: HTMLDivElement[] = [];
  private handles: HandleInfo[] = [];
  private extraEls: HTMLElement[] = [];  // 회전 연결선 등 부가 요소
  private previewEl: HTMLDivElement | null = null;
  private static readonly HANDLE_SIZE = 8; // px (화면 고정)
  private static readonly ROTATE_HANDLE_SIZE = 10; // px
  private static readonly ROTATE_HANDLE_GAP = 20; // px (상단 중앙에서 위로)

  constructor(
    private container: HTMLElement,
    private virtualScroll: VirtualScroll,
    private showRotateHandle = false,
  ) {
    this.layer = document.createElement('div');
    this.layer.className = 'table-object-layer';
    this.layer.style.cssText = 'position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:7;';
    const scrollContent = container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  /** 표 바운딩박스 기준으로 외곽선 + 핸들을 렌더링한다 */
  render(
    tableBBox: { pageIndex: number; x: number; y: number; width: number; height: number },
    zoom: number,
    angleDeg: number = 0,
  ): void {
    if (angleDeg === 0) {
      this.renderMultiPage([tableBBox], zoom);
      return;
    }
    // 회전된 도형: CSS 회전 테두리 + 실제 회전된 꼭짓점에 핸들 배치
    this.clear();
    this.ensureAttached();

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageOffset = this.virtualScroll.getPageOffset(tableBBox.pageIndex);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(tableBBox.pageIndex);
    const pageLeft = (contentWidth - pageDisplayWidth) / 2;

    const left = pageLeft + tableBBox.x * zoom;
    const top = pageOffset + tableBBox.y * zoom;
    const w = tableBBox.width * zoom;
    const h = tableBBox.height * zoom;
    const cx = left + w / 2;
    const cy = top + h / 2;

    const rad = angleDeg * Math.PI / 180;
    const cosA = Math.cos(rad);
    const sinA = Math.sin(rad);
    const rot = (lx: number, ly: number): [number, number] =>
      [cx + lx * cosA - ly * sinA, cy + lx * sinA + ly * cosA];

    // 외곽선 (CSS 회전)
    const border = document.createElement('div');
    border.style.cssText =
      `position:absolute;` +
      `left:${left}px;top:${top}px;` +
      `width:${w}px;height:${h}px;` +
      `border:1px solid #000;box-sizing:border-box;pointer-events:none;` +
      `transform:rotate(${angleDeg}deg);`;
    this.layer.appendChild(border);
    this.borders.push(border);

    // 8개 핸들 (실제 회전된 꼭짓점/변 중점)
    const hs = TableObjectRenderer.HANDLE_SIZE;
    const half = hs / 2;
    const positions: { dir: HandleDirection; lx: number; ly: number }[] = [
      { dir: 'nw', lx: -w / 2, ly: -h / 2 },
      { dir: 'n',  lx: 0,      ly: -h / 2 },
      { dir: 'ne', lx:  w / 2, ly: -h / 2 },
      { dir: 'e',  lx:  w / 2, ly: 0      },
      { dir: 'se', lx:  w / 2, ly:  h / 2 },
      { dir: 's',  lx: 0,      ly:  h / 2 },
      { dir: 'sw', lx: -w / 2, ly:  h / 2 },
      { dir: 'w',  lx: -w / 2, ly: 0      },
    ];
    for (const pos of positions) {
      const [px, py] = rot(pos.lx, pos.ly);
      const el = document.createElement('div');
      el.style.cssText =
        `position:absolute;` +
        `left:${px - half}px;top:${py - half}px;` +
        `width:${hs}px;height:${hs}px;` +
        `background:#fff;border:1px solid #000;box-sizing:border-box;pointer-events:none;`;
      this.layer.appendChild(el);
      this.handles.push({ dir: pos.dir, el, cx: px, cy: py });
    }

    // 회전 핸들 (도형 위쪽 방향으로 gap만큼 이동)
    if (this.showRotateHandle) {
      const rhs = TableObjectRenderer.ROTATE_HANDLE_SIZE;
      const rhalf = rhs / 2;
      const gap = TableObjectRenderer.ROTATE_HANDLE_GAP;
      const [topCx, topCy] = rot(0, -h / 2);
      const [rcx, rcy] = rot(0, -h / 2 - gap);

      // 연결선 (SVG)
      const svgEl = document.createElement('div');
      svgEl.style.cssText = 'position:absolute;left:0;top:0;width:0;height:0;pointer-events:none;';
      svgEl.innerHTML =
        `<svg style="position:absolute;overflow:visible;pointer-events:none;">` +
        `<line x1="${topCx}" y1="${topCy}" x2="${rcx}" y2="${rcy}" stroke="#4CAF50" stroke-width="1"/>` +
        `</svg>`;
      this.layer.appendChild(svgEl);
      this.extraEls.push(svgEl);

      // 원형 회전 핸들
      const el = document.createElement('div');
      el.style.cssText =
        `position:absolute;` +
        `left:${rcx - rhalf}px;top:${rcy - rhalf}px;` +
        `width:${rhs}px;height:${rhs}px;` +
        `background:#4CAF50;border:1px solid #388E3C;border-radius:50%;box-sizing:border-box;pointer-events:none;`;
      this.layer.appendChild(el);
      this.handles.push({ dir: 'rotate', el, cx: rcx, cy: rcy });
    }
  }

  /** 다중 페이지 표 바운딩박스 기준으로 외곽선 + 핸들을 렌더링한다 */
  renderMultiPage(
    bboxes: { pageIndex: number; x: number; y: number; width: number; height: number }[],
    zoom: number,
  ): void {
    this.clear();
    this.ensureAttached();
    if (bboxes.length === 0) return;

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;

    for (const tableBBox of bboxes) {
      const pageOffset = this.virtualScroll.getPageOffset(tableBBox.pageIndex);
      const pageDisplayWidth = this.virtualScroll.getPageWidth(tableBBox.pageIndex);
      const pageLeft = (contentWidth - pageDisplayWidth) / 2;

      const left = pageLeft + tableBBox.x * zoom;
      const top = pageOffset + tableBBox.y * zoom;
      const width = tableBBox.width * zoom;
      const height = tableBBox.height * zoom;

      // 외곽선 — HWP 스타일 (검은색 실선)
      const border = document.createElement('div');
      border.style.cssText =
        `position:absolute;` +
        `left:${left}px;top:${top}px;` +
        `width:${width}px;height:${height}px;` +
        `border:1px solid #000;box-sizing:border-box;pointer-events:none;`;
      this.layer.appendChild(border);
      this.borders.push(border);
    }

    // 각 페이지 bbox마다 8개 핸들 생성
    const hs = TableObjectRenderer.HANDLE_SIZE;
    const half = hs / 2;
    for (const bbox of bboxes) {
      const po = this.virtualScroll.getPageOffset(bbox.pageIndex);
      const pdw = this.virtualScroll.getPageWidth(bbox.pageIndex);
      const pl = (contentWidth - pdw) / 2;
      const l = pl + bbox.x * zoom;
      const t = po + bbox.y * zoom;
      const w = bbox.width * zoom;
      const h = bbox.height * zoom;

      const positions: { dir: HandleDirection; cx: number; cy: number }[] = [
        { dir: 'nw', cx: l, cy: t },
        { dir: 'n', cx: l + w / 2, cy: t },
        { dir: 'ne', cx: l + w, cy: t },
        { dir: 'e', cx: l + w, cy: t + h / 2 },
        { dir: 'se', cx: l + w, cy: t + h },
        { dir: 's', cx: l + w / 2, cy: t + h },
        { dir: 'sw', cx: l, cy: t + h },
        { dir: 'w', cx: l, cy: t + h / 2 },
      ];

      for (const pos of positions) {
        const el = document.createElement('div');
        el.style.cssText =
          `position:absolute;` +
          `left:${pos.cx - half}px;top:${pos.cy - half}px;` +
          `width:${hs}px;height:${hs}px;` +
          `background:#fff;border:1px solid #000;box-sizing:border-box;pointer-events:none;`;
        this.layer.appendChild(el);
        this.handles.push({ dir: pos.dir, el, cx: pos.cx, cy: pos.cy });
      }

      // 회전 핸들 (그림/글상자 전용)
      if (this.showRotateHandle) {
        const rhs = TableObjectRenderer.ROTATE_HANDLE_SIZE;
        const rhalf = rhs / 2;
        const gap = TableObjectRenderer.ROTATE_HANDLE_GAP;
        const rcx = l + w / 2;
        const rcy = t - gap;

        // 연결선: 상단 중앙 → 회전 핸들
        const line = document.createElement('div');
        line.style.cssText =
          `position:absolute;` +
          `left:${rcx}px;top:${rcy}px;` +
          `width:1px;height:${gap}px;` +
          `background:#4CAF50;pointer-events:none;`;
        this.layer.appendChild(line);
        this.extraEls.push(line);

        // 원형 회전 핸들
        const el = document.createElement('div');
        el.style.cssText =
          `position:absolute;` +
          `left:${rcx - rhalf}px;top:${rcy - rhalf}px;` +
          `width:${rhs}px;height:${rhs}px;` +
          `background:#4CAF50;border:1px solid #388E3C;border-radius:50%;box-sizing:border-box;pointer-events:none;`;
        this.layer.appendChild(el);
        this.handles.push({ dir: 'rotate', el, cx: rcx, cy: rcy });
      }
    }
  }

  /** 직선/연결선 개체 선택: 시작점/끝점 핸들 + 직선 표시 (연결선은 중간점 추가) */
  renderLine(
    lineBBox: { pageIndex: number; x1: number; y1: number; x2: number; y2: number;
                x: number; y: number; width: number; height: number },
    zoom: number,
    midPoint?: { x: number; y: number },
  ): void {
    this.clear();
    this.ensureAttached();

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageOffset = this.virtualScroll.getPageOffset(lineBBox.pageIndex);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(lineBBox.pageIndex);
    const pageLeft = (contentWidth - pageDisplayWidth) / 2;

    const sx = pageLeft + lineBBox.x1 * zoom;
    const sy = pageOffset + lineBBox.y1 * zoom;
    const ex = pageLeft + lineBBox.x2 * zoom;
    const ey = pageOffset + lineBBox.y2 * zoom;

    // 직선 표시 (SVG) — 연결선은 점선 불필요 (마커만 표시)
    if (!midPoint) {
      const svgEl = document.createElement('div');
      svgEl.style.cssText = 'position:absolute;left:0;top:0;width:100%;height:100%;pointer-events:none;';
      svgEl.innerHTML = `<svg style="width:100%;height:100%;overflow:visible"><line x1="${sx}" y1="${sy}" x2="${ex}" y2="${ey}" stroke="#000" stroke-width="1" stroke-dasharray="4,2"/></svg>`;
      this.layer.appendChild(svgEl);
      this.extraEls.push(svgEl);
    }

    // 시작점/끝점 핸들 (사각형)
    const hs = TableObjectRenderer.HANDLE_SIZE;
    const half = hs / 2;
    const handlePoints: [HandleDirection, number, number][] = [
      ['sw', sx, sy],
      ['ne', ex, ey],
    ];
    // 연결선 중간점 핸들
    if (midPoint) {
      const mx = pageLeft + midPoint.x * zoom;
      const my = pageOffset + midPoint.y * zoom;
      handlePoints.push(['s', mx, my]); // 's'를 중간점 핸들로 사용
    }
    for (const [dir, cx, cy] of handlePoints) {
      const el = document.createElement('div');
      el.style.cssText =
        `position:absolute;` +
        `left:${cx - half}px;top:${cy - half}px;` +
        `width:${hs}px;height:${hs}px;` +
        `background:#fff;border:1px solid #000;box-sizing:border-box;pointer-events:none;`;
      this.layer.appendChild(el);
      this.handles.push({ dir, el, cx, cy });
    }
  }

  /** 마우스 좌표가 어떤 핸들 위인지 판별한다. scroll-content 기준 좌표. */
  getHandleAtPoint(x: number, y: number): HandleDirection | null {
    // 회전 핸들 우선 검사 (더 작은 원형이므로 먼저 체크)
    for (const h of this.handles) {
      if (h.dir === 'rotate') {
        const rt = TableObjectRenderer.ROTATE_HANDLE_SIZE / 2 + 2;
        if (Math.abs(x - h.cx) <= rt && Math.abs(y - h.cy) <= rt) {
          return h.dir;
        }
      }
    }
    const tolerance = TableObjectRenderer.HANDLE_SIZE / 2 + 2;
    for (const h of this.handles) {
      if (h.dir === 'rotate') continue;
      if (Math.abs(x - h.cx) <= tolerance && Math.abs(y - h.cy) <= tolerance) {
        return h.dir;
      }
    }
    return null;
  }

  /** 모든 오버레이를 제거한다 */
  clear(): void {
    for (const b of this.borders) b.remove();
    this.borders = [];
    for (const h of this.handles) h.el.remove();
    this.handles = [];
    for (const el of this.extraEls) el.remove();
    this.extraEls = [];
    this.clearDragPreview();
  }

  /** 핸들·연결선은 남긴 채 테두리만 제거한다 (드래그 예비선 교체용) */
  private clearBordersOnly(): void {
    for (const b of this.borders) b.remove();
    this.borders = [];
  }

  /** 드래그 예비선을 제거한다 */
  clearDragPreview(): void {
    if (this.previewEl) {
      this.previewEl.remove();
      this.previewEl = null;
    }
  }

  /** 핸들은 그대로 두고 회전각이 적용된 드래그 예비 테두리만 렌더링한다 */
  renderDragPreview(
    bbox: { pageIndex: number; x: number; y: number; width: number; height: number },
    zoom: number,
    angleDeg: number = 0,
  ): void {
    this.clearBordersOnly();
    this.clearDragPreview();
    this.ensureAttached();

    const scrollContent = this.container.querySelector('#scroll-content');
    const contentWidth = scrollContent?.clientWidth ?? 0;
    const pageOffset = this.virtualScroll.getPageOffset(bbox.pageIndex);
    const pageDisplayWidth = this.virtualScroll.getPageWidth(bbox.pageIndex);
    const pageLeft = (contentWidth - pageDisplayWidth) / 2;

    const left = pageLeft + bbox.x * zoom;
    const top = pageOffset + bbox.y * zoom;
    const width = bbox.width * zoom;
    const height = bbox.height * zoom;

    const el = document.createElement('div');
    el.style.cssText =
      `position:absolute;` +
      `left:${left}px;top:${top}px;` +
      `width:${width}px;height:${height}px;` +
      `border:1px solid #000;box-sizing:border-box;pointer-events:none;`;
    if (angleDeg !== 0) {
      el.style.transform = `rotate(${angleDeg}deg)`;
    }
    this.layer.appendChild(el);
    this.previewEl = el;
  }

  /** 레이어가 DOM에 없으면 재부착한다 */
  private ensureAttached(): void {
    if (this.layer.parentElement) return;
    const scrollContent = this.container.querySelector('#scroll-content');
    if (scrollContent) {
      scrollContent.appendChild(this.layer);
    }
  }

  dispose(): void {
    this.clear();
    this.layer.remove();
  }
}
