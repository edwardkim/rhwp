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

test('devtools websocket forwards patch messages and reconnects without reloading', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const applied: string[] = [];
  const scheduled: Array<() => void> = [];

  class FakeWebSocket {
    onmessage: ((event: MessageEvent) => void) | null = null;
    onclose: ((event: CloseEvent) => void) | null = null;
    readonly url: string;
    closed = false;

    constructor(url: string) {
      this.url = url;
      sockets.push(this);
    }

    close(): void {
      this.closed = true;
    }
  }

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage(message: string) {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url),
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

const REJECTION_CODES = [
  'not-json',
  'foreign-build-id',
  'missing-jump-table',
  'undeserializable-jump-table',
  'patch-rejected',
];

test('every devserver outcome reaches a reporter instead of being discarded', async () => {
  const { socket, signals, disconnect } = await connectWithSignals(message => message);

  [...REJECTION_CODES, 'not-hot-reload', 'patch-dispatched'].forEach(code =>
    socket.onmessage?.({ data: code } as MessageEvent),
  );
  socket.onmessage?.({ data: new Uint8Array([1]) } as unknown as MessageEvent);

  assert.deepEqual(
    signals,
    [...REJECTION_CODES, 'not-hot-reload', 'patch-dispatched'].map(code => ({
      kind: 'outcome',
      code,
    })),
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
    `다섯 사유는 서로 다른 문구로 구별돼야 한다: ${JSON.stringify(messages)}`,
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
  assert.match(canvasView, /subsecondRevisionWatcher\.stop\(\)/);
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
