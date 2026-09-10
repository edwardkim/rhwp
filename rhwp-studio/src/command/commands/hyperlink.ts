import type { CommandDef } from '../types';
import { HyperlinkDialog, confirmHyperlinkEdit } from '@/ui/hyperlink-dialog';
import { hyperlinkRange, selectedHyperlink, applyHyperlinkFormat } from '@/core/hyperlink';
import { showToast } from '@/ui/toast';

export const hyperlinkCommand: CommandDef = {
  id: 'insert:hyperlink',
  label: '하이퍼링크',
  icon: 'icon-hyperlink',
  shortcutLabel: 'Ctrl+K+H',
  opensDialog: true,
  canExecute: ctx => ctx.hasDocument && ctx.isEditable && !ctx.isFormMode
    && !ctx.inCellSelectionMode && !ctx.inPictureObjectSelection && !ctx.inTableObjectSelection,
  execute(services, params) {
    const ih = services.getInputHandler();
    if (!ih) return;
    try {
      if (!ih.canEditHyperlink()) throw new Error('본문·표 셀·글상자의 글자에서 하이퍼링크를 편집해 주세요.');
      const selection = ih.hasSelection() ? structuredClone(ih.getSelection()) : null;
      const range = hyperlinkRange(ih.getCursorPosition(), selection);
      const { target, pos, start, end } = range;
      const context = services.wasm.getHyperlinkContext(target);
      const generation = services.wasm.documentGeneration;
      const existing = selectedHyperlink(context.links, start, end);
      const canInsertText = !existing && start === end;
      const text = existing?.text ?? Array.from(context.text).slice(start, end).join('');
      const applyEdit = (edit: import('@/ui/hyperlink-dialog').HyperlinkEdit) => {
        if (services.wasm.documentGeneration !== generation || services.getInputHandler() !== ih || !services.getContext().isEditable
          || services.getContext().isFormMode || !ih.canEditHyperlink()) {
          throw new Error('편집 상태가 바뀌었습니다. 대화상자를 닫고 다시 열어 주세요.');
        }
        // 모달을 연 뒤 외부 편집이 일어났다면 오래된 범위를 적용하지 않는다.
        if (JSON.stringify(services.wasm.getHyperlinkContext(target)) !== JSON.stringify(context)) {
          throw new Error('문서 내용이 바뀌었습니다. 대화상자를 닫고 다시 열어 주세요.');
        }
        if (edit.kind === 'save' && existing?.uri === edit.uri && existing.text === edit.text) return;
        if (edit.kind === 'save' && (canInsertText || edit.text !== text) && (!edit.text.trim() || /[\r\n\t\x00-\x1f\x7f]/.test(edit.text))) {
          throw new Error('표시할 글자를 한 줄로 입력해 주세요.');
        }
        let applied = false;
        ih.executeOperation({
          kind: 'snapshot',
          operationType: 'editHyperlink',
          selectionBefore: selection ? { ...selection, blockPhase: null } : null,
          operation: wasm => {
            if (edit.kind === 'remove') {
              if (!existing) return null;
              wasm.removeHyperlink(target, existing.fieldId);
              applyHyperlinkFormat(wasm, target, existing.start, existing.end, null);
            } else if (existing) {
              wasm.updateHyperlink(target, existing.fieldId, edit.uri);
              if (edit.text !== text) wasm.replaceHyperlinkText(target, existing.fieldId, edit.text);
            } else {
              let linkEnd = end;
              if (canInsertText) {
                if (target.cellPath.length) {
                  const path = target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) => ({ controlIndex, cellIndex, cellParaIndex }));
                  wasm.insertTextInCellByPath(target.section, target.para, JSON.stringify(path), start, edit.text);
                } else {
                  wasm.insertText(target.section, target.para, start, edit.text);
                }
                linkEnd = start + Array.from(edit.text).length;
              }
              // 문자열 삽입 뒤 필드 검증이 실패해도 snapshot이 전체 작업을 복원한다.
              const fieldId = wasm.insertHyperlink(target, start, linkEnd, edit.uri);
              if (!canInsertText && edit.text !== text) {
                wasm.replaceHyperlinkText(target, fieldId, edit.text);
                linkEnd = start + Array.from(edit.text).length;
              }
              applyHyperlinkFormat(wasm, target, start, linkEnd, '#0000ff');
              applied = true;
              return { ...pos, charOffset: linkEnd };
            }
            applied = true;
            return { ...pos, charOffset: existing && edit.kind === 'save' ? existing.start + Array.from(edit.text).length : existing?.end ?? end };
          },
        });
        if (!applied) throw new Error('하이퍼링크를 적용할 수 없습니다. 편집 모드를 확인해 주세요.');
      };
      if (params?.action === 'remove') {
        if (existing) applyEdit({ kind: 'remove' });
        ih.focus();
        return;
      }
      const open = () => {
        if (services.wasm.documentGeneration !== generation || services.getInputHandler() !== ih) return;
        const dialog = new HyperlinkDialog({ text, uri: existing?.uri ?? '', existing: !!existing, canInsertText }, applyEdit);
        dialog.afterClose = () => ih.focus();
        dialog.show();
      };
      if (existing && params?.action !== 'edit') confirmHyperlinkEdit(open, () => ih.focus());
      else open();
    } catch (error) {
      showToast({ message: error instanceof Error ? error.message : String(error) });
    }
  },
};

export const editHyperlinkCommand: CommandDef = {
  ...hyperlinkCommand, id: 'hyperlink:edit', label: '하이퍼링크 고치기', shortcutLabel: undefined,
  execute: services => hyperlinkCommand.execute(services, { action: 'edit' }),
};
export const removeHyperlinkCommand: CommandDef = {
  ...hyperlinkCommand, id: 'hyperlink:remove', label: '하이퍼링크 지우기', shortcutLabel: undefined, opensDialog: false,
  execute: services => hyperlinkCommand.execute(services, { action: 'remove' }),
};
