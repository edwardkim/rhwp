import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(r => { resolve = r; });
  return { promise, resolve };
}

test('실제 CanvasView mutation 선택은 대기 중 바뀐 epoch·current·dispose를 폐기한다', async () => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    for (const boundary of ['epoch', 'revision', 'dispose', 'current']) {
      const pending = deferred<boolean>();
      const guards: Array<() => boolean> = [];
      let current = true;
      let applied = 0;
      let scheduled = 0;
      const selection = { backend: 'canvas2d' };
      const view = Object.assign(Object.create(CanvasView.prototype), {
        disposed: false, rendererSelectionEpoch: 0, zoomRefreshRequired: false,
        autoRendererReselectionTimer: null,
        rendererSession: {
          pinAutoMutationRevision: () => selection,
          isCurrent: () => current,
        },
        wasm: { prepareCanvasMetrics: (_backend: string, isCurrent: () => boolean) => {
          guards.push(isCurrent);
          return pending.promise;
        } },
        applyRendererSelection: () => { applied++; return false; },
        scheduleAutoRendererReselection: () => { scheduled++; },
      });
      const first = view.selectMutationRevision();
      assert.equal(guards[0](), true, `${boundary}: 현재 작업만 metrics를 반영할 수 있다`);
      if (boundary === 'epoch') view.rendererSelectionEpoch++;
      if (boundary === 'dispose') view.disposed = true;
      if (boundary === 'current') current = false;
      let second: Promise<unknown> | undefined;
      if (boundary === 'revision') second = view.selectMutationRevision();
      assert.equal(guards[0](), false, `${boundary}: 오래된 metrics 반영 차단`);
      if (second) assert.equal(guards[1](), true, '두 번째 revision만 metrics를 반영한다');
      pending.resolve(false);
      assert.equal(await first, null, `${boundary}: 오래된 완료를 반환하지 않는다`);
      if (second) await second;
      assert.equal(applied, second ? 1 : 0, `${boundary}: 최신 작업만 renderer에 적용`);
      assert.equal(scheduled, second ? 1 : 0, `${boundary}: 최신 작업만 후속 갱신 예약`);
    }
  } finally {
    await vite.close();
  }
});

test('실제 mutation refresh는 dispose 뒤 배치·선택 완료 이벤트를 발행하지 않는다', async () => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const pending = deferred<unknown>();
    const calls: string[] = [];
    const view = Object.assign(Object.create(CanvasView.prototype), {
      disposed: false,
      selectMutationRevision: () => pending.promise,
      rendererSession: { isCurrent: () => true },
      refreshPages: () => calls.push('layout'),
      eventBus: { emit: () => calls.push('selection') },
    });
    const refresh = view.refreshPagesForMutation();
    view.disposed = true;
    pending.resolve({ selection: {} });
    await refresh;
    assert.deepEqual(calls, []);
    view.disposed = false;
    await view.refreshPagesForMutation();
    assert.deepEqual(calls, ['layout', 'selection'], '완료 이벤트는 확정 배치 뒤에만 발행');
  } finally {
    await vite.close();
  }
});

test('이전 quiet 타이머가 새 mutation의 metrics 대기를 무효화하지 않는다', async (t) => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const { RendererSession } = await vite.ssrLoadModule('/src/view/renderer-session.ts');
    const rendererSession = new RendererSession(
      { backend: 'auto', source: 'url' }, { mode: 'default', source: 'default' },
      { preference: 'software', requested: 'software' }, 'screen',
      async () => { throw new Error('이 검사에서는 backend를 초기화하지 않는다'); },
    );
    t.mock.timers.enable({ apis: ['setTimeout'] });
    const metrics = deferred<boolean>();
    let quietSelections = 0;
    const calls: string[] = [];
    const view = Object.assign(Object.create(CanvasView.prototype), {
      disposed: false, rendererSelectionEpoch: 0, zoomRefreshRequired: false,
      autoRendererReselectionTimer: null, rendererSession,
      wasm: { prepareCanvasMetrics: () => metrics.promise },
      applyRendererSelection: () => false,
      refreshPages: () => calls.push('layout'),
      eventBus: { emit: (event: string) => calls.push(event) },
      selectNextDocumentRevision: async () => {
        quietSelections++;
        view.rendererSelectionEpoch++;
        rendererSession.invalidateDocument();
        return null;
      },
    });
    view.scheduleAutoRendererReselection(); // 이전 편집의 quiet 예약.
    const mutation = view.refreshPagesForMutation();
    t.mock.timers.tick(300);
    metrics.resolve(false);
    await mutation;
    assert.equal(quietSelections, 0, '이전 예약이 대기 중인 최신 mutation을 선점하지 않는다');
    assert.deepEqual(calls, ['layout', 'document-layout-refreshed']);
    t.mock.timers.tick(300);
    assert.equal(quietSelections, 1, '최신 mutation 뒤의 quiet 재선택은 유지한다');
  } finally {
    t.mock.timers.reset();
    await vite.close();
  }
});

test('quiet 재선택의 최신 배치 완료만 선택 재투영에 전달한다', async (t) => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    t.mock.timers.enable({ apis: ['setTimeout'] });
    for (const boundary of ['changed', 'unchanged', 'stale', 'dispose']) {
      const pending = deferred<unknown>();
      let current = true;
      const calls: string[] = [];
      const view = Object.assign(Object.create(CanvasView.prototype), {
        disposed: false, autoRendererReselectionTimer: null,
        rendererSession: { isCurrent: () => current },
        selectNextDocumentRevision: () => pending.promise,
        refreshPages: () => calls.push('layout'),
        eventBus: { emit: (event: string) => calls.push(event) },
      });
      view.scheduleAutoRendererReselection();
      t.mock.timers.tick(300);
      if (boundary === 'stale') current = false;
      if (boundary === 'dispose') view.disposed = true;
      pending.resolve({ backendChanged: boundary !== 'unchanged', selection: {} });
      await Promise.resolve();
      assert.deepEqual(calls, boundary === 'changed' ? ['layout', 'document-layout-refreshed'] : [], boundary);
    }
  } finally {
    t.mock.timers.reset();
    await vite.close();
  }
});

test('자원 준비 중의 viewport·page replay는 최신 revision 준비 뒤에만 재개한다', async () => {
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CanvasView } = await vite.ssrLoadModule('/src/view/canvas-view.ts');
    const first = deferred<boolean>();
    const second = deferred<boolean>();
    let revision = 0;
    let prepares = 0;
    let configured = 0;
    const view = Object.assign(Object.create(CanvasView.prototype), {
      disposed: false, rendererSelectionEpoch: 0, zoomRefreshRequired: false,
      rendererSelectionPending: false, rendererRefreshDeferred: false,
      viewportManager: {
        cancelPendingZoomRaster: () => false,
        isZoomAnimating: () => assert.fail('자원 준비 전에 viewport replay가 시작됨'),
      },
      rendererSession: {
        invalidateDocument: () => { revision++; },
        resolve: async () => ({ backend: 'canvaskit', diagnostics: {
          decisionKey: String(revision), renderProfile: 'screen',
        } }),
        isCurrent: (selection: any) => selection.diagnostics.decisionKey === String(revision),
      },
      wasm: { prepareCanvasMetrics: () => (++prepares === 1 ? first : second).promise },
      pageRenderer: {
        configure: () => { configured++; return false; },
        invalidateDocumentRevision() {},
      },
      eventBus: { emit() {} },
    });
    const old = view.selectNextDocumentRevision();
    await Promise.resolve(); await Promise.resolve();
    // 이전 쪽 목록과 폐기된 renderer 자원에는 접근 자체를 하지 않아야 한다.
    view.updateVisiblePages('scroll');
    view.renderPage(9);
    assert.equal(view.renderCanvas(9, {}), true, '대기 중에는 기존 surface를 유지한다');
    const latest = view.selectNextDocumentRevision();
    await Promise.resolve(); await Promise.resolve();
    first.resolve(false);
    assert.equal(await old, null);
    assert.equal(configured, 0, '오래된 완료는 replay를 재개하지 않는다');
    view.updateVisiblePages('resize');
    second.resolve(false);
    const selected = await latest;
    assert.equal(configured, 1);
    assert.equal(selected.backendChanged, true, 'backend가 같아도 대기 중 viewport 요청을 다시 그린다');
    assert.equal(view.deferRenderUntilRendererReady(), false, '최신 자원 준비 후에는 replay를 허용한다');
  } finally {
    await vite.close();
  }
});
