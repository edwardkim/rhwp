/**
 * [#6053] 다이얼로그 안에서만 쓰는 우클릭 메뉴 — 전역 커맨드를 만들지 않는다.
 *
 * 왜 `ContextMenu`(같은 폴더)를 쓰지 않는가: 그쪽은 **커맨드 레지스트리 구동**이라
 * `registry.get(cmdId)` 가 없으면 항목을 조용히 버린다. 그리드 조작 6종을 전역 커맨드로
 * 등재하면 커맨드 표면과 `[data-cmd]` 마크업 드리프트 가드가 함께 딸려오는데, 이 항목들은
 * 다이얼로그 밖에서 뜻이 없다. 그래서 동작만 새로 두고 **CSS 는 그대로 재사용**한다
 * (`context-menu`/`md-item`/`md-sep` — 신규 CSS 0줄).
 *
 * 모달 위에 뜬다: `.context-menu` z-index 20000 > `.modal-overlay` 10000.
 *
 * 키 처리가 이 파일의 핵심이다. `ModalDialog` 는 `document` capture 에서 **모든 키를
 * `stopPropagation()`** 하고 ESC 를 닫기로, 비-입력 요소의 Enter 를 [확인] 클릭으로 바꾼다.
 * 그래서 이 메뉴는 `window` capture 에 달아야 하고(capture 는 `window → document` 순),
 * 자기가 처리한 키는 전파를 끊어야 한다. 안 그러면 ESC 가 다이얼로그까지 닫고, 메뉴에서
 * 누른 Enter 가 다이얼로그를 확인해 버린다.
 */

export interface LocalMenuItem {
  type: 'command' | 'separator';
  /** 표시 문구. `separator` 면 없다. */
  label?: string;
  /** 비활성 사유 — 있으면 항목이 꺼지고 `title` 로 이유를 말한다. */
  disabledReason?: string;
  /** 활성 상태에서 붙는 안내 — 막지는 않고 알려만 준다(예: 원형에 계열 추가). */
  note?: string;
  run?: () => void;
}

export class LocalContextMenu {
  private el: HTMLDivElement | null = null;
  private keyHandler: ((e: KeyboardEvent) => void) | null = null;
  private outsideHandler: ((e: MouseEvent) => void) | null = null;

  /** clientX/Y 에 메뉴를 연다. 이미 열려 있으면 먼저 닫는다. */
  show(x: number, y: number, items: LocalMenuItem[]): void {
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

      const row = document.createElement('div');
      row.className = 'md-item';
      row.appendChild(document.createTextNode(item.label ?? ''));

      if (item.disabledReason) {
        row.classList.add('disabled');
        row.title = item.disabledReason;
      } else if (item.note) {
        row.title = item.note;
      }

      row.addEventListener('click', (e) => {
        e.stopPropagation();
        if (row.classList.contains('disabled')) return;
        const run = item.run;
        this.hide();
        run?.();
      });

      menu.appendChild(row);
    }

    document.body.appendChild(menu);
    this.el = menu;

    // 화면 경계 보정 — ContextMenu 와 같은 규칙.
    const rect = menu.getBoundingClientRect();
    if (x + rect.width > window.innerWidth) x = window.innerWidth - rect.width - 2;
    if (y + rect.height > window.innerHeight) y = window.innerHeight - rect.height - 2;
    if (x < 0) x = 0;
    if (y < 0) y = 0;
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;

    // window capture — document 에 달린 ModalDialog 핸들러보다 먼저 돌아야 한다.
    this.keyHandler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' && e.key !== 'Enter') return;
      // 메뉴가 먹은 키는 다이얼로그로 넘기지 않는다(ESC=다이얼로그 닫기 / Enter=[확인]).
      e.stopPropagation();
      e.preventDefault();
      this.hide();
    };
    window.addEventListener('keydown', this.keyHandler, true);

    // 외부 클릭 닫기 — 메뉴를 연 그 클릭에 바로 닫히지 않도록 다음 프레임에 등록한다.
    requestAnimationFrame(() => {
      if (!this.el) return;
      this.outsideHandler = (e: MouseEvent) => {
        if (this.el && !this.el.contains(e.target as Node)) this.hide();
      };
      document.addEventListener('mousedown', this.outsideHandler, true);
    });
  }

  hide(): void {
    if (this.keyHandler) {
      window.removeEventListener('keydown', this.keyHandler, true);
      this.keyHandler = null;
    }
    if (this.outsideHandler) {
      document.removeEventListener('mousedown', this.outsideHandler, true);
      this.outsideHandler = null;
    }
    this.el?.remove();
    this.el = null;
  }

  get isOpen(): boolean {
    return this.el !== null;
  }

  dispose(): void {
    this.hide();
  }
}
