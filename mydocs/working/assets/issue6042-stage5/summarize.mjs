import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function read(name) {
  return JSON.parse(fs.readFileSync(path.join(here, name), 'utf8'));
}

function percentile(values, fraction) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  if (sorted.length === 0) return null;
  return sorted[Math.max(0, Math.ceil(sorted.length * fraction) - 1)];
}

function round(value, digits = 1) {
  if (!Number.isFinite(value)) return value;
  return Number(value.toFixed(digits));
}

function stats(values) {
  const numbers = values.filter(Number.isFinite);
  return {
    n: numbers.length,
    p50: round(percentile(numbers, 0.5)),
    p95: round(percentile(numbers, 0.95)),
    max: round(numbers.length === 0 ? null : Math.max(...numbers)),
    mean: round(numbers.length === 0 ? null : numbers.reduce((a, b) => a + b, 0) / numbers.length),
  };
}

function counter(trace, name, field) {
  return trace?.counters?.[name]?.[field] ?? 0;
}

function summarizeTraceRows(rows) {
  const traces = rows.map((row) => row.trace);
  const longTasks = rows.flatMap((row) => row.longTasks ?? []);
  return {
    samples: rows.length,
    complete: traces.filter((trace) => trace?.status === 'complete').length,
    knownWorkNextFrameMs: stats(rows.map((row) => row.sample?.knownWorkNextFrameMs)),
    visibleFirstMs: stats(traces.map((trace) => trace?.milestones?.visibleFirst)),
    visibleStableMs: stats(traces.map((trace) => trace?.milestones?.visibleStable)),
    retainedCompleteMs: stats(traces.map((trace) => trace?.milestones?.retainedComplete)),
    visibilityUpdateMs: stats(traces.map((trace) => counter(trace, 'visibility.update', 'inclusiveMs'))),
    budgetRefreshMs: stats(traces.map((trace) => counter(trace, 'budget.refresh', 'inclusiveMs'))),
    rasterCalls: stats(traces.map((trace) => counter(trace, 'raster.main', 'calls'))),
    rasterInclusiveMs: stats(traces.map((trace) => counter(trace, 'raster.main', 'inclusiveMs'))),
    frameIntervalsMs: stats(traces.flatMap((trace) => trace?.frames ?? [])),
    longTasks: {
      count: longTasks.length,
      totalMs: round(longTasks.reduce((sum, task) => sum + (task.ms ?? 0), 0)),
    },
    errors: rows.flatMap((row) => row.errors ?? []).length,
  };
}

function scrollRows(names) {
  return names.flatMap((name) => {
    const document = read(name);
    return document.samples.map((sample, index) => ({
      sample,
      trace: document.evidence.traces[index],
      longTasks: document.evidence.longTasks.filter((task) => {
        const trace = document.evidence.traces[index];
        const next = document.evidence.traces[index + 1];
        return task.at >= trace.startedAt && (!next || task.at < next.startedAt);
      }),
      errors: document.evidence.errors,
      final: document.evidence,
    }));
  });
}

function finalLedger(name) {
  const evidence = read(name).evidence;
  return {
    activePixels: evidence.activePixels,
    idlePoolPixels: evidence.idlePoolPixels,
    detachedCachePixels: evidence.detachedCache?.cachedPixels,
    reservedPixels: evidence.detachedCache?.reservedPixels,
    totalAccountedPixels: evidence.detachedCache?.totalAccountedPixels,
    overBudgetMandatory: evidence.detachedCache?.overBudgetMandatory,
    pendingImages: evidence.pendingImages,
    pendingPrefetch: evidence.pendingPrefetch,
  };
}

function percentChange(before, after) {
  if (!Number.isFinite(before) || before === 0 || !Number.isFinite(after)) return null;
  return round(((after - before) / before) * 100);
}

function comparison(before, after, metric) {
  return {
    p50Percent: percentChange(before[metric].p50, after[metric].p50),
    p95Percent: percentChange(before[metric].p95, after[metric].p95),
  };
}

function coldRows(side) {
  const document = read('hwpspec-4col-34-cold-alternating.json');
  return document.rounds.map((round) => round[side]);
}

function summarizeZoom(names) {
  const rounds = names.flatMap((name) => read(name).rounds);
  const byZoom = {};
  for (const zoom of [...new Set(rounds.map((round) => round.zoom))].sort((a, b) => a - b)) {
    const selected = rounds.filter((round) => round.zoom === zoom);
    byZoom[zoom] = {
      samples: selected.length,
      complete: selected.filter((round) => round.trace.status === 'complete').length,
      previewMs: stats(selected.map((round) => round.trace.milestones.preview)),
      focusedSharpMs: stats(selected.map((round) => round.trace.milestones.focusedSharp)),
      visibleStableMs: stats(selected.map((round) => round.trace.milestones.visibleStable)),
      retainedCompleteMs: stats(selected.map((round) => round.trace.milestones.retainedComplete)),
      rasterCalls: stats(selected.map((round) => counter(round.trace, 'raster.main', 'calls'))),
      finalPixels: [...new Set(selected.map((round) => round.final.pixels))],
      errors: selected.flatMap((round) => round.errors ?? []).length,
    };
  }
  return byZoom;
}

const hwpspecBeforeWarmRows = scrollRows(['hwpspec-4col-34-a1.json', 'hwpspec-4col-34-a2.json']);
const hwpspecAfterWarmRows = scrollRows(['hwpspec-4col-34-b1.json', 'hwpspec-4col-34-b2.json']);
const examBeforeRows = scrollRows(['exam-4col-34-a1.json', 'exam-4col-34-a2.json']);
const examAfterRows = scrollRows(['exam-4col-34-b1.json', 'exam-4col-34-b2.json']);

const hwpspecColdBefore = summarizeTraceRows(coldRows('before'));
const hwpspecColdAfter = summarizeTraceRows(coldRows('after'));
const hwpspecWarmBefore = summarizeTraceRows(hwpspecBeforeWarmRows);
const hwpspecWarmAfter = summarizeTraceRows(hwpspecAfterWarmRows);
const examBefore = summarizeTraceRows(examBeforeRows);
const examAfter = summarizeTraceRows(examAfterRows);
const examBeforeDown = summarizeTraceRows(examBeforeRows.filter((row) => row.sample.round % 2 === 0));
const examBeforeUp = summarizeTraceRows(examBeforeRows.filter((row) => row.sample.round % 2 === 1));
const examAfterDown = summarizeTraceRows(examAfterRows.filter((row) => row.sample.round % 2 === 0));
const examAfterUp = summarizeTraceRows(examAfterRows.filter((row) => row.sample.round % 2 === 1));

const summary = {
  schemaVersion: 1,
  generatedBy: 'summarize.mjs',
  comparison: {
    before: { label: 'Stage 3', sha: '5f5d60071' },
    after: { label: 'Stage 4', sha: '6f2d82d24' },
    browser: 'Chromium 151 / Canvas2D',
    viewportCssPx: { width: 1280, height: 720 },
    actualDpr: 2,
  },
  alertThresholds: {
    p50Regression: 'max(10ms, 5%)',
    p95Regression: 'max(25ms, 10%)',
  },
  hwpspecFourColumns34Cold: {
    before: hwpspecColdBefore,
    after: hwpspecColdAfter,
    change: {
      knownWorkNextFrameMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'knownWorkNextFrameMs'),
      visibleFirstMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'visibleFirstMs'),
      visibleStableMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'visibleStableMs'),
      retainedCompleteMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'retainedCompleteMs'),
      visibilityUpdateMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'visibilityUpdateMs'),
      rasterInclusiveMs: comparison(hwpspecColdBefore, hwpspecColdAfter, 'rasterInclusiveMs'),
    },
  },
  hwpspecFourColumns34Warm: {
    before: hwpspecWarmBefore,
    after: hwpspecWarmAfter,
    beforeFinalLedger: finalLedger('hwpspec-4col-34-a2.json'),
    afterFinalLedger: finalLedger('hwpspec-4col-34-b2.json'),
  },
  examFourColumns34: {
    before: examBefore,
    after: examAfter,
    down: { before: examBeforeDown, after: examAfterDown },
    up: { before: examBeforeUp, after: examAfterUp },
    change: {
      overallKnownWorkNextFrameMs: comparison(examBefore, examAfter, 'knownWorkNextFrameMs'),
      overallRetainedCompleteMs: comparison(examBefore, examAfter, 'retainedCompleteMs'),
      upwardKnownWorkNextFrameMs: comparison(examBeforeUp, examAfterUp, 'knownWorkNextFrameMs'),
      upwardRetainedCompleteMs: comparison(examBeforeUp, examAfterUp, 'retainedCompleteMs'),
    },
    beforeFinalLedger: finalLedger('exam-4col-34-a2.json'),
    afterFinalLedger: finalLedger('exam-4col-34-b2.json'),
  },
  fourPageZoom50To100: {
    single: {
      before: summarizeZoom(['four-page-single-50-100-a1.json', 'four-page-single-50-100-a2.json']),
      after: summarizeZoom(['four-page-single-50-100-b1.json', 'four-page-single-50-100-b2.json']),
    },
    double: {
      before: summarizeZoom(['four-page-double-50-100-a1.json', 'four-page-double-50-100-a2.json']),
      after: summarizeZoom(['four-page-double-50-100-b1.json', 'four-page-double-50-100-b2.json']),
    },
  },
  verdict: {
    stage5Accepted: false,
    reason: 'exam_kor 4-column 34% upward return adds speculative raster work and breaches the predeclared retained-complete p95 regression threshold',
  },
};

fs.writeFileSync(path.join(here, 'summary.json'), `${JSON.stringify(summary, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));
