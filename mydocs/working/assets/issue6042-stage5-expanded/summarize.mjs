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

function summarizeRows(input) {
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
    cacheTakeCalls: stats(traces.map(trace => counter(trace, 'cache.take', 'calls'))),
    errors: input.flatMap(row => row.errors).length,
  };
}

function delta(before, after) {
  const absolute = round(after - before);
  return {
    absolute,
    percent: before === 0 ? null : round((absolute / before) * 100),
  };
}

function benchPair(name, baselineFiles, correctedFiles) {
  const baseline = summarizeRows(rows(baselineFiles));
  const corrected = summarizeRows(rows(correctedFiles));
  const p50Limit = Math.max(10, baseline.retainedCompleteMs.p50 * 0.05);
  const p95Limit = Math.max(25, baseline.retainedCompleteMs.p95 * 0.10);
  const p50Delta = delta(baseline.retainedCompleteMs.p50, corrected.retainedCompleteMs.p50);
  const p95Delta = delta(baseline.retainedCompleteMs.p95, corrected.retainedCompleteMs.p95);
  return {
    name,
    baseline,
    corrected,
    retainedCompleteDelta: { p50: p50Delta, p95: p95Delta },
    limitsMs: { p50: round(p50Limit), p95: round(p95Limit) },
    accepted:
      corrected.complete === corrected.samples
      && corrected.errors === 0
      && p50Delta.absolute <= p50Limit
      && p95Delta.absolute <= p95Limit,
  };
}

function snapshotErrors(snapshot) {
  return snapshot?.errors?.length ?? 0;
}

function schedulerSettled(snapshot) {
  const scheduler = snapshot?.scheduler;
  return !scheduler || (
    scheduler.visibleQueued === 0
    && scheduler.prefetchQueued === 0
    && scheduler.frameScheduled === false
    && scheduler.idleScheduled === false
  );
}

function snapshotHealthy(snapshot) {
  return snapshotErrors(snapshot) === 0
    && snapshot?.pendingImages === 0
    && snapshot?.pendingPrefetch === 0
    && schedulerSettled(snapshot)
    && snapshot?.targetViewport?.scope === snapshot?.appliedViewport?.scope
    && snapshot?.targetViewport?.zoom === snapshot?.appliedViewport?.zoom;
}

function layoutSignature(snapshot, visibleOnly = false) {
  return {
    pageCount: snapshot.pageCount,
    zoom: snapshot.zoom,
    columns: snapshot.columns,
    visible: snapshot.visible,
    retained: snapshot.retained,
    scroll: snapshot.scroll,
    pages: snapshot.pages.filter(page => !visibleOnly || page.visible).map(page => ({
      page: page.page,
      visible: page.visible,
      dpr: page.dpr,
      scale: page.scale,
      surfaces: page.surfaces,
      pixels: page.pixels,
      box: page.box,
      flowImages: page.flowImages,
    })),
  };
}

function near(a, b, tolerance = 3) {
  return Math.abs(a - b) <= tolerance;
}

function layoutEquivalent(a, b, visibleOnly = false) {
  const left = layoutSignature(a, visibleOnly);
  const right = layoutSignature(b, visibleOnly);
  if (left.pageCount !== right.pageCount || left.zoom !== right.zoom || left.columns !== right.columns) return false;
  if (JSON.stringify(left.visible) !== JSON.stringify(right.visible)) return false;
  if (JSON.stringify(left.retained) !== JSON.stringify(right.retained)) return false;
  if (!near(left.scroll.x, right.scroll.x) || !near(left.scroll.y, right.scroll.y)) return false;
  if (left.pages.length !== right.pages.length) return false;
  return left.pages.every((page, index) => {
    const other = right.pages[index];
    return page.page === other.page
      && page.visible === other.visible
      && page.dpr === other.dpr
      && page.scale === other.scale
      && page.pixels === other.pixels
      && page.flowImages === other.flowImages
      && JSON.stringify(page.surfaces) === JSON.stringify(other.surfaces)
      && near(page.box.x, other.box.x)
      && near(page.box.y, other.box.y)
      && near(page.box.width, other.box.width)
      && near(page.box.height, other.box.height);
  });
}

function pairedSnapshots(baselineName, correctedName, select, compare = layoutEquivalent) {
  const baseline = select(read(baselineName));
  const corrected = select(read(correctedName));
  return {
    samples: baseline.length,
    healthy: baseline.every(snapshotHealthy) && corrected.every(snapshotHealthy),
    layoutEquivalent: baseline.length === corrected.length
      && baseline.every((snapshot, index) => compare(snapshot, corrected[index])),
  };
}

const performance = {
  canvas2dExamSingle50: benchPair('Canvas2D exam_kor single 50%', [
    'canvas2d-exam-single-50-stage3-a1.json', 'canvas2d-exam-single-50-stage3-a2.json',
  ], [
    'canvas2d-exam-single-50-corrected-b1.json', 'canvas2d-exam-single-50-corrected-b2.json',
  ]),
  canvas2dExamDouble50: benchPair('Canvas2D exam_kor double 50%', [
    'canvas2d-exam-double-50-stage3-a1.json', 'canvas2d-exam-double-50-stage3-a2.json',
  ], [
    'canvas2d-exam-double-50-corrected-b1.json', 'canvas2d-exam-double-50-corrected-b2.json',
  ]),
  canvas2dHwpspecSingle50: benchPair('Canvas2D hwpspec single 50%', [
    'canvas2d-hwpspec-single-50-stage3-a1.json', 'canvas2d-hwpspec-single-50-stage3-a2.json',
  ], [
    'canvas2d-hwpspec-single-50-corrected-b1.json', 'canvas2d-hwpspec-single-50-corrected-b2.json',
  ]),
  canvas2dHwpspecDouble50: benchPair('Canvas2D hwpspec double 50%', [
    'canvas2d-hwpspec-double-50-stage3-a1.json', 'canvas2d-hwpspec-double-50-stage3-a2.json',
  ], [
    'canvas2d-hwpspec-double-50-corrected-b1.json', 'canvas2d-hwpspec-double-50-corrected-b2.json',
  ]),
  canvasKitExamFour34: benchPair('CanvasKit exam_kor four columns 34%', [
    'canvaskit-exam-4col-34-stage3-a1.json', 'canvaskit-exam-4col-34-stage3-a2.json',
  ], [
    'canvaskit-exam-4col-34-corrected-b1.json', 'canvaskit-exam-4col-34-corrected-b2.json',
  ]),
  canvasKitHwpspecFour34: benchPair('CanvasKit hwpspec four columns 34%', [
    'canvaskit-hwpspec-4col-34-stage3-a1.json', 'canvaskit-hwpspec-4col-34-stage3-a2.json',
  ], [
    'canvaskit-hwpspec-4col-34-corrected-b1.json', 'canvaskit-hwpspec-4col-34-corrected-b2.json',
  ]),
  canvas2dKpsAiFour34: benchPair('Canvas2D kps-ai four columns 34%', [
    'canvas2d-kps-ai-4col-34-stage3.json',
  ], [
    'canvas2d-kps-ai-4col-34-corrected.json',
  ]),
};

const functional = {
  autoZoom: pairedSnapshots(
    'canvas2d-exam-auto-zoom-sequence-stage3.json',
    'canvas2d-exam-auto-zoom-sequence-corrected.json',
    value => value.map(row => row.snapshot),
    (baseline, corrected) => layoutEquivalent(baseline, corrected, true),
  ),
  horizontal: pairedSnapshots(
    'canvas2d-exam-horizontal-stage3.json',
    'canvas2d-exam-horizontal-corrected.json',
    value => [value.initial, ...value.moves.map(move => move.snapshot ?? move)],
  ),
  facing: pairedSnapshots(
    'canvas2d-four-page-facing-stage3.json',
    'canvas2d-four-page-facing-corrected.json',
    value => [value.top, value.bottom],
  ),
  lastIncompleteRow: pairedSnapshots(
    'canvas2d-prosecutor-21p-last-row-stage3.json',
    'canvas2d-prosecutor-21p-last-row-corrected.json',
    value => [value],
  ),
  fixedSingleZoom: pairedSnapshots(
    'canvas2d-four-page-single-zoom-stage3.json',
    'canvas2d-four-page-single-zoom-corrected.json',
    value => value.map(row => row.snapshot),
  ),
  fixedDoubleZoom: pairedSnapshots(
    'canvas2d-four-page-double-zoom-stage3.json',
    'canvas2d-four-page-double-zoom-corrected.json',
    value => value.map(row => row.snapshot),
  ),
  canvas2dKtxZoom: pairedSnapshots(
    'canvas2d-ktx-4-layer-zoom-stage3.json',
    'canvas2d-ktx-4-layer-zoom-corrected.json',
    value => value,
  ),
  canvas2dKtxZoomRepeat2: pairedSnapshots(
    'canvas2d-ktx-4-layer-zoom-stage3-repeat2.json',
    'canvas2d-ktx-4-layer-zoom-corrected-repeat2.json',
    value => value,
  ),
  canvasKitKtxZoom: pairedSnapshots(
    'canvaskit-ktx-4-layer-zoom-stage3.json',
    'canvaskit-ktx-4-layer-zoom-corrected.json',
    value => value,
  ),
  documentSwitch: pairedSnapshots(
    'canvas2d-document-switch-stage3.json',
    'canvas2d-document-switch-corrected.json',
    value => value.map(row => row.snapshot),
  ),
  autoBackend: pairedSnapshots(
    'auto-backend-exam-stage3.json',
    'auto-backend-exam-corrected.json',
    value => [value],
  ),
};

const rapidBaseline = read('canvas2d-exam-rapid-reverse-stage3.json');
const rapidCorrected = read('canvas2d-exam-rapid-reverse-corrected.json');
functional.rapidReverse = {
  healthy: snapshotHealthy(rapidBaseline.final) && snapshotHealthy(rapidCorrected.final),
  statusCountsEqual:
    rapidBaseline.traceStatusCounts.complete === rapidCorrected.traceStatusCounts.complete
    && rapidBaseline.traceStatusCounts.superseded === rapidCorrected.traceStatusCounts.superseded,
  layoutEquivalent: layoutEquivalent(rapidBaseline.final, rapidCorrected.final),
  baselineStatusCounts: rapidBaseline.traceStatusCounts,
  correctedStatusCounts: rapidCorrected.traceStatusCounts,
};

const ktxZoomRuns = {
  baseline1: read('canvas2d-ktx-4-layer-zoom-stage3.json'),
  baseline2: read('canvas2d-ktx-4-layer-zoom-stage3-repeat2.json'),
  corrected1: read('canvas2d-ktx-4-layer-zoom-corrected.json'),
  corrected2: read('canvas2d-ktx-4-layer-zoom-corrected-repeat2.json'),
};
const maxScrollDelta = (left, right) => Math.max(...left.map((snapshot, index) =>
  Math.max(
    Math.abs(snapshot.scroll.x - right[index].scroll.x),
    Math.abs(snapshot.scroll.y - right[index].scroll.y),
  )));
functional.zoomCoordinateRepeatability = {
  baselineWithinRevisionMaxCssPx: maxScrollDelta(ktxZoomRuns.baseline1, ktxZoomRuns.baseline2),
  correctedWithinRevisionMaxCssPx: maxScrollDelta(ktxZoomRuns.corrected1, ktxZoomRuns.corrected2),
  firstCrossRevisionMaxCssPx: maxScrollDelta(ktxZoomRuns.baseline1, ktxZoomRuns.corrected1),
  repeatedCrossRevisionMaxCssPx: maxScrollDelta(ktxZoomRuns.baseline2, ktxZoomRuns.corrected2),
  correctedNotWorseThanThreeCssPx: maxScrollDelta(ktxZoomRuns.baseline2, ktxZoomRuns.corrected2) <= 3,
};

const editBaseline = read('canvas2d-edit-undo-redo-stage3.json');
const editCorrected = read('canvas2d-edit-undo-redo-corrected.json');
functional.editUndoRedo = {
  healthy: Object.values(editBaseline).every(snapshotHealthy) && Object.values(editCorrected).every(snapshotHealthy),
  fourDistinctInvalidationScopes: [editBaseline, editCorrected].every(value =>
    new Set(Object.values(value).map(snapshot => snapshot.targetViewport.scope)).size === 4),
  layoutEquivalent: Object.keys(editBaseline).every(key => layoutEquivalent(editBaseline[key], editCorrected[key])),
};

const canvasKitIdentity = {
  baseline: read('canvaskit-surface-identity-stage3.json'),
  corrected: read('canvaskit-surface-identity-corrected.json'),
};
functional.canvasKitIdentity = {
  baselineIsCanvasKit: canvasKitIdentity.baseline.every(row => row.key.includes('backend:canvaskit')),
  correctedIsCanvasKit: canvasKitIdentity.corrected.every(row => row.key.includes('backend:canvaskit')),
};

const autoBaseline = read('auto-backend-exam-stage3.json');
const autoCorrected = read('auto-backend-exam-corrected.json');
const autoIdentityBaseline = read('auto-backend-identity-stage3.json');
const autoIdentityCorrected = read('auto-backend-identity-corrected.json');
const autoBackendOf = rows => rows[0]?.key.match(/backend:([^|]+)/)?.[1] ?? 'unknown';
functional.autoFallback = {
  baselineBackend: autoBackendOf(autoIdentityBaseline),
  correctedBackend: autoBackendOf(autoIdentityCorrected),
  healthy: snapshotHealthy(autoBaseline) && snapshotHealthy(autoCorrected),
};

const stage4Correction = JSON.parse(fs.readFileSync(
  path.join(here, '..', 'issue6042-stage5-correction', 'summary.json'),
  'utf8',
));

const summary = {
  schemaVersion: 1,
  generatedBy: 'summarize.mjs',
  comparison: {
    baseline: { label: 'Stage 3', sha: '5f5d60071' },
    corrected: { label: 'Stage 4 correction', sha: '63d29e68b' },
    browser: 'Chromium 151',
    viewportCssPx: { width: 1280, height: 720 },
    actualDpr: 2,
  },
  coverage: {
    renderers: ['Canvas2D', 'CanvasKit', 'auto→Canvas2D'],
    documents: ['exam_kor.hwp', 'hwpspec.hwp', 'kps-ai.hwp', 'basic/KTX.hwp', '4-page real HWP', '21-page layered HWP'],
    arrangements: ['auto', 'single', 'double', 'fixed four', 'facing'],
    interactions: ['vertical scroll', 'horizontal scroll', 'rapid reverse', 'zoom 34/50/100/200', 'document switch', 'edit/undo/redo'],
    automationLimits: {
      actualDpr1: 'not measured: in-app browser context is fixed to DPR 2',
      liveViewportResize: 'not measured: in-app browser viewport is fixed to 1280x720',
      imageFailureInjection: 'not measured: successful 4-layer/image lifecycle only',
      userVisualApproval: 'pending',
    },
  },
  alertThresholds: {
    p50Regression: 'max(10ms, 5%)',
    p95Regression: 'max(25ms, 10%)',
  },
  stage4CorrectionAccepted: stage4Correction.verdict.accepted,
  performance,
  functional,
};

const pairedFunctionalAccepted = [
  functional.autoZoom,
  functional.horizontal,
  functional.facing,
  functional.lastIncompleteRow,
  functional.fixedSingleZoom,
  functional.fixedDoubleZoom,
  functional.canvas2dKtxZoom,
  functional.canvas2dKtxZoomRepeat2,
  functional.canvasKitKtxZoom,
  functional.documentSwitch,
  functional.autoBackend,
].every(result => result.healthy && result.layoutEquivalent);

summary.verdict = {
  automatedMatrixAccepted:
    summary.stage4CorrectionAccepted
    && Object.values(performance).every(result => result.accepted)
    && pairedFunctionalAccepted
    && functional.rapidReverse.healthy
    && functional.rapidReverse.statusCountsEqual
    && functional.rapidReverse.layoutEquivalent
    && functional.zoomCoordinateRepeatability.correctedNotWorseThanThreeCssPx
    && functional.editUndoRedo.healthy
    && functional.editUndoRedo.fourDistinctInvalidationScopes
    && functional.editUndoRedo.layoutEquivalent
    && functional.canvasKitIdentity.baselineIsCanvasKit
    && functional.canvasKitIdentity.correctedIsCanvasKit
    && functional.autoFallback.healthy
    && functional.autoFallback.baselineBackend === functional.autoFallback.correctedBackend,
  fullStage5Accepted: false,
  remaining: ['actual DPR 1', 'live viewport resize', 'image failure injection', 'user visual approval'],
};

fs.writeFileSync(path.join(here, 'summary.json'), `${JSON.stringify(summary, null, 2)}\n`);
console.log(JSON.stringify(summary, null, 2));
