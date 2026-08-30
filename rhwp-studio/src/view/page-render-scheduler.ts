export const MULTI_COLUMN_SYNC_RENDER_BUDGET = 1;

export interface SchedulerHost {
  requestAnimationFrame(callback: (time: number) => void): number;
  cancelAnimationFrame(id: number): void;
}

function defaultHost(): SchedulerHost {
  const raf = globalThis.requestAnimationFrame?.bind(globalThis);
  const caf = globalThis.cancelAnimationFrame?.bind(globalThis);
  if (raf && caf) {
    return { requestAnimationFrame: raf, cancelAnimationFrame: caf };
  }
  return {
    requestAnimationFrame: (callback) => globalThis.setTimeout(() => callback(0), 0) as unknown as number,
    cancelAnimationFrame: (id) => globalThis.clearTimeout(id),
  };
}

/**
 * 단일/두 쪽은 보이는 쪽을 한 프레임에서 모두 그려 첫 페인트가 기준선보다
 * 나빠지지 않게 한다. 한 행 여러 쪽(3열 이상)은 한 slice에 1페이지만 동기 렌더한다.
 */
export function syncVisibleRenderBudget(columns: number, missingVisibleCount: number): number {
  if (missingVisibleCount <= 0) return 0;
  if (columns <= 2) return missingVisibleCount;
  return Math.min(MULTI_COLUMN_SYNC_RENDER_BUDGET, missingVisibleCount);
}

/**
 * 가시 페이지 후속 작업을 rAF slice로 나눈다. 새 프레임(generation)이 오면
 * 이전 slice는 버리고, 취소된 콜백은 render를 호출하지 않는다.
 */
export class PageRenderScheduler {
  private generation = 0;
  private visibleRaf: number | null = null;
  private visibleQueue: number[] = [];
  private readonly host: SchedulerHost;

  constructor(host?: SchedulerHost) {
    this.host = host ?? defaultHost();
  }

  beginFrame(): number {
    this.generation += 1;
    this.cancelVisible();
    return this.generation;
  }

  currentGeneration(): number {
    return this.generation;
  }

  cancelVisible(): void {
    if (this.visibleRaf !== null) {
      this.host.cancelAnimationFrame(this.visibleRaf);
      this.visibleRaf = null;
    }
    this.visibleQueue = [];
  }

  pendingVisibleCount(): number {
    return this.visibleQueue.length;
  }

  scheduleVisible(
    pages: readonly number[],
    generation: number,
    render: (pageIdx: number) => void,
  ): void {
    this.visibleQueue = pages.slice();
    const tick = (time: number) => {
      void time;
      this.visibleRaf = null;
      if (generation !== this.generation) {
        this.visibleQueue = [];
        return;
      }
      const pageIdx = this.visibleQueue.shift();
      if (pageIdx === undefined) return;
      render(pageIdx);
      if (this.visibleQueue.length > 0) {
        this.visibleRaf = this.host.requestAnimationFrame(tick);
      }
    };
    if (this.visibleQueue.length > 0) {
      this.visibleRaf = this.host.requestAnimationFrame(tick);
    }
  }
}
