import { t } from '../i18n/index.ts';
import type { MemoAnchor } from './memo';
import type { HyperlinkTarget } from './hyperlink';

export interface MemoInfo extends MemoAnchor {
  id: number; index: number; parentId: number | null; author: string; date: string | null; text: string;
  page: number; editable: boolean; readOnlyReason: string | null;
}
export type MemoEdit = { kind: 'insert'; author: string; text: string }
  | { kind: 'update'; id: number; text: string } | { kind: 'remove'; id: number };

export class MemoEditError extends Error {}

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

export function validateMemoEdit(edit: MemoEdit, memos: readonly MemoInfo[]): void {
  if (edit.kind !== 'remove' && !edit.text.trim()) throw new Error(t('dialog.memo.blank'));
  if (edit.kind === 'insert') return;
  const existing = memos.find(m => m.id === edit.id);
  if (!existing) throw new Error(t('dialog.memo.missing'));
  if (edit.kind === 'remove' && memos.some(m => m.parentId === edit.id)) {
    throw new Error(t('dialog.memo.hasReplies'));
  }
  if (!existing.editable) throw new Error(t('dialog.memo.readOnly'));
}
