import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { lookupPdfTwin, sha256Hex } from '../src/dev/pdf-twin-client.ts';
import { fetchWithBusyRetry } from '../src/dev/pdf-reference-fetch.ts';
import { DOCUMENT_ERROR_LOG_PATH } from '../src/dev/pdf-twin-contract.ts';
import {
  computeReferencePixelDiff,
  detectHorizontalRuleBands,
} from '../src/dev/pdf-reference-diff.ts';
import {
  buildFidelityScanReport,
  firstDocumentDivergence,
  isFidelityDocumentCurrent,
  isFidelityScanCurrent,
  isStructuralPageDivergence,
  type FidelityPageObservation,
} from '../src/dev/fidelity-scan.ts';
import {
  formatDocumentError,
  findFirstLineBreakError,
  formatFirstLineBreakError,
  sendDocumentErrorLine,
} from '../src/dev/document-error-log.ts';
import { nextDiagnosticRenderGeneration } from '../src/view/page-reference-layer.ts';
import { LayoutTraceMailbox, PdfReferenceOverlay } from '../src/dev/pdf-reference-overlay.ts';

test('a line-break failure yields one detailed first-error CLI line', () => {
  assert.equal(formatFirstLineBreakError(3, [{
    coordinates: {
      sectionIdx: 0,
      parentParaIdx: 4,
      cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }],
      groupPath: [2],
    },
    comparison: {
      comparable: true,
      matches: false,
      firstMismatchIndex: 1,
      storedMismatchUtf16Start: 37,
      freshMismatchUtf16Start: 39,
      storedMismatchRowPart: 'single',
      freshMismatchRowPart: 'single',
      storedStartsTruncated: false,
      storedUtf16Starts: [0, 37, 77, 114],
      freshStartsTruncated: false,
      freshUtf16Starts: [0, 39, 80, 119],
    },
  }]), 'line-break: [page=3 target=s0/p4/c0.0.0/g2 at=1 expected=37:single actual=39:single]');
  assert.equal(formatDocumentError('page-count', [
    ['page', 384],
    ['expected', 383],
    ['actual', 390],
  ]), 'page-count: [page=384 expected=383 actual=390]');
  assert.equal(formatDocumentError('paint', [
    ['page', 3],
    ['ratio', 0.058526],
    ['pdfOnly', 210],
    ['rhwpOnly', 317],
    ['colorOnly', 12],
    ['bounds', '14,22,160,81'],
  ]), 'paint: [page=3 ratio=0.058526 pdfOnly=210 rhwpOnly=317 colorOnly=12 bounds=14,22,160,81]');
});

test('incomplete line evidence stays silent instead of fabricating a diagnosis', () => {
  assert.equal(formatFirstLineBreakError(3, [{
    comparison: { matches: false },
  }]), null);
  assert.equal(formatFirstLineBreakError(3, [{
    coordinates: { sectionIdx: 0, paragraphIdx: 4 },
    comparison: {
      comparable: true,
      matches: false,
      firstMismatchIndex: 0,
      storedStartsTruncated: true,
      storedUtf16Starts: [0],
      freshStartsTruncated: false,
      freshUtf16Starts: [1],
    },
  }]), null);
});

test('a late line mismatch stays deliverable without serializing the whole paragraph', () => {
  const line = formatFirstLineBreakError(3, [{
    coordinates: { sectionIdx: 0, paragraphIdx: 4 },
    comparison: {
      comparable: true,
      matches: false,
      firstMismatchIndex: 1_500,
      storedMismatchUtf16Start: 20_000,
      freshMismatchUtf16Start: 20_001,
      storedMismatchRowPart: 'single',
      freshMismatchRowPart: 'single',
      storedStartsTruncated: true,
      storedUtf16Starts: Array.from({ length: 128 }, (_, index) => index),
      freshStartsTruncated: true,
      freshUtf16Starts: Array.from({ length: 128 }, (_, index) => index),
    },
  }]);
  assert.equal(line, 'line-break: [page=3 target=s0/p4 at=1500 expected=20000:single actual=20001:single]');
  assert.ok(line!.length < 4_096);
});

test('a rejected log request is reported instead of silently losing the document error', async () => {
  let body: unknown = null;
  let headers: HeadersInit | undefined;
  let endpoint: RequestInfo | URL | null = null;
  await assert.rejects(sendDocumentErrorLine(
    'page-count: [page=4 expected=3 actual=4]',
    'a'.repeat(43),
    (async (input, init) => {
      endpoint = input;
      body = init?.body;
      headers = init?.headers;
      return { ok: false, status: 500 } as Response;
    }) as typeof fetch,
  ), /rejected \(500\)/);
  assert.equal(endpoint, DOCUMENT_ERROR_LOG_PATH);
  assert.equal(body, 'page-count: [page=4 expected=3 actual=4]');
  assert.equal((headers as Record<string, string>)['x-rhwp-harness-capability'], 'a'.repeat(43));
});

test('bounded line traversal finds the first mismatch in a later result batch', () => {
  const calls: number[] = [];
  const matching = Array.from({ length: 100 }, () => ({ comparison: { matches: true } }));
  const line = findFirstLineBreakError(3, (start) => {
    calls.push(start);
    return start === 0
      ? { total: 101, nextOffset: 100, items: matching }
      : {
          total: 101,
          nextOffset: null,
          items: [{
            coordinates: { sectionIdx: 0, paragraphIdx: 101 },
            comparison: {
              comparable: true,
              matches: false,
              firstMismatchIndex: 1,
              storedMismatchUtf16Start: 12,
              freshMismatchUtf16Start: 13,
              storedMismatchRowPart: 'single',
              freshMismatchRowPart: 'single',
              storedStartsTruncated: false,
              storedUtf16Starts: [0, 12],
              freshStartsTruncated: false,
              freshUtf16Starts: [0, 13],
            },
          }],
        };
  });
  assert.deepEqual(calls, [0, 100]);
  assert.equal(line, 'line-break: [page=3 target=s0/p101 at=1 expected=12:single actual=13:single]');
});

test('minor glyph noise stays quiet while a shifted line is reported as broken layout', () => {
  const page = {
    pageIndex: 0,
    hwpSize: { width: 256, height: 346 },
    referenceSize: { width: 256, height: 346 },
    referenceHeightDelta: 0,
    mismatchPixels: 2085,
    pdfOnlyPixels: 1000,
    hwpOnlyPixels: 1000,
    colorMismatchPixels: 85,
    comparedPixels: 88832,
    mismatchRatio: 0.023471,
    meanAbsoluteError: 4,
    maxAbsoluteError: 255,
    bounds: { x: 29, y: 27, width: 223, height: 316 },
    horizontalRuleDelta: {
      hwpCount: 0,
      pdfCount: 0,
      countDelta: 0,
      pairedCount: 0,
      maxCenterDelta: null,
      meanCenterDelta: null,
    },
    inkRowDelta: {
      hwpCount: 20,
      pdfCount: 20,
      countDelta: 0,
      pairedCount: 20,
      maxCenterDelta: 0.5,
      meanCenterDelta: 0.2,
    },
  } as const;

  assert.equal(isStructuralPageDivergence(page), false);
  assert.equal(isStructuralPageDivergence({
    ...page,
    horizontalRuleDelta: { ...page.horizontalRuleDelta, countDelta: -1 },
  }), true);
  assert.equal(isStructuralPageDivergence({
    ...page,
    mismatchRatio: 0.058526,
    inkRowDelta: {
      ...page.inkRowDelta,
      countDelta: 1,
      maxCenterDelta: 3.5,
    },
  }), true);
});

test('browser document identity uses SHA-256 bytes', async () => {
  const bytes = new TextEncoder().encode('same selected document');
  assert.equal(
    await sha256Hex(bytes),
    createHash('sha256').update(bytes).digest('hex'),
  );
});

test('busy PDF lookup retries finitely and reference retry obeys abort', async () => {
  let attempts = 0;
  const found = {
    status: 'found' as const,
    pdfName: 'reference.pdf',
    pdfPageUrl: '/__rhwp_harness/pdf-page/token',
    pdfPageWidth: 2048,
    pdfPageCount: 1,
    relativeDirectory: '.',
    errorLogCapability: 'a'.repeat(43),
  };
  const result = await lookupPdfTwin('doc.hwp', new Uint8Array([1]), (async () => {
    attempts += 1;
    return attempts === 1
      ? new Response('', { status: 503, headers: { 'Retry-After': '0' } })
      : Response.json(found);
  }) as typeof fetch);
  assert.deepEqual(result, found);
  assert.equal(attempts, 2);

  const controller = new AbortController();
  let abortedAttempts = 0;
  await assert.rejects(fetchWithBusyRetry('/page', { signal: controller.signal }, (async () => {
    abortedAttempts += 1;
    controller.abort();
    return new Response('', { status: 503, headers: { 'Retry-After': '1' } });
  }) as typeof fetch), { name: 'AbortError' });
  assert.equal(abortedAttempts, 1);
});

test('pixel and line evidence distinguish the direction and location of drift', () => {
  const mask = new Uint8ClampedArray(4);
  const metrics = computeReferencePixelDiff(
    new Uint8ClampedArray([255, 255, 255, 255]),
    new Uint8ClampedArray([0, 0, 0, 255]),
    1,
    1,
    1,
    mask,
  );
  assert.equal(metrics.pdfOnlyPixels, 1);
  assert.deepEqual(Array.from(mask.slice(0, 3)), [220, 53, 69]);

  const pixels = new Uint8ClampedArray(4 * 3 * 4).fill(255);
  for (let x = 0; x < 4; x++) pixels.set([0, 0, 0, 255], (4 + x) * 4);
  assert.equal(detectHorizontalRuleBands(pixels, 4, 3).bands[0]?.centerY, 1);
});

const emptyBands = { totalBands: 0, truncated: false, bands: [] };
const observation = (pageIndex: number, mismatchRatio = 0): FidelityPageObservation => ({
  pageIndex,
  hwpSize: { width: 10, height: 10 },
  referenceSize: { width: 10, height: 10 },
  mismatch: {
    comparedPixels: 100,
    mismatchPixels: Math.round(mismatchRatio * 100),
    pdfOnlyPixels: 0,
    hwpOnlyPixels: 0,
    colorMismatchPixels: 0,
    mismatchRatio,
    meanAbsoluteError: 0,
    maxAbsoluteError: 0,
    bounds: mismatchRatio ? { x: 0, y: 0, width: 1, height: 1 } : null,
  },
  hwpHorizontalRules: emptyBands,
  pdfHorizontalRules: emptyBands,
  hwpInkRows: emptyBands,
  pdfInkRows: emptyBands,
});

test('a superseded trace batch cannot cross render revisions and the mailbox stays bounded', () => {
  let revision = 'R1';
  const mailbox = new LayoutTraceMailbox(() => revision);
  mailbox.push('r1-failed');
  revision = 'R2';
  for (let index = 0; index < 6; index++) mailbox.push(`r2-${index}`);
  assert.deepEqual(mailbox.takeCurrent(), ['r2-2', 'r2-3', 'r2-4', 'r2-5']);
  assert.deepEqual(mailbox.takeCurrent(), []);
});

test('a render change during reference loading cannot publish stale evidence', async () => {
  const previousWindow = (globalThis as any).window;
  const browser: Record<string, unknown> = {};
  (globalThis as any).window = browser;
  let renderGeneration = 1;
  let referenceReady!: (value: unknown) => void;
  let loadingStarted!: () => void;
  let errorDelivered!: () => void;
  const reference = new Promise(resolve => { referenceReady = resolve; });
  const started = new Promise<void>(resolve => { loadingStarted = resolve; });
  const deliveredCurrent = new Promise<void>(resolve => { errorDelivered = resolve; });
  let captures = 0;
  const overlay = new PdfReferenceOverlay('/__rhwp_harness/pdf-page/token', 1, 1, 'ref.pdf', {
    errorLogCapability: 'a'.repeat(43),
    documentDigest: 'digest',
    documentGeneration: 1,
    referenceGeneration: 1,
    getDocumentDigest: () => 'digest',
    getDocumentGeneration: () => 1,
    getHwpPageCount: () => 1,
    capturePage: async () => {
      const functionName = captures++ === 0 ? 'stale_page_only' : 'current_page_only';
      return {
        width: 1,
        height: 1,
        pixels: new Uint8ClampedArray(4),
        layoutTrace: JSON.stringify([{
          function: functionName,
          args: {},
          durationMs: 1,
          depth: 0,
        }]),
      };
    },
    getRenderGeneration: () => renderGeneration,
  });
  const internal = overlay as any;
  const delivered: string[] = [];
  let loads = 0;
  internal.loadReferencePixels = () => {
    loads += 1;
    if (loads === 1) {
      loadingStarted();
      return reference;
    }
    return Promise.resolve({
      width: 1, height: 1, surfaceHeight: 1, pixels: new Uint8ClampedArray(4),
    });
  };
  internal.observePage = () => observation(0, 0.1);
  internal.deliverDocumentError = (line: string) => {
    delivered.push(line);
    errorDelivered();
  };
  try {
    const scan = (browser.__rhwpFidelityHarness as any).scan();
    await started;
    renderGeneration = 2;
    referenceReady({ width: 1, height: 1, surfaceHeight: 1, pixels: new Uint8ClampedArray(4) });
    assert.equal(await scan, null);
    await deliveredCurrent;
    assert.equal(loads, 2, 'the stale scan is replaced with a current-generation scan');
    assert.equal(delivered.length, 1);
    assert.match(delivered[0], /current_page_only/);
    assert.doesNotMatch(delivered[0], /stale_page_only/);
  } finally {
    await overlay.destroy();
    if (previousWindow === undefined) delete (globalThis as any).window;
    else (globalThis as any).window = previousWindow;
  }
});

test('the page layout trace accompanies both semantic and paint failures', async () => {
  const previousWindow = (globalThis as any).window;
  let hasLineBreak = true;
  const browser: any = {
    setTimeout: globalThis.setTimeout,
    clearTimeout: globalThis.clearTimeout,
    rhwpDev: {
      lineBreakVisible: () => ({
        total: 1,
        nextOffset: null,
        items: hasLineBreak ? [{
          coordinates: { sectionIdx: 0, paragraphIdx: 4 },
          comparison: {
            comparable: true,
            matches: false,
            firstMismatchIndex: 1,
            storedMismatchUtf16Start: 12,
            freshMismatchUtf16Start: 13,
            storedMismatchRowPart: 'single',
            freshMismatchRowPart: 'single',
            storedStartsTruncated: false,
            storedUtf16Starts: [0, 12],
            freshStartsTruncated: false,
            freshUtf16Starts: [0, 13],
          },
        }] : [],
      }),
    },
  };
  (globalThis as any).window = browser;
  const traceLifecycle: string[] = [];
  const semanticTrace = JSON.stringify([{
    function: 'layout_frame_commit_row',
    args: { top: 12, line_height: 20, result_top: 32 },
    durationMs: 2,
    depth: 0,
  }]);
  const traces = ['[]', semanticTrace];
  const overlay = new PdfReferenceOverlay('/__rhwp_harness/pdf-page/token', 1, 1, 'ref.pdf', {
    errorLogCapability: 'a'.repeat(43),
    documentDigest: 'digest', documentGeneration: 1, referenceGeneration: 1,
    getDocumentDigest: () => 'digest', getDocumentGeneration: () => 1,
    getHwpPageCount: () => 1,
    traceLayout: (run) => {
      traceLifecycle.push('begin:7');
      try {
        return run();
      } finally {
        traceLifecycle.push('end:7:true');
      }
    },
    takeLayoutTrace: () => [traces.shift() ?? '[]'],
    capturePage: async () => ({ width: 1, height: 1, pixels: new Uint8ClampedArray(4) }),
    getRenderGeneration: () => 1,
  });
  const internal = overlay as any;
  const delivered: string[] = [];
  internal.loadReferencePixels = async () => ({
    width: 1, height: 1, surfaceHeight: 1, pixels: new Uint8ClampedArray(4),
  });
  let mismatchRatio = 0.01;
  internal.observePage = () => observation(0, mismatchRatio);
  internal.deliverDocumentError = (line: string) => delivered.push(line);
  try {
    await browser.__rhwpFidelityHarness.scan();
    assert.deepEqual(delivered, [
      'line-break: [page=1 target=s0/p4 at=1 expected=12:single actual=13:single] ' +
      'trace=[{"function":"layout_frame_commit_row","args":{"top":12,"line_height":20,' +
      '"result_top":32},"durationMs":2,"depth":0}]',
    ]);
    assert.deepEqual(traceLifecycle, ['begin:7', 'end:7:true']);
    hasLineBreak = false;
    mismatchRatio = 0.1;
    traces.push('[]', semanticTrace);
    await browser.__rhwpFidelityHarness.scan();
    assert.match(delivered[1], /^paint: \[page=1 /);
    assert.match(delivered[1], /"function":"layout_frame_commit_row"/);
  } finally {
    await overlay.destroy();
    if (previousWindow === undefined) delete (globalThis as any).window;
    else (globalThis as any).window = previousWindow;
  }
});

const identity = {
  documentKey: 'abcdefghijklmnopqrstuvwx',
  documentDigest: null,
  documentGeneration: 1,
  referenceGeneration: 1,
  renderGeneration: 1,
  pdfName: 'reference.pdf',
};

test('the report names the first broken page and a missing final page', () => {
  const paint = buildFidelityScanReport({
    scanId: 1,
    trigger: 'baseline',
    identity,
    hwpPageCount: 3,
    pdfPageCount: 3,
    observations: [observation(0), observation(1, 0.1), observation(2, 0.1)],
    startedAt: 0,
    completedAt: 1,
  });
  assert.equal(paint.firstStructuralDivergencePage, 1);

  const missing = buildFidelityScanReport({
    ...paint,
    scanId: 2,
    trigger: 'subsecond-patch',
    hwpPageCount: 3,
    pdfPageCount: 4,
    observations: [0, 1, 2].map(page => observation(page)),
  });
  assert.equal(missing.firstDivergentPage, 3);
});

test('the earliest broken page wins even when a later error has another kind', () => {
  assert.deepEqual(firstDocumentDivergence({
    firstStructuralDivergencePage: 0,
    hwpPageCount: 390,
    pdfPageCount: 383,
    pageCountDelta: 7,
  }), { kind: 'structural', pageIndex: 0 });
  assert.deepEqual(firstDocumentDivergence({
    firstStructuralDivergencePage: 400,
    hwpPageCount: 390,
    pdfPageCount: 383,
    pageCountDelta: 7,
  }), { kind: 'page-count', pageIndex: 383 });
});

test('a new render generation invalidates the previous completed error report', () => {
  const report = { identity: { renderGeneration: 4 } };
  assert.equal(isFidelityScanCurrent(report, 4), true);
  assert.equal(isFidelityScanCurrent(report, 5), false);
  const fallbackGeneration = nextDiagnosticRenderGeneration(4, false, true);
  assert.equal(fallbackGeneration, 5);
  assert.equal(isFidelityScanCurrent(report, fallbackGeneration), false);
});

test('a new document identity invalidates the previous error even when page counts match', () => {
  const expected = { documentDigest: 'old', documentGeneration: 4 };
  assert.equal(isFidelityDocumentCurrent(expected, {
    documentDigest: 'old',
    documentGeneration: 4,
  }), true);
  assert.equal(isFidelityDocumentCurrent(expected, {
    documentDigest: 'new',
    documentGeneration: 5,
  }), false);
});
