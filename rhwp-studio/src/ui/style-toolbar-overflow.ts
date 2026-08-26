export const STYLE_TOOLBAR_FULL_ROW_MIN = 976;
export const STYLE_TOOLBAR_COMMAND_INLINE_MIN = 460;
export const STYLE_TOOLBAR_OVERFLOW_QUERY = `(max-width: ${STYLE_TOOLBAR_COMMAND_INLINE_MIN - 1}px)`;

const PARAGRAPH_BUTTON_SELECTOR = '.sb-paragraph-ribbon-group .sb-btn';

/**
 * 좁은 화면에서 문단 명령을 같은 DOM 그대로 더보기 panel에 노출한다.
 *
 * CSS가 460px 이상에서는 host/panel을 display:contents로 풀기 때문에 runtime reparent나
 * command 복제가 필요 없다. controller는 좁은 화면의 open/focus/접근성 상태만 소유한다.
 */
export class StyleToolbarOverflowController {
  private readonly mediaQuery: MediaQueryList;
  private readonly paragraphButtons: HTMLButtonElement[];
  private readonly stateObserver: MutationObserver;
  private open = false;

  private readonly onTriggerClick = (): void => {
    this.setOpen(!this.open, !this.open);
  };

  private readonly onTriggerKeyDown = (event: KeyboardEvent): void => {
    if (event.key !== 'ArrowDown') return;
    event.preventDefault();
    this.setOpen(true, true);
  };

  private readonly onPanelClick = (event: MouseEvent): void => {
    const target = event.target;
    if (!(target instanceof Element) || !target.closest(PARAGRAPH_BUTTON_SELECTOR)) return;
    this.setOpen(false, false, true);
  };

  private readonly onDocumentPointerDown = (event: PointerEvent): void => {
    const target = event.target;
    if (!this.open || !(target instanceof Node) || this.host.contains(target)) return;
    this.setOpen(false);
  };

  private readonly onWindowKeyDown = (event: KeyboardEvent): void => {
    if (!this.open || event.key !== 'Escape') return;
    event.preventDefault();
    event.stopPropagation();
    this.setOpen(false, false, true);
  };

  private readonly onMediaChange = (): void => {
    this.syncLayoutMode();
  };

  constructor(
    private readonly host: HTMLElement,
    private readonly trigger: HTMLButtonElement,
    private readonly panel: HTMLElement,
    matchMedia: (query: string) => MediaQueryList = window.matchMedia.bind(window),
  ) {
    this.mediaQuery = matchMedia(STYLE_TOOLBAR_OVERFLOW_QUERY);
    this.paragraphButtons = Array.from(
      panel.querySelectorAll<HTMLButtonElement>(PARAGRAPH_BUTTON_SELECTOR),
    );
    this.stateObserver = new MutationObserver(() => this.syncIndicator());

    trigger.addEventListener('click', this.onTriggerClick);
    trigger.addEventListener('keydown', this.onTriggerKeyDown);
    panel.addEventListener('click', this.onPanelClick);
    document.addEventListener('pointerdown', this.onDocumentPointerDown, true);
    window.addEventListener('keydown', this.onWindowKeyDown, true);
    this.mediaQuery.addEventListener('change', this.onMediaChange);
    this.stateObserver.observe(panel, {
      attributes: true,
      subtree: true,
      attributeFilter: ['class', 'disabled', 'aria-pressed'],
    });

    this.syncLayoutMode();
    this.syncIndicator();
  }

  private syncLayoutMode(): void {
    this.open = false;
    this.host.classList.remove('open');
    this.trigger.setAttribute('aria-expanded', 'false');
    this.panel.hidden = this.mediaQuery.matches;
  }

  private setOpen(next: boolean, focusFirst = false, returnFocus = false): void {
    const open = this.mediaQuery.matches && next;
    this.open = open;
    this.host.classList.toggle('open', open);
    this.trigger.setAttribute('aria-expanded', String(open));
    this.panel.hidden = this.mediaQuery.matches ? !open : false;

    if (open && focusFirst) {
      requestAnimationFrame(() => this.paragraphButtons.find(button => !button.disabled)?.focus());
    } else if (!open && returnFocus) {
      this.trigger.focus();
    }
  }

  private syncIndicator(): void {
    const hasActiveCommand = this.paragraphButtons.some(button =>
      button.classList.contains('active') || button.getAttribute('aria-pressed') === 'true',
    );
    const allCommandsDisabled = this.paragraphButtons.length > 0
      && this.paragraphButtons.every(button => button.disabled);

    this.trigger.classList.toggle('active', hasActiveCommand);
    this.trigger.disabled = allCommandsDisabled;
    this.trigger.setAttribute(
      'aria-label',
      hasActiveCommand ? '문단 정렬 더보기, 현재 정렬 포함' : '문단 정렬 더보기',
    );
  }

  dispose(): void {
    this.setOpen(false);
    this.trigger.removeEventListener('click', this.onTriggerClick);
    this.trigger.removeEventListener('keydown', this.onTriggerKeyDown);
    this.panel.removeEventListener('click', this.onPanelClick);
    document.removeEventListener('pointerdown', this.onDocumentPointerDown, true);
    window.removeEventListener('keydown', this.onWindowKeyDown, true);
    this.mediaQuery.removeEventListener('change', this.onMediaChange);
    this.stateObserver.disconnect();
  }
}

export function initStyleToolbarOverflow(container: HTMLElement): StyleToolbarOverflowController {
  const host = container.querySelector<HTMLElement>('.sb-overflow-host');
  const trigger = container.querySelector<HTMLButtonElement>('#btn-style-overflow');
  const panel = container.querySelector<HTMLElement>('#style-overflow-panel');
  if (!host || !trigger || !panel) {
    throw new Error('서식 도구 모음 더보기 DOM 계약이 누락되었습니다.');
  }
  return new StyleToolbarOverflowController(host, trigger, panel);
}
