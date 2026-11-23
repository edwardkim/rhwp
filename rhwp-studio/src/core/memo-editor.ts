import { t } from '../i18n/index.ts';
import type { MemoAnchor } from './memo';
import type { HyperlinkTarget } from './hyperlink';

export interface MemoInfo extends MemoAnchor {
  id: number; index: number; parentId: number | null; author: string; date: string | null; text: string;
  page: number; editable: boolean; readOnlyReason: string | null;
}
export type MemoEdit = { kind: 'insert'; author: string; text: string }
  | { kind: 'reply'; id: number; author: string; text: string }
  | { kind: 'update'; id: number; text: string }
  | { kind: 'remove'; id: number; confirmedReplies?: number };

export interface MemoComposeRequest {
  kind: 'insert' | 'update' | 'reply';
  cardId: number | null;
  anchor?: MemoAnchor;
  restoreFocus: () => void;
  author: string;
  text: string;
  submit: (author: string, text: string) => void;
}

export class MemoEditError extends Error {}

const memoWordBreak = /[ \t\r\n\u3000]/u;

/** 웹한글의 커서 메모는 공백 사이 낱말과 양끝 경계를 사용한다.
 * 메모 API의 위치는 Unicode scalar 인덱스다.
 */
export function memoRangeAtCaret(text: string, caret: number): { start: number; end: number } {
  const chars = Array.from(text);
  let last = { start: caret, end: caret };
  let end = 0;
  while (end < chars.length) {
    while (end < chars.length && memoWordBreak.test(chars[end])) end++;
    const start = end;
    while (end < chars.length && !memoWordBreak.test(chars[end])) end++;
    if (start === end) break;
    last = { start, end };
    if (caret <= end) break;
  }
  return last;
}

/** WASM은 실패를 JSON으로 반환하기도 하므로 snapshot을 성공 처리하지 않는다. */
export function requireMemoSuccess(value: string): { id?: number } {
  const result = JSON.parse(value);
  if (result?.ok !== true) {
    const messages: Record<string, string> = {
      memo_not_found: 'dialog.memo.missing',
      memo_invalid_range: 'dialog.memo.invalidTarget',
      memo_overlap: 'dialog.memo.overlap',
      memo_unsupported_place: 'dialog.memo.unsupportedTarget',
      memo_read_only: 'dialog.memo.readOnly',
      memo_text_too_long: 'dialog.memo.textTooLong',
      memo_invalid_author: 'dialog.memo.invalidAuthor',
    };
    const key = typeof result?.code === 'string' && Object.hasOwn(messages, result.code)
      ? messages[result.code] : 'dialog.memo.failed';
    throw new MemoEditError(t(key), { cause: result?.message });
  }
  return result;
}

export function memoTargetPath(target: HyperlinkTarget): string {
  return JSON.stringify(target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) =>
    ({ controlIndex, cellIndex, cellParaIndex })));
}

export function countMemoReplies(id: number, memos: readonly MemoInfo[]): number {
  const children = new Map<number, number[]>();
  for (const memo of memos) {
    if (memo.parentId === null) continue;
    const siblings = children.get(memo.parentId) ?? [];
    siblings.push(memo.id);
    children.set(memo.parentId, siblings);
  }
  const visited = new Set([id]);
  const queue = [id];
  for (const parent of queue) {
    for (const child of children.get(parent) ?? []) {
      if (visited.has(child)) continue;
      visited.add(child);
      queue.push(child);
    }
  }
  return visited.size - 1;
}

export function validateMemoEdit(edit: MemoEdit, memos: readonly MemoInfo[]): void {
  if (edit.kind !== 'remove' && !edit.text.trim()) throw new Error(t('dialog.memo.blank'));
  if (edit.kind === 'insert') return;
  const existing = memos.find(m => m.id === edit.id);
  if (!existing) throw new Error(t('dialog.memo.missing'));
  if (!existing.editable) throw new Error(t('dialog.memo.readOnly'));
  if (edit.kind === 'reply' && existing.parentId !== null) throw new Error(t('dialog.memo.invalidTarget'));
  if (edit.kind === 'remove' && countMemoReplies(edit.id, memos) !== (edit.confirmedReplies ?? 0)) {
    throw new Error(t(edit.confirmedReplies === undefined ? 'dialog.memo.hasReplies' : 'dialog.memo.changed'));
  }
}
