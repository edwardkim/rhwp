/**
 * Issue #2214 focused GREEN regression and optional diagnostic.
 *
 * The default path is a focused HWP/HWPX regression that verifies the 44th
 * cell-flow boundary, pre-cursor pagination, exact tree/caret state, and the
 * absence of an additional flush through the 50th input. The original
 * timeline/PNG controls remain available behind --diagnose.
 *
 * Usage:
 *   node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless
 *   node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless --runs=1  # local smoke
 *   node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless --diagnose
 *   node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless --diagnose \
 *     --formats=hwp --runs=1
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';
import {
  closeBrowser,
  closePage,
  createPage,
  launchBrowser,
  loadApp,
} from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '../..');
const OUTPUT_ROOT = process.env.ISSUE2214_OUTPUT_ROOT
  ?? path.join(REPO_ROOT, 'output/poc/task2214/stage4');
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
const MAX_INSERTS = 128;

const SAMPLES = Object.freeze({
  hwp: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwp'),
  hwpx: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwpx'),
});

function cliValue(name, fallback) {
  const arg = process.argv.find((value) => value.startsWith(`--${name}=`));
  return arg ? arg.slice(name.length + 3) : fallback;
}

function sanitizeLabel(value) {
  return String(value).replace(/[^a-zA-Z0-9._-]+/g, '-');
}

function sha256(buffer) {
  return crypto.createHash('sha256').update(buffer).digest('hex');
}

function percentile(values, quantile) {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * quantile) - 1));
  return sorted[index];
}

function summarizeDurations(values) {
  return {
    count: values.length,
    p50Ms: percentile(values, 0.5),
    p95Ms: percentile(values, 0.95),
    maxMs: values.length ? Math.max(...values) : null,
  };
}

function writeJson(filePath, value) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
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
  const requestId = `issue2214-${format}-${crypto.randomUUID()}`;

  await page.evaluate(({ base64, name, id }) => {
    const binary = atob(base64);
    const payload = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) payload[i] = binary.charCodeAt(i);

    window.__issue2214LoadResult = null;
    const off = window.__eventBus.on('open-document-bytes:done', (result) => {
      if (result?.requestId !== id) return;
      off();
      window.__issue2214LoadResult = result;
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
    result = await page.evaluate(() => window.__issue2214LoadResult);
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
    const canvasView = window.__canvasView;
    return {
      sourceFormat: wasm.getSourceFormat(),
      pageCount: wasm.pageCount,
      inputActive: input.isActive(),
      hasPageZeroCanvas: Boolean(canvasView.canvasPool.getCanvas(0)),
      fontsStatus: document.fonts.status,
      devicePixelRatio: window.devicePixelRatio,
      zoom: canvasView.viewportManager.getZoom(),
      viewport: { width: window.innerWidth, height: window.innerHeight },
    };
  });
  assert.equal(state.sourceFormat, format, `${format}: source format mismatch`);
  assert.equal(state.pageCount, 115, `${format}: expected 115 pages`);
  assert.equal(state.inputActive, true, `${format}: input handler inactive`);
  assert.equal(state.hasPageZeroCanvas, true, `${format}: page zero canvas missing`);
  assert.equal(state.fontsStatus, 'loaded', `${format}: fonts are not ready`);
  return { ...state, modalChoices };
}

async function moveToTarget(page) {
  const position = await page.evaluate((target) => {
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
  assert.equal(position.focused, true, 'hidden textarea was not focused');
  assert.equal(position.position.charOffset, TARGET.charOffset, 'target char offset mismatch');
  assert.equal(position.position.cellParaIndex, TARGET.cellParaIndex, 'target cell paragraph mismatch');
  return position;
}

async function dispatchInputSynchronously(page, text = '1') {
  return page.evaluate((value) => {
    const input = window.__inputHandler;
    input.textarea.value = value;
    const startedAt = performance.now();
    input.textarea.dispatchEvent(new InputEvent('input', {
      bubbles: true,
      inputType: 'insertText',
      data: value,
    }));
    const wasm = window.__wasm;
    const position = input.cursor.getPosition();
    const length = wasm.getCellParagraphLength(0, 0, 2, 2, 5);
    const modelText = wasm.getTextInCell(0, 0, 2, 2, 5, 0, length);
    return {
      durationMs: performance.now() - startedAt,
      cursor: position,
      modelLength: length,
      modelTail: modelText.slice(-40),
      lineInfo: wasm.getLineInfoInCell(0, 0, 2, 2, 5, position.charOffset),
      pending: input.hasDeferredPaginationPending(),
    };
  }, text);
}

async function readLineCount(page) {
  return page.evaluate(() => {
    const input = window.__inputHandler;
    const offset = input.cursor.getPosition().charOffset;
    return window.__wasm.getLineInfoInCell(0, 0, 2, 2, 5, offset).lineCount;
  });
}

async function typeKeyboardOnes(page, count) {
  const durations = [];
  for (let i = 0; i < count; i += 1) {
    const startedAt = performance.now();
    await page.keyboard.type('1');
    durations.push(performance.now() - startedAt);
  }
  return durations;
}

async function restoreTrace(page) {
  await page.evaluate(() => {
    const trace = window.__issue2214Trace;
    if (!trace) return;
    for (const restore of trace.restores ?? []) restore();
    for (const off of trace.offs ?? []) off();
    window.__issue2214Trace = null;
  });
}

async function installTrace(page) {
  await restoreTrace(page);
  await page.evaluate(() => {
    const trace = {
      startedAt: performance.now(),
      events: [],
      restores: [],
      offs: [],
      sequence: 0,
    };
    const record = (type, detail = {}) => {
      trace.events.push({
        sequence: ++trace.sequence,
        type,
        atMs: performance.now() - trace.startedAt,
        ...detail,
      });
    };
    const wrap = (object, name, type, describeArgs = () => ({}), describeResult = () => ({})) => {
      const original = object?.[name];
      if (typeof original !== 'function') return;
      object[name] = function issue2214Wrapped(...args) {
        const startedAt = performance.now();
        let result;
        try {
          result = original.apply(this, args);
          return result;
        } finally {
          record(type, {
            ...describeArgs(args),
            ...describeResult(result),
            durationMs: performance.now() - startedAt,
          });
        }
      };
      trace.restores.push(() => { object[name] = original; });
    };

    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const canvasView = window.__canvasView;
    const renderer = canvasView.pageRenderer;

    wrap(wasm, 'flushDeferredPagination', 'wasm.flushDeferredPagination', (args) => ({ argsLength: args.length }));
    wrap(input, 'flushDeferredPaginationIfNeeded', 'input.flushDeferredPaginationIfNeeded', (args) => ({
      reason: args[0] ?? 'manual',
      emitChange: args[1] ?? true,
    }), (result) => ({ result }));
    wrap(wasm, 'renderPageToCanvasFiltered', 'wasm.renderPageToCanvasFiltered', (args) => ({
      pageIndex: args[0],
      scale: args[2],
      layerKind: args[3],
    }));
    wrap(wasm, 'insertTextInCellDeferredPagination', 'wasm.insertTextInCellDeferredPagination', (args) => ({
      sectionIndex: args[0],
      parentParaIndex: args[1],
      controlIndex: args[2],
      cellIndex: args[3],
      cellParaIndex: args[4],
      charOffset: args[5],
      textLength: String(args[6] ?? '').length,
    }), (result) => ({
      resultCharOffset: result?.charOffset ?? null,
      paginationDeferred: result?.paginationDeferred ?? null,
      cellFlowChanged: result?.cellFlowChanged ?? null,
    }));
    wrap(wasm, 'getCursorRectByPathNear', 'wasm.getCursorRectByPathNear', (args) => ({
      sectionIndex: args[0],
      parentParaIndex: args[1],
      charOffset: args[3],
      hintPage: args[4],
    }));
    wrap(wasm, 'getCursorRectByPath', 'wasm.getCursorRectByPath', (args) => ({
      sectionIndex: args[0],
      parentParaIndex: args[1],
      charOffset: args[3],
    }));
    wrap(input.cursor, 'updateRect', 'CursorState.updateRect');
    wrap(input.cursor, 'moveTo', 'CursorState.moveTo', (args) => ({
      charOffset: args[0]?.charOffset,
      cellParaIndex: args[0]?.cellParaIndex,
    }));
    wrap(input, 'executeOperation', 'InputHandler.executeOperation', (args) => ({
      kind: args[0]?.kind,
      commandType: args[0]?.command?.type,
    }));
    wrap(input, 'prepareTextMutationBeforeCursor', 'InputHandler.prepareTextMutationBeforeCursor', (args) => ({
      deferredPagination: args[0]?.deferredPagination ?? null,
      cellFlowChanged: args[0]?.cellFlowChanged ?? null,
      paginationCompleted: args[0]?.paginationCompleted ?? null,
    }), (result) => ({ result }));
    wrap(input, 'refreshAfterOperation', 'InputHandler.refreshAfterOperation', (args) => ({
      requested: args[0],
      fallback: args[1],
      commandType: args[2],
    }));
    wrap(input, 'afterPageLocalEdit', 'InputHandler.afterPageLocalEdit');
    wrap(input, 'flushDeferredPaginationForCellOverflow', 'InputHandler.flushDeferredPaginationForCellOverflow',
      () => ({}), (result) => ({ result }));
    wrap(input, 'updateCaret', 'InputHandler.updateCaret');
    wrap(renderer, 'renderPage', 'PageRenderer.renderPage', (args) => ({
      pageIndex: args[0],
      renderScale: args[2],
      zoom: args[3],
      dpr: args[4],
      context: args[5] ?? {},
    }), (result) => ({ result }));
    wrap(canvasView, 'refreshInvalidatedPageNow', 'CanvasView.refreshInvalidatedPageNow', (args) => ({
      pageIndex: args[0],
      context: args[1] ?? {},
    }));

    trace.offs.push(window.__eventBus.on('document-page-invalidated', (payload) => {
      record('event.document-page-invalidated', { payload });
    }));
    trace.offs.push(window.__eventBus.on('document-changed', (payload) => {
      record('event.document-changed', { payload });
    }));
    window.__issue2214Trace = trace;
  });
}

async function collectTrace(page) {
  return page.evaluate(() => {
    const trace = window.__issue2214Trace;
    const events = trace?.events ?? [];
    const count = (type) => events.filter((event) => event.type === type).length;
    return {
      startedAt: trace?.startedAt ?? null,
      events,
      counts: {
        wasmFlush: count('wasm.flushDeferredPagination'),
        inputFlush: count('input.flushDeferredPaginationIfNeeded'),
        invalidation: count('event.document-page-invalidated'),
        documentChanged: count('event.document-changed'),
        filteredRender: count('wasm.renderPageToCanvasFiltered'),
        pageRender: count('PageRenderer.renderPage'),
        refreshNow: count('CanvasView.refreshInvalidatedPageNow'),
        deferredInsert: count('wasm.insertTextInCellDeferredPagination'),
        cursorRectNear: count('wasm.getCursorRectByPathNear'),
        cursorRect: count('wasm.getCursorRectByPath'),
        executeOperation: count('InputHandler.executeOperation'),
        prepareTextMutation: count('InputHandler.prepareTextMutationBeforeCursor'),
      },
    };
  });
}

async function resolveCompositedClip(page) {
  const clip = await page.evaluate(() => {
    const canvasView = window.__canvasView;
    const wasm = window.__wasm;
    const canvas = canvasView.canvasPool.getCanvas(0);
    if (!canvas) throw new Error('page zero canvas unavailable');
    const rect = canvas.getBoundingClientRect();
    const tree = wasm.getPageLayerTreeObject(0);
    const ops = [];
    const visit = (node) => {
      if (!node) return;
      if (node.kind === 'group') for (const child of node.children ?? []) visit(child);
      else if (node.kind === 'clipRect') visit(node.child);
      else if (node.kind === 'leaf') ops.push(...(node.ops ?? []));
    };
    visit(tree.root);
    const targets = ops.filter((op) => {
      if (op.type !== 'textRun') return false;
      const key = op.source?.stableSourceKey ?? '';
      return key.startsWith('section:0/para:5/') && key.includes('/cell:0:2:2:5:');
    });
    const minY = targets.length
      ? Math.min(...targets.map((op) => op.bbox?.y ?? 0))
      : (window.__inputHandler.cursor.getRect()?.y ?? 200);
    const scale = tree.pageWidth > 0 ? rect.width / tree.pageWidth : 1;
    const y = Math.max(0, Math.floor(rect.top + (minY - 35) * scale));
    return {
      x: Math.max(0, Math.floor(rect.left)),
      y,
      width: Math.max(1, Math.floor(rect.width)),
      height: 240,
      pageWidth: tree.pageWidth,
      scale,
      targetOpCount: targets.length,
    };
  });
  assert.ok(clip.targetOpCount > 0, 'target paragraph textRun ops were not found');
  return clip;
}

async function captureCompositedCrop(page, filePath, clip) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  await page.evaluate(() => {
    for (const element of document.querySelectorAll('#scroll-content .caret, #scroll-content .caret-composition')) {
      element.dataset.issue2214Visibility = element.style.visibility;
      element.style.visibility = 'hidden';
    }
  });
  try {
    await page.screenshot({
      path: filePath,
      clip: { x: clip.x, y: clip.y, width: clip.width, height: clip.height },
      captureBeyondViewport: true,
    });
  } finally {
    await page.evaluate(() => {
      for (const element of document.querySelectorAll('#scroll-content .caret, #scroll-content .caret-composition')) {
        element.style.visibility = element.dataset.issue2214Visibility ?? '';
        delete element.dataset.issue2214Visibility;
      }
    });
  }
  const bytes = readFileSync(filePath);
  return { filePath, sha256: sha256(bytes), size: bytes.length, clip };
}

async function collectState(page, label, timelineStart) {
  return page.evaluate(({ checkpoint, startedAt }) => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const canvasView = window.__canvasView;
    const position = input.cursor.getPosition();
    const rect = input.cursor.getRect();
    const length = wasm.getCellParagraphLength(0, 0, 2, 2, 5);
    const text = wasm.getTextInCell(0, 0, 2, 2, 5, 0, length);

    const lineByIndex = new Map();
    for (let offset = 0; offset <= length; offset += 1) {
      try {
        const info = wasm.getLineInfoInCell(0, 0, 2, 2, 5, offset);
        if (info && !lineByIndex.has(info.lineIndex)) lineByIndex.set(info.lineIndex, info);
      } catch { /* end offsets differ by format; retain successfully exposed lines */ }
    }
    const lines = Array.from(lineByIndex.values()).sort((a, b) => a.lineIndex - b.lineIndex);

    const pageIndex = Number.isInteger(rect?.pageIndex) ? rect.pageIndex : 0;
    const tree = wasm.getPageLayerTreeObject(pageIndex);
    const ops = [];
    const visit = (node) => {
      if (!node) return;
      if (node.kind === 'group') for (const child of node.children ?? []) visit(child);
      else if (node.kind === 'clipRect') visit(node.child);
      else if (node.kind === 'leaf') ops.push(...(node.ops ?? []));
    };
    visit(tree.root);
    const targetOps = ops.filter((op) => {
      if (op.type !== 'textRun') return false;
      const key = op.source?.stableSourceKey ?? '';
      return key.startsWith('section:0/para:5/') && key.includes('/cell:0:2:2:5:');
    });

    let layoutRuns = [];
    let layoutError = null;
    try {
      const layout = JSON.parse(wasm.doc.getPageTextLayout(pageIndex));
      layoutRuns = (layout.runs ?? []).filter((run) => (
        run.secIdx === 0
        && run.parentParaIdx === 0
        && run.controlIdx === 2
        && run.cellIdx === 2
        && run.cellParaIdx === 5
      )).map((run) => ({
        text: run.text,
        x: run.x,
        y: run.y,
        w: run.w,
        h: run.h,
        charStart: run.charStart,
        fontFamily: run.fontFamily,
        fontSize: run.fontSize,
      }));
    } catch (error) {
      layoutError = error instanceof Error ? error.message : String(error);
    }

    const caret = document.querySelector('#scroll-content .caret');
    const caretRect = caret?.getBoundingClientRect();
    const trace = window.__issue2214Trace;
    return {
      label: checkpoint,
      atMs: performance.now() - startedAt,
      model: {
        length,
        text,
        tail: text.slice(-40),
        lineCount: lines[0]?.lineCount ?? null,
        lines,
      },
      cursor: { position, rect },
      pagination: {
        pending: input.hasDeferredPaginationPending(),
        pageCount: wasm.pageCount,
      },
      renderState: {
        pendingRaf: canvasView.textEditRefreshRafId !== null,
        pendingPages: Array.from(canvasView.pendingTextEditRefreshes.keys()),
        verificationScheduled: canvasView.textEditStaticLayerVerifyTimers.has(pageIndex),
      },
      layerTree: {
        pageIndex,
        targetText: targetOps.map((op) => op.displayText ?? op.text ?? '').join(''),
        targetOps: targetOps.map((op) => ({
          text: op.displayText ?? op.text ?? '',
          bbox: op.bbox,
          baseline: op.baseline,
          stableSourceKey: op.source?.stableSourceKey ?? null,
          utf16Range: op.source?.utf16Range ?? null,
        })),
      },
      pageTextLayout: {
        targetText: layoutRuns.map((run) => run.text ?? '').join(''),
        runs: layoutRuns,
        error: layoutError,
      },
      caret: caret ? {
        display: getComputedStyle(caret).display,
        left: caret.style.left,
        top: caret.style.top,
        height: caret.style.height,
        clientRect: caretRect ? {
          x: caretRect.x,
          y: caretRect.y,
          width: caretRect.width,
          height: caretRect.height,
        } : null,
      } : null,
      traceEventCount: trace?.events.length ?? 0,
    };
  }, { checkpoint: label, startedAt: timelineStart });
}

async function captureCheckpoint(page, directory, label, timelineStart, clip) {
  const state = await collectState(page, label, timelineStart);
  const screenshotPath = path.join(directory, `${sanitizeLabel(label)}.png`);
  state.screenshot = await captureCompositedCrop(page, screenshotPath, clip);
  return state;
}

async function discoverTransition(page, format, bytes, runNumber) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToTarget(page);
  const initialLineCount = await readLineCount(page);
  assert.equal(initialLineCount, 4, `${format} run ${runNumber}: expected initial four lines`);

  const handlerDurationsMs = [];
  let transitionAt = null;
  for (let inserted = 1; inserted <= MAX_INSERTS; inserted += 1) {
    const startedAt = performance.now();
    await page.keyboard.type('1');
    handlerDurationsMs.push(performance.now() - startedAt);
    const lineCount = await readLineCount(page);
    if (lineCount > initialLineCount) {
      transitionAt = inserted;
      break;
    }
  }
  assert.ok(transitionAt, `${format} run ${runNumber}: no 4→5 line transition within ${MAX_INSERTS} inputs`);
  const model = await page.evaluate(() => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const length = wasm.getCellParagraphLength(0, 0, 2, 2, 5);
    return {
      length,
      textTail: wasm.getTextInCell(0, 0, 2, 2, 5, Math.max(0, length - 40), Math.min(40, length)),
      cursor: input.cursor.getPosition(),
      lineCount: wasm.getLineInfoInCell(0, 0, 2, 2, 5, input.cursor.getPosition().charOffset).lineCount,
      pending: input.hasDeferredPaginationPending(),
    };
  });
  return {
    format,
    runNumber,
    inputPath: 'page.keyboard.type',
    transitionAt,
    initialLineCount,
    finalLineCount: model.lineCount,
    handler: summarizeDurations(handlerDurationsMs),
    boundaryHandlerMs: handlerDurationsMs.at(-1),
    model,
    load,
  };
}

async function prepareScenario(page, format, bytes, transitionAt) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToTarget(page);
  const keyboardDurationsMs = await typeKeyboardOnes(page, Math.max(0, transitionAt - 1));
  await waitTwoRafs(page);
  const clip = await resolveCompositedClip(page);
  await installTrace(page);
  return { load, clip, keyboardDurationsMs };
}

async function runTimeline(page, format, bytes, transitionAt) {
  const directory = path.join(OUTPUT_ROOT, format, 'timeline');
  const prepared = await prepareScenario(page, format, bytes, transitionAt);
  const timelineStart = await page.evaluate(() => performance.now());
  const checkpoints = [];
  checkpoints.push(await captureCheckpoint(page, directory, 'before-boundary', timelineStart, prepared.clip));

  const boundaryInput = await dispatchInputSynchronously(page, '1');
  checkpoints.push(await captureCheckpoint(page, directory, 'sync-after-boundary', timelineStart, prepared.clip));
  await waitTwoRafs(page);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-2raf', timelineStart, prepared.clip));
  await delay(page, 100);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-100ms', timelineStart, prepared.clip));
  await delay(page, 750);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-850ms', timelineStart, prepared.clip));
  await delay(page, 750);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-1600ms', timelineStart, prepared.clip));

  const plusOne = await dispatchInputSynchronously(page, '1');
  await waitTwoRafs(page);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-n-plus-1', timelineStart, prepared.clip));
  const plusTwo = await dispatchInputSynchronously(page, '1');
  await waitTwoRafs(page);
  checkpoints.push(await captureCheckpoint(page, directory, 'after-n-plus-2', timelineStart, prepared.clip));

  const trace = await collectTrace(page);
  assert.equal(trace.counts.wasmFlush, 1, `${format}: timeline expected one boundary pagination flush`);
  return {
    kind: 'timeline',
    format,
    transitionAt,
    load: prepared.load,
    clip: prepared.clip,
    inputDurations: {
      beforeBoundaryKeyboard: summarizeDurations(prepared.keyboardDurationsMs),
      boundarySyncMs: boundaryInput.durationMs,
      plusOneSyncMs: plusOne.durationMs,
      plusTwoSyncMs: plusTwo.durationMs,
    },
    checkpoints,
    trace,
  };
}

async function runFullLayerControl(page, format, bytes, transitionAt) {
  const directory = path.join(OUTPUT_ROOT, format, 'full-layer-control');
  const prepared = await prepareScenario(page, format, bytes, transitionAt);
  const timelineStart = await page.evaluate(() => performance.now());
  const boundaryInput = await dispatchInputSynchronously(page, '1');
  await waitTwoRafs(page);
  const before = await captureCheckpoint(page, directory, 'before-full-layer', timelineStart, prepared.clip);
  const pageIndex = before.cursor.rect?.pageIndex ?? 0;
  await page.evaluate((value) => {
    window.__eventBus.emit('document-page-invalidated', { pageIndex: value, reason: 'unknown' });
  }, pageIndex);
  await waitTwoRafs(page);
  const after = await captureCheckpoint(page, directory, 'after-full-layer', timelineStart, prepared.clip);
  const trace = await collectTrace(page);
  assert.equal(trace.counts.wasmFlush, 1, `${format}: full-layer control expected one boundary pagination flush`);
  return {
    kind: 'full-layer-control',
    format,
    transitionAt,
    load: prepared.load,
    clip: prepared.clip,
    boundaryInputMs: boundaryInput.durationMs,
    checkpoints: [before, after],
    trace,
  };
}

async function runFlushControl(page, format, bytes, transitionAt) {
  const directory = path.join(OUTPUT_ROOT, format, 'flush-control');
  const prepared = await prepareScenario(page, format, bytes, transitionAt);
  const timelineStart = await page.evaluate(() => performance.now());
  const boundaryInput = await dispatchInputSynchronously(page, '1');
  await waitTwoRafs(page);
  const before = await captureCheckpoint(page, directory, 'before-explicit-flush', timelineStart, prepared.clip);
  const traceBefore = await collectTrace(page);
  assert.equal(traceBefore.counts.wasmFlush, 1, `${format}: boundary pagination must precede explicit control`);

  const explicit = await page.evaluate(() => {
    const input = window.__inputHandler;
    const position = input.cursor.getPosition();
    const startedAt = performance.now();
    const flushed = input.flushDeferredPaginationIfNeeded('issue-2214-e2e', false);
    input.cursor.moveTo(position);
    window.__eventBus.emit('document-view-changed');
    return { flushed, durationMs: performance.now() - startedAt };
  });
  assert.equal(explicit.flushed, false, `${format}: boundary effect must already be consumed`);
  await waitTwoRafs(page);
  const after = await captureCheckpoint(page, directory, 'after-explicit-flush', timelineStart, prepared.clip);
  const trace = await collectTrace(page);
  assert.equal(trace.counts.wasmFlush, 1, `${format}: explicit control must not add a second pagination flush`);
  return {
    kind: 'flush-control',
    format,
    transitionAt,
    load: prepared.load,
    clip: prepared.clip,
    boundaryInputMs: boundaryInput.durationMs,
    explicit,
    checkpoints: [before, after],
    traceBefore,
    trace,
  };
}

function comparePng(actualPath, expectedPath, diffPath) {
  const actual = PNG.sync.read(readFileSync(actualPath));
  const expected = PNG.sync.read(readFileSync(expectedPath));
  if (actual.width !== expected.width || actual.height !== expected.height) {
    return {
      comparable: false,
      actual: { width: actual.width, height: actual.height },
      expected: { width: expected.width, height: expected.height },
    };
  }
  const diff = new PNG({ width: actual.width, height: actual.height });
  const changedPixelCount = pixelmatch(
    actual.data,
    expected.data,
    diff.data,
    actual.width,
    actual.height,
    { threshold: 0.1 },
  );
  let minX = actual.width;
  let minY = actual.height;
  let maxX = -1;
  let maxY = -1;
  let maxChannelDelta = 0;
  for (let y = 0; y < actual.height; y += 1) {
    for (let x = 0; x < actual.width; x += 1) {
      const offset = (y * actual.width + x) * 4;
      let changed = false;
      for (let channel = 0; channel < 4; channel += 1) {
        const delta = Math.abs(actual.data[offset + channel] - expected.data[offset + channel]);
        maxChannelDelta = Math.max(maxChannelDelta, delta);
        if (delta > 0) changed = true;
      }
      if (changed) {
        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        maxX = Math.max(maxX, x);
        maxY = Math.max(maxY, y);
      }
    }
  }
  mkdirSync(path.dirname(diffPath), { recursive: true });
  writeFileSync(diffPath, PNG.sync.write(diff));
  return {
    comparable: true,
    width: actual.width,
    height: actual.height,
    threshold: 0.1,
    changedPixelCount,
    diffRatio: changedPixelCount / (actual.width * actual.height),
    maxChannelDelta,
    rawDiffBbox: maxX >= 0 ? { minX, minY, maxX, maxY } : null,
    diffPath,
    diffSha256: sha256(readFileSync(diffPath)),
  };
}

function checkpointByLabel(result, label) {
  const checkpoint = result.checkpoints.find((value) => value.label === label);
  assert.ok(checkpoint, `checkpoint not found: ${label}`);
  return checkpoint;
}

function classifyFormat(result) {
  const timelineAtN = checkpointByLabel(result.timeline, 'after-2raf');
  const fullAfter = checkpointByLabel(result.fullLayer, 'after-full-layer');
  const flushAfter = checkpointByLabel(result.flush, 'after-explicit-flush');
  const comparisons = {
    timelineVsFlush: comparePng(
      timelineAtN.screenshot.filePath,
      flushAfter.screenshot.filePath,
      path.join(OUTPUT_ROOT, result.format, 'diff', 'timeline-vs-flush.png'),
    ),
    fullLayerVsFlush: comparePng(
      fullAfter.screenshot.filePath,
      flushAfter.screenshot.filePath,
      path.join(OUTPUT_ROOT, result.format, 'diff', 'full-layer-vs-flush.png'),
    ),
  };

  const modelLatest = timelineAtN.model.text.endsWith('1'.repeat(result.transitionAt));
  const linesLatest = timelineAtN.model.lineCount === 5;
  const treeMatchesFlush = timelineAtN.layerTree.targetText === flushAfter.layerTree.targetText;
  const layoutMatchesFlush = timelineAtN.pageTextLayout.targetText === flushAfter.pageTextLayout.targetText;
  const pixelStale = comparisons.timelineVsFlush.comparable
    && comparisons.timelineVsFlush.changedPixelCount > 0;
  const fullLayerRecovers = comparisons.fullLayerVsFlush.comparable
    && comparisons.fullLayerVsFlush.changedPixelCount === 0;

  let cause = 'existing-green';
  if (!modelLatest || !linesLatest) cause = 'document-core-mutation-or-reflow';
  else if (pixelStale && fullLayerRecovers) cause = 'static-reuse-or-page-local-render-context';
  else if (pixelStale && !layoutMatchesFlush && !fullLayerRecovers) cause = 'stale-layout-input';
  else if (pixelStale && !treeMatchesFlush && !fullLayerRecovers) cause = 'stale-cached-page-tree';
  else if (pixelStale && !fullLayerRecovers) cause = 'explicit-pagination-only-or-multiple-boundaries';

  return {
    verdict: pixelStale ? 'RED' : 'GREEN',
    cause,
    modelLatest,
    linesLatest,
    treeMatchesFlush,
    layoutMatchesFlush,
    fullLayerRecovers,
    comparisons,
  };
}

async function runFormat(page, format, bytes, discoveryRuns) {
  console.log(`\n[${format.toUpperCase()}] keyboard transition discovery (${discoveryRuns} runs)`);
  const discoveries = [];
  for (let run = 1; run <= discoveryRuns; run += 1) {
    const discovery = await discoverTransition(page, format, bytes, run);
    discoveries.push(discovery);
    console.log(`  run ${run}: N=${discovery.transitionAt}, boundary=${discovery.boundaryHandlerMs.toFixed(2)}ms`);
  }
  const transitionValues = new Set(discoveries.map((value) => value.transitionAt));
  assert.equal(transitionValues.size, 1, `${format}: keyboard transition N is not deterministic`);
  const transitionAt = discoveries[0].transitionAt;

  console.log(`  timeline at N=${transitionAt}`);
  const timeline = await runTimeline(page, format, bytes, transitionAt);
  console.log('  independent page-only full-layer control');
  const fullLayer = await runFullLayerControl(page, format, bytes, transitionAt);
  console.log('  independent explicit-pagination control');
  const flush = await runFlushControl(page, format, bytes, transitionAt);

  const result = {
    format,
    transitionAt,
    discoveries,
    timeline,
    fullLayer,
    flush,
  };
  result.classification = classifyFormat(result);
  writeJson(path.join(OUTPUT_ROOT, `${format}-diagnostic.json`), result);
  console.log(`  verdict=${result.classification.verdict}, cause=${result.classification.cause}`);
  return result;
}

function assertApprox(actual, expected, label, tolerance = 0.2) {
  assert.equal(typeof actual, 'number', `${label}: expected a number`);
  assert.ok(
    Math.abs(actual - expected) <= tolerance,
    `${label}: expected ${expected} +/- ${tolerance}, got ${actual}`,
  );
}

async function readFocusedSnapshot(page) {
  return page.evaluate((target) => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    const position = input.cursor.getPosition();
    const rect = input.cursor.getRect();
    const length = wasm.getCellParagraphLength(
      target.sectionIndex,
      target.parentParaIndex,
      target.controlIndex,
      target.cellIndex,
      target.cellParaIndex,
    );
    const text = wasm.getTextInCell(
      target.sectionIndex,
      target.parentParaIndex,
      target.controlIndex,
      target.cellIndex,
      target.cellParaIndex,
      0,
      length,
    );
    const lineInfo = wasm.getLineInfoInCell(
      target.sectionIndex,
      target.parentParaIndex,
      target.controlIndex,
      target.cellIndex,
      target.cellParaIndex,
      position.charOffset,
    );
    return {
      model: { length, text, lineCount: lineInfo.lineCount },
      cursor: { position, rect },
      pagination: {
        pending: input.hasDeferredPaginationPending(),
        pageCount: wasm.pageCount,
      },
    };
  }, TARGET);
}

function assertFocusedSnapshot(format, runNumber, inserted, snapshot, expectedText) {
  const prefix = `${format} run ${runNumber} input ${inserted}`;
  const expectedLength = TARGET.charOffset + inserted;
  assert.equal(snapshot.model.length, expectedLength, `${prefix}: model length`);
  assert.equal(snapshot.model.text, expectedText, `${prefix}: model text`);
  assert.equal(snapshot.model.lineCount, inserted < 44 ? 4 : 5, `${prefix}: line count`);
  assert.equal(snapshot.pagination.pageCount, 115, `${prefix}: page count`);

  const position = snapshot.cursor.position;
  assert.equal(position.sectionIndex, TARGET.sectionIndex, `${prefix}: cursor section`);
  assert.equal(position.parentParaIndex, TARGET.parentParaIndex, `${prefix}: cursor parent paragraph`);
  assert.equal(position.controlIndex, TARGET.controlIndex, `${prefix}: cursor control`);
  assert.equal(position.cellIndex, TARGET.cellIndex, `${prefix}: cursor cell`);
  assert.equal(position.cellParaIndex, TARGET.cellParaIndex, `${prefix}: cursor cell paragraph`);
  assert.equal(position.charOffset, expectedLength, `${prefix}: cursor offset`);
  assert.deepEqual(position.cellPath, TARGET.cellPath, `${prefix}: cursor cell path`);

  const rect = snapshot.cursor.rect;
  assert.equal(rect?.pageIndex, 0, `${prefix}: cursor page`);
  assert.equal(rect?.cellOverflowed, false, `${prefix}: cursor must not use overflow fallback`);
  assertApprox(rect?.cellBounds?.h, 945.9, `${prefix}: cell bounds`);
  assert.equal(
    snapshot.pagination.pending,
    inserted !== 44,
    `${prefix}: deferred pagination pending state`,
  );
}

function targetRunStart(op, label) {
  const match = op.stableSourceKey?.match(/\/char:(\d+)(?:\/|$)/);
  assert.ok(match, `${label}: target TextRun stableSourceKey lacks char offset`);
  return Number(match[1]);
}

function assertExactFocusedState(
  format,
  runNumber,
  state,
  expectedText,
  expectedLineCount,
  expectedBoundsHeight,
  expectedPending,
) {
  const prefix = `${format} run ${runNumber} ${state.label}`;
  const expectedLength = expectedText.length;
  assert.equal(state.model.length, expectedLength, `${prefix}: model length`);
  assert.equal(state.model.text, expectedText, `${prefix}: model text`);
  assert.equal(state.model.lineCount, expectedLineCount, `${prefix}: line count`);
  assert.equal(state.pagination.pageCount, 115, `${prefix}: page count`);
  assert.equal(state.pagination.pending, expectedPending, `${prefix}: pending state`);
  assert.equal(state.layerTree.pageIndex, 0, `${prefix}: tree page`);
  assert.equal(state.layerTree.targetText, expectedText, `${prefix}: layer tree text`);
  assert.equal(state.pageTextLayout.error, null, `${prefix}: page text layout error`);
  assert.equal(state.pageTextLayout.targetText, expectedText, `${prefix}: page text layout text`);

  const ops = state.layerTree.targetOps;
  assert.ok(ops.length > 0, `${prefix}: target TextRuns`);
  let contiguousEnd = 0;
  for (const [index, op] of ops.entries()) {
    const start = targetRunStart(op, `${prefix} op ${index}`);
    assert.equal(start, contiguousEnd, `${prefix}: TextRun ${index} must be contiguous`);
    const textLength = String(op.text ?? '').length;
    assert.ok(textLength > 0, `${prefix}: TextRun ${index} must advance`);
    contiguousEnd = start + textLength;
  }
  assert.equal(contiguousEnd, expectedLength, `${prefix}: layer tree UTF-16 end`);

  const position = state.cursor.position;
  assert.equal(position.charOffset, expectedLength, `${prefix}: cursor offset`);
  assert.equal(position.sectionIndex, TARGET.sectionIndex, `${prefix}: cursor section`);
  assert.equal(position.parentParaIndex, TARGET.parentParaIndex, `${prefix}: cursor parent paragraph`);
  assert.equal(position.controlIndex, TARGET.controlIndex, `${prefix}: cursor control`);
  assert.equal(position.cellIndex, TARGET.cellIndex, `${prefix}: cursor cell`);
  assert.equal(position.cellParaIndex, TARGET.cellParaIndex, `${prefix}: cursor cell paragraph`);
  assert.deepEqual(position.cellPath, TARGET.cellPath, `${prefix}: cursor cell path`);

  const rect = state.cursor.rect;
  const lastOp = ops.at(-1);
  assert.equal(rect?.pageIndex, 0, `${prefix}: cursor page`);
  assert.equal(rect?.cellOverflowed, false, `${prefix}: cursor overflow fallback`);
  assertApprox(rect?.cellBounds?.h, expectedBoundsHeight, `${prefix}: cell bounds`);
  assertApprox(rect?.x, lastOp.bbox.x + lastOp.bbox.width, `${prefix}: cursor x at tree end`);
  assertApprox(rect?.y, lastOp.bbox.y, `${prefix}: cursor y at tree end`);
  assertApprox(rect?.height, lastOp.bbox.height, `${prefix}: cursor height`);

  assert.ok(state.caret, `${prefix}: DOM caret`);
  assert.equal(state.caret.display, 'block', `${prefix}: DOM caret display`);
  assert.ok(state.caret.clientRect?.width > 0, `${prefix}: DOM caret width`);
  assertApprox(state.caret.clientRect?.height, rect.height, `${prefix}: DOM caret height`);
}

function assertFocusedTrace(format, runNumber, trace) {
  const prefix = `${format} run ${runNumber}`;
  const inserts = trace.events.filter((event) => event.type === 'wasm.insertTextInCellDeferredPagination');
  const effects = trace.events.filter((event) => event.type === 'InputHandler.prepareTextMutationBeforeCursor');
  const flushes = trace.events.filter((event) => event.type === 'wasm.flushDeferredPagination');
  const inputFlushes = trace.events.filter((event) => event.type === 'input.flushDeferredPaginationIfNeeded');
  const cursorQueries = trace.events.filter((event) => event.type === 'wasm.getCursorRectByPathNear');
  const operations = trace.events.filter((event) => event.type === 'InputHandler.executeOperation');

  assert.equal(inserts.length, 50, `${prefix}: deferred insert count`);
  assert.equal(effects.length, 50, `${prefix}: consumed mutation effect count`);
  assert.equal(flushes.length, 1, `${prefix}: WASM flush count`);
  assert.equal(inputFlushes.length, 1, `${prefix}: input flush count`);
  assert.equal(operations.length, 50, `${prefix}: operation count`);

  for (let index = 0; index < inserts.length; index += 1) {
    const inserted = index + 1;
    const expectedFlowChange = inserted === 44;
    assert.equal(inserts[index].charOffset, TARGET.charOffset + index, `${prefix}: input ${inserted} source offset`);
    assert.equal(inserts[index].resultCharOffset, TARGET.charOffset + inserted, `${prefix}: input ${inserted} result offset`);
    assert.equal(inserts[index].paginationDeferred, true, `${prefix}: input ${inserted} deferred result`);
    assert.equal(inserts[index].cellFlowChanged, expectedFlowChange, `${prefix}: input ${inserted} flow result`);
    assert.equal(effects[index].deferredPagination, true, `${prefix}: input ${inserted} consumed deferred effect`);
    assert.equal(effects[index].cellFlowChanged, expectedFlowChange, `${prefix}: input ${inserted} consumed flow effect`);
    assert.equal(effects[index].paginationCompleted, false, `${prefix}: input ${inserted} deferred completion state`);
  }

  const boundaryInsert = inserts[43];
  const boundaryFlush = flushes[0];
  const firstCursorAfterBoundary = cursorQueries.find((event) => event.sequence > boundaryInsert.sequence);
  assert.ok(firstCursorAfterBoundary, `${prefix}: cursor query after boundary`);
  assert.ok(
    boundaryInsert.sequence < boundaryFlush.sequence,
    `${prefix}: mutation result must precede boundary flush`,
  );
  assert.ok(
    boundaryFlush.sequence < firstCursorAfterBoundary.sequence,
    `${prefix}: boundary flush must precede cursor query`,
  );

  const operationDurations = operations.map((event) => event.durationMs);
  const stableOperationDurations = operationDurations.filter((_, index) => index !== 43);
  const stableKeyboardDurations = trace.keyboardDurationsMs.filter((_, index) => index !== 43);
  const cursorDurations = cursorQueries.map((event) => event.durationMs);
  return {
    counts: trace.counts,
    ordering: {
      boundaryMutationSequence: boundaryInsert.sequence,
      boundaryFlushSequence: boundaryFlush.sequence,
      boundaryCursorSequence: firstCursorAfterBoundary.sequence,
    },
    timing: {
      keyboardStable: summarizeDurations(stableKeyboardDurations),
      keyboardBoundaryMs: trace.keyboardDurationsMs[43],
      operationStable: summarizeDurations(stableOperationDurations),
      operationBoundaryMs: operationDurations[43],
      boundaryFlushMs: boundaryFlush.durationMs,
      cursorQueries: summarizeDurations(cursorDurations),
      boundaryCursorMs: firstCursorAfterBoundary.durationMs,
    },
  };
}

async function runFocusedFormat(page, format, bytes, runNumber) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToTarget(page);
  const timelineStart = await page.evaluate(() => performance.now());
  const initial = await collectState(page, 'initial-warm', timelineStart);
  const initialText = initial.model.text;
  assert.equal(initialText.length, TARGET.charOffset, `${format} run ${runNumber}: initial text length`);
  assertExactFocusedState(format, runNumber, initial, initialText, 4, 945.9, false);
  const clip = await resolveCompositedClip(page);
  const visualDirectory = path.join(OUTPUT_ROOT, 'focused', format, `run-${runNumber}`);
  const visual = { beforeBoundary: null, boundary: [], comparisons: [] };

  await installTrace(page);
  const keyboardDurationsMs = [];
  const checkpoints = { initial };
  for (let inserted = 1; inserted <= 50; inserted += 1) {
    const startedAt = performance.now();
    await page.keyboard.type('1');
    keyboardDurationsMs.push(performance.now() - startedAt);

    const expectedText = `${initialText}${'1'.repeat(inserted)}`;
    const snapshot = await readFocusedSnapshot(page);
    assertFocusedSnapshot(format, runNumber, inserted, snapshot, expectedText);

    if (inserted === 43) {
      const traceAt43 = await collectTrace(page);
      assert.equal(traceAt43.counts.wasmFlush, 0, `${format} run ${runNumber}: inputs 1-43 flush count`);
      await waitTwoRafs(page);
      checkpoints.at43 = await collectState(page, 'after-43-2raf', timelineStart);
      assertExactFocusedState(format, runNumber, checkpoints.at43, expectedText, 4, 945.9, true);
      visual.beforeBoundary = await captureCompositedCrop(
        page,
        path.join(visualDirectory, 'after-43.png'),
        clip,
      );
    }

    if (inserted === 44) {
      const boundaryStates = [];
      boundaryStates.push(await collectState(page, 'after-44-sync', timelineStart));
      await waitTwoRafs(page);
      boundaryStates.push(await collectState(page, 'after-44-2raf', timelineStart));
      visual.boundary.push(await captureCompositedCrop(
        page,
        path.join(visualDirectory, 'after-44-2raf.png'),
        clip,
      ));
      await delay(page, 100);
      boundaryStates.push(await collectState(page, 'after-44-100ms', timelineStart));
      visual.boundary.push(await captureCompositedCrop(
        page,
        path.join(visualDirectory, 'after-44-100ms.png'),
        clip,
      ));
      await delay(page, 750);
      boundaryStates.push(await collectState(page, 'after-44-850ms', timelineStart));
      visual.boundary.push(await captureCompositedCrop(
        page,
        path.join(visualDirectory, 'after-44-850ms.png'),
        clip,
      ));
      await delay(page, 750);
      boundaryStates.push(await collectState(page, 'after-44-1600ms', timelineStart));
      visual.boundary.push(await captureCompositedCrop(
        page,
        path.join(visualDirectory, 'after-44-1600ms.png'),
        clip,
      ));
      for (const state of boundaryStates) {
        assertExactFocusedState(format, runNumber, state, expectedText, 5, 945.9, false);
        const traceAtCheckpoint = await collectTrace(page);
        assert.equal(
          traceAtCheckpoint.counts.wasmFlush,
          1,
          `${format} run ${runNumber} ${state.label}: cumulative flush count`,
        );
      }
      const transitionComparison = comparePng(
        visual.boundary[0].filePath,
        visual.beforeBoundary.filePath,
        path.join(visualDirectory, 'diff-43-vs-44.png'),
      );
      assert.equal(transitionComparison.comparable, true, `${format} run ${runNumber}: transition crop comparable`);
      assert.ok(transitionComparison.changedPixelCount > 0, `${format} run ${runNumber}: boundary crop must change`);
      visual.comparisons.push({ label: '43-vs-44', ...transitionComparison });
      for (let index = 1; index < visual.boundary.length; index += 1) {
        const comparison = comparePng(
          visual.boundary[index].filePath,
          visual.boundary[0].filePath,
          path.join(visualDirectory, `diff-44-${index}.png`),
        );
        assert.equal(comparison.comparable, true, `${format} run ${runNumber}: stable crop ${index} comparable`);
        assert.equal(comparison.changedPixelCount, 0, `${format} run ${runNumber}: boundary crop ${index} stable`);
        assert.equal(
          visual.boundary[index].sha256,
          visual.boundary[0].sha256,
          `${format} run ${runNumber}: boundary crop ${index} exact hash`,
        );
        visual.comparisons.push({ label: `44-stable-${index}`, ...comparison });
      }
      checkpoints.at44 = boundaryStates;
    }
  }

  await waitTwoRafs(page);
  const finalText = `${initialText}${'1'.repeat(50)}`;
  checkpoints.at50 = await collectState(page, 'after-50-2raf', timelineStart);
  assertExactFocusedState(format, runNumber, checkpoints.at50, finalText, 5, 945.9, true);

  const trace = await collectTrace(page);
  trace.keyboardDurationsMs = keyboardDurationsMs;
  assert.equal(trace.counts.wasmFlush, 1, `${format} run ${runNumber}: inputs 45-50 add no flush`);
  const traceSummary = assertFocusedTrace(format, runNumber, trace);
  console.log(
    `  run ${runNumber}: GREEN, flush=1, boundary=${traceSummary.timing.operationBoundaryMs.toFixed(2)}ms, `
      + `flushTime=${traceSummary.timing.boundaryFlushMs.toFixed(2)}ms, `
      + `stableP95=${traceSummary.timing.operationStable.p95Ms.toFixed(2)}ms`,
  );

  return {
    format,
    runNumber,
    load,
    transitionAt: 44,
    pageCount: checkpoints.at50.pagination.pageCount,
    finalLength: checkpoints.at50.model.length,
    trace: traceSummary,
    visual,
  };
}

function assertRawBoundaryTrace(format, kind, trace) {
  const prefix = `${format} ${kind} raw boundary`;
  const inserts = trace.events.filter((event) => event.type === 'wasm.insertTextInCellDeferredPagination');
  const effects = trace.events.filter((event) => event.type === 'InputHandler.prepareTextMutationBeforeCursor');
  const flushes = trace.events.filter((event) => event.type === 'wasm.flushDeferredPagination');
  const cursorQueries = trace.events.filter((event) => event.type === 'wasm.getCursorRectByPathNear');

  assert.equal(inserts.length, 1, `${prefix}: deferred insert count`);
  assert.equal(inserts[0].charOffset, TARGET.charOffset + 43, `${prefix}: source offset`);
  assert.equal(inserts[0].resultCharOffset, TARGET.charOffset + 44, `${prefix}: result offset`);
  assert.equal(inserts[0].paginationDeferred, true, `${prefix}: deferred result`);
  assert.equal(inserts[0].cellFlowChanged, true, `${prefix}: flow boundary result`);
  assert.equal(effects.length, 1, `${prefix}: consumed effect count`);
  assert.equal(effects[0].deferredPagination, true, `${prefix}: consumed deferred effect`);
  assert.equal(effects[0].cellFlowChanged, true, `${prefix}: consumed flow effect`);
  assert.equal(effects[0].paginationCompleted, false, `${prefix}: deferred completion state`);
  assert.equal(flushes.length, 1, `${prefix}: cumulative flush count`);

  const firstCursorQuery = cursorQueries.find((event) => event.sequence > inserts[0].sequence);
  assert.ok(firstCursorQuery, `${prefix}: cursor query after mutation`);
  assert.ok(inserts[0].sequence < flushes[0].sequence, `${prefix}: mutation must precede flush`);
  assert.ok(flushes[0].sequence < firstCursorQuery.sequence, `${prefix}: flush must precede cursor query`);
}

function assertRawStableTrace(format, kind, trace) {
  const prefix = `${format} ${kind} raw stable`;
  const inserts = trace.events.filter((event) => event.type === 'wasm.insertTextInCellDeferredPagination');
  const effects = trace.events.filter((event) => event.type === 'InputHandler.prepareTextMutationBeforeCursor');
  const flushes = trace.events.filter((event) => event.type === 'wasm.flushDeferredPagination');
  const cursorQueries = trace.events.filter((event) => event.type === 'wasm.getCursorRectByPathNear');

  assert.equal(inserts.length, 1, `${prefix}: deferred insert count`);
  assert.equal(inserts[0].charOffset, TARGET.charOffset, `${prefix}: source offset`);
  assert.equal(inserts[0].resultCharOffset, TARGET.charOffset + 1, `${prefix}: result offset`);
  assert.equal(inserts[0].paginationDeferred, true, `${prefix}: deferred result`);
  assert.equal(inserts[0].cellFlowChanged, false, `${prefix}: stable flow result`);
  assert.equal(effects.length, 1, `${prefix}: consumed effect count`);
  assert.equal(effects[0].deferredPagination, true, `${prefix}: consumed deferred effect`);
  assert.equal(effects[0].cellFlowChanged, false, `${prefix}: consumed stable effect`);
  assert.equal(effects[0].paginationCompleted, false, `${prefix}: deferred completion state`);
  assert.equal(flushes.length, 0, `${prefix}: stable input flush count`);
  assert.ok(
    cursorQueries.some((event) => event.sequence > inserts[0].sequence),
    `${prefix}: cursor query after mutation`,
  );
}

async function dispatchRawInput(page, kind, text) {
  await page.evaluate(({ rawKind, rawText }) => {
    const input = window.__inputHandler;
    const textarea = input.textarea;
    if (rawKind === 'ime') {
      textarea.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true, data: '' }));
      textarea.value = rawText;
      textarea.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        data: rawText,
        inputType: 'insertCompositionText',
        isComposing: true,
      }));
      textarea.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: rawText }));
      return;
    }

    input._isIOS = true;
    textarea.value = rawText;
    textarea.dispatchEvent(new InputEvent('input', {
      bubbles: true,
      data: rawText,
      inputType: 'insertText',
    }));
  }, { rawKind: kind, rawText: text });
}

async function runRawStableSmoke(page, format, bytes, kind) {
  await restoreTrace(page);
  await openDocumentThroughApp(page, format, bytes);
  await moveToTarget(page);
  const initial = await readFocusedSnapshot(page);
  const initialText = initial.model.text;
  assert.equal(initialText.length, TARGET.charOffset, `${format} ${kind} stable: initial text length`);

  await installTrace(page);
  await dispatchRawInput(page, kind, '1');
  await delay(page, 150);
  await waitTwoRafs(page);

  const timelineStart = await page.evaluate(() => performance.now());
  const finalState = await collectState(page, `${kind}-raw-stable-after-150ms`, timelineStart);
  assertExactFocusedState(
    format,
    `${kind}-raw-stable`,
    finalState,
    `${initialText}1`,
    4,
    945.9,
    true,
  );
  const trace = await collectTrace(page);
  assertRawStableTrace(format, kind, trace);

  if (kind === 'ios') {
    await page.evaluate(() => {
      window.__inputHandler._isIOS = false;
    });
  }
  await restoreTrace(page);
  console.log(`  ${kind} raw stable smoke: GREEN, flush=0`);
  return {
    format,
    kind,
    variant: 'stable',
    finalLength: finalState.model.length,
    pageCount: finalState.pagination.pageCount,
    flushCount: trace.counts.wasmFlush,
  };
}

async function runRawBoundarySmoke(page, format, bytes, kind) {
  await restoreTrace(page);
  await openDocumentThroughApp(page, format, bytes);
  await moveToTarget(page);
  const initial = await readFocusedSnapshot(page);
  const initialText = initial.model.text;
  assert.equal(initialText.length, TARGET.charOffset, `${format} ${kind}: initial text length`);

  await typeKeyboardOnes(page, 43);
  const beforeBoundary = await readFocusedSnapshot(page);
  assertFocusedSnapshot(
    format,
    `${kind}-raw`,
    43,
    beforeBoundary,
    `${initialText}${'1'.repeat(43)}`,
  );

  await installTrace(page);
  await dispatchRawInput(page, kind, '1');

  await delay(page, 150);
  await waitTwoRafs(page);
  const finalText = `${initialText}${'1'.repeat(44)}`;
  const timelineStart = await page.evaluate(() => performance.now());
  const finalState = await collectState(page, `${kind}-raw-after-150ms`, timelineStart);
  assertExactFocusedState(
    format,
    `${kind}-raw`,
    finalState,
    finalText,
    5,
    945.9,
    false,
  );
  const trace = await collectTrace(page);
  assertRawBoundaryTrace(format, kind, trace);

  if (kind === 'ios') {
    await page.evaluate(() => {
      window.__inputHandler._isIOS = false;
    });
  }
  await restoreTrace(page);
  console.log(`  ${kind} raw smoke: GREEN, flush=1`);
  return {
    format,
    kind,
    variant: 'boundary',
    finalLength: finalState.model.length,
    pageCount: finalState.pagination.pageCount,
    flushCount: trace.counts.wasmFlush,
  };
}

async function runFocusedMain() {
  const formats = cliValue('formats', 'hwp,hwpx')
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  const runs = Number(cliValue('runs', '3'));
  assert.ok(Number.isInteger(runs) && runs > 0, '--runs must be a positive integer');
  for (const format of formats) assert.ok(SAMPLES[format], `unsupported format: ${format}`);

  mkdirSync(OUTPUT_ROOT, { recursive: true });
  const fixtures = Object.fromEntries(formats.map((format) => {
    const bytes = readFileSync(SAMPLES[format]);
    return [format, {
      path: SAMPLES[format],
      bytes,
      size: bytes.length,
      sha256: sha256(bytes),
    }];
  }));

  const browser = await launchBrowser();
  const page = await createPage(browser, 1280, 900);
  page.on('console', (message) => {
    if (message.type() === 'error' || message.type() === 'warning') {
      console.log(`  [browser:${message.type()}] ${message.text()}`);
    }
  });
  page.on('pageerror', (error) => console.log(`  [browser:pageerror] ${error.message}`));

  const startedAt = new Date().toISOString();
  const results = [];
  const rawSmokes = [];
  try {
    await loadApp(page);
    for (const format of formats) {
      console.log(`\n[${format.toUpperCase()}] focused GREEN (${runs} runs)`);
      for (let runNumber = 1; runNumber <= runs; runNumber += 1) {
        results.push(await runFocusedFormat(page, format, fixtures[format].bytes, runNumber));
      }
      rawSmokes.push(await runRawStableSmoke(page, format, fixtures[format].bytes, 'ime'));
      rawSmokes.push(await runRawStableSmoke(page, format, fixtures[format].bytes, 'ios'));
      rawSmokes.push(await runRawBoundarySmoke(page, format, fixtures[format].bytes, 'ime'));
      rawSmokes.push(await runRawBoundarySmoke(page, format, fixtures[format].bytes, 'ios'));
    }
  } finally {
    await restoreTrace(page).catch(() => {});
    await closePage(page).catch(() => {});
    await closeBrowser(browser).catch(() => {});
  }

  const summary = {
    issue: 2214,
    mode: 'focused-green',
    startedAt,
    finishedAt: new Date().toISOString(),
    environment: {
      viteUrl: process.env.VITE_URL ?? 'http://localhost:7700',
      chromePath: process.env.CHROME_PATH ?? process.env.PUPPETEER_EXECUTABLE_PATH ?? null,
      viewport: { width: 1280, height: 900, deviceScaleFactor: 1 },
      runs,
    },
    fixtures: Object.fromEntries(Object.entries(fixtures).map(([format, value]) => [format, {
      path: value.path,
      size: value.size,
      sha256: value.sha256,
    }])),
    results,
    rawSmokes,
  };
  writeJson(path.join(OUTPUT_ROOT, 'focused-summary.json'), summary);
  console.log(`\nIssue #2214 focused GREEN written to ${OUTPUT_ROOT}`);
}

async function runDiagnosticMain() {
  assert.ok(process.argv.includes('--diagnose'), 'Stage 1 runner requires --diagnose');
  const formats = cliValue('formats', 'hwp,hwpx')
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  const discoveryRuns = Number(cliValue('runs', '3'));
  assert.ok(Number.isInteger(discoveryRuns) && discoveryRuns > 0, '--runs must be a positive integer');
  for (const format of formats) assert.ok(SAMPLES[format], `unsupported format: ${format}`);

  mkdirSync(OUTPUT_ROOT, { recursive: true });
  const fixtures = Object.fromEntries(formats.map((format) => {
    const bytes = readFileSync(SAMPLES[format]);
    return [format, {
      path: SAMPLES[format],
      bytes,
      size: bytes.length,
      sha256: sha256(bytes),
    }];
  }));

  const browser = await launchBrowser();
  const page = await createPage(browser, 1280, 900);
  page.on('console', (message) => {
    if (message.type() === 'error' || message.type() === 'warning') {
      console.log(`  [browser:${message.type()}] ${message.text()}`);
    }
  });
  page.on('pageerror', (error) => console.log(`  [browser:pageerror] ${error.message}`));

  const startedAt = new Date().toISOString();
  const results = [];
  try {
    await loadApp(page);
    for (const format of formats) {
      results.push(await runFormat(page, format, fixtures[format].bytes, discoveryRuns));
    }
  } finally {
    await restoreTrace(page).catch(() => {});
    await closePage(page).catch(() => {});
    await closeBrowser(browser).catch(() => {});
  }

  const summary = {
    issue: 2214,
    mode: 'diagnose',
    startedAt,
    finishedAt: new Date().toISOString(),
    environment: {
      viteUrl: process.env.VITE_URL ?? 'http://localhost:7700',
      chromePath: process.env.CHROME_PATH ?? process.env.PUPPETEER_EXECUTABLE_PATH ?? null,
      viewport: { width: 1280, height: 900, deviceScaleFactor: 1 },
      discoveryRuns,
    },
    fixtures: Object.fromEntries(Object.entries(fixtures).map(([format, value]) => [format, {
      path: value.path,
      size: value.size,
      sha256: value.sha256,
    }])),
    formats: results.map((result) => ({
      format: result.format,
      transitionAt: result.transitionAt,
      discoveryNs: result.discoveries.map((value) => value.transitionAt),
      classification: result.classification,
    })),
  };
  writeJson(path.join(OUTPUT_ROOT, 'summary.json'), summary);
  console.log(`\nIssue #2214 diagnostic written to ${OUTPUT_ROOT}`);
  console.log(JSON.stringify(summary.formats, null, 2));
}

const selectedMain = process.argv.includes('--diagnose') ? runDiagnosticMain : runFocusedMain;
selectedMain().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
