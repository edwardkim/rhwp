import type { EditorContext } from './types';

type FormatPasteContext = Pick<
  EditorContext,
  'hasDocument' | 'hasCopiedFormat' | 'isFormMode' | 'hasSelection' | 'inCellSelectionMode'
>;

/** 모양 붙여넣기는 복사 상태와 명확한 적용 대상이 있을 때만 활성화한다. */
export function canExecuteFormatPaste(ctx: FormatPasteContext): boolean {
  return ctx.hasDocument
    && ctx.hasCopiedFormat
    && !ctx.isFormMode
    && (ctx.hasSelection || ctx.inCellSelectionMode);
}
