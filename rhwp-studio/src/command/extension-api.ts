import type { CommandDef } from './types';
import type { CommandRegistry } from './registry';
import type { CommandDispatcher } from './dispatcher';
import { formatShortcutLabel } from '@/engine/navigation-keymap';

/**
 * 고객사 확장 API
 *
 * 고객사(외부)에서 커스텀 커맨드를 등록하고 메뉴 항목을 추가/제거할 수 있다.
 * 확장 커맨드 ID는 반드시 `ext:` 접두사를 사용해야 한다.
 */
export class StudioExtensionAPI {
  constructor(
    private registry: CommandRegistry,
    private dispatcher: CommandDispatcher,
    private menuContainer: HTMLElement,
  ) {}

  /** 확장 커맨드 등록 (ID는 반드시 ext: 접두사 필요) */
  registerCommand(def: CommandDef): void {
    if (!def.id.startsWith('ext:')) {
      throw new Error(`확장 커맨드 ID는 "ext:" 접두사가 필요합니다: ${def.id}`);
    }
    this.registry.register(def);
  }

  /** 확장 커맨드 제거 */
  removeCommand(id: string): void {
    if (!id.startsWith('ext:')) return;
    this.registry.unregister(id);
  }

  /** 커맨드 실행 */
  executeCommand(commandId: string, params?: Record<string, unknown>): boolean {
    return this.dispatcher.dispatch(commandId, params);
  }

  /** 메뉴에 항목 추가 */
  addMenuItem(menuId: string, commandId: string, position?: 'top' | 'bottom'): void {
    const menuItem = this.menuContainer.querySelector(
      `.menu-item[data-menu="${menuId}"] .md-body`,
    );
    if (!menuItem) {
      console.warn(`[ExtensionAPI] 메뉴를 찾을 수 없습니다: ${menuId}`);
      return;
    }

    const def = this.registry.get(commandId);
    if (!def) {
      console.warn(`[ExtensionAPI] 미등록 커맨드: ${commandId}`);
      return;
    }

    const item = document.createElement('div');
    item.className = 'md-item';
    item.dataset.cmd = commandId;
    const label = document.createElement('span');
    label.className = 'md-label';
    label.textContent = def.label;
    item.appendChild(label);
    if (def.shortcutLabel) {
      const shortcut = document.createElement('span');
      shortcut.className = 'md-shortcut';
      shortcut.textContent = formatShortcutLabel(def.shortcutLabel);
      item.appendChild(shortcut);
    }

    if (position === 'top') {
      menuItem.prepend(item);
    } else {
      menuItem.appendChild(item);
    }
  }

  /** 메뉴에서 항목 제거 */
  removeMenuItem(commandId: string): void {
    const item = this.menuContainer.querySelector(`.md-item[data-cmd="${commandId}"]`);
    item?.remove();
  }
}
