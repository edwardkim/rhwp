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
 * 같은 본문 문단 또는 같은 표 셀 내부의 짧은 insert/delete만 허용한다.
 * 실제 flow 변경은 mutation effect가 별도로 차단한다.
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
  const beforeInCell = beforePos.parentParaIndex !== undefined;
  const afterInCell = afterPos.parentParaIndex !== undefined;
  if (beforeInCell !== afterInCell) return false;
  if (beforePos.sectionIndex !== afterPos.sectionIndex) return false;
  if (!beforeInCell) {
    return beforePos.paragraphIndex === afterPos.paragraphIndex;
  }
  if (beforePos.parentParaIndex !== afterPos.parentParaIndex) return false;
  if (beforePos.controlIndex !== afterPos.controlIndex) return false;
  if (beforePos.cellIndex !== afterPos.cellIndex) return false;
  if (beforePos.cellParaIndex !== afterPos.cellParaIndex) return false;
  return sameCellPath(beforePos.cellPath, afterPos.cellPath);
}
