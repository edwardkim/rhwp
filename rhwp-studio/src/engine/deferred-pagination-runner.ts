import type { DeferredPaginationResult } from '@/core/wasm-bridge';

export interface DeferredPaginationClient {
  beginDeferredPagination(fragmentBudget: number): DeferredPaginationResult;
  stepDeferredPagination(fragmentBudget: number): DeferredPaginationResult;
  cancelDeferredPagination(): boolean;
}

type ScheduledTask = ReturnType<typeof setTimeout>;

/**
 * #2424 continuation을 한 macrotask당 한 fragment budget씩 전진시킨다.
 * 입력이 오면 `start()`가 기존 shadow job과 예약 task를 폐기하고 최신 revision으로 교체한다.
 */
export class DeferredPaginationRunner {
  private scheduled: ScheduledTask | null = null;
  private active = false;

  constructor(
    private readonly client: DeferredPaginationClient,
    private readonly onComplete: (result: DeferredPaginationResult) => void,
    private readonly onFallback: (result: DeferredPaginationResult) => void,
    private readonly fragmentBudget = 1,
    private readonly scheduleTask: (callback: () => void) => ScheduledTask = (callback) =>
      setTimeout(callback, 0),
    private readonly cancelTask: (task: ScheduledTask) => void = (task) => clearTimeout(task),
  ) {}

  isActive(): boolean {
    return this.active;
  }

  start(): DeferredPaginationResult {
    this.cancelScheduledTask();
    this.client.cancelDeferredPagination();
    this.active = false;
    const result = this.client.beginDeferredPagination(this.fragmentBudget);
    this.accept(result);
    return result;
  }

  cancel(): void {
    this.cancelScheduledTask();
    if (this.active) {
      this.client.cancelDeferredPagination();
    }
    this.active = false;
  }

  private accept(result: DeferredPaginationResult): void {
    if (result.status === 'pending') {
      this.active = true;
      this.scheduleNextStep();
      return;
    }
    this.active = false;
    if (result.status === 'complete') {
      this.onComplete(result);
      return;
    }
    this.onFallback(result);
  }

  private scheduleNextStep(): void {
    if (!this.active || this.scheduled !== null) return;
    this.scheduled = this.scheduleTask(() => {
      this.scheduled = null;
      if (!this.active) return;
      try {
        this.accept(this.client.stepDeferredPagination(this.fragmentBudget));
      } catch {
        this.active = false;
        this.onFallback({
          ok: false,
          status: 'fallback',
          revision: 0,
          fragmentsProcessed: 0,
          pageCount: 0,
        });
      }
    });
  }

  private cancelScheduledTask(): void {
    if (this.scheduled === null) return;
    this.cancelTask(this.scheduled);
    this.scheduled = null;
  }
}
