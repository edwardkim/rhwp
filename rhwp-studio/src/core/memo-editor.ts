import type { MemoAnchor } from './memo';
import type { HyperlinkTarget } from './hyperlink';

export interface MemoInfo extends MemoAnchor {
  id: number; index: number; parentId: number | null; author: string; date: string | null; text: string;
  page: number; editable: boolean; readOnlyReason: string | null;
}
export type MemoEdit = { kind: 'insert'; author: string; text: string }
  | { kind: 'update'; id: number; text: string } | { kind: 'remove'; id: number };

/** WASM은 실패를 JSON으로 반환하기도 하므로 snapshot을 성공 처리하지 않는다. */
export function requireMemoSuccess(value: string): void {
  const result = JSON.parse(value);
  if (result?.ok !== true) throw new Error(result?.message || '메모를 변경하지 못했습니다.');
}

export function memoTargetPath(target: HyperlinkTarget): string {
  return JSON.stringify(target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) =>
    ({ controlIndex, cellIndex, cellParaIndex })));
}

export function validateMemoEdit(edit: MemoEdit, memos: readonly MemoInfo[]): void {
  if (edit.kind !== 'remove' && !edit.text.trim()) throw new Error('메모 내용을 입력해 주세요.');
  if (edit.kind === 'insert') return;
  const existing = memos.find(m => m.id === edit.id);
  if (!existing) throw new Error('메모가 변경되거나 삭제되었습니다. 다시 열어 주세요.');
  if (edit.kind === 'remove' && memos.some(m => m.parentId === edit.id)) {
    throw new Error('댓글이 있는 메모입니다. 댓글을 먼저 삭제해 주세요.');
  }
  if (!existing.editable) throw new Error('이 메모는 현재 편집할 수 없습니다.');
}
