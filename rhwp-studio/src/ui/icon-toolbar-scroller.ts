import { closeToolbarSplitMenus } from './toolbar-split-menu.ts';

const SCROLL_EPSILON = 1;
const SCROLL_ANIMATION_DURATION_MS = 240;
const SCROLL_EXIT_CLASS = 'tb-scroll-nav-transitioning-out';
const DIVIDER_SELECTOR = ':scope > .tb-sep';

type ScrollDirection = -1 | 1;

export function hasIconToolbarOverflow(contentWidth: number, availableWidth: number): boolean {
  return contentWidth - availableWidth > SCROLL_EPSILON;
}

export function clampIconToolbarScroll(
  current: number,
  maximum: number,
  overflowing: boolean,
): number {
  return overflowing ? Math.max(0, Math.min(current, maximum)) : 0;
}

export function iconToolbarNavigationState(
  current: number,
  maximum: number,
  navigationHidden: boolean,
): { atStart: boolean; atEnd: boolean } {
  return {
    atStart: navigationHidden || current <= SCROLL_EPSILON,
    atEnd: navigationHidden || current >= maximum - SCROLL_EPSILON,
  };
}

export function adjacentIconToolbarDividerTarget(
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
  private scrollAnimationFrame: number | null = null;
  private exitingButton: HTMLButtonElement | null = null;
  private overflowing = false;

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
    if (modeChanged) this.scheduleRefresh();
  };

  private readonly onScroll = (): void => {
    if (!this.overflowing) {
      if (Math.abs(this.viewport.scrollLeft) > SCROLL_EPSILON) {
        this.viewport.scrollLeft = 0;
      }
      return;
    }
    closeToolbarSplitMenus(this.track);
    this.updateNavigationState();
  };

  private readonly onPreviousClick = (): void => {
    if (this.previousButton.getAttribute('aria-disabled') === 'true') return;
    this.scrollToAdjacentDivider(-1);
  };

  private readonly onNextClick = (): void => {
    if (this.nextButton.getAttribute('aria-disabled') === 'true') return;
    this.scrollToAdjacentDivider(1);
  };

  private readonly onViewportKeyDown = (event: KeyboardEvent): void => {
    if (event.target !== this.viewport) return;
    if (!this.overflowing) return;
    if (event.key === 'ArrowLeft') {
      event.preventDefault();
      this.scrollToAdjacentDivider(-1);
    } else if (event.key === 'ArrowRight') {
      event.preventDefault();
      this.scrollToAdjacentDivider(1);
    } else if (event.key === 'Home') {
      event.preventDefault();
      this.scrollTo(0);
    } else if (event.key === 'End') {
      event.preventDefault();
      this.scrollTo(this.maxScrollLeft());
    }
  };

  private readonly onViewportFocusIn = (event: FocusEvent): void => {
    if (!this.overflowing) return;
    const target = event.target;
    if (!(target instanceof Element)) return;
    const command = target.closest('.tb-btn') as HTMLElement | null;
    if (!command || !this.track.contains(command)) return;

    this.cancelScrollAnimation();

    const viewportRect = this.viewport.getBoundingClientRect();
    const commandRect = command.getBoundingClientRect();
    const dividerGap = this.navigationEdgeGap();
    const visibleLeft = this.previousButton.classList.contains('tb-scroll-nav-edge-hidden')
      ? viewportRect.left
      : this.previousButton.getBoundingClientRect().right + dividerGap;
    const visibleRight = this.nextButton.classList.contains('tb-scroll-nav-edge-hidden')
      ? viewportRect.right
      : this.nextButton.getBoundingClientRect().left - dividerGap;
    if (commandRect.left < visibleLeft) {
      this.viewport.scrollLeft -= visibleLeft - commandRect.left;
    } else if (commandRect.right > visibleRight) {
      this.viewport.scrollLeft += commandRect.right - visibleRight;
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

    this.refresh();
  }

  private scheduleRefresh(): void {
    if (this.animationFrame !== null) return;
    this.animationFrame = requestAnimationFrame(() => {
      this.animationFrame = null;
      this.refresh();
    });
  }

  private refresh(): void {
    // 실제 native scroll viewport보다 콘텐츠가 넓을 때만 입력과 이동 버튼을 활성화한다.
    const overflowing = hasIconToolbarOverflow(
      this.track.scrollWidth,
      this.viewport.clientWidth,
    );
    this.overflowing = overflowing;
    this.viewport.classList.toggle('tb-scroll-viewport-enabled', overflowing);

    this.previousButton.hidden = !overflowing;
    this.nextButton.hidden = !overflowing;
    const maximum = this.maxScrollLeft();
    this.viewport.scrollLeft = clampIconToolbarScroll(
      this.viewport.scrollLeft,
      maximum,
      overflowing,
    );
    this.updateNavigationState(maximum);
  }

  private visibleDividerBoundaries(direction: ScrollDirection): number[] {
    const trackLeft = this.track.getBoundingClientRect().left;
    const viewportLeft = this.viewport.getBoundingClientRect().left;
    const navigationRect = direction > 0
      ? this.nextButton.getBoundingClientRect()
      : this.previousButton.getBoundingClientRect();
    const dividerGap = this.navigationEdgeGap();
    const targetBoundary = direction > 0
      ? navigationRect.left - dividerGap - viewportLeft
      : navigationRect.right + dividerGap - viewportLeft;
    return Array.from(this.track.querySelectorAll<HTMLElement>(DIVIDER_SELECTOR))
      .filter(divider => !divider.hidden && getComputedStyle(divider).display !== 'none' && divider.offsetWidth > 0)
      .map((divider) => {
        const rect = divider.getBoundingClientRect();
        // divider와 nav 사이에 일반 도구 그룹 간격을 남겨 버튼에 붙어 보이지 않게 한다.
        return direction > 0
          ? rect.right - trackLeft - targetBoundary
          : rect.left - trackLeft - targetBoundary;
      })
      .sort((left, right) => left - right);
  }

  private navigationEdgeGap(): number {
    return Number.parseFloat(
      getComputedStyle(this.root).getPropertyValue('--tb-scroll-nav-edge-gap'),
    ) || 0;
  }

  private scrollToAdjacentDivider(direction: ScrollDirection): void {
    if (!this.overflowing) return;
    const current = this.viewport.scrollLeft;
    const boundaries = this.visibleDividerBoundaries(direction);
    const target = adjacentIconToolbarDividerTarget(
      boundaries,
      current,
      direction,
      this.maxScrollLeft(),
    );
    this.scrollTo(target);
  }

  private scrollTo(left: number): void {
    if (!this.overflowing) {
      this.viewport.scrollLeft = 0;
      this.updateNavigationState(0);
      return;
    }
    const target = Math.max(0, Math.min(left, this.maxScrollLeft()));
    const start = this.viewport.scrollLeft;
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.cancelScrollAnimation();

    if (reducedMotion || Math.abs(target - start) <= SCROLL_EPSILON) {
      this.viewport.scrollLeft = target;
      this.updateNavigationState(this.maxScrollLeft());
      return;
    }

    const maximum = this.maxScrollLeft();
    const destinationButton = target <= SCROLL_EPSILON
      ? this.previousButton
      : target >= maximum - SCROLL_EPSILON
        ? this.nextButton
        : null;
    if (destinationButton) {
      this.exitingButton = destinationButton;
      destinationButton.style.setProperty(
        '--tb-scroll-exit-duration',
        `${SCROLL_ANIMATION_DURATION_MS}ms`,
      );
      destinationButton.classList.add(SCROLL_EXIT_CLASS);
    }

    let startedAt: number | null = null;
    const distance = target - start;
    const animate = (timestamp: number): void => {
      startedAt ??= timestamp;
      const progress = Math.min(1, (timestamp - startedAt) / SCROLL_ANIMATION_DURATION_MS);
      const eased = 1 - ((1 - progress) ** 3);
      this.viewport.scrollLeft = start + (distance * eased);
      this.updateNavigationState(maximum);

      if (progress < 1) {
        this.scrollAnimationFrame = requestAnimationFrame(animate);
        return;
      }

      this.viewport.scrollLeft = target;
      this.updateNavigationState(maximum);
      this.scrollAnimationFrame = null;
      this.finishScrollExit();
    };
    this.scrollAnimationFrame = requestAnimationFrame(animate);
  }

  private finishScrollExit(): void {
    if (!this.exitingButton) return;
    this.exitingButton.classList.remove(SCROLL_EXIT_CLASS);
    this.exitingButton.style.removeProperty('--tb-scroll-exit-duration');
    this.exitingButton = null;
  }

  private cancelScrollAnimation(): void {
    if (this.scrollAnimationFrame !== null) {
      cancelAnimationFrame(this.scrollAnimationFrame);
      this.scrollAnimationFrame = null;
    }
    this.finishScrollExit();
  }

  private maxScrollLeft(): number {
    return Math.max(0, this.viewport.scrollWidth - this.viewport.clientWidth);
  }

  private syncNavigationButton(button: HTMLButtonElement, atEdge: boolean): void {
    if (atEdge && document.activeElement === button) {
      this.viewport.focus({ preventScroll: true });
    }
    button.tabIndex = atEdge ? -1 : 0;
    button.classList.toggle('tb-scroll-nav-edge-hidden', atEdge);
    const nextValue = atEdge ? 'true' : 'false';
    if (button.getAttribute('aria-disabled') !== nextValue) {
      button.setAttribute('aria-disabled', nextValue);
    }
    if (button.getAttribute('aria-hidden') !== nextValue) {
      button.setAttribute('aria-hidden', nextValue);
    }
  }

  private updateNavigationState(maximum = this.maxScrollLeft()): void {
    const navigationHidden = this.previousButton.hasAttribute('hidden')
      || this.nextButton.hasAttribute('hidden');
    const current = this.viewport.scrollLeft;
    const { atStart, atEnd } = iconToolbarNavigationState(current, maximum, navigationHidden);
    this.syncNavigationButton(this.previousButton, atStart);
    this.syncNavigationButton(this.nextButton, atEnd);
  }

  resetToStart(): void {
    this.cancelScrollAnimation();
    this.viewport.scrollLeft = 0;
    this.scheduleRefresh();
  }

  dispose(): void {
    if (this.animationFrame !== null) cancelAnimationFrame(this.animationFrame);
    this.cancelScrollAnimation();
    this.resizeObserver.disconnect();
    this.modeObserver.disconnect();
    this.previousButton.removeEventListener('click', this.onPreviousClick);
    this.nextButton.removeEventListener('click', this.onNextClick);
    this.viewport.removeEventListener('scroll', this.onScroll);
    this.viewport.removeEventListener('keydown', this.onViewportKeyDown);
    this.viewport.removeEventListener('focusin', this.onViewportFocusIn);
  }
}

export function initIconToolbarScroller(root: HTMLElement | null): IconToolbarScroller | null {
  if (!root) return null;
  const viewport = root.querySelector<HTMLElement>('#icon-toolbar-viewport');
  const track = root.querySelector<HTMLElement>('.tb-scroll-track');
  const previousButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-prev');
  const nextButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-next');
  if (!viewport || !track || !previousButton || !nextButton) return null;
  return new IconToolbarScroller(root, viewport, track, previousButton, nextButton);
}
