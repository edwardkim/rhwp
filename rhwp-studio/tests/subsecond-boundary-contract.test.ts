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
  assert.ok(main.startsWith("import '@/core/runtime-diagnostics';"));
});

test('handled browser runtime signals do not become uncaught diagnostics', async () => {
  const listeners = new Map<string, (event: any) => void>();
  const previousWindow = (globalThis as any).window;
  (globalThis as any).window = {
    addEventListener(type: string, listener: EventListenerOrEventListenerObject, capture?: boolean) {
      assert.equal(capture, true);
      listeners.set(type, listener as (event: any) => void);
    },
  };
  try {
    await import(`../src/core/runtime-diagnostics.ts?startup=${Date.now()}`);
  } finally {
    if (previousWindow === undefined) delete (globalThis as any).window;
    else (globalThis as any).window = previousWindow;
  }

  const event = (value: Record<string, unknown>) => {
    const calls: string[] = [];
    return {
      ...value,
      calls,
      preventDefault: () => calls.push('preventDefault'),
      stopImmediatePropagation: () => calls.push('stopImmediatePropagation'),
    };
  };
  for (const message of [
    'ResizeObserver loop completed with undelivered notifications.',
    'ResizeObserver loop limit exceeded',
  ]) {
    const resize = event({ message });
    listeners.get('error')!(resize);
    assert.deepEqual(resize.calls, ['stopImmediatePropagation', 'preventDefault']);
  }
  const unrelated = event({ message: 'real application failure' });
  listeners.get('error')!(unrelated);
  assert.deepEqual(unrelated.calls, []);
  assert.deepEqual([...listeners.keys()], ['error']);
});

test('WasmBridge reuses dx eager initialization instead of creating another instance', async () => {
  const { initializeWasmOnce } = await import('../src/core/wasm-init.ts');
  const instance = { memory: 'dx-memory' };
  let fallbackCalls = 0;
  assert.equal(
    await initializeWasmOnce(
      async () => (fallbackCalls += 1, { memory: 'fallback-memory' }),
      { __dx_mainPromise: Promise.resolve(instance) },
    ),
    instance,
  );
  assert.equal(fallbackCalls, 0);

  let resolveFallback!: (value: { memory: string }) => void;
  const fallbackScope = {};
  const fallback = () => {
    fallbackCalls += 1;
    return new Promise<{ memory: string }>(resolve => { resolveFallback = resolve; });
  };
  const first = initializeWasmOnce(fallback, fallbackScope);
  const second = initializeWasmOnce(fallback, fallbackScope);
  assert.equal(first, second);
  assert.equal(fallbackCalls, 1);
  resolveFallback({ memory: 'fallback-memory' });
  assert.deepEqual(await Promise.all([first, second]), [
    { memory: 'fallback-memory' },
    { memory: 'fallback-memory' },
  ]);
  assert.equal(initializeWasmOnce(fallback, fallbackScope), first);

  const retryScope = {};
  const failure = new Error('initialization failed');
  const rejected = () => (fallbackCalls += 1, Promise.reject(failure));
  const rejectedFirst = initializeWasmOnce(rejected, retryScope);
  const rejectedSecond = initializeWasmOnce(rejected, retryScope);
  assert.equal(rejectedFirst, rejectedSecond);
  assert.deepEqual(
    (await Promise.allSettled([rejectedFirst, rejectedSecond])).map(result => result.status),
    ['rejected', 'rejected'],
  );
  assert.deepEqual(
    await initializeWasmOnce(async () => (fallbackCalls += 1, { memory: 'retry-memory' }), retryScope),
    { memory: 'retry-memory' },
  );
  assert.equal(fallbackCalls, 3);
});

test('일반 Studio 클래스는 개발용 소켓과 감시자를 소유하지 않는다', () => {
  const bridge = source('src/core/wasm-bridge.ts');
  const canvasView = source('src/view/canvas-view.ts');

  assert.doesNotMatch(bridge, /subsecond-runtime|startRenderCodeReload|getRenderCodeReload|disconnectSubsecondDevtools/);
  assert.doesNotMatch(canvasView, /subsecond-runtime|startRenderCodeReloadWatch|renderCodeReloadWatcher|render-code-reloaded/);
  assert.match(source('src/main.ts'), /refreshPages\(\{ throwOnPageInfoError: true \}\)/);
});

test('wiring guard: layout tracing never owns a browser await', () => {
  const view = source('src/view/canvas-view.ts');
  const main = source('src/main.ts');
  const load = view.slice(
    view.indexOf('async loadDocument('),
    view.indexOf('/**\n   * PgUp/PgDn', view.indexOf('async loadDocument(')),
  );
  const resolved = load.indexOf('await this.rendererSession.resolve(this.wasm)');
  const traced = load.indexOf('trace(() => this.finishDocumentLoad(epoch, selection))');
  assert.ok(resolved >= 0 && traced > resolved);
  assert.doesNotMatch(load, /trace\(async/);
  assert.match(load, /finishDocumentLoad[\s\S]*this\.wasm\.getPageInfo[\s\S]*this\.updateVisiblePages/);
  assert.match(
    main,
    /run: \(operation\) => \{[\s\S]*activateSubsecondTrace[\s\S]*return operation\(\)[\s\S]*deactivateSubsecondTrace/,
  );
  assert.match(
    main,
    /capturePage: async[\s\S]*beginLayoutTraceSession\(\)[\s\S]*capturePageForDiagnostics\([\s\S]*session\.run[\s\S]*retain = true[\s\S]*finally \{[\s\S]*session\.end\(retain\)/,
  );
  assert.match(
    main,
    /const observeRenderCodeRevision = \(current = currentRenderCodeRevision\(\)\)[\s\S]*revision === null\) revision = current[\s\S]*revision !== current\) stable = false/,
  );
  assert.match(
    main,
    /new LayoutTraceMailbox\(currentRenderCodeRevision\)[\s\S]*push\(initialLayoutTrace\.serialized, initialLayoutTrace\.renderCodeRevision\)/,
  );
  assert.doesNotMatch(main, /withLayoutTraceAsync|SubsecondTraceQueue/);

  const capture = view.slice(
    view.indexOf('async capturePageForDiagnostics('),
    view.indexOf('/** DEV 정답지 레이어', view.indexOf('async capturePageForDiagnostics(')),
  );
  const tracedRender = capture.indexOf('return trace(() => diagnosticPageRenderer.renderPage');
  const first = capture.indexOf('const first = render(canvas)');
  const settled = capture.indexOf('await diagnosticPageRenderer.waitForReRender');
  const final = capture.indexOf('const final = render(renderedCanvas)');
  const composite = capture.indexOf("const composite = document.createElement('canvas')");
  assert.ok(tracedRender >= 0 && tracedRender < first && first < settled && settled < final && final < composite);
  assert.match(capture, /const render[\s\S]*fallbackFromResourceFailure[\s\S]*const first = render/);
});

test('a final-only diagnostic render failure takes the same Canvas2D fallback', () => {
  const view = source('src/view/canvas-view.ts');
  const start = view.indexOf('const render = (target: HTMLCanvasElement)');
  const end = view.indexOf('\n        const first = render(canvas);', start);
  const expression = view.slice(start, end)
    .replace('const render = ', '')
    .replace('(target: HTMLCanvasElement): PageRenderResult | null =>', '(target) =>')
    .replace(/;\s*$/, '');
  let renders = 0;
  const commits: unknown[] = [];
  const owner = {
    diagnosticRenderBackend: 'canvaskit',
    activeRendererDecisionKey: 'decision',
    rendererSession: {
      isAutoRequest: () => true,
      fallbackFromResourceFailure: (error: unknown, key: string) => ({ error, key }),
    },
    commitCanvasKitFallback: (fallback: unknown) => commits.push(fallback),
  };
  const render = Function(
    'trace',
    'diagnosticPageRenderer',
    'pageIdx',
    'renderScale',
    `return (${expression});`,
  ).call(
    owner,
    (run: () => unknown) => run(),
    { renderPage: () => (++renders === 1 ? { renderedCanvas: 'first' } : (() => { throw new Error('final'); })()) },
    0,
    1,
  ) as (target: unknown) => unknown;

  assert.deepEqual(render('canvas'), { renderedCanvas: 'first' });
  assert.equal(render('first'), null);
  assert.equal(commits.length, 1);
});

test('a first-document trace is discarded when a patch lands before page layout', async () => {
  const main = source('src/main.ts');
  const start = main.indexOf('function beginLayoutTraceSession()');
  const end = main.indexOf('\n\nfunction withLayoutTrace', start);
  const declaration = main.slice(start, end)
    .replace(/function beginLayoutTraceSession\(\): \{[\s\S]*?\n\} \{/, 'function beginLayoutTraceSession() {')
    .replace('let revision: string | null = null;', 'let revision = null;');
  let revision: string | null = null;
  let retained: boolean | null = null;
  const begin = Function(
    'subsecondTraceExports',
    'currentRenderCodeRevision',
    `${declaration}; return beginLayoutTraceSession;`,
  )(
    () => ({
      beginSubsecondTrace: () => 7,
      activateSubsecondTrace: () => {},
      deactivateSubsecondTrace: () => {},
      endSubsecondTrace: (_token: number, retain: boolean) => {
        retained = retain;
        return retain ? '[{"function":"old"}]' : '[]';
      },
    }),
    () => revision,
  ) as () => {
    run<T>(operation: () => T): T;
    observeRenderCodeRevision(revision: string | null): void;
    end(retain: boolean): string;
    renderCodeRevision(): string | null;
  };

  assert.match(
    main,
    /const \[docInfo, openedRenderCodeRevision\] = await loadDocumentForOpen\(data, fileName\);\n\s*traceSession\?\.observeRenderCodeRevision\(openedRenderCodeRevision\);/,
  );
  const session = begin();
  revision = 'R1';
  session.observeRenderCodeRevision(revision);
  revision = 'R2';
  session.run(() => {});
  assert.equal(session.renderCodeRevision(), null);
  assert.equal(session.end(true), '[]');
  assert.equal(retained, false);

  const unknown = begin();
  unknown.observeRenderCodeRevision(null);
  revision = 'R2';
  unknown.run(() => {});
  assert.equal(unknown.renderCodeRevision(), null);
  assert.equal(unknown.end(true), '[]');
  assert.equal(retained, false);

  const passwordLoader = main.slice(
    main.indexOf('async function loadEncryptedDocumentFromPrompt'),
    main.indexOf('\n\nasync function loadDocumentForOpen'),
  )
    .replace(/async function loadEncryptedDocumentFromPrompt\([\s\S]*?\): Promise<readonly \[DocumentInfo, string \| null\]> \{/, 'async function loadEncryptedDocumentFromPrompt(data, fileName) {')
    .replace('let retryMessage: string | undefined;', 'let retryMessage;')
    .replace('let documentInfo: DocumentInfo;', 'let documentInfo;')
    .replaceAll(' as const', '');
  let derivedUnder: string | null = null;
  revision = 'R1';
  const open = Function(
    'showHwpPasswordDialog', 'DocumentOpenCancelledError', 'wasm',
    'isPasswordRejectedError', 'passwordOpenFailure', 'currentRenderCodeRevision',
    `${passwordLoader}; return loadEncryptedDocumentFromPrompt;`,
  )(
    async () => 'secret', class extends Error {},
    { loadEncryptedDocument: () => { derivedUnder = revision; return { pageCount: 1 }; } },
    () => false, (error: unknown) => error, () => revision,
  ) as (data: Uint8Array, fileName: string) => Promise<readonly [unknown, string | null]>;
  const opening = open(new Uint8Array(), 'secret.hwp');
  queueMicrotask(() => { revision = 'R2'; });
  const [, owner] = await opening;
  assert.equal(derivedUnder, 'R1');
  assert.equal(owner, 'R1');
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
