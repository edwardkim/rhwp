import type { CommandDef } from '../types';
import { hyperlinkRange } from '@/core/hyperlink';
import { StudioCommandError } from '@/core/command-error';
import { MemoEditError, validateMemoEdit } from '@/core/memo-editor';
import { MemoDialog } from '@/ui/memo-dialog';
import { showToast } from '@/ui/toast';
import { t } from '../../i18n/index.ts';

export const memoCommand: CommandDef = {
  id: 'insert:comment', label: t('command.insert.comment.label'), icon: 'icon-comment', opensDialog: true,
  canExecute: ctx => ctx.hasDocument && ctx.isEditable && !ctx.isFormMode
    && !ctx.inCellSelectionMode && !ctx.inPictureObjectSelection && !ctx.inTableObjectSelection,
  execute(services) {
    const ih = services.getInputHandler();
    if (!ih || document.querySelector('[data-memo-dialog]')) return;
    try {
      if (!ih.canEditHyperlink()) throw new Error(t('dialog.memo.invalidTarget'));
      const wasm = services.wasm;
      const selection = ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
      let insertion: { range: ReturnType<typeof hyperlinkRange>; context: string } | null = null;
      try {
        const range = hyperlinkRange(ih.getCursorPosition(), selection);
        insertion = { range, context: JSON.stringify(wasm.getHyperlinkContext(range.target)) };
      } catch {
        // 새 메모 위치를 확인할 수 없는 경우에도 기존 메모는 ID로 관리할 수 있다.
      }
      const generation = wasm.documentGeneration;
      let memos = wasm.getMemos();
      let fingerprint = JSON.stringify(memos);
      const dialog = new MemoDialog(memos, insertion !== null, memo => wasm.getMemoDisplayPage(memo), edit => {
        if (wasm.documentGeneration !== generation || services.getInputHandler() !== ih
          || !services.getContext().isEditable || services.getContext().isFormMode || !ih.canEditHyperlink()
          || JSON.stringify(wasm.getMemos()) !== fingerprint) {
          throw new Error(t('dialog.memo.changed'));
        }
        const range = insertion?.range;
        if (edit.kind === 'insert') {
          if (!range) throw new Error(t('dialog.memo.invalidTarget'));
          if (JSON.stringify(wasm.getHyperlinkContext(range.target)) !== insertion?.context) {
            throw new Error(t('dialog.memo.changed'));
          }
        }
        validateMemoEdit(edit, memos);
        if (edit.kind === 'update' && memos.find(m => m.id === edit.id)?.text === edit.text) return memos;
        const before = ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
        const cursor = ih.getCursorPosition();
        let applied = false;
        try {
          ih.executeOperation({ kind: 'snapshot', operationType: 'editMemo',
            selectionBefore: before ? { ...before, blockPhase: null } : null,
            operation: wasm => {
              if (edit.kind === 'insert') {
                if (!range) throw new Error(t('dialog.memo.invalidTarget'));
                wasm.insertMemo(range.target, range.start, range.end, edit.author, edit.text);
              }
              else if (edit.kind === 'update') wasm.updateMemo(edit.id, edit.text);
              else wasm.removeMemo(edit.id);
              applied = true;
              return edit.kind === 'insert' && range ? range.pos : cursor;
            },
          });
        } catch (error) {
          if (error instanceof StudioCommandError && error.cause instanceof MemoEditError) throw error.cause;
          throw error;
        }
        if (!applied) throw new Error(t('dialog.memo.failed'));
        memos = wasm.getMemos(); fingerprint = JSON.stringify(memos);
        return memos;
      });
      dialog.afterClose = () => ih.focus(); dialog.show();
    } catch (error) { showToast({ message: error instanceof Error ? error.message : String(error) }); }
  },
};
