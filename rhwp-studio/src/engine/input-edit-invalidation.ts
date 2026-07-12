import type { CellPathEntry, DocumentPosition } from '@/core/types';

const PAGE_LOCAL_TEXT_COMMANDS = new Set(['insertText', 'deleteText']);
export const MAX_PAGE_LOCAL_TEXT_EDIT_CHARS = 8;

export interface PageLocalTextEditOptions {
  insertedText?: string;
  deleteCount?: number;
  beforePageIndex?: number | null;
  afterPageIndex?: number | null;
}

function sameCellPath(a: CellPathEntry[] | undefined, b: CellPathEntry[] | undefined): boolean {
  const left = a ?? [];
  const right = b ?? [];
  if (left.length !== right.length) return false;
  return left.every((value, index) => {
    const other = right[index];
    return (
      value.controlIndex === other.controlIndex &&
      value.cellIndex === other.cellIndex &&
      value.cellParaIndex === other.cellParaIndex
    );
  });
}

/**
 * 페이지 단위 canvas 재렌더만으로 처리할 수 있는 보수적인 텍스트 편집인지 판정한다.
 *
 * 현재는 표 셀 내부의 단일 insert/delete 텍스트 편집만 허용한다. 본문 문단 편집,
 * 문단 병합/분할, 붙여넣기, 객체/표 구조 변경은 page flow 변동 가능성이 크므로
 * 기존 full document refresh 경로를 유지한다.
 */
export function isPageLocalTextEditCommand(
  commandType: string,
  beforePos: DocumentPosition,
  afterPos: DocumentPosition,
  options: PageLocalTextEditOptions = {},
): boolean {
  if (!PAGE_LOCAL_TEXT_COMMANDS.has(commandType)) return false;
  if (
    typeof options.beforePageIndex === 'number' &&
    typeof options.afterPageIndex === 'number' &&
    options.beforePageIndex !== options.afterPageIndex
  ) {
    return false;
  }
  if (commandType === 'insertText') {
    if (typeof options.insertedText === 'string') {
      if (options.insertedText.length > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
      if (/[\r\n\t]/.test(options.insertedText)) return false;
    } else {
      const insertedLength = afterPos.charOffset - beforePos.charOffset;
      if (insertedLength > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS) return false;
    }
  } else if (commandType === 'deleteText') {
    if (
      typeof options.deleteCount === 'number' &&
      options.deleteCount > MAX_PAGE_LOCAL_TEXT_EDIT_CHARS
    ) {
      return false;
    }
  }
  if (beforePos.parentParaIndex === undefined || afterPos.parentParaIndex === undefined) return false;
  if (beforePos.sectionIndex !== afterPos.sectionIndex) return false;
  if (beforePos.parentParaIndex !== afterPos.parentParaIndex) return false;
  if (beforePos.controlIndex !== afterPos.controlIndex) return false;
  if (beforePos.cellIndex !== afterPos.cellIndex) return false;
  if (beforePos.cellParaIndex !== afterPos.cellParaIndex) return false;
  return sameCellPath(beforePos.cellPath, afterPos.cellPath);
}
