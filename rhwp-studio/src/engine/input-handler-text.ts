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
  deleteTextWithMutationEffects,
  replaceTextWithMutationEffects,
  captureTextFragment,
  cellParaIndexOf,
  IMMEDIATE_TEXT_MUTATION_EFFECTS,
  NO_TEXT_MUTATION_EFFECTS,
  TextMutationEffectAccumulator,
} from './command';
import type { TextMutationEffects } from './command';
import type { DocumentPosition } from '@/core/types';
import { showConfirm } from '@/ui/confirm-dialog';
import { tryConfirmDeleteHyperlink } from './input-handler-hyperlink-delete';
import type { NavigationKeyInput } from './navigation-keymap';

/**
 * [#2548] WASM 삭제/조회 count 는 Rust `Paragraph::delete_text_at` 의 char(Unicode
 * scalar) 단위다. JS `String.length`(UTF-16 code unit)를 넘기면 astral 문자(😀 등)에서
 * 실제보다 많이 지워 인접 문자를 잃는다 — [#2337-review] 가 undo/HF/FN 경로에 적용한
 * 계약을 IME 조합 경로에도 맞춘다. [#7489] 조합 뒤 캐럿 오프셋도 같은 scalar 단위다.
 */
function charCount(s: string): number {
  return [...s].length;
}

// Intl.Segmenter 가 없는 브라우저(Firefox 124 이하)에서는 모듈을 깨지 않고 code point 로 센다.
const GRAPHEMES = typeof Intl.Segmenter === 'function' ? new Intl.Segmenter(undefined, { granularity: 'grapheme' }) : null;

/** 문자소(grapheme) 단위로 나눈다 — 이모지·결합 문자열도 한 글자다. */
function graphemes(s: string): string[] {
  return GRAPHEMES ? Array.from(GRAPHEMES.segment(s), (g) => g.segment) : [...s];
}

/**
 * [#7489] 수정(덮어쓰기) 모드에서 `text`가 덮어쓸 캐럿 뒤 글자를 돌려준다. 삽입 모드면 ''.
 *
 * 한컴처럼 입력한 글자 수만큼 같은 문단의 일반 글자만 덮어쓴다. 글자는 입력과 문서 모두 문자소로
 * 센다. 문단 끝, 탭·강제 줄바꿈 같은 제어 문자, 본문의 글자처럼 취급한 개체를 만나면 멈추고
 * 나머지는 삽입한다. 누름틀 안에서는 그 끝을 넘지 않고, 누름틀 밖(빠져나온 끝 포함)에서는 누름틀
 * 글자와 빈 누름틀 앞에서 멈춘다. 빠져나온 누름틀 시작에서는 덮어쓰지 않는다. 양식 모드에서 지울
 * 수 없는 글자도 덮어쓰지 않는다. 머리말/꼬리말·각주는 글자를 읽는 API가 없어 삽입을 유지한다.
 * 조회가 실패해도 삽입한다.
 */
function overwrittenTextAt(this: any, pos: DocumentPosition, text: string): string {
  if (this.insertMode !== false || !text) return '';
  if (this.cursor.isInHeaderFooter() || this.cursor.isInFootnote()) return '';
  try {
    const limit = graphemes(text).length;
    const fi = this.wasm.getFieldInfoAt(pos);
    let fieldEnd = Infinity; // 누름틀 안이면 그 끝
    if (fi.inField && fi.fieldType === 'clickhere') {
      if (this.isAtExitedFieldStart?.(pos, fi)) return '';
      if (!this.isAtExitedFieldEnd?.(pos, fi)) fieldEnd = fi.endCharIdx ?? 0;
    }
    // 마지막 글자가 잘리지 않게, 한 글자 더 보이거나 문단이 끝날 때까지 넓혀 읽는다.
    let next: string[];
    for (let n = limit + 1; ; n *= 2) {
      const read = this.getTextAt(pos, n);
      next = graphemes(read);
      if (next.length > limit || charCount(read) < n) break;
    }
    const inBody = pos.parentParaIndex === undefined;
    const logicalAt = (offset: number): number =>
      this.wasm.textToLogicalOffset(pos.sectionIndex, pos.paragraphIndex, offset);
    let offset = pos.charOffset;
    let logical = inBody ? logicalAt(offset) : 0;
    let before = fi; // offset 자리의 누름틀 조회
    let covered = '';
    for (const ch of next.slice(0, limit)) {
      if (ch < ' ') break;
      const end = offset + charCount(ch);
      if (end > fieldEnd) break;
      if (fieldEnd === Infinity) {
        // 누름틀 조회는 시작·끝 자리를 모두 안으로 치고 앞선 누름틀을 먼저 돌려준다.
        // 글자 양쪽 자리를 조회해 이 글자에 걸친 누름틀이 있으면 멈춘다. 빈 누름틀은 닿기만 해도
        // 멈춘다 — 글자를 지우면 삽입 자리로 당겨져 친 글자를 값으로 받는다.
        const after = this.wasm.getFieldInfoAt({ ...pos, charOffset: end });
        const blocks = (f: any) =>
          f.inField && (f.startCharIdx === f.endCharIdx || (f.startCharIdx < end && f.endCharIdx > offset));
        if (blocks(before) || blocks(after)) break;
        before = after;
      }
      if (inBody) {
        // 개체는 텍스트에 없고 논리 오프셋만 한 칸 차지한다 — 글자 길이보다 더 뛰면 개체가 끼어 있다.
        const after = logicalAt(end);
        if (after - logical > end - offset) break;
        logical = after;
      }
      covered += ch;
      offset = end;
    }
    if (!covered || !this.canDeleteTextInFormMode?.(pos, offset - pos.charOffset)) return '';
    return covered;
  } catch {
    return '';
  }
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

/** IME 조합 종료 후에도 현재 편집 모드의 키보드 처리 경로를 사용한다. */
function processPendingNav(this: any, nav: NavigationKeyInput): boolean {
  const key = nav.code || nav.key || '';
  // 조합 확정 Enter는 줄바꿈을 추가하지 않는다. 일반 keydown을 재생하지 않으므로
  // 이 경로에서 pagination barrier를 수행한다. 다른 탐색 키의 barrier는 onKeyDown이 맡는다.
  if (key === 'Enter') {
    this.flushDeferredPaginationIfNeeded('before-navigation', false);
    return true;
  }

  // IME의 key='Process'와 조합 플래그를 넘기면 다시 보류된다. 물리 키와 수정자만
  // 재생해 머리말/꼬리말·각주·셀의 커서 소유자, 선택과 화면 이동을 일반 입력과 공유한다.
  let handled = false;
  this.onKeyDown({
    ...nav,
    key,
    code: key,
    isComposing: false,
    keyCode: 0,
    preventDefault() { handled = true; },
  } as KeyboardEvent);
  return handled;
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
    const target = {
      sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo,
      previewPage: this.cursor.hfPreviewPage,
    };
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
      this.executeOperation({ kind: 'record', command: new MergeParagraphInHeaderFooterCommand(target, paraIdx, result.hfParaIndex, result.charOffset, paraIdx, 0, result.removedParaMeta) });
      this.cursor.setHfCursorPosition(result.hfParaIndex, result.charOffset);
      this.afterEdit();
    }
    return;
  }

  const { charOffset } = pos;

  if (tryConfirmDeleteHyperlink(this, pos, 'backward')) return;

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
    const target = {
      sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo,
      previewPage: this.cursor.hfPreviewPage,
    };
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
        this.executeOperation({ kind: 'record', command: new MergeParagraphInHeaderFooterCommand(target, paraIdx + 1, result.hfParaIndex, result.charOffset, result.hfParaIndex, result.charOffset, result.removedParaMeta) });
        this.cursor.setHfCursorPosition(result.hfParaIndex, result.charOffset);
        this.afterEdit();
      }
    } catch { /* ignore */ }
    return;
  }

  if (tryConfirmDeleteHyperlink(this, pos)) return;

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

function clearCellBlockLetterImeFollowup(this: any): void {
  this.textarea.value = '';
  this.isComposing = false;
  this.compositionAnchor = null;
  this.compositionLength = 0;
  this._lastCompositionText = '';
  this._lastComposedText = '';
  this.caret.hideComposition();
  this.resetRawTextMutationEffects();
}

export function onCompositionStart(this: any): void {
  if (this._cellBlockLetterImeGuard?.consume('compositionstart')) {
    clearCellBlockLetterImeFollowup.call(this);
    return;
  }

  this.resetRawTextMutationEffects();
  this.headerFooterSelectionComposition = false;
  // [#7489] null = 첫 조합 글자가 들어올 때 덮어쓸 글자를 정한다. 선택을 지웠으면 덮어쓰지 않는다.
  this._compositionCovered = this.cursor.hasSelection() ? '' : null;
  // 선택 영역이 있으면 삭제 후 조합 시작
  if (
    this.cursor.isInHeaderFooter()
    && this.getNonEmptyHeaderFooterSelection()
  ) {
    if (!this.beginHeaderFooterSelectionComposition()) {
      this.textarea.value = '';
      return;
    }
  } else if (!this.cursor.isInHeaderFooter() && this.cursor.hasSelection()) {
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
  if (this._cellBlockLetterImeGuard?.consume('compositionend', this.textarea.value)) {
    clearCellBlockLetterImeFollowup.call(this);
    return;
  }

  const anchor = this.compositionAnchor;
  const finalLength = this.compositionLength;
  const headerFooterSelectionComposition = this.headerFooterSelectionComposition === true;
  const covered: string = this._compositionCovered || '';
  const fragmentId: number | null = this._compositionFragment;
  this._compositionCovered = '';
  this._compositionFragment = null;

  this.isComposing = false;
  this.compositionAnchor = null;
  this.compositionLength = 0;
  this.textarea.value = '';
  this.caret.hideComposition();
  this.updateCaret();
  this.resetRawTextMutationEffects();

  // [#7489] 조합이 글자 없이 끝나면(취소) 덮었던 글자를 범위·글자 모양까지 되살린다.
  // 기록할 편집은 없다.
  if (anchor && finalLength === 0 && covered) {
    if (fragmentId !== null) {
      // 되돌리기(handleUndo)와 같은 순서로 지연 조판을 먼저 끝내고 문단을 되돌린다.
      this.flushDeferredPaginationIfNeeded('before-undo', false);
      this.wasm.restoreDeleteFragment(fragmentId);
      this.prepareTextMutationBeforeCursor(IMMEDIATE_TEXT_MUTATION_EFFECTS);
    } else {
      this.insertTextAtRaw(anchor, covered);
      this.consumeRawTextMutationBeforeCursor();
    }
    this.afterEdit();
  }

  // 더블 자음 분리 방지: compositionEnd 시점에 조합 완료된 텍스트 기억
  // 직후 유령 input 이벤트에서 동일 텍스트가 오면 무시
  this._lastComposedText = (finalLength > 0 && this._lastCompositionText) ? this._lastCompositionText : '';

  // 조합 중 WASM 직접 호출로 이미 문서에 삽입된 텍스트를
  // Command로 기록하여 Undo 가능하게 한다.
  // [Task #2337] 머리말/꼬리말·각주 모드도 이제 기록한다(본문 스냅샷 undo 의 무언 파괴 차단).
  if (anchor && finalLength > 0 && !headerFooterSelectionComposition) {
    if (this.cursor.isInHeaderFooter()) {
      // HF 는 신뢰할 텍스트 read 가 없어 getTextAt(본문 리더)을 쓸 수 없으므로 조합 텍스트
      // (_lastCompositionText)를 그대로 기록한다. anchor.charOffset = 조합 시작 오프셋,
      // hfParaIdx 는 조합 중 불변.
      const composed = this._lastCompositionText || '';
      if (composed) {
        const target = {
          sectionIdx: this.cursor.hfSectionIdx,
          isHeader: this.cursor.headerFooterMode === 'header',
          applyTo: this.cursor.hfApplyTo,
          previewPage: this.cursor.hfPreviewPage,
        };
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
        // execute() 없이 히스토리에만 기록 (텍스트는 이미 문서에 있음). 덮어쓴 글자와 덮기 전
        // 문단 조각도 함께 기록해 한 번의 되돌리기로 되살린다.
        this.executeOperation({
          kind: 'record',
          command: new InsertTextCommand(
            anchor, insertedText, undefined, undefined, covered, fragmentId === null ? [] : [fragmentId],
          ),
        });
      } else if (fragmentId !== null) {
        this.wasm.discardDeleteFragment(fragmentId);
      }
    }
  }
  if (headerFooterSelectionComposition) {
    this.finishHeaderFooterSelectionComposition();
  }

  // 조합 종료 후 대기 중인 탐색 키 처리 (IME 조합 중 방향키 등)
  if (this._pendingNavAfterIME) {
    const nav = this._pendingNavAfterIME;
    this._pendingNavAfterIME = null;
    if (processPendingNav.call(this, nav)) {
      this._imeNavigationGuard?.markReplayed();
    } else {
      // 재생이 소비하지 않은 키(HF/FN Tab 등)는 native keydown의 기본 동작을 허용한다.
      this._imeNavigationGuard?.reset();
    }
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

  if (this._cellBlockLetterImeGuard?.consume('input', this.textarea.value)) {
    e?.preventDefault();
    clearCellBlockLetterImeFollowup.call(this);
    return;
  }

  const text = this.textarea.value;
  // const inputType = e?.inputType ?? 'unknown';
  // const inputData = e?.data ?? '';
  // const isComp = e?.isComposing ?? false;

  // IME 조합 중: 이전 조합 텍스트 삭제 → 현재 조합 텍스트 삽입 (실시간 렌더링)
  // Undo 스택에는 기록하지 않음 (compositionend에서 한 번에 기록)
  if (this.isComposing && this.compositionAnchor) {
    let anchor = this.compositionAnchor;
    const beforePageIndex = this.cursor.getRect()?.pageIndex;
    if (!this.canInsertTextInFormMode?.(anchor)) {
      this.textarea.value = '';
      return;
    }
    this.resetRawTextMutationEffects();

    // [#7489] 수정 모드: 음절마다 첫 조합 글자가 캐럿 뒤 글자 하나를 덮는다. 그 글자를 조합
    // 길이에 더해 두면 아래 replace 한 번으로 함께 지워진다.
    const covering = this._compositionCovered === null && text;
    if (covering) {
      this._compositionCovered = overwrittenTextAt.call(this, anchor, text);
      if (this._compositionCovered) {
        this._compositionFragment = captureTextFragment(this.wasm, anchor);
        // 앵커가 누름틀 밖인지 편집기의 빠져나온 끝 상태로 지금 정한다(빠져나온 시작에서는 덮지 않는다).
        // 누름틀 조회는 끝 위치도 안으로 치고, 그 상태는 첫 갱신 뒤 캐럿이 다음 누름틀에 들어가면 지워진다.
        const fi = this.wasm.getFieldInfoAt(anchor);
        this._compositionOutsideField = !fi.inField || this.isAtExitedFieldEnd?.(anchor, fi);
      }
      this.compositionLength += charCount(this._compositionCovered);
    }
    // [#7489] 덮은 글자 뒤 캐럿이 누름틀 시작에 서면 그 누름틀이 활성화된다. 앵커가 누름틀 밖이면
    // replace 전마다 해제해, 지웠다 다시 넣는 조합 글자가 누름틀 안으로 끌려가지 않게 한다.
    if (this._compositionCovered && this._compositionOutsideField) this.wasm.clearActiveField();
    try {
      this.replaceTextAtRaw(anchor, this.compositionLength, text);
    } catch (err) {
      // wasm 의 deferred replace 범위 가드가 거부하면(외부 변이로 앵커·길이가 낡은
      // 경합) 여기서 던진 채 두면 onInput 전체가 죽어 조합 추적이 낡은 값으로
      // wedge 된다. 조합을 현재 캐럿에 재정박하고 이번 조합 텍스트를 새로 삽입해
      // 입력 스트림을 잇는다 — 실패분은 다음 캐럿 이동에서 자연 동기화된다.
      console.warn('[InputHandler] 조합 replace 거부 — 현재 캐럿에 재정박:', err);
      // 머리말/꼬리말·각주 모드에서는 cursor.getPosition()이 진입 전 본문 위치로
      // 고정돼 있고 실제 오프셋은 hfCharOffset/fnCharOffset에 있다(onCompositionStart와
      // 동일 규약). 이 override 없이 그대로 쓰면 insertTextAtRaw/deleteTextAt이 정확한
      // hfParaIdx/hfSectionIdx에 엉뚱한 본문 charOffset을 실어 보낸다.
      anchor = this.cursor.isInHeaderFooter()
        ? { ...this.cursor.getPosition(), charOffset: this.cursor.hfCharOffset }
        : this.cursor.isInFootnote()
          ? { ...this.cursor.getPosition(), charOffset: this.cursor.fnCharOffset }
          : { ...this.cursor.getPosition() };
      this.compositionAnchor = anchor;
      this.compositionLength = 0;
      // 거부된 replace는 이번에 덮으려던 글자도 지우지 않았다.
      if (covering) {
        this._compositionCovered = '';
        if (this._compositionFragment !== null) this.wasm.discardDeleteFragment(this._compositionFragment);
        this._compositionFragment = null;
      }
      try {
        this.replaceTextAtRaw(anchor, 0, text);
      } catch (err2) {
        console.warn('[InputHandler] 조합 재정박 삽입 실패 — 이번 업데이트 무시:', err2);
        this.textarea.value = '';
        return;
      }
    }
    // 다음 조합 업데이트의 삭제 count는 scalar 단위다.
    this.compositionLength = charCount(text);
    if (text) this._lastCompositionText = text;
    // [#4162] 캐럿 대기 서식이 있으면 이번 조합 텍스트 전체(매 갱신마다 새로 깔린 range)에
    // 적용한다. Command 를 거치지 않는 raw 삽입이라 InsertTextCommand 의 서식 적용을 못 탄다.
    this.applyPendingCharShapeToRange?.(anchor, charCount(text));

    // cursor.moveTo() 내부의 exact lookup 전에 deferred mutation을 등록하고,
    // 실제 cell-flow 경계에서만 동기 flush한다.
    const boundaryHandled = this.consumeRawTextMutationBeforeCursor();
    const newOffset = anchor.charOffset + charCount(text);
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

    this.replaceTextAtRaw(this._iosAnchor, this._iosLength, text);
    this._iosLength = charCount(text);

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

    clearTimeout(this._iosInputTimer);
    const iosAnchor = this._iosAnchor;
    const iosAfterPos = this.cursor.getPosition();
    const beforePageIndex = this._iosBeforePageIndex;
    const afterPageIndex = this.cursor.getRect()?.pageIndex;
    const requiresFullRefresh = this._iosRequiresFullRefresh;
    this._iosRequiresFullRefresh = false;
    this.afterTextInputEdit(iosAnchor, iosAfterPos, {
      insertedText: text,
      beforePageIndex,
      afterPageIndex,
    }, requiresFullRefresh);
    this.textarea.focus();
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
      if (this.getNonEmptyHeaderFooterSelection()) {
        this.replaceHeaderFooterSelection(text, {
          operationType: 'replaceSelectionInHeaderFooter',
        });
        return;
      }
      const target = {
        sectionIdx: this.cursor.hfSectionIdx, isHeader, applyTo: this.cursor.hfApplyTo,
        previewPage: this.cursor.hfPreviewPage,
      };
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
  // 선택 영역은 삽입 모드와 똑같이 입력으로 바뀌고, 뒤 글자를 더 덮어쓰지 않는다.
  const hadSelection = this.cursor.hasSelection();
  if (hadSelection) {
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
  // [#4162] 선택 없이 지정한 서식은 예약(pending)돼 있다 — 있으면 삽입 커맨드에 실어 보낸다.
  const replaced = hadSelection ? '' : overwrittenTextAt.call(this, insertPos, text);
  this.executeOperation({
    kind: 'command',
    command: new InsertTextCommand(insertPos, text, undefined, this.getPendingCharShape?.(), replaced),
  });
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
    return IMMEDIATE_TEXT_MUTATION_EFFECTS;
  }
  // 각주 편집 모드
  if (this.cursor.isInFootnote()) {
    this.wasm.insertTextInFootnote(
      this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
      this.cursor.fnInnerParaIdx, pos.charOffset, text,
    );
    return IMMEDIATE_TEXT_MUTATION_EFFECTS;
  }
  return insertTextWithMutationEffects(this.wasm, pos, text);
}

export function replaceTextAtRaw(
  this: any,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects {
  if (!this.canInsertTextInFormMode?.(pos)) return NO_TEXT_MUTATION_EFFECTS;
  if (deleteCount > 0 && !this.canDeleteTextInFormMode?.(pos, deleteCount)) {
    return NO_TEXT_MUTATION_EFFECTS;
  }
  if (!this.cursor.isInHeaderFooter() && !this.cursor.isInFootnote()) {
    return replaceTextWithMutationEffects(this.wasm, pos, deleteCount, text);
  }

  const effects = new TextMutationEffectAccumulator();
  if (deleteCount > 0) {
    effects.add(deleteTextAt.call(this, pos, deleteCount));
  }
  if (text.length > 0) {
    effects.add(insertTextAtRaw.call(this, pos, text));
  }
  return effects.consume();
}

export function deleteTextAt(this: any, pos: DocumentPosition, count: number): TextMutationEffects {
  if (!this.canDeleteTextInFormMode?.(pos, count)) return NO_TEXT_MUTATION_EFFECTS;
  // 머리말/꼬리말 편집 모드
  if (this.cursor.isInHeaderFooter()) {
    const isHeader = this.cursor.headerFooterMode === 'header';
    this.wasm.deleteTextInHeaderFooter(
      this.cursor.hfSectionIdx, isHeader, this.cursor.hfApplyTo,
      this.cursor.hfParaIdx, pos.charOffset, count,
    );
    return NO_TEXT_MUTATION_EFFECTS;
  }
  // 각주 편집 모드
  if (this.cursor.isInFootnote()) {
    this.wasm.deleteTextInFootnote(
      this.cursor.fnSectionIdx, this.cursor.fnParaIdx, this.cursor.fnControlIdx,
      this.cursor.fnInnerParaIdx, pos.charOffset, count,
    );
    return NO_TEXT_MUTATION_EFFECTS;
  }
  return deleteTextWithMutationEffects(this.wasm, pos, count);
}
