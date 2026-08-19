import { sha256 } from '@noble/hashes/sha2.js';
import { bytesToHex } from '@noble/hashes/utils.js';

import type { EventBus } from '../core/event-bus.ts';
import type { DocumentPosition, FieldInfoResult } from '../core/types.ts';
import {
  DocumentAgentError,
  isDocumentAgentError,
  type RhwpApplyTextCommandV1,
  type RhwpBodyParagraphTargetV1,
  type RhwpDocumentStateV1,
  type RhwpRevertTextCommandV1,
  type RhwpSelectionContextV1,
  type RhwpTextCommandReceiptV1,
} from './types.ts';

const MAX_PARAGRAPH_LENGTH = 4000;
const COMMAND_BUDGET_MS = 3000;
const encoder = new TextEncoder();

type ExportableDocument = {
  exportHwp(): Uint8Array;
  exportHwpx(): Uint8Array;
};

export interface DocumentAgentWasm {
  readonly documentGeneration: number;
  readonly pageCount: number;
  getSourceFormat(): string;
  getSectionCount(): number;
  getParagraphCount(section: number): number;
  getParagraphLength(section: number, paragraph: number): number;
  getTextRange(section: number, paragraph: number, offset: number, count: number): string;
  getControlTextPositions(section: number, paragraph: number): number[];
  getCharPropertiesAt(section: number, paragraph: number, offset: number): { charShapeId?: number };
  getParaPropertiesAt(section: number, paragraph: number): { paraShapeId?: number };
  getStyleAt(section: number, paragraph: number): { id: number; name: string };
  getFieldInfoAt(position: DocumentPosition): FieldInfoResult;
  replaceText(
    section: number,
    paragraph: number,
    offset: number,
    length: number,
    text: string,
  ): { ok: boolean; charOffset?: number; newLength?: number };
  setCharShapeId(
    section: number,
    paragraph: number,
    start: number,
    end: number,
    charShapeId: number,
  ): string;
  setParaShapeId(section: number, paragraph: number, paraShapeId: number): string;
  getPageOfPosition(section: number, paragraph: number): { ok: boolean; page?: number };
  borrowDocumentHandle(): ExportableDocument | null;
  beginDeferredPagination?(): void;
  flushDeferredPagination?(): void;
  cancelDeferredPagination?(): void;
}

export interface DocumentAgentInput {
  getCursorPosition(): DocumentPosition;
  getSelection(): { start: DocumentPosition; end: DocumentPosition } | null;
  executeDocumentAgentOperation(desc: {
    kind: 'snapshot';
    operationType: string;
    operation: (wasm: DocumentAgentWasm) => DocumentPosition | null;
    meta?: {
      actionId?: string;
      domain?: 'text';
      refresh?: 'full';
      dirtyScope?: 'paragraph';
      selection?: 'moveToResult';
    };
  }, render: () => Promise<void>): Promise<void>;
  focusBodyParagraph(section: number, paragraph: number, length: number): boolean;
}

export interface TargetEvidence {
  text: string;
  textSha256: string;
  formatSha256: string;
  adjacentContextSha256: string;
  charShapeId: number;
  paraShapeId: number;
  styleId: number;
}

interface AgentJournalEntry {
  command: RhwpApplyTextCommandV1;
  applyBindingSha256: string;
  applyReceipt: RhwpTextCommandReceiptV1;
  beforeText: string;
  beforeTarget: RhwpBodyParagraphTargetV1;
  afterTarget: RhwpBodyParagraphTargetV1;
  beforeEvidence: TargetEvidence;
  afterEvidence: TargetEvidence;
  nonTargetManifestSha256: string;
  status: 'applied' | 'reverted';
  revertBindingSha256?: string;
  revertReceipt?: RhwpTextCommandReceiptV1;
}

interface DocumentAgentControllerDeps {
  wasm: DocumentAgentWasm;
  input: DocumentAgentInput;
  eventBus: EventBus;
  isDirty(): boolean;
  render(): Promise<void>;
  now?: () => number;
}

function digestBytes(value: Uint8Array): string {
  return bytesToHex(sha256(value));
}

function digestText(value: string): string {
  return digestBytes(encoder.encode(value));
}

function codePointLength(value: string): number {
  return Array.from(value).length;
}

function safeId(value: number | undefined, label: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new DocumentAgentError('TARGET_FORMAT_MISMATCH', `${label}을 확인할 수 없습니다.`);
  }
  return value as number;
}

function assertTargetCoordinates(wasm: DocumentAgentWasm, target: RhwpBodyParagraphTargetV1): void {
  if (target.kind !== 'body_paragraph'
      || target.charOffset !== 0
      || !Number.isSafeInteger(target.section)
      || !Number.isSafeInteger(target.paragraph)
      || !Number.isSafeInteger(target.length)
      || target.section < 0
      || target.paragraph < 0
      || target.length < 0
      || target.length > MAX_PARAGRAPH_LENGTH
      || target.section >= wasm.getSectionCount()
      || target.paragraph >= wasm.getParagraphCount(target.section)
      || wasm.getParagraphLength(target.section, target.paragraph) !== target.length) {
    throw new DocumentAgentError('TARGET_NOT_FOUND', 'exact body paragraph target을 찾을 수 없습니다.');
  }
}

function charShapeRuns(wasm: DocumentAgentWasm, section: number, paragraph: number, length: number): number[] {
  const ids: number[] = [];
  const count = Math.max(length, 1);
  for (let offset = 0; offset < count; offset += 1) {
    ids.push(safeId(
      wasm.getCharPropertiesAt(section, paragraph, offset).charShapeId,
      'charShapeId',
    ));
  }
  return ids;
}

function paragraphSemantic(
  wasm: DocumentAgentWasm,
  section: number,
  paragraph: number,
): Record<string, unknown> {
  const length = wasm.getParagraphLength(section, paragraph);
  const text = length > 0 ? wasm.getTextRange(section, paragraph, 0, length) : '';
  return {
    section,
    paragraph,
    length,
    textSha256: digestText(text),
    paraShapeId: safeId(
      wasm.getParaPropertiesAt(section, paragraph).paraShapeId,
      'paraShapeId',
    ),
    styleId: safeId(wasm.getStyleAt(section, paragraph).id, 'styleId'),
    charShapeIds: charShapeRuns(wasm, section, paragraph, length),
    controls: wasm.getControlTextPositions(section, paragraph),
  };
}

function adjacentContextSha256(
  wasm: DocumentAgentWasm,
  target: RhwpBodyParagraphTargetV1,
): string {
  const previous = target.paragraph > 0
    ? paragraphSemantic(wasm, target.section, target.paragraph - 1)
    : null;
  const next = target.paragraph + 1 < wasm.getParagraphCount(target.section)
    ? paragraphSemantic(wasm, target.section, target.paragraph + 1)
    : null;
  return digestText(JSON.stringify({ schemaVersion: 1, previous, next }));
}

function hasField(
  wasm: DocumentAgentWasm,
  target: RhwpBodyParagraphTargetV1,
): boolean {
  for (let offset = 0; offset <= target.length; offset += 1) {
    if (wasm.getFieldInfoAt({
      sectionIndex: target.section,
      paragraphIndex: target.paragraph,
      charOffset: offset,
    }).inField) return true;
  }
  return false;
}

export function collectTargetEvidence(
  wasm: DocumentAgentWasm,
  target: RhwpBodyParagraphTargetV1,
): TargetEvidence {
  assertTargetCoordinates(wasm, target);
  if (wasm.getControlTextPositions(target.section, target.paragraph).length > 0 || hasField(wasm, target)) {
    throw new DocumentAgentError(
      'TARGET_FORMAT_MISMATCH',
      '컨트롤 또는 필드가 포함된 문단은 에이전트 명령으로 편집할 수 없습니다.',
    );
  }
  const charShapeIds = charShapeRuns(wasm, target.section, target.paragraph, target.length);
  const charShapeId = charShapeIds[0];
  if (!charShapeIds.every(id => id === charShapeId)) {
    throw new DocumentAgentError(
      'TARGET_FORMAT_MISMATCH',
      '혼합 글자 서식 문단은 에이전트 명령으로 편집할 수 없습니다.',
    );
  }
  const paraShapeId = safeId(
    wasm.getParaPropertiesAt(target.section, target.paragraph).paraShapeId,
    'paraShapeId',
  );
  const styleId = safeId(wasm.getStyleAt(target.section, target.paragraph).id, 'styleId');
  const text = target.length > 0
    ? wasm.getTextRange(target.section, target.paragraph, 0, target.length)
    : '';
  return {
    text,
    textSha256: digestText(text),
    formatSha256: digestText(JSON.stringify({
      schemaVersion: 1,
      charShapeId,
      paraShapeId,
      styleId,
    })),
    adjacentContextSha256: adjacentContextSha256(wasm, target),
    charShapeId,
    paraShapeId,
    styleId,
  };
}

function nonTargetManifestSha256(
  wasm: DocumentAgentWasm,
  target: RhwpBodyParagraphTargetV1,
): string {
  const paragraphs: Array<Record<string, unknown>> = [];
  for (let section = 0; section < wasm.getSectionCount(); section += 1) {
    for (let paragraph = 0; paragraph < wasm.getParagraphCount(section); paragraph += 1) {
      if (section === target.section && paragraph === target.paragraph) continue;
      paragraphs.push(paragraphSemantic(wasm, section, paragraph));
    }
  }
  return digestText(JSON.stringify({
    schemaVersion: 1,
    sectionCount: wasm.getSectionCount(),
    paragraphCounts: Array.from(
      { length: wasm.getSectionCount() },
      (_, section) => wasm.getParagraphCount(section),
    ),
    paragraphs,
  }));
}

function exportDocumentSha256(wasm: DocumentAgentWasm, format: 'hwp' | 'hwpx'): string {
  const document = wasm.borrowDocumentHandle();
  if (!document) throw new DocumentAgentError('TARGET_NOT_FOUND', '문서가 로드되지 않았습니다.');
  return digestBytes(format === 'hwpx' ? document.exportHwpx() : document.exportHwp());
}

function commandBinding(command: RhwpApplyTextCommandV1): string {
  return digestText(JSON.stringify({
    schemaVersion: command.schemaVersion,
    commandId: command.commandId,
    expectedDocumentEpoch: command.expectedDocumentEpoch,
    expectedChangeSeq: command.expectedChangeSeq,
    expectedDocumentSha256: command.expectedDocumentSha256,
    target: command.target,
    expectedBeforeSha256: command.expectedBeforeSha256,
    expectedFormatSha256: command.expectedFormatSha256,
    expectedAdjacentContextSha256: command.expectedAdjacentContextSha256,
    replacement: command.replacement,
  }));
}

function revertBinding(command: RhwpRevertTextCommandV1): string {
  return digestText(JSON.stringify({
    schemaVersion: command.schemaVersion,
    commandId: command.commandId,
    expectedDocumentEpoch: command.expectedDocumentEpoch,
    expectedChangeSeq: command.expectedChangeSeq,
    expectedAfterDocumentSha256: command.expectedAfterDocumentSha256,
    expectedAfterSha256: command.expectedAfterSha256,
  }));
}

function sameBodyPosition(a: DocumentPosition, b: DocumentPosition): boolean {
  return a.parentParaIndex === undefined
    && b.parentParaIndex === undefined
    && a.sectionIndex === b.sectionIndex
    && a.paragraphIndex === b.paragraphIndex;
}

function samePosition(a: DocumentPosition, b: DocumentPosition): boolean {
  return sameBodyPosition(a, b) && a.charOffset === b.charOffset;
}

export class DocumentAgentController {
  private readonly deps: DocumentAgentControllerDeps;
  private readonly now: () => number;
  private documentEpoch: number;
  private changeSeq = 0;
  private latest: AgentJournalEntry | null = null;
  private readonly journal = new Map<string, AgentJournalEntry>();
  private readonly offMutation: () => void;

  constructor(deps: DocumentAgentControllerDeps) {
    this.deps = deps;
    this.now = deps.now ?? (() => performance.now());
    this.documentEpoch = deps.wasm.documentGeneration;
    this.offMutation = deps.eventBus.on('document-mutated', () => {
      this.syncGeneration();
      this.changeSeq += 1;
    });
  }

  dispose(): void {
    this.offMutation();
  }

  getDocumentState(): RhwpDocumentStateV1 {
    this.syncGeneration();
    const format = this.currentFormat();
    return {
      schemaVersion: 1,
      format,
      documentEpoch: this.documentEpoch,
      changeSeq: this.changeSeq,
      dirty: this.deps.isDirty(),
      pageCount: this.deps.wasm.pageCount,
      documentSha256: exportDocumentSha256(this.deps.wasm, format),
    };
  }

  getSelectionContext(): RhwpSelectionContextV1 {
    this.syncGeneration();
    const position = this.deps.input.getCursorPosition();
    const page = this.pageFor(position.sectionIndex, position.paragraphIndex);
    const selection = this.deps.input.getSelection();
    const collapsed = !selection || samePosition(selection.start, selection.end);
    let target: RhwpBodyParagraphTargetV1 | null = null;
    let editable = false;
    let selectedTextSha256: string | null = null;

    if (position.parentParaIndex === undefined
        && position.sectionIndex >= 0
        && position.paragraphIndex >= 0
        && position.sectionIndex < this.deps.wasm.getSectionCount()
        && position.paragraphIndex < this.deps.wasm.getParagraphCount(position.sectionIndex)) {
      const length = this.deps.wasm.getParagraphLength(
        position.sectionIndex,
        position.paragraphIndex,
      );
      if (length <= MAX_PARAGRAPH_LENGTH) {
        target = {
          kind: 'body_paragraph',
          section: position.sectionIndex,
          paragraph: position.paragraphIndex,
          charOffset: 0,
          length,
        };
        try {
          collectTargetEvidence(this.deps.wasm, target);
          this.currentFormat();
          editable = true;
        } catch {
          editable = false;
        }
      }
    }

    if (!collapsed && selection && sameBodyPosition(selection.start, selection.end)) {
      const length = selection.end.charOffset - selection.start.charOffset;
      if (length >= 0) {
        selectedTextSha256 = digestText(this.deps.wasm.getTextRange(
          selection.start.sectionIndex,
          selection.start.paragraphIndex,
          selection.start.charOffset,
          length,
        ));
      }
    }

    return {
      schemaVersion: 1,
      documentEpoch: this.documentEpoch,
      changeSeq: this.changeSeq,
      page,
      editable,
      collapsed,
      target,
      selectedTextSha256,
    };
  }

  async applyTextCommand(command: RhwpApplyTextCommandV1): Promise<RhwpTextCommandReceiptV1> {
    this.syncGeneration();
    const binding = commandBinding(command);
    const replay = this.journal.get(command.commandId);
    if (replay) {
      if (replay.status === 'applied'
          && replay.applyBindingSha256 === binding
          && this.isCurrentReceiptState(
            replay.applyReceipt,
            replay.afterTarget,
            replay.applyReceipt.afterTextSha256,
          )) {
        return replay.applyReceipt;
      }
      throw new DocumentAgentError(
        'COMMAND_REPLAY_MISMATCH',
        '같은 commandId가 다른 binding 또는 terminal 상태로 이미 사용되었습니다.',
      );
    }

    const startedAt = this.now();
    const state = this.getDocumentState();
    this.assertApplyFence(command, state);
    const beforeEvidence = collectTargetEvidence(this.deps.wasm, command.target);
    if (beforeEvidence.textSha256 !== command.expectedBeforeSha256) {
      throw new DocumentAgentError('TARGET_PREIMAGE_MISMATCH', 'target before SHA가 다릅니다.');
    }
    if (beforeEvidence.formatSha256 !== command.expectedFormatSha256) {
      throw new DocumentAgentError('TARGET_FORMAT_MISMATCH', 'target format SHA가 다릅니다.');
    }
    if (beforeEvidence.adjacentContextSha256 !== command.expectedAdjacentContextSha256) {
      throw new DocumentAgentError('TARGET_CONTEXT_MISMATCH', 'target adjacent context SHA가 다릅니다.');
    }
    const beforeNonTarget = nonTargetManifestSha256(this.deps.wasm, command.target);
    this.assertWithinBudget(startedAt);

    const afterTarget: RhwpBodyParagraphTargetV1 = {
      ...command.target,
      length: codePointLength(command.replacement),
    };
    let afterEvidence!: TargetEvidence;
    let afterDocumentSha256 = '';
    const beforeSeq = this.changeSeq;

    try {
      await this.deps.input.executeDocumentAgentOperation({
        kind: 'snapshot',
        operationType: 'document-agent:apply',
        operation: (wasm) => {
          this.assertRuntimeFence(command.expectedDocumentEpoch, beforeSeq);
          let deferred = false;
          try {
            wasm.beginDeferredPagination?.();
            deferred = true;
            const result = wasm.replaceText(
              command.target.section,
              command.target.paragraph,
              0,
              command.target.length,
              command.replacement,
            );
            if (!result.ok || result.newLength !== afterTarget.length) {
              throw new DocumentAgentError('TRANSACTION_FAILED', 'replaceText가 target을 교체하지 못했습니다.');
            }
            if (afterTarget.length > 0) {
              wasm.setCharShapeId(
                command.target.section,
                command.target.paragraph,
                0,
                afterTarget.length,
                beforeEvidence.charShapeId,
              );
            }
            wasm.setParaShapeId(
              command.target.section,
              command.target.paragraph,
              beforeEvidence.paraShapeId,
            );
            wasm.flushDeferredPagination?.();
            deferred = false;

            afterEvidence = collectTargetEvidence(wasm, afterTarget);
            if (afterEvidence.text !== command.replacement) {
              throw new DocumentAgentError('TARGET_PREIMAGE_MISMATCH', 'target postimage가 replacement와 다릅니다.');
            }
            if (afterEvidence.formatSha256 !== beforeEvidence.formatSha256) {
              throw new DocumentAgentError('TARGET_FORMAT_MISMATCH', 'target format이 변경되었습니다.');
            }
            if (afterEvidence.adjacentContextSha256 !== beforeEvidence.adjacentContextSha256) {
              throw new DocumentAgentError('TARGET_CONTEXT_MISMATCH', 'adjacent context가 변경되었습니다.');
            }
            if (nonTargetManifestSha256(wasm, afterTarget) !== beforeNonTarget) {
              throw new DocumentAgentError('NON_TARGET_CHANGED', 'target 밖 semantic manifest가 변경되었습니다.');
            }
            if (wasm.pageCount !== state.pageCount) {
              throw new DocumentAgentError('PAGE_COUNT_CHANGED', '문서 페이지 수가 변경되었습니다.');
            }
            afterDocumentSha256 = exportDocumentSha256(wasm, state.format);
            this.assertWithinBudget(startedAt);
            return {
              sectionIndex: afterTarget.section,
              paragraphIndex: afterTarget.paragraph,
              charOffset: afterTarget.length,
            };
          } catch (error) {
            if (deferred) {
              try { wasm.cancelDeferredPagination?.(); } catch { /* snapshot rollback이 최종 복구한다. */ }
            }
            throw error;
          }
        },
        meta: {
          actionId: 'document-agent:apply',
          domain: 'text',
          refresh: 'full',
          dirtyScope: 'paragraph',
          selection: 'moveToResult',
        },
      }, async () => {
        await this.deps.render();
        this.assertWithinBudget(startedAt);
      });
    } catch (error) {
      throw this.normalizeExecutionError(error);
    }

    if (this.changeSeq !== beforeSeq + 1) {
      throw new DocumentAgentError(
        'TRANSACTION_FAILED',
        'changeSeq가 정확히 1 증가하지 않았습니다.',
        false,
      );
    }
    const receipt: RhwpTextCommandReceiptV1 = {
      schemaVersion: 1,
      commandId: command.commandId,
      operation: 'apply',
      documentEpoch: this.documentEpoch,
      beforeChangeSeq: beforeSeq,
      afterChangeSeq: this.changeSeq,
      beforeDocumentSha256: state.documentSha256,
      afterDocumentSha256,
      beforeTextSha256: beforeEvidence.textSha256,
      afterTextSha256: afterEvidence.textSha256,
      formatSha256: afterEvidence.formatSha256,
      adjacentContextSha256: afterEvidence.adjacentContextSha256,
      pageCountBefore: state.pageCount,
      pageCountAfter: this.deps.wasm.pageCount,
      target: command.target,
    };
    const entry: AgentJournalEntry = {
      command: structuredClone(command),
      applyBindingSha256: binding,
      applyReceipt: receipt,
      beforeText: beforeEvidence.text,
      beforeTarget: command.target,
      afterTarget,
      beforeEvidence,
      afterEvidence,
      nonTargetManifestSha256: beforeNonTarget,
      status: 'applied',
    };
    this.journal.set(command.commandId, entry);
    this.latest = entry;
    this.emitDocumentAgentChanged({
      schemaVersion: 1,
      reason: 'agent_apply',
      documentEpoch: this.documentEpoch,
      changeSeq: this.changeSeq,
      commandId: command.commandId,
    });
    return receipt;
  }

  async revertTextCommand(command: RhwpRevertTextCommandV1): Promise<RhwpTextCommandReceiptV1> {
    this.syncGeneration();
    const binding = revertBinding(command);
    const entry = this.journal.get(command.commandId);
    if (entry?.status === 'reverted') {
      if (entry.revertBindingSha256 === binding
          && entry.revertReceipt
          && this.isCurrentReceiptState(
            entry.revertReceipt,
            entry.beforeTarget,
            entry.revertReceipt.afterTextSha256,
          )) return entry.revertReceipt;
      throw new DocumentAgentError(
        'COMMAND_REPLAY_MISMATCH',
        'revert command binding 또는 현재 terminal receipt 상태가 다릅니다.',
      );
    }
    if (!entry || this.latest !== entry || entry.status !== 'applied'
        || this.changeSeq !== entry.applyReceipt.afterChangeSeq) {
      throw new DocumentAgentError('COMMAND_NOT_LATEST', '가장 최근 exact command만 되돌릴 수 있습니다.');
    }

    const startedAt = this.now();
    const state = this.getDocumentState();
    if (command.expectedDocumentEpoch !== state.documentEpoch) {
      throw new DocumentAgentError('DOCUMENT_EPOCH_MISMATCH', 'document epoch가 다릅니다.');
    }
    if (command.expectedChangeSeq !== state.changeSeq) {
      throw new DocumentAgentError('CHANGE_SEQ_MISMATCH', 'changeSeq가 다릅니다.');
    }
    if (command.expectedAfterDocumentSha256 !== state.documentSha256) {
      throw new DocumentAgentError('DOCUMENT_SHA_MISMATCH', 'after document SHA가 다릅니다.');
    }
    const currentEvidence = collectTargetEvidence(this.deps.wasm, entry.afterTarget);
    if (currentEvidence.textSha256 !== command.expectedAfterSha256
        || currentEvidence.textSha256 !== entry.applyReceipt.afterTextSha256) {
      throw new DocumentAgentError('TARGET_PREIMAGE_MISMATCH', 'revert target after SHA가 다릅니다.');
    }
    if (nonTargetManifestSha256(this.deps.wasm, entry.afterTarget)
        !== entry.nonTargetManifestSha256) {
      throw new DocumentAgentError('COMMAND_NOT_LATEST', 'target 밖 변경이 있어 되돌릴 수 없습니다.');
    }
    this.assertWithinBudget(startedAt);

    let afterDocumentSha256 = '';
    let revertedEvidence!: TargetEvidence;
    const beforeSeq = this.changeSeq;
    try {
      await this.deps.input.executeDocumentAgentOperation({
        kind: 'snapshot',
        operationType: 'document-agent:revert',
        operation: (wasm) => {
          this.assertRuntimeFence(command.expectedDocumentEpoch, beforeSeq);
          let deferred = false;
          try {
            wasm.beginDeferredPagination?.();
            deferred = true;
            const result = wasm.replaceText(
              entry.afterTarget.section,
              entry.afterTarget.paragraph,
              0,
              entry.afterTarget.length,
              entry.beforeText,
            );
            if (!result.ok || result.newLength !== entry.beforeTarget.length) {
              throw new DocumentAgentError('TRANSACTION_FAILED', 'inverse replaceText가 실패했습니다.');
            }
            if (entry.beforeTarget.length > 0) {
              wasm.setCharShapeId(
                entry.beforeTarget.section,
                entry.beforeTarget.paragraph,
                0,
                entry.beforeTarget.length,
                entry.beforeEvidence.charShapeId,
              );
            }
            wasm.setParaShapeId(
              entry.beforeTarget.section,
              entry.beforeTarget.paragraph,
              entry.beforeEvidence.paraShapeId,
            );
            wasm.flushDeferredPagination?.();
            deferred = false;

            revertedEvidence = collectTargetEvidence(wasm, entry.beforeTarget);
            if (revertedEvidence.textSha256 !== entry.beforeEvidence.textSha256) {
              throw new DocumentAgentError('TARGET_PREIMAGE_MISMATCH', 'before target 복원이 일치하지 않습니다.');
            }
            if (revertedEvidence.formatSha256 !== entry.beforeEvidence.formatSha256) {
              throw new DocumentAgentError('TARGET_FORMAT_MISMATCH', 'before format 복원이 일치하지 않습니다.');
            }
            if (revertedEvidence.adjacentContextSha256
                !== entry.beforeEvidence.adjacentContextSha256) {
              throw new DocumentAgentError('TARGET_CONTEXT_MISMATCH', 'before context 복원이 일치하지 않습니다.');
            }
            if (nonTargetManifestSha256(wasm, entry.beforeTarget)
                !== entry.nonTargetManifestSha256) {
              throw new DocumentAgentError('NON_TARGET_CHANGED', 'revert가 target 밖을 변경했습니다.');
            }
            if (wasm.pageCount !== entry.applyReceipt.pageCountBefore) {
              throw new DocumentAgentError('PAGE_COUNT_CHANGED', 'revert 뒤 페이지 수가 다릅니다.');
            }
            afterDocumentSha256 = exportDocumentSha256(wasm, state.format);
            this.assertWithinBudget(startedAt);
            return {
              sectionIndex: entry.beforeTarget.section,
              paragraphIndex: entry.beforeTarget.paragraph,
              charOffset: entry.beforeTarget.length,
            };
          } catch (error) {
            if (deferred) {
              try { wasm.cancelDeferredPagination?.(); } catch { /* snapshot rollback이 최종 복구한다. */ }
            }
            throw error;
          }
        },
        meta: {
          actionId: 'document-agent:revert',
          domain: 'text',
          refresh: 'full',
          dirtyScope: 'paragraph',
          selection: 'moveToResult',
        },
      }, async () => {
        await this.deps.render();
        this.assertWithinBudget(startedAt);
      });
    } catch (error) {
      throw this.normalizeExecutionError(error);
    }

    if (this.changeSeq !== beforeSeq + 1) {
      throw new DocumentAgentError(
        'TRANSACTION_FAILED',
        'revert changeSeq가 정확히 1 증가하지 않았습니다.',
        false,
      );
    }
    const receipt: RhwpTextCommandReceiptV1 = {
      schemaVersion: 1,
      commandId: command.commandId,
      operation: 'revert',
      documentEpoch: this.documentEpoch,
      beforeChangeSeq: beforeSeq,
      afterChangeSeq: this.changeSeq,
      beforeDocumentSha256: state.documentSha256,
      afterDocumentSha256,
      beforeTextSha256: currentEvidence.textSha256,
      afterTextSha256: revertedEvidence.textSha256,
      formatSha256: revertedEvidence.formatSha256,
      adjacentContextSha256: revertedEvidence.adjacentContextSha256,
      pageCountBefore: state.pageCount,
      pageCountAfter: this.deps.wasm.pageCount,
      target: entry.beforeTarget,
    };
    entry.status = 'reverted';
    entry.revertBindingSha256 = binding;
    entry.revertReceipt = receipt;
    this.emitDocumentAgentChanged({
      schemaVersion: 1,
      reason: 'agent_revert',
      documentEpoch: this.documentEpoch,
      changeSeq: this.changeSeq,
      commandId: command.commandId,
    });
    return receipt;
  }

  focusTarget(target: RhwpBodyParagraphTargetV1): { focused: boolean; page: number } {
    this.syncGeneration();
    assertTargetCoordinates(this.deps.wasm, target);
    const page = this.pageFor(target.section, target.paragraph);
    return {
      focused: this.deps.input.focusBodyParagraph(target.section, target.paragraph, target.length),
      page,
    };
  }

  private currentFormat(): 'hwp' | 'hwpx' {
    const format = this.deps.wasm.getSourceFormat();
    if (format !== 'hwp' && format !== 'hwpx') {
      throw new DocumentAgentError(
        'CAPABILITY_UNSUPPORTED',
        `document agent는 HWP/HWPX만 지원합니다: ${format}`,
      );
    }
    return format;
  }

  private pageFor(section: number, paragraph: number): number {
    const result = this.deps.wasm.getPageOfPosition(section, paragraph);
    if (!result.ok || !Number.isSafeInteger(result.page) || (result.page as number) < 0) {
      return 1;
    }
    return (result.page as number) + 1;
  }

  private assertApplyFence(command: RhwpApplyTextCommandV1, state: RhwpDocumentStateV1): void {
    if (command.expectedDocumentEpoch !== state.documentEpoch) {
      throw new DocumentAgentError('DOCUMENT_EPOCH_MISMATCH', 'document epoch가 다릅니다.');
    }
    if (command.expectedChangeSeq !== state.changeSeq) {
      throw new DocumentAgentError('CHANGE_SEQ_MISMATCH', 'changeSeq가 다릅니다.');
    }
    if (command.expectedDocumentSha256 !== state.documentSha256) {
      throw new DocumentAgentError('DOCUMENT_SHA_MISMATCH', 'document SHA가 다릅니다.');
    }
  }

  private assertRuntimeFence(expectedEpoch: number, expectedSeq: number): void {
    this.syncGeneration();
    if (this.documentEpoch !== expectedEpoch) {
      throw new DocumentAgentError('DOCUMENT_EPOCH_MISMATCH', 'transaction 직전 epoch가 바뀌었습니다.');
    }
    if (this.changeSeq !== expectedSeq) {
      throw new DocumentAgentError('CHANGE_SEQ_MISMATCH', 'transaction 직전 changeSeq가 바뀌었습니다.');
    }
  }

  private assertWithinBudget(startedAt: number): void {
    if (this.now() - startedAt > COMMAND_BUDGET_MS) {
      throw new DocumentAgentError('COMMAND_TOO_SLOW', '문서 명령이 3초 상한을 넘었습니다.');
    }
  }

  private syncGeneration(): void {
    const generation = this.deps.wasm.documentGeneration;
    if (generation === this.documentEpoch) return;
    this.documentEpoch = generation;
    this.changeSeq = 0;
    this.latest = null;
    this.journal.clear();
  }

  private normalizeExecutionError(error: unknown): DocumentAgentError {
    if (isDocumentAgentError(error)) {
      if (error.code === 'TRANSACTION_FAILED' && error.recovered === undefined) {
        return new DocumentAgentError(error.code, error.message, true);
      }
      return error;
    }
    const executionError = error as {
      code?: unknown;
      recovered?: unknown;
      cause?: unknown;
      message?: unknown;
    };
    const nestedCause = executionError.cause as { code?: unknown; message?: unknown } | undefined;
    if (executionError.code === 'RENDER_FAILED'
        && typeof executionError.recovered === 'boolean') {
      if (nestedCause?.code === 'COMMAND_TOO_SLOW') {
        return new DocumentAgentError(
          'COMMAND_TOO_SLOW',
          typeof nestedCause.message === 'string' ? nestedCause.message : 'command time budget을 초과했습니다.',
          executionError.recovered,
        );
      }
      return new DocumentAgentError(
        'RENDER_FAILED',
        typeof executionError.message === 'string' ? executionError.message : 'render commit이 실패했습니다.',
        executionError.recovered,
      );
    }
    return new DocumentAgentError(
      'TRANSACTION_FAILED',
      error instanceof Error ? error.message : String(error),
      !(error instanceof AggregateError),
    );
  }

  private isCurrentReceiptState(
    receipt: RhwpTextCommandReceiptV1,
    target: RhwpBodyParagraphTargetV1,
    expectedTextSha256: string,
  ): boolean {
    try {
      const state = this.getDocumentState();
      if (state.documentEpoch !== receipt.documentEpoch
          || state.changeSeq !== receipt.afterChangeSeq
          || state.documentSha256 !== receipt.afterDocumentSha256) return false;
      return collectTargetEvidence(this.deps.wasm, target).textSha256 === expectedTextSha256;
    } catch {
      return false;
    }
  }

  private emitDocumentAgentChanged(event: {
    schemaVersion: 1;
    reason: 'agent_apply' | 'agent_revert';
    documentEpoch: number;
    changeSeq: number;
    commandId: string;
  }): void {
    try {
      this.deps.eventBus.emit('document-agent-changed', event);
    } catch (error) {
      // transaction과 journal은 이미 commit됐다. 관측자 한 곳의 실패로 RPC 성공을 뒤집지 않는다.
      console.error('[DocumentAgentController] documentChanged observer 실패:', error);
    }
  }
}
