import fs from 'node:fs';
import path from 'node:path';

import {
  captureCanvasScreenshot,
  closeBrowser,
  closePage,
  comparePngBuffers,
  createPage,
  launchBrowser,
  loadApp,
  loadHwpFile,
} from './helpers.mjs';

const DEFAULT_BROWSER_PARITY_THRESHOLDS = {
  ignoreChannelDelta: 8,
  maxDiffRatio: 0.005,
};
const BROWSER_PARITY_RATIO_THRESHOLDS = new Set([
  'threshold',
  'maxDiffRatio',
  'inkMaskMaxDiffRatio',
  'nonInkMaxDiffRatio',
  'solidInkMaxDiffRatio',
]);
const BROWSER_PARITY_PIXEL_THRESHOLDS = new Set([
  'maxDiffPixels',
  'inkMaskMaxDiffPixels',
  'nonInkMaxDiffPixels',
  'solidInkMaxDiffPixels',
  'minimumInkPixels',
]);
const BROWSER_PARITY_BYTE_THRESHOLDS = new Set([
  'ignoreChannelDelta',
  'inkMaskWhiteDelta',
  'inkMaskAlphaThreshold',
]);
const BROWSER_PARITY_ALLOWED_THRESHOLDS = new Set([
  ...BROWSER_PARITY_RATIO_THRESHOLDS,
  ...BROWSER_PARITY_PIXEL_THRESHOLDS,
  ...BROWSER_PARITY_BYTE_THRESHOLDS,
  'inkMaskNeighborhoodRadius',
]);
const ALLOWED_CANVASKIT_SURFACES = new Set(['auto', 'webgpu', 'webgl', 'software']);
const BACKENDS = [
  {
    key: 'canvas2d',
    queryForProfile(profile) {
      return `?renderer=canvas2d&renderProfile=${encodeURIComponent(profile)}`;
    },
    filenameForProfile(profile) {
      return `canvas2d-${profile}.png`;
    },
  },
  {
    key: 'canvaskit-compat',
    queryForProfile(profile) {
      const surfaceQuery = options.canvaskitSurface === 'auto'
        ? ''
        : `&canvaskitSurface=${encodeURIComponent(options.canvaskitSurface)}`;
      return `?renderer=canvaskit&canvaskitMode=compat&renderProfile=${encodeURIComponent(profile)}${surfaceQuery}`;
    },
    filenameForProfile(profile) {
      const surfaceSuffix = options.canvaskitSurface === 'auto' ? '' : `-${options.canvaskitSurface}`;
      return `canvaskit-compat-${profile}${surfaceSuffix}.png`;
    },
  },
  {
    key: 'canvaskit-default',
    queryForProfile(profile) {
      const surfaceQuery = options.canvaskitSurface === 'auto'
        ? ''
        : `&canvaskitSurface=${encodeURIComponent(options.canvaskitSurface)}`;
      return `?renderer=canvaskit&canvaskitMode=default&renderProfile=${encodeURIComponent(profile)}${surfaceQuery}`;
    },
    filenameForProfile(profile) {
      const surfaceSuffix = options.canvaskitSurface === 'auto' ? '' : `-${options.canvaskitSurface}`;
      return `canvaskit-default-${profile}${surfaceSuffix}.png`;
    },
  },
];
const ALLOWED_PROFILES = new Set(['screen', 'print', 'high-quality', 'fast-preview']);

function browserParityThresholdsForSample(sample) {
  const sampleThresholds = sample?.browserParityThresholds;
  if (!sampleThresholds || typeof sampleThresholds !== 'object') {
    return { ...DEFAULT_BROWSER_PARITY_THRESHOLDS };
  }
  const thresholds = { ...DEFAULT_BROWSER_PARITY_THRESHOLDS };
  for (const [key, value] of Object.entries(sampleThresholds)) {
    if (!BROWSER_PARITY_ALLOWED_THRESHOLDS.has(key)) {
      throw new Error(`unsupported browser parity threshold for ${sample.id}: ${key}`);
    }
    const nullable = BROWSER_PARITY_PIXEL_THRESHOLDS.has(key)
      || (BROWSER_PARITY_RATIO_THRESHOLDS.has(key) && key !== 'threshold');
    if (value === null && nullable) {
      thresholds[key] = value;
      continue;
    }
    if (typeof value !== 'number' || !Number.isFinite(value)) {
      throw new Error(`browser parity threshold ${key} for ${sample.id} must be a finite number`);
    }
    if (BROWSER_PARITY_RATIO_THRESHOLDS.has(key) && (value < 0 || value > 1)) {
      throw new Error(`browser parity threshold ${key} for ${sample.id} must be within 0..1`);
    }
    if (BROWSER_PARITY_PIXEL_THRESHOLDS.has(key) && (!Number.isInteger(value) || value < 0)) {
      throw new Error(`browser parity threshold ${key} for ${sample.id} must be a non-negative integer`);
    }
    if (BROWSER_PARITY_BYTE_THRESHOLDS.has(key)
      && (!Number.isInteger(value) || value < 0 || value > 255)) {
      throw new Error(`browser parity threshold ${key} for ${sample.id} must be an integer within 0..255`);
    }
    if (key === 'inkMaskNeighborhoodRadius'
      && (!Number.isInteger(value) || value < 0 || value > 16)) {
      throw new Error(
        `browser parity threshold ${key} for ${sample.id} must be an integer within 0..16`,
      );
    }
    thresholds[key] = value;
  }
  return thresholds;
}

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    manifest: '',
    output: '',
    filter: '',
    profiles: 'screen,fast-preview',
    canvaskitSurface: process.env.RHWP_CANVASKIT_SURFACE ?? 'auto',
    readinessOnly: false,
  };

  for (const arg of args) {
    if (arg.startsWith('--manifest=')) {
      options.manifest = arg.slice('--manifest='.length);
      continue;
    }
    if (arg.startsWith('--output=')) {
      options.output = arg.slice('--output='.length);
      continue;
    }
    if (arg.startsWith('--filter=')) {
      options.filter = arg.slice('--filter='.length);
      continue;
    }
    if (arg.startsWith('--profiles=')) {
      options.profiles = arg.slice('--profiles='.length);
      continue;
    }
    if (arg.startsWith('--canvaskit-surface=')) {
      options.canvaskitSurface = arg.slice('--canvaskit-surface='.length);
      continue;
    }
    if (arg === '--readiness-only') {
      options.readinessOnly = true;
      continue;
    }
  }

  if (!options.manifest) {
    throw new Error('missing --manifest=/abs/path/to/manifest.json');
  }
  if (!options.output) {
    throw new Error('missing --output=/abs/path/to/output-dir');
  }
  return options;
}

function parseProfiles(rawProfiles) {
  const profiles = rawProfiles
    .split(',')
    .map((profile) => profile.trim().toLowerCase())
    .filter(Boolean);
  if (profiles.length === 0) {
    throw new Error('at least one layered render profile must be specified');
  }

  const deduped = [];
  const seen = new Set();
  for (const profile of profiles) {
    if (!ALLOWED_PROFILES.has(profile)) {
      throw new Error(`unsupported layered render profile: ${profile}`);
    }
    if (seen.has(profile)) {
      continue;
    }
    seen.add(profile);
    deduped.push(profile);
  }
  return deduped;
}

function normalizeSamples(manifest, filterText, readinessOnly) {
  const filter = filterText.trim().toLowerCase();
  return (manifest.samples ?? []).map((sample) => {
    const file = String(sample.file || '').trim();
    if (!file || file.includes('\0') || file.includes('\\') || file.includes('?') || file.includes('#')) {
      throw new Error(`invalid baseline sample file: ${sample.file}`);
    }
    if (file.startsWith('/') || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(file)) {
      throw new Error(`baseline sample file must be relative to /samples: ${sample.file}`);
    }
    let decoded = file;
    try {
      decoded = decodeURIComponent(file);
    } catch {
      throw new Error(`baseline sample file has invalid URL escaping: ${sample.file}`);
    }
    if (decoded !== file) {
      throw new Error(`baseline sample file must not use percent-encoding: ${sample.file}`);
    }
    const parts = file.split('/');
    if (parts.some((part) => !part || part === '.' || part === '..')) {
      throw new Error(`baseline sample file escapes /samples: ${sample.file}`);
    }

    const id = String(sample.id || path.basename(file, path.extname(file))).trim();
    if (!id || !/^[A-Za-z0-9._-]+$/.test(id)) {
      throw new Error(`invalid baseline sample id: ${sample.id}`);
    }
    const normalizedSample = {
      ...sample,
      id,
      file,
      category: sample.category || 'uncategorized',
      page: sample.page ?? 0,
    };
    const normalizedThresholds = browserParityThresholdsForSample(normalizedSample);
    if (readinessOnly
      && normalizedSample.canvaskitReadinessGate === true
      && (!Number.isInteger(normalizedThresholds.minimumInkPixels)
        || normalizedThresholds.minimumInkPixels <= 0)) {
      throw new Error(`readiness sample ${id} requires a positive minimumInkPixels threshold`);
    }
    return normalizedSample;
  }).filter((sample) => {
    if (readinessOnly && sample.canvaskitReadinessGate !== true) {
      return false;
    }
    if (!filter) {
      return true;
    }
    return String(sample.id).toLowerCase().includes(filter)
      || String(sample.file).toLowerCase().includes(filter)
      || String(sample.category).toLowerCase().includes(filter);
  });
}

async function resetRendererDiagnostics(page) {
  await page.evaluate(() => {
    const pageRenderer = window.__canvasView?.pageRenderer;
    pageRenderer?.canvas2dRenderer?.resetImageEffectDiagnostics?.();
    pageRenderer?.canvaskitRenderer?.resetImageEffectDiagnostics?.();
  });
}

async function readRendererDiagnostics(page, pageIndex) {
  return await page.evaluate((targetPageIndex) => {
    const pageRenderer = window.__canvasView?.pageRenderer;
    const canvas2d = pageRenderer?.canvas2dRenderer?.getImageEffectDiagnostics?.() ?? null;
    const canvaskit = pageRenderer?.canvaskitRenderer?.getImageEffectDiagnostics?.() ?? null;
    const surfaceDiagnostics = pageRenderer?.canvaskitRenderer?.getSurfaceDiagnostics?.() ?? null;
    const canvaskitRender = window.__canvasView
      ?.getCanvasKitRenderDiagnostics?.(targetPageIndex) ?? null;
    const trackedCanvas = window.__canvasView?.canvasPool?.getCanvas?.(targetPageIndex) ?? null;
    return {
      runtime: {
        activeBackend: window.__renderBackend ?? null,
        request: window.__rendererRuntimeRequest ?? null,
        backendFallbackReason: window.__renderBackendFallbackReason ?? null,
        canvasOwnershipTracked: trackedCanvas instanceof HTMLCanvasElement && trackedCanvas.isConnected,
      },
      imageEffects: {
        canvas2d,
        canvaskit,
      },
      surfaceDiagnostics,
      canvaskitRender,
    };
  }, pageIndex);
}

const options = parseArgs();
const requestedCanvasKitSurface = options.canvaskitSurface.trim().toLowerCase();
if (requestedCanvasKitSurface === 'sw' || requestedCanvasKitSurface === 'cpu') {
  options.canvaskitSurface = 'software';
} else if (requestedCanvasKitSurface === 'gpu') {
  options.canvaskitSurface = 'webgpu';
} else if (ALLOWED_CANVASKIT_SURFACES.has(requestedCanvasKitSurface)) {
  options.canvaskitSurface = requestedCanvasKitSurface;
} else {
  throw new Error(
    `unsupported CanvasKit surface: ${options.canvaskitSurface} `
      + `(allowed: ${[...ALLOWED_CANVASKIT_SURFACES].join(', ')}; aliases: gpu, sw, cpu)`,
  );
}
const manifest = JSON.parse(fs.readFileSync(options.manifest, 'utf8'));
if (options.readinessOnly && options.filter.trim()) {
  throw new Error('--readiness-only cannot be combined with --filter');
}
const samples = normalizeSamples(manifest, options.filter, options.readinessOnly);
const profiles = parseProfiles(options.profiles);
if (options.readinessOnly && (profiles.length !== 1 || profiles[0] !== 'screen')) {
  throw new Error('--readiness-only requires --profiles=screen');
}
if (options.readinessOnly && options.canvaskitSurface !== 'auto') {
  throw new Error('--readiness-only requires --canvaskit-surface=auto');
}
const activeBackends = options.readinessOnly
  ? BACKENDS.filter((backend) => backend.key !== 'canvaskit-compat')
  : BACKENDS;

if (samples.length === 0) {
  throw new Error('manifest filter removed every sample');
}

fs.mkdirSync(options.output, { recursive: true });

const results = [];
let browser = null;
let page = null;
let browserVersion = null;
const chromiumBuildId = process.env.RHWP_CHROMIUM_BUILD_ID ?? null;
let captureError = null;

try {
  browser = await launchBrowser();
  browserVersion = await browser.version();
  page = await createPage(browser, 1280, 900);
  for (const sample of samples) {
    if (sample.page !== 0) {
      throw new Error(
        `browser baseline currently supports only page=0 samples: ${sample.id} requested page=${sample.page}`,
      );
    }

    console.log(`\n[baseline] ${sample.id} (${sample.category})`);

    for (const profile of profiles) {
      for (const backend of activeBackends) {
        const totalStartedAt = performance.now();
        const appLoadStartedAt = performance.now();
        await loadApp(page, backend.queryForProfile(profile));
        const appLoadMs = performance.now() - appLoadStartedAt;

        await resetRendererDiagnostics(page);
        const documentLoadStartedAt = performance.now();
        await loadHwpFile(page, sample.file);
        const documentLoadAndInitialRenderMs = performance.now() - documentLoadStartedAt;

        const sampleDir = path.join(options.output, sample.id);
        const outputPath = path.join(sampleDir, backend.filenameForProfile(profile));
        const screenshotStartedAt = performance.now();
        await captureCanvasScreenshot(page, outputPath, `Baseline ${backend.key} (${profile})`);
        const screenshotMs = performance.now() - screenshotStartedAt;
        const diagnostics = await readRendererDiagnostics(page, sample.page);
        results.push({
          sampleId: sample.id,
          file: sample.file,
          category: sample.category,
          backend: backend.key,
          profile,
          canvaskitSurface: backend.key.startsWith('canvaskit') ? options.canvaskitSurface : null,
          readinessGateRequired: options.readinessOnly
            && sample.canvaskitReadinessGate === true
            && backend.key === 'canvaskit-default'
            && profile === 'screen'
            && options.canvaskitSurface === 'auto',
          path: outputPath,
          timings: {
            appLoadMs,
            documentLoadAndInitialRenderMs,
            screenshotMs,
            totalMs: performance.now() - totalStartedAt,
          },
          diagnostics,
        });
      }
    }
  }
} catch (error) {
  captureError = error instanceof Error ? error.message : String(error);
  console.error(`[baseline] browser capture failed: ${captureError}`);
} finally {
  if (page) await closePage(page).catch(() => {});
  if (browser) await closeBrowser(browser).catch(() => {});
}

const reportPath = path.join(options.output, 'browser-baseline-report.json');
const browserBackendComparisons = [];
for (const sample of samples) {
  for (const profile of profiles) {
    const baseline = results.find((entry) => (
      entry.sampleId === sample.id
        && entry.backend === 'canvas2d'
        && entry.profile === profile
    ));
    for (const targetBackend of activeBackends
      .map((backend) => backend.key)
      .filter((backend) => backend.startsWith('canvaskit'))) {
      const target = results.find((entry) => (
        entry.sampleId === sample.id
          && entry.backend === targetBackend
          && entry.profile === profile
      ));
      if (!baseline || !target) {
        browserBackendComparisons.push({
          sampleId: sample.id,
          category: sample.category,
          profile,
          baselineBackend: 'canvas2d',
          targetBackend,
          status: 'missing',
          baselinePath: baseline?.path ?? null,
          targetPath: target?.path ?? null,
        });
        continue;
      }

      try {
        const thresholds = browserParityThresholdsForSample(sample);
        const diff = await comparePngBuffers(
          fs.readFileSync(baseline.path),
          fs.readFileSync(target.path),
          thresholds,
        );
        browserBackendComparisons.push({
          sampleId: sample.id,
          category: sample.category,
          profile,
          baselineBackend: 'canvas2d',
          targetBackend,
          canvaskitSurface: target.canvaskitSurface ?? null,
          status: 'compared',
          baselinePath: baseline.path,
          targetPath: target.path,
          thresholds,
          diff: {
            passed: diff.passed,
            hasVisualBudget: diff.hasVisualBudget,
            passMetric: diff.passMetric,
            width: diff.width,
            height: diff.height,
            exactDiffPixels: diff.exactDiffPixels,
            exactDiffRatio: diff.exactDiffRatio,
            tolerantDiffPixels: diff.rawTolerantDiffPixels,
            tolerantDiffRatio: diff.rawTolerantDiffRatio,
            inkMaskDiffPixels: diff.rawInkMaskDiffPixels,
            inkMaskDiffRatio: diff.rawInkMaskDiffRatio,
            nonInkDiffPixels: diff.rawNonInkDiffPixels,
            nonInkDiffRatio: diff.rawNonInkDiffRatio,
            solidInkDiffPixels: diff.rawSolidInkDiffPixels,
            solidInkDiffRatio: diff.rawSolidInkDiffRatio,
            tolerantBudgetPassed: diff.tolerantBudgetPassed,
            inkMaskBudgetPassed: diff.inkMaskBudgetPassed,
            nonInkBudgetPassed: diff.nonInkBudgetPassed,
            solidInkBudgetPassed: diff.solidInkBudgetPassed,
            expectedInkPixels: diff.expectedInkPixels,
            actualInkPixels: diff.actualInkPixels,
            minimumInkPixels: diff.minimumInkPixels,
            minimumInkBudgetPassed: diff.minimumInkBudgetPassed,
            selectedDiffPixels: diff.diffPixels,
            selectedDiffRatio: diff.diffRatio,
            maxChannelDelta: diff.maxChannelDelta,
            meanAbsChannelDelta: diff.meanAbsChannelDelta,
          },
        });
      } catch (error) {
        browserBackendComparisons.push({
          sampleId: sample.id,
          category: sample.category,
          profile,
          baselineBackend: 'canvas2d',
          targetBackend,
          canvaskitSurface: target.canvaskitSurface ?? null,
          status: 'error',
          baselinePath: baseline.path,
          targetPath: target.path,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }
  }
}
const browserBackendCompared = browserBackendComparisons.filter((item) => item.status === 'compared');
const browserBackendSummaryByTarget = new Map();
const browserBackendSummaryByProfile = new Map();
const browserBackendSummaryByCategory = new Map();
for (const item of browserBackendComparisons) {
  for (const [summaryMap, keyField, keyValue] of [
    [browserBackendSummaryByTarget, 'targetBackend', item.targetBackend],
    [browserBackendSummaryByProfile, 'profile', item.profile],
    [browserBackendSummaryByCategory, 'category', item.category],
  ]) {
    if (!summaryMap.has(keyValue)) {
      summaryMap.set(keyValue, {
        [keyField]: keyValue,
        total: 0,
        compared: 0,
        passed: 0,
        failed: 0,
        missing: 0,
        errors: 0,
        worstSelectedDiffRatio: 0,
        worstTolerantDiffRatio: 0,
        worstMaxChannelDelta: 0,
      });
    }
    const summary = summaryMap.get(keyValue);
    summary.total += 1;
    if (item.status === 'missing') {
      summary.missing += 1;
      continue;
    }
    if (item.status === 'error') {
      summary.errors += 1;
      continue;
    }
    if (item.status !== 'compared') {
      continue;
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
    if (typeof item.diff?.tolerantDiffRatio === 'number') {
      summary.worstTolerantDiffRatio = Math.max(
        summary.worstTolerantDiffRatio,
        item.diff.tolerantDiffRatio,
      );
    }
    if (typeof item.diff?.maxChannelDelta === 'number') {
      summary.worstMaxChannelDelta = Math.max(
        summary.worstMaxChannelDelta,
        item.diff.maxChannelDelta,
      );
    }
  }
}
const browserBackendParity = {
  mode: 'reportOnly',
  backendPairs: [
    ...activeBackends
      .map((backend) => backend.key)
      .filter((backend) => backend.startsWith('canvaskit'))
      .map((backend) => ['canvas2d', backend]),
  ],
  thresholds: DEFAULT_BROWSER_PARITY_THRESHOLDS,
  summary: {
    total: browserBackendComparisons.length,
    compared: browserBackendCompared.length,
    passed: browserBackendCompared.filter((item) => item.diff?.passed).length,
    failed: browserBackendCompared.filter((item) => !item.diff?.passed).length,
    missing: browserBackendComparisons.filter((item) => item.status === 'missing').length,
    errors: browserBackendComparisons.filter((item) => item.status === 'error').length,
  },
  summaryByTargetBackend: [...browserBackendSummaryByTarget.values()]
    .sort((left, right) => left.targetBackend.localeCompare(right.targetBackend)),
  summaryByProfile: [...browserBackendSummaryByProfile.values()]
    .sort((left, right) => left.profile.localeCompare(right.profile)),
  summaryByCategory: [...browserBackendSummaryByCategory.values()]
    .sort((left, right) => String(left.category).localeCompare(String(right.category))),
  worstComparisons: browserBackendCompared
    .map((item) => ({
      sampleId: item.sampleId,
      category: item.category,
      profile: item.profile,
      targetBackend: item.targetBackend,
      canvaskitSurface: item.canvaskitSurface ?? null,
      passed: !!item.diff?.passed,
      selectedDiffPixels: item.diff?.selectedDiffPixels ?? 0,
      selectedDiffRatio: item.diff?.selectedDiffRatio ?? 0,
      tolerantDiffRatio: item.diff?.tolerantDiffRatio ?? 0,
      maxChannelDelta: item.diff?.maxChannelDelta ?? 0,
      meanAbsChannelDelta: item.diff?.meanAbsChannelDelta ?? 0,
    }))
    .sort((left, right) => (
      right.selectedDiffRatio - left.selectedDiffRatio
        || right.tolerantDiffRatio - left.tolerantDiffRatio
        || right.maxChannelDelta - left.maxChannelDelta
        || left.sampleId.localeCompare(right.sampleId)
        || left.targetBackend.localeCompare(right.targetBackend)
        || left.profile.localeCompare(right.profile)
    ))
    .slice(0, 10),
  comparisons: browserBackendComparisons,
};

const canvaskitReadinessChecks = [];
for (const result of results.filter((entry) => entry.readinessGateRequired)) {
  const runtime = result.diagnostics?.runtime ?? {};
  const renderDiagnostics = result.diagnostics?.canvaskitRender ?? null;
  const comparison = browserBackendComparisons.find((entry) => (
    entry.sampleId === result.sampleId
      && entry.profile === result.profile
      && entry.targetBackend === result.backend
  ));
  const blockers = [];
  if (runtime.activeBackend !== 'canvaskit') {
    blockers.push('backendNotActive');
  }
  if (runtime.request?.backend?.backend !== 'canvaskit'
    || runtime.request?.backend?.source !== 'url') {
    blockers.push('explicitCanvasKitRequestMissing');
  }
  if (runtime.request?.canvaskitMode?.mode !== 'default'
    || runtime.request?.canvaskitMode?.source !== 'url') {
    blockers.push('canvaskitModeRequestMismatch');
  }
  if (runtime.request?.canvaskitSurface?.preference !== 'auto'
    || runtime.request?.canvaskitSurface?.requested !== 'auto') {
    blockers.push('canvaskitSurfaceRequestMismatch');
  }
  if (runtime.backendFallbackReason !== null && runtime.backendFallbackReason !== undefined) {
    blockers.push(`backendFallback:${runtime.backendFallbackReason}`);
  }
  if (runtime.canvasOwnershipTracked !== true) {
    blockers.push('canvasOwnershipMismatch');
  }
  if (renderDiagnostics === null) {
    blockers.push('diagnosticsUnavailable');
  } else {
    if (renderDiagnostics.mode !== 'default') {
      blockers.push('canvaskitModeMismatch');
    }
    if (renderDiagnostics.surfacePreference !== 'auto') {
      blockers.push('canvaskitSurfacePreferenceMismatch');
    }
    for (const blocker of renderDiagnostics.readinessBlockers ?? []) {
      blockers.push(`runtime:${blocker}`);
    }
    if (renderDiagnostics.passesRuntimeReadinessGate !== true) {
      blockers.push('runtime:readinessGateFailed');
    }
  }
  if (!comparison || comparison.status === 'missing') {
    blockers.push('visualComparisonMissing');
  } else if (comparison.status === 'error') {
    blockers.push('visualComparisonError');
  } else if (comparison.diff?.hasVisualBudget !== true) {
    blockers.push('visualThresholdMissing');
  } else if (comparison.diff?.passed !== true) {
    blockers.push('visualParityFailed');
  }

  canvaskitReadinessChecks.push({
    sampleId: result.sampleId,
    category: result.category,
    profile: result.profile,
    targetBackend: result.backend,
    canvaskitSurface: result.canvaskitSurface,
    passed: blockers.length === 0,
    blockers: [...new Set(blockers)].sort(),
    activeBackend: runtime.activeBackend ?? null,
    canvasOwnershipTracked: runtime.canvasOwnershipTracked === true,
    request: runtime.request ?? null,
    backendFallbackReason: runtime.backendFallbackReason ?? null,
    expectedUnsupportedOps: renderDiagnostics?.lastExpectedUnsupportedOps ?? [],
    unexpectedUnsupportedOps: renderDiagnostics?.lastUnexpectedUnsupportedOps ?? [],
    renderError: renderDiagnostics?.lastRenderError ?? null,
    surfaceBackend: renderDiagnostics?.surfaceBackend ?? null,
    surfaceFallbackReason: renderDiagnostics?.surfaceFallbackReason ?? null,
    visualComparisonStatus: comparison?.status ?? 'missing',
    visualComparisonPassed: comparison?.diff?.passed ?? false,
    selectedDiffRatio: comparison?.diff?.selectedDiffRatio ?? null,
    expectedInkPixels: comparison?.diff?.expectedInkPixels ?? null,
    actualInkPixels: comparison?.diff?.actualInkPixels ?? null,
    minimumInkPixels: comparison?.diff?.minimumInkPixels ?? null,
    minimumInkBudgetPassed: comparison?.diff?.minimumInkBudgetPassed ?? false,
  });
}
const missingReadinessChecks = options.readinessOnly
  ? Math.max(0, samples.length - canvaskitReadinessChecks.length)
  : 0;
const failedReadinessChecks = canvaskitReadinessChecks.filter((entry) => !entry.passed).length
  + missingReadinessChecks;
const canvaskitReadinessGate = {
  mode: 'selectedCorpus',
  criteria: {
    sampleFlag: 'canvaskitReadinessGate',
    profile: 'screen',
    targetBackend: 'canvaskit-default',
    canvaskitSurface: 'auto',
    requireActiveBackend: true,
    requireRuntimeReadiness: true,
    requireVisualParity: true,
  },
  summary: {
    total: options.readinessOnly ? samples.length : 0,
    evaluated: canvaskitReadinessChecks.length,
    passed: canvaskitReadinessChecks.filter((entry) => entry.passed).length,
    failed: captureError && failedReadinessChecks === 0 ? 1 : failedReadinessChecks,
    missing: missingReadinessChecks,
  },
  captureError,
  checks: canvaskitReadinessChecks,
};

fs.writeFileSync(
  reportPath,
  JSON.stringify(
    {
      manifest: options.manifest,
      browserVersion,
      chromiumBuildId,
      captureError,
      sampleCount: samples.length,
      profiles,
      canvaskitSurface: options.canvaskitSurface,
      results,
      browserBackendParity,
      canvaskitReadinessGate,
    },
    null,
    2,
  ),
);
console.log(`\n[baseline] browser report: ${reportPath}`);
if (canvaskitReadinessGate.summary.failed > 0) {
  for (const check of canvaskitReadinessChecks.filter((entry) => !entry.passed)) {
    console.error(
      `[baseline] CanvasKit readiness failed: ${check.sampleId} (${check.blockers.join(', ')})`,
    );
  }
  process.exitCode = 1;
}
if (captureError && !options.readinessOnly) {
  process.exitCode = 1;
}
