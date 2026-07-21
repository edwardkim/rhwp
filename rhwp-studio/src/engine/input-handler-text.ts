/** input-handler text methods — extracted from InputHandler class */
/* eslint-disable @typescript-eslint/no-explicit-any */

import {
  InsertTextCommand,
  DeleteTextCommand,
  MergeParagraphCommand,
  MergeNextParagraphCommand,
  MergeParagraphInCellCommand,
  MergeNextParagraphInCellCommand,
  InsertTextInHeaderFooterCommand,
  DeleteTextInHeaderFooterCommand,
  MergeParagraphInHeaderFooterCommand,
  InsertTextInFootnoteCommand,
  insertTextWithMutationEffects,
  cellParaIndexOf,
  NO_TEXT_MUTATION_EFFECTS,
} from './command';
import type { TextMutationEffects } from './command';
import type { DocumentPosition } from '@/core/types';
import { showConfirm } from '@/ui/confirm-dialog';
import {
  detectPlatformKind,
  getNavigationAction,
  shouldSuppressUnmappedNavigation,
  type NavigationAction,
  type NavigationKeyInput,
} from './navigation-keymap';

/**
 * [#2548] WASM 삭제/조회 count 는 Rust `Paragraph::delete_text_at` 의 char(Unicode
 * scalar) 단위다. JS `String.length`(UTF-16 code unit)를 넘기면 astral 문자(😀 등)에서
 * 실제보다 많이 지워 인접 문자를 잃는다 — [#2337-review] 가 undo/HF/FN 경로에 적용한
 * 계약을 IME 조합 경로에도 맞춘다.
 *
 * 주의: *커서 오프셋* 은 studio 의 UTF-16 관례를 유지한다(command.ts `charCount` 주석,
 * tests/undo-delete-char-count.test.ts 참조). 여기서는 삭제/조회 count 에만 쓴다.
 */
function charCount(s: string): number {
  return [...s].length;
}

const FOOTNOTE_DELETE_TITLE = '각주 삭제';
const FOOTNOTE_DELETE_MESSAGE = '각주를 삭제하시겠습니까?';

function tryConfirmRemoveClickHereAtBoundary(
  this: any,
  pos: DocumentPosition,
  direction: 'backward' | 'forward',
): boolean {
  if (this.isFormMode?.()) return false;
  try {
    const fi = this.wasm.getFieldInfoAt(pos);
    if (!fi.inField || fi.fieldType !== 'clickhere') return false;
    const start = fi.startCharIdx ?? -1;
    const end = fi.endCharIdx ?? -1;
    if (start < 0 || end < 0) return false;

    const atBoundary = direction === 'forward'
      ? pos.charOffset >= end
      : pos.charOffset <= start || (pos.charOffset >= end && this.isAtExitedFieldEnd?.(pos, fi));
    if (!atBoundary) return false;

    return this.confirmRemoveCurrentField?.() ?? true;
  } catch {
    return false;
  }
}

/** IME 조합 종료 후 대기 중인 탐색 키를 처리한다 */
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

function processPendingNav(this: any, nav: NavigationKeyInput): void {
  const { code, shiftKey } = nav;
  const platform = detectPlatformKind();
  const action = getNavigationAction(nav, platform);
  if (action) {
    executeNavigationAction.call(this, action, shiftKey);
    return;
  }
  if (shouldSuppressUnmappedNavigation(nav, platform)) return;

  // 방향키 처리
  if (code === 'ArrowLeft' || code === 'ArrowRight' ||
      code === 'ArrowUp' || code === 'ArrowDown') {
    const vertical = this.cursor.isInVerticalCell?.() ?? false;
    if (shiftKey) {
      this.cursor.setAnchor();
    } else {
      this.cursor.clearSelection();
    }
    let moveH: number | null = null;
    let moveV: number | null = null;
    if (code === 'ArrowLeft') {
      if (vertical) moveV = -1; else moveH = -1;
    } else if (code === 'ArrowRight') {
      if (vertical) moveV = 1; else moveH = 1;
    } else if (code === 'ArrowUp') {
      if (vertical) moveH = -1; else moveV = -1;
    } else {
      if (vertical) moveH = 1; else moveV = 1;
    }
    if (!shiftKey && moveH === 1 && this.tryEnterExitedFieldStart?.()) {
      this.updateCaret();
      return;
    }
    if (!shiftKey && moveH === -1 && this.tryEnterExitedFieldEnd?.()) {
      this.updateCaret();
      return;
    }
    if (!shiftKey && moveH === -1 && this.tryExitCurrentFieldStart?.()) {
      this.updateCaret();
      return;
    }
    if (!shiftKey && moveH === 1 && this.tryExitCurrentFieldEnd?.()) {
      this.updateCaret();
      return;
    }
    if (moveH !== null) this.cursor.moveHorizontal(moveH);
    if (moveV !== null) this.cursor.moveVertical(moveV);
    this.updateCaret();
  } else if (code === 'Home') {
    if (shiftKey) this.cursor.setAnchor(); else this.cursor.clearSelection();
    this.cursor.moveToLineStart();
    this.markCurrentFieldStartOutside?.();
    this.updateCaret();
  } else if (code === 'End') {
    if (shiftKey) this.cursor.setAnchor(); else this.cursor.clearSelection();
    this.cursor.moveToLineEnd();
    this.markCurrentFieldEndOutside?.();
    this.updateCaret();
  } else if (code === 'Enter') {
    // Enter는 조합 확정만으로 충분 (줄바꿈은 별도 처리 불필요)
  }
}

function tryDeleteBodyFootnoteAtCursor(
  this: any,
  pos: DocumentPosition,
  direction: 'backward' | 'forward',
): boolean {
  if (pos.parentParaIndex !== undefined || pos.cellPath || pos.isTextBox) return false;

  try {
    const hit = this.wasm.getFootnoteAtCursor(
      pos.sectionIndex,
      pos.paragraphIndex,
      pos.charOffset,
      direction,
    );
    if (!hit.hit || hit.controlIndex === undefined) return false;

    const sectionIndex = hit.sectionIndex ?? pos.sectionIndex;
    const paragraphIndex = hit.paragraphIndex ?? pos.paragraphIndex;
    const controlIndex = hit.controlIndex;

    void showConfirm(FOOTNOTE_DELETE_TITLE, FOOTNOTE_DELETE_MESSAGE)
      .then((ok) => {
        if (!ok) {
          this.textarea?.focus();
          return;
        }
        this.executeOperation({
          kind: 'snapshot',
          operationType: 'deleteFootnote',
          operation: (wasm: any) => {
            const result = wasm.deleteFootnote(sectionIndex, paragraphIndex, controlIndex);
            return {
              sectionIndex: result.sectionIndex,
              paragraphIndex: result.paragraphIndex,
              charOffset: result.charOffset,
            };
          },
        });
        this.textarea?.focus();
      })
      .catch(() => {
        this.textarea?.focus();
      });
    return true;
  } catch {
    return false;
  }
}

export function handleBackspace(this: any, pos: DocumentPosition, inCell: boolean): void {
  if (this.isFormMode?.() && !this.canEditCurrentFormField?.()) return;
  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    const hfOff = this.cursor.hfCharOffset;
    const target = { sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo };
    const paraIdx = this.cursor.hfParaIdx;
    if (hfOff > 0) {
      // [Task #2337] 삭제 텍스트를 WASM 반환에서 확보해 역연산(재삽입) 기록. Backspace 이므로
      // undo 후 커서는 hfOff(삭제 전 위치)로 복귀.
      const res = JSON.parse(this.wasm.deleteTextInHeaderFooter(target.sectionIdx, isHeader, target.applyTo, paraIdx, hfOff - 1, 1));
      this.executeOperation({ kind: 'record', command: new DeleteTextInHeaderFooterCommand(target, paraIdx, hfOff - 1, res.deletedText ?? '', hfOff) });
      this.cursor.setHfCursorPosition(paraIdx, hfOff - 1);
      this.afterEdit();
    } else if (paraIdx > 0) {
      // 문단 시작에서 Backspace → 이전 문단과 병합
      const result = JSON.parse(this.wasm.mergeParagraphInHeaderFooter(target.sectionIdx, isHeader, target.applyTo, paraIdx));
      // Backspace 병합: 병합 전 커서는 (paraIdx, 0).
      this.executeOperation({ kind: 'record', command: new MergeParagraphInHeaderFooterCommand(target, paraIdx, result.hfParaIndex, result.charOffset, paraIdx, 0) });
      this.cursor.setHfCursorPosition(result.hfParaIndex, result.charOffset);
      this.afterEdit();
    }
    return;
  }

  const { charOffset } = pos;

  // 필드 경계 보호: 필드 시작 위치에서는 Backspace 차단
  try {
    const fi = this.wasm.getFieldInfoAt(pos);
    if (fi.inField && this.isAtExitedFieldStart?.(pos, fi)) {
      // 누름틀 시작 바깥에서는 Backspace가 앞쪽 본문 글자를 지운다.
    } else if (fi.inField && charOffset <= fi.startCharIdx) {
      if (tryConfirmRemoveClickHereAtBoundary.call(this, pos, 'backward')) return;
      return;
    }
    if (fi.inField && this.isAtExitedFieldEnd?.(pos, fi)) {
      if (tryConfirmRemoveClickHereAtBoundary.call(this, pos, 'backward')) return;
    }
  } catch { /* 무시 */ }

  if (inCell) {
    if (charOffset > 0) {
      const deletePos = { ...pos, charOffset: charOffset - 1 };
      this.executeOperation({ kind: 'command', command: new DeleteTextCommand(deletePos, 1, 'backward') });
    } else if (cellParaIndexOf(pos) > 0) {
      // 셀 문단 시작에서 Backspace → 이전 셀 문단과 병합.
      // [#2717] 중첩 셀에서 flat `pos.cellParaIndex` 는 hit-test 가 cellPath[0](최외곽)로 채운
      // 바깥 셀의 문단 인덱스라, 그대로 쓰면 안쪽 셀 2번째 문단에서 병합이 통째로 누락되고
      // (바깥이 0), 안쪽 첫 문단에서는 cellParaIndex:-1 경로로 병합이 실행된다(바깥이 ≥1).
      // 아래 handleDelete(:307 useCellPath) 와 동일하게 안쪽 축으로 판정한다.
      this.executeOperation({ kind: 'command', command: new MergeParagraphInCellCommand(pos) });
    }
  } else {
    const { sectionIndex: sec, paragraphIndex: para } = pos;
    if (tryDeleteBodyFootnoteAtCursor.call(this, pos, 'backward')) return;
    if (charOffset > 0) {
      const deletePos = { ...pos, charOffset: charOffset - 1 };
      this.executeOperation({ kind: 'command', command: new DeleteTextCommand(deletePos, 1, 'backward') });
    } else if (para > 0) {
      // 문단 시작에서 Backspace → 이전 문단과 병합
      this.executeOperation({ kind: 'command', command: new MergeParagraphCommand({ sectionIndex: sec, paragraphIndex: para, charOffset: 0 }) });
    }
  }
}

export function handleDelete(this: any, pos: DocumentPosition, inCell: boolean): void {
  if (this.isFormMode?.() && !this.canEditCurrentFormField?.()) return;
  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    const target = { sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo };
    try {
      const paraIdx = this.cursor.hfParaIdx;
      const info = JSON.parse(this.wasm.getHeaderFooterParaInfo(target.sectionIdx, isHeader, target.applyTo, paraIdx));
      const hfOff = this.cursor.hfCharOffset;
      if (hfOff < info.charCount) {
        // Delete(forward): 커서는 hfOff 유지 → undo 후에도 hfOff.
        const res = JSON.parse(this.wasm.deleteTextInHeaderFooter(target.sectionIdx, isHeader, target.applyTo, paraIdx, hfOff, 1));
        this.executeOperation({ kind: 'record', command: new DeleteTextInHeaderFooterCommand(target, paraIdx, hfOff, res.deletedText ?? '', hfOff) });
        this.afterEdit();
      } else if (paraIdx + 1 < info.paraCount) {
        // 문단 끝에서 Delete → 다음 문단(paraIdx+1)을 현재 문단으로 병합. 병합 전 커서는 (paraIdx, 끝).
        const result = JSON.parse(this.wasm.mergeParagraphInHeaderFooter(target.sectionIdx, isHeader, target.applyTo, paraIdx + 1));
        this.executeOperation({ kind: 'record', command: new MergeParagraphInHeaderFooterCommand(target, paraIdx + 1, result.hfParaIndex, result.charOffset, result.hfParaIndex, result.charOffset) });
        this.cursor.setHfCursorPosition(result.hfParaIndex, result.charOffset);
        this.afterEdit();
      }
    } catch { /* ignore */ }
    return;
  }

  const { charOffset } = pos;

  // 필드 경계 보호: 필드 끝 위치에서는 Delete 차단
  try {
    const fi = this.wasm.getFieldInfoAt(pos);
    if (fi.inField && charOffset >= fi.endCharIdx) {
      if (tryConfirmRemoveClickHereAtBoundary.call(this, pos, 'forward')) return;
      return;
    }
  } catch { /* 무시 */ }

  if (inCell) {
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const ci = pos.controlIndex!;
    const cei = pos.cellIndex!;
    const useCellPath = (pos.cellPath?.length ?? 0) > 0;
    const cpi = useCellPath ? pos.cellPath![pos.cellPath!.length - 1].cellParaIndex : pos.cellParaIndex!;
    const pathJson = useCellPath ? JSON.stringify(pos.cellPath) : '';
    const paraLen = useCellPath
      ? this.wasm.getCellParagraphLengthByPath(sec, ppi, pathJson)
      : this.wasm.getCellParagraphLength(sec, ppi, ci, cei, cpi);
    if (charOffset < paraLen) {
      this.executeOperation({ kind: 'command', command: new DeleteTextCommand(pos, 1, 'forward') });
    } else {
      // 셀 문단 끝에서 Delete → 다음 셀 문단과 병합
      const paraCount = useCellPath
        ? this.wasm.getCellParagraphCountByPath(sec, ppi, pathJson)
        : this.wasm.getCellParagraphCount(sec, ppi, ci, cei);
      if (cpi + 1 < paraCount) {
        this.executeOperation({ kind: 'command', command: new MergeNextParagraphInCellCommand(pos) });
      }
    }
  } else {
    const { sectionIndex: sec, paragraphIndex: para } = pos;
    if (tryDeleteBodyFootnoteAtCursor.call(this, pos, 'forward')) return;
    const paraLen = this.wasm.getParagraphLength(sec, para);
    if (charOffset < paraLen) {
      this.executeOperation({ kind: 'command', command: new DeleteTextCommand(pos, 1, 'forward') });
    } else {
      // 문단 끝에서 Delete → 다음 문단과 병합
      const paraCount = this.wasm.getParagraphCount(sec);
      if (para + 1 < paraCount) {
        this.executeOperation({ kind: 'command', command: new MergeNextParagraphCommand(pos) });
      }
    }
  }
}

export function onCompositionStart(this: any): void {
  this.resetRawTextMutationEffects();
  // 선택 영역이 있으면 삭제 후 조합 시작
  if (this.cursor.hasSelection()) {
    if (!this.canDeleteSelectionInFormMode?.()) {
      this.textarea.value = '';
      return;
    }
    this.deleteSelection();
  }
  let basePos = this.cursor.isInHeaderFooter()
    ? { ...this.cursor.getPosition(), charOffset: this.cursor.hfCharOffset }
    : this.cursor.isInFootnote()
      ? { ...this.cursor.getPosition(), charOffset: this.cursor.fnCharOffset }
      : this.cursor.getPosition();
  if (!this.cursor.isInHeaderFooter() && !this.cursor.isInFootnote()) {
    basePos = this.prepareClickHereInputPosition?.() ?? basePos;
  }
  if (!this.canInsertTextInFormMode?.(basePos)) {
    this.textarea.value = '';
    this.isComposing = false;
    this.compositionAnchor = null;
    this.compositionLength = 0;
    return;
  }

  this.isComposing = true;
  if (this.cursor.isInHeaderFooter()) {
    // 머리말/꼬리말 모드에서는 hfCharOffset을 anchor의 charOffset으로 사용
    this.compositionAnchor = basePos;
  } else if (this.cursor.isInFootnote()) {
    // 각주 모드에서는 fnCharOffset을 anchor의 charOffset으로 사용
    this.compositionAnchor = basePos;
  } else {
    this.compositionAnchor = basePos;
  }
  this.compositionLength = 0;
}

export function onCompositionEnd(this: any): void {
  const anchor = this.compositionAnchor;
  const finalLength = this.compositionLength;

  this.isComposing = false;
  this.compositionAnchor = null;
  this.compositionLength = 0;
  this.textarea.value = '';
  this.caret.hideComposition();
  this.updateCaret();
  this.resetRawTextMutationEffects();

  // 더블 자음 분리 방지: compositionEnd 시점에 조합 완료된 텍스트 기억
  // 직후 유령 input 이벤트에서 동일 텍스트가 오면 무시
  this._lastComposedText = (finalLength > 0 && this._lastCompositionText) ? this._lastCompositionText : '';

  // 조합 중 WASM 직접 호출로 이미 문서에 삽입된 텍스트를
  // Command로 기록하여 Undo 가능하게 한다.
  // [Task #2337] 머리말/꼬리말·각주 모드도 이제 기록한다(본문 스냅샷 undo 의 무언 파괴 차단).
  if (anchor && finalLength > 0) {
    if (this.cursor.isInHeaderFooter()) {
      // HF 는 신뢰할 텍스트 read 가 없어 getTextAt(본문 리더)을 쓸 수 없으므로 조합 텍스트
      // (_lastCompositionText)를 그대로 기록한다. anchor.charOffset = 조합 시작 오프셋,
      // hfParaIdx 는 조합 중 불변.
      const composed = this._lastCompositionText || '';
      if (composed) {
        const target = { sectionIdx: this.cursor.hfSectionIdx, isHeader: this.cursor.headerFooterMode === 'header', applyTo: this.cursor.hfApplyTo };
        this.executeOperation({ kind: 'record', command: new InsertTextInHeaderFooterCommand(target, this.cursor.hfParaIdx, anchor.charOffset, composed) });
      }
    } else if (this.cursor.isInFootnote()) {
      const composed = this._lastCompositionText || '';
      if (composed) {
        const target = {
          sectionIdx: this.cursor.fnSectionIdx, paraIdx: this.cursor.fnParaIdx, controlIdx: this.cursor.fnControlIdx,
          footnoteIndex: this.cursor.fnFootnoteIndex, pageNum: this.cursor.fnPageNum,
        };
        this.executeOperation({ kind: 'record', command: new InsertTextInFootnoteCommand(target, this.cursor.fnInnerParaIdx, anchor.charOffset, composed) });
      }
    } else {
      const insertedText = this.getTextAt(anchor, finalLength);
      if (insertedText) {
        // execute() 없이 히스토리에만 기록 (텍스트는 이미 문서에 있음)
        this.executeOperation({ kind: 'record', command: new InsertTextCommand(anchor, insertedText) });
      }
    }
  }

  // 조합 종료 후 대기 중인 탐색 키 처리 (IME 조합 중 방향키 등)
  if (this._pendingNavAfterIME) {
    const nav = this._pendingNavAfterIME;
    this._pendingNavAfterIME = null;
    processPendingNav.call(this, nav);
  }
}

export function getTextAt(this: any, pos: DocumentPosition, count: number): string {
  try {
    if ((pos.cellPath?.length ?? 0) > 0 && pos.parentParaIndex !== undefined) {
      return this.wasm.getTextInCellByPath(pos.sectionIndex, pos.parentParaIndex, JSON.stringify(pos.cellPath), pos.charOffset, count);
    } else if (pos.parentParaIndex !== undefined) {
      return this.wasm.getTextInCell(pos.sectionIndex, pos.parentParaIndex, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, count);
    } else {
      return this.wasm.getTextRange(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, count);
    }
  } catch {
    return '';
  }
}

export function onInput(this: any, e?: InputEvent): void {
  if (!this.active) return;

  const text = this.textarea.value;
  // const inputType = e?.inputType ?? 'unknown';
  // const inputData = e?.data ?? '';
  // const isComp = e?.isComposing ?? false;

  // IME 조합 중: 이전 조합 텍스트 삭제 → 현재 조합 텍스트 삽입 (실시간 렌더링)
  // Undo 스택에는 기록하지 않음 (compositionend에서 한 번에 기록)
  if (this.isComposing && this.compositionAnchor) {
    const anchor = this.compositionAnchor;
    const beforePageIndex = this.cursor.getRect()?.pageIndex;
    if (!this.canInsertTextInFormMode?.(anchor)) {
      this.textarea.value = '';
      return;
    }
    this.resetRawTextMutationEffects();

    // 이전 조합 텍스트 삭제
    if (this.compositionLength > 0) {
      this.deleteTextAt(anchor, this.compositionLength);
    }

    // 현재 조합 텍스트 삽입
    if (text) {
      this.insertTextAtRaw(anchor, text);
      // 다음 조합 업데이트에서 deleteTextAt 의 삭제 count 로 쓰이므로 scalar 단위.
      this.compositionLength = charCount(text);
      this._lastCompositionText = text; // 더블 자음 분리 방지용
    } else {
      this.compositionLength = 0;
    }

    // cursor.moveTo() 내부의 exact lookup 전에 deferred mutation을 등록하고,
    // 실제 cell-flow 경계에서만 동기 flush한다.
    const boundaryHandled = this.consumeRawTextMutationBeforeCursor();
    const newOffset = anchor.charOffset + text.length;
    if (this.cursor.isInHeaderFooter()) {
      this.cursor.setHfCursorPosition(this.cursor.hfParaIdx, newOffset);
    } else if (this.cursor.isInFootnote()) {
      this.cursor.setFnCursorPosition(this.cursor.fnInnerParaIdx, newOffset);
    } else {
      this.cursor.moveTo({ ...anchor, charOffset: newOffset });
    }

    const afterPos = this.cursor.getPosition();
    const afterPageIndex = this.cursor.getRect()?.pageIndex;
    this.afterTextInputEdit(anchor, afterPos, {
      insertedText: text,
      beforePageIndex,
      afterPageIndex,
    }, boundaryHandled);
    return;
  }

  // iOS 폴백: composition 이벤트 없이 input만으로 한글 조합 처리
  // iOS contentEditable에서는 compositionStart/End가 발생하지 않는다.
  // div의 textContent를 건드리지 않고, 이전 상태와 비교하여 변경분만 처리.
  // iOS 폴백: iOS Safari/Chrome은 한글 조합을 compositionStart/End 없이
  // deleteContentBackward + insertText 쌍으로 처리한다.
  // div의 textContent(value)가 iOS에 의해 완벽하게 관리되므로,
  // 매 input마다 문서의 이전 삽입을 삭제하고 현재 value 전체로 교체한다.
  // 주의: afterEdit() 호출 시 document-changed 이벤트가 Canvas를 재렌더링하면서
  // div의 focus/textContent를 교란하므로, 렌더링은 디바운스하여 마지막에 한 번만 수행.
  if (this._isIOS && !this.isComposing) {
    // 앵커 설정 (첫 입력 시)
    if (!this._iosAnchor) {
      this._iosRequiresFullRefresh = false;
      this._iosBeforePageIndex = this.cursor.getRect()?.pageIndex;
      if (this.cursor.isInHeaderFooter()) {
        this._iosAnchor = { ...this.cursor.getPosition(), charOffset: this.cursor.hfCharOffset };
      } else if (this.cursor.isInFootnote()) {
        this._iosAnchor = { ...this.cursor.getPosition(), charOffset: this.cursor.fnCharOffset };
      } else {
        this._iosAnchor = this.prepareClickHereInputPosition?.() ?? this.cursor.getPosition();
      }
      this._iosLength = 0;
    }
    if (!this.canInsertTextInFormMode?.(this._iosAnchor)) {
      this.textarea.value = '';
      return;
    }
    this.resetRawTextMutationEffects();

    // 이전 삽입 전부 삭제
    if (this._iosLength > 0 && this._iosAnchor) {
      this.deleteTextAt(this._iosAnchor, this._iosLength);
    }

    // 현재 div 전체 텍스트를 문서에 삽입 (빈 값이면 삭제만)
    if (text) {
      this.insertTextAtRaw(this._iosAnchor, text);
      this._iosLength = charCount(text);
    } else {
      this._iosLength = 0;
    }

    const boundaryHandled = this.consumeRawTextMutationBeforeCursor();
    this._iosRequiresFullRefresh = this._iosRequiresFullRefresh || boundaryHandled;

    // 커서 이동 (렌더링 없이 문서만 갱신)
    const newOffset = this._iosAnchor.charOffset + (text?.length || 0);
    if (this.cursor.isInHeaderFooter()) {
      this.cursor.setHfCursorPosition(this.cursor.hfParaIdx, newOffset);
    } else if (this.cursor.isInFootnote()) {
      this.cursor.setFnCursorPosition(this.cursor.fnInnerParaIdx, newOffset);
    } else {
      this.cursor.moveTo({ ...this._iosAnchor, charOffset: newOffset });
    }

    // 렌더링 디바운스: 빠른 연속 입력 중에는 렌더링 생략,
    // 마지막 입력 후 100ms 뒤에 한 번만 렌더링
    clearTimeout(this._iosInputTimer);
    const iosAnchor = this._iosAnchor;
    const iosAfterPos = this.cursor.getPosition();
    const beforePageIndex = this._iosBeforePageIndex;
    const afterPageIndex = this.cursor.getRect()?.pageIndex;
    this._iosInputTimer = setTimeout(() => {
      const requiresFullRefresh = this._iosRequiresFullRefresh;
      this._iosRequiresFullRefresh = false;
      this.afterTextInputEdit(iosAnchor, iosAfterPos, {
        insertedText: text,
        beforePageIndex,
        afterPageIndex,
      }, requiresFullRefresh);
      // 렌더링 후 div 포커스 복원 (afterEdit가 포커스를 뺏을 수 있음)
      this.textarea.focus();
    }, 100);
    return;
  }

  // 일반 입력 (비조합) → Command로 실행
  if (!text) return;

  // 더블 자음 분리 방지: compositionEnd 직후 유령 input 이벤트 감지
  // 각주/머리말꼬리말 모드에서 조합 완료 직후 동일 텍스트가 오면 무시
  if (this._lastComposedText && text === this._lastComposedText) {
    this._lastComposedText = '';
    this.textarea.value = '';
    return;
  }
  this._lastComposedText = '';
  this.textarea.value = '';

  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    try {
      const target = { sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo };
      const paraIdx = this.cursor.hfParaIdx;
      const charOffset = this.cursor.hfCharOffset;
      this.wasm.insertTextInHeaderFooter(target.sectionIdx, isHeader, target.applyTo, paraIdx, charOffset, text);
      // [Task #2337] 히스토리 기록 → 본문 스냅샷 undo 가 이 편집을 무언 파괴하지 않게 한다.
      this.executeOperation({ kind: 'record', command: new InsertTextInHeaderFooterCommand(target, paraIdx, charOffset, text) });
      this.cursor.setHfCursorPosition(paraIdx, charOffset + text.length);
      this.afterEdit();
    } catch (err) {
      console.error('[HF-input] insertTextInHeaderFooter 실패:', err);
    }
    return;
  }

  // 각주 편집 모드
  if (this.cursor.isInFootnote()) {
    try {
      const target = {
        sectionIdx: this.cursor.fnSectionIdx, paraIdx: this.cursor.fnParaIdx, controlIdx: this.cursor.fnControlIdx,
        footnoteIndex: this.cursor.fnFootnoteIndex, pageNum: this.cursor.fnPageNum,
      };
      const innerParaIdx = this.cursor.fnInnerParaIdx;
      const charOffset = this.cursor.fnCharOffset;
      this.wasm.insertTextInFootnote(target.sectionIdx, target.paraIdx, target.controlIdx, innerParaIdx, charOffset, text);
      this.executeOperation({ kind: 'record', command: new InsertTextInFootnoteCommand(target, innerParaIdx, charOffset, text) });
      this.cursor.setFnCursorPosition(innerParaIdx, charOffset + text.length);
      this.afterEdit();
    } catch (err) {
      console.error('[FN-input] insertTextInFootnote 실패:', err);
    }
    return;
  }

  // 선택 영역이 있으면 먼저 삭제
  let insertPos = this.prepareClickHereInputPosition?.() ?? this.cursor.getPosition();
  let refreshClickHereGuide = this.isClickHereGuidePosition?.(insertPos) === true;
  if (this.cursor.hasSelection()) {
    if (!this.canDeleteSelectionInFormMode?.()) {
      this.textarea.value = '';
      return;
    }
    this.deleteSelection();
    insertPos = this.prepareClickHereInputPosition?.() ?? this.cursor.getPosition();
    refreshClickHereGuide = this.isClickHereGuidePosition?.(insertPos) === true;
  }
  if (!this.canInsertTextInFormMode?.(insertPos)) {
    this.textarea.value = '';
    return;
  }
  this.executeOperation({ kind: 'command', command: new InsertTextCommand(insertPos, text) });
  if (refreshClickHereGuide) {
    this.refreshClickHereAfterFirstInput?.();
  }
}

export function insertTextAtRaw(this: any, pos: DocumentPosition, text: string): TextMutationEffects {
  if (!this.canInsertTextInFormMode?.(pos)) return NO_TEXT_MUTATION_EFFECTS;
  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    this.wasm.insertTextInHeaderFooter(
      this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
      this.cursor.hfParaIdx, pos.charOffset, text,
    );
    return NO_TEXT_MUTATION_EFFECTS;
  }
  // 각주 편집 모드
  if (this.cursor.isInFootnote()) {
    this.wasm.insertTextInFootnote(
      this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
      this.cursor.fnInnerParaIdx, pos.charOffset, text,
    );
    return NO_TEXT_MUTATION_EFFECTS;
  }
  return insertTextWithMutationEffects(this.wasm, pos, text);
}

export function deleteTextAt(this: any, pos: DocumentPosition, count: number): void {
  if (!this.canDeleteTextInFormMode?.(pos, count)) return;
  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    this.wasm.deleteTextInHeaderFooter(
      this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
      this.cursor.hfParaIdx, pos.charOffset, count,
    );
    return;
  }
  // 각주 편집 모드
  if (this.cursor.isInFootnote()) {
    this.wasm.deleteTextInFootnote(
      this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
      this.cursor.fnInnerParaIdx, pos.charOffset, count,
    );
    return;
  }
  if ((pos.cellPath?.length ?? 0) > 0 && pos.parentParaIndex !== undefined) {
    this.wasm.deleteTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, JSON.stringify(pos.cellPath), pos.charOffset, count);
  } else if (pos.parentParaIndex !== undefined) {
    const { sectionIndex: sec, parentParaIndex: ppi, controlIndex: ci, cellIndex: cei, cellParaIndex: cpi, charOffset } = pos;
    this.wasm.deleteTextInCell(sec, ppi!, ci!, cei!, cpi!, charOffset, count);
  } else {
    const { sectionIndex: sec, paragraphIndex: para, charOffset } = pos;
    this.wasm.deleteText(sec, para, charOffset, count);
  }
}
