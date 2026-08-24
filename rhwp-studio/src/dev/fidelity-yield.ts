const IDLE_TIMEOUT_MS = 500;

type IdleWindow = Window & {
  requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number;
  cancelIdleCallback?: (id: number) => void;
};

export class DiagnosticPauseGate {
  private waiting = new Set<() => void>();
  private value = false;

  get paused(): boolean {
    return this.value;
  }

  set(paused: boolean): void {
    if (this.value === paused) return;
    this.value = paused;
    if (paused) return;
    for (const resume of Array.from(this.waiting)) resume();
  }

  wait(signal: AbortSignal): Promise<void> {
    if (!this.value || signal.aborted) return Promise.resolve();
    return new Promise(resolve => {
      const resume = (): void => {
        signal.removeEventListener('abort', resume);
        this.waiting.delete(resume);
        resolve();
      };
      this.waiting.add(resume);
      signal.addEventListener('abort', resume, { once: true });
      if (!this.value || signal.aborted) resume();
    });
  }
}

export function yieldToInteractiveWork(signal: AbortSignal): Promise<void> {
  if (signal.aborted || (typeof document !== 'undefined' && document.visibilityState !== 'visible')) {
    return Promise.resolve();
  }
  return new Promise(resolve => {
    const idleWindow = window as IdleWindow;
    let idleId: number | null = null;
    let timerId: number | null = null;
    const finish = (): void => {
      signal.removeEventListener('abort', finish);
      if (idleId !== null) idleWindow.cancelIdleCallback?.(idleId);
      if (timerId !== null) window.clearTimeout(timerId);
      resolve();
    };
    signal.addEventListener('abort', finish, { once: true });
    const hasIdleCallback = typeof idleWindow.requestIdleCallback === 'function';
    timerId = window.setTimeout(finish, hasIdleCallback ? IDLE_TIMEOUT_MS : 16);
    if (hasIdleCallback) idleId = idleWindow.requestIdleCallback(finish, { timeout: IDLE_TIMEOUT_MS });
  });
}
