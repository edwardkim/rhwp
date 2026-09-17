import type { CommandDef } from '../types';
import { FieldEditDialog } from '@/ui/field-edit-dialog';
import { FindDialog, navigateToSearchHit } from '@/ui/find-dialog';
import { GotoDialog } from '@/ui/goto-dialog';
import { HistoryDialog } from '@/ui/history-dialog';
import { CompareDialog } from '@/ui/compare-dialog';
import { CompareSessionStore } from '@/compare/session';
import { canExecuteFormatPaste } from '../format-paste-availability';

import { t } from '../../i18n/index.ts';
/** 검색 대화상자 싱글톤 — 열려 있으면 재사용 */
let findDialogInstance: FindDialog | null = null;
/** 싱글톤: 문서 이력 관리 대화상자 */
let historyDialogInstance: HistoryDialog | null = null;
/** 싱글톤: 두 파일 문서 비교 대화상자 */
let compareDialogInstance: CompareDialog | null = null;
/** 비교/이력 공용 세션 스토어 */
let compareSessionStore: CompareSessionStore | null = null;

export const editCommands: CommandDef[] = [
  {
    id: 'edit:undo',
    label: t('command.edit.undo.label'),
    icon: 'icon-undo',
    shortcutLabel: 'Ctrl+Z',
    canExecute: (ctx) => ctx.hasDocument && ctx.canUndo,
    execute(services) {
      services.getInputHandler()?.performUndo();
    },
  },
  {
    id: 'edit:redo',
    label: t('command.edit.redo.label'),
    icon: 'icon-redo',
    shortcutLabel: 'Ctrl+Shift+Z',
    canExecute: (ctx) => ctx.hasDocument && ctx.canRedo,
    execute(services) {
      services.getInputHandler()?.performRedo();
    },
  },
  {
    id: 'edit:cut',
    label: t('command.edit.cut.label'),
    icon: 'icon-cut',
    shortcutLabel: 'Ctrl+X',
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performCut();
    },
  },
  {
    id: 'edit:copy',
    label: t('command.edit.copy.label'),
    icon: 'icon-copy',
    shortcutLabel: 'Ctrl+C',
    canExecute: (ctx) => ctx.hasDocument && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performCopy();
    },
  },
  {
    id: 'edit:paste',
    label: t('command.edit.paste.label'),
    icon: 'icon-paste',
    shortcutLabel: 'Ctrl+V',
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode,
    execute(services) {
      services.getInputHandler()?.performPaste();
    },
  },
  {
    id: 'edit:format-copy',
    label: t('command.edit.formatCopy.label'),
    icon: 'icon-format-copy',
    shortcutLabel: 'Alt+C',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.performFormatCopy();
    },
  },
  {
    id: 'edit:format-paste',
    label: t('command.edit.formatPaste.label'),
    icon: 'icon-format-copy',
    canExecute: canExecuteFormatPaste,
    execute(services) {
      services.getInputHandler()?.performFormatPaste();
    },
  },
  {
    id: 'edit:delete',
    label: t('command.edit.delete.label'),
    icon: 'icon-delete',
    shortcutLabel: 'Ctrl+E',
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performDelete();
    },
  },
  {
    id: 'edit:select-all',
    label: t('command.edit.selectAll.label'),
    icon: 'icon-select-all',
    shortcutLabel: 'Ctrl+A',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.performSelectAll();
    },
  },
  {
    id: 'edit:find',
    opensDialog: true,
    label: t('command.edit.find.label'),
    icon: 'icon-find',
    shortcutLabel: 'Ctrl+F',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (findDialogInstance && findDialogInstance.isOpen()) {
        findDialogInstance.focusInput();
        return;
      }
      findDialogInstance = new FindDialog(services, 'find');
      findDialogInstance.show();
    },
  },
  {
    id: 'edit:find-replace',
    opensDialog: true,
    label: t('command.edit.findReplace.label'),
    icon: 'icon-find-replace',
    shortcutLabel: 'Ctrl+F2',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (findDialogInstance && findDialogInstance.isOpen()) {
        findDialogInstance.switchMode('replace');
        findDialogInstance.focusInput();
        return;
      }
      findDialogInstance = new FindDialog(services, 'replace');
      findDialogInstance.show();
    },
  },
  {
    id: 'edit:find-again',
    label: t('command.edit.findAgain.label'),
    shortcutLabel: 'Ctrl+L',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      if (findDialogInstance && findDialogInstance.isOpen()) {
        findDialogInstance.findNext();
      } else if (FindDialog.lastQuery) {
        // 대화상자 없이 WASM 직접 검색
        const ih = services.getInputHandler();
        if (!ih) return;
        const pos = ih.getCursorPosition();
        const result = services.wasm.searchText(
          FindDialog.lastQuery, pos.sectionIndex, pos.paragraphIndex,
          pos.charOffset, true, FindDialog.lastCaseSensitive,
          // [#3865] 대화상자와 같은 범위를 본다 — 한쪽만 셀을 다루면 같은 문서에서
          // Ctrl+F 는 표 안을 찾는데 F3 는 못 찾는 상태가 된다.
          true,
        );
        // 이동 규칙도 대화상자와 공유한다(셀 매치는 셀 좌표로 간다).
        navigateToSearchHit(ih, result);
      }
    },
  },
  {
    id: 'edit:compare-documents',
    opensDialog: true,
    label: t('command.edit.compareDocuments.label'),
    shortcutLabel: 'Alt+Shift+V',
    canExecute: () => true,
    execute(services) {
      if (!compareSessionStore) {
        compareSessionStore = new CompareSessionStore(services.eventBus);
      }
      if (historyDialogInstance?.isOpen()) historyDialogInstance.hide();
      if (compareDialogInstance && compareDialogInstance.isOpen()) return;
      compareDialogInstance = new CompareDialog(services, compareSessionStore);
      compareDialogInstance.show();
    },
  },
  {
    id: 'edit:document-history',
    opensDialog: true,
    label: t('command.edit.documentHistory.label'),
    shortcutLabel: 'Ctrl+Shift+H',
    canExecute: () => true,
    execute(services) {
      if (!compareSessionStore) {
        compareSessionStore = new CompareSessionStore(services.eventBus);
      }
      if (compareDialogInstance?.isOpen()) compareDialogInstance.hide();
      if (historyDialogInstance && historyDialogInstance.isOpen()) {
        return;
      }
      historyDialogInstance = new HistoryDialog(services, compareSessionStore);
      historyDialogInstance.show();
    },
  },
  {
    id: 'edit:goto',
    opensDialog: true,
    label: t('command.edit.goto.label'),
    shortcutLabel: 'Alt+G',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const dialog = new GotoDialog(services);
      dialog.show();
    },
  },
  {
    id: 'field:edit',
    opensDialog: true,
    label: t('command.field.edit.registryLabel'),
    shortcutLabel: 'Ctrl+M,K',
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode && ctx.inField,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      const fi = (ih as any).getFieldInfo?.();
      console.log('[field:edit] fieldInfo:', fi);
      if (!fi || fi.fieldId == null) return;
      const props = services.wasm.getClickHereProps(fi.fieldId);
      console.log('[field:edit] props:', props);
      if (!props.ok) return;

      const dialog = new FieldEditDialog();
      const restoreEditorFocus = () => {
        requestAnimationFrame(() => {
          (ih as any).updateCaret?.();
          ih.focus();
        });
      };
      dialog.onApply = (newProps) => {
        console.log('[field:edit] apply:', newProps);
        try {
          // [Task #2377] 누름틀 속성 갱신은 안내문 텍스트를 바꿀 수 있다(문자 수 변경) —
          // snapshot 으로 라우팅(일반 모드 전용 커맨드). 실패 시 throw 로 엔트리 생성을 막는다.
          ih.executeOperation({
            kind: 'snapshot',
            operationType: 'updateFieldProps',
            operation: (wasm) => {
              const result = wasm.updateClickHereProps(
                fi.fieldId, newProps.guide, newProps.memo, newProps.name, newProps.editable,
              );
              if (!result.ok) throw new Error('updateClickHereProps not ok');
              return ih.getCursorPosition();
            },
          });
          // [Task #2370] 수동 emit 제거 — 라우터의 'full' refresh(afterEdit)가 이미
          // 'document-mutated'/'document-changed' 를 emit 한다(insert:field 와 동형).
        } catch (err) {
          console.warn('[field:edit] 누름틀 고치기 실패:', err);
        }
      };
      dialog.onClose = restoreEditorFocus;
      dialog.showWith({
        guide: props.guide ?? '',
        memo: props.memo ?? '',
        name: props.name ?? '',
        editable: props.editable ?? true,
      });
    },
  },
  {
    id: 'field:remove',
    label: t('command.field.remove.registryLabel'),
    canExecute: (ctx) => ctx.hasDocument && !ctx.isFormMode && ctx.inField,
    execute(services) {
      const ih = services.getInputHandler();
      if (ih) (ih as any).confirmRemoveCurrentField?.();
    },
  },
];
