/** DEV 수동 기록 구간. 물리 gesture 종료를 추정하거나 제품의 zoom/렌더를 제어하지 않는다. */
export interface ZoomSessionState {
  zoom: number;
  animating: boolean;
  columns: number;
  /** 이전 진단 JSON과 호환하며 새 observer는 항상 세 필드를 기록한다. */
  inputActive?: boolean;
  rasterPending?: boolean;
  zoomGeneration?: number;
  /** A/B opt-in rAF의 cached 상태. 실제 compositor paint/최종 화질의 보증이 아니다. */
  scrollY?: number;
  visiblePages?: number[];
  missingBitmapPages?: number[];
  currentRasterPages?: number[];
  visibleQueued?: number;
  prefetchQueued?: number;
  pendingImages?: number;
}

export interface ZoomSessionInput {
  eventTimestamp: number;
  deltaX: number;
  deltaY: number;
  deltaMode: number;
  ctrlKey: boolean;
  metaKey: boolean;
  trusted: boolean;
}

interface ZoomSession {
  scope: string;
  startedAt: number;
  endedAt: number | null;
  status: 'recording' | 'stopped' | 'interrupted' | 'timeout';
  reason: string | null;
  knownWorkReadyAtStop: boolean | null;
  inputs: (ZoomSessionInput & { at: number } & ZoomSessionState)[];
  spans: ({ boundary: string; page: number | null; at: number; ms: number } & ZoomSessionState)[];
  frames: ({ at: number; gap: number | null } & ZoomSessionState)[];
  longTasks: { at: number; ms: number }[];
  counters: Record<string, { calls: number; inclusiveMs: number; maxMs: number }>;
  dropped: { inputs: number; spans: number; frames: number; longTasks: number };
}

export class ZoomSessionObservation {
  private session: ZoomSession | null = null;
  private lastFrame: number | null = null;
  private readonly limit: number;
  private readonly maxMs: number;

  constructor(limit = 8192, maxMs = 20_000) {
    if (!Number.isInteger(limit) || limit < 1 || !Number.isFinite(maxMs) || maxMs <= 0) {
      throw new Error('핀치 기록 상한은 양수여야 합니다');
    }
    this.limit = limit;
    this.maxMs = maxMs;
  }

  get recording(): boolean { return this.session?.status === 'recording'; }

  start(scope: string, at: number): void {
    this.session = {
      scope, startedAt: at, endedAt: null, status: 'recording', reason: null,
      knownWorkReadyAtStop: null, inputs: [], spans: [], frames: [], longTasks: [], counters: {},
      dropped: { inputs: 0, spans: 0, frames: 0, longTasks: 0 },
    };
    this.lastFrame = null;
  }

  input(scope: string, at: number, input: ZoomSessionInput, state: ZoomSessionState): void {
    const s = this.accept(scope, at);
    if (!s) return;
    if (s.inputs.length < this.limit) s.inputs.push({ ...input, ...state, at: at - s.startedAt });
    else s.dropped.inputs++;
  }

  span(scope: string, boundary: string, start: number, end: number, page: number | null, state: ZoomSessionState): void {
    const s = this.accept(scope, start);
    if (!s) return;
    const ms = Math.max(0, end - start);
    const counter = s.counters[boundary] ??= { calls: 0, inclusiveMs: 0, maxMs: 0 };
    counter.calls++;
    counter.inclusiveMs += ms;
    counter.maxMs = Math.max(counter.maxMs, ms);
    if (s.spans.length < this.limit) s.spans.push({ boundary, page, at: start - s.startedAt, ms, ...state });
    else s.dropped.spans++;
  }

  frame(scope: string, at: number, state: ZoomSessionState): void {
    const s = this.accept(scope, at);
    if (!s) return;
    if (s.frames.length < Math.min(this.limit, 2048)) {
      s.frames.push({ at: at - s.startedAt, gap: this.lastFrame === null ? null : at - this.lastFrame, ...state });
    } else s.dropped.frames++;
    this.lastFrame = at;
  }

  longTask(scope: string, start: number, ms: number): void {
    const s = this.accept(scope, start);
    if (!s) return;
    if (s.longTasks.length < Math.min(this.limit, 256)) s.longTasks.push({ at: start - s.startedAt, ms });
    else s.dropped.longTasks++;
  }

  stop(at: number, knownWorkReady: boolean): void {
    if (!this.recording) return;
    this.finish('stopped', at, 'manual stop; not gesture end or presentation completion');
    if (this.session?.status === 'stopped') this.session.knownWorkReadyAtStop = knownWorkReady;
  }

  interrupt(at: number, reason: string): void { this.finish('interrupted', at, reason); }
  snapshot(): ZoomSession | null { return structuredClone(this.session); }
  clear(): void { this.session = null; this.lastFrame = null; }

  private accept(scope: string, at: number): ZoomSession | null {
    const s = this.session;
    if (!s || !this.recording) return null;
    if (!Number.isFinite(at) || at < s.startedAt) return null;
    if (s.scope !== scope) { this.interrupt(at, 'document/renderer/content changed'); return null; }
    if (at - s.startedAt >= this.maxMs) { this.finish('timeout', at, 'recording time limit'); return null; }
    return s;
  }

  private finish(status: 'stopped' | 'interrupted' | 'timeout', at: number, reason: string): void {
    if (!this.session || !this.recording) return;
    this.session.status = status;
    this.session.endedAt = at;
    this.session.reason = reason;
  }
}
