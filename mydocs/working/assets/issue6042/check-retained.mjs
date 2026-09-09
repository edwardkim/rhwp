// #6042에 남긴 원시의 무결성과 선택한 핵심 수치만 재검산한다. 새 성능 측정/전체 matrix 재검산이 아니다.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const assets = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = name => JSON.parse(fs.readFileSync(path.join(assets, name), 'utf8'));
const manifest = read('issue6042/retained-manifest.json');
for (const entry of manifest.files) {
  const bytes = fs.readFileSync(path.join(assets, entry.path));
  assert.equal(bytes.length, entry.bytes, `${entry.path}: byte length`);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, `${entry.path}: SHA-256`);
  if (entry.path.endsWith('.json')) JSON.parse(bytes.toString('utf8'));
}

const round = value => Number(value.toFixed(1));
function stats(values) {
  assert(values.length > 0 && values.every(Number.isFinite), 'missing/non-finite metric');
  const sorted = [...values].sort((a, b) => a - b);
  return { p50: round(sorted[Math.ceil(sorted.length * 0.50) - 1]),
    p95: round(sorted[Math.ceil(sorted.length * 0.95) - 1]) };
}
const count = (row, name, field = 'calls') => row.trace.counters[name]?.[field] ?? 0;
const metrics = {
  knownWorkNextFrameMs: row => row.sample.knownWorkNextFrameMs,
  visibleFirstMs: row => row.trace.milestones.visibleFirst,
  visibleStableMs: row => row.trace.milestones.visibleStable,
  retainedCompleteMs: row => row.trace.milestones.retainedComplete,
  visibilityUpdateMs: row => count(row, 'visibility.update', 'inclusiveMs'),
  budgetRefreshMs: row => count(row, 'budget.refresh', 'inclusiveMs'),
  rasterCalls: row => count(row, 'raster.main'),
  rasterInclusiveMs: row => count(row, 'raster.main', 'inclusiveMs'),
  cacheTakeCalls: row => count(row, 'cache.take'),
};
let checkedSeries = 0;
function verify(label, rows, expected, expectedCount, selectors = metrics) {
  assert.equal(rows.length, expectedCount, `${label}: sample count`);
  if (expected.samples !== undefined) assert.equal(rows.length, expected.samples, label);
  assert(rows.every(row => row.trace.status === 'complete'), `${label}: incomplete trace`);
  if (expected.complete !== undefined) assert.equal(rows.length, expected.complete, label);
  assert(rows.every(row => (row.errors ?? []).length === 0), `${label}: errors`);
  if (expected.errors !== undefined) assert.equal(expected.errors, 0, label);
  for (const [key, select] of Object.entries(selectors)) {
    if (!Object.hasOwn(expected, key)) continue;
    if (expected[key].n !== undefined) assert.equal(expected[key].n, rows.length, `${label}.${key}.n`);
    assert.deepEqual(stats(rows.map(select)), { p50: expected[key].p50, p95: expected[key].p95 }, `${label}.${key}`);
    checkedSeries++;
  }
}
function scrollRows(names) {
  return names.flatMap(name => {
    const document = read(name);
    assert.equal(document.samples.length, 20, `${name}: complete block required`);
    assert.equal(document.evidence.traces.length, document.samples.length, `${name}: trace/sample pairing`);
    return document.samples.map((sample, index) => {
      assert.equal(sample.round, index, `${name}: round ordering`);
      return { sample, trace: document.evidence.traces[index], errors: document.evidence.errors };
    });
  });
}
function blocks(folder, prefix, side) {
  return [1, 2].map(n => `${folder}/${prefix}-${side}${n}.json`);
}

const cold = read('issue6042-stage5/hwpspec-4col-34-cold-alternating.json');
const coldSummary = read('issue6042-stage5/summary.json').hwpspecFourColumns34Cold;
assert.equal(cold.rounds.length, 20);
for (const side of ['before', 'after']) {
  const rows = cold.rounds.map(run => run[side]);
  verify(`cold.${side}`, rows, coldSummary[side], 20);
  const tasks = rows.flatMap(row => row.longTasks ?? []);
  assert.deepEqual({ count: tasks.length, totalMs: round(tasks.reduce((sum, task) => sum + task.ms, 0)) },
    coldSummary[side].longTasks, `cold.${side}.longTasks`);
}

const correction = read('issue6042-stage5-correction/summary.json');
for (const [side, suffix] of [['baseline', 'stage3-a'], ['corrected', 'corrected-b']]) {
  const exam = scrollRows(blocks('issue6042-stage5-correction', 'exam-4col-34', suffix));
  verify(`exam.overall.${side}`, exam, correction.overall[side], 40);
  for (const [direction, parity] of [['down', 0], ['up', 1]]) {
    verify(`exam.${direction}.${side}`, exam.filter(row => row.sample.round % 2 === parity),
      correction[direction][side], 20);
  }
  const warm = scrollRows(blocks('issue6042-stage5-correction', 'hwpspec-4col-34', suffix));
  verify(`hwpspec.warm.${side}`, warm, correction.hwpspecWarm[side], 40);
}

const double = read('issue6042-stage5-expanded/summary.json').performance.canvas2dExamDouble50;
for (const [side, suffix] of [['baseline', 'stage3-a'], ['corrected', 'corrected-b']]) {
  verify(`double50.${side}`, scrollRows(blocks('issue6042-stage5-expanded', 'canvas2d-exam-double-50', suffix)),
    double[side], 40);
}
for (const p of ['p50', 'p95']) {
  const delta = round(double.corrected.retainedCompleteMs[p] - double.baseline.retainedCompleteMs[p]);
  assert.equal(delta, double.retainedCompleteDelta[p].absolute, `double50.${p}.delta`);
}

const quality = read('issue6042-stage5-scroll-quality/summary.json').performance20Rounds;
for (const side of ['before', 'after']) {
  verify(`quality.${side}`, scrollRows([`issue6042-stage5-scroll-quality/exam-double-100-${side}-20.json`]),
    quality[side], 20, {
      syncMs: row => row.sample.syncMs,
      visibleFirstMs: metrics.visibleFirstMs,
      visibleBitmapStableMs: metrics.visibleStableMs,
      // 당시 summary 이름은 retainedComplete를 가리킨다. runner의 추가 rAF 대기와 구분한다.
      settledKnownWorkMs: metrics.retainedCompleteMs,
      mainRasterCalls: metrics.rasterCalls,
    });
}
for (const [name, metric] of [['sync', 'syncMs'], ['settledKnownWork', 'settledKnownWorkMs']]) {
  for (const [p, label] of [['p50', 'P50'], ['p95', 'P95']]) {
    assert.equal(round(quality.after[metric][p] - quality.before[metric][p]),
      quality.deltaAfterMinusBeforeMs[`${name}${label}`], `quality.${name}.${p}.delta`);
  }
}

console.log(`PASS: ${manifest.files.length} retained files (SHA-256/bytes/JSON), ${checkedSeries} p50/p95 series`);
console.log('PASS: cold 20 pairs, correction 40/document/revision, double50 40/revision, quality 20/revision');
console.log('Historical evidence only; not a fresh benchmark, full-matrix replay, or current-head performance gate.');
