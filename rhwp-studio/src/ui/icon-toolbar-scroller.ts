const SCROLL_EPSILON = 1;
const SCROLL_ANIMATION_DURATION_MS = 240;
const SCROLL_EXIT_CLASS = 'tb-scroll-nav-transitioning-out';
const DIVIDER_SELECTOR = ':scope > .tb-sep';

type ScrollDirection = -1 | 1;

export function hasIconToolbarOverflow(contentWidth: number, availableWidth: number): boolean {
  return contentWidth - availableWidth > SCROLL_EPSILON;
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
    this.scrollToAdjacentDivider(-1);
  };

  private readonly onNextClick = (): void => {
    this.scrollToAdjacentDivider(1);
  };

  private readonly onViewportKeyDown = (event: KeyboardEvent): void => {
    if (event.target !== this.viewport) return;
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
    const target = event.target;
    if (!(target instanceof Element)) return;
    const command = target.closest('.tb-btn') as HTMLElement | null;
    if (!command || !this.track.contains(command)) return;

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

    // overlay nav의 표시 여부와 무관하게 평상시 양끝 padding을 기준으로 overflow를 판정한다.
    const restingInlinePadding = Number.parseFloat(
      getComputedStyle(this.root).getPropertyValue('--tb-resting-inline-padding'),
    ) || 0;
    const availableWithoutNavigation = Math.max(
      0,
      this.root.clientWidth - (restingInlinePadding * 2),
    );
    const overflowing = hasIconToolbarOverflow(this.track.scrollWidth, availableWithoutNavigation);

    this.root.classList.toggle('tb-scroll-overflowing', overflowing);
    this.previousButton.hidden = !overflowing;
    this.nextButton.hidden = !overflowing;
    if (!overflowing) this.viewport.scrollLeft = 0;
    this.updateNavigationState();
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
    const target = Math.max(0, Math.min(left, this.maxScrollLeft()));
    const start = this.viewport.scrollLeft;
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.cancelScrollAnimation();

    if (reducedMotion || Math.abs(target - start) <= SCROLL_EPSILON) {
      this.viewport.scrollLeft = target;
      this.updateNavigationState();
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
      this.updateNavigationState();

      if (progress < 1) {
        this.scrollAnimationFrame = requestAnimationFrame(animate);
        return;
      }

      this.viewport.scrollLeft = target;
      this.updateNavigationState();
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

  private updateNavigationState(): void {
    const navigationHidden = this.previousButton.hasAttribute('hidden')
      || this.nextButton.hasAttribute('hidden');
    const current = this.viewport.scrollLeft;
    const maximum = this.maxScrollLeft();
    const atStart = navigationHidden || current <= SCROLL_EPSILON;
    const atEnd = navigationHidden || current >= maximum - SCROLL_EPSILON;
    this.previousButton.disabled = atStart;
    this.nextButton.disabled = atEnd;
    this.previousButton.classList.toggle('tb-scroll-nav-edge-hidden', atStart);
    this.nextButton.classList.toggle('tb-scroll-nav-edge-hidden', atEnd);
    this.previousButton.setAttribute('aria-hidden', atStart ? 'true' : 'false');
    this.nextButton.setAttribute('aria-hidden', atEnd ? 'true' : 'false');
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

export function initIconToolbarScroller(root: HTMLElement): IconToolbarScroller | null {
  const viewport = root.querySelector<HTMLElement>('#icon-toolbar-viewport');
  const track = root.querySelector<HTMLElement>('.tb-scroll-track');
  const previousButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-prev');
  const nextButton = root.querySelector<HTMLButtonElement>('#icon-toolbar-next');
  if (!viewport || !track || !previousButton || !nextButton) return null;
  return new IconToolbarScroller(root, viewport, track, previousButton, nextButton);
}
