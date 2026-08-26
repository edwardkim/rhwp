const SCROLL_EPSILON = 1;
const GROUP_SELECTOR = ':scope > .tb-group';

type ScrollDirection = -1 | 1;

export function hasIconToolbarOverflow(contentWidth: number, availableWidth: number): boolean {
  return contentWidth - availableWidth > SCROLL_EPSILON;
}

export function adjacentIconToolbarGroupTarget(
  boundaries: readonly number[],
  current: number,
  direction: ScrollDirection,
  maximum: number,
): number {
  let target = direction > 0 ? maximum : 0;
  if (direction > 0) {
    target = boundaries.find(boundary => boundary > current + SCROLL_EPSILON) ?? target;
  } else {
    for (let index = boundaries.length - 1; index >= 0; index--) {
      if (boundaries[index] < current - SCROLL_EPSILON) {
        target = boundaries[index];
        break;
      }
    }
  }
  return Math.max(0, Math.min(target, maximum));
}

/**
 * 기본 도구 상자를 한 줄로 유지하면서 넘치는 기존 group DOM을 좌우로 탐색한다.
 *
 * 명령의 identity·상태·listener는 건드리지 않는다. 이 controller는 viewport의 scroll 위치와
 * 이동 버튼의 hidden/disabled 상태만 소유한다.
 */
export class IconToolbarScroller {
  private readonly root: HTMLElement;
  private readonly viewport: HTMLElement;
  private readonly track: HTMLElement;
  private readonly previousButton: HTMLButtonElement;
  private readonly nextButton: HTMLButtonElement;
  private readonly resizeObserver: ResizeObserver;
  private readonly modeObserver: MutationObserver;
  private animationFrame: number | null = null;
  private resetOnRefresh = false;

  private readonly onResize = (): void => {
    this.scheduleRefresh();
  };

  private readonly onModeMutation = (records: MutationRecord[]): void => {
    const modeChanged = records.some((record) => {
      const target = record.target;
      return target instanceof HTMLElement
        && target.parentElement === this.track
        && (target.classList.contains('tb-group') || target.classList.contains('tb-sep'))
        && (record.attributeName === 'style' || record.attributeName === 'hidden');
    });
    if (modeChanged) this.scheduleRefresh(true);
  };

  private readonly onScroll = (): void => {
    this.track.querySelectorAll('.tb-split.open').forEach((split) => {
      split.classList.remove('open');
      split.querySelector('.tb-split-arrow')?.setAttribute('aria-expanded', 'false');
    });
    this.updateNavigationState();
  };

  private readonly onPreviousClick = (): void => {
    this.scrollToAdjacentGroup(-1);
  };

  private readonly onNextClick = (): void => {
    this.scrollToAdjacentGroup(1);
  };

  private readonly onViewportKeyDown = (event: KeyboardEvent): void => {
    if (event.target !== this.viewport) return;
    if (event.key === 'ArrowLeft') {
      event.preventDefault();
      this.scrollToAdjacentGroup(-1);
    } else if (event.key === 'ArrowRight') {
      event.preventDefault();
      this.scrollToAdjacentGroup(1);
    } else if (event.key === 'Home') {
      event.preventDefault();
      this.scrollTo(0);
    } else if (event.key === 'End') {
      event.preventDefault();
      this.scrollTo(this.maxScrollLeft());
    }
  };

  private readonly onViewportFocusIn = (event: FocusEvent): void => {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const command = target.closest('.tb-btn') as HTMLElement | null;
    if (!command || !this.track.contains(command)) return;

    const viewportRect = this.viewport.getBoundingClientRect();
    const commandRect = command.getBoundingClientRect();
    if (commandRect.left < viewportRect.left) {
      this.viewport.scrollLeft -= viewportRect.left - commandRect.left;
    } else if (commandRect.right > viewportRect.right) {
      this.viewport.scrollLeft += commandRect.right - viewportRect.right;
    }
    this.updateNavigationState();
  };

  constructor(
    root: HTMLElement,
    viewport: HTMLElement,
    track: HTMLElement,
    previousButton: HTMLButtonElement,
    nextButton: HTMLButtonElement,
  ) {
    this.root = root;
    this.viewport = viewport;
    this.track = track;
    this.previousButton = previousButton;
    this.nextButton = nextButton;
    previousButton.addEventListener('click', this.onPreviousClick);
    nextButton.addEventListener('click', this.onNextClick);
    viewport.addEventListener('scroll', this.onScroll, { passive: true });
    viewport.addEventListener('keydown', this.onViewportKeyDown);
    viewport.addEventListener('focusin', this.onViewportFocusIn);

    this.resizeObserver = new ResizeObserver(this.onResize);
    this.resizeObserver.observe(root);
    this.resizeObserver.observe(viewport);
    this.resizeObserver.observe(track);

    this.modeObserver = new MutationObserver(this.onModeMutation);
    this.modeObserver.observe(track, {
      attributes: true,
      subtree: true,
      attributeFilter: ['style', 'hidden'],
    });

    this.refresh(true);
  }

  private scheduleRefresh(reset = false): void {
    this.resetOnRefresh ||= reset;
    if (this.animationFrame !== null) return;
    this.animationFrame = requestAnimationFrame(() => {
      this.animationFrame = null;
      const shouldReset = this.resetOnRefresh;
      this.resetOnRefresh = false;
      this.refresh(shouldReset);
    });
  }

  private refresh(reset: boolean): void {
    if (reset) this.viewport.scrollLeft = 0;

    // nav가 이미 보이는 상태에서도 "nav를 숨긴 전체 가용 폭"으로 판정해 resize hysteresis를 막는다.
    const navigationWidth = (this.previousButton.hidden ? 0 : this.previousButton.offsetWidth)
      + (this.nextButton.hidden ? 0 : this.nextButton.offsetWidth);
    const availableWithoutNavigation = this.viewport.clientWidth + navigationWidth;
    const overflowing = hasIconToolbarOverflow(this.track.scrollWidth, availableWithoutNavigation);

    this.root.classList.toggle('tb-scroll-overflowing', overflowing);
    this.previousButton.hidden = !overflowing;
    this.nextButton.hidden = !overflowing;
    if (!overflowing) this.viewport.scrollLeft = 0;
    this.updateNavigationState();
  }

  private visibleGroupBoundaries(): number[] {
    return Array.from(this.track.querySelectorAll<HTMLElement>(GROUP_SELECTOR))
      .filter(group => !group.hidden && getComputedStyle(group).display !== 'none' && group.offsetWidth > 0)
      .map(group => group.offsetLeft)
      .sort((left, right) => left - right);
  }

  private scrollToAdjacentGroup(direction: ScrollDirection): void {
    const current = this.viewport.scrollLeft;
    const boundaries = this.visibleGroupBoundaries();
    const target = adjacentIconToolbarGroupTarget(
      boundaries,
      current,
      direction,
      this.maxScrollLeft(),
    );
    this.scrollTo(target);
  }

  private scrollTo(left: number): void {
    const target = Math.max(0, Math.min(left, this.maxScrollLeft()));
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.viewport.scrollTo({ left: target, behavior: reducedMotion ? 'auto' : 'smooth' });
  }

  private maxScrollLeft(): number {
    return Math.max(0, this.viewport.scrollWidth - this.viewport.clientWidth);
  }

  private updateNavigationState(): void {
    const navigationHidden = this.previousButton.hasAttribute('hidden')
      || this.nextButton.hasAttribute('hidden');
    const current = this.viewport.scrollLeft;
    const maximum = this.maxScrollLeft();
    this.previousButton.disabled = navigationHidden || current <= SCROLL_EPSILON;
    this.nextButton.disabled = navigationHidden || current >= maximum - SCROLL_EPSILON;
  }

  dispose(): void {
    if (this.animationFrame !== null) cancelAnimationFrame(this.animationFrame);
    this.resizeObserver.disconnect();
    this.modeObserver.disconnect();
    this.previousButton.removeEventListener('click', this.onPreviousClick);
    this.nextButton.removeEventListener('click', this.onNextClick);
    this.viewport.removeEventListener('scroll', this.onScroll);
    this.viewport.removeEventListener('keydown', this.onViewportKeyDown);
    this.viewport.removeEventListener('focusin', this.onViewportFocusIn);
  }
}

export function initIconToolbarScroller(root: HTMLElement): IconToolbarScroller | null {
  const viewport = root.querySelector<HTMLElement>('#icon-toolbar-viewport');
  const track = root.querySelector<HTMLElement>('.tb-scroll-track');
  const previousButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-prev');
  const nextButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-next');
  if (!viewport || !track || !previousButton || !nextButton) return null;
  return new IconToolbarScroller(root, viewport, track, previousButton, nextButton);
}
