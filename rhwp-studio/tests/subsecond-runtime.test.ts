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
  revision = 'patch-two';
  frames.flush();
  assert.deepEqual(repaints, ['patch-one', 'patch-two'], '패치 버그를 고치면 재도색이 살아나야 한다');

  watcher.stop();
  assert.equal(frames.pendingCount, 0);
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
        return true;
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

test('devtools websocket resets its reconnect backoff once the socket reopens', async () => {
  const { connectSubsecondDevtools } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const scheduled: Array<() => void> = [];
  const delays: number[] = [];

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: () => true,
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
    },
  );

  // dx serve 가 꺼져 재연결이 이어지는 동안에는 대기가 두 배씩 늘어난다.
  sockets[0]?.onclose?.({ code: 1006 } as CloseEvent);
  scheduled.shift()?.();
  sockets[1]?.onclose?.({ code: 1006 } as CloseEvent);
  scheduled.shift()?.();
  assert.deepEqual(delays, [250, 500]);
  assert.equal(sockets.length, 3);

  // dx serve 가 돌아오면 다음 끊김은 다시 최소 대기에서 시작해야 한다.
  sockets[2]?.onopen?.({} as Event);
  sockets[2]?.onclose?.({ code: 1006 } as CloseEvent);
  assert.deepEqual(
    delays,
    [250, 500, 250],
    '연결이 성공하면 백오프가 최소값으로 돌아가야 한다',
  );

  disconnect?.();
});

test('patch budget warns that applied patches are never reclaimed', async () => {
  const { SubsecondPatchBudget } = await loadRuntime();
  const warnings: string[] = [];
  const budget = new SubsecondPatchBudget({
    warn: message => warnings.push(message),
    warnEveryPatches: 3,
    measureHeapBytes: () => 512 * 1024 * 1024,
  });

  budget.recordApplied();
  budget.recordApplied();
  assert.deepEqual(warnings, [], '임계값 아래에서는 경고하지 않는다');

  budget.recordApplied();
  assert.equal(warnings.length, 1);
  assert.match(warnings[0], /3/, '경고는 누적 패치 수를 담는다');
  assert.match(warnings[0], /512/, '경고는 측정한 선형 메모리 크기를 담는다');

  budget.recordApplied();
  budget.recordApplied();
  assert.equal(warnings.length, 1);
  budget.recordApplied();
  assert.equal(warnings.length, 2, '임계값을 넘길 때마다 다시 경고한다');
  assert.match(warnings[1], /6/);
});

test('devtools websocket counts only applied patches toward the budget', async () => {
  const { connectSubsecondDevtools, SubsecondPatchBudget } = await loadRuntime();
  const sockets: FakeWebSocket[] = [];
  const warnings: string[] = [];
  const patchBudget = new SubsecondPatchBudget({
    warn: message => warnings.push(message),
    warnEveryPatches: 2,
  });

  const disconnect = connectSubsecondDevtools(
    {
      applySubsecondDevtoolsMessage: (message: string) => message.includes('HotReload'),
    },
    {
      location: { protocol: 'http:', host: 'localhost:7701' },
      createWebSocket: url => new FakeWebSocket(url, sockets),
      setTimeout: () => 0,
      clearTimeout: () => {},
      patchBudget,
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
