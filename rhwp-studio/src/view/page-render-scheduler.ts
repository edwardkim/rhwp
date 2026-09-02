export type PageRenderWorkClass = 'visible' | 'retained-transition' | 'prefetch';

export interface PageRenderWork {
  pageIndex: number;
  /** 낮을수록 먼저 실행한다. 같은 priority는 먼저 들어온 page가 우선한다. */
  priority: number;
  rasterKey: string;
  workClass: PageRenderWorkClass;
  isValid(): boolean;
  run(): void;
}

export interface PageRenderIdleDeadline {
  didTimeout: boolean;
  timeRemaining(): number;
}

export interface PageRenderSchedulerHost {
  now(): number;
  requestFrame(callback: () => void): number;
  cancelFrame(id: number): void;
  requestIdle?: (
    callback: (deadline: PageRenderIdleDeadline) => void,
    options: { timeout: number },
  ) => number;
  cancelIdle?: (id: number) => void;
  setTimeout(callback: () => void, delayMs: number): number;
  clearTimeout(id: number): void;
}

export interface PageRenderSchedulerOptions {
  /** page 내부 raster는 선점할 수 없으므로 page 경계에서만 적용되는 soft budget이다. */
  visibleSliceBudgetMs?: number;
  maxVisiblePagesPerSlice?: number;
  idleTimeoutMs?: number;
  timeoutFallbackDelayMs?: number;
  scrollSettleDelayMs?: number;
}

export interface PageRenderSchedulerSnapshot {
  generation: number;
  visibleQueued: number;
  prefetchQueued: number;
  frameScheduled: boolean;
  idleScheduled: boolean;
  scrollSettleScheduled: boolean;
  visibleSlices: number;
  visibleExecuted: number;
  prefetchExecuted: number;
  prefetchAdmissionRejected: number;
  staleDropped: number;
  maxQueueDepth: number;
}

interface QueuedWork {
  work: PageRenderWork;
  generation: number;
  sequence: number;
}

type DeferredTask =
  | { kind: 'idle'; id: number }
  | { kind: 'timeout'; id: number };

const DEFAULT_VISIBLE_SLICE_BUDGET_MS = 4;
const DEFAULT_MAX_VISIBLE_PAGES_PER_SLICE = 2;
const DEFAULT_IDLE_TIMEOUT_MS = 1000;
const DEFAULT_TIMEOUT_FALLBACK_DELAY_MS = 250;
const DEFAULT_SCROLL_SETTLE_DELAY_MS = 150;

function positiveFinite(value: number | undefined, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) && value > 0
    ? value
    : fallback;
}

function positiveInteger(value: number | undefined, fallback: number): number {
  return Math.max(1, Math.floor(positiveFinite(value, fallback)));
}

/**
 * page 단위 visible/prefetch 작업 스케줄러.
 *
 * 같은 page는 Map 한 칸만 소유한다. 새 scroll generation은 큐 내용을 최신 작업으로 교체하되 이미 예약한
 * frame 자체는 유지해 연속 입력이 매번 rAF를 취소·재예약하며 진행을 굶기지 않게 한다. visible가 하나라도
 * 남아 있으면 idle prefetch는 실행하지 않는다.
 */
export class PageRenderScheduler {
  private readonly host: PageRenderSchedulerHost;
  private readonly visibleSliceBudgetMs: number;
  private readonly maxVisiblePagesPerSlice: number;
  private readonly idleTimeoutMs: number;
  private readonly timeoutFallbackDelayMs: number;
  private readonly scrollSettleDelayMs: number;
  private generation = 0;
  private sequence = 0;
  private visible = new Map<number, QueuedWork>();
  private prefetch = new Map<number, QueuedWork>();
  private frameId: number | null = null;
  private deferredTask: DeferredTask | null = null;
  private scrollSettleTimerId: number | null = null;
  private visibleSlices = 0;
  private visibleExecuted = 0;
  private prefetchExecuted = 0;
  private prefetchAdmissionRejected = 0;
  private staleDropped = 0;
  private maxQueueDepth = 0;

  constructor(
    host: PageRenderSchedulerHost,
    options: PageRenderSchedulerOptions = {},
  ) {
    this.host = host;
    this.visibleSliceBudgetMs = positiveFinite(
      options.visibleSliceBudgetMs,
      DEFAULT_VISIBLE_SLICE_BUDGET_MS,
    );
    this.maxVisiblePagesPerSlice = positiveInteger(
      options.maxVisiblePagesPerSlice,
      DEFAULT_MAX_VISIBLE_PAGES_PER_SLICE,
    );
    this.idleTimeoutMs = positiveFinite(options.idleTimeoutMs, DEFAULT_IDLE_TIMEOUT_MS);
    this.timeoutFallbackDelayMs = positiveFinite(
      options.timeoutFallbackDelayMs,
      DEFAULT_TIMEOUT_FALLBACK_DELAY_MS,
    );
    this.scrollSettleDelayMs = positiveFinite(
      options.scrollSettleDelayMs,
      DEFAULT_SCROLL_SETTLE_DELAY_MS,
    );
  }

  /** 최신 desired work로 교체한다. 작은 visible 집합만 기존 동기 fast path로 실행한다. */
  setDesiredWork(
    generation: number,
    visible: readonly PageRenderWork[],
    prefetch: readonly PageRenderWork[],
    allowVisibleFastPath: boolean,
  ): void {
    this.generation = generation;
    this.visible = this.replaceQueue(this.visible, visible, generation);
    this.prefetch = this.replaceQueue(this.prefetch, prefetch, generation);
    for (const pageIndex of this.visible.keys()) this.prefetch.delete(pageIndex);
    this.maxQueueDepth = Math.max(
      this.maxQueueDepth,
      this.visible.size + this.prefetch.size,
    );

    if (this.visible.size > 0) {
      this.cancelDeferredTask();
      if (allowVisibleFastPath && this.visible.size <= this.maxVisiblePagesPerSlice) {
        this.runVisibleSlice();
        // 이전 generation이 예약해 둔 frame이 있더라도 fast path가 최신 visible을 모두
        // 완료했다면 빈 callback을 남기지 않는다. 실제 작업이 남은 경우에는 아래 slice가
        // 같은 frame을 그대로 재사용한다.
        this.cancelFrameIfEmpty();
      } else {
        this.ensureFrame();
      }
      return;
    }
    this.cancelFrameIfEmpty();
    this.ensureDeferredTask();
  }

  cancelAll(): void {
    this.visible.clear();
    this.prefetch.clear();
    if (this.frameId !== null) {
      this.host.cancelFrame(this.frameId);
      this.frameId = null;
    }
    this.cancelDeferredTask();
    this.cancelScrollSettle();
  }

  /** 마지막 scroll 입력 하나만 정착 경계로 전달한다. */
  scheduleScrollSettle(callback: () => void): void {
    this.cancelScrollSettle();
    this.scrollSettleTimerId = this.host.setTimeout(() => {
      this.scrollSettleTimerId = null;
      callback();
    }, this.scrollSettleDelayMs);
  }

  cancelScrollSettle(): void {
    if (this.scrollSettleTimerId === null) return;
    this.host.clearTimeout(this.scrollSettleTimerId);
    this.scrollSettleTimerId = null;
  }

  /** retained 예산상 보존 이득이 없어 dispatch 전에 거절한 선택 prefetch를 기록한다. */
  recordPrefetchAdmissionRejected(count = 1): void {
    if (!Number.isFinite(count) || count <= 0) return;
    this.prefetchAdmissionRejected += Math.floor(count);
  }

  snapshot(): PageRenderSchedulerSnapshot {
    return {
      generation: this.generation,
      visibleQueued: this.visible.size,
      prefetchQueued: this.prefetch.size,
      frameScheduled: this.frameId !== null,
      idleScheduled: this.deferredTask !== null,
      scrollSettleScheduled: this.scrollSettleTimerId !== null,
      visibleSlices: this.visibleSlices,
      visibleExecuted: this.visibleExecuted,
      prefetchExecuted: this.prefetchExecuted,
      prefetchAdmissionRejected: this.prefetchAdmissionRejected,
      staleDropped: this.staleDropped,
      maxQueueDepth: this.maxQueueDepth,
    };
  }

  private replaceQueue(
    previous: Map<number, QueuedWork>,
    desired: readonly PageRenderWork[],
    generation: number,
  ): Map<number, QueuedWork> {
    const next = new Map<number, QueuedWork>();
    for (const work of desired) {
      if (!Number.isInteger(work.pageIndex) || work.pageIndex < 0) continue;
      const prior = previous.get(work.pageIndex);
      const sequence = prior?.work.rasterKey === work.rasterKey
        ? prior.sequence
        : ++this.sequence;
      const existing = next.get(work.pageIndex);
      if (existing && existing.work.priority <= work.priority) continue;
      next.set(work.pageIndex, { work, generation, sequence });
    }
    return next;
  }

  private ensureFrame(): void {
    if (this.frameId !== null || this.visible.size === 0) return;
    this.frameId = this.host.requestFrame(() => {
      this.frameId = null;
      this.runVisibleSlice();
    });
  }

  private runVisibleSlice(): void {
    if (this.visible.size === 0) {
      this.ensureDeferredTask();
      return;
    }
    this.visibleSlices += 1;
    const startedAt = this.host.now();
    let executed = 0;
    while (this.visible.size > 0 && executed < this.maxVisiblePagesPerSlice) {
      const queued = this.takeNext(this.visible);
      if (!queued) break;
      if (queued.generation !== this.generation || !queued.work.isValid()) {
        this.staleDropped += 1;
        continue;
      }
      queued.work.run();
      executed += 1;
      this.visibleExecuted += 1;
      if (this.host.now() - startedAt >= this.visibleSliceBudgetMs) break;
    }
    if (this.visible.size > 0) this.ensureFrame();
    else this.ensureDeferredTask();
  }

  private ensureDeferredTask(): void {
    if (
      this.deferredTask !== null
      || this.visible.size > 0
      || this.prefetch.size === 0
    ) return;

    // 이미 붙어 있는 surface의 target DPR 전환은 speculative allocation이 아니다. 한 task에
    // 한 쪽만 처리해 입력 기회를 남기되, 다음 idle frame을 기다리는 불필요한 공백은 두지 않는다.
    if (this.peekNext(this.prefetch)?.work.workClass === 'retained-transition') {
      this.deferredTask = {
        kind: 'timeout',
        id: this.host.setTimeout(() => {
          this.deferredTask = null;
          this.runOnePrefetch({ didTimeout: true, timeRemaining: () => 0 });
        }, 0),
      };
      return;
    }

    if (this.host.requestIdle) {
      this.deferredTask = {
        kind: 'idle',
        id: this.host.requestIdle((deadline) => {
          this.deferredTask = null;
          this.runOnePrefetch(deadline);
        }, { timeout: this.idleTimeoutMs }),
      };
      return;
    }
    this.deferredTask = {
      kind: 'timeout',
      id: this.host.setTimeout(() => {
        this.deferredTask = null;
        this.runOnePrefetch({ didTimeout: true, timeRemaining: () => 0 });
      }, this.timeoutFallbackDelayMs),
    };
  }

  private runOnePrefetch(deadline: PageRenderIdleDeadline): void {
    if (this.visible.size > 0) {
      this.ensureFrame();
      return;
    }
    if (!deadline.didTimeout && deadline.timeRemaining() <= 0) {
      this.ensureDeferredTask();
      return;
    }
    const queued = this.takeNext(this.prefetch);
    if (queued) {
      if (queued.generation !== this.generation || !queued.work.isValid()) {
        this.staleDropped += 1;
      } else {
        queued.work.run();
        this.prefetchExecuted += 1;
      }
    }
    this.ensureDeferredTask();
  }

  private takeNext(queue: Map<number, QueuedWork>): QueuedWork | null {
    const selected = this.peekNext(queue);
    if (selected) queue.delete(selected.work.pageIndex);
    return selected;
  }

  private peekNext(queue: Map<number, QueuedWork>): QueuedWork | null {
    let selected: QueuedWork | null = null;
    for (const queued of queue.values()) {
      if (
        selected === null
        || queued.work.priority < selected.work.priority
        || (
          queued.work.priority === selected.work.priority
          && queued.sequence < selected.sequence
        )
      ) selected = queued;
    }
    return selected;
  }

  private cancelFrameIfEmpty(): void {
    if (this.visible.size > 0 || this.frameId === null) return;
    this.host.cancelFrame(this.frameId);
    this.frameId = null;
  }

  private cancelDeferredTask(): void {
    const task = this.deferredTask;
    this.deferredTask = null;
    if (!task) return;
    if (task.kind === 'idle') this.host.cancelIdle?.(task.id);
    else this.host.clearTimeout(task.id);
  }
}

type IdleCapableWindow = Window & {
  requestIdleCallback?: (
    callback: (deadline: PageRenderIdleDeadline) => void,
    options?: { timeout: number },
  ) => number;
  cancelIdleCallback?: (id: number) => void;
};

export function createBrowserPageRenderSchedulerHost(
  target: Window,
): PageRenderSchedulerHost {
  const idleTarget = target as IdleCapableWindow;
  return {
    now: () => target.performance.now(),
    requestFrame: callback => target.requestAnimationFrame(callback),
    cancelFrame: id => target.cancelAnimationFrame(id),
    requestIdle: idleTarget.requestIdleCallback
      ? (callback, options) => idleTarget.requestIdleCallback!(callback, options)
      : undefined,
    cancelIdle: idleTarget.cancelIdleCallback
      ? id => idleTarget.cancelIdleCallback!(id)
      : undefined,
    setTimeout: (callback, delayMs) => target.setTimeout(callback, delayMs),
    clearTimeout: id => target.clearTimeout(id),
  };
}
