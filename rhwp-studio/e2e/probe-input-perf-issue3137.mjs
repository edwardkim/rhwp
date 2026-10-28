/**
 * Issue #3137 giant-cell input performance matrix.
 *
 * This is a diagnostic benchmark, not a wall-clock CI gate. It isolates the
 * common stable-input path from the #2214 boundary/correctness suite and writes
 * reproducible JSON/TSV evidence for:
 *
 *   HWP/HWPX × English/digit/IME × 0/80/150/250ms requested cadence
 *
 * The semantic contract (115 pages, exact model/cursor result, no synchronous
 * pagination flush) is asserted. The 16.7ms frame budget is reported by
 * default and can be enforced after an optimization with
 * --enforce-frame-budget.
 *
 * Build and run from the repository root:
 *
 *   wasm-pack build --target web --out-dir pkg
 *   cd rhwp-studio
 *   npm run e2e:issue-3137-perf
 *
 * Focused smoke:
 *
 *   npm run e2e:issue-3137-perf -- \
 *     --formats=hwp --kinds=english --cadences=0 --iterations=3 --warmups=1
 *
 * focused geometry 최적화 게이트(Stage 3):
 *
 *   npm run e2e:issue-3137-perf -- --require-focused-geometry
 *
 * focused page-tree repaint 최적화 게이트(Stage 4):
 *
 *   npm run e2e:issue-3137-perf -- \
 *     --require-focused-geometry --require-focused-repaint
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  closeBrowser,
  closePage,
  createPage,
  launchBrowser,
  loadApp,
} from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '../..');
const FRAME_BUDGET_MS = 1000 / 60;

const TARGET = Object.freeze({
  sectionIndex: 0,
  paragraphIndex: 5,
  charOffset: 130,
  parentParaIndex: 0,
  controlIndex: 2,
  cellIndex: 2,
  cellParaIndex: 5,
  cellPath: [{ controlIndex: 2, cellIndex: 2, cellParaIndex: 5 }],
});

const SAMPLES = Object.freeze({
  hwp: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwp'),
  hwpx: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwpx'),
});

const INPUT_KINDS = Object.freeze({
  english: Object.freeze({
    finalText: 'a',
    phases: Object.freeze([
      Object.freeze({
        phase: 'a',
        data: 'a',
        inputType: 'insertText',
        isComposing: false,
        compositionEnd: false,
      }),
    ]),
  }),
  digit: Object.freeze({
    finalText: '1',
    phases: Object.freeze([
      Object.freeze({
        phase: '1',
        data: '1',
        inputType: 'insertText',
        isComposing: false,
        compositionEnd: false,
      }),
    ]),
  }),
  ime: Object.freeze({
    finalText: '한',
    phases: Object.freeze([
      Object.freeze({
        phase: 'ㅎ',
        data: 'ㅎ',
        inputType: 'insertCompositionText',
        isComposing: true,
        compositionEnd: false,
      }),
      Object.freeze({
        phase: '하',
        data: '하',
        inputType: 'insertCompositionText',
        isComposing: true,
        compositionEnd: false,
      }),
      Object.freeze({
        phase: '한',
        data: '한',
        inputType: 'insertCompositionText',
        isComposing: true,
        compositionEnd: true,
      }),
    ]),
  }),
});

const EVENT_TYPES = Object.freeze({
  mutationInsert: 'wasm.insertTextInCellDeferredPagination',
  mutationReplace: 'wasm.replaceTextInCellDeferredPagination',
  operation: 'InputHandler.executeOperation',
  cursorNear: 'wasm.getCursorRectByPathNear',
  cursorPath: 'wasm.getCursorRectByPath',
  cursorCell: 'wasm.getCursorRectInCell',
  cursorPrepare: 'CursorState.prepareFocusedCellCursorGeometry',
  cursorUpdate: 'CursorState.updateRect',
  pageRefresh: 'CanvasView.refreshInvalidatedPageNow',
  pageRender: 'PageRenderer.renderPage',
  pagePatchRender: 'WasmBridge.renderPagePatchToCanvasFiltered',
  pageFullRender: 'WasmBridge.renderPageToCanvasFiltered',
  begin: 'wasm.beginDeferredPagination',
  step: 'wasm.stepDeferredPagination',
  flush: 'wasm.flushDeferredPagination',
  inputFlush: 'input.flushDeferredPaginationIfNeeded',
});

function cliValue(name, fallback) {
  const prefix = `--${name}=`;
  const arg = process.argv.find((value) => value.startsWith(prefix));
  return arg ? arg.slice(prefix.length) : fallback;
}

function hasFlag(name) {
  return process.argv.includes(`--${name}`);
}

function parseStringList(name, fallback, allowed) {
  const values = cliValue(name, fallback)
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  assert.ok(values.length > 0, `--${name} must not be empty`);
  for (const value of values) {
    assert.ok(allowed.has(value), `unsupported --${name} value: ${value}`);
  }
  return [...new Set(values)];
}

function parsePositiveInteger(name, fallback, minimum = 1) {
  const value = Number(cliValue(name, String(fallback)));
  assert.ok(
    Number.isInteger(value) && value >= minimum,
    `--${name} must be an integer >= ${minimum}`,
  );
  return value;
}

function parseCadences() {
  const values = cliValue('cadences', '0,80,150,250')
    .split(',')
    .map((value) => Number(value.trim()));
  assert.ok(values.length > 0, '--cadences must not be empty');
  for (const value of values) {
    assert.ok(Number.isFinite(value) && value >= 0, `invalid cadence: ${value}`);
  }
  return [...new Set(values)];
}

function parseConfig() {
  const formats = parseStringList('formats', 'hwp,hwpx', new Set(Object.keys(SAMPLES)));
  const kinds = parseStringList('kinds', 'english,digit,ime', new Set(Object.keys(INPUT_KINDS)));
  const outputValue = cliValue(
    'output',
    process.env.ISSUE3137_OUTPUT_ROOT ?? path.join(REPO_ROOT, 'output/poc/task3137/perf-baseline'),
  );
  return {
    formats,
    kinds,
    cadencesMs: parseCadences(),
    runs: parsePositiveInteger('runs', 1),
    iterations: parsePositiveInteger('iterations', 20),
    warmups: parsePositiveInteger('warmups', 2, 0),
    outputRoot: path.resolve(outputValue),
    browserMode: cliValue('mode', 'host'),
    allowSyncFlush: hasFlag('allow-sync-flush'),
    enforceFrameBudget: hasFlag('enforce-frame-budget'),
    requireFocusedGeometry: hasFlag('require-focused-geometry'),
    requireFocusedRepaint: hasFlag('require-focused-repaint'),
  };
}

function percentile(values, quantile) {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil(sorted.length * quantile) - 1),
  );
  return sorted[index];
}

function summarize(values) {
  const finite = values.filter((value) => Number.isFinite(value));
  const total = finite.reduce((sum, value) => sum + value, 0);
  return {
    count: finite.length,
    minMs: finite.length ? Math.min(...finite) : null,
    p50Ms: percentile(finite, 0.5),
    p95Ms: percentile(finite, 0.95),
    maxMs: finite.length ? Math.max(...finite) : null,
    meanMs: finite.length ? total / finite.length : null,
  };
}

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function writeJson(filePath, value) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function commandOutput(command, args) {
  try {
    return execFileSync(command, args, {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return null;
  }
}

function readFileFingerprint(filePath) {
  if (!existsSync(filePath)) return null;
  const bytes = readFileSync(filePath);
  const stat = statSync(filePath);
  return {
    path: filePath,
    size: bytes.length,
    sha256: sha256(bytes),
    modifiedAt: stat.mtime.toISOString(),
  };
}

async function waitTwoRafs(page) {
  await page.evaluate(() => new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(resolve));
  }));
}

async function delay(page, ms) {
  await page.evaluate((value) => new Promise((resolve) => setTimeout(resolve, value)), ms);
}

async function clickBlockingModalChoice(page) {
  return page.evaluate(() => {
    const allowed = new Set(['그대로 보기', '대체 글꼴로 보기']);
    const buttons = Array.from(document.querySelectorAll('button'));
    const button = buttons.find((candidate) => {
      const label = candidate.textContent?.trim() ?? '';
      const style = getComputedStyle(candidate);
      return allowed.has(label) && style.display !== 'none' && style.visibility !== 'hidden';
    });
    if (!button) return null;
    const label = button.textContent?.trim() ?? '';
    button.click();
    return label;
  });
}

async function openDocumentThroughApp(page, format, bytes) {
  const fileName = path.basename(SAMPLES[format]);
  const encoded = bytes.toString('base64');
  const requestId = `issue3137-${format}-${crypto.randomUUID()}`;

  await page.evaluate(({ base64, name, id }) => {
    const binary = atob(base64);
    const payload = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      payload[index] = binary.charCodeAt(index);
    }

    window.__issue3137LoadResult = null;
    const off = window.__eventBus.on('open-document-bytes:done', (result) => {
      if (result?.requestId !== id) return;
      off();
      window.__issue3137LoadResult = result;
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes: payload,
      fileName: name,
      fileHandle: null,
      skipUnsavedGuard: true,
      requestId: id,
    });
  }, { base64: encoded, name: fileName, id: requestId });

  const deadline = Date.now() + 90_000;
  const modalChoices = [];
  let result = null;
  while (Date.now() < deadline) {
    const choice = await clickBlockingModalChoice(page);
    if (choice) modalChoices.push(choice);
    result = await page.evaluate(() => window.__issue3137LoadResult);
    if (result) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(result, `${format}: open-document-bytes:done timeout`);
  assert.equal(result.ok, true, `${format}: document load failed: ${result.error ?? 'unknown'}`);

  await page.evaluate(() => document.fonts.ready);
  await waitTwoRafs(page);
  const state = await page.evaluate(() => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    return {
      sourceFormat: wasm.getSourceFormat(),
      pageCount: wasm.pageCount,
      inputActive: input.isActive(),
      fontsStatus: document.fonts.status,
      devicePixelRatio: window.devicePixelRatio,
      userAgent: navigator.userAgent,
    };
  });
  assert.equal(state.sourceFormat, format, `${format}: source format mismatch`);
  assert.equal(state.pageCount, 115, `${format}: expected 115 pages`);
  assert.equal(state.inputActive, true, `${format}: input handler inactive`);
  assert.equal(state.fontsStatus, 'loaded', `${format}: fonts are not ready`);
  return { ...state, modalChoices };
}

async function moveToTarget(page) {
  const state = await page.evaluate((target) => {
    const input = window.__inputHandler;
    input.cursor.clearSelection();
    input.cursor.moveTo(target);
    input.cursor.resetPreferredX();
    input.updateCaret();
    input.focus();
    return {
      position: input.cursor.getPosition(),
      rect: input.cursor.getRect(),
      focused: document.activeElement === input.textarea,
    };
  }, TARGET);
  assert.equal(state.focused, true, 'hidden textarea was not focused');
  assert.equal(state.position.charOffset, TARGET.charOffset, 'target char offset mismatch');
  assert.equal(state.position.cellParaIndex, TARGET.cellParaIndex, 'target cell paragraph mismatch');
  assert.deepEqual(state.position.cellPath, TARGET.cellPath, 'target cell path mismatch');
  return state;
}

async function readFocusedModel(page) {
  return page.evaluate((target) => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const length = wasm.getCellParagraphLength(
      target.sectionIndex,
      target.parentParaIndex,
      target.controlIndex,
      target.cellIndex,
      target.cellParaIndex,
    );
    return {
      length,
      text: wasm.getTextInCell(
        target.sectionIndex,
        target.parentParaIndex,
        target.controlIndex,
        target.cellIndex,
        target.cellParaIndex,
        0,
        length,
      ),
      cursor: input.cursor.getPosition(),
      cursorRect: input.cursor.getRect(),
      pageCount: wasm.pageCount,
      pending: input.hasDeferredPaginationPending(),
    };
  }, TARGET);
}

async function restoreTrace(page) {
  await page.evaluate(() => {
    const trace = window.__issue3137Trace;
    if (!trace) return;
    for (const restore of trace.restores ?? []) restore();
    trace.longTaskObserver?.disconnect();
    window.__issue3137Trace = null;
  });
}

async function installTrace(page) {
  await restoreTrace(page);
  await page.evaluate(() => {
    const trace = {
      startedAt: performance.now(),
      sequence: 0,
      currentSampleId: null,
      events: [],
      samples: [],
      longTasks: [],
      longTaskSequence: 0,
      restores: [],
      longTaskSupported: false,
      longTaskObserver: null,
    };

    trace.record = (type, detail = {}) => {
      const at = performance.now();
      trace.events.push({
        sequence: ++trace.sequence,
        type,
        sampleId: trace.currentSampleId,
        startTime: at,
        atMs: at - trace.startedAt,
        ...detail,
      });
    };

    trace.reset = () => {
      trace.startedAt = performance.now();
      trace.sequence = 0;
      trace.currentSampleId = null;
      trace.events.length = 0;
      trace.samples.length = 0;
      trace.longTasks.length = 0;
      trace.longTaskSequence = 0;
    };

    const wrap = (
      object,
      name,
      type,
      describeArgs = () => ({}),
      describeResult = () => ({}),
    ) => {
      const original = object?.[name];
      if (typeof original !== 'function') return;
      object[name] = function issue3137Wrapped(...args) {
        const sampleId = trace.currentSampleId;
        const startedAt = performance.now();
        let result;
        try {
          result = original.apply(this, args);
          return result;
        } finally {
          const finishedAt = performance.now();
          trace.events.push({
            sequence: ++trace.sequence,
            type,
            sampleId,
            startTime: startedAt,
            atMs: startedAt - trace.startedAt,
            endTime: finishedAt,
            durationMs: finishedAt - startedAt,
            ...describeArgs(args),
            ...describeResult(result),
          });
        }
      };
      trace.restores.push(() => {
        object[name] = original;
      });
    };

    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const canvasView = window.__canvasView;
    const renderer = canvasView.pageRenderer;
    const describePaginationResult = (result) => ({
      status: result?.status ?? null,
      revision: result?.revision ?? null,
      fragmentsProcessed: result?.fragmentsProcessed ?? null,
      pageCount: result?.pageCount ?? null,
    });

    wrap(
      wasm,
      'insertTextInCellDeferredPagination',
      'wasm.insertTextInCellDeferredPagination',
      (args) => ({
        sectionIndex: args[0],
        parentParaIndex: args[1],
        controlIndex: args[2],
        cellIndex: args[3],
        cellParaIndex: args[4],
        charOffset: args[5],
        text: String(args[6] ?? ''),
      }),
      (result) => ({
        resultCharOffset: result?.charOffset ?? null,
        paginationDeferred: result?.paginationDeferred ?? null,
        cellFlowChanged: result?.cellFlowChanged ?? null,
        focusedGeometryProvided: Boolean(result?.focusedCursorGeometry),
        focusedPageTreePatched: result?.focusedPageTreePatched === true,
        focusedPagePatchProvided: Boolean(result?.focusedPagePatch),
      }),
    );
    wrap(
      wasm,
      'replaceTextInCellDeferredPagination',
      'wasm.replaceTextInCellDeferredPagination',
      (args) => ({
        sectionIndex: args[0],
        parentParaIndex: args[1],
        controlIndex: args[2],
        cellIndex: args[3],
        cellParaIndex: args[4],
        charOffset: args[5],
        deleteCount: args[6],
        text: String(args[7] ?? ''),
      }),
      (result) => ({
        resultCharOffset: result?.charOffset ?? null,
        paginationDeferred: result?.paginationDeferred ?? null,
        cellFlowChanged: result?.cellFlowChanged ?? null,
        focusedGeometryProvided: Boolean(result?.focusedCursorGeometry),
        focusedPageTreePatched: result?.focusedPageTreePatched === true,
        focusedPagePatchProvided: Boolean(result?.focusedPagePatch),
      }),
    );
    wrap(
      wasm,
      'beginDeferredPagination',
      'wasm.beginDeferredPagination',
      (args) => ({ fragmentBudget: args[0] }),
      describePaginationResult,
    );
    wrap(
      wasm,
      'stepDeferredPagination',
      'wasm.stepDeferredPagination',
      (args) => ({ fragmentBudget: args[0] }),
      describePaginationResult,
    );
    wrap(
      wasm,
      'flushDeferredPagination',
      'wasm.flushDeferredPagination',
      (args) => ({ argsLength: args.length }),
    );
    wrap(
      input,
      'flushDeferredPaginationIfNeeded',
      'input.flushDeferredPaginationIfNeeded',
      (args) => ({
        reason: args[0] ?? 'manual',
        emitChange: args[1] ?? true,
      }),
      (result) => ({ result }),
    );
    const describeCursorNearArgs = (args) => ({
      sectionIndex: args[0],
      parentParaIndex: args[1],
      charOffset: args[3],
      hintPage: args[4],
    });
    wrap(
      wasm,
      'getCursorRectByPathNear',
      'wasm.getCursorRectByPathNear',
      describeCursorNearArgs,
    );
    wrap(
      wasm,
      'getCursorRectByPath',
      'wasm.getCursorRectByPath',
      (args) => ({
        sectionIndex: args[0],
        parentParaIndex: args[1],
        charOffset: args[3],
      }),
    );
    wrap(
      wasm,
      'getCursorRectInCell',
      'wasm.getCursorRectInCell',
      (args) => ({
        sectionIndex: args[0],
        parentParaIndex: args[1],
        controlIndex: args[2],
        cellIndex: args[3],
        cellParaIndex: args[4],
        charOffset: args[5],
      }),
    );
    wrap(
      input,
      'executeOperation',
      'InputHandler.executeOperation',
      (args) => ({
        kind: args[0]?.kind ?? null,
        commandType: args[0]?.command?.type ?? null,
      }),
    );
    wrap(
      input,
      'prepareTextMutationBeforeCursor',
      'InputHandler.prepareTextMutationBeforeCursor',
      (args) => ({
        deferredPagination: args[0]?.documentPaginationPending ?? null,
        cellFlowChanged: args[0]?.flowChanged ?? null,
        paginationCompleted: args[0]?.paginationCompleted ?? null,
      }),
      (result) => ({ result }),
    );
    wrap(
      input.cursor,
      'prepareFocusedCellCursorGeometry',
      'CursorState.prepareFocusedCellCursorGeometry',
      (args) => ({
        baseRevision: args[0]?.baseRevision ?? null,
        revision: args[0]?.revision ?? null,
      }),
      (result) => ({ prepared: result === true }),
    );
    wrap(input.cursor, 'updateRect', 'CursorState.updateRect');
    wrap(input, 'updateCaret', 'InputHandler.updateCaret');
    wrap(
      canvasView,
      'refreshInvalidatedPageNow',
      'CanvasView.refreshInvalidatedPageNow',
      (args) => ({
        pageIndex: args[0],
        context: args[1] ?? {},
      }),
    );
    wrap(
      renderer,
      'renderPage',
      'PageRenderer.renderPage',
      (args) => ({
        pageIndex: args[0],
        renderScale: args[2],
        zoom: args[3],
        dpr: args[4],
      }),
    );
    wrap(
      wasm,
      'renderPagePatchToCanvasFiltered',
      'WasmBridge.renderPagePatchToCanvasFiltered',
      (args) => ({
        pageIndex: args[0],
        renderScale: args[2],
        layerKind: args[3],
        patch: args[4] ?? null,
      }),
    );
    wrap(
      wasm,
      'renderPageToCanvasFiltered',
      'WasmBridge.renderPageToCanvasFiltered',
      (args) => ({
        pageIndex: args[0],
        renderScale: args[2],
        layerKind: args[3],
      }),
    );

    if (
      typeof PerformanceObserver === 'function'
      && PerformanceObserver.supportedEntryTypes?.includes('longtask')
    ) {
      trace.longTaskSupported = true;
      trace.longTaskObserver = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          trace.longTasks.push({
            id: ++trace.longTaskSequence,
            name: entry.name,
            entryType: entry.entryType,
            startTime: entry.startTime,
            durationMs: entry.duration,
            attribution: Array.from(entry.attribution ?? []).map((item) => ({
              name: item.name,
              containerType: item.containerType,
              containerSrc: item.containerSrc,
              containerId: item.containerId,
              containerName: item.containerName,
            })),
          });
        }
      });
      trace.longTaskObserver.observe({ type: 'longtask' });
    }

    window.__issue3137Trace = trace;
  });
}

async function resetTrace(page) {
  await page.evaluate(() => {
    const trace = window.__issue3137Trace;
    if (!trace) throw new Error('issue3137 trace is not installed');
    trace.reset();
  });
}

async function runInputSequence(page, {
  kind,
  logicalInputs,
  cadenceMs,
  samplePrefix,
}) {
  const definition = INPUT_KINDS[kind];
  return page.evaluate(async (config) => {
    const trace = window.__issue3137Trace;
    if (!trace) throw new Error('issue3137 trace is not installed');
    const input = window.__inputHandler;
    const textarea = input.textarea;
    const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const nextRaf = () => new Promise((resolve) => {
      requestAnimationFrame(() => resolve(performance.now()));
    });
    const samples = [];
    const pendingRafs = [];
    let sampleIndex = 0;
    let previousStartTime = null;

    const runSample = async (logicalIndex, phase) => {
      const sampleId = `${config.samplePrefix}-${sampleIndex + 1}`;
      const startedAt = performance.now();
      const actualStartIntervalMs = previousStartTime === null
        ? null
        : startedAt - previousStartTime;
      previousStartTime = startedAt;
      trace.currentSampleId = sampleId;
      trace.record('sample.start', {
        sampleId,
        kind: config.kind,
        logicalIndex,
        sampleIndex,
        phase: phase.phase,
        requestedCadenceMs: config.cadenceMs,
      });

      textarea.value = phase.data;
      textarea.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        data: phase.data,
        inputType: phase.inputType,
        isComposing: phase.isComposing,
      }));
      if (phase.compositionEnd) {
        textarea.dispatchEvent(new CompositionEvent('compositionend', {
          bubbles: true,
          data: phase.data,
        }));
      }
      const syncCompletedAt = performance.now();
      const cursor = input.cursor.getPosition();
      const sample = {
        sampleId,
        kind: config.kind,
        logicalIndex,
        sampleIndex,
        phase: phase.phase,
        data: phase.data,
        startedAt,
        syncCompletedAt,
        firstRafAt: null,
        secondRafAt: null,
        syncDispatchMs: syncCompletedAt - startedAt,
        inputToFirstRafMs: null,
        inputToSecondRafMs: null,
        requestedCadenceMs: config.cadenceMs,
        actualStartIntervalMs,
        cursorOffset: cursor.charOffset,
      };
      trace.samples.push(sample);
      samples.push(sample);
      trace.currentSampleId = null;

      pendingRafs.push((async () => {
        sample.firstRafAt = await nextRaf();
        sample.secondRafAt = await nextRaf();
        sample.inputToFirstRafMs = sample.firstRafAt - startedAt;
        sample.inputToSecondRafMs = sample.secondRafAt - startedAt;
        trace.record('sample.end', {
          sampleId,
          kind: config.kind,
          logicalIndex,
          sampleIndex: sample.sampleIndex,
          phase: phase.phase,
          inputToSecondRafMs: sample.inputToSecondRafMs,
        });
      })());
      sampleIndex += 1;

      const remainingCadenceMs = config.cadenceMs - (performance.now() - startedAt);
      // Even a requested 0ms cadence yields to a new browser task. This keeps
      // consecutive inputs realistic without forcing each one to wait for 2 rAF.
      await wait(Math.max(0, remainingCadenceMs));
    };

    for (let logicalIndex = 0; logicalIndex < config.logicalInputs; logicalIndex += 1) {
      if (config.kind === 'ime') {
        textarea.dispatchEvent(new CompositionEvent('compositionstart', {
          bubbles: true,
          data: '',
        }));
      }
      for (const phase of config.phases) {
        await runSample(logicalIndex, phase);
      }
    }
    await Promise.all(pendingRafs);
    return samples;
  }, {
    kind,
    phases: definition.phases,
    logicalInputs,
    cadenceMs,
    samplePrefix,
  });
}

async function collectTrace(page) {
  return page.evaluate(() => {
    const trace = window.__issue3137Trace;
    if (!trace) throw new Error('issue3137 trace is not installed');
    const events = trace.events.map((event) => ({ ...event }));
    const counts = {};
    for (const event of events) {
      counts[event.type] = (counts[event.type] ?? 0) + 1;
    }
    return {
      startedAt: trace.startedAt,
      longTaskSupported: trace.longTaskSupported,
      events,
      samples: trace.samples.map((sample) => ({ ...sample })),
      longTasks: trace.longTasks
        .filter((entry) => entry.startTime >= trace.startedAt)
        .map((entry) => ({ ...entry })),
      counts,
    };
  });
}

function eventsForSample(trace, sampleId, types) {
  const allowed = new Set(types);
  return trace.events.filter(
    (event) => event.sampleId === sampleId && allowed.has(event.type),
  );
}

function sumDurations(events) {
  return events.reduce((sum, event) => sum + (event.durationMs ?? 0), 0);
}

function overlappingLongTasks(trace, sample) {
  return trace.longTasks.filter((entry) => {
    const endTime = entry.startTime + entry.durationMs;
    return entry.startTime < sample.secondRafAt && endTime > sample.startedAt;
  });
}

function buildSampleMetrics(trace) {
  return trace.samples.map((sample) => {
    const mutationEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.mutationInsert,
      EVENT_TYPES.mutationReplace,
    ]);
    const operationEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.operation,
    ]);
    const cursorNearEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.cursorNear,
    ]);
    const cursorAllEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.cursorNear,
      EVENT_TYPES.cursorPath,
      EVENT_TYPES.cursorCell,
    ]);
    const cursorPrepareEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.cursorPrepare,
    ]);
    const cursorUpdateEvents = eventsForSample(trace, sample.sampleId, [
      EVENT_TYPES.cursorUpdate,
    ]);
    const longTasks = overlappingLongTasks(trace, sample);

    assert.equal(
      mutationEvents.length,
      1,
      `${sample.sampleId}: expected exactly one deferred mutation`,
    );
    if (sample.kind === 'ime') {
      assert.ok(
        operationEvents.length <= 1,
        `${sample.sampleId}: IME update has unexpected executeOperation calls`,
      );
    } else {
      assert.equal(
        operationEvents.length,
        1,
        `${sample.sampleId}: expected exactly one input operation`,
      );
    }
    assert.equal(
      mutationEvents[0].paginationDeferred,
      true,
      `${sample.sampleId}: mutation must keep pagination deferred`,
    );
    assert.ok(
      cursorNearEvents.length <= 1,
      `${sample.sampleId}: duplicate path-near cursor query`,
    );
    assert.equal(
      cursorPrepareEvents.length,
      1,
      `${sample.sampleId}: expected exactly one focused geometry prepare`,
    );
    assert.equal(
      cursorUpdateEvents.length,
      1,
      `${sample.sampleId}: expected exactly one cursor update`,
    );
    return {
      ...sample,
      stable: mutationEvents.every((event) => event.cellFlowChanged === false),
      focusedGeometryProvided: mutationEvents[0].focusedGeometryProvided === true,
      focusedPageTreePatched: mutationEvents[0].focusedPageTreePatched === true,
      focusedPagePatchProvided: mutationEvents[0].focusedPagePatchProvided === true,
      focusedGeometryPrepared: cursorPrepareEvents[0].prepared === true,
      mutationKind: mutationEvents[0].type,
      mutationMs: sumDurations(mutationEvents),
      // IME updates mutate through the raw input path and only compositionend
      // records history through executeOperation(). Keep their whole synchronous
      // input handler duration comparable with the #2424 baseline.
      operationMs: sample.kind === 'ime'
        ? sample.syncDispatchMs
        : sumDurations(operationEvents),
      operationSource: sample.kind === 'ime' ? 'input-dispatch' : 'execute-operation',
      executeOperationMs: sumDurations(operationEvents),
      executeOperationCount: operationEvents.length,
      cursorQueryMs: sumDurations(cursorNearEvents),
      cursorQueryCount: cursorNearEvents.length,
      cursorAllMs: sumDurations(cursorAllEvents),
      cursorAllCount: cursorAllEvents.length,
      cursorPrepareMs: sumDurations(cursorPrepareEvents),
      cursorUpdateMs: sumDurations(cursorUpdateEvents),
      cursorUpdateCount: cursorUpdateEvents.length,
      longTaskCount: longTasks.length,
      longTaskTotalMs: sumDurations(longTasks),
      longTaskMaxMs: longTasks.length
        ? Math.max(...longTasks.map((entry) => entry.durationMs))
        : 0,
      longTasks: longTasks.map((entry) => ({
        id: entry.id,
        durationMs: entry.durationMs,
      })),
    };
  });
}

function summarizeSampleMetrics(sampleMetrics) {
  const summarizeGroup = (values) => {
    const longTasks = new Map();
    for (const value of values) {
      for (const entry of value.longTasks) longTasks.set(entry.id, entry);
    }
    const uniqueLongTasks = [...longTasks.values()];
    return {
      sampleCount: values.length,
      operation: summarize(values.map((value) => value.operationMs)),
      executeOperation: summarize(values.map((value) => value.executeOperationMs)),
      mutation: summarize(values.map((value) => value.mutationMs)),
      cursorQuery: summarize(values.map((value) => value.cursorQueryMs)),
      cursorAll: summarize(values.map((value) => value.cursorAllMs)),
      cursorPrepare: summarize(values.map((value) => value.cursorPrepareMs)),
      cursorUpdate: summarize(values.map((value) => value.cursorUpdateMs)),
      syncDispatch: summarize(values.map((value) => value.syncDispatchMs)),
      inputToFirstRaf: summarize(values.map((value) => value.inputToFirstRafMs)),
      inputToSecondRaf: summarize(values.map((value) => value.inputToSecondRafMs)),
      actualStartInterval: summarize(
        values
          .map((value) => value.actualStartIntervalMs)
          .filter((value) => value !== null),
      ),
      longTasks: {
        count: uniqueLongTasks.length,
        totalMs: uniqueLongTasks.reduce((sum, value) => sum + value.durationMs, 0),
        maxMs: uniqueLongTasks.length
          ? Math.max(...uniqueLongTasks.map((value) => value.durationMs))
          : 0,
      },
    };
  };

  const phases = {};
  for (const phase of new Set(sampleMetrics.map((value) => value.phase))) {
    phases[phase] = summarizeGroup(sampleMetrics.filter((value) => value.phase === phase));
  }
  const stable = sampleMetrics.filter((value) => value.stable);
  return {
    all: summarizeGroup(sampleMetrics),
    stable: summarizeGroup(stable),
    phases,
  };
}

function summarizeRepaintMetrics(trace) {
  const refresh = trace.events.filter((event) => event.type === EVENT_TYPES.pageRefresh);
  const renderPage = trace.events.filter((event) => event.type === EVENT_TYPES.pageRender);
  const focusedPatch = trace.events.filter(
    (event) => event.type === EVENT_TYPES.pagePatchRender,
  );
  const fullPage = trace.events.filter((event) => event.type === EVENT_TYPES.pageFullRender);
  return {
    refresh: summarize(refresh.map((event) => event.durationMs)),
    renderPage: summarize(renderPage.map((event) => event.durationMs)),
    focusedPatch: summarize(focusedPatch.map((event) => event.durationMs)),
    fullPage: summarize(fullPage.map((event) => event.durationMs)),
    refreshCount: refresh.length,
    renderPageCount: renderPage.length,
    focusedPatchCount: focusedPatch.length,
    fullPageCount: fullPage.length,
  };
}

function traceCount(trace, type) {
  return trace.counts[type] ?? 0;
}

function scenarioSlug({ format, kind, cadenceMs, runNumber }) {
  return `${format}-${kind}-${cadenceMs}ms-run-${runNumber}`;
}

function toSummaryScenario(result) {
  const { raw, ...summary } = result;
  return summary;
}

function formatMs(value) {
  return value === null || value === undefined ? 'n/a' : value.toFixed(2);
}

async function runScenario(page, fixture, config, scenario, pageErrors) {
  await restoreTrace(page);
  const errorCountBefore = pageErrors.length;
  const load = await openDocumentThroughApp(page, scenario.format, fixture.bytes);
  await moveToTarget(page);
  const initial = await readFocusedModel(page);
  assert.equal(initial.length, TARGET.charOffset, `${scenario.format}: initial text length`);

  await installTrace(page);
  if (config.warmups > 0) {
    const warmupSamples = await runInputSequence(page, {
      kind: scenario.kind,
      logicalInputs: config.warmups,
      cadenceMs: 0,
      samplePrefix: `${scenarioSlug(scenario)}-warmup`,
    });
    const expectedWarmupOffset = TARGET.charOffset + config.warmups;
    assert.equal(
      warmupSamples.at(-1)?.cursorOffset,
      expectedWarmupOffset,
      `${scenarioSlug(scenario)}: warm-up cursor offset`,
    );
  }
  await delay(page, 0);
  await resetTrace(page);

  const samples = await runInputSequence(page, {
    kind: scenario.kind,
    logicalInputs: config.iterations,
    cadenceMs: scenario.cadenceMs,
    samplePrefix: scenarioSlug(scenario),
  });
  await delay(page, 0);
  const trace = await collectTrace(page);
  const final = await readFocusedModel(page);
  const definition = INPUT_KINDS[scenario.kind];
  const expectedText = `${initial.text}${definition.finalText.repeat(config.warmups + config.iterations)}`;
  const expectedOffset = TARGET.charOffset + config.warmups + config.iterations;

  assert.equal(final.text, expectedText, `${scenarioSlug(scenario)}: final model text`);
  assert.equal(final.length, expectedOffset, `${scenarioSlug(scenario)}: final model length`);
  assert.equal(final.cursor.charOffset, expectedOffset, `${scenarioSlug(scenario)}: final cursor offset`);
  assert.equal(final.pageCount, 115, `${scenarioSlug(scenario)}: final page count`);
  assert.equal(
    pageErrors.length,
    errorCountBefore,
    `${scenarioSlug(scenario)}: browser page error`,
  );

  const sampleMetrics = buildSampleMetrics(trace);
  if (process.env.ISSUE3743_CAPTURE_PRECONTRACT === '1') {
    writeJson(path.join(config.outputRoot, `precontract-${scenarioSlug(scenario)}.json`), {
      load, initial, final, samples, trace, sampleMetrics,
      metrics: { ...summarizeSampleMetrics(sampleMetrics), repaint: summarizeRepaintMetrics(trace) },
    });
  }
  const expectedSampleCount = config.iterations * INPUT_KINDS[scenario.kind].phases.length;
  assert.equal(samples.length, expectedSampleCount, `${scenarioSlug(scenario)}: browser sample count`);
  assert.equal(trace.samples.length, expectedSampleCount, `${scenarioSlug(scenario)}: trace sample count`);
  assert.equal(sampleMetrics.length, expectedSampleCount, `${scenarioSlug(scenario)}: metric sample count`);
  assert.equal(
    sampleMetrics.filter((value) => value.stable).length,
    expectedSampleCount,
    `${scenarioSlug(scenario)}: flow-boundary input contaminated stable matrix`,
  );
  for (const sample of sampleMetrics) {
    assert.equal(
      sample.cursorOffset,
      TARGET.charOffset + config.warmups + sample.logicalIndex + 1,
      `${sample.sampleId}: cursor offset`,
    );
  }
  if (config.requireFocusedGeometry) {
    assert.equal(
      sampleMetrics.filter((value) => value.focusedGeometryProvided).length,
      expectedSampleCount,
      `${scenarioSlug(scenario)}: focused geometry mutation payload count`,
    );
    assert.equal(
      sampleMetrics.filter((value) => value.focusedGeometryPrepared).length,
      expectedSampleCount,
      `${scenarioSlug(scenario)}: focused geometry prepared count`,
    );
    assert.equal(
      sampleMetrics.reduce((sum, value) => sum + value.cursorAllCount, 0),
      0,
      `${scenarioSlug(scenario)}: stable focused geometry exact cursor fallback count`,
    );
  }
  if (config.requireFocusedRepaint) {
    assert.equal(
      sampleMetrics.filter((value) => value.focusedPageTreePatched).length,
      expectedSampleCount,
      `${scenarioSlug(scenario)}: focused page-tree patch count`,
    );
    assert.equal(
      sampleMetrics.filter((value) => value.focusedPagePatchProvided).length,
      expectedSampleCount,
      `${scenarioSlug(scenario)}: focused page repaint payload count`,
    );
  }

  const flushCount = traceCount(trace, EVENT_TYPES.flush);
  const inputFlushCount = traceCount(trace, EVENT_TYPES.inputFlush);
  if (!config.allowSyncFlush) {
    assert.equal(flushCount, 0, `${scenarioSlug(scenario)}: synchronous WASM flush count`);
    assert.equal(inputFlushCount, 0, `${scenarioSlug(scenario)}: input flush count`);
  }
  assert.equal(
    traceCount(trace, EVENT_TYPES.begin),
    0,
    `${scenarioSlug(scenario)}: stable scenario resumable begin count`,
  );
  assert.equal(
    traceCount(trace, EVENT_TYPES.step),
    0,
    `${scenarioSlug(scenario)}: stable scenario resumable step count`,
  );
  assert.equal(
    traceCount(trace, EVENT_TYPES.cursorCell),
    0,
    `${scenarioSlug(scenario)}: duplicate direct cell cursor lookup count`,
  );

  const metrics = {
    ...summarizeSampleMetrics(sampleMetrics),
    repaint: summarizeRepaintMetrics(trace),
  };
  if (config.requireFocusedRepaint) {
    assert.equal(
      metrics.repaint.focusedPatchCount,
      metrics.repaint.renderPageCount,
      `${scenarioSlug(scenario)}: every measured repaint must use the focused patch renderer`,
    );
    assert.equal(
      metrics.repaint.fullPageCount,
      0,
      `${scenarioSlug(scenario)}: stable focused repaint must not use full-page Canvas replay`,
    );
  }
  const frameBudgetMet = (
    (metrics.stable.operation.p95Ms ?? Infinity) <= FRAME_BUDGET_MS
    && (metrics.stable.cursorUpdate.p95Ms ?? Infinity) <= FRAME_BUDGET_MS
    && (metrics.repaint.renderPage.p95Ms ?? Infinity) <= FRAME_BUDGET_MS
  );
  const rawFile = path.join('raw', `${scenarioSlug(scenario)}.json`);
  const result = {
    ...scenario,
    logicalInputs: config.iterations,
    eventSamples: expectedSampleCount,
    warmups: config.warmups,
    load,
    counts: {
      mutationInsert: traceCount(trace, EVENT_TYPES.mutationInsert),
      mutationReplace: traceCount(trace, EVENT_TYPES.mutationReplace),
      operation: traceCount(trace, EVENT_TYPES.operation),
      cursorNear: traceCount(trace, EVENT_TYPES.cursorNear),
      cursorPath: traceCount(trace, EVENT_TYPES.cursorPath),
      cursorCell: traceCount(trace, EVENT_TYPES.cursorCell),
      cursorPrepare: traceCount(trace, EVENT_TYPES.cursorPrepare),
      cursorUpdate: traceCount(trace, EVENT_TYPES.cursorUpdate),
      focusedGeometryProvided: sampleMetrics.filter(
        (value) => value.focusedGeometryProvided,
      ).length,
      focusedGeometryPrepared: sampleMetrics.filter(
        (value) => value.focusedGeometryPrepared,
      ).length,
      focusedPageTreePatched: sampleMetrics.filter(
        (value) => value.focusedPageTreePatched,
      ).length,
      focusedPagePatchProvided: sampleMetrics.filter(
        (value) => value.focusedPagePatchProvided,
      ).length,
      focusedPagePatchRendered: metrics.repaint.focusedPatchCount,
      fullPageRendered: metrics.repaint.fullPageCount,
      wasmFlush: flushCount,
      inputFlush: inputFlushCount,
      wasmBegin: traceCount(trace, EVENT_TYPES.begin),
      wasmStep: traceCount(trace, EVENT_TYPES.step),
    },
    metrics,
    frameBudgetMs: FRAME_BUDGET_MS,
    frameBudgetMet,
    rawFile,
    raw: {
      scenario,
      initial,
      final,
      sampleMetrics,
      trace,
    },
  };

  await restoreTrace(page);
  console.log(
    `[${scenario.format.toUpperCase()} ${scenario.kind} ${scenario.cadenceMs}ms run ${scenario.runNumber}] `
      + `op p95=${formatMs(metrics.stable.operation.p95Ms)}ms, `
      + `cursor update p95=${formatMs(metrics.stable.cursorUpdate.p95Ms)}ms, `
      + `exact=${traceCount(trace, EVENT_TYPES.cursorNear)}, `
      + `render p95=${formatMs(metrics.repaint.renderPage.p95Ms)}ms `
      + `(patch=${metrics.repaint.focusedPatchCount}, full=${metrics.repaint.fullPageCount}), `
      + `2rAF p95=${formatMs(metrics.stable.inputToSecondRaf.p95Ms)}ms, `
      + `long=${metrics.stable.longTasks.count}, flush=${flushCount}`,
  );
  return result;
}

function summaryTsv(summary) {
  const header = [
    'format',
    'kind',
    'cadence_ms',
    'run',
    'logical_inputs',
    'event_samples',
    'stable_samples',
    'operation_p50_ms',
    'operation_p95_ms',
    'execute_operation_p50_ms',
    'execute_operation_p95_ms',
    'mutation_p50_ms',
    'mutation_p95_ms',
    'cursor_p50_ms',
    'cursor_p95_ms',
    'cursor_update_p50_ms',
    'cursor_update_p95_ms',
    'exact_cursor_count',
    'focused_geometry_count',
    'focused_page_tree_patch_count',
    'focused_page_patch_payload_count',
    'focused_page_patch_render_count',
    'focused_page_patch_render_p95_ms',
    'full_page_render_count',
    'full_page_render_p95_ms',
    'page_refresh_p95_ms',
    'page_render_p95_ms',
    'input_to_2raf_p50_ms',
    'input_to_2raf_p95_ms',
    'actual_interval_p50_ms',
    'actual_interval_p95_ms',
    'long_task_count',
    'long_task_max_ms',
    'wasm_flush',
    'wasm_begin',
    'wasm_step',
    'frame_budget_met',
  ];
  const rows = summary.scenarios.map((scenario) => {
    const stable = scenario.metrics.stable;
    return [
      scenario.format,
      scenario.kind,
      scenario.cadenceMs,
      scenario.runNumber,
      scenario.logicalInputs,
      scenario.eventSamples,
      stable.sampleCount,
      stable.operation.p50Ms,
      stable.operation.p95Ms,
      stable.executeOperation.p50Ms,
      stable.executeOperation.p95Ms,
      stable.mutation.p50Ms,
      stable.mutation.p95Ms,
      stable.cursorQuery.p50Ms,
      stable.cursorQuery.p95Ms,
      stable.cursorUpdate.p50Ms,
      stable.cursorUpdate.p95Ms,
      scenario.counts.cursorNear + scenario.counts.cursorPath + scenario.counts.cursorCell,
      scenario.counts.focusedGeometryPrepared,
      scenario.counts.focusedPageTreePatched,
      scenario.counts.focusedPagePatchProvided,
      scenario.counts.focusedPagePatchRendered,
      scenario.metrics.repaint.focusedPatch.p95Ms,
      scenario.counts.fullPageRendered,
      scenario.metrics.repaint.fullPage.p95Ms,
      scenario.metrics.repaint.refresh.p95Ms,
      scenario.metrics.repaint.renderPage.p95Ms,
      stable.inputToSecondRaf.p50Ms,
      stable.inputToSecondRaf.p95Ms,
      stable.actualStartInterval.p50Ms,
      stable.actualStartInterval.p95Ms,
      stable.longTasks.count,
      stable.longTasks.maxMs,
      scenario.counts.wasmFlush,
      scenario.counts.wasmBegin,
      scenario.counts.wasmStep,
      scenario.frameBudgetMet,
    ].map((value) => value ?? '').join('\t');
  });
  return `${[header.join('\t'), ...rows].join('\n')}\n`;
}

function writeSummaryArtifacts(summary, outputRoot) {
  writeJson(path.join(outputRoot, 'summary.json'), summary);
  writeFileSync(path.join(outputRoot, 'summary.tsv'), summaryTsv(summary));
}

async function main() {
  const config = parseConfig();
  mkdirSync(config.outputRoot, { recursive: true });

  const fixtures = Object.fromEntries(config.formats.map((format) => {
    const bytes = readFileSync(SAMPLES[format]);
    return [format, {
      path: SAMPLES[format],
      bytes,
      size: bytes.length,
      sha256: sha256(bytes),
    }];
  }));

  const wasmFingerprint = readFileFingerprint(path.join(REPO_ROOT, 'pkg/rhwp_bg.wasm'));
  assert.ok(
    wasmFingerprint,
    'pkg/rhwp_bg.wasm is missing; run wasm-pack build --target web --out-dir pkg',
  );

  const browser = await launchBrowser();
  const page = await createPage(browser, 1280, 900);
  const pageErrors = [];
  page.on('console', (message) => {
    if (message.type() === 'error' || message.type() === 'warning') {
      console.log(`  [browser:${message.type()}] ${message.text()}`);
    }
  });
  page.on('pageerror', (error) => {
    pageErrors.push({ message: error.message, stack: error.stack ?? null });
    console.log(`  [browser:pageerror] ${error.message}`);
  });

  const summary = {
    schemaVersion: '5.0',
    issue: 3137,
    status: 'running',
    startedAt: new Date().toISOString(),
    finishedAt: null,
    frameBudgetMs: FRAME_BUDGET_MS,
    config: {
      formats: config.formats,
      kinds: config.kinds,
      cadencesMs: config.cadencesMs,
      runs: config.runs,
      iterations: config.iterations,
      warmups: config.warmups,
      allowSyncFlush: config.allowSyncFlush,
      enforceFrameBudget: config.enforceFrameBudget,
      requireFocusedGeometry: config.requireFocusedGeometry,
      requireFocusedRepaint: config.requireFocusedRepaint,
    },
    environment: {
      gitHead: commandOutput('git', ['rev-parse', 'HEAD']),
      gitBranch: commandOutput('git', ['branch', '--show-current']),
      gitWorktreeDirty: Boolean(commandOutput('git', ['status', '--porcelain'])),
      viteUrl: process.env.VITE_URL ?? 'http://localhost:7700',
      browserMode: config.browserMode,
      freshChromeProfile: config.browserMode === 'headless',
      chromePath: process.env.CHROME_PATH ?? process.env.PUPPETEER_EXECUTABLE_PATH ?? null,
      browserVersion: null,
      userAgent: null,
      viewport: { width: 1280, height: 900, deviceScaleFactor: 1 },
      nodeVersion: process.version,
      platform: process.platform,
      arch: process.arch,
      wasmBuildCommand: 'wasm-pack build --target web --out-dir pkg',
      wasm: wasmFingerprint,
      harness: readFileFingerprint(fileURLToPath(import.meta.url)),
    },
    fixtures: Object.fromEntries(Object.entries(fixtures).map(([format, fixture]) => [
      format,
      {
        path: fixture.path,
        size: fixture.size,
        sha256: fixture.sha256,
      },
    ])),
    pageErrors,
    scenarios: [],
    error: null,
  };
  writeSummaryArtifacts(summary, config.outputRoot);

  let thrown = null;
  try {
    await loadApp(page);
    summary.environment.browserVersion = await browser.version();
    summary.environment.userAgent = await page.evaluate(() => navigator.userAgent);

    for (const format of config.formats) {
      for (const kind of config.kinds) {
        for (const cadenceMs of config.cadencesMs) {
          for (let runNumber = 1; runNumber <= config.runs; runNumber += 1) {
            const scenario = { format, kind, cadenceMs, runNumber };
            const result = await runScenario(
              page,
              fixtures[format],
              config,
              scenario,
              pageErrors,
            );
            writeJson(path.join(config.outputRoot, result.rawFile), result.raw);
            summary.scenarios.push(toSummaryScenario(result));
            writeSummaryArtifacts(summary, config.outputRoot);
          }
        }
      }
    }

    const budgetFailures = summary.scenarios.filter((scenario) => !scenario.frameBudgetMet);
    if (config.enforceFrameBudget) {
      assert.equal(
        budgetFailures.length,
        0,
        `frame budget exceeded in ${budgetFailures.length} scenario(s)`,
      );
    }
    summary.status = 'passed';
  } catch (error) {
    thrown = error;
    summary.status = 'failed';
    summary.error = {
      name: error?.name ?? 'Error',
      message: error?.message ?? String(error),
      stack: error?.stack ?? null,
    };
  } finally {
    summary.finishedAt = new Date().toISOString();
    summary.pageErrors = pageErrors;
    await restoreTrace(page).catch(() => {});
    await closePage(page).catch(() => {});
    await closeBrowser(browser).catch(() => {});
    writeSummaryArtifacts(summary, config.outputRoot);
  }

  if (thrown) throw thrown;
  console.log(`\nIssue #3137 performance matrix written to ${config.outputRoot}`);
  console.log(`Scenarios: ${summary.scenarios.length}, status=${summary.status}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
