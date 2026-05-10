import type { CommandDef } from '../types';
import { FieldEditDialog } from '@/ui/field-edit-dialog';
import { FindDialog } from '@/ui/find-dialog';
import { GotoDialog } from '@/ui/goto-dialog';

/** 검색 대화상자 싱글톤 — 열려 있으면 재사용 */
let findDialogInstance: FindDialog | null = null;

export const editCommands: CommandDef[] = [
  {
    id: 'edit:undo',
    label: '되돌리기',
    icon: 'icon-undo',
    shortcutLabel: 'Ctrl+Z',
    canExecute: (ctx) => ctx.hasDocument && ctx.canUndo,
    execute(services) {
      services.getInputHandler()?.performUndo();
    },
  },
  {
    id: 'edit:redo',
    label: '다시 실행',
    icon: 'icon-redo',
    shortcutLabel: 'Ctrl+Shift+Z',
    canExecute: (ctx) => ctx.hasDocument && ctx.canRedo,
    execute(services) {
      services.getInputHandler()?.performRedo();
    },
  },
  {
    id: 'edit:cut',
    label: '오려 두기',
    icon: 'icon-cut',
    shortcutLabel: 'Ctrl+X',
    canExecute: (ctx) => ctx.hasDocument && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performCut();
    },
  },
  {
    id: 'edit:copy',
    label: '복사하기',
    icon: 'icon-copy',
    shortcutLabel: 'Ctrl+C',
    canExecute: (ctx) => ctx.hasDocument && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performCopy();
    },
  },
  {
    id: 'edit:paste',
    label: '붙이기',
    icon: 'icon-paste',
    shortcutLabel: 'Ctrl+V',
    canExecute: (ctx) => ctx.hasDocument,
    execute() {
      document.execCommand('paste');
    },
  },
  {
    id: 'edit:format-copy',
    label: '모양 복사',
    icon: 'icon-format-copy',
    shortcutLabel: 'Ctrl+Alt+C',
    canExecute: () => false, // 미구현
    execute() { /* TODO */ },
  },
  {
    id: 'edit:delete',
    label: '지우기',
    icon: 'icon-delete',
    shortcutLabel: 'Ctrl+E',
    canExecute: (ctx) => ctx.hasDocument && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
    execute(services) {
      services.getInputHandler()?.performDelete();
    },
  },
  {
    id: 'edit:select-all',
    label: '모두 선택',
    icon: 'icon-select-all',
    shortcutLabel: 'Ctrl+A',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      services.getInputHandler()?.performSelectAll();
    },
  },
  {
    id: 'edit:find',
    label: '찾기(F)',
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
    label: '찾아 바꾸기(E)',
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
    label: '다시 찾기(X)',
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
        );
        if (result.found) {
          ih.moveCursorTo({
            sectionIndex: result.sec!,
            paragraphIndex: result.para!,
            charOffset: result.charOffset!,
          });
          const cursor = (ih as any).cursor;
          if (cursor) {
            cursor.setAnchor();
            cursor.moveTo({
              sectionIndex: result.sec!,
              paragraphIndex: result.para!,
              charOffset: result.charOffset! + result.length!,
            });
          }
          (ih as any).updateCaret?.();
        }
      }
    },
  },
  {
    id: 'edit:goto',
    label: '찾아가기(G)',
    shortcutLabel: 'Alt+G',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const dialog = new GotoDialog(services);
      dialog.show();
    },
  },
  {
    id: 'field:edit',
    label: '누름틀 고치기(E)...',
    shortcutLabel: 'Ctrl+N,K',
    canExecute: (ctx) => ctx.hasDocument && ctx.inField,
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
      dialog.onApply = (newProps) => {
        console.log('[field:edit] apply:', newProps);
        const result = services.wasm.updateClickHereProps(
          fi.fieldId, newProps.guide, newProps.memo, newProps.name, newProps.editable,
        );
        console.log('[field:edit] updateResult:', result);
        if (result.ok) {
          services.eventBus.emit('document-changed');
        }
      };
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
    label: '누름틀 지우기(J)',
    canExecute: (ctx) => ctx.hasDocument && ctx.inField,
    execute(services) {
      const ih = services.getInputHandler();
      if (ih) (ih as any).removeCurrentField();
    },
  },
];
