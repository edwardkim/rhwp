import type { CommandDef } from '../types';
import { hyperlinkRange } from '@/core/hyperlink';
import { validateMemoEdit } from '@/core/memo-editor';
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
      if (!ih.canEditHyperlink()) throw new Error('본문이나 표 셀의 글자에서 메모를 열어 주세요.');
      const selection = ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
      const range = hyperlinkRange(ih.getCursorPosition(), selection);
      const generation = services.wasm.documentGeneration;
      const context = JSON.stringify(services.wasm.getHyperlinkContext(range.target));
      let memos = services.wasm.getMemos();
      let fingerprint = JSON.stringify(memos);
      const dialog = new MemoDialog(memos, edit => {
        if (services.wasm.documentGeneration !== generation || services.getInputHandler() !== ih
          || !services.getContext().isEditable || services.getContext().isFormMode || !ih.canEditHyperlink()
          || JSON.stringify(services.wasm.getHyperlinkContext(range.target)) !== context
          || JSON.stringify(services.wasm.getMemos()) !== fingerprint) {
          throw new Error('문서나 편집 상태가 바뀌었습니다. 메모 창을 닫고 다시 열어 주세요.');
        }
        validateMemoEdit(edit, memos);
        if (edit.kind === 'update' && memos.find(m => m.id === edit.id)?.text === edit.text) return memos;
        const before = ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
        let applied = false;
        ih.executeOperation({ kind: 'snapshot', operationType: 'editMemo',
          selectionBefore: before ? { ...before, blockPhase: null } : null,
          operation: wasm => {
            if (edit.kind === 'insert') wasm.insertMemo(range.target, range.start, range.end, edit.author, edit.text);
            else if (edit.kind === 'update') wasm.updateMemo(edit.id, edit.text);
            else wasm.removeMemo(edit.id);
            applied = true;
            return range.pos;
          },
        });
        if (!applied) throw new Error('메모를 반영하지 못했습니다.');
        memos = services.wasm.getMemos(); fingerprint = JSON.stringify(memos);
        return memos;
      });
      dialog.afterClose = () => ih.focus(); dialog.show();
    } catch (error) { showToast({ message: error instanceof Error ? error.message : String(error) }); }
  },
};
