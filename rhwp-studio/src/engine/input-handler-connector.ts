/** 연결선 드로잉 모드 — input-handler에서 호출되는 헬퍼 함수 */
/* eslint-disable @typescript-eslint/no-explicit-any */

/** 개체의 4방향 연결점 (상/우/하/좌 중점) */
export interface ConnectionPoint {
  /** 페이지 내 절대 X (page px) */
  x: number;
  /** 페이지 내 절대 Y (page px) */
  y: number;
  /** 연결점 인덱스 (0=상, 1=우, 2=하, 3=좌) */
  index: number;
  /** 소속 개체 참조 */
  sec: number;
  ppi: number;
  ci: number;
  /** 개체 instance_id (SubjectID용) */
  instanceId: number;
}

/** 페이지 위의 모든 도형에서 마우스에 가장 가까운 연결점을 찾는다. */
export function findNearestConnectionPoint(
  this: any,
  pageIdx: number,
  pageX: number,
  pageY: number,
  threshold: number = 10,
  excludeRef?: { sec: number; ppi: number; ci: number },
): ConnectionPoint | null {
  try {
    const layout = this.wasm.getPageControlLayout(pageIdx);
    let best: ConnectionPoint | null = null;
    let bestDist = threshold;

    for (const ctrl of layout.controls) {
      // 연결 가능 개체: shape, image, group (표/수식/선 제외)
      if (ctrl.type !== 'shape' && ctrl.type !== 'image' && ctrl.type !== 'group') continue;
      if (ctrl.secIdx === undefined || ctrl.paraIdx === undefined || ctrl.controlIdx === undefined) continue;
      // 시작 개체 자신은 제외
      if (excludeRef && ctrl.secIdx === excludeRef.sec && ctrl.paraIdx === excludeRef.ppi && ctrl.controlIdx === excludeRef.ci) continue;

      const cx = ctrl.x + ctrl.w / 2;
      const cy = ctrl.y + ctrl.h / 2;
      // 4방향 연결점
      const points = [
        { x: cx, y: ctrl.y, index: 0 },           // 상
        { x: ctrl.x + ctrl.w, y: cy, index: 1 },  // 우
        { x: cx, y: ctrl.y + ctrl.h, index: 2 },  // 하
        { x: ctrl.x, y: cy, index: 3 },            // 좌
      ];

      for (const pt of points) {
        const dist = Math.hypot(pageX - pt.x, pageY - pt.y);
        if (dist < bestDist) {
          bestDist = dist;
          // instanceId는 getShapeProperties에서 조회 (나중에 연결 시)
          best = {
            x: pt.x, y: pt.y, index: pt.index,
            sec: ctrl.secIdx, ppi: ctrl.paraIdx, ci: ctrl.controlIdx,
            instanceId: 0, // 나중에 채움
          };
        }
      }
    }
    return best;
  } catch {
    return null;
  }
}

/** 연결점 오버레이: 호버된 개체의 4방향 연결점을 표시 */
export function showConnectionPointOverlay(
  this: any,
  pageIdx: number,
  pageX: number,
  pageY: number,
): void {
  removeConnectionPointOverlay.call(this);

  const layout = this.wasm.getPageControlLayout(pageIdx);
  const sc = this.container.querySelector('#scroll-content');
  if (!sc) return;
  const zoom = this.viewportManager.getZoom();
  const po = this.virtualScroll.getPageOffset(pageIdx);
  const pw = this.virtualScroll.getPageWidth(pageIdx);
  const pl = this.virtualScroll.getPageLeftResolved(pageIdx, sc.clientWidth);

  // 마우스 근처 개체 찾기 (bbox 내부)
  for (const ctrl of layout.controls) {
    if (ctrl.type !== 'shape' && ctrl.type !== 'image' && ctrl.type !== 'group') continue;
    if (pageX < ctrl.x || pageX > ctrl.x + ctrl.w || pageY < ctrl.y || pageY > ctrl.y + ctrl.h) continue;

    const cx = ctrl.x + ctrl.w / 2;
    const cy = ctrl.y + ctrl.h / 2;
    const points = [
      { x: cx, y: ctrl.y },
      { x: ctrl.x + ctrl.w, y: cy },
      { x: cx, y: ctrl.y + ctrl.h },
      { x: ctrl.x, y: cy },
    ];

    const overlay = document.createElement('div');
    overlay.className = 'connector-points-overlay';
    overlay.style.position = 'absolute';
    overlay.style.pointerEvents = 'none';
    overlay.style.zIndex = '1000';

    for (const pt of points) {
      const screenX = pl + pt.x * zoom;
      const screenY = po + pt.y * zoom;
      const dot = document.createElement('div');
      dot.style.cssText = `
        position: absolute; left: ${screenX - 4}px; top: ${screenY - 4}px;
        width: 8px; height: 8px; background: white; border: 1.5px solid #4a90d9;
        pointer-events: none;
      `;
      // 가까운 점은 연두색
      const dist = Math.hypot(pageX - pt.x, pageY - pt.y);
      if (dist < 10) {
        dot.style.background = '#7fdb7f';
        dot.style.borderColor = '#2d8a2d';
      }
      overlay.appendChild(dot);
    }

    sc.appendChild(overlay);
    this.connectorOverlay = overlay;
    break; // 첫 번째 매칭 개체만
  }
}

/** 연결점 오버레이 제거 */
export function removeConnectionPointOverlay(this: any): void {
  if (this.connectorOverlay) {
    this.connectorOverlay.remove();
    this.connectorOverlay = null;
  }
}

/** 연결선 드로잉 미리보기 (시작점에서 현재 마우스까지 선) */
export function updateConnectorPreview(
  this: any,
  startX: number, startY: number,
  endX: number, endY: number,
  pageIdx: number,
): void {
  removeConnectorPreview.call(this);
  const sc = this.container.querySelector('#scroll-content');
  if (!sc) return;
  const zoom = this.viewportManager.getZoom();
  const po = this.virtualScroll.getPageOffset(pageIdx);
  const pw = this.virtualScroll.getPageWidth(pageIdx);
  const pl = this.virtualScroll.getPageLeftResolved(pageIdx, sc.clientWidth);

  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('class', 'connector-preview');
  svg.style.cssText = `position: absolute; left: 0; top: 0; width: 100%; height: 100%; pointer-events: none; z-index: 999;`;

  const x1 = pl + startX * zoom;
  const y1 = po + startY * zoom;
  const x2 = pl + endX * zoom;
  const y2 = po + endY * zoom;

  const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  line.setAttribute('x1', String(x1));
  line.setAttribute('y1', String(y1));
  line.setAttribute('x2', String(x2));
  line.setAttribute('y2', String(y2));
  line.setAttribute('stroke', '#4a90d9');
  line.setAttribute('stroke-width', '1.5');
  line.setAttribute('stroke-dasharray', '4,4');
  svg.appendChild(line);

  sc.appendChild(svg);
  (this as any)._connectorPreviewSvg = svg;
}

export function removeConnectorPreview(this: any): void {
  if ((this as any)._connectorPreviewSvg) {
    (this as any)._connectorPreviewSvg.remove();
    (this as any)._connectorPreviewSvg = null;
  }
}

/** 연결선 드로잉 완료: 시작/끝 연결점으로 연결선 생성 */
export function finishConnectorDrawing(
  this: any,
  startPt: ConnectionPoint,
  endPt: ConnectionPoint,
  connectorType: string,
): void {
  removeConnectorPreview.call(this);
  removeConnectionPointOverlay.call(this);

  // SHAPE_COMPONENT inst_id 조회 (= SubjectID, 변환 불필요)
  try {
    const startProps = this.wasm.getShapeProperties(startPt.sec, startPt.ppi, startPt.ci);
    startPt.instanceId = startProps.scInstId ?? startProps.instanceId ?? 0;
  } catch { /* ignore */ }
  try {
    const endProps = this.wasm.getShapeProperties(endPt.sec, endPt.ppi, endPt.ci);
    endPt.instanceId = endProps.scInstId ?? endProps.instanceId ?? 0;
  } catch { /* ignore */ }

  // 연결선의 위치/크기 계산 (HWPUNIT)
  const PX2HWP = 7200 / 96;
  const minX = Math.min(startPt.x, endPt.x);
  const minY = Math.min(startPt.y, endPt.y);
  const maxX = Math.max(startPt.x, endPt.x);
  const maxY = Math.max(startPt.y, endPt.y);
  const wHwp = Math.max(Math.round((maxX - minX) * PX2HWP), 1);
  const hHwp = Math.max(Math.round((maxY - minY) * PX2HWP), 1);
  const horzOffset = Math.round(minX * PX2HWP);
  const vertOffset = Math.round(minY * PX2HWP);
  const lineFlipX = startPt.x > endPt.x;
  const lineFlipY = startPt.y > endPt.y;

  // 커서 위치에서 sec/para/charOffset
  const pos = this.cursor.getPosition();
  const sec = pos.sectionIndex;
  const paraIdx = pos.paragraphIndex;
  const charOffset = pos.charOffset;

  try {
    const result = this.wasm.createShapeControl({
      sectionIdx: sec,
      paraIdx,
      charOffset,
      width: wHwp,
      height: hHwp,
      horzOffset,
      vertOffset,
      shapeType: connectorType,
      lineFlipX,
      lineFlipY,
      // 연결선 전용 필드
      startSubjectID: startPt.instanceId,
      startSubjectIndex: startPt.index,
      endSubjectID: endPt.instanceId,
      endSubjectIndex: endPt.index,
    });
    if (result.ok) {
      this.eventBus.emit('document-changed');
      this.cursor.enterPictureObjectSelectionDirect(sec, result.paraIdx, result.controlIdx, 'line');
      this.caret.hide();
      this.selectionRenderer.clear();
      this.renderPictureObjectSelection();
      this.eventBus.emit('picture-object-selection-changed', true);
    }
  } catch (err) {
    console.warn('[Connector] 연결선 생성 실패:', err);
  }
}

/** 연결선 드로잉 모드 종료 */
export function exitConnectorDrawingMode(this: any): void {
  this.connectorDrawingMode = false;
  this.connectorStartRef = null;
  removeConnectorPreview.call(this);
  removeConnectionPointOverlay.call(this);
  this.container.style.cursor = '';
}
