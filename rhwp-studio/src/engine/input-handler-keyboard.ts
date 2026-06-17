/** input-handler keyboard methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import { InsertTextCommand, InsertLineBreakCommand, InsertTabCommand, SplitParagraphCommand, SplitParagraphInCellCommand } from './command';
import { matchShortcut, defaultShortcuts } from '@/command/shortcut-map';
import * as _connector from './input-handler-connector';
import {
  detectPlatformKind,
  getNavigationAction,
  shouldSuppressUnmappedNavigation,
  type NavigationAction,
  type NavigationKeyInput,
} from './navigation-keymap';
import type { DocumentPosition, CellPathLike } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';

const RHWP_CLIPBOARD_MARKER_RE = /<!--\s*rhwp-studio-clipboard:([A-Za-z0-9._:-]+)\s*-->/;

function createRhwpClipboardToken(): string {
  try {
    if (globalThis.crypto?.randomUUID) return globalThis.crypto.randomUUID();
  } catch { /* fallback below */ }
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

function escapeClipboardHtmlText(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function fallbackClipboardHtml(text: string): string {
  const lines = (text || '').split(/\r?\n/);
  const body = lines.map(line => `<p>${escapeClipboardHtmlText(line)}</p>`).join('\n');
  return `<html><body>\n<!--StartFragment-->\n${body}\n<!--EndFragment-->\n</body></html>`;
}

function markRhwpClipboardHtml(html: string, token: string): string {
  const marker = `<!--rhwp-studio-clipboard:${token}-->`;
  const base = html || fallbackClipboardHtml('');
  if (RHWP_CLIPBOARD_MARKER_RE.test(base)) {
    return base.replace(RHWP_CLIPBOARD_MARKER_RE, marker);
  }
  if (base.includes('<!--StartFragment-->')) {
    return base.replace('<!--StartFragment-->', `${marker}\n<!--StartFragment-->`);
  }
  return `${marker}\n${base}`;
}

function readRhwpClipboardToken(html: string): string | null {
  return RHWP_CLIPBOARD_MARKER_RE.exec(html)?.[1] ?? null;
}

function hasCurrentRhwpClipboardMarker(self: any, html: string): boolean {
  const token = readRhwpClipboardToken(html);
  return !!token && token === self.rhwpClipboardToken;
}

function isNestedCellPosition(pos: DocumentPosition): boolean {
  return pos.parentParaIndex !== undefined && (pos.cellPath?.length ?? 0) > 1;
}

type PictureDeleteRef = {
  sec: number;
  ppi: number;
  ci: number;
  type: 'image' | 'shape' | 'equation' | 'group' | 'line';
  cellPath?: CellPathLike;
};

function deleteSelectedObject(wasm: WasmBridge, ref: PictureDeleteRef): void {
  if (ref.type === 'image') {
    if (ref.cellPath && ref.cellPath.length > 0) {
      wasm.deleteCellPictureControlByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci);
    } else {
      wasm.deletePictureControl(ref.sec, ref.ppi, ref.ci);
    }
  } else if (ref.type === 'equation') {
    wasm.deleteEquationControl(ref.sec, ref.ppi, ref.ci);
  } else {
    wasm.deleteShapeControl(ref.sec, ref.ppi, ref.ci);
  }
}

function toNavigationKeyInput(e: KeyboardEvent): NavigationKeyInput {
  return {
    key: e.key,
    code: e.code,
    shiftKey: e.shiftKey,
    ctrlKey: e.ctrlKey,
    metaKey: e.metaKey,
    altKey: e.altKey,
  };
}

function executeNavigationAction(this: any, action: NavigationAction, shiftKey: boolean): void {
  if (shiftKey) this.cursor.setAnchor();
  else this.cursor.clearSelection();

  switch (action) {
    case 'wordBackward':
      this.cursor.moveToWordBoundary(-1);
      break;
    case 'wordForward':
      this.cursor.moveToWordBoundary(1);
      break;
    case 'lineStart':
      this.cursor.moveToLineStart();
      this.markCurrentFieldStartOutside?.();
      break;
    case 'lineEnd':
      this.cursor.moveToLineEnd();
      this.markCurrentFieldEndOutside?.();
      break;
    case 'paragraphBackward':
      this.cursor.moveToParagraphBoundary(-1);
      break;
    case 'paragraphForward':
      this.cursor.moveToParagraphBoundary(1);
      break;
  }

  this.updateCaret();
  if (shiftKey) this.updateSelection();
}

function handleNavigationShortcut(this: any, e: KeyboardEvent): boolean {
  const input = toNavigationKeyInput(e);
  const platform = detectPlatformKind();
  const action = getNavigationAction(input, platform);
  if (action) {
    e.preventDefault();
    executeNavigationAction.call(this, action, e.shiftKey);
    return true;
  }
  if (shouldSuppressUnmappedNavigation(input, platform)) {
    e.preventDefault();
    return true;
  }
  return false;
}

function positionAfterPasteResult(pos: DocumentPosition, parsed: any): DocumentPosition {
  const newPos: DocumentPosition = {
    sectionIndex: pos.sectionIndex,
    paragraphIndex: parsed.paraIdx ?? pos.paragraphIndex,
    charOffset: parsed.charOffset ?? pos.charOffset,
  };

  if (pos.parentParaIndex !== undefined) {
    const nextCellParaIndex = parsed.cellParaIdx ?? parsed.cellParaIndex ?? pos.cellParaIndex;
    newPos.parentParaIndex = pos.parentParaIndex;
    newPos.controlIndex = pos.controlIndex;
    newPos.cellIndex = pos.cellIndex;
    newPos.cellParaIndex = nextCellParaIndex;
    if (pos.cellPath) {
      newPos.cellPath = pos.cellPath.map((entry, index) =>
        index === pos.cellPath!.length - 1
          ? { ...entry, cellParaIndex: nextCellParaIndex ?? entry.cellParaIndex }
          : entry,
      );
    }
  }

  return newPos;
}

function pastePlainText(this: any, text: string, hasSelection: boolean): void {
  if (hasSelection) {
    this.deleteSelection();
  }
  if (!text) return;

  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    if (lines[i]) {
      this.executeOperation({ kind: 'command', command: new InsertTextCommand(this.cursor.getPosition(), lines[i]) });
    }
    if (i < lines.length - 1 && !this.cursor.isInCell()) {
      this.executeOperation({ kind: 'command', command: new SplitParagraphCommand(this.cursor.getPosition()) });
    }
  }
}

export function prepareRhwpInternalClipboardHtml(self: any, html: string, text = ''): string {
  const token = createRhwpClipboardToken();
  self.rhwpClipboardToken = token;
  return markRhwpClipboardHtml(html || fallbackClipboardHtml(text), token);
}

export async function writeTextHtmlToClipboard(text: string, html: string): Promise<void> {
  if (typeof ClipboardItem === 'undefined' || !navigator.clipboard?.write) {
    await navigator.clipboard.writeText(text);
    return;
  }
  const item = new ClipboardItem({
    'text/plain': new Blob([text], { type: 'text/plain' }),
    'text/html': new Blob([html], { type: 'text/html' }),
  });
  await navigator.clipboard.write([item]);
}

/** 비-PNG 이미지를 PNG Blob으로 변환한다. PNG는 그대로 반환. */
async function convertToPngBlob(data: Uint8Array, mime: string): Promise<Blob> {
  // new Uint8Array(data)로 ArrayBuffer 기반 복사 — WASM 반환 Uint8Array의 SharedArrayBuffer 호환 문제 방지
  const buf = new Uint8Array(data);
  if (mime === 'image/png') return new Blob([buf], { type: 'image/png' });
  const img = new Image();
  const url = URL.createObjectURL(new Blob([buf], { type: mime }));
  try {
    img.src = url;
    await img.decode();
    const canvas = document.createElement('canvas');
    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;
    canvas.getContext('2d')!.drawImage(img, 0, 0);
    return new Promise<Blob>((resolve, reject) =>
      canvas.toBlob(b => b ? resolve(b) : reject(new Error('toBlob failed')), 'image/png'));
  } finally {
    URL.revokeObjectURL(url);
  }
}

/** [Task #1161] 선택된 picture ref 의 cellPath 를 native cellPathJson 인자로 변환.
 * 셀/글상자 밖 picture(본문)는 빈 문자열 → native 가 본문 경로로 처리. */
export function pictureCellPathJson(
  ref: { cellPath?: Array<{ controlIndex: number; cellIndex: number; cellParaIndex: number }> } | null,
): string {
  return ref && ref.cellPath && ref.cellPath.length > 0 ? JSON.stringify(ref.cellPath) : '';
}

/** 이미지 컨트롤의 바이너리를 포함하여 시스템 클립보드에 기록한다. */
export async function writeImageToClipboard(
  wasm: WasmBridge, sec: number, ppi: number, ci: number,
  text: string, html: string, cellPathJson = '',
): Promise<void> {
  const imageData = wasm.getControlImageData(sec, ppi, ci, cellPathJson);
  const mime = wasm.getControlImageMime(sec, ppi, ci, cellPathJson);
  const pngBlob = await convertToPngBlob(imageData, mime);
  const item = new ClipboardItem({
    'text/plain': new Blob([text], { type: 'text/plain' }),
    'text/html': new Blob([html], { type: 'text/html' }),
    'image/png': pngBlob,
  });
  await navigator.clipboard.write([item]);
}

/** 코드 단축키 → 커맨드 ID 매핑 (Ctrl+K,? 형태) */
const chordMapK: Record<string, string> = {
  b: 'insert:bookmark',
  ㅠ: 'insert:bookmark', // 한글 IME 상태
  n: 'format:para-num-shape',
  ㅜ: 'format:para-num-shape', // 한글 IME 상태
};

/** 코드 단축키 → 커맨드 ID 매핑 (Ctrl+M,? 형태)
 *
 * 한컴 표준 영역 영역 Ctrl+N 영역 영역 chord 시작 영역 영역 Chrome 영역 영역 reserved shortcut
 * (새 창) 영역 영역 JS 차단 불가 영역 영역 Ctrl+M 영역 영역 변경 (PR #786 후속 정정).
 */
const chordMapM: Record<string, string> = {
  n: 'insert:footnote',
  ㅜ: 'insert:footnote', // 한글 IME
  s: 'page:hide',
  ㄴ: 'page:hide', // 한글 IME
  m: 'insert:equation',
  ㅡ: 'insert:equation', // 한글 IME
};

/** 코드 단축키 → 커맨드 ID 매핑 (Alt+V,? 형태 — 보기 메뉴) */
const chordMapV: Record<string, string> = {
  t: 'view:border-transparent',
  ㅅ: 'view:border-transparent', // 한글 IME
};

/** 코드 단축키 → 커맨드 ID 매핑 (Ctrl+G,? 형태 — 보기/조판 메뉴) */
const chordMapG: Record<string, string> = {
  c: 'view:ctrl-mark',        // 조판 부호
  ㅊ: 'view:ctrl-mark',       // 한글 IME
  t: 'view:para-mark',        // 문단 부호
  ㅅ: 'view:para-mark',       // 한글 IME
  p: 'view:zoom-fit-page',    // 쪽 맞춤
  ㅍ: 'view:zoom-fit-page',   // 한글 IME
  w: 'view:zoom-fit-width',   // 폭 맞춤
  ㅈ: 'view:zoom-fit-width',  // 한글 IME
  q: 'view:zoom-100',         // 100%
  ㅂ: 'view:zoom-100',        // 한글 IME
};

/**
 * 키보드 이벤트 처리 순서:
 *

 * 1. 코드 단축키 2번째 키 (Ctrl+K → ? / Ctrl+M → ?)
 * 2. 특수 모드 탈출 (연결선/다각형/이미지/글상자 배치 모드 → Escape)
 * 3. IME 조합 중 네비게이션 키 보류
 * 4. 편집 모드별 키 처리 (머리말꼬리말 / 각주)
 * 5. F5 셀 선택 모드
 * 6. 셀 선택 모드 키 처리
 * 7. 그림/표 객체 선택 모드 키 처리
 * 8. 플랫폼별 navigation shortcut 처리
 * 9. Ctrl/Meta 조합 → handleCtrlKey() → shortcut-map.ts 단축키 테이블 경유
 * 10. Alt 조합 → shortcut-map.ts 단축키 테이블 경유
 * 11. 본문 키 처리 (Esc, Backspace, Enter, Arrow 등)
 *
 * 새 단축키 추가 시: shortcut-map.ts의 defaultShortcuts 테이블에 등록
 */
export function onKeyDown(this: any, e: KeyboardEvent): void {
  if (!this.active) return;

  // ─── 1. 코드 단축키 2번째 키 처리 (Ctrl+K → ? / Ctrl+M → ?) ───
  if (this._pendingChordK) {
    this._pendingChordK = false;
    const key = e.key.toLowerCase();
    const cmdId = chordMapK[key];
    if (cmdId && this.dispatcher) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }
  if (this._pendingChordM) {
    this._pendingChordM = false;
    const key = e.key.toLowerCase();
    const cmdId = chordMapM[key];
    if (cmdId && this.dispatcher) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }
  if (this._pendingChordV) {
    this._pendingChordV = false;
    const key = e.key.toLowerCase();
    const cmdId = chordMapV[key];
    if (cmdId && this.dispatcher) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }
  if (this._pendingChordG) {
    this._pendingChordG = false;
    const key = e.key.toLowerCase();
    const cmdId = chordMapG[key];
    if (cmdId && this.dispatcher) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }

  // 연결선 드로잉 모드
  if (this.connectorDrawingMode) {
    if (e.key === 'Escape') {
      e.preventDefault();
      _connector.exitConnectorDrawingMode.call(this);
      return;
    }
    return; // 다른 키 무시
  }

  // 다각형 그리기 모드
  if (this.polygonDrawingMode) {
    if (e.key === 'Escape') {
      e.preventDefault();
      if (this.polygonPoints.length >= 2) {
        this.finishPolygonDrawing(); // 현재까지 그린 다각형 확정
      } else {
        this.cancelPolygonDrawing();
      }
      return;
    }
    if (e.key === 'Backspace') {
      e.preventDefault();
      this.polygonPoints.pop();
      if (this.polygonPoints.length === 0) {
        this.cancelPolygonDrawing();
      } else {
        const last = this.polygonPoints[this.polygonPoints.length - 1];
        this.updatePolygonOverlay(this.polygonMousePos?.x ?? last.x, this.polygonMousePos?.y ?? last.y);
      }
      return;
    }
    return; // 다른 키 무시
  }

  // 그림 배치 모드에서 Escape → 취소
  if (this.imagePlacementMode && e.key === 'Escape') {
    e.preventDefault();
    this.cancelImagePlacement();
    return;
  }

  // 글상자 배치 모드에서 Escape → 취소
  if (this.textboxPlacementMode && e.key === 'Escape') {
    e.preventDefault();
    this.cancelTextboxPlacement();
    return;
  }

  // IME 조합 중 처리 (한국어 IME에서 e.key는 항상 'Process'이므로 e.code로 판별)
  if (e.isComposing || e.keyCode === 229) {
    // [PR #786 후속] Ctrl+M chord 1번째/2번째 키 영역 영역 IME 합성 중 영역 영역도 활성화.
    // 한글 IME 영역 영역 e.key === 'Process' 영역 영역, e.code (KeyM/KeyN/KeyS/KeyF/KeyK 등) 영역 영역 판별.
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === 'KeyM') {
      e.preventDefault();
      this._pendingChordM = true;
      return;
    }
    // chord 2번째 키 — _pendingChordM 활성화 시 e.code 영역 영역 chordMapM lookup
    if (this._pendingChordM) {
      this._pendingChordM = false;
      const codeToKey: Record<string, string> = {
        KeyM: 'm', KeyN: 'n', KeyS: 's', KeyF: 'f', KeyK: 'k',
      };
      const key = codeToKey[e.code];
      if (key && this.dispatcher) {
        const cmdId = chordMapM[key];
        if (cmdId) {
          e.preventDefault();
          this.dispatcher.dispatch(cmdId);
          return;
        }
      }
    }
    const navCodes = ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown',
                      'Home', 'End', 'Escape', 'Enter', 'Tab',
                      'PageUp', 'PageDown'];
    if (navCodes.includes(e.code)) {
      // 브라우저가 조합을 자연스럽게 종료하도록 두고,
      // compositionEnd 후 탐색 키를 처리하도록 예약
      this._pendingNavAfterIME = {
        key: e.key, code: e.code, shiftKey: e.shiftKey,
        ctrlKey: e.ctrlKey, metaKey: e.metaKey, altKey: e.altKey,
      };
    }
    return;
  }

  // ─── 머리말/꼬리말 편집 모드 키보드 처리 ──────────────────
  if (this.cursor.isInHeaderFooter()) {
    // Shift+Esc 또는 Esc → 편집 모드 탈출
    if (e.key === 'Escape') {
      e.preventDefault();
      // 현재 보고 있는 페이지 기억
      const hfPage = this.cursor.rect?.pageIndex ?? 0;
      this.cursor.exitHeaderFooterMode();
      this.eventBus.emit('headerFooterModeChanged', 'none');
      // 해당 페이지의 본문 첫 문단 시작점으로 커서 이동
      try {
        const pageInfo = this.wasm.getPageInfo(hfPage);
        const bodyX = pageInfo.marginLeft + 1;
        const bodyY = pageInfo.marginTop + pageInfo.marginHeader + 1;
        const hit = this.wasm.hitTest(hfPage, bodyX, bodyY);
        if (hit.paragraphIndex < 0xFFFFFF00) {
          this.cursor.moveTo(hit);
        }
      } catch { /* hitTest 실패 시 기존 위치 유지 */ }
      this.afterEdit();
      this.textarea?.focus();
      return;
    }

    // 방향키 → 머리말/꼬리말 내 이동
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const delta = e.key === 'ArrowLeft' ? -1 : 1;
      this.cursor.moveHorizontalInHf(delta);
      this.updateCaret();
      return;
    }

    // Shift+Enter → 머리말/꼬리말 내 강제 줄바꿈
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      const isHeader = this.cursor.headerFooterMode === 'header';
      try {
        this.wasm.insertTextInHeaderFooter(
          this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
          this.cursor.hfParaIdx, this.cursor.hfCharOffset, '\n',
        );
        this.cursor.setHfCursorPosition(this.cursor.hfParaIdx, this.cursor.hfCharOffset + 1);
        this.afterEdit();
      } catch { /* ignore */ }
      return;
    }

    // Enter → 머리말/꼬리말 내 문단 분할
    if (e.key === 'Enter') {
      e.preventDefault();
      const isHeader = this.cursor.headerFooterMode === 'header';
      try {
        const result = JSON.parse(this.wasm.splitParagraphInHeaderFooter(
          this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
          this.cursor.hfParaIdx, this.cursor.hfCharOffset,
        ));
        this.cursor.setHfCursorPosition(result.hfParaIndex, 0);
        this.afterEdit();
      } catch { /* ignore */ }
      return;
    }

    // Backspace / Delete는 handleBackspace/handleDelete에서 처리
    if (e.key === 'Backspace' || e.key === 'Delete') {
      e.preventDefault();
      const pos = this.cursor.getPosition();
      if (e.key === 'Backspace') {
        this.handleBackspace(pos, false);
      } else {
        this.handleDelete(pos, false);
      }
      return;
    }

    // 기타 키 (문자 입력)는 기본 처리로 전달 (textarea의 input 이벤트로 처리)
    return;
  }

  // ─── 각주 편집 모드 키보드 처리 ──────────────────────────
  if (this.cursor.isInFootnote()) {
    // Shift+Esc 또는 Escape → 주석 편집 모드 탈출
    if (e.key === 'Escape') {
      e.preventDefault();
      this.cursor.exitFootnoteMode();
      this.eventBus.emit('footnoteModeChanged', false);
      this.afterEdit();
      this.textarea?.focus();
      return;
    }

    // 방향키 → 각주 내 이동
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const delta = e.key === 'ArrowLeft' ? -1 : 1;
      this.cursor.moveHorizontalInFn(delta);
      this.updateCaret();
      return;
    }

    // Enter → 각주 내 문단 분할
    if (e.key === 'Enter') {
      e.preventDefault();
      try {
        const result = this.wasm.splitParagraphInFootnote(
          this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
          this.cursor.fnInnerParaIdx, this.cursor.fnCharOffset,
        );
        this.cursor.setFnCursorPosition(result.fnParaIndex, 0);
        this.afterEdit();
      } catch { /* ignore */ }
      return;
    }

    // Backspace / Delete
    if (e.key === 'Backspace' || e.key === 'Delete') {
      e.preventDefault();
      if (e.key === 'Backspace') {
        if (this.cursor.fnCharOffset > 0) {
          try {
            this.wasm.deleteTextInFootnote(
              this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
              this.cursor.fnInnerParaIdx, this.cursor.fnCharOffset - 1, 1,
            );
            this.cursor.setFnCursorPosition(this.cursor.fnInnerParaIdx, this.cursor.fnCharOffset - 1);
            this.afterEdit();
          } catch { /* ignore */ }
        } else if (this.cursor.fnInnerParaIdx > 0) {
          // 문단 시작에서 Backspace → 이전 문단과 병합
          try {
            const result = this.wasm.mergeParagraphInFootnote(
              this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
              this.cursor.fnInnerParaIdx,
            );
            this.cursor.setFnCursorPosition(result.fnParaIndex, result.charOffset);
            this.afterEdit();
          } catch { /* ignore */ }
        }
      } else {
        // Delete
        try {
          this.wasm.deleteTextInFootnote(
            this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
            this.cursor.fnInnerParaIdx, this.cursor.fnCharOffset, 1,
          );
          this.afterEdit();
        } catch { /* ignore */ }
      }
      return;
    }

    // 기타 키 (문자 입력)는 textarea의 input 이벤트로 처리
    return;
  }

  // ─── F5 블록 선택 모드 진입/해제 ────────────────────────────────
  if (e.key === 'F5') {
    e.preventDefault();
    if (this.cursor.isInCell() && !this.cursor.isInTextBox()) {
      if (this.cursor.isInCellSelectionMode()) {
        this.cursor.advanceCellSelectionPhase();
        this.updateCellSelection();
      } else {
        if (this.cursor.enterCellSelectionMode()) {
          this.caret.hide();
          this.selectionRenderer.clear();
          this.updateCellSelection();
        }
      }
    } else {
      // 본문 블록 선택 모드 (#220)
      if (this.cursor.isInBlockSelectionMode()) {
        this.cursor.exitBlockSelectionMode();
        this.selectionRenderer.clear();
        this.updateCaret();
      } else {
        this.cursor.enterBlockSelectionMode();
        this.updateSelection();
      }
    }
    return;
  }

  // ─── F3 선택 영역 확장 (#220) ──────────────────────────
  if (e.key === 'F3') {
    e.preventDefault();
    if (!this.cursor.isInBlockSelectionMode()) {
      this.cursor.enterBlockSelectionMode();
    }
    this.cursor.expandSelection();
    this.updateSelection();
    return;
  }

  // ─── 그림/글상자 객체 선택 모드 중 키 처리 ──────────────────────────
  if (this.cursor.isInPictureObjectSelection()) {
    if (e.key === 'Escape') {
      e.preventDefault();
      this.cursor.moveOutOfSelectedPicture();
      this.pictureObjectRenderer?.clear();
      this.eventBus.emit('picture-object-selection-changed', false);
      this.updateCaret();
      return;
    }
    // Enter → 글상자 내부 텍스트 편집 진입
    if (e.key === 'Enter') {
      const ref = this.cursor.getSelectedPictureRef();
      if (ref && ref.type === 'shape') {
        e.preventDefault();
        this.cursor.exitPictureObjectSelection();
        this.pictureObjectRenderer?.clear();
        this.eventBus.emit('picture-object-selection-changed', false);
        this.enterTextboxEditing(ref.sec, ref.ppi, ref.ci);
        return;
      }
    }
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        this.cursor.moveOutOfSelectedPicture();
        this.pictureObjectRenderer?.clear();
        this.eventBus.emit('picture-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'deleteObject', operation: (wasm: WasmBridge) => {
          deleteSelectedObject(wasm, ref);
          return this.cursor.getPosition();
        }});
      }
      return;
    }
    // Ctrl+C → 개체 복사 (clipboard 이벤트가 textarea에서 발생하지 않으므로 직접 처리)
    if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
      e.preventDefault();
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        try {
          const cellPathJson = pictureCellPathJson(ref);
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci, cellPathJson);
          const text = this.wasm.getClipboardText() || '[그림]';
          let html = '';
          try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci, cellPathJson) || ''; } catch { /* 무시 */ }
          const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
          if (ref.type === 'image') {
            writeImageToClipboard(this.wasm, ref.sec, ref.ppi, ref.ci, text, markedHtml, cellPathJson)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          } else {
            writeTextHtmlToClipboard(text, markedHtml)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          }
        } catch (err) {
          console.warn('[InputHandler] 개체 복사 실패:', err);
        }
      }
      return;
    }
    // Ctrl+X → 개체 잘라내기
    if ((e.ctrlKey || e.metaKey) && e.key === 'x') {
      e.preventDefault();
      const ref = this.cursor.getSelectedPictureRef();
      if (ref) {
        try {
          const cellPathJson = pictureCellPathJson(ref);
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci, cellPathJson);
          const text = this.wasm.getClipboardText() || '[그림]';
          let html = '';
          try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci, cellPathJson) || ''; } catch { /* 무시 */ }
          const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
          if (ref.type === 'image') {
            writeImageToClipboard(this.wasm, ref.sec, ref.ppi, ref.ci, text, markedHtml, cellPathJson)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          } else {
            writeTextHtmlToClipboard(text, markedHtml)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          }
        } catch (err) {
          console.warn('[InputHandler] 개체 복사 실패:', err);
        }
        this.cursor.moveOutOfSelectedPicture();
        this.pictureObjectRenderer?.clear();
        this.eventBus.emit('picture-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'cutObject', operation: (wasm: WasmBridge) => {
          deleteSelectedObject(wasm, ref);
          return this.cursor.getPosition();
        }});
      }
      return;
    }
    // Ctrl+V → 개체 선택 해제 후 붙여넣기 (paste 이벤트로 처리)
    if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
      this.cursor.moveOutOfSelectedPicture();
      this.pictureObjectRenderer?.clear();
      this.eventBus.emit('picture-object-selection-changed', false);
      // paste 이벤트에서 처리되도록 폴스루 (preventDefault 하지 않음)
      return;
    }
    // 방향키 → 개체 위치 이동, Shift+방향키 → 개체 크기 조절 (#1231 한컴 정합)
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown' ||
        e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const arrow = e.key as 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight';
      if (e.shiftKey) {
        this.resizeSelectedPicture(arrow);
      } else {
        this.moveSelectedPicture(arrow);
      }
      return;
    }
    // Shift/Ctrl/Alt/Meta 키만 누름 → 무시
    if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) return;
    // 기타 키 → 개체 선택 해제 후 일반 처리로 폴스루
    this.exitPictureObjectSelectionIfNeeded();
  }

  // ─── 표 객체 선택 모드 중 키 처리 ──────────────────────────
  if (this.cursor.isInTableObjectSelection()) {
    if (e.key === 'Escape') {
      e.preventDefault();
      // 표 객체 선택 → 표 밖으로 커서 이동
      this.cursor.moveOutOfSelectedTable();
      this.eventBus.emit('table-object-selection-changed', false);
      this.updateCaret();
      // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
      // this.checkTransparentBordersTransition();
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      // 표 객체 선택 → 셀 편집 복귀
      this.cursor.exitTableObjectSelection();
      this.eventBus.emit('table-object-selection-changed', false);
      this.updateCaret();
      return;
    }
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      // 표 객체 선택 → 표 삭제
      const ref = this.cursor.getSelectedTableRef();
      if (ref) {
        if (ref.cellPath && ref.cellPath.length > 1) {
          // 중첩 표 삭제는 미지원 — 선택만 해제
          this.cursor.moveOutOfSelectedTable();
          this.eventBus.emit('table-object-selection-changed', false);
          this.updateCaret();
          // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
          // this.checkTransparentBordersTransition();
        } else {
          this.cursor.moveOutOfSelectedTable();
          this.eventBus.emit('table-object-selection-changed', false);
          this.executeOperation({ kind: 'snapshot', operationType: 'deleteTable', operation: (wasm: WasmBridge) => {
            wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
            return this.cursor.getPosition();
          }});
          // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
          // this.checkTransparentBordersTransition();
        }
      }
      return;
    }
    // Ctrl+C → 표 복사
    if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
      e.preventDefault();
      const ref = this.cursor.getSelectedTableRef();
      if (ref) {
        try {
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci);
          const text = this.wasm.getClipboardText();
          if (text) {
            let html = '';
            try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci) || ''; } catch { /* 무시 */ }
            const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
            writeTextHtmlToClipboard(text, markedHtml)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          }
        } catch (err) {
          console.warn('[InputHandler] 표 복사 실패:', err);
        }
      }
      return;
    }
    // Ctrl+X → 표 잘라내기
    if ((e.ctrlKey || e.metaKey) && e.key === 'x') {
      e.preventDefault();
      const ref = this.cursor.getSelectedTableRef();
      if (ref && !(ref.cellPath && ref.cellPath.length > 1)) {
        try {
          this.wasm.copyControl(ref.sec, ref.ppi, ref.ci);
          const text = this.wasm.getClipboardText();
          if (text) {
            let html = '';
            try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci) || ''; } catch { /* 무시 */ }
            const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
            writeTextHtmlToClipboard(text, markedHtml)
              .catch(() => navigator.clipboard.writeText(text).catch(() => {}));
          }
        } catch (err) {
          console.warn('[InputHandler] 표 복사 실패:', err);
        }
        this.cursor.moveOutOfSelectedTable();
        this.eventBus.emit('table-object-selection-changed', false);
        this.executeOperation({ kind: 'snapshot', operationType: 'cutTable', operation: (wasm: WasmBridge) => {
          wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
          return this.cursor.getPosition();
        }});
        // [Task #394] 셀 진입 자동 ON 로직 비활성화 — input-handler.ts 의 코멘트 참고.
        // this.checkTransparentBordersTransition();
      }
      return;
    }
    // Ctrl+V → 표 선택 해제 후 붙여넣기 (paste 이벤트로 위임)
    if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
      this.cursor.moveOutOfSelectedTable();
      this.eventBus.emit('table-object-selection-changed', false);
      return;
    }
    // 방향키 → 표 위치 이동
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown' ||
        e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      this.moveSelectedTable(e.key as 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight');
      return;
    }
    // 수정자 키만 누른 경우 무시
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    // 그 외 키 → 표 객체 선택 해제 후 기본 키 처리
    this.cursor.exitTableObjectSelection();
    this.eventBus.emit('table-object-selection-changed', false);
    // fall through
  }

  // ─── 본문 블록 선택 모드 해제 (#220) ──────────────────────
  if (this.cursor.isInBlockSelectionMode() && e.key === 'Escape') {
    e.preventDefault();
    this.cursor.exitBlockSelectionMode();
    this.selectionRenderer.clear();
    this.updateCaret();
    return;
  }

  // ─── 셀 선택 모드 중 키 처리 ────────────────────────────
  if (this.cursor.isInCellSelectionMode()) {
    if (e.key === 'Escape') {
      e.preventDefault();
      // 셀 선택 모드 → 표 객체 선택 모드
      this.cursor.exitCellSelectionMode();
      this.cellSelectionRenderer?.clear();
      if (this.cursor.enterTableObjectSelection()) {
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderTableObjectSelection();
        this.eventBus.emit('table-object-selection-changed', true);
      } else {
        this.updateCaret();
      }
      return;
    }
    // Ctrl+방향키: 셀 크기 조절
    if ((e.ctrlKey || e.metaKey) && (
        e.key === 'ArrowUp' || e.key === 'ArrowDown' ||
        e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
      e.preventDefault();
      const phase = this.cursor.getCellSelectionPhase();
      if (phase === 3) {
        // phase 3: 전체 표 비율 리사이즈 (모든 셀에 동일 delta)
        this.resizeTableProportional(e.key as 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight');
      } else {
        // phase 1, 2: 선택 셀 크기 조절 (이웃 셀 반대 delta)
        this.resizeCellByKeyboard(e.key as 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight');
      }
      return;
    }
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown' ||
        e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const dr = e.key === 'ArrowUp' ? -1 : e.key === 'ArrowDown' ? 1 : 0;
      const dc = e.key === 'ArrowLeft' ? -1 : e.key === 'ArrowRight' ? 1 : 0;
      const phase = this.cursor.getCellSelectionPhase();
      if (phase === 2) {
        // phase 2: 범위 확장 (anchor 고정, focus만 이동)
        this.cursor.expandCellSelection(dr, dc);
      } else if (phase === 3) {
        // phase 3: 전체 선택 상태에서 방향키 → 무시 (Ctrl+방향키는 위에서 리사이즈 처리)
      } else {
        // phase 1: 단일 셀 이동
        this.cursor.moveCellSelection(dr, dc);
      }
      this.updateCellSelection();
      return;
    }
    // M: 셀 합치기, S: 셀 나누기
    if (e.key === 'm' || e.key === 'M') {
      e.preventDefault();
      this.dispatcher?.dispatch('table:cell-merge');
      return;
    }
    if (e.key === 's' || e.key === 'S') {
      e.preventDefault();
      this.dispatcher?.dispatch('table:cell-split');
      return;
    }
    // 수정자 키(Shift/Ctrl/Alt/Meta)만 누른 경우 무시
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') {
      return;
    }
    // 그 외 키 → 셀 선택 모드 종료 후 기존 처리로 넘김
    this.cursor.exitCellSelectionMode();
    this.cellSelectionRenderer?.clear();
    this.updateCaret();
    // fall through: 아래 기존 키 처리 계속 진행
  }

  if (handleNavigationShortcut.call(this, e)) return;

  // Ctrl/Meta 조합 처리 (Ctrl+Enter, Ctrl+C 등 모두 shortcut-map.ts에서 정의)
  if (e.ctrlKey || e.metaKey) {
    this.handleCtrlKey(e);
    return;
  }

  // Alt 조합 단축키 처리
  // - Alt+Backspace → 이전 단어 삭제 (아래 Backspace/Delete case)
  // - Alt+Delete → 표 안 영역 영역 'table:delete-col' (기존 동작 보존),
  //                표 외 영역 영역 다음 단어 삭제 (아래 Backspace/Delete case)
  const isAltWordKey = e.altKey && (
    e.key === 'Backspace' ||
    (e.key === 'Delete' && !this.cursor.isInCell())
  );
  if (e.altKey && !isAltWordKey && this.dispatcher) {
    // Alt+V → Chord 대기 (보기 메뉴 단축키, 한컴 Alt+V,T 계승)
    if ((e.key === 'v' || e.key === 'V' || e.key === 'ㅍ') && !e.shiftKey && !e.ctrlKey) {
      e.preventDefault();
      this._pendingChordV = true;
      return;
    }
    const cmdId = matchShortcut(e, defaultShortcuts);
    if (cmdId) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }

  // ─── Esc: 가장 안쪽 컨테이너부터 escape ──
  //  - 글상자 안 표 셀 → 표 객체 선택 (안 표가 가장 안쪽)
  //  - 글상자 안 (안 표 외 위치) → 글상자 객체 선택
  //  - 본문 표 셀 → 표 객체 선택
  //  한컴 UX 정합 (`feedback_visual_judgment_authority`).
  if (e.key === 'Escape') {
    e.preventDefault();
    const inCell = this.cursor.isInCell();
    const inTextBox = this.cursor.isInTextBox();
    const nestingDepth = this.cursor.nestingDepth();
    // [Task #919] 글상자 안 표 셀 (cellPath.length >= 2 + isTextBox) → 표 객체 선택.
    // enterTableObjectSelection 이 가장 안쪽 셀 (innermost) 의 표를 선택.
    const inNestedTableInTextBox = inTextBox && nestingDepth >= 2;
    if (inNestedTableInTextBox) {
      // 글상자 안 표 → 표 객체 선택 (가장 안쪽)
      const entered = this.cursor.enterTableObjectSelection();
      if (entered) {
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderTableObjectSelection();
        this.eventBus.emit('table-object-selection-changed', true);
      }
    } else if (inTextBox) {
      // 글상자 편집 (안 표 외 영역) → 글상자 객체 선택
      const pos = this.cursor.getPosition();
      const sec = pos.sectionIndex;
      const ppi = pos.parentParaIndex!;
      const ci = pos.controlIndex!;
      // 컨트롤 타입 판별: getPictureProperties 성공 → image, 아니면 shape
      let objType: 'image' | 'shape' = 'shape';
      try { this.wasm.getPictureProperties(sec, ppi, ci); objType = 'image'; } catch { /* shape */ }
      this.cursor.clearSelection();
      this.cursor.enterPictureObjectSelectionDirect(sec, ppi, ci, objType);
      this.caret.hide();
      this.selectionRenderer.clear();
      this.renderPictureObjectSelection();
      this.eventBus.emit('picture-object-selection-changed', true);
    } else if (inCell) {
      // 본문 셀 편집 모드 → 표 객체 선택
      const entered = this.cursor.enterTableObjectSelection();
      if (entered) {
        this.caret.hide();
        this.selectionRenderer.clear();
        this.renderTableObjectSelection();
        this.eventBus.emit('table-object-selection-changed', true);
      }
    }
    return;
  }

  // F11은 onF11Intercept(capture)에서 handleF11()로 직접 호출됨

  const pos = this.cursor.getPosition();
  const inCell = this.cursor.isInCell();

  switch (e.key) {
    case 'Backspace':
    case 'Delete': {
      e.preventDefault();
      if (this.isFormMode?.() && e.altKey) return;
      if (this.cursor.hasSelection()) {
        this.deleteSelection();
      } else if (e.altKey) {
        // Alt/Option+Backspace/Delete: 단어 삭제 (macOS standard)
        this.cursor.setAnchor();
        this.cursor.moveToWordBoundary(e.key === 'Backspace' ? -1 : 1);
        if (this.cursor.hasSelection()) this.deleteSelection();
      } else if (e.key === 'Backspace') {
        this.handleBackspace(pos, inCell);
      } else {
        this.handleDelete(pos, inCell);
      }
      break;
    }
    case 'Enter': {
      e.preventDefault();
      if (this.isFormMode?.()) return;
      if (this.cursor.hasSelection()) this.deleteSelection();
      if (e.shiftKey) {
        // Shift+Enter: 강제 줄바꿈 (문단 유지, 줄만 바꿈)
        this.executeOperation({ kind: 'command', command: new InsertLineBreakCommand(this.cursor.getPosition()) });
      } else if (inCell) {
        this.executeOperation({ kind: 'command', command: new SplitParagraphInCellCommand(this.cursor.getPosition()) });
      } else {
        this.executeOperation({ kind: 'command', command: new SplitParagraphCommand(this.cursor.getPosition()) });
      }
      break;
    }
    case 'ArrowLeft':
    case 'ArrowRight':
    case 'ArrowUp':
    case 'ArrowDown': {
      e.preventDefault();
      const vertical = this.cursor.isInVerticalCell();
      // 세로쓰기 셀: ↑↓=글자이동(horizontal), ←→=줄이동(vertical)
      // 가로쓰기:    ←→=글자이동(horizontal), ↑↓=줄이동(vertical)
      let moveH: number | null = null;
      let moveV: number | null = null;
      if (e.key === 'ArrowLeft') {
        if (vertical) moveV = -1; else moveH = -1;
      } else if (e.key === 'ArrowRight') {
        if (vertical) moveV = 1; else moveH = 1;
      } else if (e.key === 'ArrowUp') {
        if (vertical) moveH = -1; else moveV = -1;
      } else { // ArrowDown
        if (vertical) moveH = 1; else moveV = 1;
      }
      if (e.shiftKey) {
        this.cursor.setAnchor();
      } else {
        this.cursor.clearSelection();
      }
      if (!e.shiftKey && moveH === 1 && this.tryEnterExitedFieldStart?.()) {
        this.updateCaret();
        break;
      }
      if (!e.shiftKey && moveH === -1 && this.tryEnterExitedFieldEnd?.()) {
        this.updateCaret();
        break;
      }
      if (!e.shiftKey && moveH === -1 && this.tryExitCurrentFieldStart?.()) {
        this.updateCaret();
        break;
      }
      if (!e.shiftKey && moveH === 1 && this.tryExitCurrentFieldEnd?.()) {
        this.updateCaret();
        break;
      }
      if (moveH !== null) this.cursor.moveHorizontal(moveH);
      if (moveV !== null) this.cursor.moveVertical(moveV);
      this.updateCaret();
      if (e.shiftKey) this.updateSelection();
      break;
    }
    case 'PageUp':
    case 'PageDown': {
      e.preventDefault();
      const vpSize = this.viewportManager.getViewportSize();
      const scrollY = this.viewportManager.getScrollY();
      const vpCenter = scrollY + vpSize.height / 2;
      const currentPage = this.virtualScroll.getPageAtY(vpCenter);
      const targetPage = e.key === 'PageUp'
        ? Math.max(0, currentPage - 1)
        : Math.min(this.virtualScroll.pageCount - 1, currentPage + 1);
      if (targetPage !== currentPage) {
        const targetOffset = this.virtualScroll.getPageOffset(targetPage);
        this.viewportManager.setScrollTop(targetOffset - this.virtualScroll.gap);
      }
      break;
    }
    case 'Home': {
      e.preventDefault();
      if (e.shiftKey) {
        this.cursor.setAnchor();
        this.cursor.moveToLineStart();
      } else {
        this.cursor.clearSelection();
        this.cursor.moveToLineStart();
      }
      this.markCurrentFieldStartOutside?.();
      this.updateCaret();
      if (e.shiftKey) this.updateSelection();
      break;
    }
    case 'End': {
      e.preventDefault();
      if (e.shiftKey) {
        this.cursor.setAnchor();
        this.cursor.moveToLineEnd();
      } else {
        this.cursor.clearSelection();
        this.cursor.moveToLineEnd();
      }
      this.markCurrentFieldEndOutside?.();
      this.updateCaret();
      if (e.shiftKey) this.updateSelection();
      break;
    }
    case 'Tab': {
      e.preventDefault();
      if (this.isFormMode?.()) {
        this.moveToAdjacentFormField?.(e.shiftKey ? -1 : 1);
        return;
      }
      if (e.shiftKey) {
        this.applyHangingIndentAtCursor();
        break;
      }
      // 탭 문자 삽입 (본문·표 셀·글상자 공통)
      this.executeOperation({ kind: 'command', command: new InsertTabCommand(this.cursor.getPosition()) });
      break;
    }
    case 'Insert': {
      e.preventDefault();
      this.insertMode = !this.insertMode;
      this.eventBus.emit('insert-mode-changed', this.insertMode);
      break;
    }
    default: {
      // Function 키(F1~F12) 등 Ctrl 없는 단축키 처리
      if (this.dispatcher) {
        const cmdId = matchShortcut(e, defaultShortcuts);
        if (cmdId) {
          e.preventDefault();
          this.dispatcher.dispatch(cmdId);
        }
      }
      break;
    }
  }
}

export function handleCtrlKey(this: any, e: KeyboardEvent): void {
  // Ctrl+/ → 커맨드 팔레트 열기
  if (e.key === '/' && !e.shiftKey && !e.altKey) {
    e.preventDefault();
    this.commandPalette?.open();
    return;
  }

  // 커맨드 시스템 경유 단축키 처리
  if (this.dispatcher) {
    const cmdId = matchShortcut(e, defaultShortcuts);
    if (cmdId) {
      e.preventDefault();
      this.dispatcher.dispatch(cmdId);
      return;
    }
  }

  // ─── 코드 단축키 1번째 키 (Ctrl+K / Ctrl+M) ───
  if ((e.key === 'k' || e.key === 'K' || e.key === 'ㅏ') && !e.shiftKey && !e.altKey) {
    e.preventDefault();
    this._pendingChordK = true;
    return;
  }
  // [PR #786 후속] Ctrl+N 영역 영역 Chrome reserved shortcut (새 창) 영역 영역 JS 차단 불가
  // 영역 영역 Ctrl+M 영역 영역 chord 1번째 키 영역 영역 변경.
  if ((e.key === 'm' || e.key === 'M' || e.key === 'ㅡ') && !e.shiftKey && !e.altKey) {
    e.preventDefault();
    this._pendingChordM = true;
    return;
  }
  if ((e.key === 'g' || e.key === 'G' || e.key === 'ㅎ') && !e.shiftKey && !e.altKey) {
    e.preventDefault();
    this._pendingChordG = true;
    return;
  }

  // 커맨드 시스템에 없는 직접 처리 (Ctrl/Cmd+Backspace, Ctrl+Home/End, Ctrl/Cmd+Arrow 등)
  switch (e.key.toLowerCase()) {
    case 'backspace': {
      e.preventDefault();
      if (this.isFormMode?.()) return;
      if (this.cursor.hasSelection()) {
        this.deleteSelection();
      } else if (e.metaKey && !e.ctrlKey) {
        // Cmd+Backspace (macOS): 줄 시작까지 삭제
        this.cursor.setAnchor();
        this.cursor.moveToLineStart();
        if (this.cursor.hasSelection()) this.deleteSelection();
      } else {
        // Ctrl+Backspace (Win/Linux): 이전 단어 경계까지 삭제
        this.cursor.setAnchor();
        this.cursor.moveToWordBoundary(-1);
        if (this.cursor.hasSelection()) this.deleteSelection();
      }
      break;
    }
    case 'delete': {
      if (!e.ctrlKey) break;
      e.preventDefault();
      if (this.isFormMode?.()) return;
      if (this.cursor.hasSelection()) {
        this.deleteSelection();
      } else {
        // Ctrl+Delete (Win/Linux): 다음 단어 경계까지 삭제
        this.cursor.setAnchor();
        this.cursor.moveToWordBoundary(1);
        if (this.cursor.hasSelection()) this.deleteSelection();
      }
      break;
    }
    case 'home': {
      e.preventDefault();
      if (e.shiftKey) {
        this.cursor.setAnchor();
        this.cursor.moveToDocumentStart();
      } else {
        this.cursor.clearSelection();
        this.cursor.moveToDocumentStart();
      }
      this.updateCaret();
      break;
    }
    case 'end': {
      e.preventDefault();
      if (e.shiftKey) {
        this.cursor.setAnchor();
        this.cursor.moveToDocumentEnd();
      } else {
        this.cursor.clearSelection();
        this.cursor.moveToDocumentEnd();
      }
      this.updateCaret();
      break;
    }
    case 'arrowleft': {
      e.preventDefault();
      if (e.shiftKey) this.cursor.setAnchor();
      else this.cursor.clearSelection();
      this.cursor.moveToLineStart();
      this.updateCaret();
      break;
    }
    case 'arrowright': {
      e.preventDefault();
      if (e.shiftKey) this.cursor.setAnchor();
      else this.cursor.clearSelection();
      this.cursor.moveToLineEnd();
      this.updateCaret();
      break;
    }
    case 'arrowup': {
      e.preventDefault();
      if (e.shiftKey) this.cursor.setAnchor();
      else this.cursor.clearSelection();
      // [Issue #784 후속] macOS Cmd+↑ = 문서 시작 (macOS 표준).
      // Windows/Linux Ctrl+↑ = 이전 문단 (한컴 표준).
      if (e.metaKey && !e.ctrlKey) {
        this.cursor.moveToDocumentStart();
      } else {
        this.cursor.moveToParagraphBoundary(-1);
      }
      this.updateCaret();
      break;
    }
    case 'arrowdown': {
      e.preventDefault();
      if (e.shiftKey) this.cursor.setAnchor();
      else this.cursor.clearSelection();
      // [Issue #784 후속] macOS Cmd+↓ = 문서 끝 (macOS 표준).
      // Windows/Linux Ctrl+↓ = 다음 문단 (한컴 표준).
      if (e.metaKey && !e.ctrlKey) {
        this.cursor.moveToDocumentEnd();
      } else {
        this.cursor.moveToParagraphBoundary(1);
      }
      this.updateCaret();
      break;
    }
    // 그 외 Ctrl 조합 (줌 등)은 브라우저 기본 동작 허용
  }
}

export function handleSelectAll(this: any): void {
  // anchor를 문서 시작, focus를 문서 끝으로 설정
  this.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
  this.cursor.setAnchor();
  this.cursor.moveToDocumentEnd();
  this.updateCaret();
}

export function onCopy(this: any, e: ClipboardEvent): void {
  if (!this.active) return;

  // 개체(글상자/그림) 선택 모드 → 개체 복사
  if (this.cursor.isInPictureObjectSelection()) {
    const ref = this.cursor.getSelectedPictureRef();
    if (ref) {
      e.preventDefault();
      try {
        const cellPathJson = pictureCellPathJson(ref);
        this.wasm.copyControl(ref.sec, ref.ppi, ref.ci, cellPathJson);
        const text = this.wasm.getClipboardText() || '[그림]';
        let html = '';
        try { html = this.wasm.exportControlHtml(ref.sec, ref.ppi, ref.ci, cellPathJson) || ''; } catch { /* HTML 내보내기 실패는 fallback */ }
        const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
        if (e.clipboardData) {
          if (text) e.clipboardData.setData('text/plain', text);
          e.clipboardData.setData('text/html', markedHtml);
        }
        // 이미지 컨트롤이면 image/png Blob 포함 클립보드 기록
        if (ref.type === 'image') {
          writeImageToClipboard(this.wasm, ref.sec, ref.ppi, ref.ci, text, markedHtml, cellPathJson)
            .catch(() => {});
        }
      } catch (err) {
        console.warn('[InputHandler] 개체 복사 실패:', err);
      }
    }
    return;
  }

  if (!this.cursor.hasSelection()) return;
  e.preventDefault();

  const sel = this.cursor.getSelectionOrdered();
  if (!sel) return;
  const { start, end } = sel;

  try {
    // WASM 내부 클립보드에 복사 (서식 보존)
    if (start.parentParaIndex !== undefined) {
      this.wasm.copySelectionInCell(
        start.sectionIndex, start.parentParaIndex, start.controlIndex!, start.cellIndex!,
        start.cellParaIndex!, start.charOffset,
        end.cellParaIndex!, end.charOffset,
      );
    } else {
      this.wasm.copySelection(
        start.sectionIndex,
        start.paragraphIndex, start.charOffset,
        end.paragraphIndex, end.charOffset,
      );
    }

    // 시스템 클립보드에 플레인 텍스트 + HTML 설정
    const text = this.wasm.getClipboardText();
    if (e.clipboardData) {
      if (text) e.clipboardData.setData('text/plain', text);
      // HTML 내보내기 (표/서식 보존)
      let html = '';
      try {
        if (start.parentParaIndex !== undefined) {
          html = this.wasm.exportSelectionInCellHtml(
            start.sectionIndex, start.parentParaIndex, start.controlIndex!, start.cellIndex!,
            start.cellParaIndex!, start.charOffset,
            end.cellParaIndex!, end.charOffset,
          );
        } else {
          html = this.wasm.exportSelectionHtml(
            start.sectionIndex,
            start.paragraphIndex, start.charOffset,
            end.paragraphIndex, end.charOffset,
          );
        }
      } catch { /* HTML 내보내기 실패는 fallback HTML 사용 */ }
      const markedHtml = prepareRhwpInternalClipboardHtml(this, html, text);
      e.clipboardData.setData('text/html', markedHtml);
    }
  } catch (err) {
    console.warn('[InputHandler] 복사 실패:', err);
  }
}

export function onCut(this: any, e: ClipboardEvent): void {
  if (!this.active) return;
  if (this.isFormMode?.()) {
    e.preventDefault();
    return;
  }

  // 개체 선택 모드 → 개체 잘라내기 (복사 후 삭제)
  if (this.cursor.isInPictureObjectSelection()) {
    const ref = this.cursor.getSelectedPictureRef();
    if (ref) {
      this.onCopy(e); // 클립보드에 복사
      this.cursor.moveOutOfSelectedPicture();
      this.pictureObjectRenderer?.clear();
      this.eventBus.emit('picture-object-selection-changed', false);
      this.executeOperation({ kind: 'snapshot', operationType: 'cutObject', operation: (wasm: WasmBridge) => {
        deleteSelectedObject(wasm, ref);
        return this.cursor.getPosition();
      }});
    }
    return;
  }

  if (!this.cursor.hasSelection()) return;
  // 먼저 복사
  this.onCopy(e);
  // 선택 영역 삭제
  this.deleteSelection();
}

export function onPaste(this: any, e: ClipboardEvent): void {
  if (!this.active) return;
  e.preventDefault();
  if (this.isFormMode?.()) return;

  // 개체/표 선택 모드 해제 후 붙여넣기 진행
  if (this.cursor.isInPictureObjectSelection()) {
    this.cursor.moveOutOfSelectedPicture();
    this.pictureObjectRenderer?.clear();
    this.eventBus.emit('picture-object-selection-changed', false);
  }
  if (this.cursor.isInTableObjectSelection()) {
    this.cursor.moveOutOfSelectedTable();
    this.eventBus.emit('table-object-selection-changed', false);
  }

  // 선택 영역 삭제 여부 캡처 (스냅샷 내부에서 처리)
  const hasSelection = this.cursor.hasSelection();

  const pos = this.cursor.getPosition();
  const clipboardData = e.clipboardData;
  const html = clipboardData?.getData('text/html') || '';
  const text = clipboardData?.getData('text/plain') || '';
  const hasCurrentInternalMarker = hasCurrentRhwpClipboardMarker(this, html);

  // 현재 rhwp-studio 내부 복사 marker가 있을 때만 내부 클립보드 사용 (서식 보존)
  if (this.wasm.hasInternalClipboard() && (!clipboardData || hasCurrentInternalMarker)) {
    // 컨트롤(개체) 붙여넣기 — 본문에서만 허용
    if (this.wasm.clipboardHasControl() && pos.parentParaIndex === undefined) {
      this.executeOperation({ kind: 'snapshot', operationType: 'pasteControl', operation: (wasm: WasmBridge) => {
        if (hasSelection) this.deleteSelection();
        const p = this.cursor.getPosition();
        const result = wasm.pasteControl(p.sectionIndex, p.paragraphIndex, p.charOffset);
        const parsed = JSON.parse(result);
        if (parsed.ok) {
          const newParaIdx = (parsed.paraIdx ?? p.paragraphIndex) + 1;
          return {
            sectionIndex: p.sectionIndex,
            paragraphIndex: newParaIdx,
            charOffset: 0,
          } as DocumentPosition;
        }
        return p;
      }});
      return;
    }

    // 내부 클립보드 텍스트 붙여넣기 (서식 보존)
    this.executeOperation({ kind: 'snapshot', operationType: 'pasteInternal', operation: (wasm: WasmBridge) => {
      this.pastedFieldEndOutsidePending = false;
      if (hasSelection) this.deleteSelection();
      const p = this.cursor.getPosition();
      let result: string;
      if (isNestedCellPosition(p)) {
        result = wasm.pasteInternalInCellByPath(
          p.sectionIndex, p.parentParaIndex!, JSON.stringify(p.cellPath), p.charOffset,
        );
      } else if (p.parentParaIndex !== undefined) {
        result = wasm.pasteInternalInCell(
          p.sectionIndex, p.parentParaIndex, p.controlIndex!,
          p.cellIndex!, p.cellParaIndex!, p.charOffset,
        );
      } else {
        result = wasm.pasteInternal(p.sectionIndex, p.paragraphIndex, p.charOffset);
      }
      const parsed = JSON.parse(result);
      if (parsed.ok) {
        if (parsed.containsField === true) {
          this.pastedFieldEndOutsidePending = true;
        }
        return positionAfterPasteResult(p, parsed);
      }
      return p;
    }});
    return;
  }

  // 외부 클립보드: 이미지 파일이 있으면 그림으로 삽입
  const items = clipboardData?.items;
  if (items) {
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.kind === 'file' && item.type.startsWith('image/')) {
        const file = item.getAsFile();
        if (file) {
          pasteImageFile.call(this, file, hasSelection);
          return;
        }
      }
    }
  }

  // 외부 클립보드: HTML이 있으면 pasteHtml로 표/서식 보존 붙여넣기
  if (html) {
    this.executeOperation({ kind: 'snapshot', operationType: 'pasteHtml', operation: (wasm: WasmBridge) => {
      if (hasSelection) this.deleteSelection();
      const p = this.cursor.getPosition();
      let result: string;
      if (isNestedCellPosition(p)) {
        result = wasm.pasteHtmlInCellByPath(
          p.sectionIndex, p.parentParaIndex!, JSON.stringify(p.cellPath), p.charOffset, html,
        );
      } else if (p.parentParaIndex !== undefined) {
        result = wasm.pasteHtmlInCell(
          p.sectionIndex, p.parentParaIndex, p.controlIndex!,
          p.cellIndex!, p.cellParaIndex!, p.charOffset, html,
        );
      } else {
        result = wasm.pasteHtml(p.sectionIndex, p.paragraphIndex, p.charOffset, html);
      }
      const parsed = JSON.parse(result);
      if (parsed.ok) {
        return positionAfterPasteResult(p, parsed);
      }
      return p;
    }});
    return;
  }

  // 플레인 텍스트 붙여넣기 (fallback — 기존 InsertTextCommand 사용, 정밀 undo 유지)
  pastePlainText.call(this, text, hasSelection);
}

/** 클립보드의 이미지 파일을 커서 위치에 삽입한다. */
async function pasteImageFile(this: any, file: File, hasSelection: boolean): Promise<void> {
  try {
    const data = new Uint8Array(await file.arrayBuffer());
    const ext = (file.type.split('/')[1] || 'png').replace('jpeg', 'jpg');

    // 이미지 크기 측정
    const img = new Image();
    const url = URL.createObjectURL(file);
    try {
      img.src = url;
      await img.decode();
    } finally {
      URL.revokeObjectURL(url);
    }

    // px → HWPUNIT (1px = 75 HWPUNIT at 96 DPI)
    let wHwp = Math.round(img.naturalWidth * 75);
    let hHwp = Math.round(img.naturalHeight * 75);

    // 열 폭 초과 시 비례 축소
    const pos = this.cursor.getPosition();
    try {
      const pageDef = this.wasm.getPageDef(pos.sectionIndex);
      const colWidth = pageDef.width - pageDef.marginLeft - pageDef.marginRight;
      if (wHwp > colWidth) {
        const ratio = colWidth / wHwp;
        wHwp = Math.round(colWidth);
        hHwp = Math.round(hHwp * ratio);
      }
    } catch { /* 페이지 정보 없으면 그대로 */ }

    const natW = img.naturalWidth;
    const natH = img.naturalHeight;

    // 스냅샷으로 삽입 (Undo 지원)
    this.executeOperation({ kind: 'snapshot', operationType: 'pasteImage', operation: (wasm: WasmBridge) => {
      if (hasSelection) this.deleteSelection();
      const p = this.cursor.getPosition();
      // 표 셀 안 paste (#1151): floating picture 분기 — parentParaIndex + cellPath 전달.
      const inCell = (p.cellPath?.length ?? 0) > 0 && p.parentParaIndex !== undefined;
      const paraForCall = inCell ? p.parentParaIndex! : p.paragraphIndex;
      const cellPathJson = inCell ? JSON.stringify(p.cellPath) : '';
      const result = wasm.insertPicture(
        p.sectionIndex, paraForCall, p.charOffset,
        cellPathJson, data, wHwp, hHwp, natW, natH, ext, '',
      );
      if (result.ok) {
        return {
          sectionIndex: p.sectionIndex,
          paragraphIndex: result.paraIdx + 1,
          charOffset: 0,
        } as DocumentPosition;
      }
      return p;
    }});
  } catch (err) {
    console.warn('[InputHandler] 클립보드 이미지 삽입 실패:', err);
  }
}

/** 기존 컨트롤 선택 상태를 모두 해제한다 */
function clearAllControlSelection(self: any): void {
  if (self.cursor.isInTableObjectSelection()) {
    self.cursor.exitTableObjectSelection();
    self.tableObjectRenderer?.clear();
  }
  if (self.cursor.isInPictureObjectSelection()) {
    self.cursor.exitPictureObjectSelection();
    self.pictureObjectRenderer?.clear();
  }
  if (self.cursor.hasSelection()) {
    self.cursor.clearSelection();
  }
}

/** F11: 이전 방향 가장 가까운 컨트롤 선택 */
export function handleF11(this: any): void {
  try {
    // 현재 선택 상태에 따라 검색 시작점 결정
    // - 필드 텍스트 선택 중: anchor(선택 시작점)에서 검색 → 같은 필드 재선택 방지
    // - 표/그림 객체 선택 중: 선택된 컨트롤 위치에서 검색
    // - 그 외: 현재 커서 위치
    let searchSec: number, searchPara: number, searchCharOffset: number;

    if (this.cursor.isInTableObjectSelection()) {
      const ref = this.cursor.getSelectedTableRef();
      searchSec = ref!.sec; searchPara = ref!.ppi; searchCharOffset = 0;
    } else if (this.cursor.isInPictureObjectSelection()) {
      const ref = this.cursor.getSelectedPictureRef();
      searchSec = ref!.sec; searchPara = ref!.ppi;
      // 선택된 도형의 텍스트 위치를 검색 시작점으로 사용
      const ctrlPositions = this.wasm.getControlTextPositions?.(ref!.sec, ref!.ppi);
      searchCharOffset = ctrlPositions?.[ref!.ci] ?? 0;
    } else if (this.cursor.hasSelection()) {
      const sel = this.cursor.getSelection()!;
      searchSec = sel.anchor.sectionIndex;
      searchPara = sel.anchor.paragraphIndex;
      searchCharOffset = sel.anchor.charOffset;
    } else {
      const pos = this.cursor.getPosition();
      searchSec = pos.sectionIndex; searchPara = pos.paragraphIndex; searchCharOffset = pos.charOffset;
    }

    const result = this.wasm.findNearestControlBackward(searchSec, searchPara, searchCharOffset);

    if (result.type === 'none') {
      // 더 이상 이전 컨트롤 없음 → 현재 선택 해제
      // 선택 해제 후 커서를 원래 검색 위치에 두어 다시 F11 시 재선택 가능
      const hadSelection = this.cursor.isInTableObjectSelection()
        || this.cursor.isInPictureObjectSelection()
        || this.cursor.hasSelection();
      clearAllControlSelection(this);
      if (hadSelection) {
        // 커서를 검색 시작 위치의 다음 문단으로 이동 (컨트롤 문단 다음)
        const paraCount = this.wasm.getParagraphCount(searchSec);
        const nextPara = Math.min(searchPara + 1, paraCount - 1);
        try { this.cursor.moveTo({ sectionIndex: searchSec, paragraphIndex: nextPara, charOffset: 0 }); } catch {}
      }
      this.updateCaret();
      return;
    }

    // 새 컨트롤 선택 전 기존 선택 모두 해제
    clearAllControlSelection(this);

    if (result.type === 'table') {
      // 표 전용 문단은 커서를 놓을 수 없으므로 표 다음 문단으로 커서 이동
      const paraCount = this.wasm.getParagraphCount(result.sec);
      const cursorPara = Math.min(result.para + 1, paraCount - 1);
      try { this.cursor.moveTo({ sectionIndex: result.sec, paragraphIndex: cursorPara, charOffset: 0 }); } catch {}
      this.cursor.enterTableObjectSelectionDirect(result.sec, result.para, result.ci);
      this.updateCaret();
      this.renderTableObjectSelection();
    } else if (result.type === 'shape' || result.type === 'picture' || result.type === 'equation') {
      // 개체 문단도 커서 위치 오류 가능 → try/catch
      try { this.cursor.moveTo({ sectionIndex: result.sec, paragraphIndex: result.para, charOffset: 0 }); } catch {}
      // 도형 타입 세분화: 직선은 'line' (2점 핸들용)
      let ctrlType: string = result.type === 'picture' ? 'image'
        : result.type === 'equation' ? 'equation'
        : 'shape';
      if (ctrlType === 'shape') {
        // getPageControlLayout에서 line 타입 확인
        try {
          const pageCount = this.wasm.pageCount;
          for (let p = 0; p < pageCount; p++) {
            const layout = this.wasm.getPageControlLayout(p);
            for (const ctrl of layout.controls) {
              if (ctrl.type === 'line' && ctrl.secIdx === result.sec && ctrl.paraIdx === result.para && ctrl.controlIdx === result.ci) {
                ctrlType = 'line';
                break;
              }
            }
            if (ctrlType === 'line') break;
          }
        } catch { /* ignore */ }
      }
      this.cursor.enterPictureObjectSelectionDirect(
        result.sec, result.para, result.ci, ctrlType as any,
      );
      this.updateCaret();
      this.renderPictureObjectSelection();
    } else if (result.type === 'bookmark') {
      // 책갈피: 해당 위치로 커서 이동
      const charPos = result.charPos ?? 0;
      try { this.cursor.moveTo({ sectionIndex: result.sec, paragraphIndex: result.para, charOffset: charPos }); } catch {}
      this.updateCaret();
      // 책갈피 대화상자를 열어 수정/삭제 가능하게
      this.dispatcher?.dispatch('insert:bookmark');
    } else if (result.type === 'field') {
      // 누름틀: 필드 텍스트 블록 선택 (charPos = 필드의 텍스트 내 위치)
      const fieldPos = { sectionIndex: result.sec, paragraphIndex: result.para, charOffset: result.charPos ?? 0 };
      const fi = this.wasm.getFieldInfoAt(fieldPos);
      if (fi.inField && fi.startCharIdx !== undefined && fi.endCharIdx !== undefined) {
        const startPos = { sectionIndex: result.sec, paragraphIndex: result.para, charOffset: fi.startCharIdx };
        const endPos = { sectionIndex: result.sec, paragraphIndex: result.para, charOffset: fi.endCharIdx };
        // anchor를 끝에, focus(커서)를 시작에 → 캐럿이 입력시작위치에 표시
        this.cursor.moveTo(endPos);
        this.cursor.setAnchor();
        this.cursor.moveTo(startPos);
        this.updateCaret();
        this.eventBus.emit('field-info-changed', {
          fieldId: fi.fieldId, fieldType: fi.fieldType, guideName: fi.guideName,
        });
      }
    }
  } catch (err) {
    console.warn('[F11] error:', err);
  }
}

/** Shift+F11: 순방향(→) 가장 가까운 컨트롤 선택 */
export function handleShiftF11(this: any): void {
  try {
    let searchSec: number, searchPara: number, searchCharOffset: number;

    if (this.cursor.isInTableObjectSelection()) {
      const ref = this.cursor.getSelectedTableRef();
      searchSec = ref!.sec; searchPara = ref!.ppi;
      const ctrlPositions = this.wasm.getControlTextPositions?.(ref!.sec, ref!.ppi);
      searchCharOffset = ctrlPositions?.[ref!.ci] ?? 0;
    } else if (this.cursor.isInPictureObjectSelection()) {
      const ref = this.cursor.getSelectedPictureRef();
      searchSec = ref!.sec; searchPara = ref!.ppi;
      const ctrlPositions = this.wasm.getControlTextPositions?.(ref!.sec, ref!.ppi);
      searchCharOffset = ctrlPositions?.[ref!.ci] ?? 0;
    } else {
      const pos = this.cursor.getPosition();
      searchSec = pos.sectionIndex; searchPara = pos.paragraphIndex; searchCharOffset = pos.charOffset;
    }

    const result = this.wasm.findNearestControlForward(searchSec, searchPara, searchCharOffset);

    if (result.type === 'none') {
      const hadSelection = this.cursor.isInTableObjectSelection()
        || this.cursor.isInPictureObjectSelection();
      clearAllControlSelection(this);
      if (hadSelection) {
        const paraCount = this.wasm.getParagraphCount(searchSec);
        const nextPara = Math.min(searchPara + 1, paraCount - 1);
        try { this.cursor.moveTo({ sectionIndex: searchSec, paragraphIndex: nextPara, charOffset: 0 }); } catch {}
      }
      this.updateCaret();
      return;
    }

    clearAllControlSelection(this);

    if (result.type === 'table') {
      const paraCount = this.wasm.getParagraphCount(result.sec);
      const cursorPara = Math.min(result.para + 1, paraCount - 1);
      try { this.cursor.moveTo({ sectionIndex: result.sec, paragraphIndex: cursorPara, charOffset: 0 }); } catch {}
      this.cursor.enterTableObjectSelectionDirect(result.sec, result.para, result.ci);
      this.updateCaret();
      this.renderTableObjectSelection();
    } else if (result.type === 'shape' || result.type === 'picture' || result.type === 'equation') {
      try { this.cursor.moveTo({ sectionIndex: result.sec, paragraphIndex: result.para, charOffset: 0 }); } catch {}
      let ctrlType: string = result.type === 'picture' ? 'image'
        : result.type === 'equation' ? 'equation'
        : 'shape';
      if (ctrlType === 'shape') {
        try {
          const pageCount = this.wasm.pageCount;
          for (let p = 0; p < pageCount; p++) {
            const layout = this.wasm.getPageControlLayout(p);
            for (const ctrl of layout.controls) {
              if (ctrl.type === 'line' && ctrl.secIdx === result.sec && ctrl.paraIdx === result.para && ctrl.controlIdx === result.ci) {
                ctrlType = 'line';
                break;
              }
            }
            if (ctrlType === 'line') break;
          }
        } catch { /* ignore */ }
      }
      this.cursor.enterPictureObjectSelectionDirect(
        result.sec, result.para, result.ci, ctrlType as any,
      );
      this.updateCaret();
      this.renderPictureObjectSelection();
    }
  } catch (err) {
    console.warn('[Shift+F11] error:', err);
  }
}
