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

test('a multi-page paragraph reports the page containing its first differing break', async () => {
  const previousWindow = (globalThis as any).window;
  const previousLog = console.log;
  (globalThis as any).window = {};
  console.log = () => {};
  try {
    const { initRhwpDev } = await import('../src/core/rhwp-dev.ts');
    const { formatFirstLineBreakError } = await import('../src/dev/document-error-log.ts');
    const provenance = {
      textUtf16Length: 200,
      coordinates: { sectionIdx: 0, paragraphIdx: 4 },
      comparison: {
        comparable: true, matches: false, firstMismatchIndex: 1,
        storedMismatchUtf16Start: 100 as number | null,
        freshMismatchUtf16Start: 105 as number | null,
        storedMismatchRowPart: 'single' as 'single' | 'first' | null,
        freshMismatchRowPart: 'single' as 'single' | 'first' | null,
        storedStartsTruncated: false, storedUtf16Starts: [0, 100],
        freshStartsTruncated: false, freshUtf16Starts: [0, 105],
      },
    };
    let pageRuns = [0, 1].map(page => ({
      charStart: page * 100,
      text: 'x'.repeat(100),
      streamStartUtf16: page * 100,
      streamEndUtf16: (page + 1) * 100,
    }));
    const doc = {
      getPageTextLayout: (page: number) => JSON.stringify({
        runs: [{
          secIdx: 0, paraIdx: 4, x: 0, y: 0, ...pageRuns[page],
        }],
      }),
      getLineBreakProvenance: () => JSON.stringify(provenance),
    };
    initRhwpDev({ pageCount: 2, doc } as never);
    const dev = (globalThis as any).window.rhwpDev;
    assert.deepEqual(dev.lineBreakVisible(0).items, []);
    const secondPage = dev.lineBreakVisible(1).items;
    assert.equal(secondPage.length, 1);
    assert.match(formatFirstLineBreakError(2, secondPage) ?? '', /^line-break: \[page=2 /);

    provenance.textUtf16Length = 100;
    for (const [stored, fresh] of [[[0], [0, 100]], [[0, 100], [0]]]) {
      provenance.comparison.storedUtf16Starts = stored;
      provenance.comparison.freshUtf16Starts = fresh;
      provenance.comparison.storedMismatchUtf16Start = stored[1] ?? null;
      provenance.comparison.freshMismatchUtf16Start = fresh[1] ?? null;
      provenance.comparison.storedMismatchRowPart = stored[1] == null ? null : 'single';
      provenance.comparison.freshMismatchRowPart = fresh[1] == null ? null : 'single';
      assert.equal(dev.lineBreakVisible(0).items.length, 1, 'paragraph EOF belongs to its last page');
    }

    for (const [runs, length, stored, fresh] of [
      [
        [
          { charStart: 0, text: '😀', streamStartUtf16: 0, streamEndUtf16: 2 },
          { charStart: 1, text: 'A', streamStartUtf16: 2, streamEndUtf16: 3 },
        ],
        3, [0, 2], [0, 3],
      ],
      [
        [
          { charStart: 0, text: 'A', streamStartUtf16: 0, streamEndUtf16: 8 },
          { charStart: 1, text: 'B', streamStartUtf16: 8, streamEndUtf16: 9 },
        ],
        9, [0, 8], [0, 9],
      ],
    ] as const) {
      pageRuns = [...runs];
      provenance.textUtf16Length = length;
      provenance.comparison.storedUtf16Starts = [...stored];
      provenance.comparison.freshUtf16Starts = [...fresh];
      provenance.comparison.storedMismatchUtf16Start = stored[1];
      provenance.comparison.freshMismatchUtf16Start = fresh[1];
      provenance.comparison.storedMismatchRowPart = 'single';
      provenance.comparison.freshMismatchRowPart = 'single';
      assert.deepEqual(dev.lineBreakVisible(0).items, []);
      assert.equal(dev.lineBreakVisible(1).items.length, 1);
    }

    pageRuns = [
      { charStart: 0, text: 'A', streamStartUtf16: 0, streamEndUtf16: 9 },
      { charStart: 1, text: 'B', streamStartUtf16: 9, streamEndUtf16: 10 },
    ];
    provenance.textUtf16Length = 10;
    provenance.comparison.firstMismatchIndex = 0;
    provenance.comparison.storedUtf16Starts = [0];
    provenance.comparison.freshUtf16Starts = [0];
    provenance.comparison.storedMismatchUtf16Start = 0;
    provenance.comparison.freshMismatchUtf16Start = 0;
    provenance.comparison.storedMismatchRowPart = 'single';
    provenance.comparison.freshMismatchRowPart = 'first';
    assert.equal(dev.lineBreakVisible(0).items.length, 1, 'page 1 owns its leading control prefix');
    assert.deepEqual(dev.lineBreakVisible(1).items, []);

    provenance.comparison.firstMismatchIndex = 1;
    provenance.comparison.storedUtf16Starts = [0, 9];
    provenance.comparison.freshUtf16Starts = [0, 10];
    provenance.comparison.storedMismatchUtf16Start = 9;
    provenance.comparison.freshMismatchUtf16Start = 10;
    provenance.comparison.storedMismatchRowPart = 'single';
    provenance.comparison.freshMismatchRowPart = 'single';
    assert.deepEqual(dev.lineBreakVisible(0).items, []);
    assert.equal(dev.lineBreakVisible(1).items.length, 1, 'lifted HWPX boundary owns page 2');

    provenance.textUtf16Length = 200;
    provenance.comparison.firstMismatchIndex = 128;
    provenance.comparison.storedStartsTruncated = true;
    provenance.comparison.freshStartsTruncated = true;
    provenance.comparison.storedUtf16Starts = Array(128).fill(0);
    provenance.comparison.freshUtf16Starts = Array(128).fill(0);
    pageRuns = [0, 1].map(page => ({
      charStart: page * 100,
      text: 'x'.repeat(100),
      streamStartUtf16: page * 100,
      streamEndUtf16: (page + 1) * 100,
    }));
    for (const [stored, fresh, storedPart, freshPart] of [
      [150, 155, 'single', 'single'],
      [null, 150, null, 'single'],
      [150, null, 'single', null],
      [150, 150, 'single', 'first'],
    ] as const) {
      provenance.comparison.storedMismatchUtf16Start = stored;
      provenance.comparison.freshMismatchUtf16Start = fresh;
      provenance.comparison.storedMismatchRowPart = storedPart;
      provenance.comparison.freshMismatchRowPart = freshPart;
      assert.deepEqual(dev.lineBreakVisible(0).items, []);
      const items = dev.lineBreakVisible(1).items;
      assert.equal(items.length, 1);
      const line = formatFirstLineBreakError(2, items) ?? '';
      assert.ok(line.length < 4_096);
      if (stored === fresh) assert.match(line, /expected=150:single actual=150:first/);
    }
  } finally {
    console.log = previousLog;
    if (previousWindow === undefined) delete (globalThis as any).window;
    else (globalThis as any).window = previousWindow;
  }
});
