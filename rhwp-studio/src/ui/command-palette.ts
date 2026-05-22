import type { CommandRegistry } from '@/command/registry';
import type { CommandDispatcher } from '@/command/dispatcher';
import type { CommandDef } from '@/command/types';
import { formatShortcutLabel } from '@/engine/navigation-keymap';

/**
 * `/` 커맨드 팔레트
 *
 * 편집 영역에서 `/` 키를 누르면 열리는 검색형 커맨드 실행창.
 * Notion/Linear/GitHub 패턴: 한글/영문 레이블 + 단축키로 필터링.
 */
export class CommandPalette {
  private overlay: HTMLDivElement | null = null;
  private input: HTMLInputElement | null = null;
  private list: HTMLDivElement | null = null;
  private items: CommandDef[] = [];
  private selectedIdx = 0;
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(
    private registry: CommandRegistry,
    private dispatcher: CommandDispatcher,
  ) {}

  /** 팔레트를 열고 검색 입력에 포커스 */
  open(): void {
    if (this.overlay) return; // 이미 열림

    this.items = this.buildItems();
    this.selectedIdx = 0;

    this.overlay = document.createElement('div');
    this.overlay.className = 'cp-overlay';

    const panel = document.createElement('div');
    panel.className = 'cp-panel';

    // 입력 영역
    const inputWrap = document.createElement('div');
    inputWrap.className = 'cp-input-wrap';

    const slash = document.createElement('span');
    slash.className = 'cp-slash';
    slash.textContent = '/';

    this.input = document.createElement('input');
    this.input.type = 'text';
    this.input.className = 'cp-input';
    this.input.placeholder = '커맨드 검색...';
    this.input.autocomplete = 'off';
    this.input.spellcheck = false;

    inputWrap.appendChild(slash);
    inputWrap.appendChild(this.input);
    panel.appendChild(inputWrap);

    // 결과 목록
    this.list = document.createElement('div');
    this.list.className = 'cp-list';
    panel.appendChild(this.list);

    this.overlay.appendChild(panel);
    document.body.appendChild(this.overlay);

    this.renderList(this.items);

    // 입력 이벤트
    this.input.addEventListener('input', () => {
      this.selectedIdx = 0;
      const filtered = this.filter(this.input!.value);
      this.renderList(filtered);
    });

    // 키보드 캡처
    this.captureHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        this.close();
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        e.stopPropagation();
        this.moveSelection(1);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        e.stopPropagation();
        this.moveSelection(-1);
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        e.stopPropagation();
        this.executeSelected();
        return;
      }
      // 그 외 키는 input으로 전달 허용 — stopPropagation만 (편집 영역 방지)
      e.stopPropagation();
    };
    document.addEventListener('keydown', this.captureHandler, true);

    // 오버레이 바깥 클릭 시 닫기
    this.overlay.addEventListener('mousedown', (e) => {
      if (e.target === this.overlay) this.close();
    });

    this.input.focus();
  }

  close(): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this.overlay?.remove();
    this.overlay = null;
    this.input = null;
    this.list = null;
  }

  isOpen(): boolean {
    return this.overlay !== null;
  }

  // ─── private ────────────────────────────────────────────────

  /** 팔레트에 노출할 커맨드 목록 구성 (disabled 제외) */
  private buildItems(): CommandDef[] {
    const all: CommandDef[] = [];
    for (const id of this.registry.getAllIds()) {
      const def = this.registry.get(id)!;
      // canExecute가 항상 false인 stub는 제외
      if (def.canExecute) {
        const ctx = {
          hasDocument: false, hasSelection: false, inTable: false,
          inCellSelectionMode: false, inTableObjectSelection: false,
          inPictureObjectSelection: false, inField: false, isEditable: true,
          canUndo: false, canRedo: false, zoom: 1.0, showControlCodes: false,
        };
        // canExecute(ctx=hasDocument:false) → false 인 경우도 목록에는 포함
        // (실행 시점에 dispatcher가 다시 판단)
      }
      all.push(def);
    }
    return all;
  }

  /** 검색어로 필터링 */
  private filter(query: string): CommandDef[] {
    const q = query.trim().toLowerCase();
    if (!q) return this.items;
    return this.items.filter(def => {
      if (def.label.toLowerCase().includes(q)) return true;
      if (def.id.toLowerCase().includes(q)) return true;
      if (def.shortcutLabel && (def.shortcutLabel.toLowerCase().includes(q) || formatShortcutLabel(def.shortcutLabel).toLowerCase().includes(q))) return true;
      return false;
    });
  }

  private renderList(filtered: CommandDef[]): void {
    if (!this.list) return;
    this.list.replaceChildren();

    if (filtered.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'cp-empty';
      empty.textContent = '검색 결과 없음';
      this.list.appendChild(empty);
      return;
    }

    filtered.forEach((def, idx) => {
      const row = document.createElement('div');
      row.className = 'cp-item' + (idx === this.selectedIdx ? ' cp-item--selected' : '');
      row.dataset.idx = String(idx);

      const labelEl = document.createElement('span');
      labelEl.className = 'cp-item-label';
      labelEl.textContent = def.label;

      row.appendChild(labelEl);

      if (def.shortcutLabel) {
        const kbd = document.createElement('span');
        kbd.className = 'cp-item-shortcut';
        kbd.textContent = formatShortcutLabel(def.shortcutLabel);
        row.appendChild(kbd);
      }

      row.addEventListener('mousedown', (e) => {
        e.preventDefault();
        this.selectedIdx = idx;
        this.executeSelected(filtered);
      });

      row.addEventListener('mousemove', () => {
        this.selectedIdx = idx;
        this.updateSelection(filtered.length);
      });

      this.list!.appendChild(row);
    });

    // 현재 선택 항목이 보이도록 스크롤
    this.scrollToSelected();
  }

  private moveSelection(delta: number): void {
    if (!this.list) return;
    const count = this.list.querySelectorAll('.cp-item').length;
    if (count === 0) return;
    this.selectedIdx = (this.selectedIdx + delta + count) % count;
    this.updateSelection(count);
    this.scrollToSelected();
  }

  private updateSelection(count: number): void {
    if (!this.list) return;
    this.list.querySelectorAll('.cp-item').forEach((el, i) => {
      el.classList.toggle('cp-item--selected', i === this.selectedIdx);
    });
    void count; // suppress unused warning
  }

  private scrollToSelected(): void {
    if (!this.list) return;
    const sel = this.list.querySelector('.cp-item--selected') as HTMLElement | null;
    sel?.scrollIntoView({ block: 'nearest' });
  }

  private executeSelected(filtered?: CommandDef[]): void {
    const items = filtered ?? this.filter(this.input?.value ?? '');
    const def = items[this.selectedIdx];
    if (!def) return;
    this.close();
    this.dispatcher.dispatch(def.id);
  }
}
