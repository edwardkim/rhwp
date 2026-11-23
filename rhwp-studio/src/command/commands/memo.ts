import type { DocumentPosition } from '@/core/types';
import { showConfirm } from '@/ui/confirm-dialog';
import type { CommandDef } from '../types';
import { hyperlinkRange } from '@/core/hyperlink';
import { StudioCommandError } from '@/core/command-error';
import { countMemoReplies, MemoEditError, memoRangeAtCaret, validateMemoEdit, type MemoEdit, type MemoComposeRequest } from '@/core/memo-editor';
import { MemoDialog } from '@/ui/memo-dialog';
import { showToast } from '@/ui/toast';
import { t } from '../../i18n/index.ts';

export const memoCommand: CommandDef = {
  id: 'insert:comment', label: t('command.insert.comment.label'), icon: 'icon-comment', opensDialog: true,
  canExecute: ctx => ctx.hasDocument && ctx.isEditable && !ctx.isFormMode
    && !ctx.inCellSelectionMode && !ctx.inPictureObjectSelection && !ctx.inTableObjectSelection,
  execute(services, params) {
    const ih = services.getInputHandler();
    if (!ih || document.querySelector('[data-memo-dialog]')) return;
    try {
      if (!ih.canEditHyperlink()) throw new Error(t('dialog.memo.invalidTarget'));
      const wasm = services.wasm;
      if (params?.generation !== undefined && params.generation !== wasm.documentGeneration) return;
      const pointer = params?.position as DocumentPosition | undefined;
      const selection = pointer ? params?.selection as ReturnType<typeof ih.getSelection> ?? null
        : ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
      let insertion: { range: ReturnType<typeof hyperlinkRange>; context: string } | null = null;
      try {
        const range = hyperlinkRange(pointer ?? ih.getCursorPosition(), selection);
        const { section, para, cellPath } = range.target;
        const context = wasm.getHyperlinkContext(range.target);
        if (cellPath.length === 0) {
          range.start = wasm.logicalToTextOffset(section, para, range.start);
          range.end = wasm.logicalToTextOffset(section, para, range.end);
        }
        const anchor = selection ? range : memoRangeAtCaret(context.text, range.start);
        insertion = { range: { ...range, start: anchor.start, end: anchor.end }, context: JSON.stringify(context) };
      } catch {
        // 새 메모 위치를 확인할 수 없는 경우에도 기존 메모는 ID로 관리할 수 있다.
      }
      const generation = wasm.documentGeneration;
      let memos = wasm.getMemos();
      let fingerprint = JSON.stringify(memos);
      const requestedId = typeof params?.memoId === 'number' ? params.memoId : null;
      const apply = (edit: MemoEdit) => {
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
              } else if (edit.kind === 'reply') {
                wasm.insertMemoReply(edit.id, edit.author, edit.text);
              } else if (edit.kind === 'update') {
                wasm.updateMemo(edit.id, edit.text);
              } else {
                wasm.removeMemo(edit.id);
              }
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
      };
      const compose = params?.compose;
      const chosen = memos.find(memo => memo.id === requestedId);
      if (compose === 'remove' && chosen?.editable) {
        const count = countMemoReplies(chosen.id, memos);
        void showConfirm(t('dialog.memo.delete'), count
          ? t('dialog.memo.deleteThreadConfirm', { p1: count }) : t('dialog.memo.deleteOneConfirm'))
          .then(ok => { if (ok) apply({ kind: 'remove', id: chosen.id, confirmedReplies: count }); })
          .catch(error => showToast({ message: String(error) }));
        return;
      }
      if (compose === 'insert' ? insertion !== null : (compose === 'update' || compose === 'reply') && chosen?.editable) {
        if (compose === 'reply' && chosen?.parentId !== null) return;
        const kind = compose as 'insert' | 'update' | 'reply';
        const range = kind === 'insert' ? insertion?.range : undefined;
        const request: MemoComposeRequest = {
          anchor: range && {
            sectionIndex: range.target.section,
            paraIndex: range.target.para,
            startCharIdx: range.start,
            endCharIdx: range.end,
            path: range.target.cellPath.map(([controlIndex, cellIndex, paraIndex], i, path) =>
              range.pos.isTextBox && i === path.length - 1
                ? { type: 'textbox', controlIndex, paraIndex }
                : { type: 'cell', controlIndex, cellIndex, paraIndex }),
          },
          kind, cardId: chosen?.parentId ?? chosen?.id ?? null,
          restoreFocus: () => ih.focus(),
          author: kind === 'update' ? chosen!.author : '',
          text: kind === 'update' ? chosen!.text : '',
          submit(author, text) {
            if (kind === 'insert') apply({ kind, author, text });
            else if (kind === 'reply') apply({ kind, id: chosen!.id, author, text });
            else apply({ kind, id: chosen!.id, text });
          },
        };
        services.eventBus.emit('memo-compose', request);
        return;
      }
      if (compose === 'insert') throw new Error(t('dialog.memo.invalidTarget'));
      const dialog = new MemoDialog(memos, insertion !== null, memo => wasm.getMemoDisplayPage(memo), apply, {
        initialId: requestedId,
        goTo(memo) {
          if (wasm.documentGeneration !== generation || services.getInputHandler() !== ih) return false;
          const current = wasm.getMemos().find(item => item.id === memo.id);
          const target = current ? wasm.getMemoTarget(current) : null;
          return !!target && ih.focusMemoRange(target.start, target.end, target.rect);
        },
      });
      dialog.afterClose = () => ih.focus(); dialog.show();
    } catch (error) { showToast({ message: error instanceof Error ? error.message : String(error) }); }
  },
};

export const memoInsertCommand: CommandDef = {
  ...memoCommand,
  id: 'insert:memo-here',
  execute: services => memoCommand.execute(services, { compose: 'insert' }),
};
