import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision: string | null = null;
  let invalidations = 0;
  const repaints: string[] = [];
  const watcher = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => {
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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let repaintCount = 0;
  const disabled = new RenderCodeReloadWatcher(
    {
      isAvailable: () => false,
      getRenderCodeRevision: () => 'disabled',
      rebuildDerivedState: () => true,
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
  const active = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => true,
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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  let invalidations = 0;
  const repaints: string[] = [];
  const watcher = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => {
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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  let repaintThrows = true;
  const repaints: string[] = [];
  const watcher = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => true,
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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  const watcher = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => true,
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
  const { RenderCodeReloadWatcher } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  const repaints: string[] = [];
  const watcher = new RenderCodeReloadWatcher(
    {
      isAvailable: () => true,
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => true,
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

test('render capabilities stay silent on a plain WASM build and follow the current document', async () => {
  const { createRenderCodeReload } = await loadRuntime();

  // 일반 wasm-pack 빌드 — 세 export 중 어느 것도 없다.
  const plain = createRenderCodeReload({}, () => ({}));
  assert.equal(plain.isAvailable(), false);
  assert.equal(plain.getRenderCodeRevision(), null);
  assert.equal(
    plain.rebuildDerivedState(),
    false,
    '무효화하지 못했으면 감시자가 재도색을 부르지 않도록 false 여야 한다',
  );

  // dx 핫패치 빌드 — 문서는 열고 닫을 때마다 바뀌므로 게터로 따라가야 한다.
  let invalidated = 0;
  let openDocument: object | null = null;
  const hotpatch = createRenderCodeReload(
    { subsecondProbe: () => 41 },
    () => openDocument,
  );
  assert.equal(hotpatch.isAvailable(), true);
  assert.equal(hotpatch.getRenderCodeRevision(), null, '문서가 없으면 리비전도 없다');
  assert.equal(hotpatch.rebuildDerivedState(), false);

  openDocument = {
    getRenderCodeRevision: () => 'aaaa:bbbb',
    rebuildDerivedState: () => {
      invalidated += 1;
    },
  };
  assert.equal(hotpatch.getRenderCodeRevision(), 'aaaa:bbbb');
  assert.equal(hotpatch.rebuildDerivedState(), true);
  assert.equal(invalidated, 1);

  openDocument = null;
  assert.equal(hotpatch.getRenderCodeRevision(), null, '문서를 닫으면 다시 리비전이 없다');
});

test('development render runtime owns one watcher and releases it for a later realm setup', async () => {
  const { startDevelopmentRenderRuntime } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let revision = 'baseline';
  let rebuilds = 0;
  const repaints: string[] = [];
  const document = {
    getRenderCodeRevision: () => revision,
    rebuildDerivedState: () => {
      rebuilds += 1;
    },
  };
  const exports = { subsecondProbe: () => 41 };
  const options = {
    scheduler: {
      requestAnimationFrame: frames.request,
      cancelAnimationFrame: frames.cancel,
    },
  };

  const stop = startDevelopmentRenderRuntime(exports, () => document, next => repaints.push(next), options);
  assert.ok(stop, '개발 빌드는 한 realm 수명 감시자를 시작해야 한다');
  assert.equal(
    startDevelopmentRenderRuntime(exports, () => document, () => {}, options),
    stop,
    '중복 시작은 같은 realm 소유자를 돌려줘야 한다',
  );
  frames.flush();
  revision = 'patched';
  frames.flush();
  assert.equal(rebuilds, 1);
  assert.deepEqual(repaints, ['patched']);

  stop();
  assert.equal(frames.pendingCount, 0, '해제선은 감시 프레임을 남기면 안 된다');

  const nextStop = startDevelopmentRenderRuntime(exports, () => document, () => {}, options);
  assert.ok(nextStop, '해제 뒤 다음 Studio realm은 새 감시자를 시작할 수 있어야 한다');
  assert.notEqual(nextStop, stop);
  nextStop();
});

test('development runtime replaces an older HMR realm owner', async () => {
  const { startDevelopmentRenderRuntime } = await loadRuntime();
  const frames = new FakeAnimationFrames();
  let oldStops = 0;
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondRuntime?: { implementation: number; stop: () => void };
  };
  realm.__rhwpSubsecondRuntime = {
    implementation: 1,
    stop: () => {
      oldStops += 1;
      delete realm.__rhwpSubsecondRuntime;
    },
  };
  const stop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41 },
    () => ({
      getRenderCodeRevision: () => 'baseline',
      rebuildDerivedState: () => {},
    }),
    () => {},
    {
      scheduler: {
        requestAnimationFrame: frames.request,
        cancelAnimationFrame: frames.cancel,
      },
    },
  );
  assert.equal(oldStops, 1);
  assert.equal(realm.__rhwpSubsecondRuntime?.implementation, 2);
  stop?.();
});

test('development runtime startup failure releases its patch connector', async () => {
  const { startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: Array<{
    onmessage: ((event: MessageEvent) => void) | null;
    onclose: ((event: CloseEvent) => void) | null;
    onopen: ((event: Event) => void) | null;
    closed: boolean;
    close(): void;
  }> = [];
  class StartupSocket {
    onmessage: ((event: MessageEvent) => void) | null = null;
    onclose: ((event: CloseEvent) => void) | null = null;
    onopen: ((event: Event) => void) | null = null;
    closed = false;
    constructor(_url: string) {
      sockets.push(this);
    }
    close(): void {
      this.closed = true;
    }
  }
  const errorEvents = new FakeErrorEvents();
  const previousWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
  const previousWebSocket = Object.getOwnPropertyDescriptor(globalThis, 'WebSocket');
  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: {
      location: { protocol: 'http:', host: 'localhost:7701' },
      addEventListener: errorEvents.addEventListener,
      removeEventListener: errorEvents.removeEventListener,
    },
  });
  Object.defineProperty(globalThis, 'WebSocket', {
    configurable: true,
    value: StartupSocket,
  });
  try {
    assert.throws(() => startDevelopmentRenderRuntime(
      {
        subsecondProbe: () => 41,
        applySubsecondDevtoolsMessage: () => 'patch-dispatched',
      },
      () => ({
        getRenderCodeRevision: () => {
          throw new Error('baseline failed');
        },
        rebuildDerivedState: () => {},
      }),
      () => {},
      {
        scheduler: {
          requestAnimationFrame: () => 1,
          cancelAnimationFrame: () => {},
        },
      },
    ), /baseline failed/);
    assert.equal(sockets[0]?.closed, true);
    assert.equal(errorEvents.count('error'), 0);
    assert.equal(errorEvents.count('unhandledrejection'), 0);
    assert.equal(
      (globalThis as typeof globalThis & { __rhwpSubsecondRuntime?: unknown })
        .__rhwpSubsecondRuntime,
      undefined,
    );
  } finally {
    if (previousWindow) Object.defineProperty(globalThis, 'window', previousWindow);
    else delete (globalThis as typeof globalThis & { window?: unknown }).window;
    if (previousWebSocket) Object.defineProperty(globalThis, 'WebSocket', previousWebSocket);
    else delete (globalThis as typeof globalThis & { WebSocket?: unknown }).WebSocket;
  }
});

test('development runtime cleanup releases later owners after one disposer throws', async () => {
  const { startDevelopmentRenderRuntime } = await loadRuntime();
  let canceledFrames = 0;
  let clearedTimers = 0;
  const stop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41 },
    () => ({
      getRenderCodeRevision: () => 'baseline',
      rebuildDerivedState: () => {},
    }),
    () => {},
    {
      scheduler: {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => {
          canceledFrames += 1;
        },
      },
      commitObserverScheduler: {
        setTimeout: () => 1,
        clearTimeout: () => {
          clearedTimers += 1;
        },
        now: () => 0,
      },
      commitEvents: {
        addEventListener: () => {},
        removeEventListener: () => {
          throw new Error('listener removal failed');
        },
        dispatchEvent: () => true,
      },
    },
  );
  assert.throws(() => stop?.(), /listener removal failed/);
  assert.equal(canceledFrames, 1);
  assert.equal(clearedTimers, 1);
  assert.equal(
    (globalThis as typeof globalThis & { __rhwpSubsecondRuntime?: unknown })
      .__rhwpSubsecondRuntime,
    undefined,
  );
});

test('an incompatible same-realm ledger is replaced instead of migrated', async () => {
  const realm = globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown };
  realm.__rhwpSubsecondDelivery = {
    schemaVersion: 2,
    connectionSerial: 7,
    patches: [9, 10].map(patchId => ({
      identity: `/wasm/librhwp-subsecond-patch-${patchId}.wasm`,
      commit: { status: 'unknown' },
      commitAssociation: 'ambiguous',
    })),
    events: [{ sequence: 100 }],
  };
  const {
    connectSubsecondDevtools,
    getSubsecondDeliverySnapshot,
  } = await loadRuntime();
  assert.equal(getSubsecondDeliverySnapshot(), null, 'a read must not replace incompatible state');
  assert.equal((realm.__rhwpSubsecondDelivery as { schemaVersion: number }).schemaVersion, 2);
  const sockets: FakeWebSocket[] = [];
  const stop = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'not-hot-reload' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.schemaVersion, 3);
  assert.equal(ledger.connectionSerial, 1);
  assert.equal(ledger.lastReceivedPatchIdentity, null);
  assert.equal(ledger.unresolvedCommitCandidates, 0);
  assert.equal(ledger.commitCorrelationBlocked, true);
  assert.deepEqual(ledger.patches, []);
  assert.equal(ledger.events[0]?.sequence, 101);
  assert.equal(ledger.eventSequence, 101);
  stop?.();

  realm.__rhwpSubsecondDelivery = {
    schemaVersion: 3,
    unresolvedCommitCandidates: 1,
    patches: [9, 10].map(patchId => ({
      identity: `/wasm/librhwp-subsecond-patch-${patchId}.wasm`,
      commit: { status: 'unknown' },
      commitAssociation: 'ambiguous',
    })),
  };
  const secondStop = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'not-hot-reload' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  const carried = getSubsecondDeliverySnapshot()!;
  assert.equal(carried.unresolvedCommitCandidates, 1);
  assert.equal(carried.commitCorrelationBlocked, false);
  secondStop?.();
});

test('incompatible HMR keeps the last patch identity for replay dedupe', async () => {
  const identity = '/wasm/librhwp-subsecond-patch-90.wasm';
  const realm = globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown };
  realm.__rhwpSubsecondDelivery = {
    schemaVersion: 3,
    dispatchedPatches: 1,
    lastReceivedPatchIdentity: identity,
    lastDispatchedPatchIdentity: identity,
    lastCommittedPatchIdentity: identity,
    patches: [],
  };
  const {
    connectSubsecondDevtools,
    getSubsecondDeliverySnapshot,
  } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  let applied = 0;
  const stop = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: () => {
        applied += 1;
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: () => 1,
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  sockets[0]?.onopen?.({} as Event);
  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: identity } },
  }) } as MessageEvent);
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(applied, 0);
  assert.equal(ledger.dispatchedPatches, 1);
  assert.equal(ledger.reconnectReplay.status, 'replayed');
  stop?.();

  const rejectedIdentity = '/wasm/librhwp-subsecond-patch-91.wasm';
  realm.__rhwpSubsecondDelivery = {
    schemaVersion: 3,
    dispatchedPatches: 1,
    lastReceivedPatchIdentity: rejectedIdentity,
    lastDispatchedPatchIdentity: identity,
    lastCommittedPatchIdentity: identity,
    patches: [],
  };
  const retrySockets: FakeWebSocket[] = [];
  let retried = 0;
  const retryStop = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: () => {
        retried += 1;
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, retrySockets),
      setTimeout: () => 1,
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  retrySockets[0]?.onopen?.({} as Event);
  for (const replayIdentity of [identity, '/wasm/librhwp-subsecond-patch-89.wasm']) {
    retrySockets[0]?.onmessage?.({ data: JSON.stringify({
      HotReload: { jump_table: { lib: replayIdentity } },
    }) } as MessageEvent);
  }
  assert.equal(retried, 0, 'resident and older patches remain behind the proven fence');
  retrySockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: rejectedIdentity } },
  }) } as MessageEvent);
  const retriedLedger = getSubsecondDeliverySnapshot()!;
  assert.equal(retried, 1);
  assert.equal(retriedLedger.dispatchedPatches, 2);
  assert.equal(retriedLedger.lastReceivedPatchIdentity, rejectedIdentity);
  retryStop?.();
});

test('rAF commit observation correlates rebuild before the fallback timer and remains idempotent', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: unknown;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondDelivery;
  delete realm.__rhwpSubsecondRuntime;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const patchCommitEvents = new FakeErrorEvents();
  const deliveryStop = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: patchCommitEvents,
    },
  );
  const patch = JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-70.wasm' } },
  });
  sockets[0]?.onmessage?.({ data: patch } as MessageEvent);

  let epoch = 0;
  let revision = 'a:a:a:a:a:00000000';
  let refreshError: Error | null = null;
  let rebuildError: Error | null = null;
  let rebuildAvailable = true;
  const frames = new FakeAnimationFrames();
  const commitTimers: Array<() => void> = [];
  const runtimeStop = startDevelopmentRenderRuntime(
    {
      subsecondProbe: () => 41,
      getSubsecondPatchEpoch: () => epoch,
    },
    () => ({
      getRenderCodeRevision: () => revision,
      ...(rebuildAvailable
        ? {
            rebuildDerivedState: () => {
              if (rebuildError) throw rebuildError;
            },
          }
        : {}),
    }),
    () => {
      if (refreshError) throw refreshError;
    },
    {
      scheduler: {
        requestAnimationFrame: frames.request,
        cancelAnimationFrame: frames.cancel,
      },
      commitObserverScheduler: {
        setTimeout: callback => {
          commitTimers.push(callback);
          return commitTimers.length;
        },
        clearTimeout: () => {},
        now: () => 10,
      },
    },
  );
  frames.flush();
  epoch = 1;
  revision = 'b:b:b:b:b:00000001';
  frames.flush();
  patchCommitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-70.wasm' },
  });
  let ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.patches.at(-1)?.derivedStateRebuildSequence, 1);
  assert.equal(ledger.unattributedCommits, 0);
  commitTimers.shift()?.();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.unattributedCommits, 0, 'timer fallback must not consume the same epoch twice');
  assert.equal(ledger.patches.at(-1)?.patchEpoch, 1);

  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-71.wasm' } },
  }) } as MessageEvent);
  refreshError = new Error('getPageInfo failed');
  epoch = 2;
  revision = 'c:c:c:c:c:00000002';
  assert.throws(() => frames.flush(), /getPageInfo failed/);
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.patches.at(-1)?.failure, 'Error: getPageInfo failed');
  assert.equal(ledger.lastFailure, 'Error: getPageInfo failed');
  assert.ok(ledger.events.some(event =>
    event.phase === 'render-refresh-failed'
    && event.renderRevision === revision));
  patchCommitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-71.wasm' },
  });

  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-72.wasm' } },
  }) } as MessageEvent);
  refreshError = null;
  rebuildError = new Error('rebuildDerivedState failed');
  epoch = 3;
  revision = 'd:d:d:d:d:00000003';
  assert.throws(() => frames.flush(), /rebuildDerivedState failed/);
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.patches.at(-1)?.failure, 'Error: rebuildDerivedState failed');
  assert.equal(ledger.lastFailure, 'Error: rebuildDerivedState failed');
  assert.ok(ledger.events.some(event =>
    event.phase === 'derived-state-rebuild-failed'
    && event.renderRevision === revision));
  patchCommitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-72.wasm' },
  });

  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-73.wasm' } },
  }) } as MessageEvent);
  rebuildError = null;
  rebuildAvailable = false;
  epoch = 4;
  revision = 'e:e:e:e:e:00000004';
  frames.flush();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.patches.at(-1)?.failure, 'derived-state rebuild unavailable');
  assert.equal(ledger.lastFailure, 'derived-state rebuild unavailable');
  runtimeStop?.();
  deliveryStop?.();
});

test('commit notification rebuilds a hidden-tab commit before timer and rAF fallbacks', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: unknown;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondDelivery;
  delete realm.__rhwpSubsecondRuntime;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const commitEvents = new FakeErrorEvents();
  const deliveryStop = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: commitEvents,
    },
  );
  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-71.wasm' } },
  }) } as MessageEvent);
  let epoch = 0;
  let revision = 'a:a:a:a:a:00000000';
  let rebuilds = 0;
  const repaints: string[] = [];
  const frames = new FakeAnimationFrames();
  const timers: Array<() => void> = [];
  const runtimeStop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41, getSubsecondPatchEpoch: () => epoch },
    () => ({
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => {
        rebuilds += 1;
      },
    }),
    next => repaints.push(next),
    {
      scheduler: {
        requestAnimationFrame: frames.request,
        cancelAnimationFrame: frames.cancel,
      },
      commitObserverScheduler: {
        setTimeout: callback => {
          timers.push(callback);
          return timers.length;
        },
        clearTimeout: () => {},
        now: () => 20,
      },
      commitEvents,
    },
  );
  epoch = 1;
  revision = 'b:b:b:b:b:00000001';
  commitEvents.emit('rhwp-subsecond-commit', {});
  await Promise.resolve();
  let ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.patches.at(-1)?.patchEpoch, 1);
  assert.equal(rebuilds, 1);
  assert.deepEqual(repaints, ['b:b:b:b:b:00000001']);
  assert.equal(ledger.rebuiltRevisions, 1);
  assert.equal(ledger.patches.at(-1)?.derivedStateRebuildSequence, 1);
  assert.equal(
    ledger.events.filter(event => event.phase === 'derived-state-rebuild-complete').length,
    1,
  );
  timers.shift()?.();
  frames.flush();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(rebuilds, 1, 'a later rAF must not rebuild the same committed revision twice');
  assert.equal(ledger.rebuiltRevisions, 1, 'the timer must remain idempotent after the commit event');
  assert.equal(commitEvents.count('rhwp-subsecond-commit'), 1);
  runtimeStop?.();
  assert.equal(commitEvents.count('rhwp-subsecond-commit'), 0);
  deliveryStop?.();
});

test('epoch observer releases ordered patches without an open document', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: unknown;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondDelivery;
  delete realm.__rhwpSubsecondRuntime;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const commitEvents = new FakeErrorEvents();
  const applied: string[] = [];
  const deliveryStop = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: message => {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: commitEvents,
    },
  );
  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onmessage?.({ data: patch(1) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(2) } as MessageEvent);
  assert.deepEqual(applied, [patch(1)]);

  let epoch = 0;
  const timers: Array<() => void> = [];
  const runtimeStop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41, getSubsecondPatchEpoch: () => epoch },
    () => null,
    () => {},
    {
      scheduler: {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => {},
      },
      commitObserverScheduler: {
        setTimeout: callback => {
          timers.push(callback);
          return timers.length;
        },
        clearTimeout: () => {},
        now: () => 30,
      },
      commitEvents,
    },
  );
  epoch = 1;
  timers.shift()?.();
  assert.deepEqual(applied, [patch(1), patch(2)]);
  epoch = 2;
  timers.shift()?.();
  assert.deepEqual(
    getSubsecondDeliverySnapshot()!.patches.map(delivery => delivery.commitAssociation),
    ['exact', 'exact'],
  );
  runtimeStop?.();
  deliveryStop?.();
});

test('a failed rebuild does not block the next saved patch from being dispatched', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: unknown;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondDelivery;
  delete realm.__rhwpSubsecondRuntime;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const commitEvents = new FakeErrorEvents();
  const applied: string[] = [];
  const deliveryStop = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: message => {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: commitEvents,
    },
  );
  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onmessage?.({ data: patch(1) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(2) } as MessageEvent);

  let epoch = 0;
  let revision = 'base:00000000';
  const runtimeStop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41, getSubsecondPatchEpoch: () => epoch },
    () => ({
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => {
        throw new Error('patched rebuild failed');
      },
    }),
    () => {},
    {
      scheduler: {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => {},
      },
      commitObserverScheduler: {
        setTimeout: () => 1,
        clearTimeout: () => {},
        now: () => 30,
      },
      commitEvents,
    },
  );
  epoch = 1;
  revision = 'patch:00000001';
  commitEvents.emit('rhwp-subsecond-commit', {});
  await Promise.resolve();
  assert.deepEqual(applied, [patch(1), patch(2)]);
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches[0]?.commitAssociation, 'exact');
  assert.equal(ledger.patches[0]?.failure, 'Error: patched rebuild failed');
  runtimeStop?.();
  deliveryStop?.();
});

test('opening a document after a patch rebuilds it once with the latest code', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: unknown;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondDelivery;
  delete realm.__rhwpSubsecondRuntime;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const deliveryStop = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  sockets[0]?.onmessage?.({ data: JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-72.wasm' } },
  }) } as MessageEvent);
  let epoch = 0;
  let revision: string | null = null;
  let rebuilds = 0;
  const timers: Array<() => void> = [];
  const runtimeStop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41, getSubsecondPatchEpoch: () => epoch },
    () => revision === null ? null : {
      getRenderCodeRevision: () => revision,
      rebuildDerivedState: () => {
        rebuilds += 1;
      },
    },
    () => {},
    {
      scheduler: {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => {},
      },
      commitObserverScheduler: {
        setTimeout: callback => {
          timers.push(callback);
          return timers.length;
        },
        clearTimeout: () => {},
        now: () => 25,
      },
    },
  );
  epoch = 1;
  revision = 'b:b:b:b:b:00000001';
  timers.shift()?.();
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(rebuilds, 1);
  assert.equal(ledger.rebuiltRevisions, 1);
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(ledger.events.filter(event => event.phase === 'derived-state-rebuild-complete').length, 1);
  runtimeStop?.();
  deliveryStop?.();
});

test('staggered overlapping commits remain ambiguous when a newer patch arrives', async () => {
  const realm = globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: Record<string, unknown>;
    __rhwpSubsecondRuntime?: unknown;
  };
  delete realm.__rhwpSubsecondRuntime;
  const { getSubsecondDeliverySnapshot, startDevelopmentRenderRuntime } = await loadRuntime();
  const pending = (sequence: number) => {
    const phase = { status: 'pending', at: null, evidence: null };
    return {
      identity: `/wasm/librhwp-subsecond-patch-${sequence}.wasm`,
      patchId: sequence,
      connectionSerial: 1,
      receiptAt: sequence,
      outcome: 'patch-dispatched',
      dispatch: { status: 'complete', at: sequence, evidence: 'patch-dispatched' },
      fetch: { ...phase },
      instantiate: { ...phase },
      commit: { ...phase },
      commitAssociation: 'pending',
      renderRevision: null,
      patchEpoch: null,
      derivedStateRebuildSequence: null,
      failure: null,
    };
  };
  realm.__rhwpSubsecondDelivery = {
    schemaVersion: 3,
    connectionSerial: 1,
    connected: true,
    receivedMessages: 2,
    dispatchedPatches: 2,
    ignoredPatchMessages: 0,
    lastReceivedPatchIdentity: '/wasm/librhwp-subsecond-patch-81.wasm',
    lastDispatchedPatchIdentity: '/wasm/librhwp-subsecond-patch-81.wasm',
    lastCommittedPatchIdentity: null,
    lastOutcome: 'patch-dispatched',
    lastRenderRevision: null,
    lastPatchEpoch: 0,
    rebuiltRevisions: 0,
    unattributedCommits: 0,
    unresolvedCommitCandidates: 0,
    commitCorrelationBlocked: false,
    lastFailure: null,
    reconnectReplay: {
      connectionSerial: 0,
      expectedPatchIdentity: null,
      status: 'not-applicable',
      observedAt: null,
    },
    patches: [pending(80), pending(81)],
    eventSequence: 0,
    events: [],
  };
  let epoch = 0;
  const timers: Array<() => void> = [];
  const runtimeStop = startDevelopmentRenderRuntime(
    { subsecondProbe: () => 41, getSubsecondPatchEpoch: () => epoch },
    () => null,
    () => {},
    {
      scheduler: {
        requestAnimationFrame: () => 1,
        cancelAnimationFrame: () => {},
      },
      commitObserverScheduler: {
        setTimeout: callback => {
          timers.push(callback);
          return timers.length;
        },
        clearTimeout: () => {},
        now: () => 30,
      },
    },
  );
  epoch = 1;
  timers.shift()?.();
  let ledger = getSubsecondDeliverySnapshot()!;
  assert.deepEqual(ledger.patches.slice(-2).map(patch => patch.commitAssociation), [
    'ambiguous',
    'ambiguous',
  ]);
  assert.equal(ledger.lastCommittedPatchIdentity, null);
  assert.equal(ledger.unattributedCommits, 1);
  assert.equal(ledger.unresolvedCommitCandidates, 1);

  (realm.__rhwpSubsecondDelivery.patches as unknown[]).push(pending(82));
  epoch = 2;
  timers.shift()?.();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'ambiguous');
  assert.equal(ledger.lastCommittedPatchIdentity, null);
  assert.equal(ledger.unattributedCommits, 2);
  assert.equal(ledger.unresolvedCommitCandidates, 1);

  epoch = 3;
  timers.shift()?.();
  assert.equal(getSubsecondDeliverySnapshot()!.unresolvedCommitCandidates, 0);
  (realm.__rhwpSubsecondDelivery.patches as unknown[]).push(pending(83));
  epoch = 4;
  timers.shift()?.();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.patches.at(-1)?.commitAssociation, 'exact');
  assert.equal(
    ledger.lastCommittedPatchIdentity,
    '/wasm/librhwp-subsecond-patch-83.wasm',
  );
  runtimeStop?.();
});

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

  sockets[1]?.onclose?.({ code: 1001 } as CloseEvent);
  assert.equal(scheduled.length, 1, 'remote going-away also reconnects while the page is active');
  scheduled.shift()?.();
  assert.equal(sockets.length, 3);

  disconnect?.();
  assert.equal(sockets[2]?.closed, true);
});

test('devtools disconnect closes its socket after one listener cleanup throws', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const listeners = new FakeErrorEvents();
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: {
        addEventListener: listeners.addEventListener,
        removeEventListener(type, listener) {
          listeners.removeEventListener(type, listener);
          if (type === 'error') throw new Error('error-listener removal failed');
        },
      },
    },
  );
  assert.throws(() => disconnect?.(), /error-listener removal failed/);
  assert.equal(listeners.count('error'), 0);
  assert.equal(listeners.count('unhandledrejection'), 0);
  assert.equal(sockets[0]?.closed, true);
});

test('devtools connector rolls back the first listener when the second attachment throws', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const listeners = new FakeErrorEvents();
  const sockets: FakeWebSocket[] = [];
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: {
        addEventListener(type, listener) {
          if (type === 'unhandledrejection') throw new Error('second listener failed');
          listeners.addEventListener(type, listener);
        },
        removeEventListener: listeners.removeEventListener,
      },
    },
  );
  assert.equal(disconnect, null);
  assert.equal(listeners.count('error'), 0);
  assert.equal(listeners.count('unhandledrejection'), 0);
  assert.equal(sockets.length, 0, 'socket acquisition must not start after listener failure');
});

test('a full build reloads Studio once, and a restarted devserver reloads the new base', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
  let acknowledged: string | null = null;
  let reloads = 0;
  let reloadFailures = 0;
  let acknowledgementFailures = 0;
  let connectionFailures = 0;
  let rollbackTokenFailure: string | null = null;
  let applied = 0;
  let clock = 0;
  const open = (): {
    sockets: FakeWebSocket[];
    errorEvents: FakeErrorEvents;
    timers: Array<{ callback: () => void; delay: number }>;
    stop: (() => void) | null;
  } => {
    const sockets: FakeWebSocket[] = [];
    const errorEvents = new FakeErrorEvents();
    const timers: Array<{ callback: () => void; delay: number }> = [];
    const stop = connectSubsecondDevtools(
      {
        applySubsecondDevtoolsMessage: () => {
          applied += 1;
          return 'not-hot-reload';
        },
      },
      {
        location: { protocol: 'http:', host: 'localhost:7701' },
        createWebSocket: url => {
          if (connectionFailures > 0) {
            connectionFailures -= 1;
            throw new Error('replacement connection failed');
          }
          return new FakeWebSocket(url, sockets);
        },
        setTimeout: (callback, delay) => {
          timers.push({ callback, delay });
          return timers.length;
        },
        clearTimeout: () => {},
        now: () => clock,
        reportSignal: () => {},
        errorEvents,
        fullReload: {
          acknowledgedToken: () => acknowledged,
          acknowledge: token => {
            if (acknowledgementFailures > 0) {
              acknowledgementFailures -= 1;
              throw new Error('acknowledgement failed');
            }
            if (token === rollbackTokenFailure) {
              rollbackTokenFailure = null;
              throw new Error('acknowledgement rollback failed');
            }
            acknowledged = token;
          },
          reload: () => {
            reloads += 1;
            if (reloadFailures > 0) {
              reloadFailures -= 1;
              throw new Error('reload failed');
            }
          },
        },
      },
    );
    return { sockets, errorEvents, timers, stop };
  };
  const directive = (token: string) => JSON.stringify({ RhwpFullReload: token });
  const patch = JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-1.wasm' } },
  });
  const first = open();
  first.sockets[0]!.onmessage?.({ data: directive('server-a:1') } as MessageEvent);
  first.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(applied, 0, 'old connector retires before new-base patches arrive');
  first.stop?.();

  const second = open();
  second.sockets[0]!.onmessage?.({ data: directive('server-a:1') } as MessageEvent);
  second.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(applied, 1, 'replacement connector applies history after acknowledging the token');
  second.sockets[0]!.onmessage?.({ data: directive('server-b:1') } as MessageEvent);
  second.stop?.();

  const third = open();
  third.sockets[0]!.onmessage?.({ data: directive('server-b:1') } as MessageEvent);
  third.stop?.();
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(acknowledged, 'server-b:1');
  assert.equal(reloads, 2, 'one reload per server token; persistent replay is acknowledged');
  assert.ok(ledger.events.filter(event => event.phase === 'full-reload-required').length >= 4);

  reloadFailures = 1;
  const fourth = open();
  fourth.sockets[0]!.onmessage?.({ data: directive('server-c:1') } as MessageEvent);
  assert.equal(acknowledged, 'server-b:1', 'failed reload must not acknowledge its token');
  fourth.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(applied, 1, 'failed reload retires the old socket before queued patches');
  assert.equal(fourth.sockets.length, 2, 'failure reconnects to replay the directive first');
  fourth.sockets[1]!.onmessage?.({ data: directive('server-c:1') } as MessageEvent);
  assert.equal(acknowledged, 'server-c:1');
  assert.equal(reloads, 4, 'persistent directive retries after a failed reload');
  fourth.stop?.();

  acknowledgementFailures = 1;
  const fifth = open();
  fifth.sockets[0]!.onopen?.({} as Event);
  fifth.sockets[0]!.onmessage?.({ data: directive('server-d:1') } as MessageEvent);
  fifth.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(applied, 1, 'acknowledgement failure retires before new-base history');
  assert.equal(fifth.sockets[0]!.closed, true);
  assert.equal(getSubsecondDeliverySnapshot()!.connected, false);
  assert.equal(fifth.errorEvents.count('error'), 0);
  assert.equal(fifth.errorEvents.count('unhandledrejection'), 0);
  fifth.stop?.();

  reloadFailures = 1;
  const sixth = open();
  sixth.sockets[0]!.onopen?.({} as Event);
  connectionFailures = 1;
  sixth.sockets[0]!.onmessage?.({ data: directive('server-e:1') } as MessageEvent);
  sixth.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(sixth.sockets.length, 1, 'failed replacement connect is terminal');
  assert.equal(sixth.sockets[0]!.closed, true);
  assert.equal(getSubsecondDeliverySnapshot()!.connected, false);
  sixth.stop?.();

  reloadFailures = 1;
  rollbackTokenFailure = acknowledged;
  const seventh = open();
  seventh.sockets[0]!.onopen?.({} as Event);
  seventh.sockets[0]!.onmessage?.({ data: directive('server-f:1') } as MessageEvent);
  seventh.sockets[0]!.onmessage?.({ data: patch } as MessageEvent);
  assert.equal(seventh.sockets.length, 1, 'failed acknowledgement rollback must not reconnect');
  assert.equal(seventh.sockets[0]!.closed, true);
  assert.equal(getSubsecondDeliverySnapshot()!.connected, false);
  assert.equal(applied, 1, 'no reload failure path exposes new-base patches to old WASM');
  seventh.stop?.();

  const eighth = open();
  clock = 0;
  eighth.sockets[0]!.onclose?.({ code: 1006 } as CloseEvent);
  eighth.timers.find(timer => timer.delay === 250)?.callback();
  eighth.sockets[1]!.onopen?.({} as Event);
  reloadFailures = 1;
  clock = 5_000;
  eighth.sockets[1]!.onmessage?.({ data: directive('server-g:1') } as MessageEvent);
  assert.equal(eighth.sockets.length, 3);
  eighth.sockets[2]!.onclose?.({ code: 1006 } as CloseEvent);
  assert.equal(
    eighth.timers.at(-1)?.delay,
    500,
    'pre-open replacement close must not inherit the retired socket open timestamp',
  );
  eighth.stop?.();
});

test('rapid saves reach Studio once each and in order', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const applied: string[] = [];
  const commitEvents = new FakeErrorEvents();
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
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: commitEvents,
    },
  );

  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onmessage?.({ data: patch(200) } as MessageEvent);
  commitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-200.wasm' },
  });
  sockets[0]?.onmessage?.({ data: patch(200) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(199) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(201) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(202) } as MessageEvent);

  assert.deepEqual(applied, [patch(200), patch(201)]);
  commitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-201.wasm' },
  });
  assert.deepEqual(applied, [patch(200), patch(201), patch(202)]);
  disconnect?.();
});

test('a save received before reconnect still appears after the connection returns', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const reconnects: Array<{ callback: () => void; delay: number }> = [];
  const patchReadyEvents = new FakeErrorEvents();
  const applied: string[] = [];
  const stop = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: message => {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: (callback, delay) => {
        reconnects.push({ callback, delay });
        return reconnects.length;
      },
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents,
    },
  );
  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onopen?.({} as Event);
  sockets[0]?.onmessage?.({ data: patch(1) } as MessageEvent);
  sockets[0]?.onmessage?.({ data: patch(2) } as MessageEvent);
  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  reconnects.find(timer => timer.delay === 250)?.callback();
  sockets[1]?.onopen?.({} as Event);
  patchReadyEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-1.wasm' },
  });
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.deepEqual(applied, [patch(1), patch(2)]);
  assert.equal(ledger.receivedMessages, 2);
  assert.equal(ledger.patches.at(-1)?.connectionSerial, 1);
  assert.equal(
    ledger.events.filter(event =>
      event.phase === 'message-received'
      && event.patchIdentity === '/wasm/librhwp-subsecond-patch-2.wasm'
      && event.connectionSerial === 1).length,
    1,
  );
  stop?.();
});

test('websocket reconnect observes the replayed tip without applying it twice', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const timers: Array<{ callback: () => void; delay: number }> = [];
  const commitEvents = new FakeErrorEvents();
  const applied: string[] = [];
  let clock = 10;
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: message => {
      applied.push(message);
      return 'patch-dispatched';
    } },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: (callback, delay) => {
        timers.push({ callback, delay });
        return timers.length;
      },
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      patchReadyEvents: commitEvents,
      now: () => clock,
    },
  );
  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onopen?.({} as Event);
  sockets[0]?.onmessage?.({ data: patch(41) } as MessageEvent);
  commitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-41.wasm' },
  });
  sockets[0]?.onmessage?.({ data: patch(42) } as MessageEvent);
  assert.deepEqual(applied, [patch(41), patch(42)]);
  let ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.schemaVersion, 3);
  assert.equal(ledger.patches.at(-1)?.patchId, 42);
  assert.equal(ledger.patches.at(-1)?.dispatch.status, 'complete');
  assert.equal(ledger.patches.at(-1)?.fetch.status, 'pending');
  assert.ok(ledger.events.some(event => event.phase === 'message-received'));
  assert.ok(ledger.events.some(event => event.phase === 'dispatch-outcome'));
  clock = 20;
  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  timers.find(timer => timer.delay === 250)?.callback();
  sockets[1]?.onopen?.({} as Event);
  clock = 30;
  sockets[1]?.onmessage?.({ data: patch(41) } as MessageEvent);
  assert.equal(
    getSubsecondDeliverySnapshot()!.reconnectReplay.status,
    'pending',
    'older ordered history does not supersede the expected tip replay',
  );
  sockets[1]?.onmessage?.({ data: patch(42) } as MessageEvent);
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.reconnectReplay.status, 'replayed');
  assert.ok(ledger.events.some(event => event.phase === 'patch-replayed'));
  timers.find(timer => timer.delay === 1_000)?.callback();
  assert.equal(
    getSubsecondDeliverySnapshot()!.reconnectReplay.status,
    'replayed',
    'a physically received replay cannot become missed while queued',
  );
  commitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-42.wasm' },
  });
  await Promise.resolve();
  ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.dispatchedPatches, 2, 'ordered history is observed but not dispatched twice');
  assert.deepEqual(applied, [patch(41), patch(42)]);
  disconnect?.();
});

test('Studio catches up whether reconnect sees a newer save or a late replay', async () => {
  const run = async (lateReplay: boolean): Promise<string> => {
    delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
      .__rhwpSubsecondDelivery;
    const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
    const sockets: FakeWebSocket[] = [];
    const timers: Array<{ callback: () => void; delay: number }> = [];
    const commitEvents = new FakeErrorEvents();
    const disconnect = connectSubsecondDevtools(
      { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
      {
        location: { protocol: 'http:', host: 'localhost:7701' },
        createWebSocket: url => new FakeWebSocket(url, sockets),
        setTimeout: (callback, delay) => {
          timers.push({ callback, delay });
          return timers.length;
        },
        clearTimeout: () => {},
        reportSignal: () => {},
        errorEvents: new FakeErrorEvents(),
        patchReadyEvents: commitEvents,
      },
    );
    const patch = (sequence: number) => JSON.stringify({
      HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
    });
    sockets[0]?.onopen?.({} as Event);
    sockets[0]?.onmessage?.({ data: patch(90) } as MessageEvent);
    commitEvents.emit('rhwp-subsecond-patch-ready', {
      detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-90.wasm' },
    });
    sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
    timers.find(timer => timer.delay === 250)?.callback();
    sockets[1]?.onopen?.({} as Event);
    if (lateReplay) {
      timers.find(timer => timer.delay === 1_000)?.callback();
      assert.equal(
        getSubsecondDeliverySnapshot()!.reconnectReplay.status,
        'not-observed-by-deadline',
      );
      sockets[1]?.onmessage?.({ data: patch(90) } as MessageEvent);
    } else {
      sockets[1]?.onmessage?.({ data: patch(91) } as MessageEvent);
    }
    const status = getSubsecondDeliverySnapshot()!.reconnectReplay.status;
    disconnect?.();
    return status;
  };
  assert.equal(await run(false), 'superseded');
  assert.equal(await run(true), 'replayed');
});

test('a slow patch never lets a newer save overtake it', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const {
    connectSubsecondDevtools,
    describeSubsecondSignal,
    getSubsecondDeliverySnapshot,
  } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const timers: Array<{ callback: () => void; delay: number }> = [];
  const commitEvents = new FakeErrorEvents();
  const errorEvents = new FakeErrorEvents();
  const applied: string[] = [];
  const signals: CapturedSignal[] = [];
  const open = () => connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: message => {
        applied.push(message);
        return 'patch-dispatched';
      },
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: (callback, delay) => {
        timers.push({ callback, delay });
        return timers.length;
      },
      clearTimeout: () => {},
      reportSignal: signal => signals.push(signal as CapturedSignal),
      errorEvents,
      patchReadyEvents: commitEvents,
      now: () => 0,
    },
  );
  const stop = open();
  const patch = (sequence: number) => JSON.stringify({
    HotReload: { jump_table: { lib: `/wasm/librhwp-subsecond-patch-${sequence}.wasm` } },
  });
  sockets[0]?.onmessage?.({ data: patch(1) } as MessageEvent);
  errorEvents.emit('unhandledrejection', { reason: 'patch instantiate failed' });
  sockets[0]?.onmessage?.({ data: patch(2) } as MessageEvent);
  stop?.();

  const replacementStop = open();
  sockets[1]?.onopen?.({} as Event);
  sockets[1]?.onmessage?.({ data: patch(1) } as MessageEvent);
  sockets[1]?.onmessage?.({ data: patch(2) } as MessageEvent);
  timers.filter(timer => timer.delay === 10_000).at(-1)?.callback();
  sockets[1]?.onmessage?.({ data: patch(3) } as MessageEvent);

  let ledger = getSubsecondDeliverySnapshot()!;
  assert.deepEqual(applied, [patch(1)]);
  assert.equal(ledger.ignoredPatchMessages, 1);
  assert.equal(ledger.events.at(-1)?.phase, 'patch-ignored');
  assert.ok(ledger.events.some(event => event.phase === 'patch-commit-timeout'));
  assert.deepEqual(signals.at(-1), {
    kind: 'commit-timeout',
    patchIdentity: '/wasm/librhwp-subsecond-patch-1.wasm',
    timeoutMs: 10_000,
  });
  assert.match(
    describeSubsecondSignal(signals.at(-1) as never).message,
    /patch-1\.wasm[\s\S]*full rebuild/,
  );

  commitEvents.emit('rhwp-subsecond-patch-ready', {
    detail: { patchIdentity: '/wasm/librhwp-subsecond-patch-1.wasm' },
  });
  sockets[1]?.onmessage?.({ data: patch(2) } as MessageEvent);
  ledger = getSubsecondDeliverySnapshot()!;
  assert.deepEqual(applied, [patch(1), patch(2)]);
  assert.equal(ledger.dispatchedPatches, 2, 'a late exact commit safely reopens dispatch');
  replacementStop?.();
});

test('disconnect settles a pending reconnect replay as closed', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const timers: Array<{ callback: () => void; delay: number }> = [];
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'patch-dispatched' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: (callback, delay) => {
        timers.push({ callback, delay });
        return timers.length;
      },
      clearTimeout: () => {},
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
      now: () => 40,
    },
  );
  const patch = JSON.stringify({
    HotReload: { jump_table: { lib: '/wasm/librhwp-subsecond-patch-90.wasm' } },
  });
  sockets[0]?.onopen?.({} as Event);
  sockets[0]?.onmessage?.({ data: patch } as MessageEvent);
  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  timers.find(timer => timer.delay === 250)?.callback();
  sockets[1]?.onopen?.({} as Event);
  assert.equal(getSubsecondDeliverySnapshot()!.reconnectReplay.status, 'pending');
  disconnect?.();
  const ledger = getSubsecondDeliverySnapshot()!;
  assert.equal(ledger.reconnectReplay.status, 'closed');
  assert.equal(ledger.reconnectReplay.observedAt, 40);
  assert.ok(ledger.events.some(event =>
    event.phase === 'patch-replay-missed' && event.detail === 'connection-closed'));
});

test('delivery ledger keeps a bounded in-page event log', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
  const { connectSubsecondDevtools, getSubsecondDeliverySnapshot } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const disconnect = connectSubsecondDevtools(
    { applySubsecondDevtoolsMessage: () => 'not-hot-reload' },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      reportSignal: () => {},
      errorEvents: new FakeErrorEvents(),
    },
  );
  for (let index = 0; index < 200; index++) {
    sockets[0]?.onmessage?.({ data: JSON.stringify({ Ping: index }) } as MessageEvent);
  }
  const events = getSubsecondDeliverySnapshot()!.events;
  assert.equal(events.length, 128);
  assert.ok(events[0].sequence > 1, 'oldest events are evicted without resetting sequence');
  disconnect?.();
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

  dispatchEvent = (event: Event): boolean => {
    [...(this.listeners.get(event.type) ?? [])].forEach(listener => listener(event));
    return true;
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

test('applied patch outcome has one source shared by diagnostics and accumulation', async () => {
  const {
    PATCH_DISPATCHED_OUTCOME,
    SUBSECOND_OUTCOME_CODES,
    isPatchDispatchedOutcome,
  } = await loadRuntime();

  assert.ok(
    ENGINE_OUTCOME_CODES.includes(PATCH_DISPATCHED_OUTCOME),
    '누적 계수가 쓰는 적용 결과는 엔진의 DevtoolsMessageOutcome::code()에 있어야 한다',
  );
  assert.ok(
    SUBSECOND_OUTCOME_CODES.includes(PATCH_DISPATCHED_OUTCOME),
    '누적 계수가 쓰는 적용 결과는 Studio 진단 표에도 있어야 한다',
  );
  assert.equal(isPatchDispatchedOutcome(PATCH_DISPATCHED_OUTCOME), true);
  assert.equal(isPatchDispatchedOutcome('not-hot-reload'), false);
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
  assert.match(
    describeSubsecondSignal({ kind: 'outcome', code: 'undeserializable-jump-table' }).message,
    /`version` 필드[\s\S]*target\/dioxus-cli\/bin\/dx --version/,
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
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
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

  delete ((globalThis as typeof globalThis & {
    __rhwpSubsecondDelivery?: { eventSequence?: number };
  }).__rhwpSubsecondDelivery)?.eventSequence;
  const replacement = await connectWithSignals(() => 'not-hot-reload');
  replacement.errorEvents.emit('error', { message: 'resident patched code trapped' });
  assert.deepEqual(replacement.signals, [{
    kind: 'global-failure',
    eventType: 'error',
    reason: 'resident patched code trapped',
    dispatchedPatches: 1,
  }]);
  replacement.disconnect();
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

  accumulation.recordApplied(1);
  accumulation.recordApplied(2);
  assert.deepEqual(warnings, [], '임계값 아래에서는 경고하지 않는다');

  accumulation.recordApplied(3);
  assert.equal(warnings.length, 1);
  assert.match(warnings[0], /핫패치 3건\(적용 요청 기준\)/, '경고는 누적 수와 그 수의 의미를 담는다');
  assert.match(warnings[0], /512MB/, '경고는 측정한 선형 메모리 크기를 담는다');

  accumulation.recordApplied(4);
  accumulation.recordApplied(5);
  assert.equal(warnings.length, 1);
  accumulation.recordApplied(6);
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

  accumulation.recordApplied(1);
  assert.equal(warnings.length, 1, '측정 실패가 경고 자체를 삼키면 안 된다');
  assert.doesNotMatch(warnings[0], /선형 메모리 \d+MB/, '측정하지 못한 값을 지어내지 않는다');
});

test('patch accumulation keeps the realm total across an HMR-owned instance', async () => {
  const { SubsecondPatchAccumulation } = await loadRuntime();
  const warnings: string[] = [];
  const options = {
    warn: (message: string) => warnings.push(message),
    warnEveryPatches: 32,
  };
  new SubsecondPatchAccumulation(options).recordApplied(31);
  new SubsecondPatchAccumulation(options).recordApplied(32);
  assert.equal(warnings.length, 1);
  assert.match(warnings[0], /핫패치 32건\(적용 요청 기준\)/);
});

test('devtools websocket counts only applied patches toward the accumulation', async () => {
  delete (globalThis as typeof globalThis & { __rhwpSubsecondDelivery?: unknown })
    .__rhwpSubsecondDelivery;
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

/** 매니페스트·잠금 파일 한 쌍을 담은 임시 저장소 뿌리를 만든다. */
function fakeRepository(
  pin: string,
  locked: string,
  registry = '',
  source = 'registry+https://github.com/rust-lang/crates.io-index',
): string {
  const root = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-version-'));
  writeFileSync(
    path.join(root, 'Cargo.toml'),
    `[dependencies]\nsubsecond = { version = "${pin}"${registry ? `, registry = "${registry}"` : ''}, optional = true }\n`,
  );
  writeFileSync(
    path.join(root, 'Cargo.lock'),
    `[[package]]\nname = "subsecond"\nversion = "${locked}"\nsource = "${source}"\n`,
  );
  return root;
}

function fakeGitRepository(requestedRev: string, lockedRev = requestedRev): string {
  const root = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-git-source-'));
  const git = 'https://github.com/DioxusLabs/dioxus.git';
  writeFileSync(
    path.join(root, 'Cargo.toml'),
    `[dependencies]\nsubsecond = { git = "${git}", rev = "${requestedRev}", optional = true }\n`,
  );
  writeFileSync(
    path.join(root, 'Cargo.lock'),
    `[[package]]\nname = "subsecond"\nversion = "0.8.0-alpha.1"\nsource = "git+${git}?rev=${requestedRev}#${lockedRev}"\n`,
  );
  return root;
}

test('installer guard: dx source follows the pinned Subsecond runtime source', async () => {
  const { dioxusCliSource, dioxusCliVersion } = await import('../../scripts/dioxus-cli-version.mjs');
  const repoRoot = new URL('../../', import.meta.url);
  const source = dioxusCliSource(fileURLToPath(repoRoot));
  const derived = dioxusCliVersion(fileURLToPath(repoRoot));

  // 유도값은 실제로 컴파일되는 버전이어야 한다 — 잠금 파일이 해결한 값.
  assert.ok(
    readFileSync(new URL('Cargo.lock', repoRoot), 'utf8')
      .includes(`\nname = "subsecond"\nversion = "${derived}"\n`),
    `Cargo.lock 이 해결한 subsecond 버전과 유도값(${derived})이 다르다`,
  );

  // dependabot 이 `Cargo.toml` 만 올려도 설치 버전이 따라간다 — 사본이 없기 때문이다.
  const installScript = JSON.parse(
    readFileSync(new URL('rhwp-studio/package.json', repoRoot), 'utf8'),
  ).scripts['subsecond:install'];
  assert.doesNotMatch(
    installScript,
    /\d+\.\d+\.\d+/,
    `subsecond:install 에 버전 사본이 다시 생겼다: ${installScript}`,
  );
  assert.ok(installScript.includes('scripts/install-dioxus-cli.mjs'), installScript);
  assert.equal(source.version, derived);
});

test('a drifted or loose pin stops the install instead of fetching the wrong dx', async () => {
  const { dioxusCliSource, dioxusCliVersion } = await import('../../scripts/dioxus-cli-version.mjs');

  assert.equal(dioxusCliVersion(fakeRepository('=1.2.3', '1.2.3')), '1.2.3');
  assert.equal(
    dioxusCliVersion(fakeRepository('=1.2.3-alpha.1', '1.2.3-alpha.1')),
    '1.2.3-alpha.1',
  );

  // 핀과 잠금이 갈라진 상태. 그대로 설치하면 어느 쪽과도 맞지 않는 dx 가 깔린다.
  assert.throws(
    () => dioxusCliVersion(fakeRepository('=1.2.3', '1.2.4')),
    /registry 핀\(1\.2\.3\)[\s\S]*lock\(1\.2\.4/,
  );

  // 느슨한 핀이면 잠금이 언제든 앞서 나갈 수 있어 유도 자체가 성립하지 않는다.
  assert.throws(() => dioxusCliVersion(fakeRepository('1.2', '1.2.9')), /정확 핀이 아니다/);
  assert.throws(() => dioxusCliVersion(fakeRepository('=1.2.3', '1.2.3', 'custom')), /custom registry/);
  assert.throws(
    () => dioxusCliVersion(fakeRepository('=1.2.3', '1.2.3', '', 'registry+https://example.test/index')),
    /registry 핀[\s\S]*example\.test/,
  );

  const rev = '1234567890abcdef1234567890abcdef12345678';
  assert.deepEqual(dioxusCliSource(fakeGitRepository(rev)), {
    kind: 'git',
    git: 'https://github.com/DioxusLabs/dioxus.git',
    rev,
    version: '0.8.0-alpha.1',
  });
  assert.throws(
    () => dioxusCliSource(fakeGitRepository(rev, 'abcdef1234567890abcdef1234567890abcdef12')),
    /git rev[\s\S]*Cargo\.lock source/,
  );
});

test('installer guard: cargo receives the exact dx source revision', async () => {
  const { dioxusCliInstallArgs, dioxusCliSourceDir } = await import('../../scripts/install-dioxus-cli.mjs');
  const rev = '1234567890abcdef1234567890abcdef12345678';
  const source = {
    kind: 'git' as const,
    git: 'https://github.com/DioxusLabs/dioxus.git',
    rev,
    version: '0.8.0-alpha.1',
  };
  const preparedSourceDir = `/repo/target/dioxus-cli-source/${rev}-patch-generation`;
  assert.deepEqual(
    dioxusCliInstallArgs(source, '/repo', preparedSourceDir),
    [
      'install', 'dioxus-cli',
      '--path', `${preparedSourceDir}/packages/cli`,
      '--locked', '--root', '/repo/target/dioxus-cli',
    ],
  );
  const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
  assert.match(dioxusCliSourceDir(source, repoRoot), new RegExp(`${rev}-[0-9a-f]{16}$`));
});

test('installer guard: hidden dx source is rejected while build output is allowed', async () => {
  const { verifyDioxusCliCheckoutFiles } = await import('../../scripts/install-dioxus-cli.mjs');
  const checkout = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-checkout-'));
  execFileSync('git', ['init', '--quiet', checkout]);
  writeFileSync(path.join(checkout, '.git', 'info', 'exclude'), 'hidden.rs\ntarget/\n');
  mkdirSync(path.join(checkout, 'target'));
  writeFileSync(path.join(checkout, 'target', 'artifact'), 'build output\n');

  assert.doesNotThrow(() => verifyDioxusCliCheckoutFiles(checkout));
  writeFileSync(path.join(checkout, 'hidden.rs'), 'fn hidden() {}\n');
  assert.throws(() => verifyDioxusCliCheckoutFiles(checkout), /ignored source[\s\S]*hidden\.rs/);
});

test('installer guard: masked tracked dx source is rejected', async () => {
  const { verifyDioxusCliCheckoutFiles } = await import('../../scripts/install-dioxus-cli.mjs');
  const checkout = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-index-'));
  execFileSync('git', ['init', '--quiet', checkout]);
  writeFileSync(path.join(checkout, 'tracked.rs'), 'fn original() {}\n');
  execFileSync('git', ['-C', checkout, 'add', 'tracked.rs']);
  execFileSync('git', ['-C', checkout, 'update-index', '--assume-unchanged', 'tracked.rs']);
  writeFileSync(path.join(checkout, 'tracked.rs'), 'fn replaced() {}\n');

  assert.throws(() => verifyDioxusCliCheckoutFiles(checkout), /index flag[\s\S]*tracked\.rs/);
});

test('installer guard: Git filters cannot hide dx source changes', async () => {
  const { dioxusCliSourceDigest, verifyDioxusPristineCheckout } = await import('../../scripts/install-dioxus-cli.mjs');
  const checkout = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-filter-'));
  execFileSync('git', ['init', '--quiet', checkout]);
  writeFileSync(path.join(checkout, '.gitattributes'), 'tracked.rs filter=review-mask\n');
  writeFileSync(path.join(checkout, 'tracked.rs'), 'fn original() {}\n');
  execFileSync('git', ['-C', checkout, 'add', '.gitattributes', 'tracked.rs']);
  execFileSync('git', ['-C', checkout, '-c', 'user.name=review', '-c', 'user.email=review@example.test', 'commit', '--quiet', '-m', 'base']);
  const originalDigest = dioxusCliSourceDigest(checkout);
  execFileSync('git', ['-C', checkout, 'config', 'filter.review-mask.clean', "sed 's/replaced/original/'"]);
  writeFileSync(path.join(checkout, 'tracked.rs'), 'fn replaced() {}\n');

  execFileSync('git', ['-C', checkout, 'diff', '--exit-code']);
  assert.notEqual(dioxusCliSourceDigest(checkout), originalDigest);
  assert.throws(
    () => verifyDioxusPristineCheckout(checkout),
    /raw source[\s\S]*tracked\.rs/,
  );
});

test('the installer cache digest has unambiguous binary file boundaries', async () => {
  const { dioxusCliSourceDigest } = await import('../../scripts/install-dioxus-cli.mjs');
  const makeCheckout = (a: Uint8Array, b: Uint8Array): string => {
    const checkout = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-digest-'));
    execFileSync('git', ['init', '--quiet', checkout]);
    writeFileSync(path.join(checkout, 'a'), a);
    writeFileSync(path.join(checkout, 'b'), b);
    execFileSync('git', ['-C', checkout, 'add', 'a', 'b']);
    return checkout;
  };
  const left = makeCheckout(Buffer.from('x'), Buffer.from('y\0b\0z'));
  const right = makeCheckout(Buffer.from('x\0b\0y'), Buffer.from('z'));

  assert.notEqual(dioxusCliSourceDigest(left), dioxusCliSourceDigest(right));
});

test('the pristine checkout verifier hashes symlink targets as Git blobs', async () => {
  const { dioxusCliSourceDigest, verifyDioxusPristineCheckout } = await import('../../scripts/install-dioxus-cli.mjs');
  const checkout = mkdtempSync(path.join(tmpdir(), 'rhwp-dx-symlink-'));
  execFileSync('git', ['init', '--quiet', checkout]);
  writeFileSync(path.join(checkout, 'target.txt'), 'contents\n');
  mkdirSync(path.join(checkout, 'directory'));
  symlinkSync('target.txt', path.join(checkout, 'link'));
  symlinkSync('missing', path.join(checkout, 'broken-link'));
  symlinkSync('directory', path.join(checkout, 'directory-link'));
  execFileSync('git', ['-C', checkout, 'add', 'target.txt', 'link', 'broken-link', 'directory-link']);

  assert.doesNotThrow(() => verifyDioxusPristineCheckout(checkout));
  assert.match(dioxusCliSourceDigest(checkout), /^[0-9a-f]{64}$/);
});

test('installer guard: the reviewed Dioxus workaround diff is reproduced exactly', () => {
  const tipPatch = readFileSync(
    new URL('../../scripts/patches/dioxus-cli-hotpatch-tip-dependents.patch', import.meta.url),
    'utf8',
  );
  const replayPatch = readFileSync(
    new URL('../../scripts/patches/dioxus-cli-replay-latest-patch.patch', import.meta.url),
    'utf8',
  );
  assert.match(tipPatch, /workspace_crate_dep_names/);
  assert.doesNotMatch(tipPatch, /serve\/server\.rs/);
  assert.match(replayPatch, /serve\/server\.rs/);
  assert.doesNotMatch(replayPatch, /build\/builder\.rs/);
  const installer = readFileSync(
    new URL('../../scripts/install-dioxus-cli.mjs', import.meta.url),
    'utf8',
  );
  assert.match(
    installer,
    /apply', '--check', '--unidiff-zero'/,
  );
  assert.match(installer, /const PATCH_DIFF_ARGS/);
  for (const flag of ['--abbrev=8', '--no-color', '--no-textconv', '--inter-hunk-context=0', '--diff-algorithm=myers', '-O/dev/null']) {
    assert.ok(installer.includes(`'${flag}'`), flag);
  }
  assert.match(installer, /'status', '--porcelain=v1', '--untracked-files=all'/);
  assert.match(installer, /rhwp-patched-v2[\s\S]*mkdtempSync[\s\S]*preparedHead[\s\S]*preparedDiff[\s\S]*writeFileSync[\s\S]*renameSync/);
  assert.match(installer, /ls-files[\s\S]*--ignored[\s\S]*ignored source/);
  assert.doesNotMatch(installer, /'diff', '--name-only'/);
});

/**
 * 핫패치 개발 배선이 **매니페스트·설정에 선언되어 있는지** 본다.
 *
 * [#4593] 여기 정규식이 겨눠도 되는 것은 실행되지 않는 파일뿐이다 — `Cargo.toml`,
 * `vite.config.ts`, `package.json` 에서는 "이 문자열이 이 자리에 있다"가 곧 계약이고, 다른
 * 확인 방법도 없다(cargo·vite·npm 이 그 문자열을 읽는다).
 *
 * **코드 파일을 겨누는 단언은 여기 두지 않는다.** 소스에 문자열이 있다는 것은 그 코드가
 * 실행된다는 증거가 아니라서, 실제로 거짓 초록을 만들었다 — `tools/rhwp-subsecond/build.rs` 의
 * `librhwp-dioxus.rlib` 를 찾던 단언은 그 이름을 쓰는 심링크 생성이 `#[cfg(unix)]` 뒤에 있어
 * Windows 에서는 아예 컴파일되지 않는데도 초록이었다. 같은 부류로 #4579 가 `CanvasView.dispose()`
 * 안의 `stop()` 을 찾던 단언 하나를 이미 행동 단언으로 바꿨다 — 호출부가 0개라 그 `stop()` 은
 * 절대 실행되지 않았다.
 *
 * 새 계약이 생기면 물어야 할 것은 하나다: 이 문자열이 있다는 것과 이 동작이 일어난다는 것이
 * 같은가. 다르면 행동으로 확인하거나, 확인할 수 없다는 사실을 남긴다.
 */
test('hot-patch dev wiring is declared in the manifests and the vite config', () => {
  const cargo = readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8');
  const adapterCargo = readFileSync(
    new URL('../../tools/rhwp-subsecond/Cargo.toml', import.meta.url),
    'utf8',
  );
  const vite = readFileSync(new URL('../vite.config.ts', import.meta.url), 'utf8');
  const studioPackage = readFileSync(new URL('../package.json', import.meta.url), 'utf8');

  assert.match(cargo, /subsecond-dev\s*=\s*\["dep:subsecond"\]/);
  // SHA 값은 여기 적지 않는다 — 사본이 하나 더 생기면 그것이 #4580 이 없앤 드리프트다.
  // official git의 40자리 정확 rev라는 사실만 보고, lock과의 정합은 위 전용 테스트가 확인한다.
  assert.match(
    cargo,
    /subsecond\s*=\s*\{\s*git\s*=\s*"https:\/\/github\.com\/DioxusLabs\/dioxus\.git",\s*rev\s*=\s*"[0-9a-f]{40}",\s*optional\s*=\s*true\s*\}/,
  );
  assert.match(cargo, /members\s*=\s*\[[\s\S]*"tools\/rhwp-subsecond"/);
  assert.match(adapterCargo, /name\s*=\s*"rhwp-subsecond"/);
  assert.match(adapterCargo, /build\s*=\s*"build\.rs"/);
  assert.match(adapterCargo, /subsecond-dev\s*=\s*\["rhwp\/subsecond-dev"\]/);
  assert.match(vite, /['"]\/_dioxus['"]/);
  assert.match(vite, /['"]\/wasm['"][\s\S]*127\.0\.0\.1:7711/);
  assert.match(vite, /librhwp-subsecond-patch-\*\.wasm/);
  assert.match(vite, /handleHotUpdate[\s\S]*librhwp-subsecond-patch-/);
  assert.match(vite, /RHWP_SUBSECOND/);
  assert.match(vite, /rhwp-subsecond-vite/);
  assert.match(vite, /rhwp-subsecond\.js/);
  assert.match(studioPackage, /"subsecond:sync"[\s\S]*rhwp-subsecond-vite/);
  assert.match(studioPackage, /"subsecond:install"[\s\S]*scripts\/install-dioxus-cli\.mjs/);
  assert.match(studioPackage, /"subsecond:serve"[\s\S]*--package rhwp-subsecond[\s\S]*--hot-patch/);
  assert.match(studioPackage, /"subsecond:serve"[\s\S]*DIOXUS_LOG=[^"\s]*subsecond_cli_support=trace/);
  assert.match(studioPackage, /"subsecond:serve"[\s\S]*--trace[\s\S]*--log-to-file target\/subsecond-dx\.log/);
  assert.match(studioPackage, /"subsecond:serve"[\s\S]*--interactive false[\s\S]*--cargo-args=--locked[\s\S]*--keep-names/);
  assert.match(studioPackage, /"dev:subsecond"\s*:\s*"npm run subsecond:sync && RHWP_SUBSECOND=1 vite"/);
});
