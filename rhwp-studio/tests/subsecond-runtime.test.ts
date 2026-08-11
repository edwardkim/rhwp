import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

type RuntimeModule = typeof import('../src/core/subsecond-runtime.ts');

async function loadRuntime(): Promise<RuntimeModule> {
  try {
    return await import('../src/core/subsecond-runtime.ts');
  } catch (error) {
    assert.fail(`Subsecond runtime module is unavailable: ${String(error)}`);
  }
}

class FakeAnimationFrames {
  private nextId = 1;
  private callbacks = new Map<number, FrameRequestCallback>();

  request = (callback: FrameRequestCallback): number => {
    const id = this.nextId++;
    this.callbacks.set(id, callback);
    return id;
  };

  cancel = (id: number): void => {
    this.callbacks.delete(id);
  };

  flush(timestamp = 0): void {
    const callbacks = [...this.callbacks.values()];
    this.callbacks.clear();
    callbacks.forEach(callback => callback(timestamp));
  }

  get pendingCount(): number {
    return this.callbacks.size;
  }
}

test('revision watcher invalidates and repaints exactly once per changed HotFn revision', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision: string | null = null;
  let invalidations = 0;
  const repaints: string[] = [];
  const watcher = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => {
        invalidations += 1;
        return true;
      },
    },
    nextRevision => repaints.push(nextRevision),
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );

  assert.equal(watcher.start(), true);
  assert.equal(frames.pendingCount, 1);

  frames.flush();
  assert.equal(invalidations, 0, 'a missing document has no revision to repaint');

  revision = 'canvas-a:layers-a';
  frames.flush();
  assert.equal(invalidations, 0, 'the first document revision becomes the baseline');

  revision = 'canvas-b:layers-b';
  frames.flush();
  assert.equal(invalidations, 1);
  assert.deepEqual(repaints, ['canvas-b:layers-b']);

  frames.flush();
  assert.equal(invalidations, 1, 'an unchanged revision must not repaint again');

  watcher.stop();
  assert.equal(frames.pendingCount, 0);
});

test('revision watcher does not repaint after disposal or without a Subsecond bundle', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let repaintCount = 0;
  const disabled = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => false,
      getSubsecondPatchRevision: () => 'disabled',
      invalidateSubsecondRenderCaches: () => true,
    },
    () => {
      repaintCount += 1;
    },
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );
  assert.equal(disabled.start(), false);
  assert.equal(frames.pendingCount, 0);

  let revision = 'one';
  const active = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => true,
    },
    () => {
      repaintCount += 1;
    },
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );
  active.start();
  frames.flush();
  active.stop();
  revision = 'two';
  frames.flush();
  assert.equal(repaintCount, 0);
});

test('revision watcher coalesces revisions while animation frames are paused', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  let invalidations = 0;
  const repaints: string[] = [];
  const watcher = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => {
        invalidations += 1;
        return true;
      },
    },
    nextRevision => repaints.push(nextRevision),
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );

  watcher.start();
  frames.flush();

  revision = 'patch-one';
  revision = 'patch-two';
  revision = 'patch-three';
  assert.equal(frames.pendingCount, 1, 'a paused tab must retain only one scheduled check');

  frames.flush();
  assert.equal(invalidations, 1);
  assert.deepEqual(repaints, ['patch-three']);
  assert.equal(frames.pendingCount, 1, 'watching continues with one animation frame');

  watcher.stop();
});

test('revision watcher keeps watching after a repaint throws', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  let repaintThrows = true;
  const repaints: string[] = [];
  const watcher = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => true,
    },
    nextRevision => {
      repaints.push(nextRevision);
      if (repaintThrows) throw new Error('refreshPages failed');
    },
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );

  watcher.start();
  frames.flush();

  revision = 'patch-one';
  assert.throws(() => frames.flush(), /refreshPages failed/);
  assert.deepEqual(repaints, ['patch-one']);
  assert.equal(
    frames.pendingCount,
    1,
    '재도색이 던져도 감시 루프는 다음 프레임을 다시 예약해야 한다',
  );

  repaintThrows = false;
  frames.flush();
  assert.deepEqual(
    repaints,
    ['patch-one'],
    '실패한 리비전은 다시 시도하지 않는다 — 매 프레임 초당 60번 실패하지 않기 위한 대가다',
  );

  revision = 'patch-two';
  frames.flush();
  assert.deepEqual(
    repaints,
    ['patch-one', 'patch-two'],
    '패치 버그를 고쳐 새 리비전이 오면 재도색이 살아나야 한다',
  );

  watcher.stop();
  assert.equal(frames.pendingCount, 0);
});

test('a repaint that restarts the watcher leaves exactly one scheduled frame', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  const watcher = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => true,
    },
    () => {
      // 재도색이 감시자를 다시 세우는 경우 — 예약이 두 개로 갈라지면 그중 하나는
      // frameId 로 추적되지 않아 stop() 이 영원히 취소할 수 없는 루프가 된다.
      watcher.stop();
      watcher.start();
    },
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );

  watcher.start();
  frames.flush();

  revision = 'patch-one';
  frames.flush();
  assert.equal(frames.pendingCount, 1, '예약된 프레임은 언제나 한 개여야 한다');

  watcher.stop();
  assert.equal(frames.pendingCount, 0, 'stop() 뒤에는 취소되지 않은 루프가 남으면 안 된다');
});

test('a stopped revision watcher releases its frame and starts again', async () => {
  const { SubsecondRevisionWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  const repaints: string[] = [];
  const watcher = new SubsecondRevisionWatcher(
    {
      isSubsecondHotpatchEnabled: () => true,
      getSubsecondPatchRevision: () => revision,
      invalidateSubsecondRenderCaches: () => true,
    },
    nextRevision => repaints.push(nextRevision),
    {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  );

  watcher.start();
  frames.flush();
  watcher.stop();
  assert.equal(frames.pendingCount, 0, 'stop() 이 예약된 프레임을 해제해야 한다');

  revision = 'patch-one';
  frames.flush();
  assert.deepEqual(repaints, [], '멈춘 감시자는 패치를 그리지 않는다');

  assert.equal(watcher.start(), true, '멈춘 감시자는 다시 시작할 수 있어야 한다');
  assert.equal(frames.pendingCount, 1);
  frames.flush();
  assert.deepEqual(repaints, ['patch-one']);

  watcher.stop();
});

class FakeWebSocket {
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onopen: ((event: Event) => void) | null = null;
  readonly url: string;
  closed = false;

  constructor(url: string, sockets: FakeWebSocket[]) {
    this.url = url;
    sockets.push(this);
  }

  close(): void {
    this.closed = true;
  }
}

test('devtools websocket forwards patch messages and reconnects without reloading', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const applied: string[] = [];
  const scheduled: Array<() => void> = [];

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage(message: string) {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: callback => {
        scheduled.push(callback);
        return scheduled.length;
      },
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );

  assert.equal(typeof disconnect, 'function');
  assert.equal(sockets[0]?.url, 'ws://localhost:7701/_dioxus?build_id=0');
  sockets[0]?.onmessage?.({ data: '{"HotReload":{}}' } as MessageEvent);
  sockets[0]?.onmessage?.({ data: new Uint8Array([1]) } as MessageEvent);
  assert.deepEqual(applied, ['{"HotReload":{}}']);

  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  assert.equal(scheduled.length, 1);
  scheduled.shift()?.();
  assert.equal(sockets.length, 2, 'disconnect should reconnect instead of reloading the page');

  disconnect?.();
  assert.equal(sockets[1]?.closed, true);
});

class DiagnosticSocket {
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  closed = false;

  close(): void {
    this.closed = true;
  }
}

class FakeErrorEvents {
  private readonly listeners = new Map<string, Set<(event: Event) => void>>();

  addEventListener = (type: string, listener: (event: Event) => void): void => {
    const bucket = this.listeners.get(type) ?? new Set<(event: Event) => void>();
    bucket.add(listener);
    this.listeners.set(type, bucket);
  };

  removeEventListener = (type: string, listener: (event: Event) => void): void => {
    this.listeners.get(type)?.delete(listener);
  };

  emit(type: string, event: Record<string, unknown>): void {
    [...(this.listeners.get(type) ?? [])].forEach(listener =>
      listener({ type, ...event } as unknown as Event),
    );
  }

  count(type: string): number {
    return this.listeners.get(type)?.size ?? 0;
  }
}

type CapturedSignal = Record<string, unknown>;

/** 신호 수집기를 물린 devtools 소켓 하나를 세운다. */
async function connectWithSignals(outcomeFor: (message: string) => string): Promise<{
  socket: DiagnosticSocket;
  signals: CapturedSignal[];
  errorEvents: FakeErrorEvents;
  disconnect: () => void;
}> {
  const { connectSubsecondDevtools } = await loadRuntime();
  const socket = new DiagnosticSocket();
  const signals: CapturedSignal[] = [];
  const errorEvents = new FakeErrorEvents();
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: outcomeFor },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: () => socket,
      setTimeout: () => 0,
      clearTimeout: () => {},
      reportSignal: signal => signals.push(signal as CapturedSignal),
      errorEvents,
    },
  );
  assert.equal(typeof disconnect, 'function');
  return { socket, signals, errorEvents, disconnect: disconnect as () => void };
}

/**
 * 엔진이 선언한 결과 코드 전부 — `DevtoolsMessageOutcome::code()`(`src/subsecond_dev.rs`)에서 읽는다.
 *
 * [#4589] 계약의 단일 출처는 그 `match` 하나다. 여기 목록을 적으면 저장소에 세 번째 사본이
 * 생기고(실제로 그랬다), 어긋나도 컴파일은 통과해 런타임 경고로만 드러난다.
 */
function engineOutcomeCodes(): string[] {
  const source = readFileSync(new URL('../../src/subsecond_dev.rs', import.meta.url), 'utf8');
  const start = source.indexOf('pub fn code(');
  assert.notEqual(
    start,
    -1,
    'src/subsecond_dev.rs 에서 DevtoolsMessageOutcome::code() 를 찾지 못했다 — 계약의 출처가 옮겨졌다',
  );
  // `impl` 안의 메서드이므로 4칸 들여쓴 닫는 중괄호가 함수의 끝이다.
  const end = source.indexOf('\n    }', start);
  assert.notEqual(end, -1, 'code() 의 끝을 찾지 못했다');
  const codes = [...source.slice(start, end).matchAll(/=>\s*"([^"]+)"/g)].map(match => match[1]);
  assert.ok(codes.length > 0, 'code() 본문에서 결과 코드를 한 건도 읽지 못했다');
  return codes;
}

const ENGINE_OUTCOME_CODES = engineOutcomeCodes();

/**
 * 성공값(`patch-dispatched`)과 데브서버 정상 트래픽(`not-hot-reload`)을 뺀 나머지.
 *
 * 이 둘의 이름을 적어도 사본이 되지 않는다 — 바로 아래 테스트가 각각의 진단 등급을 이름으로
 * 확인하므로, 엔진에서 이름이 바뀌면 이 필터가 아니라 그 단언이 먼저 빨개진다.
 */
const REJECTION_CODES = ENGINE_OUTCOME_CODES.filter(
  code => code !== 'patch-dispatched' && code !== 'not-hot-reload',
);

test('the studio describes exactly the outcome codes the engine declares', async () => {
  const { SUBSECOND_OUTCOME_CODES } = await loadRuntime();

  assert.deepEqual(
    ENGINE_OUTCOME_CODES.filter(code => !SUBSECOND_OUTCOME_CODES.includes(code)),
    [],
    '엔진이 선언한 결과 코드를 스튜디오 표가 모른다 — 런타임에는 "읽지 못한 결과 값" 경고로만 드러난다',
  );
  assert.deepEqual(
    SUBSECOND_OUTCOME_CODES.filter(code => !ENGINE_OUTCOME_CODES.includes(code)),
    [],
    '스튜디오 표에 엔진이 더는 선언하지 않는 결과 코드가 남아 있다 — 도달하지 않는 진단 문구다',
  );
});

test('every devserver outcome reaches a reporter instead of being discarded', async () => {
  const { socket, signals, disconnect } = await connectWithSignals(message => message);

  ENGINE_OUTCOME_CODES.forEach(code => socket.onmessage?.({ data: code } as MessageEvent));
  socket.onmessage?.({ data: new Uint8Array([1]) } as unknown as MessageEvent);

  assert.deepEqual(
    signals,
    ENGINE_OUTCOME_CODES.map(code => ({ kind: 'outcome', code })),
    '결과는 한 건도 삼켜지지 않고, 이진 프레임은 결과를 만들지 않는다',
  );

  disconnect();
});

test('each outcome is rendered as its own line that names where to look next', async () => {
  const { describeSubsecondSignal } = await loadRuntime();

  const rejections = REJECTION_CODES.map(code =>
    describeSubsecondSignal({ kind: 'outcome', code }),
  );
  assert.deepEqual(
    rejections.map(diagnostic => diagnostic.level),
    REJECTION_CODES.map(() => 'warn'),
  );
  const messages = rejections.map(diagnostic => diagnostic.message);
  assert.equal(
    new Set(messages).size,
    REJECTION_CODES.length,
    `거절 사유는 서로 다른 문구로 구별돼야 한다: ${JSON.stringify(messages)}`,
  );
  messages.forEach(message =>
    assert.ok(message.length > 40, `다음에 볼 곳까지 말해야 한다: ${message}`),
  );

  assert.equal(
    describeSubsecondSignal({ kind: 'outcome', code: 'not-hot-reload' }).level,
    'debug',
    '데브서버 정상 트래픽을 경고로 올리면 소음이 된다',
  );
  assert.equal(
    describeSubsecondSignal({ kind: 'outcome', code: 'patch-dispatched' }).level,
    'info',
  );
});

test('an outcome the runtime does not know is surfaced instead of ignored', async () => {
  const { describeSubsecondSignal } = await loadRuntime();

  const unknown = describeSubsecondSignal({
    kind: 'outcome',
    code: true as unknown as string,
  });

  assert.equal(unknown.level, 'warn');
  assert.ok(unknown.message.includes('true'), `읽지 못한 값을 그대로 보여야 한다: ${unknown.message}`);
});

test('a global failure after a dispatched patch is reported, and one before it is not attributed', async () => {
  const { describeSubsecondSignal } = await loadRuntime();
  const { socket, signals, errorEvents, disconnect } = await connectWithSignals(
    message => message,
  );

  errorEvents.emit('error', { message: '패치와 무관한 오류' });
  assert.deepEqual(signals, [], '패치를 넘기기 전 오류는 핫패치 탓으로 돌리지 않는다');

  socket.onmessage?.({ data: 'patch-dispatched' } as MessageEvent);
  signals.length = 0;

  errorEvents.emit('error', { error: new Error('unreachable') });
  errorEvents.emit('unhandledrejection', { reason: 'patch fetch 404' });

  assert.deepEqual(
    signals,
    [
      {
        kind: 'global-failure',
        eventType: 'error',
        reason: 'Error: unreachable',
        dispatchedPatches: 1,
      },
      {
        kind: 'global-failure',
        eventType: 'unhandledrejection',
        reason: 'patch fetch 404',
        dispatchedPatches: 1,
      },
    ],
    'trap 과 rejection 양쪽 경로를 모두 잡는다',
  );

  const rendered = describeSubsecondSignal(signals[1] as never);
  assert.equal(rendered.level, 'warn');
  assert.ok(rendered.message.includes('patch fetch 404'), rendered.message);

  disconnect();
  assert.equal(errorEvents.count('error'), 0, 'disconnect 는 전역 청취를 남기지 않는다');
  assert.equal(errorEvents.count('unhandledrejection'), 0);
});

test('devtools websocket resets its reconnect backoff only after a connection that lasted', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const scheduled: Array<() => void> = [];
  const delays: number[] = [];
  let clock = 0;

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: () => 'not-hot-reload',
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: (callback, delay) => {
        scheduled.push(callback);
        delays.push(delay);
        return scheduled.length;
      },
      clearTimeout: () => {},
      now: () => clock,
    },
  );

  // dx serve 가 꺼져 연결조차 되지 않는 동안에는 대기가 두 배씩 늘어난다.
  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  scheduled.shift()?.();
  assert.deepEqual(delays, [250]);

  // 열리자마자 끊기는 연결(프록시는 살아 있고 dx serve 는 죽은 상태)은 되돌리지 않는다.
  sockets[1]?.onopen?.({} as Event);
  clock += 10;
  sockets[1]?.onclose?.({ code: 1006 } as CloseEvent);
  scheduled.shift()?.();
  assert.deepEqual(
    delays,
    [250, 500],
    '핸드셰이크만 성공한 연결로 백오프를 되돌리면 250ms 재연결이 영원히 돈다',
  );

  // dx serve 가 돌아와 실제로 붙어 있던 연결이 끊기면 다시 최소 대기에서 시작한다.
  sockets[2]?.onopen?.({} as Event);
  clock += 5_000;
  sockets[2]?.onclose?.({ code: 1006 } as CloseEvent);
  assert.deepEqual(
    delays,
    [250, 500, 250],
    '살아 있던 연결이 끊기면 백오프가 최소값으로 돌아가야 한다',
  );

  disconnect?.();
});

test('patch accumulation warns that applied patches are never reclaimed', async () => {
  const { SubsecondPatchAccumulation } = await loadRuntime();
  const warnings: string[] = [];
  const accumulation = new SubsecondPatchAccumulation({
    warn: message => warnings.push(message),
    warnEveryPatches: 3,
    measureHeapBytes: () => 512 * 1024 * 1024,
  });

  accumulation.recordApplied();
  accumulation.recordApplied();
  assert.deepEqual(warnings, [], '임계값 아래에서는 경고하지 않는다');

  accumulation.recordApplied();
  assert.equal(warnings.length, 1);
  assert.match(warnings[0], /핫패치 3건\(적용 요청 기준\)/, '경고는 누적 수와 그 수의 의미를 담는다');
  assert.match(warnings[0], /512MB/, '경고는 측정한 선형 메모리 크기를 담는다');

  accumulation.recordApplied();
  accumulation.recordApplied();
  assert.equal(warnings.length, 1);
  accumulation.recordApplied();
  assert.equal(warnings.length, 2, '임계값을 넘길 때마다 다시 경고한다');
  assert.match(warnings[1], /핫패치 6건\(적용 요청 기준\)/);
});

test('patch accumulation keeps warning when the heap cannot be measured', async () => {
  const { SubsecondPatchAccumulation } = await loadRuntime();
  const warnings: string[] = [];
  const accumulation = new SubsecondPatchAccumulation({
    warn: message => warnings.push(message),
    warnEveryPatches: 1,
    measureHeapBytes: () => {
      throw new Error('memory is detached');
    },
  });

  accumulation.recordApplied();
  assert.equal(warnings.length, 1, '측정 실패가 경고 자체를 삼키면 안 된다');
  assert.doesNotMatch(warnings[0], /선형 메모리 \d+MB/, '측정하지 못한 값을 지어내지 않는다');
});

test('devtools websocket counts only applied patches toward the accumulation', async () => {
  const { connectSubsecondDevtools, SubsecondPatchAccumulation } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const warnings: string[] = [];
  const patchAccumulation = new SubsecondPatchAccumulation({
    warn: message => warnings.push(message),
    warnEveryPatches: 2,
  });

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: (message: string) =>
        message.includes('HotReload') ? 'patch-dispatched' : 'not-hot-reload',
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: () => 0,
      clearTimeout: () => {},
      patchAccumulation,
    },
  );

  sockets[0]?.onmessage?.({ data: '{"HotPatchStart":null}' } as MessageEvent);
  sockets[0]?.onmessage?.({ data: new Uint8Array([1]) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: '{"HotReload":{}}' } as MessageEvent);
  assert.deepEqual(warnings, [], '적용되지 않은 메시지는 누적으로 세지 않는다');

  sockets[0]?.onmessage?.({ data: '{"HotReload":{}}' } as MessageEvent);
  assert.equal(warnings.length, 1);

  disconnect?.();
});

test('repository exposes a feature-gated dx adapter without changing normal WASM builds', () => {
  const cargo = readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8');
  const adapterCargo = readFileSync(
    new URL('../../tools/rhwp-subsecond/Cargo.toml', import.meta.url),
    'utf8',
  );
  const adapterBuild = readFileSync(
    new URL('../../tools/rhwp-subsecond/build.rs', import.meta.url),
    'utf8',
  );
  const wasmApi = readFileSync(new URL('../../src/wasm_api.rs', import.meta.url), 'utf8');
  const lib = readFileSync(new URL('../../src/lib.rs', import.meta.url), 'utf8');
  const bridge = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');
  const canvasView = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');
  const vite = readFileSync(new URL('../vite.config.ts', import.meta.url), 'utf8');
  const studioPackage = readFileSync(new URL('../package.json', import.meta.url), 'utf8');

  assert.match(cargo, /subsecond-dev\s*=\s*\["dep:subsecond"\]/);
  assert.match(cargo, /subsecond\s*=\s*\{\s*version\s*=\s*"=0\.7\.10",\s*optional\s*=\s*true\s*\}/);
  assert.match(cargo, /members\s*=\s*\[[\s\S]*"tools\/rhwp-subsecond"/);
  assert.match(adapterCargo, /name\s*=\s*"rhwp-subsecond"/);
  assert.match(adapterCargo, /build\s*=\s*"build\.rs"/);
  assert.match(adapterCargo, /subsecond-dev\s*=\s*\["rhwp\/subsecond-dev"\]/);
  assert.match(adapterBuild, /librhwp-dioxus\.rlib/);
  assert.match(lib, /cfg\(feature = "subsecond-dev"\)[\s\S]*mod subsecond_dev/);
  assert.match(wasmApi, /getSubsecondPatchRevision/);
  assert.match(wasmApi, /invalidateSubsecondRenderCaches/);
  assert.match(bridge, /connectSubsecondDevtools/);
  assert.match(bridge, /isSubsecondHotpatchEnabled/);
  assert.match(bridge, /getSubsecondPatchRevision/);
  assert.match(bridge, /invalidateSubsecondRenderCaches/);
  assert.match(canvasView, /new SubsecondRevisionWatcher/);
  assert.match(canvasView, /document-view-changed[\s\S]*subsecond-renderer[\s\S]*this\.refreshPages\(\)/);
  assert.match(vite, /['"]\/_dioxus['"]/);
  assert.match(vite, /['"]\/wasm['"][\s\S]*127\.0\.0\.1:7711/);
  assert.match(vite, /librhwp-subsecond-patch-\*\.wasm/);
  assert.match(vite, /handleHotUpdate[\s\S]*librhwp-subsecond-patch-/);
  assert.match(vite, /RHWP_SUBSECOND/);
  assert.match(vite, /rhwp-subsecond-vite/);
  assert.match(vite, /rhwp-subsecond\.js/);
  assert.match(studioPackage, /"subsecond:sync"[\s\S]*rhwp-subsecond-vite/);
  assert.match(studioPackage, /"subsecond:install"[\s\S]*dioxus-cli --version 0\.7\.10 --locked/);
  assert.match(studioPackage, /"subsecond:serve"[\s\S]*--package rhwp-subsecond[\s\S]*--hot-patch/);
  assert.match(studioPackage, /"dev:subsecond"\s*:\s*"npm run subsecond:sync && RHWP_SUBSECOND=1 vite"/);
});
