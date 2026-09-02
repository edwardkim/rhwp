/** DEV 전용 관찰 계약. DOM·렌더·scheduler를 소유하지 않는다. */
export type TraceStatus = 'running' | 'complete' | 'superseded' | 'interrupted' | 'timeout';
export type Milestone = 'preview' | 'visibleFirst' | 'focusedSharp' | 'visibleStable' | 'retainedComplete';
export interface ScrollTrace {
  id: number;
  scope: string;
  source: string;
  startedAt: number;
  status: TraceStatus;
  reason: string | null;
  milestones: Partial<Record<Milestone, number>>;
  counters: Record<string, { calls: number; inclusiveMs: number; maxMs: number; units: number }>;
  spans: { boundary: string; page: number | null; at: number; ms: number }[];
  spansDropped: number;
  frames: number[];
  framesDropped: number;
}

/** 개수 제한은 화면/문서 크기와 무관하다. export 시에만 복사/직렬화한다. */
export class ScrollObservation {
  private serial = 0;
  private samples: ScrollTrace[] = [];
  private current: ScrollTrace | null = null;
  private lastFrame: number | null = null;

  private readonly limit: number;
  private readonly detailLimit: number;
  constructor(limit = 128, detailLimit = 512) {
    if (!Number.isInteger(limit) || limit < 1 || !Number.isInteger(detailLimit) || detailLimit < 1) {
      throw new Error('관찰 buffer 크기는 양의 정수여야 합니다');
    }
    this.limit = limit;
    this.detailLimit = detailLimit;
  }

  get id(): number | null { return this.current?.status === 'running' ? this.current.id : null; }

  begin(scope: string, source: string, at: number): number {
    this.finish('superseded', 'new-interaction');
    const sample: ScrollTrace = {
      id: ++this.serial, scope, source, startedAt: at, status: 'running', reason: null,
      milestones: {}, counters: {}, spans: [], spansDropped: 0, frames: [], framesDropped: 0,
    };
    if (this.samples.length === this.limit) this.samples.shift();
    this.samples.push(sample);
    this.current = sample;
    this.lastFrame = null;
    return sample.id;
  }

  count(id: number | null, boundary: string, start: number, end: number, units = 0, page: number | null = null): void {
    const s = this.accept(id);
    if (!s) return;
    const ms = Math.max(0, end - start);
    const c = s.counters[boundary] ??= { calls: 0, inclusiveMs: 0, maxMs: 0, units: 0 };
    c.calls++;
    c.inclusiveMs += ms;
    c.maxMs = Math.max(c.maxMs, ms);
    c.units += units;
    if (s.spans.length < this.detailLimit) s.spans.push({ boundary, page, at: start - s.startedAt, ms });
    else s.spansDropped++;
  }

  mark(id: number | null, milestone: Milestone, at: number): void {
    const s = this.accept(id);
    if (!s || !Number.isFinite(at) || at < s.startedAt || s.milestones[milestone] !== undefined) return;
    s.milestones[milestone] = at - s.startedAt;
  }

  frame(id: number | null, at: number): void {
    const s = this.accept(id);
    if (!s) return;
    if (this.lastFrame !== null) {
      if (s.frames.length < this.detailLimit) s.frames.push(at - this.lastFrame);
      else s.framesDropped++;
    }
    this.lastFrame = at;
  }

  finish(status: Exclude<TraceStatus, 'running'>, reason: string | null = null): void {
    if (!this.current || this.current.status !== 'running') return;
    // 빈 큐/renderer 반환만으로 완성을 선언하는 실수를 차단한다.
    if (status === 'complete' && (this.current.milestones.visibleStable === undefined
      || this.current.milestones.retainedComplete === undefined)) {
      throw new Error('완료 milestone 없는 complete');
    }
    this.current.status = status;
    this.current.reason = reason;
  }

  snapshot(): ScrollTrace[] { return structuredClone(this.samples); }
  clear(): void { this.samples = []; this.current = null; this.lastFrame = null; }

  private accept(id: number | null): ScrollTrace | null {
    return id !== null && id === this.id ? this.current : null;
  }
}

export interface BoundaryObservation {
  key: string;
  args: unknown[];
  result: unknown;
  error: unknown;
  failed: boolean;
  startedAt: number;
  endedAt: number;
  token: number | null;
}

/** this/인수/반환(동일 Promise 포함)/예외를 보존하고 설치 전 own descriptor까지 복원한다. */
export function observeBoundary(
  target: object,
  key: string,
  now: () => number,
  token: () => number | null,
  observe: (call: BoundaryObservation) => void,
  diagnostic: (error: unknown) => void,
): () => void {
  const object = target as Record<string, unknown>;
  const descriptor = Object.getOwnPropertyDescriptor(target, key);
  const original = object[key];
  if (typeof original !== 'function') throw new Error(`관찰 경계 없음: ${key}`);
  const wrapped = function (this: unknown, ...args: unknown[]) {
    const startedAt = now();
    const id = token();
    let result: unknown;
    let error: unknown;
    let failed = false;
    try { result = Reflect.apply(original, this, args); return result; }
    catch (e) { failed = true; error = e; throw e; }
    finally {
      const endedAt = now();
      try { observe({ key, args, result, error, failed, startedAt, endedAt, token: id }); }
      catch (e) { try { diagnostic(e); } catch { /* 관찰 오류가 제품 호출을 바꾸지 않는다. */ } }
    }
  };
  Object.defineProperty(target, key, { configurable: true, writable: true, value: wrapped });
  return () => {
    // 나중에 다른 도구가 설치한 wrapper를 덮어쓰지 않는다.
    if (object[key] !== wrapped) return;
    if (descriptor) Object.defineProperty(target, key, descriptor);
    else delete object[key];
  };
}

export function surfacePixels(surfaces: readonly { width: number; height: number }[]): number {
  return surfaces.reduce((sum, surface) => sum + surface.width * surface.height, 0);
}

/**
 * retained는 예산상 시도할 수 있는 후보 집합이다. 선택 prefetch가 admission에서 거절된 쪽은
 * surface가 없으므로, 완료 대기는 실제로 materialize된 retained working set에만 적용한다.
 */
export function admittedRetainedPages(
  retainedPages: readonly number[],
  hasSurface: (page: number) => boolean,
): number[] {
  return retainedPages.filter(hasSurface);
}

export interface ObservedViewport { scope: string; zoom: number; x: number; y: number }
/** scrollTop setter는 동기지만 visibility 갱신은 다음 scroll rAF다. 이전 화면 완료를 재사용하지 않는다. */
export function viewportApplied(target: ObservedViewport, applied: ObservedViewport | null): boolean {
  return applied !== null && target.scope === applied.scope && target.zoom === applied.zoom
    && target.x === applied.x && target.y === applied.y;
}

export function flowImageState(images: readonly { complete: boolean; naturalWidth: number }[]): 'ready' | 'pending' | 'failed' {
  if (images.some(image => image.complete && image.naturalWidth === 0)) return 'failed';
  return images.every(image => image.complete) ? 'ready' : 'pending';
}

export function currentImageRequest(scope: string, capturedScope: string, token: number | undefined, capturedToken: number): boolean {
  return scope === capturedScope && token === capturedToken;
}

/** fallback이 job을 지워도 아직 pending인 decoder를 성공으로 바꾸지 않는다. */
export function imageCompletion(kind?: 'none' | 'pending' | 'decoded' | 'cached' | 'failed'): 'ready' | 'pending' | 'failed' | 'unknown' {
  if (kind === undefined) return 'unknown';
  if (kind === 'pending' || kind === 'failed') return kind;
  return 'ready';
}
