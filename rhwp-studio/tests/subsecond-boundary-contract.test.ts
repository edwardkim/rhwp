import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('개발용 런타임 import 실패는 Studio 초기화를 중단시키지 않는다', () => {
  const main = source('src/main.ts');

  assert.match(main, /if \(!import\.meta\.env\.DEV \|\| stopDevelopmentRenderRuntime \|\| !canvasView\) return;/);
  assert.match(main, /try \{[\s\S]*await import\('@\/core\/subsecond-runtime'\)/);
  assert.match(
    main,
    /catch \(error\) \{[\s\S]*console\.warn\('\[main\] 개발용 렌더 코드 교체를 시작하지 못했습니다:'/,
  );
  const canvasViewCreatedAt = main.indexOf('canvasView = new CanvasView(');
  const runtimeStartedAt = main.indexOf('await startDevelopmentRenderRuntime();', canvasViewCreatedAt);
  assert.ok(canvasViewCreatedAt >= 0 && runtimeStartedAt > canvasViewCreatedAt);
});

test('일반 Studio 클래스는 개발용 소켓과 감시자를 소유하지 않는다', () => {
  const bridge = source('src/core/wasm-bridge.ts');
  const canvasView = source('src/view/canvas-view.ts');

  assert.doesNotMatch(bridge, /subsecond-runtime|startRenderCodeReload|getRenderCodeReload|disconnectSubsecondDevtools/);
  assert.doesNotMatch(canvasView, /subsecond-runtime|startRenderCodeReloadWatch|renderCodeReloadWatcher|render-code-reloaded/);
  assert.match(source('src/main.ts'), /refreshPages\(\{ throwOnPageInfoError: true \}\)/);
});

test('비활성화된 자동 투명선 경로의 고아 이벤트를 남기지 않는다', () => {
  assert.doesNotMatch(source('src/engine/input-handler.ts'), /transparent-borders-changed/);
  assert.doesNotMatch(source('src/command/commands/view.ts'), /transparent-borders-changed/);
});

test('fidelity inspection enters through a hot WASM export', () => {
  const adapter = source('../src/subsecond_dev.rs');
  assert.match(adapter, /subsecond::register_handler/);
  assert.match(adapter, /rhwp-subsecond-commit/);
  assert.match(source('../src/wasm_api.rs'), /js_name = getLineBreakProvenance/);
  assert.match(
    source('../src/wasm_api/render_patch_boundary.rs'),
    /exports \["getLineBreakProvenance"\]/,
  );
  assert.match(source('src/core/rhwp-dev.ts'), /lineBreakVisible\(/);
});

test('visible line-break inspection distinguishes unavailable and failed batches', async () => {
  const previousWindow = (globalThis as any).window;
  const previousLog = console.log;
  const previousWarn = console.warn;
  const browser = {} as { rhwpDev?: any };
  (globalThis as any).window = browser;
  console.log = () => {};
  console.warn = () => {};
  try {
    const { initRhwpDev } = await import('../src/core/rhwp-dev.ts');
    const doc: Record<string, unknown> = {
      getPageTextLayout: () => JSON.stringify({
        runs: [0, 1].map(paraIdx => ({
          secIdx: 0, paraIdx, charStart: 0, x: 0, y: 0, text: 'x',
        })),
      }),
    };
    const wasm = { pageCount: 1, doc } as never;
    initRhwpDev(wasm);
    const missing = browser.rhwpDev.lineBreakVisible(0, { limit: 1 });
    assert.equal(missing.available, false);
    assert.equal(missing.error, 'SUBSECOND_BASE_RESTART_REQUIRED');
    assert.deepEqual(missing.items, []);

    let calls = 0;
    doc.getLineBreakProvenance = () => { throw new Error(`broken ${++calls}`); };
    initRhwpDev(wasm);
    const failed = browser.rhwpDev.lineBreakVisible(0, { limit: 1 });
    assert.equal(failed.available, true);
    assert.equal(failed.error, null);
    assert.equal(failed.errors.length, 1);
    assert.match(failed.errors[0].error, /broken 1/);
    assert.equal(failed.nextOffset, 1);
    assert.equal(calls, 1, 'one failed target still consumes the requested batch');

    doc.getPageTextLayout = () => { throw new Error('layout unavailable'); };
    const layoutFailed = browser.rhwpDev.lineBreakVisible(0, { limit: 1 });
    assert.match(layoutFailed.errors[0].error, /layout unavailable/);
  } finally {
    console.log = previousLog;
    console.warn = previousWarn;
    if (previousWindow === undefined) delete (globalThis as any).window;
    else (globalThis as any).window = previousWindow;
  }
});
