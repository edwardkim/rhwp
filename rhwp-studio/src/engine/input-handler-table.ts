/** input-handler table methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import { MoveTableCommand, MovePictureCommand, MoveShapeCommand } from './command';
import { getObjectProperties, setObjectProperties } from './input-handler-picture';
import type { CellBbox } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { BorderEdge } from './table-resize-renderer';
import {
  buildColumnResizeUpdates,
  buildCellSelectionColumnDragUpdates,
  cellOverlapsSelectionRange,
  findResizeCompensationNeighbors,
  type CellSelectionRange,
  type LocalResizeUpdate,
  type ResizeArrowKey,
} from './table-resize-updates';
import { showToast } from '@/ui/toast';

const MIN_TABLE_CELL_SIZE_HWP = 200;

function isOuterResizeEdge(self: any, edge: BorderEdge, pageBboxes: CellBbox[]): boolean {
  try {
    const { rowLines, colLines } = self.tableResizeRenderer.computeBorderLines(pageBboxes);
    if (edge.type === 'row') {
      return edge.index === 0 || edge.index === rowLines.length - 1;
    }
    return edge.index === 0 || edge.index === colLines.length - 1;
  } catch {
    return false;
  }
}

function computeResizePositionBounds(
  self: any,
  edge: BorderEdge,
  pageBboxes: CellBbox[],
): { min: number; max: number } {
  const minSizePx = MIN_TABLE_CELL_SIZE_HWP / 75;
  const { rowLines, colLines } = self.tableResizeRenderer.computeBorderLines(pageBboxes);
  const lines = edge.type === 'row'
    ? rowLines.map((line: any) => ({ pos: line.y, index: line.index }))
    : colLines.map((line: any) => ({ pos: line.x, index: line.index }));
  const lineIdx = lines.findIndex((line: any) => line.index === edge.index);
  if (lineIdx < 0) return { min: -Infinity, max: Infinity };

  const prev = lines[lineIdx - 1]?.pos;
  const next = lines[lineIdx + 1]?.pos;
  return {
    min: prev === undefined ? -Infinity : prev + minSizePx,
    max: next === undefined ? Infinity : next - minSizePx,
  };
}

function clampResizePosition(pos: number, bounds: { min: number; max: number }): number {
  return Math.min(Math.max(pos, bounds.min), bounds.max);
}

function selectTableObjectFromResize(this: any, tableRef: { sec: number; ppi: number; ci: number }): void {
  this.cursor.clearSelection();
  this.cursor.exitCellSelectionMode();
  this.cellSelectionRenderer?.clear();
  this.exitPictureObjectSelectionIfNeeded();
  this.cursor.enterTableObjectSelectionDirect(tableRef.sec, tableRef.ppi, tableRef.ci);
  this.active = true;
  this.caret.hide();
  this.fieldMarker.hide();
  this.selectionRenderer.clear();
  this.renderTableObjectSelection();
  this.eventBus.emit('table-object-selection-changed', true);
  this.eventBus.emit('command-state-changed');
  this.textarea.focus();
}

function findSingleCellResizeTarget(
  edge: BorderEdge,
  pageX: number,
  pageY: number,
  bboxes: CellBbox[],
  borderOriginalPos: number,
): { cellIdx: number; side: 'start' | 'end' } | null {
  const tolerance = 4.0;
  const rounded = (v: number) => Math.round(v * 10) / 10;
  const border = rounded(borderOriginalPos);
  const candidates: Array<{ cellIdx: number; side: 'start' | 'end'; score: number }> = [];

  for (const b of bboxes) {
    if (edge.type === 'col') {
      if (pageY < b.y - tolerance || pageY > b.y + b.h + tolerance) continue;
      const startDistance = Math.abs(rounded(b.x) - border);
      const endDistance = Math.abs(rounded(b.x + b.w) - border);
      if (startDistance <= tolerance) {
        candidates.push({ cellIdx: b.cellIdx, side: 'start', score: Math.abs(pageY - (b.y + b.h / 2)) });
      }
      if (endDistance <= tolerance) {
        candidates.push({ cellIdx: b.cellIdx, side: 'end', score: Math.abs(pageY - (b.y + b.h / 2)) });
      }
    } else {
      if (pageX < b.x - tolerance || pageX > b.x + b.w + tolerance) continue;
      const startDistance = Math.abs(rounded(b.y) - border);
      const endDistance = Math.abs(rounded(b.y + b.h) - border);
      if (startDistance <= tolerance) {
        candidates.push({ cellIdx: b.cellIdx, side: 'start', score: Math.abs(pageX - (b.x + b.w / 2)) });
      }
      if (endDistance <= tolerance) {
        candidates.push({ cellIdx: b.cellIdx, side: 'end', score: Math.abs(pageX - (b.x + b.w / 2)) });
      }
    }
  }

  if (candidates.length === 0) return null;

  const preferredSide: 'start' | 'end' =
    (edge.type === 'col' ? pageX : pageY) <= borderOriginalPos ? 'end' : 'start';
  const preferred = candidates
    .filter(c => c.side === preferredSide)
    .sort((a, b) => a.score - b.score)[0];
  if (preferred) return { cellIdx: preferred.cellIdx, side: preferred.side };

  const fallback = candidates.sort((a, b) => a.score - b.score)[0];
  return { cellIdx: fallback.cellIdx, side: fallback.side };
}

function findSingleCellResizeNeighbor(
  edge: BorderEdge,
  target: { cellIdx: number; side: 'start' | 'end' },
  bboxes: CellBbox[],
): number | null {
  const targetBox = bboxes.find(b => b.cellIdx === target.cellIdx);
  if (!targetBox) return null;

  if (edge.type === 'col') {
    const neighbor = target.side === 'end'
      ? bboxes.find(b => b.row === targetBox.row && b.col === targetBox.col + targetBox.colSpan)
      : bboxes.find(b => b.row === targetBox.row && b.col + b.colSpan === targetBox.col);
    return neighbor?.cellIdx ?? null;
  }

  const neighbor = target.side === 'end'
    ? bboxes.find(b => b.col === targetBox.col && b.row === targetBox.row + targetBox.rowSpan)
    : bboxes.find(b => b.col === targetBox.col && b.row + b.rowSpan === targetBox.row);
  return neighbor?.cellIdx ?? null;
}

function findAlignedLogicalResizeAffectedCells(
  edge: BorderEdge,
  target: { cellIdx: number; side: 'start' | 'end' },
  bboxes: CellBbox[],
): number[] {
  const targetBox = bboxes.find(b => b.cellIdx === target.cellIdx);
  if (!targetBox) return [];
  const tolerance = 1.0;
  const rounded = (v: number) => Math.round(v / tolerance) * tolerance;

  if (edge.type === 'col') {
    const boundaryCol = target.side === 'end'
      ? targetBox.col + targetBox.colSpan
      : targetBox.col;
    const targetCoord = rounded(target.side === 'end' ? targetBox.x + targetBox.w : targetBox.x);
    return [...new Set(
      bboxes
        .filter(b =>
          b.col + b.colSpan === boundaryCol &&
          Math.abs(rounded(b.x + b.w) - targetCoord) <= tolerance)
        .map(b => b.cellIdx),
    )];
  }

  const boundaryRow = target.side === 'end'
    ? targetBox.row + targetBox.rowSpan
    : targetBox.row;
  const targetCoord = rounded(target.side === 'end' ? targetBox.y + targetBox.h : targetBox.y);
  return [...new Set(
    bboxes
      .filter(b =>
        b.row + b.rowSpan === boundaryRow &&
        Math.abs(rounded(b.y + b.h) - targetCoord) <= tolerance)
      .map(b => b.cellIdx),
  )];
}

function clampCompensatedResizeDelta(
  wasm: any,
  tableRef: { sec: number; ppi: number; ci: number },
  edge: BorderEdge,
  pairs: Array<{ targetCellIdx: number; neighborCellIdxs: number[] }>,
  requestedDelta: number,
): number {
  if (requestedDelta === 0) return 0;
  const finiteLimits: number[] = [];

  for (const pair of pairs) {
    try {
      const targetProps = wasm.getCellProperties(tableRef.sec, tableRef.ppi, tableRef.ci, pair.targetCellIdx);
      const targetSize = edge.type === 'col' ? targetProps.width : targetProps.height;
      if (requestedDelta < 0 && Number.isFinite(targetSize)) {
        finiteLimits.push(Math.max(0, Math.round(targetSize - MIN_TABLE_CELL_SIZE_HWP)));
      }

      for (const neighborCellIdx of pair.neighborCellIdxs) {
        const neighborProps = wasm.getCellProperties(tableRef.sec, tableRef.ppi, tableRef.ci, neighborCellIdx);
        const neighborSize = edge.type === 'col' ? neighborProps.width : neighborProps.height;
        if (requestedDelta > 0 && Number.isFinite(neighborSize)) {
          finiteLimits.push(Math.max(0, Math.round(neighborSize - MIN_TABLE_CELL_SIZE_HWP)));
        }
      }
    } catch {
      // 조회 실패 셀은 기존 동작처럼 clamp 대상에서 제외한다.
    }
  }

  if (finiteLimits.length === 0) return requestedDelta;
  const limit = Math.min(...finiteLimits);
  if (requestedDelta > 0) return Math.min(requestedDelta, limit);
  return -Math.min(Math.abs(requestedDelta), limit);
}

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
  const coordinateAffectedCellIndices: number[] = [];

  for (const b of this.cachedCellBboxes) {
    if (edge.type === 'col') {
      if (Math.abs(ry(b.x + b.w) - ry(borderOriginalPos)) <= tolerance) {
        coordinateAffectedCellIndices.push(b.cellIdx);
      }
    } else {
      if (Math.abs(ry(b.y + b.h) - ry(borderOriginalPos)) <= tolerance) {
        coordinateAffectedCellIndices.push(b.cellIdx);
      }
    }
  }

  const resizeTarget = findSingleCellResizeTarget(
    edge,
    pageX,
    pageY,
    this.cachedCellBboxes,
    borderOriginalPos,
  );
  if (!resizeTarget) return;
  const logicalAffectedCellIndices = findAlignedLogicalResizeAffectedCells(
    edge,
    resizeTarget,
    this.cachedCellBboxes,
  );
  const affectedCellIndices = logicalAffectedCellIndices.length > 0
    ? logicalAffectedCellIndices
    : coordinateAffectedCellIndices;
  if (affectedCellIndices.length === 0) return;
  const resizeBounds = computeResizePositionBounds(this, edge, pageBboxes);

  this.isResizeDragging = true;
  this.resizeDragState = {
    edge,
    tableRef: { ...this.cachedTableRef },
    bboxes: this.cachedCellBboxes,
    pageBboxes,
    affectedCellIndices,
    borderOriginalPos,
    minResizePos: resizeBounds.min,
    maxResizePos: resizeBounds.max,
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

  const rawNewPos = this.resizeDragState.edge.type === 'row' ? pageY : pageX;
  const newPos = clampResizePosition(rawNewPos, {
    min: this.resizeDragState.minResizePos,
    max: this.resizeDragState.maxResizePos,
  });
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

  const rawNewPos = state.edge.type === 'row' ? pageY : pageX;
  const newPos = clampResizePosition(rawNewPos, {
    min: state.minResizePos,
    max: state.maxResizePos,
  });
  const deltaPagePx = newPos - state.borderOriginalPos;
  // 1 page px (96 DPI) = 75 HWPUNIT (7200/96)
  const deltaHwpUnit = Math.round(deltaPagePx * 75);

  // 너무 작은 드래그는 무시 (1px 미만)
  if (Math.abs(deltaHwpUnit) < 75) {
    const shouldSelectTable = isOuterResizeEdge(this, state.edge, state.pageBboxes);
    const tableRef = { ...state.tableRef };
    this.cleanupResizeDrag();
    if (shouldSelectTable) {
      selectTableObjectFromResize.call(this, tableRef);
    }
    return;
  }

  // 지속 가능한 geometry만 구성한다. 일반 세로 경계는 셀 선택 상태와
  // 무관하게 행 전체 높이 조절로 처리한다.
  let updates: Array<{
    cellIdx: number;
    widthDelta?: number;
    heightDelta?: number;
    localResize?: boolean;
    renderWidth?: number;
    renderHeight?: number;
  }>;
  const inCellSel = this.cursor.isInCellSelectionMode();
  const range = inCellSel ? this.cursor.getSelectedCellRange() : null;

  if (state.edge.type === 'col' && inCellSel && range) {
    // 선택 셀만 추출 — 병합 셀은 시작 좌표가 아니라 겹침으로 판정한다
    const selectedBboxes = state.affectedCellIndices
      .map((cellIdx: any) => state.bboxes.find((b: any) => b.cellIdx === cellIdx))
      .filter((b: any): b is CellBbox => b !== undefined && cellOverlapsSelectionRange(b, range));
    if (selectedBboxes.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
    // 오른쪽 이웃 보상은 병합 셀이 걸친 모든 행을 쓸어야 한다 — 시작 행만 보상하면
    // 행별 열 폭 합이 어긋나 표가 깨진다. (이 분기는 edge.type === 'col' 전용이다)
    updates = buildCellSelectionColumnDragUpdates(selectedBboxes, state.bboxes, deltaHwpUnit);
    if (updates.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
  } else {
    // 일반 모드: 균일한 내부 경계 전체를 움직이되, 반대편 이웃 셀을 보상해 표 외곽을 유지
    if (state.affectedCellIndices.length === 0) {
      this.cleanupResizeDrag();
      return;
    }
    const targetBboxes = state.affectedCellIndices
      .map((cellIdx: any) => state.bboxes.find((b: any) => b.cellIdx === cellIdx))
      .filter((b: any): b is CellBbox => b !== undefined);
    const pairs: Array<{ targetCellIdx: number; neighborCellIdxs: number[] }> =
      targetBboxes.map((bbox: CellBbox) => ({
      targetCellIdx: bbox.cellIdx,
      neighborCellIdxs: findResizeCompensationNeighbors(state.edge, bbox, state.bboxes)
        .map(neighbor => neighbor.cellIdx),
    }));
    const delta = clampCompensatedResizeDelta(
      this.wasm,
      state.tableRef,
      state.edge,
      pairs,
      deltaHwpUnit,
    );
    if (delta === 0) {
      this.cleanupResizeDrag();
      return;
    }
    updates = [];
    const addedNeighbors = new Set<number>();
    for (const pair of pairs) {
      if (state.edge.type === 'col') {
        updates.push({ cellIdx: pair.targetCellIdx, widthDelta: delta });
        for (const neighborCellIdx of pair.neighborCellIdxs) {
          if (addedNeighbors.has(neighborCellIdx)) continue;
          updates.push({ cellIdx: neighborCellIdx, widthDelta: -delta });
          addedNeighbors.add(neighborCellIdx);
        }
      } else {
        updates.push({ cellIdx: pair.targetCellIdx, heightDelta: delta });
        for (const neighborCellIdx of pair.neighborCellIdxs) {
          if (addedNeighbors.has(neighborCellIdx)) continue;
          updates.push({ cellIdx: neighborCellIdx, heightDelta: -delta });
          addedNeighbors.add(neighborCellIdx);
        }
      }
    }
  }

  if (updates.some(update => update.localResize === true)) {
    // Legacy local geometry stays outside the source model. This guard is a
    // non-blocking safety net for any stale drag state.
    this.cleanupResizeDrag();
    return;
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
  if (!hit) {
    this.imagePlacementDrag = null;
    this.container.style.cursor = 'crosshair';
    showToast({
      message: '그림을 넣을 문단을 찾지 못했습니다.\n문서 본문이나 표 셀 안쪽을 다시 클릭하세요.',
      durationMs: 5000,
    });
    return;
  }

  const sec = hit.sectionIndex;
  // 표 셀/글상자 안 클릭: cellPath 와 parentParaIndex (= 소유 본문 paragraph) 를 사용한다.
  // 표 셀은 기존 #1151 경로처럼 parent paragraph sibling floating 으로 삽입되고,
  // 글상자는 #1322 보강 경로에서 text_box 내부 paragraph control 로 삽입된다.
  const isTextBoxHit = hit.isTextBox === true;
  const inCell = (hit.cellPath?.length ?? 0) > 0 && hit.parentParaIndex !== undefined && !isTextBoxHit;
  const inTextBox = isTextBoxHit && (hit.cellPath?.length ?? 0) > 0 && hit.parentParaIndex !== undefined;
  const textBoxControlIdx = hit.controlIndex ?? hit.cellPath?.[0]?.controlIdx ?? hit.cellPath?.[0]?.controlIndex;
  // 표 셀: 외곽 표 소유 본문 para, 글상자: 글상자 소유 본문 para, 본문: 클릭 문단.
  const useParentPara = (inCell || inTextBox) && hit.parentParaIndex !== undefined;
  const paraIdx = useParentPara ? hit.parentParaIndex! : hit.paragraphIndex;
  const charOffset = hit.charOffset;
  const cellPathJson = (inCell || inTextBox) ? JSON.stringify(hit.cellPath) : '';

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

  // [Task #1151 v8 결함 C / v9 결함 E] 셀 안 + 본문 floating picture 의 paper-relative
  // offset 계산. 사용자가 드래그/클릭한 위치 (drag.startClientX/Y) 를 page (= paper)
  // 좌표로 변환. v9 결함 E 후 본문 path 도 floating sibling 으로 통합되었으므로
  // inCell 제한 제거 — 본문에서도 사용자 클릭 위치 전달 필요.
  let paperOffsetXHu: number | undefined;
  let paperOffsetYHu: number | undefined;
  {
    const scrollContent = this.container.querySelector('#scroll-content');
    if (scrollContent) {
      const contentRect = scrollContent.getBoundingClientRect();
      const dragContentX = drag.startClientX - contentRect.left;
      const dragContentY = drag.startClientY - contentRect.top;
      const pageIdx = this.virtualScroll.getPageAtPoint(dragContentX, dragContentY);
      const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
      const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
      const dragPageX = (dragContentX - pageLeft) / zoom;
      const dragPageY = (dragContentY - pageOffset) / zoom;
      if (inTextBox) {
        paperOffsetXHu = 0;
        paperOffsetYHu = 0;
        try {
          const layout = this.wasm.getPageControlLayout(pageIdx);
          const shape = layout.controls.find((ctrl: any) =>
            ctrl.type === 'shape' &&
            ctrl.secIdx === sec &&
            ctrl.paraIdx === paraIdx &&
            ctrl.controlIdx === textBoxControlIdx
          );
          if (shape) {
            const props = this.wasm.getShapeProperties(sec, paraIdx, textBoxControlIdx);
            const marginLeftPx = ((props as any).tbMarginLeft ?? 0) / 75;
            const marginTopPx = ((props as any).tbMarginTop ?? 0) / 75;
            paperOffsetXHu = Math.max(0, Math.round((dragPageX - shape.x - marginLeftPx) * 75));
            paperOffsetYHu = Math.max(0, Math.round((dragPageY - shape.y - marginTopPx) * 75));
          }
        } catch {
          // 글상자 bbox 조회 실패 시 글상자 내부 좌상단 삽입으로 fallback.
          paperOffsetXHu = 0;
          paperOffsetYHu = 0;
        }
      } else {
        paperOffsetXHu = Math.round(dragPageX * 75);
        paperOffsetYHu = Math.round(dragPageY * 75);
      }
    }
  }

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

  // WASM 호출 — 스냅샷으로 기록 (Undo 지원, pasteImage 경로와 동일 패턴)
  try {
    let insertFailedMsg: string | null = null;
    this.executeOperation({ kind: 'snapshot', operationType: 'insertPicture', operation: (wasm: WasmBridge) => {
      const result = wasm.insertPicture(
        sec, paraIdx, charOffset, cellPathJson, imgData.data,
        wHwp, hHwp, imgData.naturalWidth, imgData.naturalHeight,
        imgData.ext, desc,
        paperOffsetXHu, paperOffsetYHu,
      );
      if (!result.ok) {
        insertFailedMsg = (result as any).error || '삽입 위치 또는 이미지 정보를 확인할 수 없습니다.';
        console.warn('[InputHandler] 그림 삽입 실패:', result);
      }
      return this.cursor.getPosition();
    }});
    if (insertFailedMsg) {
      showToast({
        message: `그림 삽입에 실패했습니다.\n${insertFailedMsg}`,
        durationMs: 6000,
      });
    }
  } catch (err) {
    console.warn('[InputHandler] 그림 삽입 실패:', err);
    const msg = err instanceof Error ? err.message : String(err);
    showToast({
      message: `그림 삽입에 실패했습니다.\n${msg}`,
      durationMs: 6000,
    });
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
      const newHorzOffset = props.horzOffset + deltaH;
      const newVertOffset = props.vertOffset + deltaV;
      setObjectProperties.call(this, r, {
        horzOffset: newHorzOffset,
        vertOffset: newVertOffset,
      });
      const CmdClass = r.type === 'shape' || r.type === 'line' || r.type === 'group' ? MoveShapeCommand : MovePictureCommand;
      this.executeOperation({ kind: 'record', command:
        new CmdClass(r.sec, r.ppi, r.ci, deltaH, deltaV, props.horzOffset, props.vertOffset, r.cellPath),
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

  if (!this.moveDragState.hasMoved) {
    const threshold = 3 / Math.max(zoom, 0.1);
    const dxFromStart = px - this.moveDragState.startPageX;
    const dyFromStart = py - this.moveDragState.startPageY;
    if (Math.hypot(dxFromStart, dyFromStart) < threshold) return;
    this.moveDragState.hasMoved = true;
  }

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
  const state = this.moveDragState;

  // Undo 기록: 드래그 전체를 하나의 명령으로 기록
  if (state) {
    const { totalDeltaH, totalDeltaV, startPpi, tableRef } = state;
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

  if (state?.pendingEnterCellHit && !state.hasMoved && state.totalDeltaH === 0 && state.totalDeltaV === 0) {
    this.cursor.exitTableObjectSelection();
    this.tableObjectRenderer?.clear();
    this.eventBus.emit('table-object-selection-changed', false);
    this.cursor.clearSelection();
    this.cursor.moveTo(state.pendingEnterCellHit);
    this.cursor.resetPreferredX();
    this.cursor.setAnchor();
    this.active = true;
    this.updateCaret();
    this.textarea.focus();
  }
}

/** 세 모드 공통: 셀 bbox 조회 → 빌더 → snapshot 라우팅. */
function applyKeyboardResize(
  this: any,
  key: ResizeArrowKey,
  operationType: string,
  build: (bboxes: CellBbox[], range: CellSelectionRange, key: ResizeArrowKey) => LocalResizeUpdate[],
): void {
  const ctx = this.cursor.getCellTableContext();
  const range = this.cursor.getSelectedCellRange();
  if (!ctx || !range) return;

  let bboxes: CellBbox[];
  try {
    bboxes = this.wasm.getTableCellBboxes(ctx.sec, ctx.ppi, ctx.ci);
  } catch { return; }

  const updates = build(bboxes, range, key);
  if (updates.length === 0) return;

  try {
    this.executeOperation({
      kind: 'snapshot',
      operationType,
      operation: (wasm: any) => {
        wasm.resizeTableCells(ctx.sec, ctx.ppi, ctx.ci, updates);
        return this.cursor.getPosition();
      },
    });
    this.updateCellSelection();
  } catch (err) {
    console.warn(`[InputHandler] ${operationType} 실패:`, err);
  }
}

/** Ctrl/Cmd+방향키 — 칸/줄 전체 크기 조절 (한컴 table(size).htm). */
export function resizeCellByKeyboard(this: any, key: ResizeArrowKey): void {
  applyKeyboardResize.call(this, key, 'resizeCellByKeyboard', buildColumnResizeUpdates);
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
