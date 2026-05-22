import type { CommandDispatcher } from '@/command/dispatcher';
import type { CommandRegistry } from '@/command/registry';
import { formatShortcutLabel } from '@/engine/navigation-keymap';

/** 컨텍스트 메뉴 항목 정의 */
export interface ContextMenuItem {
  type: 'command' | 'separator';
  commandId?: string;
  label?: string;
}

/**
 * 우클릭 컨텍스트 메뉴
 *
 * - show()로 화면 좌표에 메뉴 표시
 * - CommandDispatcher 연동: canExecute 체크로 비활성 항목 표시
 * - ESC / 외부 클릭으로 닫기
 */
export class ContextMenu {
  private el: HTMLDivElement | null = null;
  private escHandler: ((e: KeyboardEvent) => void) | null = null;
  private outsideHandler: ((e: MouseEvent) => void) | null = null;

  constructor(
    private dispatcher: CommandDispatcher,
    private registry: CommandRegistry,
  ) {}

  /** clientX/Y에 메뉴를 표시한다 */
  show(x: number, y: number, items: ContextMenuItem[]): void {
    this.hide();

    const menu = document.createElement('div');
    menu.className = 'context-menu';

    for (const item of items) {
      if (item.type === 'separator') {
        const sep = document.createElement('div');
        sep.className = 'md-sep';
        menu.appendChild(sep);
        continue;
      }

      const cmdId = item.commandId!;
      const def = this.registry.get(cmdId);
      if (!def) continue;

      const row = document.createElement('div');
      row.className = 'md-item';
      row.dataset.cmd = cmdId;

      // canExecute 체크
      if (!this.dispatcher.isEnabled(cmdId)) {
        row.classList.add('disabled');
      }

      // 레이블
      const labelSpan = document.createTextNode(item.label ?? def.label);
      row.appendChild(labelSpan);

      // 단축키 표시
      if (def.shortcutLabel) {
        const shortcut = document.createElement('span');
        shortcut.className = 'md-shortcut';
        shortcut.textContent = formatShortcutLabel(def.shortcutLabel);
        row.appendChild(shortcut);
      }

      // 클릭 핸들러
      row.addEventListener('click', (e) => {
        e.stopPropagation();
        if (row.classList.contains('disabled')) return;
        this.dispatcher.dispatch(cmdId);
        this.hide();
      });

      menu.appendChild(row);
    }

    document.body.appendChild(menu);
    this.el = menu;

    // 화면 경계 보정
    const rect = menu.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    if (x + rect.width > vw) x = vw - rect.width - 2;
    if (y + rect.height > vh) y = vh - rect.height - 2;
    if (x < 0) x = 0;
    if (y < 0) y = 0;
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;

    // ESC 닫기
    this.escHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        this.hide();
      }
    };
    document.addEventListener('keydown', this.escHandler, true);

    // 외부 클릭 닫기 (다음 이벤트 루프에서 등록)
    requestAnimationFrame(() => {
      this.outsideHandler = (e: MouseEvent) => {
        if (this.el && !this.el.contains(e.target as Node)) {
          this.hide();
        }
      };
      document.addEventListener('mousedown', this.outsideHandler, true);
    });
  }

  /** 메뉴를 닫는다 */
  hide(): void {
    if (this.escHandler) {
      document.removeEventListener('keydown', this.escHandler, true);
      this.escHandler = null;
    }
    if (this.outsideHandler) {
      document.removeEventListener('mousedown', this.outsideHandler, true);
      this.outsideHandler = null;
    }
    this.el?.remove();
    this.el = null;
  }

  /** 메뉴가 열려있는가? */
  get isOpen(): boolean {
    return this.el !== null;
  }

  dispose(): void {
    this.hide();
  }
}
