import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { PNG } from 'pngjs';

import { comparePngBuffers, cropPngBuffer } from './helpers.mjs';

const DEFAULT_IGNORE_CHANNEL_DELTA = 8;
const DEFAULT_MAX_DIFF_RATIO = 0.005;
const MAX_CAPTURE_SIZE_DRIFT_PX = 1;

function parseArgs() {
  const options = {
    native: '',
    browser: '',
    output: '',
    profiles: '',
    selfTest: false,
  };

  for (const arg of process.argv.slice(2)) {
    if (arg.startsWith('--native=')) {
      options.native = arg.slice('--native='.length);
      continue;
    }
    if (arg.startsWith('--browser=')) {
      options.browser = arg.slice('--browser='.length);
      continue;
    }
    if (arg.startsWith('--output=')) {
      options.output = arg.slice('--output='.length);
      continue;
    }
    if (arg.startsWith('--profiles=')) {
      options.profiles = arg.slice('--profiles='.length);
      continue;
    }
    if (arg === '--self-test') {
      options.selfTest = true;
    }
  }

  if (options.selfTest) return options;
  if (!options.native) throw new Error('missing --native=/abs/path/native-results.json');
  if (!options.browser) throw new Error('missing --browser=/abs/path/browser-baseline-report.json');
  if (!options.output) throw new Error('missing --output=/abs/path/native-canvaskit-parity-report.json');
  return options;
}

function comparisonIdentityMismatches(
  nativeIdentity,
  canvaskitIdentity,
  nativeArtifact,
  canvaskitArtifact,
  nativeBuffer,
  canvaskitBuffer,
) {
  const mismatches = [];
  for (const field of ['schemaVersion', 'sampleId', 'documentDigest', 'page', 'profile']) {
    if (nativeIdentity?.[field] !== canvaskitIdentity?.[field]) mismatches.push(field);
  }
  if (nativeIdentity?.backend !== 'native-skia') mismatches.push('nativeBackend');
  if (canvaskitIdentity?.backend !== 'canvaskit-default') mismatches.push('canvaskitBackend');

  for (const [prefix, artifact, buffer] of [
    ['native', nativeArtifact, nativeBuffer],
    ['canvaskit', canvaskitArtifact, canvaskitBuffer],
  ]) {
    if (
      !artifact
      || !/^[0-9a-f]{64}$/.test(artifact.sha256 ?? '')
      || !Number.isSafeInteger(artifact.sizeBytes)
      || artifact.sizeBytes <= 0
    ) {
      mismatches.push(`${prefix}Artifact`);
      continue;
    }
    const actualSha256 = createHash('sha256').update(buffer).digest('hex');
    if (artifact.sha256 !== actualSha256) mismatches.push(`${prefix}ArtifactSha256`);
    if (artifact.sizeBytes !== buffer.byteLength) mismatches.push(`${prefix}ArtifactSizeBytes`);
  }
  return [...new Set(mismatches)].sort();
}

function resolveRepoPath(value, rootDir) {
  if (!value) return '';
  return path.isAbsolute(value) ? value : path.resolve(rootDir, value);
}

function nativeSkiaResult(nativeResults, sampleId, profile, rootDir) {
  const sample = nativeResults.find((entry) => entry.sampleId === sampleId);
  const backend = sample?.backends?.find((entry) => (
    entry.backend === 'native-skia' && entry.profile === profile
  ));
  const pngPath = backend?.files?.find((file) => file.endsWith('.png'));
  return {
    sample,
    backend,
    path: resolveRepoPath(pngPath, rootDir),
  };
}

function canvaskitDefaultResult(browserResults, sampleId, profile, rootDir) {
  const item = browserResults.find((entry) => (
    entry.sampleId === sampleId
      && entry.backend === 'canvaskit-default'
      && entry.profile === profile
  ));
  return {
    entry: item,
    path: resolveRepoPath(item?.path, rootDir),
  };
}

async function comparePair(nativeArtifactBuffer, canvaskitArtifactBuffer) {
  let nativeBuffer = nativeArtifactBuffer;
  let canvaskitBuffer = canvaskitArtifactBuffer;
  const nativeImage = PNG.sync.read(nativeBuffer);
  const canvaskitImage = PNG.sync.read(canvaskitBuffer);
  let sizeNormalization = null;

  if (nativeImage.width !== canvaskitImage.width || nativeImage.height !== canvaskitImage.height) {
    const widthDelta = Math.abs(nativeImage.width - canvaskitImage.width);
    const heightDelta = Math.abs(nativeImage.height - canvaskitImage.height);
    if (widthDelta > MAX_CAPTURE_SIZE_DRIFT_PX || heightDelta > MAX_CAPTURE_SIZE_DRIFT_PX) {
      throw new Error(
        `이미지 크기 불일치: ${nativeImage.width}x${nativeImage.height} `
          + `vs ${canvaskitImage.width}x${canvaskitImage.height}`,
      );
    }

    const width = Math.min(nativeImage.width, canvaskitImage.width);
    const height = Math.min(nativeImage.height, canvaskitImage.height);
    sizeNormalization = {
      strategy: 'cropToCommonTopLeft',
      maxCaptureSizeDriftPx: MAX_CAPTURE_SIZE_DRIFT_PX,
      nativeSize: { width: nativeImage.width, height: nativeImage.height },
      canvaskitSize: { width: canvaskitImage.width, height: canvaskitImage.height },
      comparedSize: { width, height },
    };
    nativeBuffer = cropPngBuffer(nativeBuffer, { x: 0, y: 0, width, height });
    canvaskitBuffer = cropPngBuffer(canvaskitBuffer, { x: 0, y: 0, width, height });
  }

  const diff = await comparePngBuffers(
    nativeBuffer,
    canvaskitBuffer,
    {
      ignoreChannelDelta: DEFAULT_IGNORE_CHANNEL_DELTA,
      maxDiffRatio: DEFAULT_MAX_DIFF_RATIO,
    },
  );
  return {
    passed: diff.passed,
    passMetric: diff.passMetric,
    width: diff.width,
    height: diff.height,
    exactDiffPixels: diff.exactDiffPixels,
    exactDiffRatio: diff.exactDiffRatio,
    tolerantDiffPixels: diff.rawTolerantDiffPixels,
    tolerantDiffRatio: diff.rawTolerantDiffRatio,
    selectedDiffPixels: diff.diffPixels,
    selectedDiffRatio: diff.diffRatio,
    maxChannelDelta: diff.maxChannelDelta,
    meanAbsChannelDelta: diff.meanAbsChannelDelta,
    ignoreChannelDelta: diff.ignoreChannelDelta,
    maxDiffRatio: diff.maxDiffRatio,
    sizeNormalization,
  };
}

function emptySummary(keyField, keyValue) {
  return {
    [keyField]: keyValue,
    total: 0,
    compared: 0,
    passed: 0,
    failed: 0,
    missing: 0,
    errors: 0,
    identityMismatches: 0,
    worstSelectedDiffRatio: 0,
    worstMaxChannelDelta: 0,
  };
}

function addComparisonToSummary(summary, item) {
  summary.total += 1;
  if (item.status === 'missing') {
    summary.missing += 1;
    return;
  }
  if (item.status === 'error') {
    summary.errors += 1;
    return;
  }
  if (item.status === 'identityMismatch') {
    summary.identityMismatches += 1;
    return;
  }
  if (item.status !== 'compared') {
    return;
  }

  summary.compared += 1;
  if (item.diff?.passed) {
    summary.passed += 1;
  } else {
    summary.failed += 1;
  }
  if (typeof item.diff?.selectedDiffRatio === 'number') {
    summary.worstSelectedDiffRatio = Math.max(
      summary.worstSelectedDiffRatio,
      item.diff.selectedDiffRatio,
    );
  }
  if (typeof item.diff?.maxChannelDelta === 'number') {
    summary.worstMaxChannelDelta = Math.max(
      summary.worstMaxChannelDelta,
      item.diff.maxChannelDelta,
    );
  }
}

function summarizeBy(comparisons, keyField) {
  const summaries = new Map();
  for (const item of comparisons) {
    const keyValue = item[keyField] ?? '';
    if (!summaries.has(keyValue)) {
      summaries.set(keyValue, emptySummary(keyField, keyValue));
    }
    addComparisonToSummary(summaries.get(keyValue), item);
  }
  return [...summaries.values()].sort((a, b) => String(a[keyField]).localeCompare(String(b[keyField])));
}

function worstComparisons(comparisons, limit = 10) {
  return comparisons
    .filter((item) => item.status === 'compared')
    .map((item) => ({
      sampleId: item.sampleId,
      category: item.category ?? null,
      diagnosticAxes: item.diagnosticAxes ?? [],
      page: item.page ?? null,
      profile: item.profile,
      canvaskitSurface: item.canvaskitSurface ?? null,
      passed: !!item.diff?.passed,
      selectedDiffPixels: item.diff?.selectedDiffPixels ?? 0,
      selectedDiffRatio: item.diff?.selectedDiffRatio ?? 0,
      maxChannelDelta: item.diff?.maxChannelDelta ?? 0,
      meanAbsChannelDelta: item.diff?.meanAbsChannelDelta ?? 0,
    }))
    .sort((a, b) => (
      b.selectedDiffRatio - a.selectedDiffRatio
        || b.maxChannelDelta - a.maxChannelDelta
        || String(a.sampleId).localeCompare(String(b.sampleId))
        || String(a.profile).localeCompare(String(b.profile))
    ))
    .slice(0, limit);
}

const options = parseArgs();
if (options.selfTest) {
  const common = {
    schemaVersion: 1,
    sampleId: 'sample',
    documentDigest: `sha256:${'0'.repeat(64)}`,
    page: 0,
    profile: 'screen',
  };
  const nativeBuffer = Buffer.from('native baseline');
  const canvaskitBuffer = Buffer.from('canvaskit baseline');
  const nativeArtifact = {
    sha256: createHash('sha256').update(nativeBuffer).digest('hex'),
    sizeBytes: nativeBuffer.byteLength,
  };
  const canvaskitArtifact = {
    sha256: createHash('sha256').update(canvaskitBuffer).digest('hex'),
    sizeBytes: canvaskitBuffer.byteLength,
  };
  const matching = comparisonIdentityMismatches(
    { ...common, backend: 'native-skia' },
    { ...common, backend: 'canvaskit-default' },
    nativeArtifact,
    canvaskitArtifact,
    nativeBuffer,
    canvaskitBuffer,
  );
  const mismatching = comparisonIdentityMismatches(
    { ...common, backend: 'native-skia' },
    { ...common, documentDigest: `sha256:${'1'.repeat(64)}`, backend: 'canvaskit-default' },
    nativeArtifact,
    canvaskitArtifact,
    nativeBuffer,
    canvaskitBuffer,
  );
  const staleArtifact = comparisonIdentityMismatches(
    { ...common, backend: 'native-skia' },
    { ...common, backend: 'canvaskit-default' },
    { ...nativeArtifact, sha256: '1'.repeat(64) },
    canvaskitArtifact,
    nativeBuffer,
    canvaskitBuffer,
  );
  const wrongArtifactSize = comparisonIdentityMismatches(
    { ...common, backend: 'native-skia' },
    { ...common, backend: 'canvaskit-default' },
    { ...nativeArtifact, sizeBytes: nativeArtifact.sizeBytes + 1 },
    canvaskitArtifact,
    nativeBuffer,
    canvaskitBuffer,
  );
  if (
    matching.length !== 0
    || mismatching.join(',') !== 'documentDigest'
    || staleArtifact.join(',') !== 'nativeArtifactSha256'
    || wrongArtifactSize.join(',') !== 'nativeArtifactSizeBytes'
  ) {
    throw new Error('native/canvaskit comparison provenance self-test failed');
  }
  console.log('native/canvaskit comparison provenance self-test passed');
  process.exit(0);
}
const rootDir = path.resolve(new URL('../..', import.meta.url).pathname);
const nativeResults = JSON.parse(fs.readFileSync(options.native, 'utf8'));
const browserReport = JSON.parse(fs.readFileSync(options.browser, 'utf8'));
const browserResults = browserReport.results ?? [];
const browserCategoryBySample = new Map();
const browserAxesBySample = new Map();
for (const entry of browserResults) {
  if (entry.sampleId && entry.category && !browserCategoryBySample.has(entry.sampleId)) {
    browserCategoryBySample.set(entry.sampleId, entry.category);
  }
  if (entry.sampleId && Array.isArray(entry.diagnosticAxes) && !browserAxesBySample.has(entry.sampleId)) {
    browserAxesBySample.set(entry.sampleId, entry.diagnosticAxes);
  }
}
const profiles = options.profiles
  ? options.profiles.split(',').map((profile) => profile.trim()).filter(Boolean)
  : browserReport.profiles ?? [];

const sampleIds = [...new Set([
  ...nativeResults.map((entry) => entry.sampleId),
  ...browserResults.map((entry) => entry.sampleId),
])].sort();

const comparisons = [];
for (const sampleId of sampleIds) {
  for (const profile of profiles) {
    const native = nativeSkiaResult(nativeResults, sampleId, profile, rootDir);
    const nativePath = native.path;
    const canvaskit = canvaskitDefaultResult(browserResults, sampleId, profile, rootDir);
    const canvaskitPath = canvaskit.path;
    const canvaskitSurface = canvaskit.entry?.canvaskitSurface ?? browserReport.canvaskitSurface ?? null;
    const category = canvaskit.entry?.category
      ?? native.sample?.category
      ?? browserCategoryBySample.get(sampleId)
      ?? null;
    const diagnosticAxes = canvaskit.entry?.diagnosticAxes
      ?? native.sample?.diagnosticAxes
      ?? browserAxesBySample.get(sampleId)
      ?? [];
    const page = canvaskit.entry?.page ?? native.sample?.page ?? null;
    const documentDigest = canvaskit.entry?.documentDigest ?? native.sample?.documentDigest ?? null;
    if (!nativePath || !canvaskitPath) {
      comparisons.push({
        sampleId,
        category,
        diagnosticAxes,
        documentDigest,
        page,
        profile,
        canvaskitSurface,
        status: 'missing',
        nativePath: nativePath || null,
        canvaskitPath: canvaskitPath || null,
      });
      continue;
    }

    try {
      const nativeIdentity = native.backend?.comparisonIdentity ?? null;
      const canvaskitIdentity = canvaskit.entry?.comparisonIdentity ?? null;
      const nativeArtifact = native.backend?.artifact ?? null;
      const canvaskitArtifact = canvaskit.entry?.artifact ?? null;
      const nativeBuffer = fs.readFileSync(nativePath);
      const canvaskitBuffer = fs.readFileSync(canvaskitPath);
      const identityMismatches = comparisonIdentityMismatches(
        nativeIdentity,
        canvaskitIdentity,
        nativeArtifact,
        canvaskitArtifact,
        nativeBuffer,
        canvaskitBuffer,
      );
      if (identityMismatches.length > 0) {
        comparisons.push({
          sampleId,
          category,
          diagnosticAxes,
          documentDigest,
          page,
          profile,
          canvaskitSurface,
          status: 'identityMismatch',
          identityMismatches,
          nativeIdentity,
          canvaskitIdentity,
          nativeArtifact,
          canvaskitArtifact,
          nativePath,
          canvaskitPath,
        });
        continue;
      }
      comparisons.push({
        sampleId,
        category,
        diagnosticAxes,
        documentDigest,
        page,
        profile,
        canvaskitSurface,
        status: 'compared',
        nativePath,
        canvaskitPath,
        diff: await comparePair(nativeBuffer, canvaskitBuffer),
      });
    } catch (error) {
      comparisons.push({
        sampleId,
        category,
        diagnosticAxes,
        documentDigest,
        page,
        profile,
        canvaskitSurface,
        status: 'error',
        nativePath,
        canvaskitPath,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
}

const compared = comparisons.filter((item) => item.status === 'compared');
const passed = compared.filter((item) => item.diff?.passed).length;
const failed = compared.length - passed;
const missing = comparisons.filter((item) => item.status === 'missing').length;
const errors = comparisons.filter((item) => item.status === 'error').length;
const identityMismatches = comparisons.filter((item) => item.status === 'identityMismatch').length;
const summaryByProfile = summarizeBy(comparisons, 'profile');
const summaryBySample = summarizeBy(comparisons, 'sampleId');
const summaryByCategory = summarizeBy(comparisons, 'category');
const summaryByCanvasKitSurface = summarizeBy(comparisons, 'canvaskitSurface');
const diagnosticAxisSummaries = new Map();
for (const item of comparisons) {
  for (const diagnosticAxis of item.diagnosticAxes ?? []) {
    if (!diagnosticAxisSummaries.has(diagnosticAxis)) {
      diagnosticAxisSummaries.set(
        diagnosticAxis,
        emptySummary('diagnosticAxis', diagnosticAxis),
      );
    }
    addComparisonToSummary(diagnosticAxisSummaries.get(diagnosticAxis), item);
  }
}
const summaryByDiagnosticAxis = [...diagnosticAxisSummaries.values()]
  .sort((left, right) => left.diagnosticAxis.localeCompare(right.diagnosticAxis));
const worst = worstComparisons(comparisons);

fs.mkdirSync(path.dirname(options.output), { recursive: true });
fs.writeFileSync(
  options.output,
  JSON.stringify(
    {
      mode: 'reportOnly',
      backendPair: ['native-skia', 'canvaskit-default'],
      canvaskitSurface: browserReport.canvaskitSurface ?? null,
      thresholds: {
        ignoreChannelDelta: DEFAULT_IGNORE_CHANNEL_DELTA,
        maxDiffRatio: DEFAULT_MAX_DIFF_RATIO,
        maxCaptureSizeDriftPx: MAX_CAPTURE_SIZE_DRIFT_PX,
      },
      summary: {
        total: comparisons.length,
        compared: compared.length,
        passed,
        failed,
        missing,
        errors,
        identityMismatches,
      },
      summaryByProfile,
      summaryBySample,
      summaryByCategory,
      summaryByDiagnosticAxis,
      summaryByCanvasKitSurface,
      worstComparisons: worst,
      comparisons,
    },
    null,
    2,
  ),
);

console.log(`[baseline] native/canvaskit parity report: ${options.output}`);
