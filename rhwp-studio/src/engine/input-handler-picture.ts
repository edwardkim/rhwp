/** input-handler picture/shape methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import { MovePictureCommand, MoveShapeCommand } from './command';

/** 클릭 좌표에서 그림, 글상자, 수식 개체를 찾는다. */
/** 점과 선분 사이 최소 거리 (px) */
function pointToSegmentDist(px: number, py: number, x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1, dy = y2 - y1;
  const lenSq = dx * dx + dy * dy;
  if (lenSq === 0) return Math.hypot(px - x1, py - y1);
  let t = ((px - x1) * dx + (py - y1) * dy) / lenSq;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
}

export function findPictureAtClick(this: any,
  pageIdx: number, pageX: number, pageY: number,
): { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line'; cellIdx?: number; cellParaIdx?: number; x1?: number; y1?: number; x2?: number; y2?: number } | null {
  try {
    const layout = this.wasm.getPageControlLayout(pageIdx);
    for (const ctrl of layout.controls) {
      if (ctrl.type !== 'image' && ctrl.type !== 'shape' && ctrl.type !== 'equation' && ctrl.type !== 'group' && ctrl.type !== 'line') continue;
      if (ctrl.secIdx === undefined || ctrl.paraIdx === undefined || ctrl.controlIdx === undefined) continue;

      if (ctrl.type === 'line') {
        // 직선: 점-선분 거리, 연결선: 곡선 경로 샘플링으로 히트 판정
        const threshold = 6;
        const dist1 = pointToSegmentDist(pageX, pageY, ctrl.x1, ctrl.y1, ctrl.x2, ctrl.y2);
        let hit = dist1 <= threshold;
        if (!hit && ctrl.w > 2 && ctrl.h > 2) {
          const sx = ctrl.x1, sy = ctrl.y1, ex = ctrl.x2, ey = ctrl.y2;
          const mx = ctrl.x + ctrl.w / 2, my = ctrl.y + ctrl.h / 2;
          // 꺽인 연결선: 가능한 모든 직각 경로 검사
          const segs: [number,number,number,number][] = [
            // 수평→수직→수평 (S자 꺽임)
            [sx,sy, mx,sy], [mx,sy, mx,ey], [mx,ey, ex,ey],
            // 수직→수평→수직 (S자 꺽임)
            [sx,sy, sx,my], [sx,my, ex,my], [ex,my, ex,ey],
            // L자 꺽임
            [sx,sy, ex,sy], [ex,sy, ex,ey],
            [sx,sy, sx,ey], [sx,ey, ex,ey],
          ];
          for (const [ax,ay,bx,by] of segs) {
            if (pointToSegmentDist(pageX, pageY, ax, ay, bx, by) <= threshold) {
              hit = true; break;
            }
          }
          // 곡선 연결선: 베지어 곡선 — 8세그먼트 샘플링
          if (!hit) {
            const c1x = mx, c1y = sy, c2x = mx, c2y = ey;
            const N = 8;
            let prevX = sx, prevY = sy;
            for (let k = 1; k <= N; k++) {
              const t = k / N;
              const u = 1 - t;
              const bx = u*u*u*sx + 3*u*u*t*c1x + 3*u*t*t*c2x + t*t*t*ex;
              const by = u*u*u*sy + 3*u*u*t*c1y + 3*u*t*t*c2y + t*t*t*ey;
              if (pointToSegmentDist(pageX, pageY, prevX, prevY, bx, by) <= threshold) {
                hit = true; break;
              }
              prevX = bx; prevY = by;
            }
          }
        }
        if (hit) {
          return { sec: ctrl.secIdx, ppi: ctrl.paraIdx, ci: ctrl.controlIdx, type: 'line',
            x1: ctrl.x1, y1: ctrl.y1, x2: ctrl.x2, y2: ctrl.y2 };
        }
      } else {
        // bbox 히트 판정
        if (pageX >= ctrl.x && pageX <= ctrl.x + ctrl.w &&
            pageY >= ctrl.y && pageY <= ctrl.y + ctrl.h) {
          return { sec: ctrl.secIdx, ppi: ctrl.paraIdx, ci: ctrl.controlIdx, type: ctrl.type, cellIdx: ctrl.cellIdx, cellParaIdx: ctrl.cellParaIdx };
        }
      }
    }
  } catch { /* ignore */ }
  return null;
}

/** 선택된 개체의 bbox를 페이지 레이아웃에서 찾는다. */
export function findPictureBbox(this: any,
  ref: { sec: number; ppi: number; ci: number; type?: 'image' | 'shape' | 'equation' | 'group' | 'line'; cellIdx?: number; cellParaIdx?: number },
): { pageIndex: number; x: number; y: number; w: number; h: number; x1?: number; y1?: number; x2?: number; y2?: number } | null {
  const matchType = ref.type ?? 'image';
  // line은 shape의 하위 타입 → layout에서 'line'으로 반환됨
  const layoutType = matchType === 'line' ? 'line' : matchType;
  try {
    const pageCount = this.wasm.pageCount;
    for (let p = 0; p < pageCount; p++) {
      const layout = this.wasm.getPageControlLayout(p);
      for (const ctrl of layout.controls) {
        if (ctrl.type === layoutType &&
            ctrl.secIdx === ref.sec && ctrl.paraIdx === ref.ppi && ctrl.controlIdx === ref.ci) {
          // 표 셀 내 수식: cellIdx/cellParaIdx도 매칭
          if (matchType === 'equation' && ref.cellIdx !== undefined) {
            if (ctrl.cellIdx !== ref.cellIdx || ctrl.cellParaIdx !== ref.cellParaIdx) continue;
          }
          return { pageIndex: p, x: ctrl.x, y: ctrl.y, w: ctrl.w, h: ctrl.h,
            x1: ctrl.x1, y1: ctrl.y1, x2: ctrl.x2, y2: ctrl.y2 };
        }
      }
    }
  } catch { /* ignore */ }
  return null;
}

/** 개체 선택 시 외곽선 + 핸들을 렌더링한다. */
export function renderPictureObjectSelection(this: any): void {
  if (!this.pictureObjectRenderer) return;

  // 다중 선택: 합산 bbox로 핸들 표시
  if (this.cursor.isMultiPictureSelection()) {
    const refs = this.cursor.getSelectedPictureRefs();
    try {
      const zoom = this.viewportManager.getZoom();
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      let pageIndex = 0;
      for (const r of refs) {
        const bbox = this.findPictureBbox(r);
        if (bbox) {
          pageIndex = bbox.pageIndex;
          minX = Math.min(minX, bbox.x);
          minY = Math.min(minY, bbox.y);
          maxX = Math.max(maxX, bbox.x + bbox.w);
          maxY = Math.max(maxY, bbox.y + bbox.h);
        }
      }
      if (minX < Infinity) {
        this.pictureObjectRenderer.render(
          { pageIndex, x: minX, y: minY, width: maxX - minX, height: maxY - minY },
          zoom,
        );
      } else {
        this.pictureObjectRenderer.clear();
      }
    } catch {
      this.pictureObjectRenderer.clear();
    }
    return;
  }

  const ref = this.cursor.getSelectedPictureRef();
  if (!ref) {
    this.pictureObjectRenderer.clear();
    return;
  }
  const matchType = ref.type ?? 'image';
  const layoutType = matchType === 'line' ? 'line' : matchType;
  try {
    const zoom = this.viewportManager.getZoom();
    const pageCount = this.wasm.pageCount;
    for (let p = 0; p < pageCount; p++) {
      const layout = this.wasm.getPageControlLayout(p);
      for (const ctrl of layout.controls) {
        if (ctrl.type === layoutType &&
            ctrl.secIdx === ref.sec && ctrl.paraIdx === ref.ppi && ctrl.controlIdx === ref.ci) {
          // 표 셀 내 수식: cellIdx/cellParaIdx도 매칭
          if (matchType === 'equation' && ref.cellIdx !== undefined) {
            if (ctrl.cellIdx !== ref.cellIdx || ctrl.cellParaIdx !== ref.cellParaIdx) continue;
          }

          if (matchType === 'line') {
            // 직선/연결선: 시작점/끝점 핸들 (꺽인/곡선 연결선은 중간점 추가)
            let midPoint: { x: number; y: number } | undefined;
            try {
              const props = this.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci);
              // connectorType >= 3: 꺽인(3~5) 또는 곡선(6~8)
              if (props.connectorType !== undefined && props.connectorType >= 3) {
                if (props.connectorMidX !== undefined && props.connectorMidY !== undefined) {
                  // 실제 꺽임/곡선 제어점 좌표 (HWPUNIT → page px)
                  const PX = 96 / 7200;
                  midPoint = {
                    x: ctrl.x + props.connectorMidX * PX,
                    y: ctrl.y + props.connectorMidY * PX,
                  };
                } else {
                  midPoint = { x: (ctrl.x1 + ctrl.x2) / 2, y: (ctrl.y1 + ctrl.y2) / 2 };
                }
              }
            } catch { /* 일반 선 */ }
            this.pictureObjectRenderer.renderLine(
              { pageIndex: p, x1: ctrl.x1, y1: ctrl.y1, x2: ctrl.x2, y2: ctrl.y2,
                x: ctrl.x, y: ctrl.y, width: ctrl.w, height: ctrl.h },
              zoom,
              midPoint,
            );
            return;
          }

          const bx = ctrl.x, by = ctrl.y, bw = ctrl.w, bh = ctrl.h;

          // 회전각 조회 (shape만)
          let rotAngle = 0;
          if (ref.type === 'shape') {
            try {
              const props = this.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci);
              rotAngle = (props.rotationAngle as number) ?? 0;
            } catch { /* ignore */ }
          }

          this.pictureObjectRenderer.render(
            { pageIndex: p, x: bx, y: by, width: bw, height: bh },
            zoom,
            rotAngle,
          );
          return;
        }
      }
    }
    this.pictureObjectRenderer.clear();
  } catch (e) {
    console.warn('[InputHandler] renderPictureObjectSelection 실패:', e);
    this.pictureObjectRenderer.clear();
  }
}

export function exitPictureObjectSelectionIfNeeded(this: any): void {
  if (this.cursor.isInPictureObjectSelection()) {
    this.cursor.exitPictureObjectSelection();
    this.pictureObjectRenderer?.clear();
    this.eventBus.emit('picture-object-selection-changed', false);
  }
}

/** 클릭 좌표가 글상자의 경계선 위인지 판정한다. */
export function isShapeBorderClick(this: any,
  pageX: number, pageY: number,
  shape: { sec: number; ppi: number; ci: number },
): boolean {
  const THRESHOLD = 3; // px
  const bbox = findPictureBbox.call(this, { ...shape, type: 'shape' as const });
  if (!bbox) return false;
  const dx = Math.min(pageX - bbox.x, bbox.x + bbox.w - pageX);
  const dy = Math.min(pageY - bbox.y, bbox.y + bbox.h - pageY);
  return dx <= THRESHOLD || dy <= THRESHOLD;
}

// ─── 개체 속성 조회 헬퍼 (그림/글상자 분기) ──────────────

/** 개체 속성을 타입에 따라 조회한다. */
export function getObjectProperties(this: any, ref: { sec: number; ppi: number; ci: number; type: string }): any {
  if (ref.type === 'shape' || ref.type === 'line' || ref.type === 'group') {
    return this.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci);
  }
  return this.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci);
}

/** 개체 속성을 타입에 따라 변경한다. */
export function setObjectProperties(this: any, ref: { sec: number; ppi: number; ci: number; type: string }, props: Record<string, unknown>): void {
  if (ref.type === 'shape' || ref.type === 'line' || ref.type === 'group') {
    this.wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, props);
  } else {
    this.wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, props);
  }
}

/** 개체를 타입에 따라 삭제한다. */
export function deleteObjectControl(this: any, ref: { sec: number; ppi: number; ci: number; type: 'image' | 'shape' | 'equation' | 'group' | 'line' }): void {
  if (ref.type === 'shape' || ref.type === 'group' || ref.type === 'line') {
    this.wasm.deleteShapeControl(ref.sec, ref.ppi, ref.ci);
  } else {
    this.wasm.deletePictureControl(ref.sec, ref.ppi, ref.ci);
  }
}

// ─── 핸들 드래그 리사이즈 ─────────────────────────

/** 1 page px = 7200/96 = 75 HWPUNIT */
const PX_TO_HWP = 7200 / 96;
const MIN_SIZE_HWP = 283; // ≈1mm

/**
 * 회전각을 반영하여 리사이즈 후 새 bbox(비회전 기준)를 계산한다.
 * - 마우스 delta를 도형 로컬 좌표계로 역변환한다.
 * - 반대편 꼭짓점(pivot)이 page 좌표에서 고정되도록 중심을 재계산한다.
 */
function calcResizedBboxRotated(
  state: any,
  e: MouseEvent,
  zoom: number,
): { x: number; y: number; width: number; height: number } {
  const angleDeg = (state.rotationAngle ?? 0) as number;
  const rad = angleDeg * Math.PI / 180;
  const cosA = Math.cos(rad);
  const sinA = Math.sin(rad);

  const dx = (e.clientX - state.startClientX) / zoom;
  const dy = (e.clientY - state.startClientY) / zoom;

  // 화면 좌표 delta → 도형 로컬 좌표계로 역변환
  const localDx = dx * cosA + dy * sinA;
  const localDy = -dx * sinA + dy * cosA;

  const w0 = state.bbox.w;
  const h0 = state.bbox.h;
  const cx0 = state.bbox.x + w0 / 2;
  const cy0 = state.bbox.y + h0 / 2;
  const dir: string = state.dir;

  // 크기 제한 없이 마우스 이동 반영 (반대편으로 넘어가면 음수 발생 가능)
  let valW = w0;
  let valH = h0;
  if (dir.includes('e')) valW = w0 + localDx;
  if (dir.includes('w')) valW = w0 - localDx;
  if (dir.includes('s')) valH = h0 + localDy;
  if (dir.includes('n')) valH = h0 - localDy;

  // 최종 출력용 크기는 절대값 사용 (최소 크기는 아주 작게만 제한)
  const MIN = 1; 
  const newW = Math.max(Math.abs(valW), MIN);
  const newH = Math.max(Math.abs(valH), MIN);

  // pivot: 드래그하지 않는 반대쪽 로컬 좌표 (원본 크기 기준)
  const pivotLocalX = dir.includes('e') ? -w0 / 2 : (dir.includes('w') ? w0 / 2 : 0);
  const pivotLocalY = dir.includes('s') ? -h0 / 2 : (dir.includes('n') ? h0 / 2 : 0);

  // pivot의 page 좌표 (고정)
  const pivotPageX = cx0 + pivotLocalX * cosA - pivotLocalY * sinA;
  const pivotPageY = cy0 + pivotLocalX * sinA + pivotLocalY * cosA;

  // 새 크기에서 pivot의 로컬 좌표 (valW/valH가 음수면 pivot 방향이 반전됨)
  const newPivotLocalX = dir.includes('e') ? -valW / 2 : (dir.includes('w') ? valW / 2 : 0);
  const newPivotLocalY = dir.includes('s') ? -valH / 2 : (dir.includes('n') ? valH / 2 : 0);

  // pivot 고정 조건으로 새 중심 계산
  const newCx = pivotPageX - (newPivotLocalX * cosA - newPivotLocalY * sinA);
  const newCy = pivotPageY - (newPivotLocalX * sinA + newPivotLocalY * cosA);

  return { x: newCx - newW / 2, y: newCy - newH / 2, width: newW, height: newH };
}

export function updatePictureResizeDrag(this: any, e: MouseEvent): void {
  if (!this.pictureResizeState || !this.pictureObjectRenderer) return;
  const zoom = this.viewportManager.getZoom();
  const state = this.pictureResizeState;

  // 핸들은 고정, 예비 테두리만 갱신
  const rotAngle = (state.rotationAngle ?? 0) as number;
  const newBbox = state.multiRefs
    ? this.calcResizedBbox(e, zoom)
    : calcResizedBboxRotated(state, e, zoom);

  // 모든 경우에 점선 프리뷰만 갱신하여 앵커는 제자리에 머물게 함
  this.pictureObjectRenderer.renderDragPreview(
    { pageIndex: state.pageIndex, ...newBbox },
    zoom,
    rotAngle,
  );

  // 다중 선택: 드래그 중 실시간으로 개체 크기/위치 반영
  if (state.multiRefs && state.multiRefs.length > 0) {
    const scaleX = newBbox.width / state.bbox.w;
    const scaleY = newBbox.height / state.bbox.h;
    const origX = state.bbox.x;
    const origY = state.bbox.y;
    const newOrigX = newBbox.x;
    const newOrigY = newBbox.y;
    const PX2HWP = PX_TO_HWP;
    const isCorner = ['nw', 'ne', 'sw', 'se'].includes(state.dir);
    try {
      for (const r of state.multiRefs) {
        const relX = r.bboxX - origX;
        const relY = r.bboxY - origY;
        // 코너: 자유 리사이즈 (scaleX, scaleY 독립 반영), 측면: 해당 축만
        const sx = (isCorner || state.dir === 'e' || state.dir === 'w') ? scaleX : 1;
        const sy = (isCorner || state.dir === 'n' || state.dir === 's') ? scaleY : 1;
        const newPx = newOrigX + relX * scaleX;
        const newPy = newOrigY + relY * scaleY;
        const deltaH = Math.round((newPx - r.bboxX) * PX2HWP);
        const deltaV = Math.round((newPy - r.bboxY) * PX2HWP);
        const newW = Math.max(Math.round(r.origWidth * sx), MIN_SIZE_HWP);
        const newH = Math.max(Math.round(r.origHeight * sy), MIN_SIZE_HWP);
        const updated: Record<string, unknown> = { width: newW, height: newH };
        if (deltaH !== 0) updated['horzOffset'] = ((r.origHorzOffset + deltaH) >>> 0);
        if (deltaV !== 0) updated['vertOffset'] = ((r.origVertOffset + deltaV) >>> 0);
        setObjectProperties.call(this, r, updated);
      }
      this.eventBus.emit('document-changed');
    } catch { /* ignore */ }
  }

  // 그룹 단일 선택: 드래그 중 실시간 크기 반영
  if (!state.multiRefs && state.ref.type === 'group') {
    const newW = Math.max(Math.round(newBbox.width * PX_TO_HWP), MIN_SIZE_HWP);
    const newH = Math.max(Math.round(newBbox.height * PX_TO_HWP), MIN_SIZE_HWP);
    const newHorzOffset = Math.round(newBbox.x * PX_TO_HWP);
    const newVertOffset = Math.round(newBbox.y * PX_TO_HWP);
    try {
      setObjectProperties.call(this, state.ref, {
        width: newW,
        height: newH,
        horzOffset: (newHorzOffset >>> 0),
        vertOffset: (newVertOffset >>> 0),
      });
      this.eventBus.emit('document-changed');
    } catch { /* ignore */ }
  }
}

export function finishPictureResizeDrag(this: any, e: MouseEvent): void {
  const state = this.pictureResizeState;
  if (!state) { this.cleanupPictureResizeDrag(); return; }

  const zoom = this.viewportManager.getZoom();
  const PX2HWP = PX_TO_HWP;

  // 다중 선택 리사이즈: 드래그 중 실시간 반영 완료 → 최종 확정만
  if (state.multiRefs && state.multiRefs.length > 0) {
    const newBbox = this.calcResizedBbox(e, zoom);
    const scaleX = newBbox.width / state.bbox.w;
    const scaleY = newBbox.height / state.bbox.h;
    const origX = state.bbox.x;
    const origY = state.bbox.y;
    const newOrigX = newBbox.x;
    const newOrigY = newBbox.y;
    const isCorner = ['nw', 'ne', 'sw', 'se'].includes(state.dir);

    try {
      for (const r of state.multiRefs) {
        const relX = r.bboxX - origX;
        const relY = r.bboxY - origY;
        const sx = isCorner ? scaleX : (state.dir === 'n' || state.dir === 's' ? 1 : scaleX);
        const sy = isCorner ? scaleX : (state.dir === 'e' || state.dir === 'w' ? 1 : scaleY);
        const newPx = newOrigX + relX * sx;
        const newPy = newOrigY + relY * sy;
        const deltaH = Math.round((newPx - r.bboxX) * PX2HWP);
        const deltaV = Math.round((newPy - r.bboxY) * PX2HWP);
        const newW = Math.max(Math.round(r.origWidth * sx), MIN_SIZE_HWP);
        const newH = Math.max(Math.round(r.origHeight * sy), MIN_SIZE_HWP);
        const updated: Record<string, unknown> = { width: newW, height: newH };
        if (deltaH !== 0) updated['horzOffset'] = ((r.origHorzOffset + deltaH) >>> 0);
        if (deltaV !== 0) updated['vertOffset'] = ((r.origVertOffset + deltaV) >>> 0);
        setObjectProperties.call(this, r, updated);
      }
      this.eventBus.emit('document-changed');
    } catch (err) {
      console.warn('[InputHandler] 다중 개체 리사이즈 실패:', err);
    }
    this.cleanupPictureResizeDrag();
    this.renderPictureObjectSelection();
    return;
  }

  // 단일 선택 리사이즈 (회전 반영: pivot 고정, 위치도 갱신)
  const newBbox = calcResizedBboxRotated(state, e, zoom);
  const newW = Math.max(Math.round(newBbox.width * PX2HWP), MIN_SIZE_HWP);
  const newH = Math.max(Math.round(newBbox.height * PX2HWP), MIN_SIZE_HWP);
  const newHorzOffset = Math.round(newBbox.x * PX2HWP);
  const newVertOffset = Math.round(newBbox.y * PX2HWP);
  const origHorzOffset = Math.round(state.bbox.x * PX2HWP);
  const origVertOffset = Math.round(state.bbox.y * PX2HWP);

  try {
    const updated: Record<string, unknown> = {};
    if (newW !== state.origWidth) updated['width'] = newW;
    if (newH !== state.origHeight) updated['height'] = newH;
    if (newHorzOffset !== origHorzOffset) updated['horzOffset'] = (newHorzOffset >>> 0);
    if (newVertOffset !== origVertOffset) updated['vertOffset'] = (newVertOffset >>> 0);
    if (Object.keys(updated).length > 0) {
      setObjectProperties.call(this, state.ref, updated);
      this.eventBus.emit('document-changed');
    }
  } catch (err) {
    console.warn('[InputHandler] 개체 리사이즈 실패:', err);
  }
  this.cleanupPictureResizeDrag();
  this.renderPictureObjectSelection();
}

export function calcResizedBbox(this: any, e: MouseEvent, zoom: number): { x: number; y: number; width: number; height: number } {
  const s = this.pictureResizeState!;
  const dx = (e.clientX - s.startClientX) / zoom; // page px
  const dy = (e.clientY - s.startClientY) / zoom;
  const MIN = 1;

  let { x, y, w, h } = s.bbox;
  const dir = s.dir;

  // 가로 크기 및 위치 계산 (Flip 허용)
  if (dir.includes('e')) {
    const valW = s.bbox.w + dx;
    w = Math.max(Math.abs(valW), MIN);
    if (valW < 0) x = s.bbox.x + valW; // 반대편으로 넘어가면 시작점 이동
  } else if (dir.includes('w')) {
    const valW = s.bbox.w - dx;
    w = Math.max(Math.abs(valW), MIN);
    if (valW >= 0) x = s.bbox.x + dx;
    else x = s.bbox.x + s.bbox.w; // 반대편으로 넘어가면 오른쪽 끝이 시작점
  }

  // 세로 크기 및 위치 계산 (Flip 허용)
  if (dir.includes('s')) {
    const valH = s.bbox.h + dy;
    h = Math.max(Math.abs(valH), MIN);
    if (valH < 0) y = s.bbox.y + valH;
  } else if (dir.includes('n')) {
    const valH = s.bbox.h - dy;
    h = Math.max(Math.abs(valH), MIN);
    if (valH >= 0) y = s.bbox.y + dy;
    else y = s.bbox.y + s.bbox.h;
  }

  return { x, y, width: w, height: h };
}

export function cleanupPictureResizeDrag(this: any): void {
  this.isPictureResizeDragging = false;
  this.pictureResizeState = null;
  this.container.style.cursor = '';
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }
  this.pictureObjectRenderer?.clearDragPreview();
}

export function updatePictureMoveDrag(this: any, e: MouseEvent): void {
  if (!this.pictureMoveState) return;
  const zoom = this.viewportManager.getZoom();
  const sc = this.container.querySelector('#scroll-content');
  if (!sc) return;
  const cr = sc.getBoundingClientRect();
  const cx = e.clientX - cr.left;
  const cy = e.clientY - cr.top;
  const pi = this.virtualScroll.getPageAtY(cy);
  const po = this.virtualScroll.getPageOffset(pi);
  const pw = this.virtualScroll.getPageWidth(pi);
  const pl = (sc.clientWidth - pw) / 2;
  const px = (cx - pl) / zoom;
  const py = (cy - po) / zoom;

  const deltaXpx = px - this.pictureMoveState.lastPageX;
  const deltaYpx = py - this.pictureMoveState.lastPageY;
  const deltaH = Math.round(deltaXpx * 75); // 1 page px = 75 HWPUNIT
  const deltaV = Math.round(deltaYpx * 75);

  if (deltaH === 0 && deltaV === 0) return;

  try {
    // 다중 선택: 모든 개체를 동일 delta로 이동
    const targets = this.pictureMoveState.multiRefs || [this.pictureMoveState.ref];
    for (const ref of targets) {
      const props = getObjectProperties.call(this, ref);
      setObjectProperties.call(this, ref, {
        horzOffset: ((props.horzOffset + deltaH) >>> 0),
        vertOffset: ((props.vertOffset + deltaV) >>> 0),
      });
    }
    this.pictureMoveState.lastPageX = px;
    this.pictureMoveState.lastPageY = py;
    this.pictureMoveState.totalDeltaH += deltaH;
    this.pictureMoveState.totalDeltaV += deltaV;
    // 연결선 자동 추적
    try { this.wasm.updateConnectorsInSection(targets[0].sec); } catch { /* ignore */ }
    this.eventBus.emit('document-changed');
    this.renderPictureObjectSelection();
  } catch (err) {
    console.warn('[InputHandler] 개체 이동 드래그 실패:', err);
  }
}

export function finishPictureMoveDrag(this: any): void {
  if (this.pictureMoveState) {
    const { totalDeltaH, totalDeltaV, multiRefs } = this.pictureMoveState;
    if (totalDeltaH !== 0 || totalDeltaV !== 0) {
      const targets = multiRefs || [{ ...this.pictureMoveState.ref, origHorzOffset: this.pictureMoveState.origHorzOffset, origVertOffset: this.pictureMoveState.origVertOffset }];
      for (const r of targets) {
        const CmdClass = (r.type === 'shape' || r.type === 'line' || r.type === 'group') ? MoveShapeCommand : MovePictureCommand;
        this.history.recordWithoutExecute(
          new CmdClass(
            r.sec, r.ppi, r.ci,
            totalDeltaH, totalDeltaV,
            r.origHorzOffset, r.origVertOffset,
          ),
        );
      }
    }
  }
  this.isPictureMoveDragging = false;
  this.pictureMoveState = null;
  this.container.style.cursor = '';
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }
}

// ─── 회전 드래그 ─────────────────────────────────

/** 회전 드래그 중: 마우스 각도에 따라 실시간 회전 적용 */
export function updatePictureRotateDrag(this: any, e: MouseEvent): void {
  if (!this.pictureRotateState) return;
  const sc = this.container.querySelector('#scroll-content');
  if (!sc) return;
  const cr = sc.getBoundingClientRect();
  const mx = e.clientX - cr.left;
  const my = e.clientY - cr.top;

  const s = this.pictureRotateState;
  const currentAngle = Math.atan2(my - s.centerY, mx - s.centerX);
  let deltaDeg = (currentAngle - s.startAngle) * (180 / Math.PI);

  // Ctrl 키: 15° 단위 스냅
  let newAngle = s.origAngle + deltaDeg;
  if (e.ctrlKey) {
    newAngle = Math.round(newAngle / 15) * 15;
  }
  // -360 ~ 360 범위로 정규화
  newAngle = ((newAngle % 360) + 360) % 360;
  if (newAngle > 180) newAngle -= 360;

  try {
    setObjectProperties.call(this, s.ref, { rotationAngle: Math.round(newAngle) });
    this.eventBus.emit('document-changed');
    // 드래그 중에는 핸들 고정 — renderPictureObjectSelection 호출 안 함
  } catch (err) {
    console.warn('[InputHandler] 개체 회전 드래그 실패:', err);
  }
}

/** 회전 드래그 종료: 핸들을 최종 회전 위치로 스냅 */
export function finishPictureRotateDrag(this: any, _e: MouseEvent): void {
  this.isPictureRotateDragging = false;
  this.pictureRotateState = null;
  this.container.style.cursor = '';
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }
  this.renderPictureObjectSelection();
}
