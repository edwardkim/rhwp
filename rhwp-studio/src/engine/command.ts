import type {
  DeferredCellTextMutationResult,
  DeferredFocusedPagePatch,
  RemovedParaMeta,
  WasmBridge,
} from '@/core/wasm-bridge';
import type { DocumentPosition, CharProperties, ParaProperties, CellPathLike, CellPathEntry, SectionDef } from '@/core/types';
import { MAX_PAGE_LOCAL_TEXT_EDIT_CHARS } from './input-edit-invalidation';
import type { LineEndpoints as LineEndpointsLike } from './object-drag-record';
import { setObjectProps, type ObjectPropsRef } from './object-props';

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
  /**
   * [Task #2370 클러스터 A] execute() 가 문서를 전혀 바꾸지 않았는가.
   * true 면 CommandHistory 가 이 명령을 undo 스택에 넣지 않는다 — 되돌릴 것이 없는
   * 엔트리는 Ctrl+Z 를 무효과로 소모하고, redo 스택을 파기하며(`execute` 는 새 명령마다
   * redo 를 버린다), 스냅샷 명령이면 예산 2슬롯을 점유해 진짜 undo 이력을 축출한다.
   * 구현하지 않으면 종전대로 항상 기록된다.
   */
  isNoOp?(): boolean;
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
  /**
   * [Task #3416] 이 명령이 실행되기 **직전의 선택 범위**. undo 후 그 선택을 되살리는 데 쓴다.
   *
   * 한컴 2024 실측: 선택을 지운 뒤 undo 하면 지우기 전 범위가 그대로 복원되고(캐럿은 선택 끝),
   * redo 하면 해제된다. 반면 선택 위에 타이핑해서 대체한 경우의 undo 는 복원하지 않는다.
   * F3 블록 선택은 **확장 단계까지** 되돌아온다 — undo 뒤 F3 를 누르면 단어에서 문단으로
   * 이어서 확장한다(단계가 초기화됐다면 다시 단어 범위에 머물렀을 것이다).
   * 그래서 이것은 **선택 삭제 계열만 구현한다** — 미구현이면 종전대로 해제된다.
   *
   * 반환값은 "그때 그랬다" 는 기록일 뿐 지금 유효하다는 보장이 아니다. 복원하는 쪽이 현재
   * 문서에서 유효한지 반드시 확인해야 한다(#2339).
   */
  selectionBefore?(): {
    start: DocumentPosition;
    end: DocumentPosition;
    /** F3 블록 선택이었으면 그 확장 단계, 아니면 `null` (실측: 한컴은 단계까지 되돌린다). */
    blockPhase: number | null;
  } | null;
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

/** text mutation의 document pagination/flow 경계와 immediate 완료를 함께 전달한다. */
export interface FocusedCellCursorGeometry {
  readonly baseRevision: number;
  readonly revision: number;
  readonly source: DocumentPosition;
  readonly target: DocumentPosition;
  readonly deltaX: number;
}

export interface TextMutationEffects {
  readonly documentPaginationPending: boolean;
  readonly flowChanged: boolean;
  readonly paginationCompleted: boolean;
  readonly focusedCursorGeometry?: FocusedCellCursorGeometry;
  readonly focusedPagePatch?: DeferredFocusedPagePatch;
}

export const NO_TEXT_MUTATION_EFFECTS: TextMutationEffects = Object.freeze({
  documentPaginationPending: false,
  flowChanged: false,
  paginationCompleted: false,
});

export const IMMEDIATE_TEXT_MUTATION_EFFECTS: TextMutationEffects = Object.freeze({
  documentPaginationPending: false,
  flowChanged: false,
  paginationCompleted: true,
});

/** raw IME/iOS 묶음에서 effect를 OR 누적하고 한 번만 소비한다. */
export class TextMutationEffectAccumulator {
  private effects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  add(effects: TextMutationEffects): void {
    const accumulatedMutation = this.effects.documentPaginationPending
      || this.effects.flowChanged
      || this.effects.paginationCompleted;
    const incomingMutation = effects.documentPaginationPending
      || effects.flowChanged
      || effects.paginationCompleted;
    // 두 mutation을 한 번에 묶으면 중간 source rect를 보장할 수 없다. 단일 mutation이거나
    // 앞뒤가 NO effect인 경우에만 focused geometry를 전달한다.
    const focusedCursorGeometry = accumulatedMutation
      ? (incomingMutation ? undefined : this.effects.focusedCursorGeometry)
      : (incomingMutation ? effects.focusedCursorGeometry : undefined);
    const focusedPagePatch = accumulatedMutation
      ? (
          incomingMutation
            ? mergeFocusedPagePatches(this.effects.focusedPagePatch, effects.focusedPagePatch)
            : this.effects.focusedPagePatch
        )
      : (incomingMutation ? effects.focusedPagePatch : undefined);
    this.effects = {
      documentPaginationPending:
        this.effects.documentPaginationPending || effects.documentPaginationPending,
      flowChanged: this.effects.flowChanged || effects.flowChanged,
      paginationCompleted: this.effects.paginationCompleted || effects.paginationCompleted,
      ...(focusedCursorGeometry ? { focusedCursorGeometry } : {}),
      ...(focusedPagePatch ? { focusedPagePatch } : {}),
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

function mergeFocusedPagePatches(
  first: DeferredFocusedPagePatch | undefined,
  second: DeferredFocusedPagePatch | undefined,
): DeferredFocusedPagePatch | undefined {
  if (!first || !second || first.pageIndex !== second.pageIndex) return undefined;
  const x = Math.min(first.x, second.x);
  const y = Math.min(first.y, second.y);
  const right = Math.max(first.x + first.width, second.x + second.width);
  const bottom = Math.max(first.y + first.height, second.y + second.height);
  return {
    pageIndex: first.pageIndex,
    x,
    y,
    width: right - x,
    height: bottom - y,
  };
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
  // [Task #2370] snapshot 의 operation 은 아무것도 바꾸지 않았을 때 `null` 을 반환해
  // "기록하지 말 것"을 알린다(그 경우 커서 이동·리프레시도 건너뛴다).
  | {
      kind: 'snapshot';
      operationType: string;
      operation: (wasm: WasmBridge) => DocumentPosition | null;
      /** 본문 좌표와 분리된 HF/FN 편집 문맥. undo/redo 뒤 같은 문맥으로 돌아간다. */
      editContext?: EditContext;
      meta?: OperationMetadata;
    }
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

export function canUseDeferredCellTextDelete(pos: DocumentPosition, count: number): boolean {
  if (!isCell(pos) || isNestedCell(pos)) return false;
  return Number.isInteger(count) && count > 0 && count <= MAX_PAGE_LOCAL_TEXT_EDIT_CHARS;
}

export function canUseDeferredCellTextReplace(
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): boolean {
  if (!isCell(pos) || isNestedCell(pos)) return false;
  if (
    !Number.isInteger(deleteCount) ||
    deleteCount < 1 ||
    deleteCount > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS
  ) {
    return false;
  }
  const textChars = charCount(text);
  if (textChars < 1 || textChars > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
  if (/[\r\n\t]/.test(text)) return false;
  return true;
}

export function canUseLocalBodyTextReplace(
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): boolean {
  if (isCell(pos)) return false;
  if (!Number.isInteger(deleteCount) || deleteCount < 0 || deleteCount > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) {
    return false;
  }
  if (deleteCount === 0 && text.length === 0) return false;
  if (charCount(text) > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
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
 *
 * [#2717] 셀 문단 경계 판정(호출자 가드)도 같은 축이어야 해서 export 한다 —
 * 축 유도를 복제하면 한쪽만 고쳐지는 회귀가 재발한다
 * (tests/undo-nested-cell-merge-offset.test.ts).
 */
export function cellParaIndexOf(pos: DocumentPosition): number {
  const path = pos.cellPath;
  return (path?.length ?? 0) > 0 ? path![path!.length - 1].cellParaIndex : pos.cellParaIndex!;
}

/**
 * [#2756] 비교용 셀 경로 — `cellParaIndexOf` 와 같은 축 규약의 경로 전체 버전.
 *
 * `cellParaIndexOf` 가 최내곽 **문단 인덱스** 하나를 주는 것과 달리, 이쪽은 셀 **정체성**을
 * 깊이별로 비교해야 하는 곳(선택 영역 정렬)에 쓴다. 두 함수는 같은 규약을 공유한다 —
 * `cellParaIndexOf(pos) === cellAxisPath(pos)[last].cellParaIndex`.
 *
 * `cellPath` 가 없는 위치(레거시 flat 좌표, `applyNavResult` 산출물)는 flat 필드로 1-depth
 * 경로를 합성한다. hit-test 가 flat 을 `cellPath[0]`(최외곽)에서 채우므로, depth 1 에서는
 * 합성 경로가 실제 경로와 완전히 같아 **동작 변화가 없다**.
 */
export function cellAxisPath(pos: DocumentPosition): CellPathEntry[] {
  if ((pos.cellPath?.length ?? 0) > 0) return pos.cellPath!;
  return [{
    controlIndex: pos.controlIndex ?? 0,
    cellIndex: pos.cellIndex ?? 0,
    cellParaIndex: pos.cellParaIndex ?? 0,
  }];
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

function focusedCellCursorGeometryFromResult(
  pos: DocumentPosition,
  result: DeferredCellTextMutationResult,
): FocusedCellCursorGeometry | undefined {
  const geometry = result.focusedCursorGeometry;
  if (
    !result.paginationDeferred
    || result.cellFlowChanged
    || !geometry
    || geometry.targetCharOffset !== result.charOffset
  ) {
    return undefined;
  }
  const cloneAt = (charOffset: number): DocumentPosition => ({
    ...pos,
    charOffset,
    cellPath: pos.cellPath?.map((entry) => ({ ...entry })),
    cursorRect: undefined,
  });
  return {
    baseRevision: geometry.baseRevision,
    revision: geometry.revision,
    source: cloneAt(geometry.sourceCharOffset),
    target: cloneAt(geometry.targetCharOffset),
    deltaX: geometry.deltaX,
  };
}

function focusedPagePatchFromResult(
  result: DeferredCellTextMutationResult,
): DeferredFocusedPagePatch | undefined {
  if (
    !result.paginationDeferred
    || result.cellFlowChanged
    || !result.focusedPageTreePatched
    || !result.focusedPagePatch
  ) {
    return undefined;
  }
  return { ...result.focusedPagePatch };
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
      const focusedCursorGeometry = focusedCellCursorGeometryFromResult(pos, result);
      const focusedPagePatch = focusedPagePatchFromResult(result);
      return {
        documentPaginationPending: result.paginationDeferred,
        flowChanged: result.cellFlowChanged,
        paginationCompleted: !result.paginationDeferred,
        ...(focusedCursorGeometry ? { focusedCursorGeometry } : {}),
        ...(focusedPagePatch ? { focusedPagePatch } : {}),
      };
    } else {
      wasm.insertTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, text);
    }
  } else if (canUseLocalBodyTextReplace(pos, 0, text)) {
    return replaceBodyTextWithMutationEffects(wasm, pos, 0, text);
  } else {
    wasm.insertText(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, text);
  }
  return IMMEDIATE_TEXT_MUTATION_EFFECTS;
}

export function replaceBodyTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects {
  const result = wasm.replaceBodyTextLocal(
    pos.sectionIndex,
    pos.paragraphIndex,
    pos.charOffset,
    deleteCount,
    text,
  );
  return {
    documentPaginationPending: result.documentPaginationPending,
    flowChanged: result.flowChanged,
    paginationCompleted: !result.documentPaginationPending,
  };
}

export function replaceCellTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  deleteCount: number,
  text: string,
): TextMutationEffects {
  const result = wasm.replaceTextInCellDeferredPagination(
    pos.sectionIndex,
    pos.parentParaIndex!,
    pos.controlIndex!,
    pos.cellIndex!,
    pos.cellParaIndex!,
    pos.charOffset,
    deleteCount,
    text,
  );
  const focusedCursorGeometry = focusedCellCursorGeometryFromResult(pos, result);
  const focusedPagePatch = focusedPagePatchFromResult(result);
  return {
    documentPaginationPending: result.paginationDeferred,
    flowChanged: result.paginationDeferred && result.cellFlowChanged,
    paginationCompleted: !result.paginationDeferred,
    ...(focusedCursorGeometry ? { focusedCursorGeometry } : {}),
    ...(focusedPagePatch ? { focusedPagePatch } : {}),
  };
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

export function deleteTextWithMutationEffects(
  wasm: WasmBridge,
  pos: DocumentPosition,
  count: number,
): TextMutationEffects {
  if (isNestedCell(pos)) {
    wasm.deleteTextInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), pos.charOffset, count);
  } else if (isCell(pos)) {
    if (canUseDeferredCellTextDelete(pos, count)) {
      const result = wasm.deleteTextInCellDeferredPagination(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, count);
      const focusedCursorGeometry = focusedCellCursorGeometryFromResult(pos, result);
      const focusedPagePatch = focusedPagePatchFromResult(result);
      return {
        documentPaginationPending: result.paginationDeferred,
        flowChanged: result.cellFlowChanged,
        paginationCompleted: !result.paginationDeferred,
        ...(focusedCursorGeometry ? { focusedCursorGeometry } : {}),
        ...(focusedPagePatch ? { focusedPagePatch } : {}),
      };
    }
    wasm.deleteTextInCell(pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!, pos.cellIndex!, pos.cellParaIndex!, pos.charOffset, count);
  } else if (canUseLocalBodyTextReplace(pos, count, '')) {
    return replaceBodyTextWithMutationEffects(wasm, pos, count, '');
  } else {
    wasm.deleteText(pos.sectionIndex, pos.paragraphIndex, pos.charOffset, count);
  }
  return IMMEDIATE_TEXT_MUTATION_EFFECTS;
}

function doDeleteTextImmediate(wasm: WasmBridge, pos: DocumentPosition, count: number): void {
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

/**
 * [#4162] 캐럿 대기 글자 모양(pending char shape) — 방금 삽입된 range 에 글자 서식을 건다.
 *
 * ApplyCharFormatCommand.execute() 의 셀/본문 분기와 같은 축이다(셀은 항상 ...ByPath).
 * from === to(빈 range)면 적용 대상이 없으므로 아무것도 하지 않는다.
 */
export function applyCharShapeModsToRange(
  wasm: WasmBridge,
  pos: DocumentPosition,
  from: number,
  to: number,
  props: Partial<CharProperties>,
): void {
  if (to <= from) return;
  const propsJson = JSON.stringify(props);
  if (isCell(pos)) {
    wasm.applyCharFormatInCellByPath(pos.sectionIndex, pos.parentParaIndex!, cellPathJson(pos), from, to, propsJson);
  } else {
    wasm.applyCharFormat(pos.sectionIndex, pos.paragraphIndex, from, to, propsJson);
  }
}

function sameCharFormat(a: Partial<CharProperties> | undefined, b: Partial<CharProperties> | undefined): boolean {
  return JSON.stringify(a ?? null) === JSON.stringify(b ?? null);
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
    /** [#4162] 선택 없이 지정한 예약 글자 모양 — 삽입된 텍스트에 그대로 건다. */
    private charFormat?: Partial<CharProperties>,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  getCharFormat(): Partial<CharProperties> | undefined {
    return this.charFormat;
  }

  execute(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    this.lastMutationEffects = insertTextWithMutationEffects(wasm, this.position, this.text);
    const after = { ...this.position, charOffset: this.position.charOffset + this.text.length };
    if (this.charFormat) {
      applyCharShapeModsToRange(wasm, this.position, this.position.charOffset, after.charOffset, this.charFormat);
    }
    return after;
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
    doDeleteTextImmediate(wasm, this.position, charCount(this.text));
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
    // [#4162] 예약 글자 모양이 다르면 하나의 undo 단위로 묶지 않는다
    if (!sameCharFormat(this.charFormat, other.charFormat)) return null;

    return new InsertTextCommand(this.position, this.text + other.text, this.timestamp, this.charFormat);
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
    this.lastMutationEffects = deleteTextWithMutationEffects(wasm, this.position, this.count);
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
    doDeleteTextImmediate(wasm, this.position, 1);
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
    doDeleteTextImmediate(wasm, this.position, 1);
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
  /** 병합으로 사라진 문단의 스코프 메타데이터 — undo 분할이 되돌린다 (Task #2342) */
  private removedParaMeta?: RemovedParaMeta;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    // 병합 전 이전 문단 길이 기억
    this.mergePointOffset = wasm.getParagraphLength(sec, para - 1);
    this.removedParaMeta = JSON.parse(wasm.mergeParagraph(sec, para)).removedParaMeta;
    return { sectionIndex: sec, paragraphIndex: para - 1, charOffset: this.mergePointOffset };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    wasm.splitParagraph(sec, para - 1, this.mergePointOffset, this.removedParaMeta);
    return { ...this.position };
  }

  mergeWith(): null { return null; }
}

// ─── [#5769] 선택 영역 삭제 — 조각(fragment) 경로 ───────

/**
 * 스냅샷 대신 조각 저장소를 쓰는 선택 삭제 명령.
 *
 * captureDeleteRange → deleteRange → (undo: restoreDeleteFragment) 순서로,
 * 스냅샷 2슬롯 대신 조각 1개(문단 클론 + 꼬리 line_segs + raw + 캐럿)로
 * 역연산한다. snapshotResourceCount 는 항상 0 — 스냅샷 예산에 기여하지 않는다.
 *
 * redo: undo 가 조각을 소비하므로 문서를 다시 캡처해 삭제한다.
 * 셀 내 삭제는 조각 API 미지원이라 DeleteSelectionCommand 생성자에서
 * SnapshotCommand 경로로 폴백한다.
 */
export class FragmentDeleteCommand implements EditCommand {
  readonly type = 'deleteSelection';
  readonly timestamp = Date.now();

  private fragmentId: number | null = null;
  private noOp = false;

  constructor(
    private cursorBefore: DocumentPosition,
    private cursorAfter: DocumentPosition,
    private selection: { start: DocumentPosition; end: DocumentPosition; blockPhase: number | null },
    private sectionIdx: number,
    private startPara: number,
    private endPara: number,
    private operation: (wasm: WasmBridge) => DocumentPosition | null,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    // 지역 변수로 받는다 — catch 에서 프로퍼티 좁히기가 풀려 TS2345 가 난다(CI 실측).
    const fragmentId = wasm.captureDeleteRange(this.sectionIdx, this.startPara, this.endPara);
    this.fragmentId = fragmentId;
    try {
      const result = this.operation(wasm);
      if (result === null) {
        this.noOp = true;
        wasm.discardDeleteFragment(fragmentId);
        this.fragmentId = null;
        return { ...this.cursorBefore };
      }
      return result;
    } catch (e) {
      wasm.discardDeleteFragment(fragmentId);
      this.fragmentId = null;
      throw e;
    }
  }

  undo(wasm: WasmBridge): DocumentPosition {
    if (this.fragmentId === null) {
      // 실행 전·이미 복원된 뒤의 undo 는 배선 버그다 — 무음 통과 대신 드러낸다.
      throw new Error(`${this.type} undo 불가 — 살아있는 삭제 조각이 없다`);
    }
    wasm.restoreDeleteFragment(this.fragmentId);
    this.fragmentId = null;
    return { ...this.cursorBefore };
  }

  mergeWith(): null { return null; }

  selectionBefore(): { start: DocumentPosition; end: DocumentPosition; blockPhase: number | null } {
    return this.selection;
  }

  snapshotResourceCount(): number {
    return 0;
  }

  isNoOp(): boolean {
    return this.noOp;
  }

  discard(wasm: WasmBridge): void {
    if (this.fragmentId !== null) {
      wasm.discardDeleteFragment(this.fragmentId);
      this.fragmentId = null;
    }
  }
}
/**
 * 선택 영역 삭제 명령.
 *
 * 비셀(basic text) 선택은 [#5769] 조각 저장소를 사용한다 — 스냅샷 2슬롯 대신
 * 조각 1개로 역연산해 스냅샷 예산을 절약한다. 셀 내 삭제는 조각 API 미지원이라
 * 기존 SnapshotCommand 경로를 유지한다(Stage 3에서 확장).
 *
 * `kind:'command'` 로 남는다 — 양식 모드 게이트(`isOperationAllowedInEditMode` 의
 * `'deleteSelection'` 분기)가 커맨드 타입에 걸려 있어, `kind:'snapshot'` 으로 바꾸면
 * 양식 모드 선택 삭제가 게이트에서 드롭돼 무언 폐기가 된다.
 */
export class DeleteSelectionCommand implements EditCommand {
  readonly type = 'deleteSelection';
  readonly timestamp = Date.now();

  /** 조각 경로(비셀) 또는 스냅샷 경로(셀). 둘 중 하나만 실체화된다. */
  private readonly fragment: FragmentDeleteCommand | null;
  private readonly snapshot: SnapshotCommand | null;
  private readonly selection: {
    start: DocumentPosition;
    end: DocumentPosition;
    blockPhase: number | null;
  };

  constructor(start: DocumentPosition, end: DocumentPosition, blockPhase: number | null = null) {
    this.selection = { start: { ...start }, end: { ...end }, blockPhase };

    if (isCell(start)) {
      // 셀 내 삭제 — 조각 API 미지원, 스냅샷 경로 유지 (Stage 3에서 확장)
      this.fragment = null;
      this.snapshot = new SnapshotCommand('deleteSelection', end, start, (wasm) => {
        wasm.deleteRangeInCellByPath(
          start.sectionIndex, start.parentParaIndex!, cellPathJson(start),
          cellParaIndexOf(start), start.charOffset, cellParaIndexOf(end), end.charOffset,
        );
        return { ...start };
      });
    } else {
      // 비셀 선택 — 조각 경로 (#5769)
      this.snapshot = null;
      this.fragment = new FragmentDeleteCommand(
        end,   // cursorBefore — undo 시 돌아갈 위치(선택 끝)
        start, // cursorAfter — execute 후 커서 위치(선택 시작)
        this.selection,
        start.sectionIndex,
        start.paragraphIndex,
        end.paragraphIndex,
        (wasm) => {
          wasm.deleteRange(
            start.sectionIndex, start.paragraphIndex, start.charOffset,
            end.paragraphIndex, end.charOffset,
          );
          return { ...start };
        },
      );
    }
  }

  execute(wasm: WasmBridge): DocumentPosition {
    return this.fragment
      ? this.fragment.execute(wasm)
      : this.snapshot!.execute(wasm);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    return this.fragment
      ? this.fragment.undo(wasm)
      : this.snapshot!.undo(wasm);
  }

  mergeWith(): null { return null; }

  selectionBefore(): { start: DocumentPosition; end: DocumentPosition; blockPhase: number | null } {
    return this.selection;
  }

  snapshotResourceCount(): number {
    return this.fragment
      ? this.fragment.snapshotResourceCount()
      : this.snapshot!.snapshotResourceCount();
  }

  isNoOp(): boolean {
    return this.fragment
      ? this.fragment.isNoOp()
      : this.snapshot?.isNoOp() ?? false;
  }

  discard(wasm: WasmBridge): void {
    this.fragment?.discard(wasm);
    this.snapshot?.discard(wasm);
  }
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
      // 대상 문단 수만큼 뮤테이터를 호출하므로 재페이지네이션을 묶는다(#4118).
      wasm.runInBatch(() => {
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
      });
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

    // 대상 수만큼 뮤테이터를 호출하므로 재페이지네이션을 묶는다(#4118).
    wasm.runInBatch(() => {
      for (const entry of entries) {
        applyParaFormatToTarget(wasm, entry.target, propsJson);
      }
    });
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

  /** 병합으로 사라진 문단의 스코프 메타데이터 — undo 분할이 되돌린다 (Task #2342) */
  private removedParaMeta?: RemovedParaMeta;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para } = this.position;
    this.removedParaMeta = JSON.parse(wasm.mergeParagraph(sec, para + 1)).removedParaMeta;
    return { ...this.position };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const { sectionIndex: sec, paragraphIndex: para, charOffset } = this.position;
    wasm.splitParagraph(sec, para, charOffset, this.removedParaMeta);
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

/**
 * [Task #3212] 머리말/꼬리말 필드(쪽 번호·전체 쪽수·파일 이름) 삽입의 역연산 명령.
 *
 * 필드는 HF 문단에 제어문자 마커로 들어가므로 역연산은 그 문자 범위 삭제다. HF 모드
 * '내부' 편집이라 snapshot 으로 기록하면 undo 가 본문 분기를 타 HF 밖으로 튕겨나가므로,
 * editContext 를 노출하는 이 명령으로 기록해 undo/redo 가 HF 모드와 오프셋을 유지한다.
 */
export class InsertFieldInHeaderFooterCommand implements EditCommand {
  readonly type = 'insertFieldInHeaderFooter';
  readonly timestamp = Date.now();
  private lastContext: EditContext;

  constructor(
    private target: HeaderFooterEditTarget,
    private paraIdx: number,
    /** redo 시 native에 다시 넘길 원래 cursor 좌표 */
    private requestedCharOffset: number,
    /** undo가 marker를 지워야 하는 실제 모델 텍스트 좌표 */
    private insertedAt: number,
    private fieldType: number,
    /** 필드 마커가 실제 모델 텍스트에서 차지한 문자 수. */
    private markerLength: number,
    /** 삽입 직후 cursor가 돌아갈 좌표. */
    private cursorAfterOffset: number,
  ) {
    this.lastContext = hfEditContext(target, paraIdx, cursorAfterOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    const result = wasm.insertFieldInHf(
      this.target.sectionIdx, this.target.isHeader, this.target.applyTo,
      this.paraIdx, this.requestedCharOffset, this.fieldType,
    );
    if (result.ok) {
      this.insertedAt = result.insertedAt;
      this.markerLength = result.insertedLength;
      this.cursorAfterOffset = result.charOffset;
    }
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.cursorAfterOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.deleteTextInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx, this.insertedAt, this.markerLength);
    this.lastContext = hfEditContext(this.target, this.paraIdx, this.requestedCharOffset);
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
    /** 병합으로 사라진 문단의 스코프 메타데이터 — undo 분할이 되돌린다 (Task #2342). */
    private removedParaMeta?: RemovedParaMeta,
  ) {
    this.lastContext = hfEditContext(target, prevParaIdx, mergeOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.paraIdx);
    this.lastContext = hfEditContext(this.target, this.prevParaIdx, this.mergeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInHeaderFooter(this.target.sectionIdx, this.target.isHeader, this.target.applyTo, this.prevParaIdx, this.mergeOffset, this.removedParaMeta);
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
    /** 병합으로 사라진 문단의 스코프 메타데이터 — undo 분할이 되돌린다 (Task #2342). */
    private removedParaMeta?: RemovedParaMeta,
  ) {
    this.lastContext = fnEditContext(target, prevInnerParaIdx, mergeOffset);
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.mergeParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.innerParaIdx);
    this.lastContext = fnEditContext(this.target, this.prevInnerParaIdx, this.mergeOffset);
    return hfFnStubPosition(this.target.sectionIdx);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.splitParagraphInFootnote(this.target.sectionIdx, this.target.paraIdx, this.target.controlIdx, this.prevInnerParaIdx, this.mergeOffset, this.removedParaMeta);
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
  private lastMutationEffects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      wasm.splitParagraphInCellByPath(sec, ppi, cellPathJson(pos), pos.charOffset);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi, pos.charOffset);
    }
    // [#4031] 네이티브 split은 paginate_if_needed()로 최신 revision을 동기 계산한다.
    // 이 선언이 pending deferred 상태를 해소해 직후 before-full-edit flush가 no-op이 된다.
    this.lastMutationEffects = IMMEDIATE_TEXT_MUTATION_EFFECTS;
    return cellParagraphPosition(pos, cpi + 1, 0);
  }

  consumeTextMutationEffects(): TextMutationEffects {
    const effects = this.lastMutationEffects;
    this.lastMutationEffects = NO_TEXT_MUTATION_EFFECTS;
    return effects;
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
  /** 사라진 문단의 스코프 메타 — undo(분할)가 되돌린다 (Task #2342). */
  private removedParaMeta?: RemovedParaMeta;

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
      this.removedParaMeta = JSON.parse(wasm.mergeParagraphInCellByPath(sec, ppi, cellPathJson(pos))).removedParaMeta;
    } else {
      this.mergePointOffset = wasm.getCellParagraphLength(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi - 1);
      this.removedParaMeta = JSON.parse(wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi)).removedParaMeta;
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
      wasm.splitParagraphInCellByPath(sec, ppi, JSON.stringify(undoPath), this.mergePointOffset, this.removedParaMeta);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi - 1, this.mergePointOffset, this.removedParaMeta);
    }
    return { ...pos };
  }

  mergeWith(): null { return null; }
}

// ─── 셀 내부 다음 문단 병합 명령 (셀 문단 끝에서 Delete) ──

export class MergeNextParagraphInCellCommand implements EditCommand {
  readonly type = 'mergeNextParagraphInCell';
  readonly timestamp = Date.now();

  /** 사라진 문단의 스코프 메타 — undo(분할)가 되돌린다 (Task #2342). */
  private removedParaMeta?: RemovedParaMeta;

  constructor(private position: DocumentPosition) {}

  execute(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      const nextPath = [...pos.cellPath!];
      nextPath[nextPath.length - 1] = { ...nextPath[nextPath.length - 1], cellParaIndex: cpi + 1 };
      this.removedParaMeta = JSON.parse(wasm.mergeParagraphInCellByPath(sec, ppi, JSON.stringify(nextPath))).removedParaMeta;
    } else {
      this.removedParaMeta = JSON.parse(wasm.mergeParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi + 1)).removedParaMeta;
    }
    return { ...pos };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    const pos = this.position;
    const sec = pos.sectionIndex;
    const ppi = pos.parentParaIndex!;
    const cpi = cellParaIndexOf(pos);
    if (isNestedCell(pos)) {
      wasm.splitParagraphInCellByPath(sec, ppi, cellPathJson(pos), pos.charOffset, this.removedParaMeta);
    } else {
      wasm.splitParagraphInCell(sec, ppi, pos.controlIndex!, pos.cellIndex!, cpi, pos.charOffset, this.removedParaMeta);
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
    // [Task #2903] execute() 는 moveTableOffset 반환값(result.ppi/ci)을 권위 소스로 삼아
    // this.resultPpi/resultCi 를 갱신하는데, undo() 는 동일한 반환값을 버리고 생성 시점의
    // stale this.ppi/this.ci 를 그대로 반환했다. 표 이동과 문단 구조 변경(삽입/삭제/병합)이
    // 같은 세션에서 섞이면 undo 후 커서가 존재하지 않거나 엉뚱한 문단을 가리킬 수 있다 —
    // execute() 와 대칭으로 반환값을 캡처해 this.ppi/this.ci 를 갱신한다.
    const result = wasm.moveTableOffset(this.sec, this.resultPpi, this.resultCi, -this.deltaH, -this.deltaV);
    this.ppi = result.ppi;
    this.ci = result.ci;
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
 * [Task #2759] 직선/연결선 끝점 드래그를 Undo/Redo 스택에 기록하기 위한 명령.
 *
 * ResizeObjectCommand 와 동일하게 드래그 중 WASM 에 이미 반영된 변경을 kind:'record' 로
 * 사후 기록한다(execute 는 redo 경로에서만 재적용). before/after 는 글로벌 끝점 좌표
 * (HWPUNIT)이며 moveLineEndpoint 는 절대 좌표 setter 라 역연산이 자명하다.
 */
export class MoveLineEndpointCommand implements EditCommand {
  readonly type = 'moveLineEndpoint';
  readonly timestamp: number;

  constructor(
    private sec: number,
    private ppi: number,
    private ci: number,
    private before: LineEndpointsLike,
    private after: LineEndpointsLike,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  execute(wasm: WasmBridge): DocumentPosition {
    wasm.moveLineEndpoint(this.sec, this.ppi, this.ci,
      this.after.sx, this.after.sy, this.after.ex, this.after.ey);
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  undo(wasm: WasmBridge): DocumentPosition {
    wasm.moveLineEndpoint(this.sec, this.ppi, this.ci,
      this.before.sx, this.before.sy, this.before.ex, this.before.ey);
    return { sectionIndex: this.sec, paragraphIndex: this.ppi, charOffset: 0 };
  }

  mergeWith(): null { return null; }
}

/**
 * [Task #3230] 개체 속성 변경의 역연산 명령 (kind:'record' 용, #2337 계열).
 *
 * 스냅샷 대신 쓰는 이유는 비용이다. 스냅샷 1개는 `Document` 통째 클론이라 문서에 비례하고
 * (실측: 30KB 공문 0.43 MB · 10MB 행정편람 10.59 MB), `SnapshotCommand` 는 before/after 로
 * 2개를 쓴다. 회전 한 번에 최대 21 MB 를 스택에 얹는 셈인데, 실제로 되돌려야 하는 것은
 * **스칼라 속성 하나**다.
 *
 * 역연산이 자명한 조건은 셋이고 회전·대칭은 셋을 다 만족한다.
 *  - setter 가 **절대값**이다(누적·토글이 아니라 `rotationAngle = 30`, `horzFlip = true`).
 *  - 호출부가 적용 **전에 현재 값을 이미 읽는다**(다음 각도·토글 반대값을 그것으로 만든다).
 *  - 그 속성만 바뀐다 — 파생 상태(레이아웃·앵커)는 setter 가 다시 계산한다.
 *
 * 뮤테이션은 호출부가 적용하고 이 명령은 기록만 담당한다(`execute` 는 redo 경로에서만 재적용).
 */
export class SetObjectPropsCommand implements EditCommand {
  readonly type = 'setObjectProps';
  readonly timestamp: number;

  constructor(
    private ref: ObjectPropsRef,
    private before: Record<string, unknown>,
    private after: Record<string, unknown>,
    timestamp?: number,
  ) {
    this.timestamp = timestamp ?? Date.now();
  }

  private apply(wasm: WasmBridge, props: Record<string, unknown>): DocumentPosition {
    setObjectProps(wasm, this.ref, props);
    return { sectionIndex: this.ref.sec, paragraphIndex: this.ref.ppi, charOffset: 0 };
  }

  execute(wasm: WasmBridge): DocumentPosition {
    return this.apply(wasm, this.after);
  }

  undo(wasm: WasmBridge): DocumentPosition {
    return this.apply(wasm, this.before);
  }

  /**
   * 적용해도 달라질 것이 없으면 무변경이다 (#2370 규약).
   *
   * 히스토리는 커맨드에게 이 질문을 하고(`history.ts` 의 `command.isNoOp?.()`), 답하지 않으면
   * "항상 바꾼다" 로 읽는다. before 와 after 가 같은데 침묵하면 그 자체가 거짓 응답이고,
   * 팬텀 엔트리가 Ctrl+Z 한 번을 무효과로 소모하며 redo 스택까지 파기한다.
   *
   * 지금 호출부(±90° 회전·`!cur` 토글)는 항상 before ≠ after 라 이 분기를 타지 않는다. 그래도
   * 답은 커맨드가 해야 한다 — before/after 를 아는 것은 이 객체뿐이고, 호출부마다 같은 비교를
   * 되풀이하는 것은 판정을 소비자로 흘리는 일이다.
   *
   * `after` 의 키만 본다. `execute` 가 적용하는 것이 그것뿐이므로 `before` 에만 있는 키는
   * 이 연산의 결과를 바꾸지 않는다.
   */
  isNoOp(): boolean {
    const keys = Object.keys(this.after);
    if (keys.length === 0) return true;
    return keys.every((key) => Object.is(this.before[key], this.after[key]));
  }

  /**
   * 병합하지 않는다. 회전 15° 를 네 번 누른 것과 60° 를 한 번 누른 것은 한컴에서도 undo
   * 횟수가 다르다 — 묶으면 되돌리기 단위가 사용자가 누른 단위와 어긋난다.
   */
  mergeWith(): null { return null; }
}

/**
 * 구역 설정(SectionDef)의 old/new 속성쌍 역연산 명령 (#5769 Stage 4).
 *
 * set_section_def 는 적용 시 구역 raw_stream 을 무효화한다 — 속성만 되돌리면 저장이
 * IR 재구성 경로로 바뀌어 원본 바이트와 어긋난다(속성을 하나도 바꾸지 않고 같은 값을
 * 한 번 적용만 해도 재현됨). 그래서 변경 직전 passthrough 를 captureSectionRaw 저널에
 * 보관했다가 undo 에서 old 재적용(raw 재무효화) 뒤 되돌린다. Rust 수렴 실증:
 * tests/cases/issue_5769_stage4_setter_convergence.rs — 바이트 왕복 동일.
 *
 * snapshotResourceCount 는 항상 0 — 스냅샷 예산을 쓰지 않는다.
 */
export class SetSectionPropsCommand implements EditCommand {
  readonly type = 'setSectionProps';
  readonly timestamp = Date.now();

  /** execute 가 잡아 둔 passthrough 캡처. undo 가 소비하고 discard 가 해제한다. */
  private captureId: number | null = null;
  private noOp = false;

  constructor(
    private sectionIdx: number,
    private before: SectionDef,
    private after: SectionDef,
    private cursorPos: DocumentPosition,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.propsEqual()) {
      this.noOp = true;
      return { ...this.cursorPos };
    }
    // redo 재실행도 같은 순서다 — undo 가 저널 항목을 소비하므로 항상 새로 캡처한다.
    // 지역 변수로 받는다 — catch 에서 프로퍼티 좁히기가 풀린다(FragmentDeleteCommand 선례).
    const captureId = wasm.captureSectionRaw(this.sectionIdx);
    this.captureId = captureId;
    try {
      wasm.setSectionDef(this.sectionIdx, this.after);
      return { ...this.cursorPos };
    } catch (e) {
      wasm.discardSectionRaw(captureId);
      this.captureId = null;
      throw e;
    }
  }

  undo(wasm: WasmBridge): DocumentPosition {
    if (this.captureId === null) {
      throw new Error(`${this.type} undo 불가 — 살아있는 raw 캡처가 없다`);
    }
    const captureId = this.captureId;
    wasm.setSectionDef(this.sectionIdx, this.before); // old 재적용 — raw 재무효화를 동반한다
    wasm.restoreSectionRaw(captureId); // passthrough 복원 — 저널 항목 소비
    this.captureId = null;
    return { ...this.cursorPos };
  }

  discard(wasm: WasmBridge): void {
    if (this.captureId !== null) {
      wasm.discardSectionRaw(this.captureId);
      this.captureId = null;
    }
  }

  snapshotResourceCount(): number { return 0; }

  isNoOp(): boolean { return this.noOp; }

  /** 병합하지 않는다 — 다이얼로그 확인 1회가 되돌리기 단위 1회다(한컴 정합). */
  mergeWith(): null { return null; }

  /** SectionDef 전 필드 비교 — after 만 보면 된다(execute 가 적용하는 것이 그것뿐). */
  private propsEqual(): boolean {
    const keys: (keyof SectionDef)[] = [
      'pageNum', 'pageNumType', 'pictureNum', 'tableNum', 'equationNum',
      'columnSpacing', 'defaultTabSpacing',
      'hideHeader', 'hideFooter', 'hideMasterPage', 'hideBorder', 'hideFill', 'hideEmptyLine',
    ];
    return keys.every((key) => Object.is(this.before[key], this.after[key]));
  }
}

/**
 * [#5769 후속] SetZOrderCommand 가 소비하는 자기기술 변경 레코드 항목(Rust moves 응답).
 */
export interface ZOrderMove {
  ppi: number;
  ci: number;
  before: number;
  after: number;
}

/**
 * [#5769 후속] Shape z 순서 변경의 역연산 커맨드 — 스냅샷 대체.
 *
 * 최초 execute 는 기존 상대 연산(changeShapeZOrder)을 그대로 실행하고 Rust 가 돌려준
 * 자기기술 레코드(moves — 대상+교환 이웃의 before/after)를 저장한다. undo/redo 는 상대
 * 연산을 다시 머리로 계산하지 않고 저장된 쌍을 절대 대입(applyShapeZOrderPairs)해 되돌린다
 * — front(max+1) 같은 연산엔 모드 역연산이 없으므로 값 복원이 유일한 참인 역연산이다
 * (#5769 선결 규약). Shape z 대입에는 부작용이 없어 속성쌍 왕복이 수렴한다.
 *
 * passthrough 생명주기는 SetSectionPropsCommand 와 같다 — 첫 뮤테이션 전에 구역 raw 를
 * capture 하고, undo 의 재무효화 뒤 restore 한다. redo 는 저널 항목이 소비됐으므로 재캡처한다.
 * 슬롯 효과: changeZOrder 스냅샷 1슬롯 → 0.
 */
export class SetZOrderCommand implements EditCommand {
  readonly type = 'changeZOrder';
  readonly timestamp = Date.now();

  /** 최초 실행에서 확정된 변경 레코드. null = 아직 실행 전. */
  private moves: ZOrderMove[] | null = null;
  private captureId: number | null = null;
  private noOp = false;

  constructor(
    private sectionIdx: number,
    private ppi: number,
    private ci: number,
    private operation: 'front' | 'forward' | 'backward' | 'back',
    private cursorPos: DocumentPosition,
  ) {}

  private pairsJson(dir: 'before' | 'after'): string {
    return JSON.stringify(
      (this.moves ?? []).map((m) => ({ ppi: m.ppi, ci: m.ci, z: m[dir] })),
    );
  }

  execute(wasm: WasmBridge): DocumentPosition {
    // 스큐 선제 차단(gpt 3차 리뷰) — 구버전 wasm 은 moves 를 주지 않아 실제 변경의
    // 역연산 기록을 만들 수 없다. 적용 **전에** 거절해 무음 변이를 원천 차단한다.
    if (!wasm.hasShapeZOrderInverse()) {
      throw new Error('changeZOrder 역연산 불가 — wasm 이 moves 응답 이전 버전이다');
    }
    const captureId = wasm.captureSectionRaw(this.sectionIdx);
    this.captureId = captureId;
    try {
      if (this.moves) {
        // redo — 상대 연산은 멱등하지 않으므로(front=항상 max+1) 저장된 after 를 대입한다.
        const r = wasm.applyShapeZOrderPairs(this.sectionIdx, this.pairsJson('after'));
        // 거절 = 기록 이후 문서가 out-of-band 로 바졌다 — 성공으로 스택을 옮기지 않는다.
        if (!r.ok) {
          throw new Error(`${this.type} redo 거부 — 기록 이후 문서가 바뀌었다`);
        }
      } else {
        // 최초 실행 — 기존 상대 연산으로 적용하고 자기기술 레코드를 받는다.
        const r = wasm.changeShapeZOrder(this.sectionIdx, this.ppi, this.ci, this.operation);
        if (r.ok && r.moves === undefined) {
          // 위 probe 를 통과했다면 도달하지 않는다(같은 릴리스가 moves 와 쌍 메서드를
          // 함께 제공한다). 도달했다면 빈 moves 흡수가 실제 변이의 기록 상실이 되므로
          // 소리 없이 넘기지 않고 실패시킨다 — 아래 catch 가 캡처를 폐기하고,
          // history 의 catch 가 엔트리 없이 전파한다.
          throw new Error(`${this.type}: wasm ok 응답에 moves 가 없다 — 빌드 짝이 어긋났다`);
        }
        const moves = r.ok ? r.moves ?? [] : [];
        if (!r.ok || moves.length === 0) {
          // 이미 맨 앞/뒤 등 무변경 — phantom 엔트리와 캡처 잔류를 막는다(#2370 계약).
          this.noOp = true;
          wasm.discardSectionRaw(captureId);
          this.captureId = null;
          return { ...this.cursorPos };
        }
        this.moves = moves;
      }
      return { ...this.cursorPos };
    } catch (e) {
      wasm.discardSectionRaw(captureId);
      this.captureId = null;
      throw e;
    }
  }

  undo(wasm: WasmBridge): DocumentPosition {
    if (!this.moves || this.captureId === null) {
      throw new Error(`${this.type} undo 불가 — 살아있는 기록/캡처가 없다`);
    }
    const captureId = this.captureId;
    wasm.applyShapeZOrderPairs(this.sectionIdx, this.pairsJson('before')); // old 재적용 — raw 재무효화 동반
    wasm.restoreSectionRaw(captureId); // passthrough 복원 — 저널 항목 소비
    this.captureId = null;
    return { ...this.cursorPos };
  }

  discard(wasm: WasmBridge): void {
    if (this.captureId !== null) {
      wasm.discardSectionRaw(this.captureId);
      this.captureId = null;
    }
  }

  snapshotResourceCount(): number { return 0; }

  isNoOp(): boolean { return this.noOp; }

  /** 병합하지 않는다 — 정렬 메뉴 1회·클릭 1회가 되돌리기 단위다(한컴 정합). */
  mergeWith(): null { return null; }
}

/**
 * [#5769 후속] 문서 전체(all) 구역 설정의 역연산 커맨드 — 다구역 raw 저널.
 *
 * `setSectionDefAll` 은 네이티브에서 구역 루프+단일 재조판일 뿐이라(#5769 후속2 실측)
 * Rust 확장 없이 조합으로 역연산화한다. 구역별 before 를 변경 전에 읽어 두고,
 * execute 는 전 구역 캡처 후 `setSectionDefAll(after)` 한 번(네이티브 효율 유지),
 * undo 는 구역별 old 재적용 뒤 캡처를 되돌린다. 저널 계약은 구역 단위라("그 구역의
 * 캡처와 복원 사이 그 구역 편집 금지") 다구역 교차 적용과 정합이다.
 *
 * 슬롯 효과: all 범위 스냅샷 1슬롯 → 0. 양식 모드 게이트는 type 이 default 차단
 * (종전 snapshot 차단과 동일)이므로 동작 변화 없다.
 */
export class SetSectionPropsAllCommand implements EditCommand {
  readonly type = 'setSectionProps';
  readonly timestamp = Date.now();

  /** execute 가 잡아 둔 구역별 passthrough 캡처. undo 가 소비하고 discard 가 해제한다. */
  private captureIds: Array<{ idx: number; id: number }> = [];
  private noOp = false;

  constructor(
    private sections: Array<{ idx: number; before: SectionDef }>,
    private after: SectionDef,
    private cursorPos: DocumentPosition,
  ) {}

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.sections.every((s) => this.propsEqual(s.before))) {
      this.noOp = true;
      return { ...this.cursorPos };
    }
    // redo 도 같은 순서다 — undo 가 저널 항목들을 소비하므로 항상 새로 캡처한다.
    // 캡처를 전부 마친 뒤 한 번 적용한다 — 적용 도중 실패가 있으면 캡처만 낭비된다.
    const captureIds: Array<{ idx: number; id: number }> = [];
    try {
      for (const s of this.sections) {
        captureIds.push({ idx: s.idx, id: wasm.captureSectionRaw(s.idx) });
      }
      wasm.setSectionDefAll(this.after);
      this.captureIds = captureIds;
      return { ...this.cursorPos };
    } catch (e) {
      for (const c of captureIds) wasm.discardSectionRaw(c.id);
      throw e;
    }
  }

  undo(wasm: WasmBridge): DocumentPosition {
    if (this.captureIds.length === 0) {
      throw new Error(`${this.type} undo 불가 — 살아있는 raw 캡처가 없다`);
    }
    const captureIds = this.captureIds;
    // old 일괄 재적용(재조판 1회 — 구역별 setter 반복의 성능 회귀 차단, gpt 3차
    // 리뷰) → 캡처 복원. 순서는 캡처와 동일하게 유지한다.
    wasm.applySectionDefsBulk(
      JSON.stringify(this.sections.map((s) => ({ idx: s.idx, def: s.before }))),
    );
    for (const c of captureIds) {
      wasm.restoreSectionRaw(c.id);
    }
    this.captureIds = [];
    return { ...this.cursorPos };
  }

  discard(wasm: WasmBridge): void {
    for (const c of this.captureIds) wasm.discardSectionRaw(c.id);
    this.captureIds = [];
  }

  snapshotResourceCount(): number { return 0; }

  isNoOp(): boolean { return this.noOp; }

  /** 병합하지 않는다 — 다이얼로그 확인 1회가 되돌리기 단위 1회다(한컴 정합). */
  mergeWith(): null { return null; }

  /** SectionDef 전 필드 비교 — after 만 보면 된다(execute 가 적용하는 것이 그것뿐). */
  private propsEqual(before: SectionDef): boolean {
    const keys: (keyof SectionDef)[] = [
      'pageNum', 'pageNumType', 'pictureNum', 'tableNum', 'equationNum',
      'columnSpacing', 'defaultTabSpacing',
      'hideHeader', 'hideFooter', 'hideMasterPage', 'hideBorder', 'hideFill', 'hideEmptyLine',
    ];
    return keys.every((key) => Object.is(before[key], this.after[key]));
  }
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
 * - 최초 실행: before 스냅샷 저장 → 작업 수행 (**after 는 저장하지 않는다**)
 * - Undo: after 스냅샷 저장 → before 스냅샷으로 복원
 * - Redo: after 스냅샷으로 복원 → 그 스냅샷 즉시 반환
 * - Discard: 살아있는 스냅샷 메모리 해제
 *
 * **[Task #5769] after 는 undo 시점에 잡는다.** 이 명령이 undo 스택 top 이라는 것은
 * 히스토리 불변식상 "현재 문서 == 이 명령 실행 직후 상태" 를 뜻하므로, 그때 찍어도 값이
 * 같다. 그래서 undo 스택 엔트리는 스냅샷 id 를 **1개**만 점유한다(redo 스택 엔트리는 2개).
 * 예산 98 기준 최악 undo 깊이가 49 → 98 로 배가된다. 복원 의미는 종전과 같은 문서 전체
 * 치환이라 문서 충실도는 달라지지 않는다.
 */
export class SnapshotCommand implements EditCommand {
  readonly type: string;
  readonly timestamp = Date.now();

  private beforeId: number | null = null;
  private afterId: number | null = null;
  private noOp = false;
  /**
   * 최초 실행이 끝났는가. redo 판별에 쓴다.
   *
   * 종전에는 `afterId !== null` 이 곧 "이미 실행됨" 이었지만, after 를 undo 시점에 잡게
   * 되면서 그 등가가 깨졌다 — 실행 직후에도 `afterId` 는 `null` 이다. 플래그 없이 두면
   * redo 가 최초 실행으로 오인돼 `beforeId` 를 덮어쓰고 앞의 스냅샷을 누수한다.
   */
  private executed = false;
  /** undo 시점 after 저장에 실패해 redo 를 제공할 수 없는 상태인가. */
  private redoUnavailable = false;

  /**
   * @param operationType 작업 종류 (예: 'pasteInternal', 'deleteControl')
   * @param cursorBefore 작업 전 커서 위치
   * @param operation 실제 작업을 수행하는 함수. 작업 후 커서 위치를 반환하며,
   *   문서를 전혀 바꾸지 않았으면 `null` 을 반환해 기록을 취소한다([Task #2370]).
   */
  constructor(
    operationType: string,
    private cursorBefore: DocumentPosition,
    private cursorAfter: DocumentPosition,
    private operation: ((wasm: WasmBridge) => DocumentPosition | null) | null,
  ) {
    this.type = `snapshot:${operationType}`;
  }

  execute(wasm: WasmBridge): DocumentPosition {
    if (this.executed) {
      // Redo: undo 시점에 잡아 둔 after 로 복원하고 **그 스냅샷을 즉시 반환한다** —
      // 복원 뒤 이 명령은 undo 스택으로 돌아가고, 그러면 after 는 다시 "현재 문서" 와
      // 같아져 들고 있을 이유가 없다. 이 반환이 없으면 undo↔redo 왕복마다 엔트리가
      // 2슬롯으로 남아 지연 저장의 이득이 사라진다.
      if (this.redoUnavailable || this.afterId === null) {
        // undo 시점 after 저장이 실패한 명령이다. 조용히 성공한 척하면 문서가 그대로인
        // 채 redo 스택만 움직여 이후 undo 가 어긋난다 — 히스토리의 실패시-드롭
        // 하이브리드(#2328)가 이 엔트리를 걷어내도록 던진다.
        throw new Error(`${this.type} redo 불가 — undo 시점 after 스냅샷 저장 실패`);
      }
      wasm.restoreSnapshot(this.afterId);
      wasm.discardSnapshot(this.afterId);
      this.afterId = null;
      return { ...this.cursorAfter };
    }

    // 최초 실행: before 저장 → 작업 수행 (after 는 undo 시점에 잡는다)
    this.executed = true;
    this.beforeId = wasm.saveSnapshot();
    // [Task #2328] operation 이 throw 하면 커맨드가 히스토리에 등록되지 못해 discard
    // 주체가 사라진다 → 스냅샷 영구 누수(orphan → WASM 무통보 축출 재발). 아래 catch 가
    // before 를 대칭적으로 해제한다.
    try {
      if (this.operation) {
        const result = this.operation(wasm);
        if (result === null) {
          // [Task #2370] 문서 무변경 — after 를 저장하지 않고 before 를 즉시 반환한다.
          // 히스토리는 isNoOp() 를 보고 이 명령을 스택에 넣지 않는다.
          this.noOp = true;
          this.operation = null;
          this.discard(wasm);
          return { ...this.cursorBefore };
        }
        this.cursorAfter = result;
      }
    } catch (operationError) {
      // [#3350] 최초 execute 가 실패하면 명령 전체를 원자적으로 되돌린다. 이 커맨드는
      // history 에 push 되기 전이므로 before 스냅샷을 가진 SnapshotCommand만 rollback을
      // 수행할 수 있다.
      try {
        if (this.beforeId !== null) {
          wasm.restoreSnapshot(this.beforeId);
        }
      } catch (rollbackError) {
        this.discard(wasm);
        throw new AggregateError(
          [operationError, rollbackError],
          `${this.type} 실행 실패 후 rollback도 실패했습니다`,
        );
      }
      this.discard(wasm);
      throw operationError;
    }

    // operation 참조 해제 (클로저에 캡처된 리소스 해제)
    this.operation = null;

    return { ...this.cursorAfter };
  }

  /** [Task #2370] operation 이 `null` 을 반환해 기록이 취소된 명령인가. */
  isNoOp(): boolean {
    return this.noOp;
  }

  undo(wasm: WasmBridge): DocumentPosition {
    // [Task #5769] after 를 여기서 잡는다. 이 명령이 undo 스택 top 이라는 것은
    // "현재 문서 == 이 명령 실행 직후 상태" 라는 뜻이므로 실행 시점에 찍은 것과 값이 같다.
    if (this.afterId === null && !this.redoUnavailable) {
      try {
        this.afterId = wasm.saveSnapshot();
      } catch {
        // 저장 실패로 **되돌리기 자체를 막지 않는다.** undo 는 수행하고 redo 만 포기한다 —
        // 여기서 던지면 사용자의 Ctrl+Z 가 아무 일도 못 하고 히스토리 엔트리까지 잃는다.
        this.redoUnavailable = true;
      }
    }
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

/**
 * 머리말/꼬리말·각주 안에서만 쓰는 스냅샷 명령.
 *
 * 일반 SnapshotCommand는 구조 편집처럼 undo 뒤 본문으로 돌아가야 하는 작업도 담당한다.
 * 그래서 편집 문맥을 일반 클래스에 붙이지 않고, 서브모드를 보존해야 하는 호출부만 이 타입을
 * 명시적으로 선택한다.
 */
export class SubmodeSnapshotCommand extends SnapshotCommand {
  constructor(
    operationType: string,
    cursorBefore: DocumentPosition,
    cursorAfter: DocumentPosition,
    operation: ((wasm: WasmBridge) => DocumentPosition | null) | null,
    private readonly context: EditContext,
  ) {
    super(operationType, cursorBefore, cursorAfter, operation);
  }

  editContext(): EditContext { return this.context; }
}