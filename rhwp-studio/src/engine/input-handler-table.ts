/** input-handler table methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import { MoveTableCommand, MovePictureCommand, MoveShapeCommand } from './command';
import { getObjectProperties, setObjectProperties } from './input-handler-picture';
import type { CellBbox } from '@/core/types';
import type { BorderEdge } from './table-resize-renderer';

export function startResizeDrag(this: any, 
  edge: BorderEdge,
  pageX: number, pageY: number,
  pageBboxes: CellBbox[],
): void {
  if (!this.cachedTableRef || !this.cachedCellBboxes || !this.tableResizeRenderer) return;

  // 경계선 원래 위치 계산
  const { rowLines, colLines } = this.tableResizeRenderer.computeBorderLines(pageBboxes);
  let borderOriginalPos: number;
  if (edge.type === 'row') {
    const line = rowLines.find((l: any) => l.index === edge.index);
    if (!line) return;
    borderOriginalPos = line.y;
  } else {
    const line = colLines.find((l: any) => l.index === edge.index);
    if (!line) return;
    borderOriginalPos = line.x;
  }

  // 영향받는 셀: 경계선에 해당하는 edge에 맞닿은 셀
  const tolerance = 1.0;
  const ry = (v: number) => Math.round(v * 10) / 10;
  const affectedCellIndices: number[] = [];

  for (const b of this.cachedCellBboxes) {
    if (edge.type === 'col') {
      if (Math.abs(ry(b.x + b.w) - ry(borderOriginalPos)) <= tolerance) {
        affectedCellIndices.push(b.cellIdx);
      }
    } else {
      if (Math.abs(ry(b.y + b.h) - ry(borderOriginalPos)) <= tolerance) {
        affectedCellIndices.push(b.cellIdx);
      }
    }
  }

  if (affectedCellIndices.length === 0) return;

  this.isResizeDragging = true;
  this.resizeDragState = {
    edge,
    tableRef: { ...this.cachedTableRef },
    bboxes: this.cachedCellBboxes,
    pageBboxes,
    affectedCellIndices,
    borderOriginalPos,
  };

  // mouseup 리스너 등록 (document 레벨)
  document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
}

export function updateResizeDrag(this: any, e: MouseEvent): void {
  if (!this.resizeDragState || !this.tableResizeRenderer) return;

  const zoom = this.viewportManager.getZoom();
  const scrollContent = this.container.querySelector('#scroll-content');
  if (!scrollContent) return;
  const contentRect = scrollContent.getBoundingClientRect();
  const contentX = e.clientX - contentRect.left;
  const contentY = e.clientY - contentRect.top;
  const pageIdx = this.resizeDragState.edge.pageIndex;
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
  const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
  const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
  const pageX = (contentX - pageLeft) / zoom;
  const pageY = (contentY - pageOffset) / zoom;

  const newPos = this.resizeDragState.edge.type === 'row' ? pageY : pageX;

  // 드래그 마커 표시
  this.tableResizeRenderer.showDragMarker(
    this.resizeDragState.edge.type,
    newPos,
    pageIdx,
    this.resizeDragState.pageBboxes,
    zoom,
  );
}

export function finishResizeDrag(this: any, e: MouseEvent): void {
  if (!this.resizeDragState || !this.tableResizeRenderer) {
    this.cleanupResizeDrag();
    return;
  }

  const state = this.resizeDragState;

  // mouseup 이벤트 좌표에서 page 좌표 계산
  const zoom = this.viewportManager.getZoom();
  const scrollContent = this.container.querySelector('#scroll-content');
  if (!scrollContent) {
    this.cleanupResizeDrag();
    return;
  }
  const contentRect = scrollContent.getBoundingClientRect();
  const contentX = e.clientX - contentRect.left;
  const contentY = e.clientY - contentRect.top;
  const pageIdx = state.edge.pageIndex;
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
  const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
  const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
  const pageX = (contentX - pageLeft) / zoom;
  const pageY = (contentY - pageOffset) / zoom;

  const newPos = state.edge.type === 'row' ? pageY : pageX;
  const deltaPagePx = newPos - state.borderOriginalPos;
  // 1 page px (96 DPI) = 75 HWPUNIT (7200/96)
  const deltaHwpUnit = Math.round(deltaPagePx * 75);

  // 너무 작은 드래그는 무시 (1px 미만)
  if (Math.abs(deltaHwpUnit) < 75) {
    this.cleanupResizeDrag();
    return;
  }

  // 셀 선택 모드: 선택 셀 + 이웃 셀 보상 (행/열 전체 너비 유지)
  // 일반 모드: 경계선에 맞닿은 모든 셀 (열/행 전체)
  let updates: Array<{ cellIdx: number; widthDelta?: number; heightDelta?: number }>;
  const inCellSel = this.cursor.isInCellSelectionMode();
  const range = inCellSel ? this.cursor.getSelectedCellRange() : null;

  if (inCellSel && range) {
    // 선택 셀만 추출
    const selectedBboxes = state.affectedCellIndices
      .map((cellIdx: any) => state.bboxes.find((b: any) => b.cellIdx === cellIdx))
      .filter((b: any): b is CellBbox =>
        b !== undefined &&
        b.row >= range.startRow && b.row <= range.endRow &&
        b.col >= range.startCol && b.col <= range.endCol);
    if (selectedBboxes.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
    updates = [];
    const addedNeighbors = new Set<number>();
    for (const bbox of selectedBboxes) {
      if (state.edge.type === 'col') {
        updates.push({ cellIdx: bbox.cellIdx, widthDelta: deltaHwpUnit });
        // 같은 행의 오른쪽 이웃 셀에 반대 delta
        const neighbor = state.bboxes.find((b: any) =>
          b.row === bbox.row && b.col === bbox.col + bbox.colSpan);
        if (neighbor && !addedNeighbors.has(neighbor.cellIdx)) {
          updates.push({ cellIdx: neighbor.cellIdx, widthDelta: -deltaHwpUnit });
          addedNeighbors.add(neighbor.cellIdx);
        }
      } else {
        updates.push({ cellIdx: bbox.cellIdx, heightDelta: deltaHwpUnit });
        // 같은 열의 아래쪽 이웃 셀에 반대 delta
        const neighbor = state.bboxes.find((b: any) =>
          b.col === bbox.col && b.row === bbox.row + bbox.rowSpan);
        if (neighbor && !addedNeighbors.has(neighbor.cellIdx)) {
          updates.push({ cellIdx: neighbor.cellIdx, heightDelta: -deltaHwpUnit });
          addedNeighbors.add(neighbor.cellIdx);
        }
      }
    }
    if (updates.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
  } else {
    // 일반 모드: 경계선에 맞닿은 모든 셀 (열/행 전체 크기 변경)
    if (state.affectedCellIndices.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
    updates = state.affectedCellIndices.map((cellIdx: any) => {
      if (state.edge.type === 'col') {
        return { cellIdx, widthDelta: deltaHwpUnit };
      } else {
        return { cellIdx, heightDelta: deltaHwpUnit };
      }
    });
  }

  // WASM 배치 API 호출 (복합 셀 보상 변경은 스냅샷으로 Undo 기록)
  try {
    this.executeOperation({
      kind: 'snapshot',
      operationType: 'resizeTableCells',
      operation: (wasm: any) => {
        wasm.resizeTableCells(
          state.tableRef.sec,
          state.tableRef.ppi,
          state.tableRef.ci,
          updates,
        );
        return this.cursor.getPosition();
      },
    });
    if (inCellSel) this.updateCellSelection();
  } catch (err) {
    console.warn('[InputHandler] resizeTableCells 실패:', err);
  }

  this.cleanupResizeDrag();
}

export function cleanupResizeDrag(this: any): void {
  this.isResizeDragging = false;
  this.resizeDragState = null;
  this.tableResizeRenderer?.clear();
  this.container.style.cursor = '';
  // 캐시 무효화 (크기 변경 후 bbox가 stale)
  this.cachedTableRef = null;
  this.cachedCellBboxes = null;
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }
}

export function cancelImagePlacement(this: any): void {
  this.imagePlacementMode = false;
  this.imagePlacementData = null;
  this.imagePlacementDrag = null;
  this.hideImagePlacementOverlay();
  this.container.style.cursor = '';
}

export function showImagePlacementOverlay(this: any, x1: number, y1: number, x2: number, y2: number): void {
  if (!this.imagePlacementOverlay) {
    this.imagePlacementOverlay = document.createElement('div');
    this.imagePlacementOverlay.style.cssText =
      'position:fixed;border:2px dashed #0078d7;background:rgba(0,120,215,0.08);pointer-events:none;z-index:9999;';
    document.body.appendChild(this.imagePlacementOverlay);
  }
  const left = Math.min(x1, x2);
  const top = Math.min(y1, y2);
  const w = Math.abs(x2 - x1);
  const h = Math.abs(y2 - y1);
  this.imagePlacementOverlay.style.left = `${left}px`;
  this.imagePlacementOverlay.style.top = `${top}px`;
  this.imagePlacementOverlay.style.width = `${w}px`;
  this.imagePlacementOverlay.style.height = `${h}px`;
}

export function hideImagePlacementOverlay(this: any): void {
  if (this.imagePlacementOverlay) {
    this.imagePlacementOverlay.remove();
    this.imagePlacementOverlay = null;
  }
}

export function finishImagePlacement(this: any, e: MouseEvent): void {
  const drag = this.imagePlacementDrag;
  const imgData = this.imagePlacementData;
  if (!drag || !imgData) { this.cancelImagePlacement(); return; }

  this.hideImagePlacementOverlay();

  // 클릭 위치에서 hitTest → 삽입할 문단 결정
  const hit = this.hitTestFromEvent(e);
  if (!hit) { this.cancelImagePlacement(); return; }

  const sec = hit.sectionIndex;
  const paraIdx = hit.paragraphIndex;
  const charOffset = hit.charOffset;

  // 크기 결정
  const zoom = this.viewportManager.getZoom();
  let wPx: number, hPx: number;
  if (drag.isDragging) {
    // 드래그 영역 크기 (화면 px → 페이지 px)
    wPx = Math.abs(drag.currentClientX - drag.startClientX) / zoom;
    hPx = Math.abs(drag.currentClientY - drag.startClientY) / zoom;
    if (wPx < 10) wPx = 10;
    if (hPx < 10) hPx = 10;
  } else {
    // 클릭만 한 경우: 원본 크기 100%
    wPx = imgData.naturalWidth;
    hPx = imgData.naturalHeight;
  }

  // px → HWPUNIT (1px = 75 HWPUNIT at 96 DPI)
  let wHwp = Math.round(wPx * 75);
  let hHwp = Math.round(hPx * 75);

  // 열 폭 초과 시 비례 축소
  try {
    const pageDef = this.wasm.getPageDef(sec);
    const colWidth = pageDef.width - pageDef.marginLeft - pageDef.marginRight;
    if (wHwp > colWidth) {
      const ratio = colWidth / wHwp;
      wHwp = Math.round(colWidth);
      hHwp = Math.round(hHwp * ratio);
    }
  } catch { /* 페이지 정보 없으면 그대로 */ }

  // 개체 설명문 생성 (한컴 기본 패턴)
  const desc = `그림입니다.\r\n원본 그림의 이름: ${imgData.fileName}\r\n원본 그림의 크기: 가로 ${imgData.naturalWidth}pixel, 세로 ${imgData.naturalHeight}pixel`;

  // WASM 호출
  try {
    const result = this.wasm.insertPicture(sec, paraIdx, charOffset, imgData.data, wHwp, hHwp, imgData.naturalWidth, imgData.naturalHeight, imgData.ext, desc);
    if (result.ok) {
      this.eventBus.emit('document-changed');
    }
  } catch (err) {
    console.warn('[InputHandler] 그림 삽입 실패:', err);
  }

  // 모드 종료
  this.imagePlacementMode = false;
  this.imagePlacementData = null;
  this.imagePlacementDrag = null;
  this.container.style.cursor = '';
}

export function moveSelectedTable(this: any, key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
  const ref = this.cursor.getSelectedTableRef();
  if (!ref) return;

  const step = Math.round(this.gridStepMm * 7200 / 25.4); // mm → HWPUNIT
  let deltaH = 0;
  let deltaV = 0;
  switch (key) {
    case 'ArrowLeft':  deltaH = -step; break;
    case 'ArrowRight': deltaH = step;  break;
    case 'ArrowUp':    deltaV = -step; break;
    case 'ArrowDown':  deltaV = step;  break;
  }

  try {
    const result = this.wasm.moveTableOffset(ref.sec, ref.ppi, ref.ci, deltaH, deltaV);
    // Undo 기록
    this.executeOperation({ kind: 'record', command:
      new MoveTableCommand(ref.sec, ref.ppi, ref.ci, deltaH, deltaV, result.ppi, result.ci),
    });
    // 문단 경계를 넘어 이동한 경우 selectedTableRef 갱신
    if (result.ppi !== ref.ppi || result.ci !== ref.ci) {
      this.cursor.updateSelectedTableRef(ref.sec, result.ppi, result.ci);
    }
    this.eventBus.emit('document-changed');
    this.renderTableObjectSelection();
  } catch (err) {
    console.warn('[InputHandler] 표 이동 실패:', err);
  }
}

export function moveSelectedPicture(this: any, key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
  const refs = this.cursor.getSelectedPictureRefs();
  const ref = this.cursor.getSelectedPictureRef();
  if (!ref) return;

  const step = Math.round(this.gridStepMm * 7200 / 25.4); // mm → HWPUNIT
  let deltaH = 0;
  let deltaV = 0;
  switch (key) {
    case 'ArrowLeft':  deltaH = -step; break;
    case 'ArrowRight': deltaH = step;  break;
    case 'ArrowUp':    deltaV = -step; break;
    case 'ArrowDown':  deltaV = step;  break;
  }

  // 다중 선택: 모든 선택된 개체를 동일 delta만큼 이동
  const targets = refs.length > 1 ? refs : [ref];
  try {
    for (const r of targets) {
      const props = getObjectProperties.call(this, r);
      if (props.treatAsChar) continue; // treat_as_char 개체는 이동 불가
      const newHorzOffset = ((props.horzOffset + deltaH) >>> 0);
      const newVertOffset = ((props.vertOffset + deltaV) >>> 0);
      setObjectProperties.call(this, r, {
        horzOffset: newHorzOffset,
        vertOffset: newVertOffset,
      });
      const CmdClass = r.type === 'shape' || r.type === 'line' || r.type === 'group' ? MoveShapeCommand : MovePictureCommand;
      this.executeOperation({ kind: 'record', command:
        new CmdClass(r.sec, r.ppi, r.ci, deltaH, deltaV, props.horzOffset, props.vertOffset),
      });
    }
    // 연결선 자동 추적
    try { this.wasm.updateConnectorsInSection(targets[0].sec); } catch { /* ignore */ }
    this.eventBus.emit('document-changed');
    this.renderPictureObjectSelection();
  } catch (err) {
    console.warn('[InputHandler] 개체 이동 실패:', err);
  }
}

export function updateMoveDrag(this: any, e: MouseEvent): void {
  if (!this.moveDragState) return;
  const zoom = this.viewportManager.getZoom();
  const sc = this.container.querySelector('#scroll-content');
  if (!sc) return;
  const cr = sc.getBoundingClientRect();
  const cx = e.clientX - cr.left;
  const cy = e.clientY - cr.top;
  const pi = this.virtualScroll.getPageAtPoint(cx, cy);
  const po = this.virtualScroll.getPageOffset(pi);
  const pw = this.virtualScroll.getPageWidth(pi);
  const pl = this.virtualScroll.getPageLeftResolved(pi, sc.clientWidth);
  const px = (cx - pl) / zoom;
  const py = (cy - po) / zoom;

  // 이전 위치와의 차이를 HWPUNIT으로 변환 (1px = 7200/96 = 75 HWPUNIT)
  const deltaXpx = px - this.moveDragState.lastPageX;
  const deltaYpx = py - this.moveDragState.lastPageY;
  const deltaH = Math.round(deltaXpx * 75);
  const deltaV = Math.round(deltaYpx * 75);

  if (deltaH === 0 && deltaV === 0) return;

  try {
    const ref = this.moveDragState.tableRef;
    const result = this.wasm.moveTableOffset(ref.sec, ref.ppi, ref.ci, deltaH, deltaV);
    if (result.ppi !== ref.ppi || result.ci !== ref.ci) {
      this.moveDragState.tableRef = { sec: ref.sec, ppi: result.ppi, ci: result.ci };
      this.cursor.updateSelectedTableRef(ref.sec, result.ppi, result.ci);
    }
    this.moveDragState.lastPageX = px;
    this.moveDragState.lastPageY = py;
    this.moveDragState.totalDeltaH += deltaH;
    this.moveDragState.totalDeltaV += deltaV;
    this.eventBus.emit('document-changed');
    this.renderTableObjectSelection();
  } catch (err) {
    console.warn('[InputHandler] 표 이동 드래그 실패:', err);
  }
}

export function finishMoveDrag(this: any): void {
  // Undo 기록: 드래그 전체를 하나의 명령으로 기록
  if (this.moveDragState) {
    const { totalDeltaH, totalDeltaV, startPpi, tableRef } = this.moveDragState;
    if (totalDeltaH !== 0 || totalDeltaV !== 0) {
      this.executeOperation({ kind: 'record', command:
        new MoveTableCommand(
          tableRef.sec, startPpi, tableRef.ci,
          totalDeltaH, totalDeltaV,
          tableRef.ppi, tableRef.ci,
        ),
      });
    }
  }
  this.isMoveDragging = false;
  this.moveDragState = null;
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }
  this.container.style.cursor = '';
}

export function resizeCellByKeyboard(this: any, key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
  const ctx = this.cursor.getCellTableContext();
  const range = this.cursor.getSelectedCellRange();
  if (!ctx || !range) return;

  const DELTA = 300; // 1 키스트로크 당 300 HWPUNIT (~1mm)
  let bboxes: CellBbox[];
  try {
    bboxes = this.wasm.getTableCellBboxes(ctx.sec, ctx.ppi, ctx.ci);
  } catch { return; }

  // 선택 범위 내 셀 bbox 추출
  const selectedBboxes = bboxes
    .filter(b => b.row >= range.startRow && b.row <= range.endRow
              && b.col >= range.startCol && b.col <= range.endCol);
  if (selectedBboxes.length === 0) return;

  const updates: Array<{ cellIdx: number; widthDelta?: number; heightDelta?: number }> = [];
  const addedNeighbors = new Set<number>(); // 이웃 셀 중복 방지

  for (const bbox of selectedBboxes) {
    const isHoriz = (key === 'ArrowLeft' || key === 'ArrowRight');
    const delta = (key === 'ArrowRight' || key === 'ArrowDown') ? DELTA : -DELTA;

    if (isHoriz) {
      updates.push({ cellIdx: bbox.cellIdx, widthDelta: delta });
      // 같은 행의 오른쪽 이웃 셀에 반대 delta (행 전체 너비 유지)
      const neighbor = bboxes.find(b =>
        b.row === bbox.row && b.col === bbox.col + bbox.colSpan);
      if (neighbor && !addedNeighbors.has(neighbor.cellIdx)) {
        updates.push({ cellIdx: neighbor.cellIdx, widthDelta: -delta });
        addedNeighbors.add(neighbor.cellIdx);
      }
    } else {
      updates.push({ cellIdx: bbox.cellIdx, heightDelta: delta });
      // 같은 열의 아래쪽 이웃 셀에 반대 delta (열 전체 높이 유지)
      const neighbor = bboxes.find(b =>
        b.col === bbox.col && b.row === bbox.row + bbox.rowSpan);
      if (neighbor && !addedNeighbors.has(neighbor.cellIdx)) {
        updates.push({ cellIdx: neighbor.cellIdx, heightDelta: -delta });
        addedNeighbors.add(neighbor.cellIdx);
      }
    }
  }

  try {
    this.executeOperation({
      kind: 'snapshot',
      operationType: 'resizeCellByKeyboard',
      operation: (wasm: any) => {
        wasm.resizeTableCells(ctx.sec, ctx.ppi, ctx.ci, updates);
        return this.cursor.getPosition();
      },
    });
    this.updateCellSelection();
  } catch (err) {
    console.warn('[InputHandler] resizeCellByKeyboard 실패:', err);
  }
}

/** 전체 표 비율 리사이즈 (phase 3, Ctrl+방향키) */
export function resizeTableProportional(this: any, key: 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): void {
  const ctx = this.cursor.getCellTableContext();
  if (!ctx) return;

  const DELTA = 200; // 1 키스트로크 당 200 HWPUNIT
  const isHoriz = (key === 'ArrowLeft' || key === 'ArrowRight');
  const delta = (key === 'ArrowRight' || key === 'ArrowDown') ? DELTA : -DELTA;

  try {
    const bboxes = this.wasm.getTableCellBboxes(ctx.sec, ctx.ppi, ctx.ci);
    const updates: Array<{ cellIdx: number; widthDelta?: number; heightDelta?: number }> = [];
    const processed = new Set<number>();

    for (const bbox of bboxes) {
      if (processed.has(bbox.cellIdx)) continue;
      processed.add(bbox.cellIdx);
      if (isHoriz) {
        updates.push({ cellIdx: bbox.cellIdx, widthDelta: delta });
      } else {
        updates.push({ cellIdx: bbox.cellIdx, heightDelta: delta });
      }
    }

    this.executeOperation({
      kind: 'snapshot',
      operationType: 'resizeTableProportional',
      operation: (wasm: any) => {
        wasm.resizeTableCells(ctx.sec, ctx.ppi, ctx.ci, updates);
        return this.cursor.getPosition();
      },
    });
    this.updateCellSelection();
  } catch (err) {
    console.warn('[InputHandler] resizeTableProportional 실패:', err);
  }
}

