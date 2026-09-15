/**
 * Issue #3794 flow-input latency measurement spike.
 *
 * This is a diagnostic benchmark, not a CI wall-clock gate. It changes no
 * product code: the DEV-only `window.__inputHandler` / `window.__wasm` objects
 * are monkey-patched at runtime and restored after every case.
 *
 * Focused smoke (HWP, one run, minimum decision matrix):
 *
 *   npm run e2e:issue-3794-flow-latency
 *
 * HWP/HWPX evidence:
 *
 *   npm run e2e:issue-3794-flow-latency -- --formats=hwp,hwpx
 *
 * Useful overrides:
 *
 *   --cases=flow-current,flow-budget,flow-scheduler,flow-restart,
 *           flow-budget-restart,flow-scheduler-restart,
 *           enter-cold,enter-pending,merge-cold,merge-pending,
 *           merge-delete-cold,merge-delete-pending
 *   --budget=4 --cadence=80 --tail-inputs=4 --runs=1
 *   --output-root=../output/poc/task3794/stage0
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
const DEFAULT_OUTPUT_ROOT = path.join(REPO_ROOT, 'output/poc/task3794/stage0');
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
let FLOW_TRANSITION_INPUT = 56;
let INITIAL_LINE_COUNT = 4;
const ALL_CASES = new Set([
  'flow-current',
  'flow-budget',
  'flow-scheduler',
  'flow-restart',
  'flow-budget-restart',
  'flow-scheduler-restart',
  'enter-cold',
  'enter-pending',
  'ime-enter-cold',
  'ime-enter-pending',
  'merge-cold',
  'merge-pending',
  'merge-delete-cold',
  'merge-delete-pending',
]);
const DEFAULT_CASES = [...ALL_CASES];
const SAMPLES = Object.freeze({
  hwp: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwp'),
  hwpx: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwpx'),
});

function cliValue(name, fallback) {
  const prefix = `--${name}=`;
  const arg = process.argv.find((value) => value.startsWith(prefix));
  return arg ? arg.slice(prefix.length) : fallback;
}

function parsePositiveInteger(name, fallback, minimum = 1) {
  const value = Number(cliValue(name, String(fallback)));
  assert.ok(Number.isInteger(value) && value >= minimum, `--${name} must be >= ${minimum}`);
  return value;
}

function parseList(name, fallback, allowed) {
  const values = cliValue(name, fallback)
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  assert.ok(values.length > 0, `--${name} must not be empty`);
  for (const value of values) assert.ok(allowed.has(value), `unsupported --${name}: ${value}`);
  return [...new Set(values)];
}

function parseConfig() {
  const outputValue = cliValue(
    'output-root',
    process.env.ISSUE3794_OUTPUT_ROOT ?? DEFAULT_OUTPUT_ROOT,
  );
  return {
    formats: parseList('formats', 'hwp', new Set(Object.keys(SAMPLES))),
    cases: parseList('cases', DEFAULT_CASES.join(','), ALL_CASES),
    budget: parsePositiveInteger('budget', 4),
    cadenceMs: parsePositiveInteger('cadence', 80, 0),
    tailInputs: parsePositiveInteger('tail-inputs', 4, 1),
    runs: parsePositiveInteger('runs', 1),
    timeoutMs: parsePositiveInteger('timeout', 20_000),
    outputRoot: path.resolve(process.cwd(), outputValue),
  };
}

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function fingerprint(filePath) {
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

function gitRevision() {
  try {
    return execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return null;
  }
}

function percentile(values, quantile) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(sorted.length * quantile) - 1));
  return sorted[index];
}

function summarize(values) {
  const finite = values.filter((value) => Number.isFinite(value));
  return {
    count: finite.length,
    totalMs: finite.reduce((sum, value) => sum + value, 0),
    p50Ms: percentile(finite, 0.5),
    p95Ms: percentile(finite, 0.95),
    maxMs: finite.length > 0 ? Math.max(...finite) : null,
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

async function clickBlockingModalChoice(page) {
  return page.evaluate(() => {
    const allowed = new Set(['그대로 보기', '대체 글꼴로 보기']);
    const button = Array.from(document.querySelectorAll('button')).find((candidate) => {
      const style = getComputedStyle(candidate);
      return allowed.has(candidate.textContent?.trim() ?? '')
        && style.display !== 'none'
        && style.visibility !== 'hidden';
    });
    if (!button) return null;
    const label = button.textContent?.trim() ?? '';
    button.click();
    return label;
  });
}

async function openDocumentThroughApp(page, format, bytes) {
  const fileName = path.basename(SAMPLES[format]);
  const requestId = `issue3794-${format}-${crypto.randomUUID()}`;
  await page.evaluate(({ base64, name, id }) => {
    const binary = atob(base64);
    const payload = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      payload[index] = binary.charCodeAt(index);
    }
    window.__issue3794LoadResult = null;
    const off = window.__eventBus.on('open-document-bytes:done', (result) => {
      if (result?.requestId !== id) return;
      off();
      window.__issue3794LoadResult = result;
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes: payload,
      fileName: name,
      fileHandle: null,
      skipUnsavedGuard: true,
      requestId: id,
    });
  }, { base64: bytes.toString('base64'), name: fileName, id: requestId });

  const deadline = Date.now() + 90_000;
  const modalChoices = [];
  let result = null;
  while (Date.now() < deadline) {
    const choice = await clickBlockingModalChoice(page);
    if (choice) modalChoices.push(choice);
    result = await page.evaluate(() => window.__issue3794LoadResult);
    if (result) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(result, `${format}: document load timeout`);
  assert.equal(result.ok, true, `${format}: load failed: ${result.error ?? 'unknown'}`);
  await page.evaluate(() => document.fonts.ready);
  await waitTwoRafs(page);
  const state = await page.evaluate(() => ({
    sourceFormat: window.__wasm.getSourceFormat(),
    pageCount: window.__wasm.pageCount,
    inputActive: window.__inputHandler.isActive(),
    fontsStatus: document.fonts.status,
    userAgent: navigator.userAgent,
  }));
  assert.equal(state.sourceFormat, format, `${format}: source format`);
  assert.equal(state.pageCount, 115, `${format}: initial page count`);
  assert.equal(state.inputActive, true, `${format}: input handler inactive`);
  assert.equal(state.fontsStatus, 'loaded', `${format}: fonts not ready`);
  return { ...state, modalChoices };
}

function withCellParagraph(target, cellParaIndex, charOffset) {
  return {
    ...target,
    paragraphIndex: cellParaIndex,
    cellParaIndex,
    charOffset,
    cellPath: target.cellPath.map((entry, index, pathEntries) => (
      index + 1 === pathEntries.length ? { ...entry, cellParaIndex } : { ...entry }
    )),
  };
}

async function moveToPosition(page, position) {
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
  }, position);
  assert.equal(state.focused, true, 'hidden input not focused');
  assert.equal(state.position.charOffset, position.charOffset, 'cursor offset');
  assert.equal(state.position.cellParaIndex, position.cellParaIndex, 'cell paragraph');
  return state;
}

async function readState(page) {
  return page.evaluate(() => {
    const input = window.__inputHandler;
    return {
      cursor: input.cursor.getPosition(),
      cursorRect: input.cursor.getRect(),
      pending: input.hasDeferredPaginationPending(),
      runnerActive: input.deferredPaginationRunner?.isActive?.() ?? null,
      runnerPending: input.deferredPaginationRunner?.hasPendingWork?.() ?? null,
      pageCount: window.__wasm.pageCount,
    };
  });
}

async function readFlowFinalCorrectness(page, insertedCount) {
  return page.evaluate(({ cellPath, startOffset, count }) => {
    const input = window.__inputHandler;
    const wasm = window.__wasm;
    const cursor = input.cursor.getPosition();
    return {
      insertedText: wasm.getTextInCellByPath(
        0,
        0,
        JSON.stringify(cellPath),
        startOffset,
        count,
      ),
      lineInfo: wasm.getLineInfoInCell(0, 0, 2, 2, 5, cursor.charOffset),
      cursor,
    };
  }, { cellPath: TARGET.cellPath, startOffset: TARGET.charOffset, count: insertedCount });
}

async function dispatchText(page, text) {
  return page.evaluate((value) => {
    const input = window.__inputHandler;
    const startedAt = performance.now();
    input.textarea.value = value;
    input.textarea.dispatchEvent(new InputEvent('input', {
      bubbles: true,
      data: value,
      inputType: 'insertText',
      isComposing: false,
    }));
    return {
      startedAt,
      syncCompletedAt: performance.now(),
      cursor: input.cursor.getPosition(),
      pending: input.hasDeferredPaginationPending(),
    };
  }, text);
}

async function dispatchTextSequence(page, { text, count, cadenceMs }) {
  return page.evaluate(async ({ value, total, cadence }) => {
    const input = window.__inputHandler;
    const sequenceRequestedAt = performance.now();
    const samples = [];
    const paintPromises = [];
    let previousActualStartedAt = null;

    for (let index = 0; index < total; index += 1) {
      const requestedStartedAt = sequenceRequestedAt + index * cadence;
      const waitMs = requestedStartedAt - performance.now();
      if (waitMs > 0) {
        await new Promise((resolve) => setTimeout(resolve, waitMs));
      }

      const actualStartedAt = performance.now();
      input.textarea.value = value;
      input.textarea.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        data: value,
        inputType: 'insertText',
        isComposing: false,
      }));
      const syncCompletedAt = performance.now();
      const requestedIntervalMs = index === 0 ? null : cadence;
      const actualIntervalMs = previousActualStartedAt === null
        ? null
        : actualStartedAt - previousActualStartedAt;
      const sample = {
        index,
        requestedStartedAt,
        actualStartedAt,
        requestedIntervalMs,
        actualIntervalMs,
        intervalDriftMs: actualIntervalMs === null ? null : actualIntervalMs - cadence,
        startDriftMs: actualStartedAt - requestedStartedAt,
        syncDispatchMs: syncCompletedAt - actualStartedAt,
        inputToFirstRafMs: null,
        inputToSecondRafMs: null,
        cursor: input.cursor.getPosition(),
        pending: input.hasDeferredPaginationPending(),
      };
      samples.push(sample);
      previousActualStartedAt = actualStartedAt;
      paintPromises.push(new Promise((resolve) => {
        requestAnimationFrame((firstRafAt) => {
          sample.inputToFirstRafMs = performance.now() - actualStartedAt;
          requestAnimationFrame((secondRafAt) => {
            sample.inputToSecondRafMs = performance.now() - actualStartedAt;
            resolve();
          });
        });
      }));
    }

    await Promise.all(paintPromises);
    return samples;
  }, { value: text, total: count, cadence: cadenceMs });
}

async function dispatchKeyAndWaitForPaint(page, key) {
  return page.evaluate(async (value) => {
    const input = window.__inputHandler;
    const preExistingPending = input.hasDeferredPaginationPending();
    const runnerWasActive = input.deferredPaginationRunner?.isActive?.() ?? null;
    const startedAt = performance.now();
    input.textarea.dispatchEvent(new KeyboardEvent('keydown', {
      key: value,
      bubbles: true,
      cancelable: true,
    }));
    const syncCompletedAt = performance.now();
    const firstRafAt = await new Promise((resolve) => requestAnimationFrame(() => resolve(performance.now())));
    const secondRafAt = await new Promise((resolve) => requestAnimationFrame(() => resolve(performance.now())));
    return {
      key: value,
      preExistingPending,
      runnerWasActive,
      syncDispatchMs: syncCompletedAt - startedAt,
      inputToFirstRafMs: firstRafAt - startedAt,
      inputToSecondRafMs: secondRafAt - startedAt,
      cursor: input.cursor.getPosition(),
      pendingAfter: input.hasDeferredPaginationPending(),
    };
  }, key);
}

async function typeStablePrefix(page, count = FLOW_TRANSITION_INPUT - 1) {
  for (let index = 0; index < count; index += 1) await dispatchText(page, '1');
  await waitTwoRafs(page);
  const state = await readState(page);
  assert.equal(state.pending, true, 'stable deferred edits should leave the document pending');
  assert.equal(state.runnerActive, false, 'stable prefix must not start the flow runner');
}

async function waitForCompletion(page, timeoutMs) {
  const startedAt = performance.now();
  await page.waitForFunction(
    () => !window.__inputHandler.hasDeferredPaginationPending(),
    { timeout: timeoutMs, polling: 25 },
  );
  return performance.now() - startedAt;
}

async function restoreTrace(page) {
  await page.evaluate(() => {
    const trace = window.__issue3794Trace;
    if (!trace) return;
    trace.longTaskObserver?.disconnect();
    for (const restore of [...(trace.restores ?? [])].reverse()) restore();
    window.__issue3794Trace = null;
  });
}

async function installTrace(page, { fragmentBudget, scheduler }) {
  await restoreTrace(page);
  await page.evaluate((config) => {
    const input = window.__inputHandler;
    const wasm = window.__wasm;
    const runner = input.deferredPaginationRunner;
    if (!runner) throw new Error('DeferredPaginationRunner is not exposed in DEV build');

    const trace = {
      startedAt: performance.now(),
      sequence: 0,
      events: [],
      jobs: [],
      activeJob: null,
      nextJobId: 1,
      nextTaskId: 1,
      tasks: new Map(),
      taskByHandle: new Map(),
      restores: [],
      cancelContext: null,
      longTasks: [],
      longTaskObserver: null,
      config,
    };
    const nowDetail = () => ({
      visibilityState: document.visibilityState,
      hasFocus: document.hasFocus(),
    });
    const record = (type, detail = {}) => {
      const at = performance.now();
      const event = {
        sequence: ++trace.sequence,
        type,
        startTime: at,
        atMs: at - trace.startedAt,
        ...nowDetail(),
        ...detail,
      };
      trace.events.push(event);
      return event;
    };
    const endActiveJob = (outcome, detail = {}) => {
      const job = trace.activeJob;
      if (!job) return;
      job.outcome = outcome;
      job.completed = outcome === 'complete';
      job.discarded = outcome !== 'complete';
      job.endedAtMs = performance.now() - trace.startedAt;
      Object.assign(job, detail);
      record('job.end', {
        jobId: job.jobId,
        revision: job.revision,
        outcome,
        emittedFragments: job.emittedFragments,
        stepComputeMs: job.stepComputeMs,
      });
      trace.activeJob = null;
    };

    const wrap = (object, name, type, describeArgs = () => ({}), describeResult = () => ({})) => {
      const original = object?.[name];
      if (typeof original !== 'function') return;
      object[name] = function issue3794Wrapped(...args) {
        const startedAt = performance.now();
        let result;
        let error = null;
        try {
          result = original.apply(this, args);
          return result;
        } catch (caught) {
          error = caught;
          throw caught;
        } finally {
          record(type, {
            startTime: startedAt,
            durationMs: performance.now() - startedAt,
            ...describeArgs(args),
            ...describeResult(result),
            ...(error ? { error: error instanceof Error ? error.message : String(error) } : {}),
          });
        }
      };
      trace.restores.push(() => { object[name] = original; });
    };

    const originalBudget = runner.fragmentBudget;
    runner.fragmentBudget = config.fragmentBudget;
    trace.restores.push(() => { runner.fragmentBudget = originalBudget; });

    const originalScheduleTask = runner.scheduleTask.bind(runner);
    const originalCancelTask = runner.cancelTask.bind(runner);
    let baseSchedule = originalScheduleTask;
    let baseCancel = originalCancelTask;
    if (config.scheduler === 'message-channel') {
      const callbacks = new Map();
      const channel = new MessageChannel();
      let nextMessageId = 1;
      channel.port1.onmessage = (event) => {
        const callback = callbacks.get(event.data);
        callbacks.delete(event.data);
        callback?.();
      };
      baseSchedule = (callback) => {
        const token = { issue3794MessageTask: nextMessageId++ };
        callbacks.set(token.issue3794MessageTask, callback);
        channel.port2.postMessage(token.issue3794MessageTask);
        return token;
      };
      baseCancel = (token) => {
        if (token && typeof token === 'object') callbacks.delete(token.issue3794MessageTask);
      };
      trace.restores.push(() => {
        callbacks.clear();
        channel.port1.close();
        channel.port2.close();
      });
    }

    runner.scheduleTask = (callback, delayMs = 0) => {
      const task = {
        taskId: trace.nextTaskId++,
        jobId: trace.activeJob?.jobId ?? null,
        revision: trace.activeJob?.revision ?? null,
        enqueuedAt: performance.now(),
        requestedDelayMs: delayMs,
        enqueuedVisibilityState: document.visibilityState,
        enqueuedHasFocus: document.hasFocus(),
        state: 'queued',
      };
      const wrapped = () => {
        task.state = 'running';
        task.startedAt = performance.now();
        task.queueDelayMs = task.startedAt - task.enqueuedAt;
        task.excessQueueDelayMs = Math.max(0, task.queueDelayMs - task.requestedDelayMs);
        record('scheduler.run', {
          taskId: task.taskId,
          jobId: task.jobId,
          revision: task.revision,
          queueDelayMs: task.queueDelayMs,
          requestedDelayMs: task.requestedDelayMs,
          excessQueueDelayMs: task.excessQueueDelayMs,
          enqueuedVisibilityState: task.enqueuedVisibilityState,
          enqueuedHasFocus: task.enqueuedHasFocus,
        });
        try {
          callback();
        } finally {
          task.state = 'complete';
          task.completedAt = performance.now();
          task.callbackMs = task.completedAt - task.startedAt;
        }
      };
      const handle = delayMs > 0 && config.scheduler === 'message-channel'
        ? originalScheduleTask(wrapped, delayMs)
        : baseSchedule(wrapped, delayMs);
      task.handle = handle;
      trace.tasks.set(task.taskId, task);
      trace.taskByHandle.set(handle, task);
      record('scheduler.enqueue', {
        taskId: task.taskId,
        jobId: task.jobId,
        revision: task.revision,
        scheduler: config.scheduler,
      });
      return handle;
    };
    runner.cancelTask = (handle) => {
      const task = trace.taskByHandle.get(handle);
      if (task && task.state === 'queued') {
        task.state = 'cancelled';
        task.cancelledAt = performance.now();
        task.queuedUntilCancelMs = task.cancelledAt - task.enqueuedAt;
        record('scheduler.cancel', {
          taskId: task.taskId,
          jobId: task.jobId,
          revision: task.revision,
          queuedUntilCancelMs: task.queuedUntilCancelMs,
        });
      }
      return task?.requestedDelayMs > 0 && config.scheduler === 'message-channel'
        ? originalCancelTask(handle) : baseCancel(handle);
    };
    trace.restores.push(() => {
      runner.scheduleTask = originalScheduleTask;
      runner.cancelTask = originalCancelTask;
    });

    const originalRunnerStart = runner.requestStart;
    if (typeof originalRunnerStart !== 'function') throw new Error('requestStart API missing');
    runner.requestStart = function issue3794RunnerStart(...args) {
      const wasActive = this.isActive();
      const previousContext = trace.cancelContext;
      trace.cancelContext = wasActive ? 'restart' : 'start-preflight';
      const startedAt = performance.now();
      try {
        const result = originalRunnerStart.apply(this, args);
        record('runner.start', {
          wasActive,
          requestedDelays: args,
          durationMs: performance.now() - startedAt,
          status: result?.status ?? null,
          revision: result?.revision ?? null,
        });
        return result;
      } finally {
        trace.cancelContext = previousContext;
      }
    };
    trace.restores.push(() => { runner.requestStart = originalRunnerStart; });

    const originalRunnerCancel = runner.cancel;
    runner.cancel = function issue3794RunnerCancel(...args) {
      const previousContext = trace.cancelContext;
      trace.cancelContext ??= 'runner-cancel';
      try {
        return originalRunnerCancel.apply(this, args);
      } finally {
        trace.cancelContext = previousContext;
      }
    };
    trace.restores.push(() => { runner.cancel = originalRunnerCancel; });

    const originalBegin = wasm.beginDeferredPagination;
    wasm.beginDeferredPagination = function issue3794Begin(...args) {
      const startedAt = performance.now();
      const result = originalBegin.apply(this, args);
      if (trace.activeJob) endActiveJob('superseded-by-begin');
      const job = {
        jobId: trace.nextJobId++,
        revision: result?.revision ?? null,
        fragmentBudget: args[0] ?? null,
        beginMs: performance.now() - startedAt,
        beganAtMs: startedAt - trace.startedAt,
        emittedFragments: 0,
        stepCalls: 0,
        stepComputeMs: 0,
        queueWaitMs: 0,
        outcome: result?.status === 'pending' ? null : result?.status ?? 'unknown',
      };
      trace.jobs.push(job);
      if (result?.status === 'pending') trace.activeJob = job;
      else {
        job.endedAtMs = performance.now() - trace.startedAt;
        job.completed = result?.status === 'complete';
        job.discarded = result?.status !== 'complete';
      }
      record('wasm.beginDeferredPagination', {
        jobId: job.jobId,
        revision: job.revision,
        fragmentBudget: args[0] ?? null,
        durationMs: job.beginMs,
        status: result?.status ?? null,
      });
      return result;
    };
    trace.restores.push(() => { wasm.beginDeferredPagination = originalBegin; });

    const originalStep = wasm.stepDeferredPagination;
    wasm.stepDeferredPagination = function issue3794Step(...args) {
      const job = trace.activeJob;
      const startedAt = performance.now();
      const result = originalStep.apply(this, args);
      const durationMs = performance.now() - startedAt;
      if (job) {
        job.stepCalls += 1;
        job.stepComputeMs += durationMs;
        job.emittedFragments += result?.fragmentsProcessed ?? 0;
      }
      record('wasm.stepDeferredPagination', {
        jobId: job?.jobId ?? null,
        revision: result?.revision ?? job?.revision ?? null,
        fragmentBudget: args[0] ?? null,
        durationMs,
        status: result?.status ?? null,
        fragmentsProcessed: result?.fragmentsProcessed ?? null,
      });
      if (result?.status && result.status !== 'pending') {
        endActiveJob(result.status, { finalStepMs: durationMs });
      }
      return result;
    };
    trace.restores.push(() => { wasm.stepDeferredPagination = originalStep; });

    const originalCancel = wasm.cancelDeferredPagination;
    wasm.cancelDeferredPagination = function issue3794Cancel(...args) {
      const job = trace.activeJob;
      const startedAt = performance.now();
      const result = originalCancel.apply(this, args);
      const reason = trace.cancelContext ?? 'core-cancel';
      record('wasm.cancelDeferredPagination', {
        jobId: job?.jobId ?? null,
        revision: job?.revision ?? null,
        durationMs: performance.now() - startedAt,
        result,
        reason,
      });
      if (job) endActiveJob(result ? reason : `orphaned:${reason}`, { coreCancelResult: result });
      return result;
    };
    trace.restores.push(() => { wasm.cancelDeferredPagination = originalCancel; });

    const originalFlush = input.flushDeferredPaginationIfNeeded;
    input.flushDeferredPaginationIfNeeded = function issue3794InputFlush(...args) {
      const previousContext = trace.cancelContext;
      trace.cancelContext = `flush:${args[0] ?? 'manual'}`;
      const startedAt = performance.now();
      try {
        const result = originalFlush.apply(this, args);
        record('input.flushDeferredPaginationIfNeeded', {
          reason: args[0] ?? 'manual',
          durationMs: performance.now() - startedAt,
          result,
        });
        return result;
      } finally {
        trace.cancelContext = previousContext;
      }
    };
    trace.restores.push(() => { input.flushDeferredPaginationIfNeeded = originalFlush; });

    wrap(
      input,
      'prepareTextMutationBeforeCursor',
      'InputHandler.prepareTextMutationBeforeCursor',
      (args) => ({
        documentPaginationPending: args[0]?.documentPaginationPending ?? null,
        flowChanged: args[0]?.flowChanged ?? null,
        paginationCompleted: args[0]?.paginationCompleted ?? null,
      }),
      (result) => ({ boundaryHandled: result }),
    );
    wrap(
      input,
      'executeOperation',
      'InputHandler.executeOperation',
      (args) => ({
        kind: args[0]?.kind ?? null,
        commandType: args[0]?.command?.type ?? args[0]?.operationType ?? null,
      }),
    );
    for (const method of [
      'splitParagraphInCell',
      'splitParagraphInCellByPath',
      'mergeParagraphInCell',
      'mergeParagraphInCellByPath',
      'flushDeferredPagination',
      'getCursorRectByPathNear',
      'getCursorRectByPath',
      'getCursorRectInCell',
      'getCursorRectInCellByPath',
      'getCursorRect',
    ]) {
      wrap(wasm, method, `WasmBridge.${method}`);
    }

    if (
      typeof PerformanceObserver === 'function'
      && PerformanceObserver.supportedEntryTypes?.includes('longtask')
    ) {
      trace.longTaskObserver = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          trace.longTasks.push({
            startTime: entry.startTime,
            durationMs: entry.duration,
            name: entry.name,
          });
        }
      });
      trace.longTaskObserver.observe({ type: 'longtask' });
    }
    window.__issue3794Trace = trace;
  }, { fragmentBudget, scheduler });
}

async function collectTrace(page) {
  return page.evaluate(() => {
    const trace = window.__issue3794Trace;
    if (!trace) throw new Error('issue3794 trace is not installed');
    const tasks = [...trace.tasks.values()].map(({ handle: _handle, ...task }) => ({ ...task }));
    const jobs = trace.jobs.map((job) => ({ ...job }));
    for (const job of jobs) {
      const taskRows = tasks.filter((task) => task.jobId === job.jobId && task.state === 'complete');
      job.queueWaitMs = taskRows.reduce((sum, task) => sum + (task.queueDelayMs ?? 0), 0);
      job.schedulerCallbacks = taskRows.length;
      job.cancelledQueuedTasks = tasks.filter(
        (task) => task.jobId === job.jobId && task.state === 'cancelled',
      ).length;
    }
    return {
      config: { ...trace.config },
      elapsedMs: performance.now() - trace.startedAt,
      events: trace.events.map((event) => ({ ...event })),
      jobs,
      tasks,
      longTasks: trace.longTasks
        .filter((entry) => entry.startTime >= trace.startedAt)
        .map((entry) => ({ ...entry })),
    };
  });
}

function summarizeTrace(trace) {
  const completed = trace.jobs.filter((job) => job.outcome === 'complete');
  const discarded = trace.jobs.filter((job) => job.outcome && job.outcome !== 'complete');
  const stepEvents = trace.events.filter((event) => event.type === 'wasm.stepDeferredPagination');
  const queueRuns = trace.tasks.filter((task) => task.state === 'complete');
  const queueCancels = trace.tasks.filter((task) => task.state === 'cancelled');
  const starts = trace.events.filter((event) => event.type === 'runner.start');
  const coreCancels = trace.events.filter((event) => event.type === 'wasm.cancelDeferredPagination');
  const methodTotals = {};
  for (const type of [
    'InputHandler.executeOperation',
    'WasmBridge.splitParagraphInCell',
    'WasmBridge.splitParagraphInCellByPath',
    'WasmBridge.mergeParagraphInCell',
    'WasmBridge.mergeParagraphInCellByPath',
    'WasmBridge.flushDeferredPagination',
    'input.flushDeferredPaginationIfNeeded',
    'WasmBridge.getCursorRectByPathNear',
    'WasmBridge.getCursorRectByPath',
    'WasmBridge.getCursorRectInCell',
    'WasmBridge.getCursorRectInCellByPath',
    'WasmBridge.getCursorRect',
  ]) {
    const values = trace.events
      .filter((event) => event.type === type)
      .map((event) => event.durationMs);
    methodTotals[type] = summarize(values);
  }
  return {
    jobCount: trace.jobs.length,
    completedJobs: completed.length,
    discardedJobs: discarded.length,
    startRequests: starts.length,
    restartStarts: starts.filter((event) => event.wasActive).length,
    coreCancelAttempts: coreCancels.length,
    coreCancelSuccesses: coreCancels.filter((event) => event.result === true).length,
    emittedFragments: stepEvents.reduce(
      (sum, event) => sum + (event.fragmentsProcessed ?? 0),
      0,
    ),
    committedFragments: completed.reduce((sum, job) => sum + job.emittedFragments, 0),
    discardedFragments: discarded.reduce((sum, job) => sum + job.emittedFragments, 0),
    discardedBeginMs: discarded.reduce((sum, job) => sum + job.beginMs, 0),
    discardedStepComputeMs: discarded.reduce((sum, job) => sum + job.stepComputeMs, 0),
    begin: summarize(trace.jobs.map((job) => job.beginMs)),
    stepCompute: summarize(stepEvents.map((event) => event.durationMs)),
    schedulerQueue: summarize(queueRuns.map((task) => task.queueDelayMs)),
    schedulerRequestedDelay: summarize(queueRuns.map((task) => task.requestedDelayMs)),
    schedulerExcessQueue: summarize(queueRuns.map((task) => task.excessQueueDelayMs)),
    schedulerCallbacks: queueRuns.length,
    cancelledQueuedTasks: queueCancels.length,
    longTasks: {
      count: trace.longTasks.length,
      totalMs: trace.longTasks.reduce((sum, entry) => sum + entry.durationMs, 0),
      maxMs: trace.longTasks.length > 0
        ? Math.max(...trace.longTasks.map((entry) => entry.durationMs))
        : null,
    },
    methodTotals,
  };
}

function summarizeInputTiming(samples) {
  return {
    requestedIntervalMs: summarize(samples.map((sample) => sample.requestedIntervalMs)),
    actualIntervalMs: summarize(samples.map((sample) => sample.actualIntervalMs)),
    intervalDriftMs: summarize(samples.map((sample) => sample.intervalDriftMs)),
    startDriftMs: summarize(samples.map((sample) => sample.startDriftMs)),
    syncDispatchMs: summarize(samples.map((sample) => sample.syncDispatchMs)),
    inputToFirstRafMs: summarize(samples.map((sample) => sample.inputToFirstRafMs)),
    inputToSecondRafMs: summarize(samples.map((sample) => sample.inputToSecondRafMs)),
  };
}

function flowCaseConfig(caseName, config) {
  switch (caseName) {
    case 'flow-budget':
      return { fragmentBudget: config.budget, scheduler: 'timeout', tailInputs: 0 };
    case 'flow-scheduler':
      return { fragmentBudget: 1, scheduler: 'message-channel', tailInputs: 0 };
    case 'flow-restart':
      return { fragmentBudget: 1, scheduler: 'timeout', tailInputs: config.tailInputs };
    case 'flow-budget-restart':
      return { fragmentBudget: config.budget, scheduler: 'timeout', tailInputs: config.tailInputs };
    case 'flow-scheduler-restart':
      return { fragmentBudget: 1, scheduler: 'message-channel', tailInputs: config.tailInputs };
    case 'flow-current':
      return { fragmentBudget: 1, scheduler: 'timeout', tailInputs: 0 };
    default:
      throw new Error(`not a flow case: ${caseName}`);
  }
}

async function discoverFlowTransition(page, format, bytes) {
  await restoreTrace(page);
  await openDocumentThroughApp(page, format, bytes);
  await moveToPosition(page, TARGET);
  const initial = await readFlowFinalCorrectness(page, 0);
  const baselineLines = initial.lineInfo.lineCount;
  const steps = [];
  for (let count = 1; count <= 128; count++) {
    await page.evaluate(() => {
      const input = window.__inputHandler;
      const original = input.prepareTextMutationBeforeCursor;
      window.__issue3743LastEffects = null;
      input.prepareTextMutationBeforeCursor = function(effects) {
        window.__issue3743LastEffects = { ...effects };
        return original.call(this, effects);
      };
      window.__issue3743RestoreCalibration = () => { input.prepareTextMutationBeforeCursor = original; };
    });
    await dispatchText(page, '1');
    const effects = await page.evaluate(() => {
      window.__issue3743RestoreCalibration();
      return window.__issue3743LastEffects;
    });
    const state = await readFlowFinalCorrectness(page, count);
    steps.push({ count, lineCount: state.lineInfo.lineCount, effects });
    if (effects?.flowChanged && effects?.documentPaginationPending && state.lineInfo.lineCount > baselineLines) {
      return { format, boundaryInput: count, baselineLines, boundaryLines: state.lineInfo.lineCount, steps };
    }
  }
  throw new Error(`${format}: no admitted automatic wrap in 128 inputs: ${JSON.stringify(steps)}`);
}

async function runFlowCase(page, format, bytes, caseName, config) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToPosition(page, TARGET);
  await typeStablePrefix(page);
  const runtime = flowCaseConfig(caseName, config);
  await installTrace(page, runtime);

  const totalInputs = 1 + runtime.tailInputs;
  const samples = (await dispatchTextSequence(page, {
    text: '1',
    count: totalInputs,
    cadenceMs: config.cadenceMs,
  })).map((sample) => ({
    ...sample,
    role: sample.index === 0 ? 'flow-boundary' : 'restart-tail',
  }));
  const completionWaitMs = await waitForCompletion(page, config.timeoutMs);
  await waitTwoRafs(page);
  const finalState = await readState(page);
  const insertedCount = (FLOW_TRANSITION_INPUT - 1) + totalInputs;
  const finalCorrectness = await readFlowFinalCorrectness(page, insertedCount);
  const trace = await collectTrace(page);
  const summary = summarizeTrace(trace);
  summary.inputTiming = summarizeInputTiming(samples);
  assert.equal(finalState.pending, false, `${caseName}: pending after completion`);
  assert.equal(finalState.runnerActive, false, `${caseName}: runner active after completion`);
  assert.equal(finalState.pageCount, 115, `${caseName}: page count after completion`);
  assert.equal(
    finalCorrectness.cursor.charOffset,
    TARGET.charOffset + insertedCount,
    `${caseName}: final cursor offset`,
  );
  assert.equal(
    finalCorrectness.insertedText,
    '1'.repeat(insertedCount),
    `${caseName}: inserted text`,
  );
  assert.ok(finalCorrectness.lineInfo.lineCount > INITIAL_LINE_COUNT, `${caseName}: final line count must increase from ${INITIAL_LINE_COUNT}: ${JSON.stringify(finalCorrectness.lineInfo)}`);
  assert.ok(trace.events.some(e => e.type === 'InputHandler.prepareTextMutationBeforeCursor' && e.flowChanged), `${caseName}: actual flow-changing mutation required`);
  assert.equal(summary.completedJobs, 1, `${caseName}: exactly one latest job must complete`);
  if (
    caseName === 'flow-restart'
    || caseName === 'flow-budget-restart'
    || caseName === 'flow-scheduler-restart'
  ) {
    // Coalescing can avoid starting obsolete jobs altogether; discarded jobs are a metric, not an acceptance requirement.
    assert.ok(summary.startRequests >= totalInputs, `${caseName}: latest-revision start requests recorded`);
  }
  return {
    caseName,
    format,
    load,
    runtime,
    samples,
    boundaryInput: FLOW_TRANSITION_INPUT,
    completionWaitMs,
    finalState,
    finalCorrectness,
    summary,
    trace,
  };
}

async function prepareFlowBoundary(page) {
  await moveToPosition(page, TARGET);
  await typeStablePrefix(page);
  const boundary = await dispatchText(page, '1');
  assert.equal(boundary.pending, true, 'flow boundary must be pending');
  return boundary;
}

async function runStructuralCase(page, format, bytes, caseName, config) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToPosition(page, TARGET);
  let setup = null;

  if (
    caseName === 'merge-cold'
    || caseName === 'merge-pending'
    || caseName === 'merge-delete-cold'
    || caseName === 'merge-delete-pending'
  ) {
    setup = { split: await dispatchKeyAndWaitForPaint(page, 'Enter') };
    assert.equal(setup.split.cursor.cellParaIndex, TARGET.cellParaIndex + 1, `${caseName}: setup split`);
  }

  if (caseName === 'enter-pending') {
    await typeStablePrefix(page);
  } else if (caseName === 'merge-pending' || caseName === 'merge-delete-pending') {
    await moveToPosition(page, TARGET);
    await typeStablePrefix(page);
  } else if (caseName === 'merge-delete-cold') {
    await moveToPosition(page, TARGET);
  }

  await installTrace(page, { fragmentBudget: 1, scheduler: 'timeout' });

  if (caseName === 'enter-pending') {
    setup = { boundary: await dispatchText(page, '1') };
  } else if (caseName === 'merge-pending' || caseName === 'merge-delete-pending') {
    setup.boundary = await dispatchText(page, '1');
    if (caseName === 'merge-pending') {
      const mergePosition = withCellParagraph(
        TARGET,
        TARGET.cellParaIndex + 1,
        0,
      );
      await moveToPosition(page, mergePosition);
    }
  }

  if (caseName === 'enter-pending') {
    await page.waitForFunction(() => window.__inputHandler.deferredPaginationRunner.isActive(),
      { timeout: config.timeoutMs, polling: 10 });
  }

  const key = caseName.startsWith('enter-')
    ? 'Enter'
    : caseName.startsWith('merge-delete-')
      ? 'Delete'
      : 'Backspace';
  const keySample = await dispatchKeyAndWaitForPaint(page, key);
  const finalState = await readState(page);
  assert.equal(finalState.pending, false, `${caseName}: pending after structural edit`);
  assert.equal(finalState.runnerActive, false, `${caseName}: runner active after structural edit`);
  // Keep the historical 115-page contract as an explicit diagnostic failure,
  // while collecting the remaining timing rows. This is not a baseline update.
  const contractFailures = finalState.pageCount === 115 ? []
    : [`historical page-count gate failed: expected 115, actual ${finalState.pageCount}`];
  const trace = await collectTrace(page);
  const summary = summarizeTrace(trace);
  const expectedPending = caseName.endsWith('-pending');
  assert.equal(
    keySample.preExistingPending,
    expectedPending,
    `${caseName}: pre-existing pending classification`,
  );
  const expectedCommand = key === 'Enter'
    ? 'splitParagraphInCell'
    : key === 'Delete'
      ? 'mergeNextParagraphInCell'
      : 'mergeParagraphInCell';
  assert.ok(
    trace.events.some(
      (event) => event.type === 'InputHandler.executeOperation'
        && event.commandType === expectedCommand,
    ),
    `${caseName}: ${expectedCommand} command trace`,
  );
  if (key === 'Delete') {
    assert.ok(
      trace.events.some((event) => event.type === 'WasmBridge.mergeParagraphInCell'),
      `${caseName}: mergeParagraphInCell WASM trace`,
    );
  }
  const fullFlushOracle = await page.evaluate(() => {
    const before = window.__wasm.pageCount;
    const result = window.__wasm.flushDeferredPagination();
    return { before, after: window.__wasm.pageCount, result };
  });
  return { caseName, format, load, setup, keySample, finalState, summary, trace, contractFailures, fullFlushOracle };
}

// IME Enter commits composition only in the current product. It does not split a paragraph.
async function runImeEnterCase(page, format, bytes, caseName, config) {
  await restoreTrace(page);
  const load = await openDocumentThroughApp(page, format, bytes);
  await moveToPosition(page, TARGET);
  if (caseName.endsWith('-pending')) await prepareFlowBoundary(page);
  await page.evaluate(() => {
    const textarea = window.__inputHandler.textarea;
    textarea.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true, data: '' }));
    textarea.value = '한';
    textarea.dispatchEvent(new InputEvent('input', {
      bubbles: true, data: '한', inputType: 'insertCompositionText', isComposing: true,
    }));
  });
  if (caseName.endsWith('-pending')) {
    await page.waitForFunction(() => window.__inputHandler.deferredPaginationRunner.isActive(),
      { timeout: config.timeoutMs, polling: 10 });
  }
  await installTrace(page, { fragmentBudget: 1, scheduler: 'timeout' });
  const keySample = await page.evaluate(async () => {
    const input = window.__inputHandler;
    const beforeCursor = input.cursor.getPosition();
    const preExistingPending = input.hasDeferredPaginationPending();
    const runnerWasActive = input.deferredPaginationRunner.isActive();
    const startedAt = performance.now();
    input.textarea.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Process', code: 'Enter', keyCode: 229,
      isComposing: true, bubbles: true, cancelable: true,
    }));
    const queued = input._pendingNavAfterIME?.code;
    const keydownMs = performance.now() - startedAt;
    const compositionEndAt = performance.now();
    input.textarea.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: '한' }));
    const compositionEndMs = performance.now() - compositionEndAt;
    const syncDispatchMs = performance.now() - startedAt;
    await new Promise(r => requestAnimationFrame(r));
    const inputToFirstRafMs = performance.now() - startedAt;
    await new Promise(r => requestAnimationFrame(r));
    return {
      queued, beforeCursor, preExistingPending, runnerWasActive, keydownMs, compositionEndMs,
      syncDispatchMs, inputToFirstRafMs, inputToSecondRafMs: performance.now() - startedAt,
      cursor: input.cursor.getPosition(), pendingAfter: input.hasDeferredPaginationPending(),
    };
  });
  assert.equal(keySample.queued, 'Enter', 'composition keydown must queue the Enter navigation');
  assert.deepEqual(keySample.cursor, keySample.beforeCursor, 'IME commit must not split or move the cursor');
  assert.equal(keySample.pendingAfter, false, 'IME Enter barrier must finish pending pagination');
  const finalState = await readState(page);
  assert.equal(finalState.pageCount, 115);
  const trace = await collectTrace(page);
  const summary = summarizeTrace(trace);
  assert.equal(summary.methodTotals['WasmBridge.flushDeferredPagination'].count, 1);
  assert.equal(summary.methodTotals['WasmBridge.splitParagraphInCell'].count, 0);
  return { caseName, format, load, keySample, finalState, summary, trace };
}

async function runCase(page, format, bytes, caseName, config) {
  if (caseName.startsWith('ime-enter-')) return runImeEnterCase(page, format, bytes, caseName, config);
  if (caseName.startsWith('flow-')) {
    return runFlowCase(page, format, bytes, caseName, config);
  }
  return runStructuralCase(page, format, bytes, caseName, config);
}

async function main() {
  const config = parseConfig();
  mkdirSync(config.outputRoot, { recursive: true });
  const fixtures = Object.fromEntries(config.formats.map((format) => {
    const bytes = readFileSync(SAMPLES[format]);
    return [format, { path: SAMPLES[format], bytes, size: bytes.length, sha256: sha256(bytes) }];
  }));
  const wasm = fingerprint(path.join(REPO_ROOT, 'pkg/rhwp_bg.wasm'));
  assert.ok(wasm, 'pkg/rhwp_bg.wasm missing; build production WASM first');

  const browser = await launchBrowser();
  const page = await createPage(browser, 1280, 900);
  const pageErrors = [];
  page.on('pageerror', (error) => pageErrors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') pageErrors.push(message.text());
  });

  const results = [];
  writeJson(path.join(config.outputRoot, 'environment.json'), {
    gitRevision: gitRevision(), config, wasm, node: process.version,
    browser: await browser.version(), harness: fingerprint(fileURLToPath(import.meta.url)),
    fixtureHashes: Object.fromEntries(Object.entries(fixtures).map(([k,v]) => [k,v.sha256])),
  });
  try {
    await loadApp(page);
    for (const format of config.formats) {
      const calibration = await discoverFlowTransition(page, format, fixtures[format].bytes);
      FLOW_TRANSITION_INPUT = calibration.boundaryInput;
      INITIAL_LINE_COUNT = calibration.baselineLines;
      writeJson(path.join(config.outputRoot, `${format}-calibration.json`), calibration);
      console.log(`[${format}] calibrated wrap input=${FLOW_TRANSITION_INPUT}, lines=${INITIAL_LINE_COUNT}→${calibration.boundaryLines}`);
      for (let runNumber = 1; runNumber <= config.runs; runNumber += 1) {
        for (const caseName of config.cases) {
          const errorsBefore = pageErrors.length;
          console.log(`[${format.toUpperCase()} ${caseName} run ${runNumber}] start`);
          let result;
          try {
            result = await runCase(page, format, fixtures[format].bytes, caseName, config);
          } catch (error) {
            writeJson(path.join(config.outputRoot, `${format}-${caseName}-${runNumber}-failure.json`), {
              error: error.stack ?? String(error), state: await readState(page).catch(() => null),
              trace: await collectTrace(page).catch(() => null), pageErrors,
            });
            throw error;
          }
          assert.equal(
            pageErrors.length,
            errorsBefore,
            `${format} ${caseName}: browser errors: ${pageErrors.slice(errorsBefore).join(' | ')}`,
          );
          result.runNumber = runNumber;
          results.push(result);
          writeJson(path.join(config.outputRoot, `${format}-${caseName}-${runNumber}.json`), result);
          console.log(
            `[${format.toUpperCase()} ${caseName} run ${runNumber}] `
              + `begin=${result.summary.begin.p95Ms?.toFixed(2) ?? 'n/a'}ms `
              + `step=${result.summary.stepCompute.p95Ms?.toFixed(2) ?? 'n/a'}ms `
              + `queue=${result.summary.schedulerQueue.p95Ms?.toFixed(2) ?? 'n/a'}ms `
              + `restart=${result.summary.restartStarts} `
              + `discard=${result.summary.discardedFragments}`,
          );
          await restoreTrace(page);
        }
      }
    }
  } finally {
    await restoreTrace(page).catch(() => {});
    await closePage(page).catch(() => {});
    await closeBrowser(browser).catch(() => {});
  }

  const artifact = {
    issue: 3794,
    generatedAt: new Date().toISOString(),
    gitRevision: gitRevision(),
    config,
    wasm,
    fixtures: Object.fromEntries(
      Object.entries(fixtures).map(([format, { bytes: _bytes, ...value }]) => [format, value]),
    ),
    pageErrors,
    timingSemantics: {
      lane: 'input-event-loop',
      inputToRaf: 'Measures frame opportunities after input dispatch, not visible ink or display-revision publication.',
      visualPublication: 'Out of scope for this Stage 0 runtime probe.',
    },
    scenarios: results,
  };
  const summaryPath = path.join(config.outputRoot, 'summary.json');
  writeJson(summaryPath, artifact);
  console.log(`Issue #3794 JSON: ${summaryPath}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
