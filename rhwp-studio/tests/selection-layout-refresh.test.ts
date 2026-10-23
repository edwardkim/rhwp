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
