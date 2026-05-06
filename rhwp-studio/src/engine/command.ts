import type { WasmBridge } from '@/core/wasm-bridge';
import type { DocumentPosition, CharProperties } from '@/core/types';

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
}

// ─── 편집 작업 서술자 (라우팅 통합) ────────────────────

/**
 * 편집 작업 서술자 — 호출부가 "무엇을 하려는가"만 기술하고,
 * 라우터(executeOperation)가 적절한 Undo 전략을 자동 선택한다.
 *
 * - command: 정밀 커맨드 (텍스트 삽입/삭제, 문단 분할/병합, 서식)
 * - snapshot: 스냅샷 기반 커맨드 (붙여넣기, 객체 삭제 등)
 * - record:  WASM 직접 호출 후 히스토리에만 기록 (IME, 객체 이동)
 */
export type OperationDescriptor =
  | { kind: 'command'; command: EditCommand }
  | { kind: 'snapshot'; operationType: string; operation: (wasm: WasmBridge) => DocumentPosition }
  | { kind: 'record'; command: EditCommand };

// ─── 본문/셀 분기 헬퍼 ────────────────────────────────

function isCell(pos: DocumentPosition): boolean {
  return pos.parentParaIndex !== undefined;
}

/** 중첩 표(depth > 1)인지 확인 */
function isNestedCell(pos: DocumentPosition): boolean {
  return (pos.cellPath?.length ?? 0) > 1;
}

/** cellPath를 WASM용 JSON 문자열로 변환 */
function cellPathJson(pos: DocumentPosition): string {
  return JSON.stringify(pos.cellPath ?? []);
}

function doInsertText(wasm: WasmBridge, pos: DocumentPosition, text: string): void {
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

  constructor(
    private position: DocumentPosition,
    private text: string,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    doInsertText(wasm, this.position, this.text);
    return { ...this.position, charOffset: this.position.charOffset + this.text.length };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    doDeleteText(wasm, this.position, this.text.length);
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
    // 삭제 전 텍스트 보존
    if (!this.deletedText) {
      this.deletedText = doGetTextRange(wasm, this.position, this.count);
    }
    doDeleteText(wasm, this.position, this.count);
    return { ...this.position };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    doInsertText(wasm, this.position, this.deletedText);
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
    doInsertText(wasm, this.position, '\n');
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
    doInsertText(wasm, this.position, '\t');
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
      const sec = start.sectionIndex;
      const ppi = start.parentParaIndex!;
      const ci = start.controlIndex!;
      const cei = start.cellIndex!;
      const startPara = start.cellParaIndex!;
      const endPara = end.cellParaIndex!;
      this.multiPara = startPara !== endPara;
      for (let p = startPara; p <= endPara; p++) {
        const pLen = wasm.getCellParagraphLength(sec, ppi, ci, cei, p);
        const from = p === startPara ? start.charOffset : 0;
        const to = p === endPara ? end.charOffset : pLen;
        if (to > from) {
          this.savedTexts.push(wasm.getTextInCell(sec, ppi, ci, cei, p, from, to - from));
        } else {
          this.savedTexts.push('');
        }
      }
      wasm.deleteRangeInCell(sec, ppi, ci, cei, startPara, start.charOffset, endPara, end.charOffset);
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
      if (text) doInsertText(wasm, start, text);
    } else {
      // 다중 문단: 마지막 텍스트부터 역순으로 복원
      // 1) 첫 문단 뒷부분 텍스트 삽입
      const firstText = this.savedTexts[0] || '';
      if (firstText) doInsertText(wasm, start, firstText);

      // 2) 중간 문단 + 마지막 문단: splitParagraph로 분리 후 텍스트 삽입
      if (isCell(start)) {
        // 셀 내 다중 문단 undo는 복잡하므로 단순 텍스트 삽입으로 대체
        // (셀 내 다중 문단 선택 삭제의 undo는 한계가 있음)
        for (let i = 1; i < this.savedTexts.length; i++) {
          const text = this.savedTexts[i];
          if (text || i < this.savedTexts.length - 1) {
            // splitParagraph는 셀 내에서 미지원 → 텍스트만 이어붙이기
            const restorePos = { ...start, charOffset: start.charOffset + firstText.length };
            if (text) doInsertText(wasm, restorePos, '\n' + text);
          }
        }
      } else {
        const sec = start.sectionIndex;
        let currentPara = start.paragraphIndex;
        let currentOffset = start.charOffset + firstText.length;
        for (let i = 1; i < this.savedTexts.length; i++) {
          wasm.splitParagraph(sec, currentPara, currentOffset);
          currentPara++;
          currentOffset = 0;
          const text = this.savedTexts[i];
          if (text) {
            wasm.insertText(sec, currentPara, 0, text);
          }
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
  prevCharShapeId?: number;
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
    const { start, end } = this;
    const propsJson = JSON.stringify(this.props);

    if (isCell(start)) {
      const sec = start.sectionIndex;
      const ppi = start.parentParaIndex!;
      const ci = start.controlIndex!;
      const cei = start.cellIndex!;
      const startPara = start.cellParaIndex!;
      const endPara = end.cellParaIndex!;

      this.entries = [];
      for (let p = startPara; p <= endPara; p++) {
        const from = p === startPara ? start.charOffset : 0;
        const to = p === endPara ? end.charOffset : wasm.getCellParagraphLength(sec, ppi, ci, cei, p);
        if (to <= from) continue;

        // undo용 이전 서식 저장
        const prevProps = wasm.getCellCharPropertiesAt(sec, ppi, ci, cei, p, from);
        this.entries.push({ paraIndex: p, startOffset: from, endOffset: to, prevCharShapeId: prevProps.charShapeId });

        wasm.applyCharFormatInCell(sec, ppi, ci, cei, p, from, to, propsJson);
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
        this.entries.push({ paraIndex: p, startOffset: from, endOffset: to, prevCharShapeId: prevProps.charShapeId });

        wasm.applyCharFormat(sec, p, from, to, propsJson);
      }
    }

    return { ...this.start };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { start } = this;

    // 이전 charShapeId로 복원
    for (const entry of this.entries) {
      if (entry.prevCharShapeId === undefined) continue;
      const restoreJson = JSON.stringify({ charShapeId: entry.prevCharShapeId });

      if (isCell(start)) {
        wasm.applyCharFormatInCell(
          start.sectionIndex, start.parentParaIndex!, start.controlIndex!, start.cellIndex!,
          entry.paraIndex, entry.startOffset, entry.endOffset, restoreJson,
        );
      } else {
        wasm.applyCharFormat(start.sectionIndex, entry.paraIndex, entry.startOffset, entry.endOffset, restoreJson);
      }
    }

    return { ...this.start };
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

// ─── 셀 내부 문단 분할 명령 (셀 내 Enter) ──────────────

export class SplitParagraphInCellCommand implements EditCommand {
  readonly type = 'splitParagraphInCell';
  readonly timestamp = Date.now();

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = pos.cellParaIndex!;
    if (isNestedCell(pos)) {
      wasm.splitParagraphInCellByPath(sec, ppi, cellPathJson(pos), pos.charOffset);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi, pos.charOffset);
    }
    return {
      ...pos,
      cellParaIndex: cpi + 1,
      charOffset: 0,
    };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = pos.cellParaIndex!;
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
    const cpi = pos.cellParaIndex!;
    // 병합 전 이전 셀 문단 길이 기억
    this.mergePointOffset = wasm.getCellParagraphLength(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi - 1);
    if (isNestedCell(pos)) {
      wasm.mergeParagraphInCellByPath(sec, ppi, cellPathJson(pos));
    } else {
      wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi);
    }
    return {
      ...pos,
      cellParaIndex: cpi - 1,
      charOffset: this.mergePointOffset,
    };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = pos.cellParaIndex!;
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
    const cpi = pos.cellParaIndex!;
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
    const cpi = pos.cellParaIndex!;
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
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const props = wasm.getPictureProperties(this.sec, this.ppi, this.ci);
    wasm.setPictureProperties(this.sec, this.ppi, this.ci, {
      horzOffset: ((props.horzOffset + this.deltaH) >>> 0),
      vertOffset: ((props.vertOffset + this.deltaV) >>> 0),
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.setPictureProperties(this.sec, this.ppi, this.ci, {
      horzOffset: this.origHorzOffset,
      vertOffset: this.origVertOffset,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof MovePictureCommand)) return null;
    if (other.sec !== this.sec || other.ppi !== this.ppi || other.ci !== this.ci) return null;
    if (other.timestamp - this.timestamp > 500) return null;

    return new MovePictureCommand(
      this.sec, this.ppi, this.ci,
      this.deltaH + other.deltaH,
      this.deltaV + other.deltaV,
      this.origHorzOffset,
      this.origVertOffset,
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
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const props = wasm.getShapeProperties(this.sec, this.ppi, this.ci);
    wasm.setShapeProperties(this.sec, this.ppi, this.ci, {
      horzOffset: ((props.horzOffset + this.deltaH) >>> 0),
      vertOffset: ((props.vertOffset + this.deltaV) >>> 0),
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.setShapeProperties(this.sec, this.ppi, this.ci, {
      horzOffset: this.origHorzOffset,
      vertOffset: this.origVertOffset,
    });
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(other: EditCommand): EditCommand | null {
    if (!(other instanceof MoveShapeCommand)) return null;
    if (other.sec !== this.sec || other.ppi !== this.ppi || other.ci !== this.ci) return null;
    if (other.timestamp - this.timestamp > 500) return null;

    return new MoveShapeCommand(
      this.sec, this.ppi, this.ci,
      this.deltaH + other.deltaH,
      this.deltaV + other.deltaV,
      this.origHorzOffset,
      this.origVertOffset,
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
    if (target.type === 'shape' || target.type === 'line' || target.type === 'group') {
      wasm.setShapeProperties(target.sec, target.ppi, target.ci, props);
    } else {
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
    if (this.operation) {
      this.cursorAfter = this.operation(wasm);
    }
    this.afterId = wasm.saveSnapshot();

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
