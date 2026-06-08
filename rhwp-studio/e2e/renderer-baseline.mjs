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
    if (value === null || (typeof value === 'number' && Number.isFinite(value))) {
      thresholds[key] = value;
    }
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

function normalizeSamples(manifest, filterText) {
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
    return {
      ...sample,
      id,
      file,
      category: sample.category || 'uncategorized',
      page: sample.page ?? 0,
    };
  }).filter((sample) => {
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

async function readRendererDiagnostics(page) {
  return await page.evaluate(() => {
    const pageRenderer = window.__canvasView?.pageRenderer;
    const canvas2d = pageRenderer?.canvas2dRenderer?.getImageEffectDiagnostics?.() ?? null;
    const canvaskit = pageRenderer?.canvaskitRenderer?.getImageEffectDiagnostics?.() ?? null;
    const surfaceDiagnostics = pageRenderer?.canvaskitRenderer?.getSurfaceDiagnostics?.() ?? null;
    return {
      imageEffects: {
        canvas2d,
        canvaskit,
      },
      surfaceDiagnostics,
    };
  });
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
const samples = normalizeSamples(manifest, options.filter);
const profiles = parseProfiles(options.profiles);

if (samples.length === 0) {
  throw new Error('manifest filter removed every sample');
}

fs.mkdirSync(options.output, { recursive: true });

const results = [];
const browser = await launchBrowser();
const page = await createPage(browser, 1280, 900);

try {
  for (const sample of samples) {
    if (sample.page !== 0) {
      throw new Error(
        `browser baseline currently supports only page=0 samples: ${sample.id} requested page=${sample.page}`,
      );
    }

    console.log(`\n[baseline] ${sample.id} (${sample.category})`);

    for (const profile of profiles) {
      for (const backend of BACKENDS) {
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
        const diagnostics = await readRendererDiagnostics(page);
        results.push({
          sampleId: sample.id,
          file: sample.file,
          category: sample.category,
          backend: backend.key,
          profile,
          canvaskitSurface: backend.key.startsWith('canvaskit') ? options.canvaskitSurface : null,
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
} finally {
  await closePage(page).catch(() => {});
  await closeBrowser(browser).catch(() => {});
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
    for (const targetBackend of ['canvaskit-compat', 'canvaskit-default']) {
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
    ['canvas2d', 'canvaskit-compat'],
    ['canvas2d', 'canvaskit-default'],
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

fs.writeFileSync(
  reportPath,
  JSON.stringify(
    {
      manifest: options.manifest,
      sampleCount: samples.length,
      profiles,
      canvaskitSurface: options.canvaskitSurface,
      results,
      browserBackendParity,
    },
    null,
    2,
  ),
);
console.log(`\n[baseline] browser report: ${reportPath}`);
