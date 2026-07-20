import type { WasmBridge } from '@/core/wasm-bridge';
import type { DocumentPosition, CharProperties, ParaProperties, CellPathLike } from '@/core/types';
import { MAX_PAGE_LOCAL_TEXT_EDIT_CHARS } from './input-edit-invalidation';

/** 편집 명령 공통 인터페이스 */
export interface EditCommand {
  readonly type: string;
  readonly timestamp: number;
  /** 명령 실행 — 실행 후 커서 위치 반환 */
  execute(wasm: WasmBridge): DocumentPosition;
  /** 역실행 — 실행 전 커서 위치 반환 */
  undo(wasm: WasmBridge): DocumentPosition;
  /** 연속 명령 병합 시도 */
  mergeWith(other: EditCommand): EditCommand | null;
  /** 리소스 해제 (스냅샷 명령의 메모리 반환 등). 스택에서 제거될 때 호출. */
  discard?(wasm: WasmBridge): void;
  /**
   * [Task #2328] 이 명령이 현재 점유한 WASM 스냅샷 id 개수(없으면 0).
   * CommandHistory 가 스냅샷 예산을 WASM 상한과 정합시키는 데 쓴다.
   */
  snapshotResourceCount?(): number;
  /** page-local refresh 판정을 위한 가벼운 텍스트 편집 payload. */
  getPageLocalTextEditOptions?(): { insertedText?: string; deleteCount?: number };
  /** 방금 실행한 mutation effect를 한 번만 반환한다. */
  consumeTextMutationEffects?(): TextMutationEffects;
  /**
   * [Task #2337] 이 커맨드의 마지막 execute/undo 후 복원할 머리말/꼬리말·각주 편집
   * 컨텍스트. 본문 커맨드는 미구현(→ 반환 없음 = 본문 모드). InputHandler 가 undo/redo
   * 시 이 값을 읽어 HF/FN 모드 재진입 + 커서 위치를 복원하고 본문 moveTo 를 건너뛴다.
   */
  editContext?(): EditContext | null;
}

/**
 * [Task #2337] 머리말/꼬리말·각주 편집 커맨드가 undo/redo 후 복원할 편집 컨텍스트.
 * 본문 DocumentPosition 과 별개인 HF/FN 커서 모드를 서술한다(cursor.ts 의
 * enterHeaderFooterMode/enterFootnoteMode + set{Hf,Fn}CursorPosition 인자에 대응).
 */
export type EditContext =
  | {
      readonly mode: 'headerFooter';
      readonly sectionIdx: number;
      readonly isHeader: boolean;
      readonly applyTo: number;
      readonly paraIdx: number;
      readonly charOffset: number;
    }
  | {
      readonly mode: 'footnote';
      readonly sectionIdx: number;
      readonly paraIdx: number;
      readonly controlIdx: number;
      readonly footnoteIndex: number;
      readonly pageNum: number;
      readonly innerParaIdx: number;
      readonly charOffset: number;
    };

/** cell text mutation의 deferred/flow 경계와 immediate pagination 완료를 함께 전달한다. */
export interface TextMutationEffects {
  readonly deferredPagination: boolean;
  readonly cellFlowChanged: boolean;
  readonly paginationCompleted: boolean;
}

export const NO_TEXT_MUTATION_EFFECTS: TextMutationEffects = Object.freeze({
  deferredPagination: false,
  cellFlowChanged: false,
  paginationCompleted: false,
});

export const IMMEDIATE_TEXT_MUTATION_EFFECTS: TextMutationEffects = Object.freeze({
  deferredPagination: false,
  cellFlowChanged: false,
  paginationCompleted: true,
});

/** raw IME/iOS 묶음에서 effect를 OR 누적하고 한 번만 소비한다. */
export class TextMutationEffectAccumulator {
  private effects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  add(effects: TextMutationEffects): void {
    this.effects = {
      deferredPagination: this.effects.deferredPagination || effects.deferredPagination,
      cellFlowChanged: this.effects.cellFlowChanged || effects.cellFlowChanged,
      paginationCompleted: this.effects.paginationCompleted || effects.paginationCompleted,
    };
  }

  consume(): TextMutationEffects {
    const effects = this.effects;
    this.effects = NO_TEXT_MUTATION_EFFECTS;
    return effects;
  }

  clear(): void {
    this.effects = NO_TEXT_MUTATION_EFFECTS;
  }
}

// ─── 편집 작업 서술자 (라우팅 통합) ────────────────────

export type EditDomain =
  | 'text'
  | 'charFormat'
  | 'paraFormat'
  | 'table'
  | 'object'
  | 'page'
  | 'field'
  | 'view'
  | 'unknown';

export type RefreshPolicy = 'auto' | 'full' | 'pageLocal' | 'selectionOnly' | 'none';

export type DirtyScope =
  | 'document'
  | 'section'
  | 'page'
  | 'paragraph'
  | 'table'
  | 'object'
  | 'none';

export type SelectionPolicy =
  | 'auto'
  | 'keep'
  | 'moveToResult'
  | 'restoreObjectSelection'
  | 'none';

export interface OperationMetadata {
  /** 메뉴/툴바/단축키 action id. */
  actionId?: string;
  /** 편집 도메인. 직접 wasm mutation을 audit 할 때 분류 기준으로 사용한다. */
  domain?: EditDomain;
  /** mutation 후 렌더링 갱신 정책. 생략하면 kind 별 기존 기본값을 따른다. */
  refresh?: RefreshPolicy;
  /** 장기적으로 renderer invalidation 최적화에 사용할 dirty 범위. */
  dirtyScope?: DirtyScope;
  /** selection/caret 복원 정책. 현재는 문서화용 metadata로만 사용한다. */
  selection?: SelectionPolicy;
}

/**
 * 편집 작업 서술자 — 호출부가 "무엇을 하려는가"만 기술하고,
 * 라우터(executeOperation)가 적절한 Undo 전략을 자동 선택한다.
 *
 * - command: 정밀 커맨드 (텍스트 삽입/삭제, 문단 분할/병합, 서식)
 * - snapshot: 스냅샷 기반 커맨드 (붙여넣기, 객체 삭제 등)
 * - record:  WASM 직접 호출 후 히스토리에만 기록 (IME, 객체 이동).
 *            아키텍처 문서에서는 recordApplied 계약으로 정의한다.
 */
export type OperationDescriptor =
  | { kind: 'command'; command: EditCommand; meta?: OperationMetadata }
  | { kind: 'snapshot'; operationType: string; operation: (wasm: WasmBridge) => DocumentPosition; meta?: OperationMetadata }
  | { kind: 'record'; command: EditCommand; meta?: OperationMetadata };

// ─── 본문/셀 분기 헬퍼 ────────────────────────────────

function isCell(pos: DocumentPosition): boolean {
  return pos.parentParaIndex !== undefined;
}

/** 중첩 표(depth > 1)인지 확인 */
function isNestedCell(pos: DocumentPosition): boolean {
  return (pos.cellPath?.length ?? 0) > 1;
}

export function canUseDeferredCellTextInsert(pos: DocumentPosition, text: string): boolean {
  if (!isCell(pos) || isNestedCell(pos)) return false;
  if (text.length === 0 || text.length > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
  if (/[\r\n\t]/.test(text)) return false;
  return true;
}

/** cellPath를 WASM용 JSON 문자열로 변환 */
function cellPathJson(pos: DocumentPosition): string {
  return JSON.stringify(pos.cellPath ?? []);
}

/** cellPath 의 최내곽(마지막) 엔트리의 cellParaIndex 를 지정 값으로 바꾼 pathJson.
 *  중첩 셀의 문단별 ByPath 호출(삭제 undo 저장과 문자 서식 적용/undo)에 쓴다. */
function cellPathJsonForPara(pos: DocumentPosition, cellParaIndex: number): string {
  const path = (pos.cellPath ?? []).map((e) => ({ ...e }));
  if (path.length > 0) path[path.length - 1].cellParaIndex = cellParaIndex;
  return JSON.stringify(path);
}

/**
 * 셀 문단 인덱스 — cellPath 가 있으면 마지막(가장 안쪽) 엔트리에서 읽는다.
 *
 * hit-test 는 flat 필드(controlIndex/cellIndex/cellParaIndex)를 `cellPath[0]`, 즉 **최외곽**
 * 엔트리에서 채운다(cursor_rect.rs 의 `outer = &ctx.path[0]`). 그래서 중첩 셀에서
 * `pos.cellParaIndex` 는 바깥 셀의 문단 인덱스이고, 안쪽 셀의 값은
 * `cellPath[last].cellParaIndex` 다. 이를 섞으면 ...ByPath API 에 바깥 축의 인덱스를 넘겨
 * 엉뚱한 문단을 병합/분할한다.
 *
 * cursor.ts(:399) 와 input-handler-text.ts(:307) 의 `useCellPath` 분기와 동일한 규칙이다.
 * depth 1 에서는 `cellPath[0]` 이 곧 최외곽이라 flat 값과 같으므로 동작 변화가 없다.
 */
function cellParaIndexOf(pos: DocumentPosition): number {
  const path = pos.cellPath;
  return (path?.length ?? 0) > 0 ? path![path!.length - 1].cellParaIndex : pos.cellParaIndex!;
}

/** 셀 문단 구조 편집 뒤 flat/path 커서 위치를 같은 문단으로 맞춘다. */
function cellParagraphPosition(
  pos: DocumentPosition,
  cellParaIndex: number,
  charOffset: number,
): DocumentPosition {
  const cellPath = pos.cellPath?.map((entry, index, path) =>
    index + 1 === path.length ? { ...entry, cellParaIndex } : entry,
  );
  return {
    ...pos,
    paragraphIndex: cellParaIndex,
    cellParaIndex,
    cellPath,
    charOffset,
    cursorRect: undefined,
  };
}

export function insertTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  text: string,
): TextMutationEffects {
  if (isNestedCell(pos)) {
    wasm.insertTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), pos.charOffset, text);
  } else if (isCell(pos)) {
    if (canUseDeferredCellTextInsert(pos, text)) {
      const result = wasm.insertTextInCellDeferredPagination(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, text);
      return {
        deferredPagination: result.paginationDeferred,
        cellFlowChanged: result.cellFlowChanged,
        paginationCompleted: !result.paginationDeferred,
      };
    } else {
      wasm.insertTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, text);
    }
  } else {
    wasm.insertText(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, text);
  }
  return IMMEDIATE_TEXT_MUTATION_EFFECTS;
}

/** undo/구조 명령의 full-refresh 복원은 flat cell에서도 immediate pagination을 사용한다. */
function doInsertTextImmediate(wasm: WasmBridge, pos: DocumentPosition, text: string): void {
  if (isNestedCell(pos)) {
    wasm.insertTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), pos.charOffset, text);
  } else if (isCell(pos)) {
    wasm.insertTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, text);
  } else {
    wasm.insertText(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, text);
  }
}

function doDeleteText(wasm: WasmBridge, pos: DocumentPosition, count: number): void {
  if (isNestedCell(pos)) {
    wasm.deleteTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), pos.charOffset, count);
  } else if (isCell(pos)) {
    wasm.deleteTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, count);
  } else {
    wasm.deleteText(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, count);
  }
}

function doGetTextRange(wasm: WasmBridge, pos: DocumentPosition, count: number): string {
  if (isNestedCell(pos)) {
    return wasm.getTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), pos.charOffset, count);
  } else if (isCell(pos)) {
    return wasm.getTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, count);
  } else {
    return wasm.getTextRange(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, count);
  }
}

// ─── 텍스트 삽입 명령 ─────────────────────────────────

export class InsertTextCommand implements EditCommand {
  readonly type = 'insertText';
  readonly timestamp: number;
  private lastMutationEffects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  constructor(
    private position: DocumentPosition,
    private text: string,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    this.lastMutationEffects = insertTextWithMutationEffects(wasm, this.position, this.text);
    return { ...this.position, charOffset: this.position.charOffset + this.text.length };
  }

  consumeTextMutationEffects(): TextMutationEffects {
    const effects = this.lastMutationEffects;
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    return effects;
  }

  getPageLocalTextEditOptions(): { insertedText: string } {
    return { insertedText: this.text };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    // [#2337-review] 삭제 count 는 char(Unicode scalar) 단위다. UTF-16 length 를 넘기면
    // astral 문자에서 실제보다 많이 지워 인접 문자를 잃는다 → HF/FN 과 동일하게 charCount.
    doDeleteText(wasm, this.position, charCount(this.text));
    return { ...this.position };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof InsertTextCommand)) return null;
    // 같은 문단/셀인지 확인
    if (other.position.sectionIndex !== this.position.sectionIndex) return null;
    if (other.position.paragraphIndex !== this.position.paragraphIndex) return null;
    if (isCell(this.position) !== isCell(other.position)) return null;
    if (isCell(this.position)) {
      if (other.position.parentParaIndex !== this.position.parentParaIndex) return null;
      if (other.position.controlIndex !== this.position.controlIndex) return null;
      if (other.position.cellIndex !== this.position.cellIndex) return null;
      if (other.position.cellParaIndex !== this.position.cellParaIndex) return null;
    }
    // 연속 위치 확인
    const expectedOffset = this.position.charOffset + this.text.length;
    if (other.position.charOffset !== expectedOffset) return null;
    // 300ms 이내
    if (other.timestamp - this.timestamp > 300) return null;
    // 줄바꿈/탭 포함 시 병합 불가
    if (other.text.includes('\n') || other.text.includes('\t')) return null;

    return new InsertTextCommand(this.position, this.text + other.text, this.timestamp);
  }
}

// ─── 텍스트 삭제 명령 ─────────────────────────────────

export class DeleteTextCommand implements EditCommand {
  readonly type = 'deleteText';
  readonly timestamp: number;

  /** undo용 삭제된 텍스트 (execute 시 보존) */
  private deletedText: string;
  private lastMutationEffects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  constructor(
    private position: DocumentPosition,
    private count: number,
    private direction: 'forward' | 'backward',
    deletedText?: string,
    timestamp?: number,
  ) {
    this.deletedText = deletedText ?? '';
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    // 삭제 전 텍스트 보존
    if (!this.deletedText) {
      this.deletedText = doGetTextRange(wasm, this.position, this.count);
    }
    doDeleteText(wasm, this.position, this.count);
    this.lastMutationEffects = IMMEDIATE_TEXT_MUTATION_EFFECTS;
    return { ...this.position };
  }

  consumeTextMutationEffects(): TextMutationEffects {
    const effects = this.lastMutationEffects;
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    return effects;
  }

  getPageLocalTextEditOptions(): { deleteCount: number } {
    return { deleteCount: this.count };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    doInsertTextImmediate(wasm, this.position, this.deletedText);
    const restoredLen = this.deletedText.length;
    return { ...this.position, charOffset: this.position.charOffset + restoredLen };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof DeleteTextCommand)) return null;
    if (other.direction !== this.direction) return null;
    if (other.timestamp - this.timestamp > 300) return null;
    // 같은 문단/셀 확인
    if (other.position.sectionIndex !== this.position.sectionIndex) return null;
    if (other.position.paragraphIndex !== this.position.paragraphIndex) return null;
    if (isCell(this.position) !== isCell(other.position)) return null;
    if (isCell(this.position)) {
      if (other.position.parentParaIndex !== this.position.parentParaIndex) return null;
      if (other.position.controlIndex !== this.position.controlIndex) return null;
      if (other.position.cellIndex !== this.position.cellIndex) return null;
      if (other.position.cellParaIndex !== this.position.cellParaIndex) return null;
    }

    if (this.direction === 'backward') {
      // Backspace: 연속 앞쪽 삭제
      if (other.position.charOffset === this.position.charOffset - other.count) {
        return new DeleteTextCommand(
          other.position, this.count + other.count, 'backward',
          other.deletedText + this.deletedText, this.timestamp,
        );
      }
    } else {
      // Delete: 같은 위치에서 연속 삭제
      if (other.position.charOffset === this.position.charOffset) {
        return new DeleteTextCommand(
          this.position, this.count + other.count, 'forward',
          this.deletedText + other.deletedText, this.timestamp,
        );
      }
    }
    return null;
  }
}

// ─── 강제 줄바꿈 명령 (Shift+Enter) ─────────────────────

export class InsertLineBreakCommand implements EditCommand {
  readonly type = 'insertLineBreak';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    doInsertTextImmediate(wasm, this.position, '\n');
    const newPos = { ...this.position, charOffset: this.position.charOffset + 1 };
    return newPos;
  }

  undo(wasm: WasmBridge): DocumentPosition {
    doDeleteText(wasm, this.position, 1);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── 탭 삽입 명령 (Tab) ──────────────────────────────

export class InsertTabCommand implements EditCommand {
  readonly type = 'insertTab';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    doInsertTextImmediate(wasm, this.position, '\t');
    const newPos = { ...this.position, charOffset: this.position.charOffset + 1 };
    return newPos;
  }

  undo(wasm: WasmBridge): DocumentPosition {
    doDeleteText(wasm, this.position, 1);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── 문단 분할 명령 (Enter) ───────────────────────────

export class SplitParagraphCommand implements EditCommand {
  readonly type = 'splitParagraph';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para, charOffset } = this.position;
    const result = JSON.parse(wasm.splitParagraph(sec, para, charOffset));
    if (result.ok) {
      return { sectionIndex: sec, paragraphIndex: result.paraIdx, charOffset: 0 };
    }
    return this.position;
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    wasm.mergeParagraph(sec, para + 1);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── 문단 병합 명령 (문단 시작에서 Backspace) ─────────

export class MergeParagraphCommand implements EditCommand {
  readonly type = 'mergeParagraph';
  readonly timestamp = Date.now();

  /** undo 시 분할 위치 (이전 문단의 원래 길이) */
  private mergePointOffset = 0;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    // 병합 전 이전 문단 길이 기억
    this.mergePointOffset = wasm.getParagraphLength(sec, para - 1);
    wasm.mergeParagraph(sec, para);
    return { sectionIndex: sec, paragraphIndex: para - 1, charOffset: this.mergePointOffset };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    wasm.splitParagraph(sec, para - 1, this.mergePointOffset);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── 선택 영역 삭제 명령 ─────────────────────────────

export class DeleteSelectionCommand implements EditCommand {
  readonly type = 'deleteSelection';
  readonly timestamp = Date.now();

  /** undo용: 삭제 전 선택 영역의 텍스트 (문단별) */
  private savedTexts: string[] = [];
  /** undo용: 삭제 전 다중 문단 여부 */
  private multiPara = false;

  constructor(
    private start: DocumentPosition,
    private end: DocumentPosition,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { start, end } = this;

    // 삭제 전 텍스트 보존 (undo용)
    this.savedTexts = [];
    if (isCell(start)) {
      // 중첩 셀 좌표 축 정합: flat controlIndex/cellIndex 는 cellPath[0](최외곽)이라
      // 중첩 셀에서 바깥 셀을 삭제한다. 최내곽 셀을 대상으로 ...ByPath 로 라우팅하고,
      // 셀 문단 인덱스는 cellPath[last] 에서 읽는다(cellParaIndexOf). undo 는 이미
      // doInsertTextImmediate 가 cellPath 로 최내곽 셀에 삽입하므로 축이 일치한다.
      const sec = start.sectionIndex;
      const ppi = start.parentParaIndex!;
      const startPara = cellParaIndexOf(start);
      const endPara = cellParaIndexOf(end);
      this.multiPara = startPara !== endPara;
      for (let p = startPara; p <= endPara; p++) {
        const pathP = cellPathJsonForPara(start, p);
        const pLen = wasm.getCellParagraphLengthByPath(sec, ppi, pathP);
        const from = p === startPara ? start.charOffset : 0;
        const to = p === endPara ? end.charOffset : pLen;
        if (to > from) {
          this.savedTexts.push(wasm.getTextInCellByPath(sec, ppi, pathP, from, to - from));
        } else {
          this.savedTexts.push('');
        }
      }
      wasm.deleteRangeInCellByPath(sec, ppi, cellPathJson(start), startPara, start.charOffset, endPara, end.charOffset);
    } else {
      const sec = start.sectionIndex;
      this.multiPara = start.paragraphIndex !== end.paragraphIndex;
      for (let p = start.paragraphIndex; p <= end.paragraphIndex; p++) {
        const pLen = wasm.getParagraphLength(sec, p);
        const from = p === start.paragraphIndex ? start.charOffset : 0;
        const to = p === end.paragraphIndex ? end.charOffset : pLen;
        if (to > from) {
          this.savedTexts.push(wasm.getTextRange(sec, p, from, to - from));
        } else {
          this.savedTexts.push('');
        }
      }
      wasm.deleteRange(sec, start.paragraphIndex, start.charOffset, end.paragraphIndex, end.charOffset);
    }

    return { ...start };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { start } = this;

    if (!this.multiPara) {
      // 같은 문단 내 삭제 → 텍스트 재삽입
      const text = this.savedTexts[0] || '';
      if (text) doInsertTextImmediate(wasm, start, text);
    } else {
      // 다중 문단: 마지막 텍스트부터 역순으로 복원
      // 1) 첫 문단 뒷부분 텍스트 삽입
      const firstText = this.savedTexts[0] || '';
      if (firstText) doInsertTextImmediate(wasm, start, firstText);

      // 2) 중간 문단 + 마지막 문단: splitParagraph로 분리 후 텍스트 삽입
      if (isCell(start)) {
        // 셀 내 다중 문단 undo는 복잡하므로 단순 텍스트 삽입으로 대체
        // (셀 내 다중 문단 선택 삭제의 undo는 한계가 있음)
        for (let i = 1; i < this.savedTexts.length; i++) {
          const text = this.savedTexts[i];
          if (text || i < this.savedTexts.length - 1) {
            // splitParagraph는 셀 내에서 미지원 → 텍스트만 이어붙이기
            const restorePos = { ...start, charOffset: start.charOffset + firstText.length };
            if (text) doInsertTextImmediate(wasm, restorePos, '\n' + text);
          }
        }
      } else {
        const sec = start.sectionIndex;
        let currentPara = start.paragraphIndex;
        let currentOffset = start.charOffset + firstText.length;
        for (let i = 1; i < this.savedTexts.length; i++) {
          wasm.splitParagraph(sec, currentPara, currentOffset);
          currentPara++;
          const text = this.savedTexts[i];
          if (text) {
            wasm.insertText(sec, currentPara, 0, text);
          }
          // 다음 분할점은 방금 복원한 텍스트 "뒤" 다. 0 으로 두면 이미 복원된 텍스트 앞을
          // 잘라 빈 문단이 끼어들고 내용이 다음 문단으로 밀린다(문단 3개 이상 선택 삭제 undo).
          currentOffset = text ? text.length : 0;
        }
      }
    }

    return { ...this.end };
  }

  mergeWith(): null { return null; }
}

// ─── 글자 서식 적용 명령 ─────────────────────────────

/** 문단 하나에 대한 서식 적용 정보 */
interface ParaFormatEntry {
  paraIndex: number;       // 본문: paragraphIndex, 셀: cellParaIndex
  startOffset: number;
  endOffset: number;
  /** undo용: 적용 전 charShapeId */
  beforeCharShapeId?: number;
  /** redo용: 적용 후 charShapeId */
  afterCharShapeId?: number;
}

export class ApplyCharFormatCommand implements EditCommand {
  readonly type = 'applyCharFormat';
  readonly timestamp = Date.now();

  private entries: ParaFormatEntry[] = [];

  constructor(
    private start: DocumentPosition,
    private end: DocumentPosition,
    private props: Partial<CharProperties>,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.entries.length > 0 && this.entries.every((entry) => entry.afterCharShapeId !== undefined)) {
      this.restoreCharShapeIds(wasm, 'after');
      return { ...this.start };
    }

    const { start, end } = this;
    const propsJson = JSON.stringify(this.props);

    if (isCell(start)) {
      // 중첩 셀 좌표 축 정합: flat controlIndex/cellIndex 는 cellPath[0](최외곽)이라 중첩
      // 셀에서 바깥 셀에 서식을 적용한다. 최내곽 셀을 ...ByPath 로 라우팅하고 문단 인덱스는
      // cellPath[last](cellParaIndexOf)에서 읽는다. undo(restoreCharShapeIds)도 같은 축.
      const sec = start.sectionIndex;
      const ppi = start.parentParaIndex!;
      const startPara = cellParaIndexOf(start);
      const endPara = cellParaIndexOf(end);

      this.entries = [];
      for (let p = startPara; p <= endPara; p++) {
        const pathP = cellPathJsonForPara(start, p);
        const from = p === startPara ? start.charOffset : 0;
        const to = p === endPara ? end.charOffset : wasm.getCellParagraphLengthByPath(sec, ppi, pathP);
        if (to <= from) continue;

        const prevProps = wasm.getCellCharPropertiesAtByPath(sec, ppi, pathP, from);
        this.entries.push({ paraIndex: p, startOffset: from, endOffset: to, beforeCharShapeId: prevProps.charShapeId });

        wasm.applyCharFormatInCellByPath(sec, ppi, pathP, from, to, propsJson);
        const afterProps = wasm.getCellCharPropertiesAtByPath(sec, ppi, pathP, from);
        this.entries[this.entries.length - 1].afterCharShapeId = afterProps.charShapeId;
      }
    } else {
      const sec = start.sectionIndex;
      const startPara = start.paragraphIndex;
      const endPara = end.paragraphIndex;

      this.entries = [];
      for (let p = startPara; p <= endPara; p++) {
        const from = p === startPara ? start.charOffset : 0;
        const to = p === endPara ? end.charOffset : wasm.getParagraphLength(sec, p);
        if (to <= from) continue;

        const prevProps = wasm.getCharPropertiesAt(sec, p, from);
        this.entries.push({ paraIndex: p, startOffset: from, endOffset: to, beforeCharShapeId: prevProps.charShapeId });

        wasm.applyCharFormat(sec, p, from, to, propsJson);
        const afterProps = wasm.getCharPropertiesAt(sec, p, from);
        this.entries[this.entries.length - 1].afterCharShapeId = afterProps.charShapeId;
      }
    }

    return { ...this.start };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    this.restoreCharShapeIds(wasm, 'before');
    return { ...this.start };
  }

  private restoreCharShapeIds(wasm: WasmBridge, side: 'before' | 'after'): void {
    const { start } = this;
    for (const entry of this.entries) {
      const charShapeId = side === 'before' ? entry.beforeCharShapeId : entry.afterCharShapeId;
      if (charShapeId === undefined) continue;

      if (isCell(start)) {
        // 중첩 셀 축 정합: 최내곽 셀에 서식 ID 복원(execute 와 동일 축).
        wasm.setCharShapeIdInCellByPath(
          start.sectionIndex, start.parentParaIndex!, cellPathJsonForPara(start, entry.paraIndex),
          entry.startOffset, entry.endOffset, charShapeId,
        );
      } else {
        wasm.setCharShapeId(start.sectionIndex, entry.paraIndex, entry.startOffset, entry.endOffset, charShapeId);
      }
    }
  }

  mergeWith(): null { return null; }
}

// ─── 문단 서식 적용 명령 ─────────────────────────────

export type ParaFormatTarget =
  | { kind: 'body'; sec: number; para: number }
  | { kind: 'cell'; sec: number; parentPara: number; controlIdx: number; cellIdx: number; cellParaIdx: number };

interface ParaShapeHistoryEntry {
  target: ParaFormatTarget;
  beforeParaShapeId: number;
  afterParaShapeId?: number;
}

function getParaShapeId(wasm: WasmBridge, target: ParaFormatTarget): number {
  const props = target.kind === 'body'
    ? wasm.getParaPropertiesAt(target.sec, target.para)
    : wasm.getCellParaPropertiesAt(
        target.sec,
        target.parentPara,
        target.controlIdx,
        target.cellIdx,
        target.cellParaIdx,
      );
  const paraShapeId = props.paraShapeId;
  if (paraShapeId === undefined) {
    throw new Error('문단 모양 ID를 조회할 수 없습니다');
  }
  return paraShapeId;
}

function applyParaFormatToTarget(wasm: WasmBridge, target: ParaFormatTarget, propsJson: string): void {
  if (target.kind === 'body') {
    wasm.applyParaFormat(target.sec, target.para, propsJson);
    return;
  }
  wasm.applyParaFormatInCell(
    target.sec,
    target.parentPara,
    target.controlIdx,
    target.cellIdx,
    target.cellParaIdx,
    propsJson,
  );
}

function restoreParaShapeId(wasm: WasmBridge, target: ParaFormatTarget, paraShapeId: number): void {
  if (target.kind === 'body') {
    wasm.setParaShapeId(target.sec, target.para, paraShapeId);
    return;
  }
  wasm.setCellParaShapeId(
    target.sec,
    target.parentPara,
    target.controlIdx,
    target.cellIdx,
    target.cellParaIdx,
    paraShapeId,
  );
}

export class ApplyParaFormatCommand implements EditCommand {
  readonly type = 'applyParaFormat';
  readonly timestamp = Date.now();

  private entries: ParaShapeHistoryEntry[] = [];

  constructor(
    private targets: ParaFormatTarget[],
    private props: Partial<ParaProperties>,
    private cursorBefore: DocumentPosition,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.entries.length > 0 && this.entries.every(entry => entry.afterParaShapeId !== undefined)) {
      for (const entry of this.entries) {
        restoreParaShapeId(wasm, entry.target, entry.afterParaShapeId!);
      }
      return { ...this.cursorBefore };
    }

    const propsJson = JSON.stringify(this.props);
    const entries: ParaShapeHistoryEntry[] = this.targets.map(target => ({
      target,
      beforeParaShapeId: getParaShapeId(wasm, target),
    }));

    for (const entry of entries) {
      applyParaFormatToTarget(wasm, entry.target, propsJson);
    }
    for (const entry of entries) {
      entry.afterParaShapeId = getParaShapeId(wasm, entry.target);
    }

    this.entries = entries;
    return { ...this.cursorBefore };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    for (const entry of this.entries) {
      restoreParaShapeId(wasm, entry.target, entry.beforeParaShapeId);
    }
    return { ...this.cursorBefore };
  }

  mergeWith(): null { return null; }
}

// ─── 문단 끝에서 Delete로 다음 문단 병합 ─────────────

export class MergeNextParagraphCommand implements EditCommand {
  readonly type = 'mergeNextParagraph';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    wasm.mergeParagraph(sec, para + 1);
    return { ...this.position };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para, charOffset } = this.position;
    wasm.splitParagraph(sec, para, charOffset);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── [Task #2337] 머리말/꼬리말·각주 편집 커맨드 ───────────────────────────
//
// HF/FN 편집은 본문과 별개 WASM 경로(insert/delete/merge/split × HeaderFooter/Footnote)
// 를 쓰고 커서도 별도 모드다. 본문 텍스트/문단 커맨드(역연산 경량)를 미러링해 히스토리에
// 기록함으로써, 본문 스냅샷 undo 가 미기록 HF/FN 편집을 무언 파괴하던 데이터 손실을 막는다.
// 최초 적용은 InputHandler 가 인라인 뮤테이션 후 kind:'record' 로 기록하므로 execute()
// 는 redo 시에만 호출된다. undo()/execute() 는 lastContext 를 각각 실행 전/후 좌표로
// 갱신하며, InputHandler 가 editContext() 로 읽어 HF/FN 커서 모드를 복원한다(커맨드는 순수).

export interface HeaderFooterEditTarget {
  readonly sectionIdx: number;
  readonly isHeader: boolean;
  readonly applyTo: number;
}

export interface FootnoteEditTarget {
  readonly sectionIdx: number;
  /** 각주 컨트롤을 담은 본문 문단 인덱스 */
  readonly paraIdx: number;
  readonly controlIdx: number;
  /** 모드 재진입용 (enterFootnoteMode 인자) */
  readonly footnoteIndex: number;
  readonly pageNum: number;
}

function hfEditContext(t: HeaderFooterEditTarget, paraIdx: number, charOffset: number): EditContext {
  return { mode: 'headerFooter', sectionIdx: t.sectionIdx, isHeader: t.isHeader, applyTo: t.applyTo, paraIdx, charOffset };
}

function fnEditContext(t: FootnoteEditTarget, innerParaIdx: number, charOffset: number): EditContext {
  return {
    mode: 'footnote', sectionIdx: t.sectionIdx, paraIdx: t.paraIdx, controlIdx: t.controlIdx,
    footnoteIndex: t.footnoteIndex, pageNum: t.pageNum, innerParaIdx, charOffset,
  };
}

/**
 * HF/FN 커맨드의 execute/undo 반환 위치는 형식상 값이다 — InputHandler 는 editContext()
 * 로 커서를 복원하며 이 본문 위치로 moveTo 하지 않는다(단, 반환은 non-null 이어야
 * history.undo/redo 가 성공으로 간주한다).
 */
function hfFnStubPosition(sectionIdx: number): DocumentPosition {
  return { sectionIndex: sectionIdx, paragraphIndex: 0, charOffset: 0 };
}

/**
 * [Task #2337-review] WASM 삭제 count 는 Rust `Paragraph::delete_text_at` 의 char(Unicode
 * scalar) 단위다. JS `String.length`(UTF-16 code unit)를 넘기면 astral 문자(😀 등)에서
 * 실제보다 많이 삭제해 undo/redo 가 인접 문자를 잃는다 → 코드포인트 수로 계산한다.
 * (커서 오프셋은 studio 의 UTF-16 관례를 유지하므로 여기서만 char 단위를 쓴다.)
 */
function charCount(s: string): number {
  return [...s].length;
}

// ── 머리말/꼬리말 ──────────────────────────────────────────

export class InsertTextInHeaderFooterCommand implements EditCommand {
  readonly type = 'insertTextInHeaderFooter';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: HeaderFooterEditTarget,
    private paraIdx: number,
    private charOffset: number,
    private text: string,
  ) {
    this.lastContext = hfEditContext(target, paraIdx, charOffset + text.length);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.insertTextInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.charOffset, this.text);
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.charOffset + this.text.length);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.deleteTextInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.charOffset, charCount(this.text));
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class DeleteTextInHeaderFooterCommand implements EditCommand {
  readonly type = 'deleteTextInHeaderFooter';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: HeaderFooterEditTarget,
    private paraIdx: number,
    private charOffset: number,
    private deletedText: string,
    /** undo(재삽입) 후 커서 오프셋 — 삭제 방향에 따라 호출부가 정한다(Backspace: charOffset+len, Delete: charOffset). */
    private cursorBeforeOffset: number,
  ) {
    this.lastContext = hfEditContext(target, paraIdx, charOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.deleteTextInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.charOffset, charCount(this.deletedText));
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.insertTextInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.charOffset, this.deletedText);
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.cursorBeforeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class SplitParagraphInHeaderFooterCommand implements EditCommand {
  readonly type = 'splitParagraphInHeaderFooter';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: HeaderFooterEditTarget,
    private paraIdx: number,
    private charOffset: number,
    /** 분할로 생긴 다음 문단 인덱스(인라인 결과의 hfParaIndex). */
    private newParaIdx: number,
  ) {
    this.lastContext = hfEditContext(target, newParaIdx, 0);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.charOffset);
    this.lastContext = hfEditContext(this.target, this.newParaIdx, 0);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.newParaIdx);
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class MergeParagraphInHeaderFooterCommand implements EditCommand {
  readonly type = 'mergeParagraphInHeaderFooter';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: HeaderFooterEditTarget,
    /** 병합되는 문단 M (M 을 M-1 로 합침). */
    private paraIdx: number,
    /** 병합 후 이전 문단 인덱스(인라인 결과 hfParaIndex, = paraIdx-1). */
    private prevParaIdx: number,
    /** 병합 지점 오프셋(이전 문단의 원래 길이, 인라인 결과 charOffset). undo 분할점 + 병합 후 커서. */
    private mergeOffset: number,
    /** 병합 전 커서(undo 복원용) — Backspace: (paraIdx,0), Delete: (prevParaIdx,mergeOffset). */
    private beforeParaIdx: number,
    private beforeOffset: number,
  ) {
    this.lastContext = hfEditContext(target, prevParaIdx, mergeOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx);
    this.lastContext = hfEditContext(this.target, this.prevParaIdx, this.mergeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.prevParaIdx, this.mergeOffset);
    this.lastContext = hfEditContext(this.target, this.beforeParaIdx, this.beforeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

// ── 각주/미주 ──────────────────────────────────────────────

export class InsertTextInFootnoteCommand implements EditCommand {
  readonly type = 'insertTextInFootnote';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: FootnoteEditTarget,
    private innerParaIdx: number,
    private charOffset: number,
    private text: string,
  ) {
    this.lastContext = fnEditContext(target, innerParaIdx, charOffset + text.length);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.insertTextInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx, this.charOffset, this.text);
    this.lastContext = fnEditContext(this.target, this.innerParaIdx, this.charOffset + this.text.length);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.deleteTextInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx, this.charOffset, charCount(this.text));
    this.lastContext = fnEditContext(this.target, this.innerParaIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class DeleteTextInFootnoteCommand implements EditCommand {
  readonly type = 'deleteTextInFootnote';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: FootnoteEditTarget,
    private innerParaIdx: number,
    private charOffset: number,
    private deletedText: string,
    private cursorBeforeOffset: number,
  ) {
    this.lastContext = fnEditContext(target, innerParaIdx, charOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.deleteTextInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx, this.charOffset, charCount(this.deletedText));
    this.lastContext = fnEditContext(this.target, this.innerParaIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.insertTextInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx, this.charOffset, this.deletedText);
    this.lastContext = fnEditContext(this.target, this.innerParaIdx, this.cursorBeforeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class SplitParagraphInFootnoteCommand implements EditCommand {
  readonly type = 'splitParagraphInFootnote';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: FootnoteEditTarget,
    private innerParaIdx: number,
    private charOffset: number,
    private newInnerParaIdx: number,
  ) {
    this.lastContext = fnEditContext(target, newInnerParaIdx, 0);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx, this.charOffset);
    this.lastContext = fnEditContext(this.target, this.newInnerParaIdx, 0);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.newInnerParaIdx);
    this.lastContext = fnEditContext(this.target, this.innerParaIdx, this.charOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

export class MergeParagraphInFootnoteCommand implements EditCommand {
  readonly type = 'mergeParagraphInFootnote';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: FootnoteEditTarget,
    private innerParaIdx: number,
    private prevInnerParaIdx: number,
    private mergeOffset: number,
    /** 병합 전 커서(undo 복원) — Backspace: (innerParaIdx,0), Delete: (prevInnerParaIdx,mergeOffset). */
    private beforeInnerParaIdx: number,
    private beforeOffset: number,
  ) {
    this.lastContext = fnEditContext(target, prevInnerParaIdx, mergeOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx);
    this.lastContext = fnEditContext(this.target, this.prevInnerParaIdx, this.mergeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.prevInnerParaIdx, this.mergeOffset);
    this.lastContext = fnEditContext(this.target, this.beforeInnerParaIdx, this.beforeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  editContext(): EditContext { return this.lastContext; }
  mergeWith(): null { return null; }
}

// ─── 셀 내부 문단 분할 명령 (셀 내 Enter) ──────────────

export class SplitParagraphInCellCommand implements EditCommand {
  readonly type = 'splitParagraphInCell';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      wasm.splitParagraphInCellByPath(sec, ppi, cellPathJson(pos), pos.charOffset);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi, pos.charOffset);
    }
    return cellParagraphPosition(pos, cpi + 1, 0);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      // undo: 분할된 다음 문단을 병합 → cellPath의 cellParaIndex를 +1로 변경
      const undoPath = [...pos.cellPath!];
      undoPath[undoPath.length - 1] = { ...undoPath[undoPath.length - 1], cellParaIndex: cpi + 1 };
      wasm.mergeParagraphInCellByPath(sec, ppi, JSON.stringify(undoPath));
    } else {
      wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi + 1);
    }
    return { ...pos };
  }

  mergeWith(): null { return null; }
}

// ─── 셀 내부 문단 병합 명령 (셀 문단 시작에서 Backspace) ──

export class MergeParagraphInCellCommand implements EditCommand {
  readonly type = 'mergeParagraphInCell';
  readonly timestamp = Date.now();

  private mergePointOffset = 0;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    // 병합 전 이전 셀 문단 길이 기억.
    // flat 필드(controlIndex/cellIndex)는 "외부 표 기준" 레거시 좌표라(types.ts DocumentPosition)
    // 중첩 셀에서는 안쪽 셀을 가리키지 못한다. 뮤테이션이 ByPath 로 분기하는 만큼 길이 조회도
    // 같은 축으로 맞춘다(cursor.ts 의 useCellPath 분기와 동형). 어긋나면 undo 가 바깥 셀에서
    // 읽은 길이로 안쪽 셀을 분할해 문단이 엉뚱한 지점에서 잘린다.
    if (isNestedCell(pos)) {
      const prevPath = pos.cellPath!.map((entry, index, path) =>
        index + 1 === path.length ? { ...entry, cellParaIndex: cpi - 1 } : entry,
      );
      this.mergePointOffset = wasm.getCellParagraphLengthByPath(sec, ppi, JSON.stringify(prevPath));
      wasm.mergeParagraphInCellByPath(sec, ppi, cellPathJson(pos));
    } else {
      this.mergePointOffset = wasm.getCellParagraphLength(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi - 1);
      wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi);
    }
    return cellParagraphPosition(pos, cpi - 1, this.mergePointOffset);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      const undoPath = [...pos.cellPath!];
      undoPath[undoPath.length - 1] = { ...undoPath[undoPath.length - 1], cellParaIndex: cpi - 1 };
      wasm.splitParagraphInCellByPath(sec, ppi, JSON.stringify(undoPath), this.mergePointOffset);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi - 1, this.mergePointOffset);
    }
    return { ...pos };
  }

  mergeWith(): null { return null; }
}

// ─── 셀 내부 다음 문단 병합 명령 (셀 문단 끝에서 Delete) ──

export class MergeNextParagraphInCellCommand implements EditCommand {
  readonly type = 'mergeNextParagraphInCell';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      const nextPath = [...pos.cellPath!];
      nextPath[nextPath.length - 1] = { ...nextPath[nextPath.length - 1], cellParaIndex: cpi + 1 };
      wasm.mergeParagraphInCellByPath(sec, ppi, JSON.stringify(nextPath));
    } else {
      wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi + 1);
    }
    return { ...pos };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      wasm.splitParagraphInCellByPath(sec, ppi, cellPathJson(pos), pos.charOffset);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi, pos.charOffset);
    }
    return { ...pos };
  }

  mergeWith(): null { return null; }
}

// ─── 표 이동 명령 ─────────────────────────────────────

export class MoveTableCommand implements EditCommand {
  readonly type = 'moveTable';
  readonly timestamp: number;

  private resultPpi: number;
  private resultCi: number;

  constructor(
    private sec: number,
    private ppi: number,
    private ci: number,
    private deltaH: number,
    private deltaV: number,
    resultPpi: number,
    resultCi: number,
    timestamp?: number,
  ) {
    this.resultPpi = resultPpi;
    this.resultCi = resultCi;
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const result = wasm.moveTableOffset(this.sec, this.ppi, this.ci, this.deltaH, this.deltaV);
    this.resultPpi = result.ppi;
    this.resultCi = result.ci;
    return { sectionIndex: this.sec, paragraphIndex: this.resultPpi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.moveTableOffset(this.sec, this.resultPpi, this.resultCi, -this.deltaH, -this.deltaV);
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof MoveTableCommand)) return null;
    if (other.sec !== this.sec) return null;
    // 연속 이동: 이전 결과 위치 == 다음 시작 위치
    if (other.ppi !== this.resultPpi || other.ci !== this.resultCi) return null;
    if (other.timestamp - this.timestamp > 500) return null;

    return new MoveTableCommand(
      this.sec, this.ppi, this.ci,
      this.deltaH + other.deltaH,
      this.deltaV + other.deltaV,
      other.resultPpi, other.resultCi,
      this.timestamp,
    );
  }
}

// ─── 그림 이동 명령 ─────────────────────────────────────

/** 두 cellPath 가 동일한지 비교 (undefined/빈배열은 본문(body-level)로 동일 취급) */
function sameCellPath(a?: CellPathLike, b?: CellPathLike): boolean {
  return JSON.stringify(a ?? []) === JSON.stringify(b ?? []);
}

/** 개체 이동 명령용 속성 조회 — cellPath 존재 시 by-path API 로 분기 */
function moveGetProps(
  wasm: WasmBridge, kind: 'image' | 'shape',
  sec: number, ppi: number, ci: number, cellPath?: CellPathLike,
): { horzOffset: number; vertOffset: number } {
  const nested = !!cellPath && cellPath.length > 0;
  if (kind === 'shape') {
    return nested ? wasm.getCellShapePropertiesByPath(sec, ppi, cellPath!, ci) : wasm.getShapeProperties(sec, ppi, ci);
  }
  return nested ? wasm.getCellPicturePropertiesByPath(sec, ppi, cellPath!, ci) : wasm.getPictureProperties(sec, ppi, ci);
}

/** 개체 이동 명령용 속성 변경 — cellPath 존재 시 by-path API 로 분기 */
function moveSetProps(
  wasm: WasmBridge, kind: 'image' | 'shape',
  sec: number, ppi: number, ci: number, cellPath: CellPathLike | undefined,
  props: Record<string, unknown>,
): void {
  const nested = !!cellPath && cellPath.length > 0;
  if (kind === 'shape') {
    if (nested) { wasm.setCellShapePropertiesByPath(sec, ppi, cellPath!, ci, props); return; }
    wasm.setShapeProperties(sec, ppi, ci, props);
    return;
  }
  if (nested) { wasm.setCellPicturePropertiesByPath(sec, ppi, cellPath!, ci, props); return; }
  wasm.setPictureProperties(sec, ppi, ci, props);
}

export class MovePictureCommand implements EditCommand {
  readonly type = 'movePicture';
  readonly timestamp: number;

  constructor(
    private sec: number,
    private ppi: number,
    private ci: number,
    private deltaH: number,
    private deltaV: number,
    private origHorzOffset: number,
    private origVertOffset: number,
    private cellPath?: CellPathLike,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const props = moveGetProps(wasm, 'image', this.sec, this.ppi, this.ci, this.cellPath);
    moveSetProps(wasm, 'image', this.sec, this.ppi, this.ci, this.cellPath, {
      horzOffset: props.horzOffset + this.deltaH,
      vertOffset: props.vertOffset + this.deltaV,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    moveSetProps(wasm, 'image', this.sec, this.ppi, this.ci, this.cellPath, {
      horzOffset: this.origHorzOffset,
      vertOffset: this.origVertOffset,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof MovePictureCommand)) return null;
    if (other.sec !== this.sec || other.ppi !== this.ppi || other.ci !== this.ci) return null;
    if (!sameCellPath(other.cellPath, this.cellPath)) return null;
    if (other.timestamp - this.timestamp > 500) return null;

    return new MovePictureCommand(
      this.sec, this.ppi, this.ci,
      this.deltaH + other.deltaH,
      this.deltaV + other.deltaV,
      this.origHorzOffset,
      this.origVertOffset,
      this.cellPath,
      this.timestamp,
    );
  }
}

export class MoveShapeCommand implements EditCommand {
  readonly type = 'moveShape';
  readonly timestamp: number;

  constructor(
    private sec: number,
    private ppi: number,
    private ci: number,
    private deltaH: number,
    private deltaV: number,
    private origHorzOffset: number,
    private origVertOffset: number,
    private cellPath?: CellPathLike,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const props = moveGetProps(wasm, 'shape', this.sec, this.ppi, this.ci, this.cellPath);
    moveSetProps(wasm, 'shape', this.sec, this.ppi, this.ci, this.cellPath, {
      horzOffset: props.horzOffset + this.deltaH,
      vertOffset: props.vertOffset + this.deltaV,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    moveSetProps(wasm, 'shape', this.sec, this.ppi, this.ci, this.cellPath, {
      horzOffset: this.origHorzOffset,
      vertOffset: this.origVertOffset,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof MoveShapeCommand)) return null;
    if (other.sec !== this.sec || other.ppi !== this.ppi || other.ci !== this.ci) return null;
    if (!sameCellPath(other.cellPath, this.cellPath)) return null;
    if (other.timestamp - this.timestamp > 500) return null;

    return new MoveShapeCommand(
      this.sec, this.ppi, this.ci,
      this.deltaH + other.deltaH,
      this.deltaV + other.deltaV,
      this.origHorzOffset,
      this.origVertOffset,
      this.cellPath,
      this.timestamp,
    );
  }
}


// ─── 개체 크기/위치 속성 변경 명령 ─────────────────────

export type ObjectResizeTarget = {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellPath?: CellPathLike;
  before: Record<string, unknown>;
  after: Record<string, unknown>;
};

/**
 * 그림/도형 리사이즈처럼 드래그 중 WASM에 이미 반영된 속성 변경을
 * Undo/Redo 스택에 기록하기 위한 명령.
 */
export class ResizeObjectCommand implements EditCommand {
  readonly type = 'resizeObject';
  readonly timestamp: number;

  constructor(
    private targets: ObjectResizeTarget[],
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  private setProps(wasm: WasmBridge, target: ObjectResizeTarget, props: Record<string, unknown>): void {
    if (target.type === 'shape' || target.type === 'line' || target.type === 'group' || target.type === 'ole') {
      if (target.cellPath && target.cellPath.length > 0) {
        wasm.setCellShapePropertiesByPath(target.sec, target.ppi, target.cellPath, target.ci, props);
        return;
      }
      wasm.setShapeProperties(target.sec, target.ppi, target.ci, props);
    } else {
      if (target.type === 'image' && target.cellPath && target.cellPath.length > 0) {
        wasm.setCellPicturePropertiesByPath(target.sec, target.ppi, target.cellPath, target.ci, props);
        return;
      }
      wasm.setPictureProperties(target.sec, target.ppi, target.ci, props);
    }
  }

  execute(wasm: WasmBridge): DocumentPosition {
    for (const target of this.targets) {
      this.setProps(wasm, target, target.after);
    }
    const first = this.targets[0];
    return { sectionIndex: first?.sec ?? 0, paragraphIndex: first?.ppi ?? 0, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    for (const target of this.targets) {
      this.setProps(wasm, target, target.before);
    }
    const first = this.targets[0];
    return { sectionIndex: first?.sec ?? 0, paragraphIndex: first?.ppi ?? 0, charOffset: 0 };
  }

  mergeWith(): null { return null; }
}

/**
 * [Task #2374] 양식 값 변경 대상 — 본문 또는 표 셀 내 컨트롤 locator + 전/후 값 JSON.
 * before/after 는 setFormValue(InCell) 에 그대로 전달되는 JSON 문자열이다.
 */
export interface FormValueTarget {
  sec: number;
  para: number;
  ci: number;
  inCell?: { tablePara: number; tableCi: number; cellIdx: number; cellPara: number };
  beforeJson: string;
  afterJson: string;
}

/**
 * [Task #2374] 양식 컨트롤 값 변경의 경량 역연산 명령 (kind:'record' 용, #2337 계열).
 *
 * 뮤테이션은 클릭 핸들러가 직접 적용하고 이 명령은 기록만 담당한다(재실행 안 함).
 * 라디오 버튼처럼 다중 쓰기(그룹 해제 + 선택)인 조작은 targets 배열로 묶어 undo/redo 가
 * 그룹 상태를 원자적으로 왕복하게 한다 — 양식 모드에서는 snapshot 이 게이트에서 드롭되므로
 * record 가 유일한 기록 경로다.
 */
export class SetFormValueCommand implements EditCommand {
  readonly type = 'setFormValue';
  readonly timestamp: number;

  constructor(
    private targets: FormValueTarget[],
    private pos: DocumentPosition,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  private apply(wasm: WasmBridge, t: FormValueTarget, json: string): void {
    if (t.inCell) {
      wasm.setFormValueInCell(t.sec, t.inCell.tablePara, t.inCell.tableCi, t.inCell.cellIdx, t.inCell.cellPara, t.ci, json);
    } else {
      wasm.setFormValue(t.sec, t.para, t.ci, json);
    }
  }

  execute(wasm: WasmBridge): DocumentPosition {
    for (const t of this.targets) this.apply(wasm, t, t.afterJson);
    return this.pos;
  }

  undo(wasm: WasmBridge): DocumentPosition {
    for (let i = this.targets.length - 1; i >= 0; i--) this.apply(wasm, this.targets[i], this.targets[i].beforeJson);
    return this.pos;
  }

  mergeWith(): null { return null; }
}

// ─── 스냅샷 기반 명령 (복잡한 작업의 Undo/Redo) ─────

/**
 * Document 스냅샷을 이용한 Undo/Redo 명령.
 *
 * 역연산 구현이 복잡한 작업(붙여넣기, 객체 삭제 등)에 사용한다.
 * - 최초 실행: before 스냅샷 저장 → 작업 수행 → after 스냅샷 저장
 * - Undo: before 스냅샷으로 복원
 * - Redo: after 스냅샷으로 복원
 * - Discard: 양쪽 스냅샷 메모리 해제
 */
export class SnapshotCommand implements EditCommand {
  readonly type: string;
  readonly timestamp = Date.now();

  private beforeId: number | null = null;
  private afterId: number | null = null;

  /**
   * @param operationType 작업 종류 (예: 'pasteInternal', 'deleteControl')
   * @param cursorBefore 작업 전 커서 위치
   * @param operation 실제 작업을 수행하는 함수. 작업 후 커서 위치를 반환.
   */
  constructor(
    operationType: string,
    private cursorBefore: DocumentPosition,
    private cursorAfter: DocumentPosition,
    private operation: ((wasm: WasmBridge) => DocumentPosition) | null,
  ) {
    this.type = `snapshot:${operationType}`;
  }

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.afterId !== null) {
      // Redo: after 스냅샷으로 복원
      wasm.restoreSnapshot(this.afterId);
      return { ...this.cursorAfter };
    }

    // 최초 실행: before 저장 → 작업 수행 → after 저장
    this.beforeId = wasm.saveSnapshot();
    // [Task #2328] operation 또는 after-save 중 어느 것이 throw 하든 커맨드가
    // 히스토리에 등록되지 못해 discard 주체가 사라진다 → 스냅샷 영구 누수(orphan
    // → WASM 무통보 축출 재발). after-save(대용량 문서 클론 시 메모리 압박 등)까지
    // try 범위에 포함해 before/after 를 대칭적으로 해제한다.
    try {
      if (this.operation) {
        this.cursorAfter = this.operation(wasm);
      }
      this.afterId = wasm.saveSnapshot();
    } catch (e) {
      this.discard(wasm); // before/after id 를 null-safe 로 해제
      throw e;
    }

    // operation 참조 해제 (클로저에 캡처된 리소스 해제)
    this.operation = null;

    return { ...this.cursorAfter };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    if (this.beforeId !== null) {
      wasm.restoreSnapshot(this.beforeId);
    }
    return { ...this.cursorBefore };
  }

  mergeWith(): null { return null; }

  /** [Task #2328] 현재 살아있는 before/after 스냅샷 id 개수. */
  snapshotResourceCount(): number {
    return (this.beforeId !== null ? 1 : 0) + (this.afterId !== null ? 1 : 0);
  }

  discard(wasm: WasmBridge): void {
    if (this.beforeId !== null) {
      wasm.discardSnapshot(this.beforeId);
      this.beforeId = null;
    }
    if (this.afterId !== null) {
      wasm.discardSnapshot(this.afterId);
      this.afterId = null;
    }
  }
}
