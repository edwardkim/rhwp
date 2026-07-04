import type { CommandRegistry } from '../command/registry';
import { formatShortcutLabel } from '../engine/navigation-keymap.ts';

export function syncMenuShortcutLabels(container: HTMLElement, registry: CommandRegistry): void {
  const items = container.querySelectorAll('.md-item[data-cmd]');
  for (const item of items) {
    const el = item as HTMLElement;
    const cmdId = el.dataset.cmd;
    if (!cmdId) continue;

    const shortcutLabel = registry.get(cmdId)?.shortcutLabel;
    if (!shortcutLabel) continue;

    let shortcut = el.querySelector('.md-shortcut') as HTMLElement | null;
    if (!shortcut) {
      shortcut = document.createElement('span');
      shortcut.className = 'md-shortcut';
      el.appendChild(shortcut);
    }
    shortcut.textContent = formatShortcutLabel(shortcutLabel);
  }
}
