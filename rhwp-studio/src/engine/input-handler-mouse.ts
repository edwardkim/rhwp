/** input-handler mouse methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import type { ContextMenuItem } from '@/ui/context-menu';
import * as _connector from './input-handler-connector';

export function onClick(this: any, e: MouseEvent): void {
  // 연결선 드로잉 모드: 연결점 클릭으로 시작/끝
  if (this.connectorDrawingMode && e.button === 0) {
    const target = e.target as HTMLElement;
    if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;
    e.preventDefault();

    const sc = this.container.querySelector('#scroll-content');
    if (!sc) return;
    const zoom = this.viewportManager.getZoom();
    const cr = sc.getBoundingClientRect();
    const cx = e.clientX - cr.left;
    const cy = e.clientY - cr.top;
    const pi = this.virtualScroll.getPageAtY(cy);
    const po = this.virtualScroll.getPageOffset(pi);
    const pw = this.virtualScroll.getPageWidth(pi);
    const pl = (sc.clientWidth - pw) / 2;
    const pageX = (cx - pl) / zoom;
    const pageY = (cy - po) / zoom;

    const cp = _connector.findNearestConnectionPoint.call(this, pi, pageX, pageY, 15,
      this.connectorStartRef ? { sec: this.connectorStartRef.sec, ppi: this.connectorStartRef.ppi, ci: this.connectorStartRef.ci } : undefined);

    this.textarea.focus();
    if (!this.connectorStartRef) {
      // 시작점 클릭
      if (cp) {
        this.connectorStartRef = { sec: cp.sec, ppi: cp.ppi, ci: cp.ci, index: cp.index, x: cp.x, y: cp.y, pageIdx: pi };
      }
    } else {
      // 끝점 클릭 → 연결선 생성
      if (cp) {
        _connector.finishConnectorDrawing.call(this,
          { ...this.connectorStartRef, instanceId: 0 } as any,
          { ...cp, instanceId: 0 } as any,
          this.connectorType);
      }
      _connector.exitConnectorDrawingMode.call(this);
    }
    return;
  }

  // 다각형 그리기 모드: 클릭으로 꼭짓점 추가
  if (this.polygonDrawingMode && e.button === 0) {
    const target = e.target as HTMLElement;
    if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;
    e.preventDefault();
    // 시작점 근접 체크 (2mm ≈ 7.6px at 96dpi)
    if (this.polygonPoints.length >= 3) {
      const first = this.polygonPoints[0];
      const dist = Math.hypot(e.clientX - first.x, e.clientY - first.y);
      if (dist < 8) {
        // 닫힌 다각형
        this.polygonPoints.push({ x: first.x, y: first.y });
        this.finishPolygonDrawing();
        return;
      }
    }
    this.polygonAddPoint(e.clientX, e.clientY);
    this.textarea?.focus(); // Backspace/Esc 키 이벤트 수신 유지
    return;
  }

  // 그림 배치 모드: 마우스다운 시 드래그 시작
  if (this.imagePlacementMode && this.imagePlacementData && e.button === 0) {
    const target = e.target as HTMLElement;
    if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;
    e.preventDefault();
    this.imagePlacementDrag = {
      startClientX: e.clientX, startClientY: e.clientY,
      currentClientX: e.clientX, currentClientY: e.clientY,
      isDragging: false,
    };
    this.showImagePlacementOverlay(e.clientX, e.clientY, e.clientX, e.clientY);
    document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
    return;
  }

  // 글상자 배치 모드: 마우스다운 시 드래그 시작
  if (this.textboxPlacementMode && e.button === 0) {
    const target = e.target as HTMLElement;
    if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;
    e.preventDefault();
    this.textboxPlacementDrag = {
      startClientX: e.clientX, startClientY: e.clientY,
      currentClientX: e.clientX, currentClientY: e.clientY,
      isDragging: false,
    };
    this.showTextboxPlacementOverlay(e.clientX, e.clientY, e.clientX, e.clientY);
    document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
    return;
  }

  // 메뉴바/툴바/스타일바 클릭은 무시
  const target = e.target as HTMLElement;
  if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;

  // 스크롤바 영역 클릭은 무시 (네이티브 스크롤 동작을 방해하지 않음)
  const containerRect = this.container.getBoundingClientRect();
  const localX = e.clientX - containerRect.left;
  const localY = e.clientY - containerRect.top;
  if (localX >= this.container.clientWidth || localY >= this.container.clientHeight) {
    return;
  }

  // 표 객체 선택 중 클릭 처리
  if (this.cursor.isInTableObjectSelection()) {
    // 우클릭 → 표 객체 선택 유지 (컨텍스트 메뉴에서 처리)
    if (e.button === 2) return;

    // 좌클릭이 표 내부이면 → 이동 드래그 시작
    const ref = this.cursor.getSelectedTableRef();
    if (ref && e.button === 0) {
      const zoom = this.viewportManager.getZoom();
      const sc = this.container.querySelector('#scroll-content');
      if (sc) {
        const cr = sc.getBoundingClientRect();
        const cx = e.clientX - cr.left;
        const cy = e.clientY - cr.top;
        const pi = this.virtualScroll.getPageAtY(cy);
        const po = this.virtualScroll.getPageOffset(pi);
        const pw = this.virtualScroll.getPageWidth(pi);
        const pl = (sc.clientWidth - pw) / 2;
        const px = (cx - pl) / zoom;
        const py = (cy - po) / zoom;
        try {
          const bbox = this.wasm.getTableBBox(ref.sec, ref.ppi, ref.ci);
          if (px >= bbox.x && px <= bbox.x + bbox.width &&
              py >= bbox.y && py <= bbox.y + bbox.height) {
            e.preventDefault();
            this.isMoveDragging = true;
            this.moveDragState = {
              tableRef: { sec: ref.sec, ppi: ref.ppi, ci: ref.ci },
              startPpi: ref.ppi,
              startPageX: px, startPageY: py,
              lastPageX: px, lastPageY: py,
              totalDeltaH: 0, totalDeltaV: 0,
            };
            this.container.style.cursor = 'move';
            document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
            this.textarea.focus();
            return;
          }
        } catch { /* bbox 조회 실패 시 무시 */ }
      }
    }

    // 표 밖 좌클릭 → 표 객체 선택 해제
    this.cursor.exitTableObjectSelection();
    this.eventBus.emit('table-object-selection-changed', false);
    this.container.style.cursor = '';
  }

  // 그림/글상자 객체 선택 중 클릭 처리
  if (this.cursor.isInPictureObjectSelection()) {
    if (e.button === 2) return; // 우클릭 → 컨텍스트 메뉴에서 처리

    // 다중 선택 상태: 핸들 리사이즈 + BBOX 내부 이동 드래그
    if (this.cursor.isMultiPictureSelection()) {
      if (e.button === 0 && !e.shiftKey) {
        const sc = this.container.querySelector('#scroll-content');
        if (sc) {
          const zoom = this.viewportManager.getZoom();
          const cr = sc.getBoundingClientRect();
          const cx = e.clientX - cr.left;
          const cy = e.clientY - cr.top;
          const pi = this.virtualScroll.getPageAtY(cy);
          const po = this.virtualScroll.getPageOffset(pi);
          const pw = this.virtualScroll.getPageWidth(pi);
          const pl = (sc.clientWidth - pw) / 2;
          const px = (cx - pl) / zoom;
          const py = (cy - po) / zoom;
          // 합산 BBOX 계산
          const refs = this.cursor.getSelectedPictureRefs();
          let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
          let bboxPage = 0;
          for (const r of refs) {
            const bbox = this.findPictureBbox(r);
            if (bbox) { bboxPage = bbox.pageIndex; minX = Math.min(minX, bbox.x); minY = Math.min(minY, bbox.y); maxX = Math.max(maxX, bbox.x + bbox.w); maxY = Math.max(maxY, bbox.y + bbox.h); }
          }

          // (1) 핸들 감지 → 리사이즈 드래그 시작
          if (minX < Infinity && this.pictureObjectRenderer) {
            const dir = this.pictureObjectRenderer.getHandleAtPoint(cx, cy);
            if (dir && dir !== 'rotate') {
              e.preventDefault();
              const combinedW = maxX - minX;
              const combinedH = maxY - minY;
              // 각 개체의 원래 크기/위치/bbox 저장
              const multiResizeRefs: { sec: number; ppi: number; ci: number; type: string; origWidth: number; origHeight: number; origHorzOffset: number; origVertOffset: number; bboxX: number; bboxY: number }[] = [];
              for (const r of refs) {
                try {
                  const p = this.getObjectProperties(r);
                  const bb = this.findPictureBbox(r);
                  if (!p.treatAsChar && bb) multiResizeRefs.push({ ...r, origWidth: p.width, origHeight: p.height, origHorzOffset: p.horzOffset, origVertOffset: p.vertOffset, bboxX: bb.x, bboxY: bb.y });
                } catch { /* skip */ }
              }
              if (multiResizeRefs.length > 0) {
                this.isPictureResizeDragging = true;
                this.pictureResizeState = {
                  dir,
                  ref: multiResizeRefs[0] as any,
                  origWidth: Math.round(combinedW * 75), // page px → HWPUNIT
                  origHeight: Math.round(combinedH * 75),
                  startClientX: e.clientX,
                  startClientY: e.clientY,
                  pageIndex: bboxPage,
                  bbox: { x: minX, y: minY, w: combinedW, h: combinedH },
                  multiRefs: multiResizeRefs,
                };
                document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
                return;
              }
            }
          }

          // (2) BBOX 내부 클릭 → 이동 드래그
          if (minX < Infinity && pi === bboxPage &&
              px >= minX && px <= maxX && py >= minY && py <= maxY) {
            const multiMoveRefs: { sec: number; ppi: number; ci: number; type: string; origHorzOffset: number; origVertOffset: number }[] = [];
            for (const r of refs) {
              try {
                const p = this.getObjectProperties(r);
                if (!p.treatAsChar) multiMoveRefs.push({ ...r, origHorzOffset: p.horzOffset, origVertOffset: p.vertOffset });
              } catch { /* skip */ }
            }
            if (multiMoveRefs.length > 0) {
              e.preventDefault();
              this.isPictureMoveDragging = true;
              this.pictureMoveState = {
                ref: multiMoveRefs[0] as any,
                origHorzOffset: multiMoveRefs[0].origHorzOffset,
                origVertOffset: multiMoveRefs[0].origVertOffset,
                startPageX: px, startPageY: py,
                lastPageX: px, lastPageY: py,
                totalDeltaH: 0, totalDeltaV: 0,
                pageIndex: pi,
                multiRefs: multiMoveRefs,
              };
              this.container.style.cursor = 'move';
              document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
              this.textarea.focus();
              return;
            }
          }
        }
        // BBOX 밖 클릭 → 선택 해제
        this.exitPictureObjectSelectionIfNeeded();
      }
      // Shift+클릭은 아래 findPictureAtClick에서 토글 처리
    } else {

    // 핸들 드래그 리사이즈 / 회전 시작 감지 (수식은 이동/리사이즈/회전 미지원)
    const selRef = this.cursor.getSelectedPictureRef();
    if (e.button === 0 && this.pictureObjectRenderer && selRef?.type !== 'equation') {
      const sc = this.container.querySelector('#scroll-content');
      if (sc) {
        const cr = sc.getBoundingClientRect();
        const cx = e.clientX - cr.left;
        const cy = e.clientY - cr.top;
        const dir = this.pictureObjectRenderer.getHandleAtPoint(cx, cy);
        if (dir) {
          e.preventDefault();
          const ref = this.cursor.getSelectedPictureRef();
          if (ref) {
            const picBbox = this.findPictureBbox(ref);
            if (picBbox) {
              // 직선/연결선: 끝점 핸들 드래그 (sw=시작, ne=끝)
              if (ref.type === 'line' && (dir === 'sw' || dir === 'ne')) {
                const zoom = this.viewportManager.getZoom();
                const po = this.virtualScroll.getPageOffset(picBbox.pageIndex);
                const pw = this.virtualScroll.getPageWidth(picBbox.pageIndex);
                const pl = (sc.clientWidth - pw) / 2;
                this.isLineEndpointDragging = true;
                this.lineEndpointState = {
                  ref: { sec: ref.sec, ppi: ref.ppi, ci: ref.ci, type: ref.type },
                  endpoint: dir === 'sw' ? 'start' : 'end',
                  pageIndex: picBbox.pageIndex,
                  pageLeft: pl, pageOffset: po, zoom,
                };
                this.container.style.cursor = 'crosshair';
                document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
                return;
              }
              if (dir === 'rotate') {
                // 회전 드래그 시작
                const zoom = this.viewportManager.getZoom();
                const po = this.virtualScroll.getPageOffset(picBbox.pageIndex);
                const pw = this.virtualScroll.getPageWidth(picBbox.pageIndex);
                const pl = (sc.clientWidth - pw) / 2;
                // 도형 중심 (scroll-content 좌표)
                const objCx = pl + (picBbox.x + picBbox.w / 2) * zoom;
                const objCy = po + (picBbox.y + picBbox.h / 2) * zoom;
                // 현재 회전각
                const props = this.getObjectProperties(ref);
                const origAngle = props.rotationAngle ?? 0;
                // 마우스→중심 각도
                const startAngle = Math.atan2(cy - objCy, cx - objCx);
                this.isPictureRotateDragging = true;
                this.pictureRotateState = {
                  ref: { sec: ref.sec, ppi: ref.ppi, ci: ref.ci, type: ref.type },
                  origAngle,
                  centerX: objCx,
                  centerY: objCy,
                  startAngle,
                  pageIndex: picBbox.pageIndex,
                };
                this.container.style.cursor = 'grabbing';
                document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
                return;
              }
              // 리사이즈 드래그 시작
              const props = this.getObjectProperties(ref);
              this.isPictureResizeDragging = true;
              this.pictureResizeState = {
                dir,
                ref: { sec: ref.sec, ppi: ref.ppi, ci: ref.ci, type: ref.type },
                origWidth: props.width,
                origHeight: props.height,
                rotationAngle: (props.rotationAngle ?? 0) as number,
                startClientX: e.clientX,
                startClientY: e.clientY,
                pageIndex: picBbox.pageIndex,
                bbox: { x: picBbox.x, y: picBbox.y, w: picBbox.w, h: picBbox.h },
              };
              document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
              return;
            }
          }
        }
      }
    }

    // 핸들 밖 클릭 → 본체 클릭이면 이동 드래그 시작, 아니면 선택 해제 (수식은 이동 미지원)
    if (e.button === 0) {
      const ref = this.cursor.getSelectedPictureRef();
      if (ref && ref.type !== 'equation') {
        const picBbox = this.findPictureBbox(ref);
        if (picBbox) {
          const sc = this.container.querySelector('#scroll-content');
          if (sc) {
            const zoom = this.viewportManager.getZoom();
            const cr = sc.getBoundingClientRect();
            const cx = e.clientX - cr.left;
            const cy = e.clientY - cr.top;
            const pi = this.virtualScroll.getPageAtY(cy);
            const po = this.virtualScroll.getPageOffset(pi);
            const pw = this.virtualScroll.getPageWidth(pi);
            const pl = (sc.clientWidth - pw) / 2;
            const px = (cx - pl) / zoom;
            const py = (cy - po) / zoom;
            if (!e.shiftKey && pi === picBbox.pageIndex &&
                px >= picBbox.x && px <= picBbox.x + picBbox.w &&
                py >= picBbox.y && py <= picBbox.y + picBbox.h) {
              try {
                const props = this.getObjectProperties(ref);
                if (!props.treatAsChar) {
                  e.preventDefault();
                  this.isPictureMoveDragging = true;
                  this.pictureMoveState = {
                    ref: { sec: ref.sec, ppi: ref.ppi, ci: ref.ci, type: ref.type },
                    origHorzOffset: props.horzOffset,
                    origVertOffset: props.vertOffset,
                    startPageX: px, startPageY: py,
                    lastPageX: px, lastPageY: py,
                    totalDeltaH: 0, totalDeltaV: 0,
                    pageIndex: pi,
                  };
                  this.container.style.cursor = 'move';
                  document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
                  this.textarea.focus();
                  return;
                }
              } catch { /* ignore */ }
            }
          }
        }
      }
    }
    // Shift+클릭: 다중 선택을 위해 선택 유지 (아래 findPictureAtClick에서 처리)
    if (!e.shiftKey) {
      this.exitPictureObjectSelectionIfNeeded();
    }
    } // else (단일 선택) 블록 끝
  }

  // 셀 선택 모드 중 클릭 처리
  if (this.cursor.isInCellSelectionMode()) {
    // 우클릭 → 셀 선택 영역 유지 (컨텍스트 메뉴에서 처리)
    if (e.button === 2) return;
    if (e.shiftKey || e.ctrlKey || e.metaKey) {
      // 클릭된 셀의 row/col 가져오기
      const cellRC = this.hitTestCellRowCol(e);
      if (cellRC) {
        e.preventDefault();
        if (e.shiftKey) {
          this.cursor.shiftSelectCell(cellRC.row, cellRC.col);
        } else {
          this.cursor.ctrlToggleCell(cellRC.row, cellRC.col);
        }
        this.updateCellSelection();
        this.textarea.focus();
        return;
      }
    }
    // 경계선 클릭 → 셀 선택 유지 + 리사이즈 드래그 시작
    if (e.button === 0 && this.tableResizeRenderer) {
      const ctx = this.cursor.getCellTableContext();
      if (ctx) {
        try {
          const bboxes = this.wasm.getTableCellBboxes(ctx.sec, ctx.ppi, ctx.ci);
          this.cachedTableRef = { sec: ctx.sec, ppi: ctx.ppi, ci: ctx.ci };
          this.cachedCellBboxes = bboxes;
          const zoom = this.viewportManager.getZoom();
          const scrollContent = this.container.querySelector('#scroll-content');
          if (scrollContent) {
            const contentRect = scrollContent.getBoundingClientRect();
            const contentX = e.clientX - contentRect.left;
            const contentY = e.clientY - contentRect.top;
            const pageIdx = this.virtualScroll.getPageAtY(contentY);
            const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
            const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
            const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;
            const pageX = (contentX - pageLeft) / zoom;
            const pageY = (contentY - pageOffset) / zoom;
            const pageBboxes = bboxes.filter((b: any) => b.pageIndex === pageIdx);
            const edge = this.tableResizeRenderer.hitTestBorder(pageX, pageY, pageBboxes);
            if (edge) {
              e.preventDefault();
              this.startResizeDrag(edge, pageX, pageY, pageBboxes);
              this.textarea.focus();
              return;
            }
          }
        } catch { /* bboxes 조회 실패 시 무시 */ }
      }
    }
    // 일반 좌클릭 → 셀 선택 모드 종료
    this.cursor.exitCellSelectionMode();
    this.cellSelectionRenderer?.clear();
  }

  // 우클릭 → 텍스트 선택 블록 유지 (컨텍스트 메뉴에서 처리)
  if (e.button === 2) {
    e.preventDefault();
    this.textarea.focus();
    return;
  }

  // 브라우저 기본 포커스 동작을 방지하여 textarea 포커스 유지
  e.preventDefault();

  const zoom = this.viewportManager.getZoom();
  const scrollContent = this.container.querySelector('#scroll-content')!;
  const contentRect = scrollContent.getBoundingClientRect();

  // 클릭 좌표 → scroll-content 내 좌표 (getBoundingClientRect가 스크롤 반영)
  const contentX = e.clientX - contentRect.left;
  const contentY = e.clientY - contentRect.top;

  // 페이지 찾기
  const pageIdx = this.virtualScroll.getPageAtY(contentY);
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);

  // CSS 중앙 정렬 보정 (left:50%; transform:translateX(-50%))
  const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
  const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;

  // 페이지 내 좌표 (줌 역산)
  const pageX = (contentX - pageLeft) / zoom;
  const pageY = (contentY - pageOffset) / zoom;

  // 표 경계선 클릭 → 리사이즈 드래그 시작
  if (e.button === 0 && this.tableResizeRenderer && this.cachedCellBboxes && this.cachedTableRef) {
    const pageBboxes = this.cachedCellBboxes.filter((b: any) => b.pageIndex === pageIdx);
    const edge = this.tableResizeRenderer.hitTestBorder(pageX, pageY, pageBboxes);
    if (edge) {
      e.preventDefault();
      this.startResizeDrag(edge, pageX, pageY, pageBboxes);
      this.textarea.focus();
      return;
    }
  }

  // 머리말/꼬리말 편집 모드에서 본문 영역 클릭 → 편집 모드 탈출
  if (this.cursor.isInHeaderFooter()) {
    try {
      const hfHit = this.wasm.hitTestHeaderFooter(pageIdx, pageX, pageY);
      if (!hfHit.hit) {
        // 본문 영역 클릭 → 편집 모드 탈출 (스크롤 없이 — 이후 hitTest에서 커서 재배치)
        this.cursor.exitHeaderFooterMode();
        this.eventBus.emit('headerFooterModeChanged', 'none');
        // 본문 hitTest로 계속 진행
      } else {
        // 머리말/꼬리말 영역 클릭 → 내부 텍스트 히트테스트로 커서 이동
        try {
          const isHeader = this.cursor.headerFooterMode === 'header';
          const inHfHit = this.wasm.hitTestInHeaderFooter(pageIdx, isHeader, pageX, pageY);
          if (inHfHit.hit && inHfHit.paraIndex !== undefined && inHfHit.charOffset !== undefined) {
            this.cursor.setHfCursorPosition(inHfHit.paraIndex, inHfHit.charOffset);
            this.updateCaret();
          }
        } catch { /* 무시 */ }
        this.textarea.focus();
        return;
      }
    } catch { /* 무시 */ }
  }

  // 각주 편집 모드에서 클릭 처리
  if (this.cursor.isInFootnote()) {
    try {
      const fnHit = this.wasm.hitTestFootnote(pageIdx, pageX, pageY);
      if (!fnHit.hit) {
        // 본문 영역 클릭 → 각주 편집 모드 탈출
        this.cursor.exitFootnoteMode();
        this.eventBus.emit('footnoteModeChanged', false);
        // 본문 hitTest로 계속 진행
      } else {
        // 각주 영역 클릭 → 내부 텍스트 히트테스트로 커서 이동
        try {
          const inFnHit = this.wasm.hitTestInFootnote(pageIdx, pageX, pageY);
          if (inFnHit.hit && inFnHit.fnParaIndex !== undefined && inFnHit.charOffset !== undefined) {
            this.cursor.setFnCursorPosition(inFnHit.fnParaIndex, inFnHit.charOffset);
            this.updateCaret();
          }
        } catch { /* 무시 */ }
        this.textarea.focus();
        return;
      }
    } catch { /* 무시 */ }
  }

  // 각주 영역 클릭 → 각주 편집 모드 진입
  if (!this.cursor.isInFootnote()) {
    try {
      const fnHit = this.wasm.hitTestFootnote(pageIdx, pageX, pageY);
      if (fnHit.hit) {
        // hitTestInFootnote로 정확한 footnoteIndex와 커서 위치를 얻기
        const inFnHit = this.wasm.hitTestInFootnote(pageIdx, pageX, pageY);
        if (inFnHit.hit && inFnHit.footnoteIndex !== undefined) {
          const pageInfo = this.wasm.getPageFootnoteInfo(pageIdx, inFnHit.footnoteIndex);
          if (pageInfo && pageInfo.sourceType === 'body') {
            this.cursor.enterFootnoteMode(
              pageInfo.sectionIdx, pageInfo.paraIdx, pageInfo.controlIdx,
              inFnHit.footnoteIndex, pageIdx,
            );
            this.eventBus.emit('footnoteModeChanged', true);
            if (inFnHit.fnParaIndex !== undefined && inFnHit.charOffset !== undefined) {
              this.cursor.setFnCursorPosition(inFnHit.fnParaIndex, inFnHit.charOffset);
            }
            this.updateCaret();
            this.textarea.focus();
            return;
          }
        }
      }
    } catch { /* 무시 */ }
  }

  try {
    const hit = this.wasm.hitTest(pageIdx, pageX, pageY);

    // 머리말/꼬리말 마커 para_index(usize::MAX - hf_idx) 감지 → 무시
    if (hit.paragraphIndex >= 0xFFFFFF00) {
      this.textarea.focus();
      return;
    }

    // 표 경계선 클릭 감지 → 표 객체 선택 (셀 내부에서 외곽 클릭)
    if (hit.parentParaIndex !== undefined && hit.controlIndex !== undefined && !hit.isTextBox) {
      if (this.isTableBorderClick(pageX, pageY, hit.sectionIndex, hit.parentParaIndex, hit.controlIndex)) {
        this.cursor.clearSelection();
        this.cursor.moveTo(hit); // 셀 위치로 이동 (유효한 렌더링 위치)
        this.cursor.enterTableObjectSelectionDirect(hit.sectionIndex, hit.parentParaIndex, hit.controlIndex);
        this.active = true;
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderTableObjectSelection();
        this.eventBus.emit('table-object-selection-changed', true);
        // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
        // this.checkTransparentBordersTransition();
        this.textarea.focus();
        return;
      }
    }

    // 표 외곽 클릭 감지 → 표 객체 선택 (셀 바깥에서 외곽 근처 클릭)
    if (hit.parentParaIndex === undefined || hit.controlIndex === undefined) {
      const tableHit = this.findTableByOuterClick(pageX, pageY, hit.sectionIndex, hit.paragraphIndex);
      if (tableHit) {
        this.cursor.clearSelection();
        this.cursor.enterTableObjectSelectionDirect(tableHit.sec, tableHit.ppi, tableHit.ci);
        this.active = true;
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderTableObjectSelection();
        this.eventBus.emit('table-object-selection-changed', true);
        // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
        // this.checkTransparentBordersTransition();
        this.textarea.focus();
        return;
      }
    }

    // 글상자 내부 텍스트 직접 히트 → 바로 캐럿 진입
    if (hit.isTextBox) {
      this.exitPictureObjectSelectionIfNeeded();
      this.cursor.clearSelection();
      this.cursor.moveTo(hit);
      this.cursor.resetPreferredX();
      this.cursor.setAnchor();
      this.active = true;
      this.isDragging = true;
      this.updateCaret();
      // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
      // this.checkTransparentBordersTransition();
      document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
      this.textarea.focus();
      return;
    }

    // 그림/글상자 클릭 감지
    {
      const picHit = this.findPictureAtClick(pageIdx, pageX, pageY);
      if (picHit) {
        // Shift+클릭: 다중 선택 + 맨 앞으로 이동
        if (e.shiftKey && this.cursor.isInPictureObjectSelection()) {
          bringShapeToFront.call(this, picHit);
          const selType = picHit.type === 'shape' ? 'shape' as const : picHit.type as any;
          this.cursor.togglePictureObjectSelection(picHit.sec, picHit.ppi, picHit.ci, selType);
          this.caret.hide();
          this.selectionRenderer.clear();
          this.renderPictureObjectSelection();
          this.eventBus.emit('picture-object-selection-changed', this.cursor.isInPictureObjectSelection());
          this.textarea.focus();
          return;
        }

        if (picHit.type === 'line') {
          // 직선 → 맨 앞으로 이동 후 객체 선택
          bringShapeToFront.call(this, picHit);
          this.cursor.clearSelection();
          this.exitPictureObjectSelectionIfNeeded();
          this.cursor.enterPictureObjectSelectionDirect(picHit.sec, picHit.ppi, picHit.ci, 'line');
          this.active = true;
          this.caret.hide();
          this.selectionRenderer.clear();
          this.renderPictureObjectSelection();
          this.eventBus.emit('picture-object-selection-changed', true);
          this.textarea.focus();
          return;
        }
        if (picHit.type === 'shape') {
          // 이미 편집 중인 같은 글상자 → hitTest 위치로 커서 이동
          if (this.cursor.isInTextBox()) {
            const pos = this.cursor.getPosition();
            if (pos.parentParaIndex === picHit.ppi && pos.controlIndex === picHit.ci) {
              this.cursor.clearSelection();
              this.cursor.moveTo(hit);
              this.cursor.resetPreferredX();
              this.cursor.setAnchor();
              this.active = true;
              this.isDragging = true;
              this.updateCaret();
              document.addEventListener('mouseup', this.onMouseUpBound, { once: true });
              this.textarea.focus();
              return;
            }
          }
          // 단일 클릭 → 객체 선택 + 맨 앞으로 이동
          bringShapeToFront.call(this, picHit);
          this.cursor.clearSelection();
          this.exitPictureObjectSelectionIfNeeded();
          this.cursor.enterPictureObjectSelectionDirect(picHit.sec, picHit.ppi, picHit.ci, 'shape');
          this.active = true;
          this.caret.hide();
          this.selectionRenderer.clear();
          this.renderPictureObjectSelection();
          this.eventBus.emit('picture-object-selection-changed', true);
          this.textarea.focus();
          return;
        }
        // 이미지/방정식 → 객체 선택 (z-order 미지원)
        this.cursor.clearSelection();
        this.exitPictureObjectSelectionIfNeeded();
        this.cursor.enterPictureObjectSelectionDirect(picHit.sec, picHit.ppi, picHit.ci, picHit.type, picHit.cellIdx, picHit.cellParaIdx);
        this.active = true;
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderPictureObjectSelection();
        this.eventBus.emit('picture-object-selection-changed', true);
        this.textarea.focus();
        return;
      }
    }

    // 양식 개체 클릭 감지
    {
      const formHit = this.wasm.getFormObjectAt(pageIdx, pageX, pageY);
      if (formHit.found) {
        this.handleFormObjectClick(formHit, pageIdx, zoom);
        this.textarea.focus();
        return;
      }
    }

    if (e.shiftKey) {
      // Shift+클릭: 현재 위치에서 클릭 위치까지 선택 확장
      this.cursor.setAnchor(); // anchor가 없으면 현재 커서 위치를 anchor로
      this.cursor.moveTo(hit);
      this.active = true;
      this.updateCaret();
      this.textarea.focus();
      return;
    }

    // 일반 클릭: 커서 배치 + 드래그 시작
    this.cursor.clearSelection();
    this.cursor.moveTo(hit);
    this.cursor.resetPreferredX();
    this.cursor.setAnchor(); // 드래그 시작점(anchor) 설정
    this.active = true;
    this.isDragging = true;

    const rect = this.cursor.getRect();
    if (rect) {
      this.caret.show(rect, zoom);
    }
    this.selectionRenderer.clear();
    this.emitCursorFormatState();
    // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
    // this.checkTransparentBordersTransition();

    // 필드(누름틀) 마커 표시 + 상태 표시줄 갱신
    this.updateFieldMarkers();

    // 드래그 종료를 위한 mouseup 리스너 (document에 등록)
    document.addEventListener('mouseup', this.onMouseUpBound, { once: true });

    // textarea에 포커스하여 키보드 입력 수신
    this.textarea.focus();
  } catch (err) {
    console.warn('[InputHandler] hitTest 실패:', err);
  }
}

export function onDblClick(this: any, e: MouseEvent): void {
  if (!this.active) return;
  if (this.imagePlacementMode || this.textboxPlacementMode) return;

  // 다각형 그리기 모드: 더블클릭으로 완료
  if (this.polygonDrawingMode) {
    e.preventDefault();
    this.finishPolygonDrawing();
    return;
  }

  const target = e.target as HTMLElement;
  if (target.closest('#menu-bar') || target.closest('#icon-toolbar') || target.closest('#style-bar')) return;

  // 머리말/꼬리말 영역 더블클릭 → 편집 모드 진입
  if (!this.cursor.isInHeaderFooter()) {
    try {
      const zoom = this.viewportManager.getZoom();
      const sc = this.container.querySelector('#scroll-content');
      if (sc) {
        const cr = sc.getBoundingClientRect();
        const contentX = e.clientX - cr.left;
        const contentY = e.clientY - cr.top;
        const pageIdx = this.virtualScroll.getPageAtY(contentY);
        if (pageIdx >= 0) {
          const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
          const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
          const pageLeft = ((sc as HTMLElement).clientWidth - pageDisplayWidth) / 2;
          const pageX = (contentX - pageLeft) / zoom;
          const pageY = (contentY - pageOffset) / zoom;
          const hfHit = this.wasm.hitTestHeaderFooter(pageIdx, pageX, pageY);
          if (hfHit.hit) {
            e.preventDefault();
            const sectionIdx = hfHit.sectionIndex ?? 0;
            const applyTo = hfHit.applyTo ?? 0;
            const isHeader = hfHit.isHeader ?? true;
            // 머리말/꼬리말이 없으면 생성
            const existing = JSON.parse(this.wasm.getHeaderFooter(sectionIdx, isHeader, applyTo));
            if (!existing.exists) {
              this.wasm.createHeaderFooter(sectionIdx, isHeader, applyTo);
            }
            this.cursor.enterHeaderFooterMode(isHeader, sectionIdx, applyTo, pageIdx);
            this.eventBus.emit('headerFooterModeChanged', isHeader ? 'header' : 'footer');
            this.updateCaret();
            this.textarea.focus();
            return;
          }
        }
      }
    } catch { /* hitTest 실패 시 무시 */ }
  }

  // 객체 선택 중 더블클릭
  if (this.cursor.isInPictureObjectSelection()) {
    const ref = this.cursor.getSelectedPictureRef();
    // 수식 객체 → 수식 편집 대화상자
    if (ref && ref.type === 'equation') {
      e.preventDefault();
      this.eventBus.emit('equation-edit-request', { sec: ref.sec, ppi: ref.ppi, ci: ref.ci });
      return;
    }
    // 글상자 객체 → 텍스트 편집 진입
    if (ref && ref.type === 'shape') {
      e.preventDefault();
      this.cursor.exitPictureObjectSelection();
      this.pictureObjectRenderer?.clear();
      this.eventBus.emit('picture-object-selection-changed', false);
      this.enterTextboxEditing(ref.sec, ref.ppi, ref.ci);
      this.textarea.focus();
      return;
    }
  }
}

export function onContextMenu(this: any, e: MouseEvent): void {
  e.preventDefault();
  if (!this.active || !this.contextMenu) return;

  // 그림 객체 선택 중 우클릭 → 그림 객체 메뉴 표시 (선택 유지)
  if (this.cursor.isInPictureObjectSelection()) {
    this.contextMenu.show(e.clientX, e.clientY, this.getPictureObjectContextMenuItems());
    return;
  }

  // 표 객체 선택 중 우클릭 → 표 객체 메뉴 표시 (선택 유지)
  if (this.cursor.isInTableObjectSelection()) {
    this.contextMenu.show(e.clientX, e.clientY, this.getTableObjectContextMenuItems());
    return;
  }

  // 머리말/꼬리말 편집 모드 우클릭 → 전용 메뉴 (글자/문단 모양 포함)
  if (this.cursor.isInHeaderFooter()) {
    this.contextMenu.show(e.clientX, e.clientY, this.getDefaultContextMenuItems());
    return;
  }

  // 클릭 좌표 → hitTest로 표 셀 내부/외부 판별
  const zoom = this.viewportManager.getZoom();
  const scrollContent = this.container.querySelector('#scroll-content')!;
  const contentRect = scrollContent.getBoundingClientRect();
  const contentX = e.clientX - contentRect.left;
  const contentY = e.clientY - contentRect.top;
  const pageIdx = this.virtualScroll.getPageAtY(contentY);
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
  const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
  const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;
  const pageX = (contentX - pageLeft) / zoom;
  const pageY = (contentY - pageOffset) / zoom;

  let inTable = false;
  try {
    const hit = this.wasm.hitTest(pageIdx, pageX, pageY);
    inTable = hit.parentParaIndex !== undefined && !hit.isTextBox;
  } catch { /* hitTest 실패 시 표 밖으로 처리 */ }

  let items: ContextMenuItem[] = inTable
    ? this.getTableContextMenuItems()
    : this.getDefaultContextMenuItems();

  // 누름틀 필드 내부이면 필드 메뉴 항목 추가
  try {
    const fi = this.wasm.getFieldInfoAt(this.cursor.getPosition());
    if (fi.inField) {
      items = [
        ...items,
        { type: 'separator' },
        { type: 'command', commandId: 'field:edit', label: '누름틀 고치기(E)...' },
        { type: 'command', commandId: 'field:remove', label: '누름틀 지우기(J)' },
      ];
    }
  } catch { /* 무시 */ }

  this.contextMenu.show(e.clientX, e.clientY, items);
}

export function onMouseMove(this: any, e: MouseEvent): void {
  // 연결선 드로잉 모드: 연결점 오버레이 + 프리뷰
  if (this.connectorDrawingMode) {
    const sc = this.container.querySelector('#scroll-content');
    if (sc) {
      const zoom = this.viewportManager.getZoom();
      const cr = sc.getBoundingClientRect();
      const cx = e.clientX - cr.left;
      const cy = e.clientY - cr.top;
      const pi = this.virtualScroll.getPageAtY(cy);
      const po = this.virtualScroll.getPageOffset(pi);
      const pw = this.virtualScroll.getPageWidth(pi);
      const pl = (sc.clientWidth - pw) / 2;
      const pageX = (cx - pl) / zoom;
      const pageY = (cy - po) / zoom;
      _connector.showConnectionPointOverlay.call(this, pi, pageX, pageY);
      if (this.connectorStartRef) {
        _connector.updateConnectorPreview.call(this,
          this.connectorStartRef.x, this.connectorStartRef.y,
          pageX, pageY, this.connectorStartRef.pageIdx ?? pi);
      }
    }
    return;
  }

  // 다각형 그리기 모드: 마우스 이동 시 프리뷰
  if (this.polygonDrawingMode && this.polygonPoints.length > 0) {
    this.updatePolygonOverlay(e.clientX, e.clientY);
    return;
  }

  // 그림 배치 모드 드래그 중
  if (this.imagePlacementMode && this.imagePlacementDrag) {
    this.imagePlacementDrag.currentClientX = e.clientX;
    this.imagePlacementDrag.currentClientY = e.clientY;
    const dx = e.clientX - this.imagePlacementDrag.startClientX;
    const dy = e.clientY - this.imagePlacementDrag.startClientY;
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
      this.imagePlacementDrag.isDragging = true;
    }
    this.showImagePlacementOverlay(
      this.imagePlacementDrag.startClientX, this.imagePlacementDrag.startClientY,
      e.clientX, e.clientY,
    );
    return;
  }

  // 글상자 배치 모드 드래그 중
  if (this.textboxPlacementMode && this.textboxPlacementDrag) {
    this.textboxPlacementDrag.currentClientX = e.clientX;
    this.textboxPlacementDrag.currentClientY = e.clientY;
    const dx = e.clientX - this.textboxPlacementDrag.startClientX;
    const dy = e.clientY - this.textboxPlacementDrag.startClientY;
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
      this.textboxPlacementDrag.isDragging = true;
    }
    this.showTextboxPlacementOverlay(
      this.textboxPlacementDrag.startClientX, this.textboxPlacementDrag.startClientY,
      e.clientX, e.clientY, e.shiftKey,
    );
    return;
  }

  // 직선 끝점 드래그 중
  if (this.isLineEndpointDragging && this.lineEndpointState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isLineEndpointDragging || !this.lineEndpointState) return;
      const st = this.lineEndpointState;
      const sc = this.container.querySelector('#scroll-content');
      if (!sc) return;
      const cr = sc.getBoundingClientRect();
      const cx = e.clientX - cr.left;
      const cy = e.clientY - cr.top;
      const px = (cx - st.pageLeft) / st.zoom;
      const py = (cy - st.pageOffset) / st.zoom;
      // page px → HWPUNIT
      const PX2HWP = 75;
      let newX = Math.round(px * PX2HWP);
      let newY = Math.round(py * PX2HWP);

      // Shift: 수평/수직/45도 스냅
      if (e.shiftKey) {
        const bbox = this.findPictureBbox(st.ref);
        if (bbox) {
          const gx1 = Math.round(bbox.x1 * PX2HWP);
          const gy1 = Math.round(bbox.y1 * PX2HWP);
          const gx2 = Math.round(bbox.x2 * PX2HWP);
          const gy2 = Math.round(bbox.y2 * PX2HWP);
          const [fx, fy] = st.endpoint === 'start' ? [gx2, gy2] : [gx1, gy1];
          const ddx = newX - fx, ddy = newY - fy;
          const angle = Math.atan2(ddy, ddx);
          const snapAngle = Math.round(angle / (Math.PI / 4)) * (Math.PI / 4);
          const dist = Math.sqrt(ddx * ddx + ddy * ddy);
          newX = fx + Math.round(dist * Math.cos(snapAngle));
          newY = fy + Math.round(dist * Math.sin(snapAngle));
        }
      }
      // 고정점: 현재 속성에서 가져옴
      try {
        const bbox = this.findPictureBbox(st.ref);
        if (!bbox) return;
        // 현재 시작/끝 글로벌 좌표
        const gx1 = Math.round(bbox.x1 * PX2HWP);
        const gy1 = Math.round(bbox.y1 * PX2HWP);
        const gx2 = Math.round(bbox.x2 * PX2HWP);
        const gy2 = Math.round(bbox.y2 * PX2HWP);
        const [sx, sy, ex, ey] = st.endpoint === 'start'
          ? [newX, newY, gx2, gy2]
          : [gx1, gy1, newX, newY];
        this.wasm.moveLineEndpoint(st.ref.sec, st.ref.ppi, st.ref.ci, sx, sy, ex, ey);
        this.eventBus.emit('document-changed');
        this.renderPictureObjectSelection();
      } catch { /* ignore */ }
    });
    return;
  }

  // 표 이동 드래그 중
  if (this.isMoveDragging && this.moveDragState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isMoveDragging || !this.moveDragState) return;
      this.updateMoveDrag(e);
    });
    return;
  }

  // 그림 이동 드래그 중
  if (this.isPictureMoveDragging && this.pictureMoveState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isPictureMoveDragging || !this.pictureMoveState) return;
      this.updatePictureMoveDrag(e);
    });
    return;
  }

  // 그림 회전 드래그 중: 실시간 각도 계산
  if (this.isPictureRotateDragging && this.pictureRotateState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isPictureRotateDragging || !this.pictureRotateState) return;
      this.updatePictureRotateDrag(e);
    });
    return;
  }

  // 그림 리사이즈 드래그 중: 실시간 피드백
  if (this.isPictureResizeDragging && this.pictureResizeState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isPictureResizeDragging || !this.pictureResizeState) return;
      this.updatePictureResizeDrag(e);

      // 드래그 중에도 커서 방향 업데이트 (Flipping 대응)
      const state = this.pictureResizeState;
      const angleDeg = (state.rotationAngle ?? 0) as number;
      this.container.style.cursor = getRotatedCursor(state.dir, angleDeg);
    });
    return;
  }

  // 리사이즈 드래그 중: 마커 위치 갱신
  if (this.isResizeDragging && this.resizeDragState) {
    if (this.dragRafId) return;
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isResizeDragging || !this.resizeDragState) return;
      this.updateResizeDrag(e);
    });
    return;
  }

  // 드래그 중: requestAnimationFrame으로 throttle하여 성능 확보
  if (this.isDragging) {
    if (this.dragRafId) return; // 이미 예약된 프레임이 있으면 건너뜀
    this.dragRafId = requestAnimationFrame(() => {
      this.dragRafId = 0;
      if (!this.isDragging) return;
      const hit = this.hitTestFromEvent(e);
      if (hit && hit.paragraphIndex < 0xFFFFFF00) {
        this.cursor.moveTo(hit);
        this.updateCaret();
      }
    });
    return;
  }

  // 그림 객체 선택 중 → 핸들 커서 변경
  if (this.cursor.isInPictureObjectSelection() && this.pictureObjectRenderer) {
    const scrollContent = this.container.querySelector('#scroll-content');
    if (!scrollContent) return;
    const contentRect = scrollContent.getBoundingClientRect();
    const x = e.clientX - contentRect.left;
    const y = e.clientY - contentRect.top;
    const dir = this.pictureObjectRenderer.getHandleAtPoint(x, y);
    if (dir) {
      if (dir === 'rotate') {
        this.container.style.cursor = 'grab';
      } else {
        // 회전된 도형의 경우 커서 방향도 회전시켜 표시
        let angleDeg = 0;
        const ref = this.cursor.getSelectedPictureRef();
        if (ref && ref.type === 'shape') {
          try {
            const props = this.getObjectProperties(ref);
            angleDeg = (props.rotationAngle ?? 0) as number;
          } catch { /* ignore */ }
        }
        this.container.style.cursor = getRotatedCursor(dir, angleDeg);
      }
    } else {
      // 핸들 밖 → 그림 본체 위이면 move 커서
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        const picBbox = this.findPictureBbox(ref);
        if (picBbox) {
          const zoom = this.viewportManager.getZoom();
          const pi = this.virtualScroll.getPageAtY(y);
          const po = this.virtualScroll.getPageOffset(pi);
          const pw = this.virtualScroll.getPageWidth(pi);
          const pl = (scrollContent.clientWidth - pw) / 2;
          const px = (x - pl) / zoom;
          const py = (y - po) / zoom;
          if (pi === picBbox.pageIndex &&
              px >= picBbox.x && px <= picBbox.x + picBbox.w &&
              py >= picBbox.y && py <= picBbox.y + picBbox.h) {
            try {
              const props = this.getObjectProperties(ref);
              this.container.style.cursor = props.treatAsChar ? '' : 'move';
            } catch {
              this.container.style.cursor = '';
            }
          } else {
            this.container.style.cursor = '';
          }
        } else {
          this.container.style.cursor = '';
        }
      } else {
        this.container.style.cursor = '';
      }
    }
    return;
  }

  // 표 객체 선택 중 → 핸들 커서 변경
  if (this.cursor.isInTableObjectSelection() && this.tableObjectRenderer) {
    const scrollContent = this.container.querySelector('#scroll-content');
    if (!scrollContent) return;
    const contentRect = scrollContent.getBoundingClientRect();
    const x = e.clientX - contentRect.left;
    const y = e.clientY - contentRect.top;

    const dir = this.tableObjectRenderer.getHandleAtPoint(x, y);
    if (dir) {
      const cursorMap: Record<string, string> = {
        nw: 'nwse-resize', se: 'nwse-resize',
        ne: 'nesw-resize', sw: 'nesw-resize',
        n: 'ns-resize', s: 'ns-resize',
        e: 'ew-resize', w: 'ew-resize',
      };
      this.container.style.cursor = cursorMap[dir] ?? '';
    } else {
      // 핸들 밖이면 표 내부인지 확인 → move 커서
      const ref = this.cursor.getSelectedTableRef();
      if (ref) {
        const zoom = this.viewportManager.getZoom();
        const pi = this.virtualScroll.getPageAtY(y);
        const po = this.virtualScroll.getPageOffset(pi);
        const pw = this.virtualScroll.getPageWidth(pi);
        const pl = (scrollContent.clientWidth - pw) / 2;
        const px = (x - pl) / zoom;
        const py = (y - po) / zoom;
        try {
          const bbox = this.wasm.getTableBBox(ref.sec, ref.ppi, ref.ci);
          if (px >= bbox.x && px <= bbox.x + bbox.width &&
              py >= bbox.y && py <= bbox.y + bbox.height) {
            this.container.style.cursor = 'move';
          } else {
            this.container.style.cursor = '';
          }
        } catch {
          this.container.style.cursor = '';
        }
      } else {
        this.container.style.cursor = '';
      }
    }
    return;
  }

  // 표 경계선 hover 감지 (RAF throttle)
  if (this.tableResizeRenderer) {
    if (this.resizeHoverRafId) return;
    this.resizeHoverRafId = requestAnimationFrame(() => {
      this.resizeHoverRafId = 0;
      this.handleResizeHover(e);
    });
  } else {
    if (this.container.style.cursor) {
      this.container.style.cursor = '';
    }
  }
}

export function handleResizeHover(this: any, e: MouseEvent): void {
  if (!this.tableResizeRenderer) return;

  const zoom = this.viewportManager.getZoom();
  const scrollContent = this.container.querySelector('#scroll-content');
  if (!scrollContent) return;
  const contentRect = scrollContent.getBoundingClientRect();
  const contentX = e.clientX - contentRect.left;
  const contentY = e.clientY - contentRect.top;
  const pageIdx = this.virtualScroll.getPageAtY(contentY);
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
  const pageDisplayWidth = this.virtualScroll.getPageWidth(pageIdx);
  const pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2;
  const pageX = (contentX - pageLeft) / zoom;
  const pageY = (contentY - pageOffset) / zoom;

  // hitTest로 표 셀 위인지 확인
  let tableRef: { sec: number; ppi: number; ci: number } | null = null;
  try {
    const hit = this.wasm.hitTest(pageIdx, pageX, pageY);
    if (hit.parentParaIndex !== undefined && hit.controlIndex !== undefined && !hit.isTextBox) {
      tableRef = { sec: hit.sectionIndex, ppi: hit.parentParaIndex, ci: hit.controlIndex };
    }
  } catch { /* hitTest 실패 시 표 밖 */ }

  if (!tableRef) {
    this.tableResizeRenderer.clear();
    this.cachedTableRef = null;
    this.cachedCellBboxes = null;
    // 개체(도형/연결선) hover 감지: 커서 변경
    const picHit = this.findPictureAtClick(pageIdx, pageX, pageY);
    this.container.style.cursor = picHit ? 'pointer' : '';
    return;
  }

  // 셀 bbox 캐싱 (같은 표면 재사용)
  if (!this.cachedTableRef ||
      this.cachedTableRef.sec !== tableRef.sec ||
      this.cachedTableRef.ppi !== tableRef.ppi ||
      this.cachedTableRef.ci !== tableRef.ci) {
    try {
      this.cachedCellBboxes = this.wasm.getTableCellBboxes(tableRef.sec, tableRef.ppi, tableRef.ci);
      this.cachedTableRef = tableRef;
    } catch {
      this.cachedCellBboxes = null;
      this.cachedTableRef = null;
    }
  }

  if (!this.cachedCellBboxes || this.cachedCellBboxes.length === 0) {
    this.tableResizeRenderer.clear();
    if (this.container.style.cursor) {
      this.container.style.cursor = '';
    }
    return;
  }

  // 해당 페이지의 셀만 필터
  const pageBboxes = this.cachedCellBboxes.filter((b: any) => b.pageIndex === pageIdx);
  if (pageBboxes.length === 0) {
    this.tableResizeRenderer.clear();
    if (this.container.style.cursor) {
      this.container.style.cursor = '';
    }
    return;
  }

  // 경계선 감지
  const edge = this.tableResizeRenderer.hitTestBorder(pageX, pageY, pageBboxes);
  if (edge) {
    this.container.style.cursor = edge.type === 'row' ? 'row-resize' : 'col-resize';
    this.tableResizeRenderer.showMarker(edge, pageBboxes, zoom);
  } else {
    this.tableResizeRenderer.clear();
    if (this.container.style.cursor) {
      this.container.style.cursor = '';
    }
  }
}

export function onMouseUp(this: any, _e: MouseEvent): void {
  // 그림 배치 모드 마우스업 → 삽입 실행
  if (this.imagePlacementMode && this.imagePlacementDrag && this.imagePlacementData) {
    this.finishImagePlacement(_e);
    return;
  }

  // 글상자 배치 모드 마우스업 → 삽입 실행
  if (this.textboxPlacementMode && this.textboxPlacementDrag) {
    this.finishTextboxPlacement(_e);
    return;
  }

  // 표 이동 드래그 종료
  if (this.isMoveDragging) {
    this.finishMoveDrag();
    return;
  }

  // 그림 이동 드래그 종료
  if (this.isPictureMoveDragging) {
    this.finishPictureMoveDrag();
    return;
  }

  // 그림 회전 드래그 종료
  if (this.isPictureRotateDragging) {
    this.finishPictureRotateDrag(_e);
    return;
  }

  // 직선 끝점 드래그 종료
  if (this.isLineEndpointDragging) {
    this.isLineEndpointDragging = false;
    this.lineEndpointState = null;
    this.container.style.cursor = '';
    if (this.dragRafId) { cancelAnimationFrame(this.dragRafId); this.dragRafId = 0; }
    return;
  }

  // 그림 리사이즈 드래그 종료
  if (this.isPictureResizeDragging) {
    this.finishPictureResizeDrag(_e);
    return;
  }

  // 리사이즈 드래그 종료
  if (this.isResizeDragging) {
    this.finishResizeDrag(_e);
    return;
  }

  if (!this.isDragging) return;
  this.isDragging = false;
  if (this.dragRafId) {
    cancelAnimationFrame(this.dragRafId);
    this.dragRafId = 0;
  }

  // anchor와 focus가 같으면 선택 해제 (단순 클릭)
  const sel = this.cursor.getSelectionOrdered();
  if (sel) {
    const { start, end } = sel;
    const samePos =
      start.sectionIndex === end.sectionIndex &&
      start.paragraphIndex === end.paragraphIndex &&
      start.charOffset === end.charOffset &&
      start.cellParaIndex === end.cellParaIndex;
    if (samePos) {
      this.cursor.clearSelection();
    }
  }

  this.updateCaret();
}


/**
 * 회전각을 반영하여 적절한 리사이즈 커서 이름을 반환한다.
 * @param dir 기본 방향 ('nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w')
 * @param angleDeg 회전각 (도)
 */
function bringShapeToFront(this: any, picHit: any): void {
  if (picHit.type === 'shape' || picHit.type === 'line' || picHit.type === 'group') {
    try {
      this.wasm.changeShapeZOrder(picHit.sec, picHit.ppi, picHit.ci, 'front');
      this.eventBus.emit('document-changed');
    } catch { /* ignore */ }
  }
}

function getRotatedCursor(dir: string, angleDeg: number): string {
  const dirs = ['n', 'ne', 'e', 'se', 's', 'sw', 'w', 'nw'];
  const idx = dirs.indexOf(dir);
  if (idx === -1) return '';

  // 45도 단위로 인덱스 시프트 (회전각 정규화)
  const normalizedAngle = ((angleDeg % 360) + 360) % 360;
  const shift = Math.round(normalizedAngle / 45);
  const rotatedDir = dirs[(idx + shift) % 8];

  const cursorMap: Record<string, string> = {
    n: 'ns-resize', s: 'ns-resize',
    e: 'ew-resize', w: 'ew-resize',
    nw: 'nwse-resize', se: 'nwse-resize',
    ne: 'nesw-resize', sw: 'nesw-resize',
  };
  return cursorMap[rotatedDir] ?? '';
}
