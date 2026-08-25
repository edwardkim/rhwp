import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { lookupPdfTwin, sha256Hex } from '../src/dev/pdf-twin-client.ts';
import {
  DOCUMENT_ERROR_CAPABILITY_HEADER,
  DOCUMENT_ERROR_LOG_PATH,
  PDF_TWIN_LOOKUP_PATH,
} from '../src/dev/pdf-twin-contract.ts';
import {
  computeReferencePixelDiff,
  detectHorizontalRuleBands,
} from '../src/dev/pdf-reference-diff.ts';
import {
  buildFidelityScanReport,
  compareDiagnosticBands,
  fingerprintPagePixels,
  firstDocumentDivergence,
  isFidelityDocumentCurrent,
  isComparableFidelityPredecessor,
  isFidelityScanCurrent,
  isStructuralPageDivergence,
  queryFidelityPage,
  queryFidelityPages,
  type FidelityPageObservation,
} from '../src/dev/fidelity-scan.ts';
import {
  formatDocumentError,
  findFirstLineBreakError,
  formatFirstLineBreakError,
  sendDocumentErrorLine,
} from '../src/dev/document-error-log.ts';
import { DiagnosticPauseGate, yieldToInteractiveWork } from '../src/dev/fidelity-yield.ts';
import { fetchReferenceWithRetry } from '../src/dev/pdf-reference-fetch.ts';
import { boundedPageRasterSize } from '../src/dev/page-raster-budget.ts';
import {
  nextDiagnosticRenderGeneration,
  waitForFontsOrAbort,
} from '../src/view/page-reference-layer.ts';

test('fidelity scan yield keeps a timer fallback when idle callbacks are suspended', async () => {
  const previousWindow = (globalThis as { window?: unknown }).window;
  const previousDocument = (globalThis as { document?: unknown }).document;
  const timers: Array<() => void> = [];
  (globalThis as { window?: unknown }).window = {
    requestIdleCallback: () => 1,
    cancelIdleCallback: () => {},
    setTimeout: (callback: () => void) => (timers.push(callback), timers.length),
    clearTimeout: () => {},
  };
  try {
    (globalThis as { document?: unknown }).document = { visibilityState: 'hidden' };
    await yieldToInteractiveWork(new AbortController().signal);
    assert.equal(timers.length, 0);
    (globalThis as { document?: unknown }).document = { visibilityState: 'visible' };
    const yielded = yieldToInteractiveWork(new AbortController().signal);
    assert.equal(timers.length, 1);
    timers[0]();
    await yielded;
  } finally {
    (globalThis as { window?: unknown }).window = previousWindow;
    (globalThis as { document?: unknown }).document = previousDocument;
  }
});

test('diagnostic pause blocks the next page capture and abort releases it', async () => {
  const gate = new DiagnosticPauseGate();
  assert.equal(Object.getOwnPropertyDescriptor(DiagnosticPauseGate.prototype, 'paused')?.set, undefined);
  const firstSignal = new AbortController();
  gate.set(true);
  let resumed = false;
  const waiting = gate.wait(firstSignal.signal).then(() => { resumed = true; });
  await Promise.resolve();
  assert.equal(resumed, false);
  gate.set(false);
  await waiting;
  assert.equal(resumed, true);

  const aborted = new AbortController();
  gate.set(true);
  const abortWaiting = gate.wait(aborted.signal);
  aborted.abort();
  await abortWaiting;
  assert.equal(gate.paused, true);
  gate.set(false);
});

test('font readiness removes its losing abort listener', async () => {
  let added = 0;
  let removed = 0;
  const signal = {
    aborted: false,
    addEventListener: () => { added += 1; },
    removeEventListener: () => { removed += 1; },
  } as unknown as AbortSignal;
  await waitForFontsOrAbort(Promise.resolve(), signal);
  assert.equal(added, 1);
  assert.equal(removed, 1);
});

test('reference bitmap ownership covers an abort that wins during decode', () => {
  const source = readFileSync(
    new URL('../src/dev/pdf-reference-overlay.ts', import.meta.url),
    'utf8',
  );
  const decodedAt = source.indexOf('const image = await createImageBitmap');
  const ownerAt = source.indexOf('try {', decodedAt);
  const abortAt = source.indexOf("if (signal.aborted) throw new DOMException('reference image load aborted'", ownerAt);
  const closeAt = source.indexOf('image.close();', abortAt);
  assert.ok(decodedAt >= 0 && decodedAt < ownerAt && ownerAt < abortAt && abortAt < closeAt);
});

test('HWP diagnostic capture rejects pathological page geometry before allocation', () => {
  assert.deepEqual(boundedPageRasterSize({ width: 794, height: 1123 }, 256), {
    width: 256,
    height: 362,
  });
  assert.throws(
    () => boundedPageRasterSize({ width: 1, height: 1_000_000_000 }, 256, 'diagnostic page'),
    /diagnostic page raster dimensions exceed the harness budget/,
  );
  assert.throws(
    () => boundedPageRasterSize({ width: Number.NaN, height: 1123 }, 256, 'diagnostic page'),
    /exceed the harness budget/,
  );
});

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
      storedStartsTruncated: false,
      storedUtf16Starts: [0, 37, 77, 114],
      freshStartsTruncated: false,
      freshUtf16Starts: [0, 39, 80, 119],
    },
  }]), 'line-break: [page=3 target=s0/p4/c0.0.0/g2 at=1 expected=0,37,77,114 actual=0,39,80,119]');
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
  const longStarts = Array.from({ length: 1_500 }, (_, index) => index);
  assert.equal(formatFirstLineBreakError(3, [{
    coordinates: { sectionIdx: 0, paragraphIdx: 4 },
    comparison: {
      comparable: true,
      matches: false,
      firstMismatchIndex: 1,
      storedStartsTruncated: false,
      storedUtf16Starts: longStarts,
      freshStartsTruncated: false,
      freshUtf16Starts: longStarts.map(value => value + 1),
    },
  }]), null, 'an undeliverable semantic line must leave paint fallback available');
});

test('a rejected log request is reported instead of silently losing the document error', async () => {
  let body: unknown = null;
  let endpoint: RequestInfo | URL | null = null;
  let capability: string | null = null;
  await assert.rejects(sendDocumentErrorLine(
    'page-count: [page=4 expected=3 actual=4]',
    'test-capability',
    (async (input, init) => {
      endpoint = input;
      body = init?.body;
      capability = new Headers(init?.headers).get(DOCUMENT_ERROR_CAPABILITY_HEADER);
      return { ok: false, status: 500 } as Response;
    }) as typeof fetch,
  ), /rejected \(500\)/);
  assert.equal(endpoint, DOCUMENT_ERROR_LOG_PATH);
  assert.equal(capability, 'test-capability');
  assert.equal(body, 'page-count: [page=4 expected=3 actual=4]');
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
              storedStartsTruncated: false,
              storedUtf16Starts: [0, 12],
              freshStartsTruncated: false,
              freshUtf16Starts: [0, 13],
            },
          }],
        };
  });
  assert.deepEqual(calls, [0, 100]);
  assert.equal(line, 'line-break: [page=3 target=s0/p101 at=1 expected=0,12 actual=0,13]');
});

test('minor glyph noise stays quiet while a shifted line is reported as broken layout', () => {
  const page = {
    pageIndex: 0,
    hwpFingerprint: 'noise',
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
    mismatchRatioDelta: null,
    bounds: { x: 29, y: 27, width: 223, height: 316 },
    changedFromPrevious: null,
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

test('PDF twin overload remains a retryable busy result', async (t) => {
  const previousFetch = globalThis.fetch;
  let endpoint: RequestInfo | URL | null = null;
  globalThis.fetch = async (input) => {
    endpoint = input;
    return new Response(
      JSON.stringify({ status: 'busy', retryAfterMs: 1_000 }),
      { status: 503, headers: { 'Content-Type': 'application/json' } },
    );
  };
  t.after(() => { globalThis.fetch = previousFetch; });
  assert.deepEqual(await lookupPdfTwin('busy.hwp', new Uint8Array([1, 2, 3])), {
    status: 'busy',
    retryAfterMs: 1_000,
  });
  assert.equal(endpoint, PDF_TWIN_LOOKUP_PATH);
});

test('a temporary PDF renderer overload does not abort the whole-document comparison', async () => {
  let attempts = 0;
  const waits: number[] = [];
  const response = await fetchReferenceWithRetry(
    '/reference.png',
    new AbortController().signal,
    {
      send: (async () => {
        attempts += 1;
        return attempts === 1
          ? new Response('{}', { status: 503, headers: { 'Retry-After': '0.1' } })
          : new Response('png', { status: 200 });
      }) as typeof fetch,
      wait: async delay => { waits.push(delay); },
    },
  );
  assert.equal(response.status, 200);
  assert.equal(attempts, 2);
  assert.deepEqual(waits, [100]);
});

test('pixel diff compares identical black and white RGB pixels', () => {
  const hwp = new Uint8ClampedArray([
    255, 255, 255, 255,
    0, 0, 0, 255,
  ]);
  const reference = new Uint8ClampedArray([
    255, 255, 255, 255,
    0, 0, 0, 255,
  ]);
  assert.deepEqual(computeReferencePixelDiff(hwp, reference, 2, 1, 1), {
    comparedPixels: 2,
    mismatchPixels: 0,
    pdfOnlyPixels: 0,
    hwpOnlyPixels: 0,
    colorMismatchPixels: 0,
    mismatchRatio: 0,
    meanAbsoluteError: 0,
    maxAbsoluteError: 0,
    bounds: null,
  });
});

test('pixel diff compares saturated colors in RGB', () => {
  const hwp = new Uint8ClampedArray([255, 230, 100, 255]);
  const reference = new Uint8ClampedArray([255, 230, 100, 255]);
  const metrics = computeReferencePixelDiff(hwp, reference, 1, 1, 1);
  assert.equal(metrics.mismatchPixels, 0);
  assert.equal(metrics.meanAbsoluteError, 0);
});

test('bidirectional mismatch mask marks HWP-only colored content', () => {
  const hwp = new Uint8ClampedArray([40, 180, 90, 255]);
  const reference = new Uint8ClampedArray([255, 255, 255, 255]);
  const mask = new Uint8ClampedArray(4);
  const metrics = computeReferencePixelDiff(hwp, reference, 1, 1, 1, mask);
  assert.equal(metrics.mismatchPixels, 1);
  assert.equal(metrics.hwpOnlyPixels, 1);
  assert.deepEqual(Array.from(mask.slice(0, 3)), [15, 118, 110]);
  assert.ok(mask[3] > 0);
});

test('bidirectional mismatch mask marks PDF-only content in red', () => {
  const hwp = new Uint8ClampedArray([255, 255, 255, 255]);
  const reference = new Uint8ClampedArray([0, 0, 0, 255]);
  const mask = new Uint8ClampedArray(4);
  const metrics = computeReferencePixelDiff(hwp, reference, 1, 1, 1, mask);
  assert.equal(metrics.pdfOnlyPixels, 1);
  assert.deepEqual(Array.from(mask.slice(0, 3)), [220, 53, 69]);
});

test('horizontal rule diagnostics group thick rules and ignore fragmented text-like ink', () => {
  const width = 20;
  const height = 10;
  const pixels = new Uint8ClampedArray(width * height * 4);
  for (let pixel = 0; pixel < width * height; pixel++) {
    pixels[pixel * 4] = 255;
    pixels[pixel * 4 + 1] = 255;
    pixels[pixel * 4 + 2] = 255;
    pixels[pixel * 4 + 3] = 255;
  }
  const ink = (x: number, y: number): void => {
    const offset = (y * width + x) * 4;
    pixels[offset] = pixels[offset + 1] = pixels[offset + 2] = 0;
  };
  for (const y of [2, 3]) for (let x = 2; x < 18; x++) ink(x, y);
  for (let x = 0; x < width; x++) ink(x, 7);
  for (let x = 0; x < width; x += 2) ink(x, 5);

  assert.deepEqual(detectHorizontalRuleBands(pixels, width, height, {
    minSpanRatio: 0.6,
    minCoverageRatio: 0.8,
  }), {
    totalBands: 2,
    truncated: false,
    bands: [
      {
        startY: 2,
        endY: 3,
        centerY: 2.5,
        thickness: 2,
        peakInkCoverage: 0.8,
        peakSpanRatio: 0.8,
      },
      {
        startY: 7,
        endY: 7,
        centerY: 7,
        thickness: 1,
        peakInkCoverage: 1,
        peakSpanRatio: 1,
      },
    ],
  });
});

test('horizontal rule diagnostics retain both ends when the report is bounded', () => {
  const width = 8;
  const height = 12;
  const pixels = new Uint8ClampedArray(width * height * 4).fill(255);
  for (const y of [1, 3, 5, 7, 9]) {
    for (let x = 0; x < width; x++) {
      const offset = (y * width + x) * 4;
      pixels[offset] = pixels[offset + 1] = pixels[offset + 2] = 0;
    }
  }

  const diagnostics = detectHorizontalRuleBands(pixels, width, height, { maxBands: 4 });
  assert.equal(diagnostics.totalBands, 5);
  assert.equal(diagnostics.truncated, true);
  assert.deepEqual(diagnostics.bands.map(band => band.centerY), [1, 3, 7, 9]);
});

test('after a patch shifts page 2, the report starts there and cascades through page 3', () => {
  const emptyBands = { totalBands: 0, truncated: false, bands: [] };
  const observation = (
    pageIndex: number,
    fingerprint: string,
    mismatchRatio: number,
  ): FidelityPageObservation => ({
    pageIndex,
    hwpFingerprint: fingerprint,
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
      bounds: mismatchRatio > 0 ? { x: 0, y: 0, width: 1, height: 1 } : null,
    },
    hwpHorizontalRules: emptyBands,
    pdfHorizontalRules: emptyBands,
    hwpInkRows: emptyBands,
    pdfInkRows: emptyBands,
  });
  const identity = {
    documentKey: 'abcdefghijklmnopqrstuvwx',
    documentDigest: 'blake3:test',
    documentGeneration: 4,
    referenceGeneration: 7,
    renderGeneration: 9,
    pdfName: 'same.pdf',
  };
  const baseline = buildFidelityScanReport({
    scanId: 1,
    trigger: 'baseline',
    identity,
    renderRevision: 'r1',
    hwpPageCount: 3,
    pdfPageCount: 3,
    observations: [
      observation(0, 'a', 0),
      observation(1, 'b', 0.1),
      observation(2, 'c', 0.1),
    ],
    previous: null,
    startedAt: 1,
    completedAt: 2,
  });
  const patched = buildFidelityScanReport({
    scanId: 2,
    trigger: 'subsecond-patch',
    identity,
    renderRevision: 'r2',
    hwpPageCount: 3,
    pdfPageCount: 3,
    observations: [
      observation(0, 'a', 0),
      observation(1, 'changed-b', 0.12),
      observation(2, 'changed-c', 0.09),
    ],
    previous: baseline,
    startedAt: 3,
    completedAt: 4,
  });
  assert.equal(patched.firstDivergentPage, 1);
  assert.equal(patched.firstStructuralDivergencePage, 1);
  assert.equal(patched.pixelMismatchPageCount, 2);
  assert.equal(patched.firstRegressionPage, 1);
  assert.deepEqual(patched.downstreamChangedPageRange, { start: 1, end: 2 });
  assert.equal(patched.previousRenderRevision, 'r1');
});

test('a missing final page is reported, while restoring it is not called a regression', () => {
  const emptyBands = { totalBands: 0, truncated: false, bands: [] };
  const observation = (pageIndex: number): FidelityPageObservation => ({
    pageIndex,
    hwpFingerprint: `page-${pageIndex}`,
    hwpSize: { width: 1, height: 1 },
    referenceSize: { width: 1, height: 1 },
    mismatch: {
      comparedPixels: 1,
      mismatchPixels: 0,
      pdfOnlyPixels: 0,
      hwpOnlyPixels: 0,
      colorMismatchPixels: 0,
      mismatchRatio: 0,
      meanAbsoluteError: 0,
      maxAbsoluteError: 0,
      bounds: null,
    },
    hwpHorizontalRules: emptyBands,
    pdfHorizontalRules: emptyBands,
    hwpInkRows: emptyBands,
    pdfInkRows: emptyBands,
  });
  const identity = {
    documentKey: 'abcdefghijklmnopqrstuvwx',
    documentDigest: null,
    documentGeneration: 1,
    referenceGeneration: 1,
    renderGeneration: 1,
    pdfName: 'reference.pdf',
  };
  const short = buildFidelityScanReport({
    scanId: 1,
    trigger: 'baseline',
    identity,
    renderRevision: 'r1',
    hwpPageCount: 3,
    pdfPageCount: 4,
    observations: [0, 1, 2].map(observation),
    previous: null,
    startedAt: 0,
    completedAt: 1,
  });
  assert.equal(short.firstDivergentPage, 3);
  assert.equal(short.firstStructuralDivergencePage, 3);
  assert.equal(short.pixelMismatchPageCount, 0);

  const corrected = buildFidelityScanReport({
    scanId: 2,
    trigger: 'subsecond-patch',
    identity: { ...identity, renderGeneration: 2 },
    renderRevision: 'r2',
    hwpPageCount: 4,
    pdfPageCount: 4,
    observations: [0, 1, 2, 3].map(observation),
    previous: short,
    startedAt: 2,
    completedAt: 3,
  });
  assert.equal(corrected.firstDivergentPage, null);
  assert.equal(corrected.firstStructuralDivergencePage, null);
  assert.equal(corrected.pixelMismatchPageCount, 0);
  assert.equal(corrected.firstRegressionPage, null);
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

test('fidelity fingerprints and diagnostic band deltas are deterministic', () => {
  const pixels = new Uint8ClampedArray([0, 1, 2, 255, 4, 5, 6, 255]);
  assert.equal(fingerprintPagePixels(pixels), fingerprintPagePixels(pixels.slice()));
  assert.notEqual(
    fingerprintPagePixels(pixels),
    fingerprintPagePixels(new Uint8ClampedArray([0, 1, 3, 255, 4, 5, 6, 255])),
  );
  assert.deepEqual(compareDiagnosticBands(
    {
      totalBands: 2,
      truncated: false,
      bands: [
        { startY: 1, endY: 1, centerY: 1, thickness: 1, peakInkCoverage: 1, peakSpanRatio: 1 },
        { startY: 5, endY: 5, centerY: 5, thickness: 1, peakInkCoverage: 1, peakSpanRatio: 1 },
      ],
    },
    {
      totalBands: 1,
      truncated: false,
      bands: [
        { startY: 3, endY: 3, centerY: 3, thickness: 1, peakInkCoverage: 1, peakSpanRatio: 1 },
      ],
    },
  ), {
    hwpCount: 2,
    pdfCount: 1,
    countDelta: 1,
    pairedCount: 1,
    maxCenterDelta: 2,
    meanCenterDelta: 2,
  });
});

test('fidelity detail queries are bounded and reject a stale scan id', () => {
  const emptyBands = { totalBands: 0, truncated: false, bands: [] };
  const identity = {
    documentKey: 'abcdefghijklmnopqrstuvwx',
    documentDigest: null,
    documentGeneration: 1,
    referenceGeneration: 1,
    renderGeneration: 1,
    pdfName: 'reference.pdf',
  };
  const observations: FidelityPageObservation[] = Array.from({ length: 120 }, (_, pageIndex) => ({
    pageIndex,
    hwpFingerprint: String(pageIndex),
    hwpSize: { width: 1, height: 1 },
    referenceSize: { width: 1, height: 1 },
    mismatch: {
      comparedPixels: 1,
      mismatchPixels: pageIndex % 2,
      pdfOnlyPixels: 0,
      hwpOnlyPixels: 0,
      colorMismatchPixels: 0,
      mismatchRatio: pageIndex % 2,
      meanAbsoluteError: 0,
      maxAbsoluteError: 0,
      bounds: null,
    },
    hwpHorizontalRules: emptyBands,
    pdfHorizontalRules: emptyBands,
    hwpInkRows: emptyBands,
    pdfInkRows: emptyBands,
  }));
  const report = buildFidelityScanReport({
    scanId: 12,
    trigger: 'manual',
    identity,
    renderRevision: 'r12',
    hwpPageCount: 120,
    pdfPageCount: 120,
    observations,
    previous: null,
    startedAt: 0,
    completedAt: 1,
  });
  const bounded = queryFidelityPages(report, { scanId: 12, limit: 500 });
  assert.equal(bounded.current, true);
  assert.equal(bounded.items.length, 100);
  const stale = queryFidelityPages(report, { scanId: 11, limit: 5 });
  assert.equal(stale.current, false);
  assert.deepEqual(stale.items, []);
  assert.deepEqual(queryFidelityPage(report, 4, 11), {
    scanId: 12,
    current: false,
    item: null,
  });
});

test('a new render generation invalidates the previous completed error report', () => {
  const report = {
    renderRevision: 'revision-1',
    identity: { renderGeneration: 4 },
  };
  assert.equal(isFidelityScanCurrent(report, {
    renderRevision: 'revision-1',
    renderGeneration: 4,
  }), true);
  assert.equal(isFidelityScanCurrent(report, {
    renderRevision: 'revision-1',
    renderGeneration: 5,
  }), false);
  assert.equal(isFidelityScanCurrent(report, {
    renderRevision: 'revision-2',
    renderGeneration: 4,
  }), false);
  const fallbackGeneration = nextDiagnosticRenderGeneration(4, false, true);
  assert.equal(fallbackGeneration, 5);
  assert.equal(isFidelityScanCurrent(report, {
    renderRevision: 'revision-1',
    renderGeneration: fallbackGeneration,
  }), false, 'same-decision CanvasKit fallback must stale the active scan');
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

test('predecessor policy accepts only the render generation immediately before a patch', () => {
  const previous = { identity: { renderGeneration: 1 } };
  assert.equal(isComparableFidelityPredecessor(previous, 'subsecond-patch', {
    previousRenderGeneration: 2,
    renderGeneration: 3,
  }), false);
  assert.equal(isComparableFidelityPredecessor(previous, 'subsecond-patch', {
    previousRenderGeneration: 1,
    renderGeneration: 2,
  }), true);
});
