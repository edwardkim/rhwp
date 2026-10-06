/** 입력 quiet는 손가락 종료가 아닌 heuristic이다. 배율의 animation 수렴과 분리한다. */
export const ZOOM_INPUT_QUIET_MS = 120;

export interface ZoomInputSettleHost {
  now(): number;
  schedule(callback: () => void, delayMs: number): ReturnType<typeof setTimeout>;
  cancel(handle: ReturnType<typeof setTimeout>): void;
}

const browserHost: ZoomInputSettleHost = {
  now: () => performance.now(),
  schedule: (callback, delay) => setTimeout(callback, delay),
  cancel: handle => clearTimeout(handle),
};

/** 입력당 O(1), timer 최대 하나. 수렴 뒤 quiet를 기다리는 동안 rAF를 돌리지 않는다. */
export class ZoomInputSettle {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private epoch = 0;
  private waiting = false;
  private converged = false;
  private lastInputAt = 0;
  private readonly onReady: (generation: number) => void;
  private readonly host: ZoomInputSettleHost;
  private readonly quietMs: number;

  constructor(
    onReady: (generation: number) => void,
    host: ZoomInputSettleHost = browserHost,
    quietMs = ZOOM_INPUT_QUIET_MS,
  ) {
    if (!Number.isFinite(quietMs) || quietMs <= 0) throw new Error('zoom quiet 시간은 양수여야 합니다');
    this.onReady = onReady;
    this.host = host;
    this.quietMs = quietMs;
  }

  get generation(): number { return this.epoch; }
  get pending(): boolean { return this.waiting; }
  get inputActive(): boolean {
    return this.waiting && this.host.now() - this.lastInputAt < this.quietMs;
  }

  input(): number {
    this.clearTimer();
    const generation = ++this.epoch;
    this.waiting = true;
    this.converged = false;
    this.lastInputAt = this.host.now();
    this.schedule(generation, this.quietMs);
    return generation;
  }

  converge(generation: number): void {
    if (!this.waiting || generation !== this.epoch) return;
    this.converged = true;
    if (!this.inputActive) this.complete();
  }

  /** 명시적 다른 조작은 현재 배율로 완료한다. 호출자가 animation을 먼저 중단해야 한다. */
  flush(): void { if (this.waiting) this.complete(); }

  /** 문서 교체·dispose 등은 완료 알림 없이 이전 세대를 무효화한다. */
  cancel(): boolean {
    if (!this.waiting) return false;
    this.clearTimer();
    this.waiting = false;
    this.converged = false;
    this.epoch++;
    return true;
  }

  private schedule(generation: number, delay: number): void {
    this.timer = this.host.schedule(() => {
      if (!this.waiting || generation !== this.epoch) return;
      this.clearTimer();
      const remaining = this.quietMs - (this.host.now() - this.lastInputAt);
      if (remaining > 0) this.schedule(generation, remaining);
      else if (this.converged) this.complete();
    }, delay);
  }

  private complete(): void {
    const generation = this.epoch;
    this.clearTimer();
    this.waiting = false;
    this.converged = false;
    // 외부 callback이 새 입력을 시작해도 새 상태를 덮어쓰지 않는다.
    this.onReady(generation);
  }

  private clearTimer(): void {
    if (this.timer !== null) this.host.cancel(this.timer);
    this.timer = null;
  }
}
