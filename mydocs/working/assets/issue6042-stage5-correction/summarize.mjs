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
  return Number.isFinite(value) ? Number(value.toFixed(digits)) : value;
}

function stats(values) {
  const finite = values.filter(Number.isFinite);
  return {
    n: finite.length,
    p50: round(percentile(finite, 0.5)),
    p95: round(percentile(finite, 0.95)),
    mean: round(finite.reduce((sum, value) => sum + value, 0) / finite.length),
  };
}

function counter(trace, boundary, field) {
  return trace?.counters?.[boundary]?.[field] ?? 0;
}

function rows(names) {
  return names.flatMap(name => {
    const document = read(name);
    return document.samples.map((sample, index) => ({
      sample,
      trace: document.evidence.traces[index],
      errors: document.evidence.errors,
      final: document.evidence,
    }));
  });
}

function summarize(input) {
  const traces = input.map(row => row.trace);
  return {
    samples: input.length,
    complete: traces.filter(trace => trace?.status === 'complete').length,
    knownWorkNextFrameMs: stats(input.map(row => row.sample.knownWorkNextFrameMs)),
    visibleFirstMs: stats(traces.map(trace => trace?.milestones?.visibleFirst)),
    visibleStableMs: stats(traces.map(trace => trace?.milestones?.visibleStable)),
    retainedCompleteMs: stats(traces.map(trace => trace?.milestones?.retainedComplete)),
    visibilityUpdateMs: stats(traces.map(trace => counter(trace, 'visibility.update', 'inclusiveMs'))),
    rasterCalls: stats(traces.map(trace => counter(trace, 'raster.main', 'calls'))),
    rasterInclusiveMs: stats(traces.map(trace => counter(trace, 'raster.main', 'inclusiveMs'))),
    cacheTakeCalls: stats(traces.map(trace => counter(trace, 'cache.take', 'calls'))),
    errors: input.flatMap(row => row.errors).length,
  };
}

function delta(before, after) {
  return {
    p50Ms: round(after.p50 - before.p50),
    p95Ms: round(after.p95 - before.p95),
    p50Percent: round(((after.p50 - before.p50) / before.p50) * 100),
    p95Percent: round(((after.p95 - before.p95) / before.p95) * 100),
  };
}

function ledger(row) {
  const evidence = row.final;
  return {
    activePixels: evidence.activePixels,
    cachedPixels: evidence.detachedCache.cachedPixels,
    reservedPixels: evidence.detachedCache.reservedPixels,
    totalAccountedPixels: evidence.detachedCache.totalAccountedPixels,
    overBudgetMandatory: evidence.detachedCache.overBudgetMandatory,
    scheduler: evidence.scheduler,
  };
}

const baselineRows = rows([
  'exam-4col-34-stage3-a1.json',
  'exam-4col-34-stage3-a2.json',
]);
const correctedRows = rows([
  'exam-4col-34-corrected-b1.json',
  'exam-4col-34-corrected-b2.json',
]);
const hwpspecBaselineRows = rows([
  'hwpspec-4col-34-stage3-a1.json',
  'hwpspec-4col-34-stage3-a2.json',
]);
const hwpspecCorrectedRows = rows([
  'hwpspec-4col-34-corrected-b1.json',
  'hwpspec-4col-34-corrected-b2.json',
]);
const directions = input => ({
  down: input.filter(row => row.sample.round % 2 === 0),
  up: input.filter(row => row.sample.round % 2 === 1),
});
const baseline = summarize(baselineRows);
const corrected = summarize(correctedRows);
const baselineDirection = directions(baselineRows);
const correctedDirection = directions(correctedRows);
const baselineUp = summarize(baselineDirection.up);
const correctedUp = summarize(correctedDirection.up);
const p50Limit = Math.max(10, baselineUp.retainedCompleteMs.p50 * 0.05);
const p95Limit = Math.max(25, baselineUp.retainedCompleteMs.p95 * 0.10);
const upwardDelta = delta(baselineUp.retainedCompleteMs, correctedUp.retainedCompleteMs);

const summary = {
  schemaVersion: 1,
  generatedBy: 'summarize.mjs',
  comparison: {
    baseline: { label: 'Stage 3', sha: '5f5d60071' },
    corrected: { label: 'Stage 4 correction', sha: '63d29e68b' },
    browser: 'Chromium 151 / Canvas2D',
    viewportCssPx: { width: 1280, height: 720 },
    actualDpr: 2,
    fixture: 'samples/exam_kor.hwp',
    arrangement: 'fixed 4 columns',
    zoom: 0.34,
  },
  alertThresholds: {
    p50Regression: 'max(10ms, 5%)',
    p95Regression: 'max(25ms, 10%)',
    resolvedP50LimitMs: round(p50Limit),
    resolvedP95LimitMs: round(p95Limit),
  },
  overall: { baseline, corrected },
  down: {
    baseline: summarize(baselineDirection.down),
    corrected: summarize(correctedDirection.down),
  },
  up: {
    baseline: baselineUp,
    corrected: correctedUp,
    retainedCompleteDelta: upwardDelta,
  },
  finalLedger: {
    baseline: ledger(baselineRows.at(-1)),
    corrected: ledger(correctedRows.at(-1)),
  },
  hwpspecWarm: {
    baseline: summarize(hwpspecBaselineRows),
    corrected: summarize(hwpspecCorrectedRows),
    finalLedger: {
      baseline: ledger(hwpspecBaselineRows.at(-1)),
      corrected: ledger(hwpspecCorrectedRows.at(-1)),
    },
  },
  verdict: {
    accepted:
      upwardDelta.p50Ms <= p50Limit
      && upwardDelta.p95Ms <= p95Limit
      && correctedUp.rasterCalls.p95 <= baselineUp.rasterCalls.p95
      && corrected.errors === 0,
  },
};

fs.writeFileSync(path.join(here, 'summary.json'), `${JSON.stringify(summary, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));
