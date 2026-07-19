import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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
const BASELINE_CAPTURE_CONTAINER_ID = 'renderer-baseline-page-container';
const BASELINE_CAPTURE_CONTAINER_SELECTOR = `#${BASELINE_CAPTURE_CONTAINER_ID}`;
const BASELINE_CAPTURE_CANVAS_ID = 'renderer-baseline-page-canvas';
const STUDIO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REPO_ROOT = path.resolve(STUDIO_ROOT, '..');
const SAMPLES_ROOT = path.resolve(REPO_ROOT, 'samples');
const SAMPLES_ROOT_REAL = fs.realpathSync(SAMPLES_ROOT);
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
const CANVASKIT_PERFORMANCE_BUDGET_KEYS = new Set([
  'maxColdDocumentLoadAndInitialRenderMs',
  'maxWarmReplayMs',
  'maxWarmRendererDurationMs',
  'maxImageCachePixels',
]);
const CANVASKIT_READINESS_EXPECTATION_KEYS = new Set([
  'glyphOutlinePayloadKinds',
  'minWarmImageCacheHits',
]);
const CANVASKIT_GLYPH_RESOURCE_PAYLOAD_KINDS = new Set(['bitmapGlyph', 'svgGlyph']);
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
      if (options.readinessOnly) {
        return `?renderer=auto&canvaskitMode=default&renderProfile=${encodeURIComponent(profile)}${surfaceQuery}`;
      }
      return `?renderer=canvaskit&canvaskitMode=default&renderProfile=${encodeURIComponent(profile)}${surfaceQuery}`;
    },
    filenameForProfile(profile) {
      const surfaceSuffix = options.canvaskitSurface === 'auto' ? '' : `-${options.canvaskitSurface}`;
      return `canvaskit-default-${profile}${surfaceSuffix}.png`;
    },
  },
];
const ALLOWED_PROFILES = new Set(['screen', 'print', 'high-quality', 'fast-preview']);

function toLayerRenderProfile(profile) {
  if (profile === 'high-quality') return 'highQuality';
  if (profile === 'fast-preview') return 'fastPreview';
  return profile;
}

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

function canvaskitPerformanceBudgetForSample(sample) {
  const budget = sample?.canvaskitPerformanceBudget;
  if (!budget || typeof budget !== 'object') return null;
  for (const [key, value] of Object.entries(budget)) {
    if (!CANVASKIT_PERFORMANCE_BUDGET_KEYS.has(key)) {
      throw new Error(`unsupported CanvasKit performance budget for ${sample.id}: ${key}`);
    }
    if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) {
      throw new Error(`CanvasKit performance budget ${key} for ${sample.id} must be positive`);
    }
  }
  if ([...CANVASKIT_PERFORMANCE_BUDGET_KEYS].some((key) => budget[key] === undefined)) {
    throw new Error(`CanvasKit performance budget for ${sample.id} must define every budget key`);
  }
  return { ...budget };
}

function canvaskitReadinessExpectationsForSample(sample) {
  const expectations = sample?.canvaskitReadinessExpectations;
  if (expectations === undefined) return null;
  if (!expectations || typeof expectations !== 'object' || Array.isArray(expectations)) {
    throw new Error(`CanvasKit readiness expectations for ${sample.id} must be an object`);
  }
  for (const key of Object.keys(expectations)) {
    if (!CANVASKIT_READINESS_EXPECTATION_KEYS.has(key)) {
      throw new Error(`unsupported CanvasKit readiness expectation for ${sample.id}: ${key}`);
    }
  }
  const payloadKinds = expectations.glyphOutlinePayloadKinds ?? [];
  if (!Array.isArray(payloadKinds)
    || payloadKinds.some((kind) => !CANVASKIT_GLYPH_RESOURCE_PAYLOAD_KINDS.has(kind))) {
    throw new Error(`CanvasKit glyph payload expectations for ${sample.id} are invalid`);
  }
  const minWarmImageCacheHits = expectations.minWarmImageCacheHits ?? 0;
  if (!Number.isInteger(minWarmImageCacheHits) || minWarmImageCacheHits < 0) {
    throw new Error(`CanvasKit warm image cache expectation for ${sample.id} must be non-negative`);
  }
  return {
    glyphOutlinePayloadKinds: [...new Set(payloadKinds)],
    minWarmImageCacheHits,
  };
}

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    manifest: '',
    output: '',
    filter: '',
    scope: 'representative',
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
    if (arg.startsWith('--scope=')) {
      options.scope = arg.slice('--scope='.length);
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
  if (options.scope !== 'representative' && options.scope !== 'full') {
    throw new Error(`unsupported baseline scope: ${options.scope}`);
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

function normalizeSamples(manifest, filterText, scope, readinessOnly) {
  if (manifest.schemaVersion !== 1) {
    throw new Error('baseline manifest schemaVersion must be 1');
  }
  const filter = filterText.trim().toLowerCase();
  const seenSampleIds = new Set();
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
    if (seenSampleIds.has(id)) {
      throw new Error(`duplicate baseline sample id: ${id}`);
    }
    seenSampleIds.add(id);
    const normalizedSample = {
      ...sample,
      id,
      file,
      category: sample.category || 'uncategorized',
      page: sample.page ?? 0,
    };
    if (!Number.isInteger(normalizedSample.page) || normalizedSample.page < 0) {
      throw new Error(`baseline sample page must be a non-negative integer: ${id}`);
    }
    if (sample.baselineTier !== 'representative' && sample.baselineTier !== 'extended') {
      throw new Error(`invalid baselineTier for baseline sample: ${id}`);
    }
    const diagnosticAxes = sample.diagnosticAxes;
    if (!Array.isArray(diagnosticAxes)
      || diagnosticAxes.length === 0
      || diagnosticAxes.length > 16
      || diagnosticAxes.some((axis) => (
        typeof axis !== 'string'
          || axis.length === 0
          || axis.length > 64
          || !/^[A-Za-z0-9._-]+$/.test(axis)
      ))
      || new Set(diagnosticAxes).size !== diagnosticAxes.length) {
      throw new Error(`invalid diagnosticAxes for baseline sample: ${id}`);
    }
    const samplePath = path.resolve(SAMPLES_ROOT, ...parts);
    if (!fs.existsSync(samplePath)) {
      throw new Error(`baseline sample file is unavailable under samples/: ${file}`);
    }
    const realSamplePath = fs.realpathSync(samplePath);
    const sampleRelativePath = path.relative(SAMPLES_ROOT_REAL, realSamplePath);
    if (!sampleRelativePath
      || sampleRelativePath === '..'
      || sampleRelativePath.startsWith(`..${path.sep}`)
      || path.isAbsolute(sampleRelativePath)
      || !fs.statSync(realSamplePath).isFile()) {
      throw new Error(`baseline sample file escapes samples/: ${file}`);
    }
    const computedDocumentDigest = `sha256:${createHash('sha256')
      .update(fs.readFileSync(realSamplePath))
      .digest('hex')}`;
    const documentDigest = sample.documentDigest ?? computedDocumentDigest;
    if (!/^sha256:[0-9a-f]{64}$/.test(documentDigest)
      || documentDigest !== computedDocumentDigest) {
      throw new Error(`baseline sample documentDigest mismatch: ${id}`);
    }
    normalizedSample.diagnosticAxes = [...diagnosticAxes];
    normalizedSample.documentDigest = documentDigest;
    const normalizedThresholds = browserParityThresholdsForSample(normalizedSample);
    normalizedSample.canvaskitPerformanceBudget = canvaskitPerformanceBudgetForSample(normalizedSample);
    normalizedSample.canvaskitReadinessExpectations = canvaskitReadinessExpectationsForSample(normalizedSample);
    if (readinessOnly
      && normalizedSample.canvaskitReadinessGate === true
      && (!Number.isInteger(normalizedThresholds.minimumInkPixels)
        || normalizedThresholds.minimumInkPixels <= 0)) {
      throw new Error(`readiness sample ${id} requires a positive minimumInkPixels threshold`);
    }
    if (readinessOnly
      && normalizedSample.canvaskitReadinessGate === true
      && normalizedSample.canvaskitPerformanceBudget === null) {
      throw new Error(`readiness sample ${id} requires a CanvasKit performance budget`);
    }
    return normalizedSample;
  }).filter((sample) => {
    if (scope === 'representative' && sample.baselineTier !== 'representative') {
      return false;
    }
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

async function readRendererDiagnostics(page, pageIndex, backendKey, profile) {
  const layerProfile = toLayerRenderProfile(profile);
  return await page.evaluate(({ targetBackend, targetPageIndex, targetProfile }) => {
    const pageRenderer = window.__canvasView?.pageRenderer;
    const canvas2d = pageRenderer?.canvas2dRenderer?.getImageEffectDiagnostics?.() ?? null;
    const canvaskit = pageRenderer?.canvaskitRenderer?.getImageEffectDiagnostics?.() ?? null;
    const surfaceDiagnostics = pageRenderer?.canvaskitRenderer?.getSurfaceDiagnostics?.() ?? null;
    const canvaskitRender = window.__canvasView
      ?.getCanvasKitRenderDiagnostics?.(targetPageIndex) ?? null;
    const selection = window.__canvasView?.getRendererSessionDiagnostics?.() ?? null;
    const trackedCanvas = window.__canvasView?.canvasPool?.getCanvas?.(targetPageIndex) ?? null;
    let replayPlan = null;
    let replayPlanError = null;
    if (targetBackend.startsWith('canvaskit')) {
      try {
        const mode = targetBackend === 'canvaskit-compat' ? 'compat' : 'default';
        const rawPlan = window.__wasm?.getCanvasKitReplayPlan?.(
          targetPageIndex,
          mode,
          targetProfile,
        );
        replayPlan = typeof rawPlan === 'string' ? JSON.parse(rawPlan) : rawPlan ?? null;
      } catch (error) {
        replayPlanError = error instanceof Error ? error.message : String(error);
      }
    }
    return {
      runtime: {
        activeBackend: window.__renderBackend ?? null,
        request: window.__rendererRuntimeRequest ?? null,
        backendFallbackReason: window.__renderBackendFallbackReason ?? null,
        selection,
        canvasOwnershipTracked: trackedCanvas instanceof HTMLCanvasElement && trackedCanvas.isConnected,
      },
      imageEffects: {
        canvas2d,
        canvaskit,
      },
      surfaceDiagnostics,
      canvaskitRender,
      replayPlan,
      replayPlanError,
    };
  }, { targetBackend: backendKey, targetPageIndex: pageIndex, targetProfile: layerProfile });
}

async function readLayerFeatureProbe(page, pageIndex, profile) {
  const layerProfile = toLayerRenderProfile(profile);
  return await page.evaluate(([targetPageIndex, targetProfile]) => {
    const tree = window.__wasm?.getPageLayerTreeObject?.(targetPageIndex, targetProfile);
    if (!tree?.root) return null;
    const glyphOutlinePayloadCounts = {};
    const stack = [tree.root];
    while (stack.length > 0) {
      const node = stack.pop();
      if (node?.kind === 'group') {
        stack.push(...(node.children ?? []));
      } else if (node?.kind === 'clipRect') {
        if (node.child) stack.push(node.child);
      } else if (node?.kind === 'leaf') {
        for (const op of node.ops ?? []) {
          if (op?.type !== 'glyphOutline' || typeof op.payloadKind !== 'string') continue;
          glyphOutlinePayloadCounts[op.payloadKind] = (glyphOutlinePayloadCounts[op.payloadKind] ?? 0) + 1;
        }
      }
    }
    return { glyphOutlinePayloadCounts };
  }, [pageIndex, layerProfile]);
}

async function measureWarmCanvasKitReplay(page, pageIndex) {
  return await page.evaluate(async (targetPageIndex) => {
    const view = window.__canvasView;
    const before = view?.getCurrentCanvasKitRenderDiagnostics?.() ?? null;
    const startedAt = performance.now();
    const rerendered = view?.rerenderPageForDiagnostics?.(targetPageIndex) === true;
    const replayMs = performance.now() - startedAt;
    const after = view?.getCurrentCanvasKitRenderDiagnostics?.() ?? null;
    return {
      replayMs,
      rerendered,
      rendererDurationMs: after?.lastRenderDurationMs ?? null,
      renderCountDelta: before && after ? after.renderCount - before.renderCount : null,
      imageCacheHitDelta: before && after ? after.imageCacheHits - before.imageCacheHits : null,
      imageCacheMissDelta: before && after ? after.imageCacheMisses - before.imageCacheMisses : null,
      imageCachePixels: after?.imageCachePixels ?? null,
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
const samples = normalizeSamples(manifest, options.filter, options.scope, options.readinessOnly);
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
for (const sample of samples) {
  if (!Number.isInteger(sample.page) || sample.page < 0) {
    throw new Error(`baseline sample page must be a non-negative integer: ${sample.id} page=${sample.page}`);
  }
}

fs.mkdirSync(options.output, { recursive: true });

const results = [];
const hardGateViolations = [];
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
    console.log(`\n[baseline] ${sample.id} (${sample.category}, page=${sample.page})`);

    for (const profile of profiles) {
      for (const backend of activeBackends) {
        const totalStartedAt = performance.now();
        const appLoadStartedAt = performance.now();
        await loadApp(page, backend.queryForProfile(profile));
        const appLoadMs = performance.now() - appLoadStartedAt;

        const loadResult = await loadHwpFile(page, sample.file);
        const documentLoadAndInitialRenderMs = loadResult.documentLoadAndInitialRenderMs;
        if (sample.page >= loadResult.pageCount) {
          throw new Error(
            `baseline sample page is out of range: ${sample.id} page=${sample.page} pageCount=${loadResult.pageCount}`,
          );
        }

        await page.evaluate(() => window.__canvasView?.pageRenderer?.cancelAll?.());
        await resetRendererDiagnostics(page);
        const selectedPageRenderStartedAt = performance.now();
        await page.evaluate(
          ({ captureCanvasId, captureContainerId, capturePageIndex }) => {
            const pageRenderer = window.__canvasView?.pageRenderer;
            const wasm = window.__wasm;
            if (!pageRenderer || !wasm) {
              throw new Error('baseline page renderer is unavailable');
            }
            document.getElementById(captureContainerId)?.remove();
            const pageInfo = wasm.getPageInfo(capturePageIndex);
            const container = document.createElement('div');
            container.id = captureContainerId;
            container.style.position = 'fixed';
            container.style.left = '0';
            container.style.top = '0';
            container.style.width = `${Math.max(1, Math.floor(pageInfo.width))}px`;
            container.style.height = `${Math.max(1, Math.floor(pageInfo.height))}px`;
            container.style.background = '#fff';
            container.style.zIndex = '2147483647';
            container.style.overflow = 'hidden';
            container.style.pointerEvents = 'none';

            const canvas = document.createElement('canvas');
            canvas.id = captureCanvasId;
            canvas.style.position = 'absolute';
            canvas.style.left = '0';
            canvas.style.top = '0';
            canvas.style.background = '#fff';
            canvas.style.pointerEvents = 'none';
            container.appendChild(canvas);
            document.body.appendChild(container);

            const result = pageRenderer.renderPage(capturePageIndex, canvas, 1.0, 1.0, 1.0);
            const renderedCanvas = result?.renderedCanvas ?? canvas;
            renderedCanvas.id = captureCanvasId;
            renderedCanvas.style.position = 'absolute';
            renderedCanvas.style.left = '0';
            renderedCanvas.style.top = '0';
            renderedCanvas.style.width = `${renderedCanvas.width}px`;
            renderedCanvas.style.height = `${renderedCanvas.height}px`;
            container.style.width = `${renderedCanvas.width}px`;
            container.style.height = `${renderedCanvas.height}px`;
          },
          {
            captureCanvasId: BASELINE_CAPTURE_CANVAS_ID,
            captureContainerId: BASELINE_CAPTURE_CONTAINER_ID,
            capturePageIndex: sample.page,
          },
        );
        await page.waitForFunction(
          ({ captureBackend }) => {
            const renderer = window.__canvasView?.pageRenderer;
            if (!renderer) return false;
            if (captureBackend === 'canvas2d') {
              const imageCache = renderer.canvas2dRenderer?.domImageCache;
              return !(imageCache instanceof Map)
                || [...imageCache.values()].every((image) => image.complete);
            }
            const diagnostics = renderer.getCurrentCanvasKitRenderDiagnostics?.();
            return (diagnostics?.localTypefacePendingCount ?? 0) === 0;
          },
          { timeout: 10000, polling: 50 },
          { captureBackend: backend.key },
        );
        const selectedPageState = await page.evaluate(
          async ({ captureBackend, captureCanvasId, captureContainerId, capturePageIndex }) => {
            const container = document.getElementById(captureContainerId);
            const pageRenderer = window.__canvasView?.pageRenderer;
            const canvas = document.getElementById(captureCanvasId);
            if (!pageRenderer || !(container instanceof HTMLDivElement)
              || !(canvas instanceof HTMLCanvasElement)) {
              throw new Error('baseline capture surface is unavailable');
            }
            const result = pageRenderer.renderPage(capturePageIndex, canvas, 1.0, 1.0, 1.0);
            const renderedCanvas = result?.renderedCanvas ?? canvas;
            renderedCanvas.id = captureCanvasId;
            renderedCanvas.style.position = 'absolute';
            renderedCanvas.style.left = '0';
            renderedCanvas.style.top = '0';
            renderedCanvas.style.width = `${renderedCanvas.width}px`;
            renderedCanvas.style.height = `${renderedCanvas.height}px`;
            container.style.width = `${renderedCanvas.width}px`;
            container.style.height = `${renderedCanvas.height}px`;

            const domImages = [...container.querySelectorAll('img')];
            await Promise.all(domImages.map(async (image) => {
              if (!image.complete) {
                await new Promise((resolve, reject) => {
                  const timeout = window.setTimeout(
                    () => reject(new Error(`baseline DOM image timed out: ${image.currentSrc || image.src}`)),
                    10000,
                  );
                  const finish = () => {
                    window.clearTimeout(timeout);
                    resolve(undefined);
                  };
                  image.addEventListener('load', finish, { once: true });
                  image.addEventListener('error', finish, { once: true });
                });
              }
              if (image.naturalWidth > 0 && typeof image.decode === 'function') {
                await image.decode().catch(() => {});
              }
            }));

            const imageCache = captureBackend === 'canvas2d'
              ? pageRenderer.canvas2dRenderer?.domImageCache
              : null;
            const imageReadiness = imageCache instanceof Map
              ? [...imageCache.values()].reduce(
                (summary, image) => {
                  if (!image.complete) summary.pending += 1;
                  else if (image.naturalWidth > 0) summary.loaded += 1;
                  else summary.failed += 1;
                  return summary;
                },
                { total: imageCache.size, loaded: 0, failed: 0, pending: 0 },
              )
              : null;
            const domImageReadiness = domImages.reduce(
              (summary, image) => {
                if (!image.complete) summary.pending += 1;
                else if (image.naturalWidth > 0) summary.loaded += 1;
                else summary.failed += 1;
                return summary;
              },
              { total: domImages.length, loaded: 0, failed: 0, pending: 0 },
            );
            return {
              width: renderedCanvas.width,
              height: renderedCanvas.height,
              imageReadiness,
              domImageReadiness,
            };
          },
          {
            captureBackend: backend.key,
            captureCanvasId: BASELINE_CAPTURE_CANVAS_ID,
            captureContainerId: BASELINE_CAPTURE_CONTAINER_ID,
            capturePageIndex: sample.page,
          },
        );
        if ((selectedPageState.imageReadiness?.pending ?? 0) > 0) {
          throw new Error(
            `baseline capture still has pending images: ${sample.id} backend=${backend.key} pending=${selectedPageState.imageReadiness.pending}`,
          );
        }
        if ((selectedPageState.domImageReadiness?.pending ?? 0) > 0
          || (selectedPageState.domImageReadiness?.failed ?? 0) > 0) {
          throw new Error(
            `baseline capture has unsettled DOM images: ${sample.id} backend=${backend.key} `
              + `pending=${selectedPageState.domImageReadiness.pending} `
              + `failed=${selectedPageState.domImageReadiness.failed}`,
          );
        }
        await page.evaluate(() => new Promise((resolve) => {
          requestAnimationFrame(() => requestAnimationFrame(resolve));
        }));
        const selectedPageRenderMs = performance.now() - selectedPageRenderStartedAt;
        const layerFeatureProbe = backend.key.startsWith('canvaskit')
          ? await readLayerFeatureProbe(page, sample.page, profile)
          : null;
        const warmReplay = backend.key.startsWith('canvaskit')
          ? await measureWarmCanvasKitReplay(page, sample.page)
          : null;

        const sampleDir = path.join(options.output, sample.id);
        const outputPath = path.join(sampleDir, backend.filenameForProfile(profile));
        const screenshotStartedAt = performance.now();
        try {
          await captureCanvasScreenshot(
            page,
            outputPath,
            `Baseline ${backend.key} (${profile})`,
            BASELINE_CAPTURE_CONTAINER_SELECTOR,
          );
        } finally {
          await page.evaluate(
            ({ captureContainerId, capturePageIndex }) => {
              window.__canvasView?.pageRenderer?.cancelReRender?.(capturePageIndex);
              document.getElementById(captureContainerId)?.remove();
            },
            {
              captureContainerId: BASELINE_CAPTURE_CONTAINER_ID,
              capturePageIndex: sample.page,
            },
          );
        }
        const screenshotMs = performance.now() - screenshotStartedAt;
        const diagnostics = await readRendererDiagnostics(page, sample.page, backend.key, profile);
        diagnostics.capture = selectedPageState;
        const artifactBytes = fs.readFileSync(outputPath);
        const runtimeMode = diagnostics.canvaskitRender?.mode
          ?? diagnostics.runtime?.request?.canvaskitMode?.mode
          ?? null;
        const actualBackend = diagnostics.runtime?.activeBackend === 'canvaskit'
          ? `canvaskit-${runtimeMode}`
          : diagnostics.runtime?.activeBackend ?? null;
        const runtimeProfile = diagnostics.runtime?.request?.renderProfile ?? null;
        const actualProfile = runtimeProfile === 'highQuality'
          ? 'high-quality'
          : runtimeProfile === 'fastPreview'
            ? 'fast-preview'
            : runtimeProfile;
        const comparisonIdentity = {
          schemaVersion: 1,
          sampleId: sample.id,
          documentDigest: sample.documentDigest,
          page: sample.page,
          profile: actualProfile,
          backend: actualBackend,
          surface: backend.key.startsWith('canvaskit')
            ? diagnostics.canvaskitRender?.surfaceBackend ?? null
            : actualBackend === 'canvas2d' ? 'canvas2d' : null,
        };
        if (actualBackend !== backend.key) {
          hardGateViolations.push({
            sampleId: sample.id,
            backend: backend.key,
            profile,
            code: 'runtimeBackendMismatch',
            detail: JSON.stringify({ actualBackend, runtime: diagnostics.runtime }),
          });
        }
        if (actualProfile !== profile) {
          hardGateViolations.push({
            sampleId: sample.id,
            backend: backend.key,
            profile,
            code: 'runtimeProfileMismatch',
            detail: JSON.stringify({ actualProfile, runtime: diagnostics.runtime }),
          });
        }
        if (backend.key.startsWith('canvaskit')) {
          const replayPlan = diagnostics.replayPlan;
          const replaySummary = replayPlan?.summary;
          const runtime = diagnostics.canvaskitRender;
          if (diagnostics.replayPlanError) {
            hardGateViolations.push({
              sampleId: sample.id,
              backend: backend.key,
              profile,
              code: 'replayPlanUnavailable',
              detail: diagnostics.replayPlanError,
            });
          } else if (!replayPlan || !replaySummary || replaySummary.totalItems <= 0) {
            hardGateViolations.push({
              sampleId: sample.id,
              backend: backend.key,
              profile,
              code: 'replayPlanEmpty',
              detail: JSON.stringify(replayPlan),
            });
          }
          if (replayPlan
            && (replayPlan.hiddenCanvas2dOverlayAllowed !== false
              || replayPlan.directReplayRequired !== true)) {
            hardGateViolations.push({
              sampleId: sample.id,
              backend: backend.key,
              profile,
              code: 'replayPlanContractMismatch',
              detail: JSON.stringify({
                hiddenCanvas2dOverlayAllowed: replayPlan.hiddenCanvas2dOverlayAllowed,
                directReplayRequired: replayPlan.directReplayRequired,
              }),
            });
          }
          if (!runtime) {
            hardGateViolations.push({
              sampleId: sample.id,
              backend: backend.key,
              profile,
              code: 'runtimeDiagnosticsUnavailable',
              detail: null,
            });
          } else {
            if (runtime.lastRenderCompleted !== true) {
              hardGateViolations.push({
                sampleId: sample.id,
                backend: backend.key,
                profile,
                code: 'runtimeRenderIncomplete',
                detail: runtime.lastRenderError ?? null,
              });
            }
            if (runtime.lastRenderError) {
              hardGateViolations.push({
                sampleId: sample.id,
                backend: backend.key,
                profile,
                code: 'runtimeRenderError',
                detail: runtime.lastRenderError,
              });
            }
            if ((runtime.lastUnexpectedUnsupportedOps ?? []).length > 0) {
              hardGateViolations.push({
                sampleId: sample.id,
                backend: backend.key,
                profile,
                code: 'runtimeUnexpectedUnsupportedOps',
                detail: JSON.stringify(runtime.lastUnexpectedUnsupportedOps),
              });
            }
          }
        }
        results.push({
          sampleId: sample.id,
          file: sample.file,
          category: sample.category,
          diagnosticAxes: sample.diagnosticAxes,
          documentDigest: sample.documentDigest,
          page: sample.page,
          backend: backend.key,
          actualBackend,
          profile,
          actualProfile,
          canvaskitSurface: backend.key.startsWith('canvaskit') ? options.canvaskitSurface : null,
          readinessGateRequired: options.readinessOnly
            && sample.canvaskitReadinessGate === true
            && backend.key === 'canvaskit-default'
            && profile === 'screen'
            && options.canvaskitSurface === 'auto',
          path: outputPath,
          comparisonIdentity,
          artifact: {
            sha256: createHash('sha256').update(artifactBytes).digest('hex'),
            sizeBytes: artifactBytes.byteLength,
            width: selectedPageState.width,
            height: selectedPageState.height,
          },
          timings: {
            appLoadMs,
            documentLoadAndInitialRenderMs,
            selectedPageRenderMs,
            warmReplayMs: warmReplay?.replayMs ?? null,
            warmRendererDurationMs: warmReplay?.rendererDurationMs ?? null,
            screenshotMs,
            totalMs: performance.now() - totalStartedAt,
          },
          diagnostics,
          layerFeatureProbe,
          warmReplay,
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
          diagnosticAxes: sample.diagnosticAxes,
          documentDigest: sample.documentDigest,
          page: sample.page,
          profile,
          baselineBackend: 'canvas2d',
          targetBackend,
          status: 'missing',
          baselinePath: baseline?.path ?? null,
          targetPath: target?.path ?? null,
        });
        continue;
      }

      const identityMismatches = [];
      for (const field of ['schemaVersion', 'sampleId', 'documentDigest', 'page', 'profile']) {
        if (baseline.comparisonIdentity?.[field] !== target.comparisonIdentity?.[field]) {
          identityMismatches.push(field);
        }
      }
      if (baseline.comparisonIdentity?.backend !== 'canvas2d') identityMismatches.push('baselineBackend');
      if (target.comparisonIdentity?.backend !== targetBackend) identityMismatches.push('targetBackend');
      if (identityMismatches.length > 0) {
        browserBackendComparisons.push({
          sampleId: sample.id,
          category: sample.category,
          diagnosticAxes: sample.diagnosticAxes,
          documentDigest: sample.documentDigest,
          page: sample.page,
          profile,
          baselineBackend: 'canvas2d',
          targetBackend,
          status: 'identityMismatch',
          identityMismatches: [...new Set(identityMismatches)].sort(),
          baselineIdentity: baseline.comparisonIdentity ?? null,
          targetIdentity: target.comparisonIdentity ?? null,
          baselinePath: baseline.path,
          targetPath: target.path,
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
          diagnosticAxes: sample.diagnosticAxes,
          documentDigest: sample.documentDigest,
          page: sample.page,
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
          diagnosticAxes: sample.diagnosticAxes,
          documentDigest: sample.documentDigest,
          page: sample.page,
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
const browserBackendSummaryByDiagnosticAxis = new Map();
for (const item of browserBackendComparisons) {
  for (const [summaryMap, keyField, keyValue] of [
    [browserBackendSummaryByTarget, 'targetBackend', item.targetBackend],
    [browserBackendSummaryByProfile, 'profile', item.profile],
    [browserBackendSummaryByCategory, 'category', item.category],
    ...(item.diagnosticAxes ?? []).map((axis) => (
      [browserBackendSummaryByDiagnosticAxis, 'diagnosticAxis', axis]
    )),
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
        identityMismatches: 0,
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
    if (item.status === 'identityMismatch') {
      summary.identityMismatches += 1;
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
    identityMismatches: browserBackendComparisons
      .filter((item) => item.status === 'identityMismatch').length,
  },
  summaryByTargetBackend: [...browserBackendSummaryByTarget.values()]
    .sort((left, right) => left.targetBackend.localeCompare(right.targetBackend)),
  summaryByProfile: [...browserBackendSummaryByProfile.values()]
    .sort((left, right) => left.profile.localeCompare(right.profile)),
  summaryByCategory: [...browserBackendSummaryByCategory.values()]
    .sort((left, right) => String(left.category).localeCompare(String(right.category))),
  summaryByDiagnosticAxis: [...browserBackendSummaryByDiagnosticAxis.values()]
    .sort((left, right) => left.diagnosticAxis.localeCompare(right.diagnosticAxis)),
  worstComparisons: browserBackendCompared
    .map((item) => ({
      sampleId: item.sampleId,
      category: item.category,
      diagnosticAxes: item.diagnosticAxes,
      page: item.page,
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

const replaySummaryByBackendProfile = new Map();
for (const result of results) {
  if (!result.backend.startsWith('canvaskit')) continue;
  const key = `${result.backend}\u0000${result.profile}`;
  if (!replaySummaryByBackendProfile.has(key)) {
    replaySummaryByBackendProfile.set(key, {
      backend: result.backend,
      profile: result.profile,
      captureCount: 0,
      totalItems: 0,
      directItems: 0,
      directRequiredItems: 0,
      compatOverlayItems: 0,
      textFallbackItems: 0,
      unsupportedItems: 0,
      hiddenOverlayViolations: 0,
      hardGateViolationCount: 0,
      runtimeRenderErrors: 0,
      runtimeUnexpectedUnsupportedOps: 0,
      planStatusCounts: {},
      planReasonCounts: {},
      planFeatureCounts: {},
      expectedUnsupportedOpCounts: {},
      unexpectedUnsupportedOpCounts: {},
    });
  }
  const summary = replaySummaryByBackendProfile.get(key);
  const replayPlan = result.diagnostics?.replayPlan ?? {};
  const planSummary = replayPlan.summary ?? {};
  const runtime = result.diagnostics?.canvaskitRender ?? {};
  summary.captureCount += 1;
  for (const field of [
    'totalItems',
    'directItems',
    'directRequiredItems',
    'compatOverlayItems',
    'textFallbackItems',
    'unsupportedItems',
    'hiddenOverlayViolations',
  ]) {
    const value = planSummary[field];
    if (typeof value === 'number' && Number.isFinite(value)) summary[field] += value;
  }
  if (runtime.lastRenderError) summary.runtimeRenderErrors += 1;
  summary.runtimeUnexpectedUnsupportedOps += runtime.lastUnexpectedUnsupportedOps?.length ?? 0;
  for (const item of replayPlan.items ?? []) {
    const status = String(item.status ?? 'unknown');
    const reason = String(item.reason ?? 'unknown');
    const feature = String(item.feature ?? 'unknown');
    summary.planStatusCounts[status] = (summary.planStatusCounts[status] ?? 0) + 1;
    summary.planReasonCounts[reason] = (summary.planReasonCounts[reason] ?? 0) + 1;
    summary.planFeatureCounts[feature] = (summary.planFeatureCounts[feature] ?? 0) + 1;
  }
  for (const op of runtime.lastExpectedUnsupportedOps ?? []) {
    summary.expectedUnsupportedOpCounts[op] = (summary.expectedUnsupportedOpCounts[op] ?? 0) + 1;
  }
  for (const op of runtime.lastUnexpectedUnsupportedOps ?? []) {
    summary.unexpectedUnsupportedOpCounts[op] = (summary.unexpectedUnsupportedOpCounts[op] ?? 0) + 1;
  }
}
for (const violation of hardGateViolations) {
  const summary = replaySummaryByBackendProfile.get(`${violation.backend}\u0000${violation.profile}`);
  if (summary) summary.hardGateViolationCount += 1;
}
const replaySummaryRows = [...replaySummaryByBackendProfile.values()]
  .sort((left, right) => (
    left.profile.localeCompare(right.profile) || left.backend.localeCompare(right.backend)
  ));
for (const summary of replaySummaryRows) {
  for (const field of [
    'planStatusCounts',
    'planReasonCounts',
    'planFeatureCounts',
    'expectedUnsupportedOpCounts',
    'unexpectedUnsupportedOpCounts',
  ]) {
    summary[field] = Object.fromEntries(
      Object.entries(summary[field]).sort(([left], [right]) => left.localeCompare(right)),
    );
  }
}
const canvaskitReplayDiagnostics = {
  mode: 'contractGateAndReportInventory',
  hardGateViolationCount: hardGateViolations.length,
  hardGateViolations,
  summaryByBackendProfile: replaySummaryRows,
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
  const sample = samples.find((entry) => entry.id === result.sampleId);
  const performanceBudget = sample?.canvaskitPerformanceBudget ?? null;
  const readinessExpectations = sample?.canvaskitReadinessExpectations ?? null;
  if (runtime.activeBackend !== 'canvaskit') {
    blockers.push('backendNotActive');
  }
  if (runtime.request?.backend?.backend !== 'canvas2d'
    || runtime.request?.backend?.source !== 'url') {
    blockers.push('legacyRequestProjectionMismatch');
  }
  if (runtime.selection?.requestedBackend !== 'auto'
    || runtime.selection?.request?.backend !== 'auto'
    || runtime.selection?.request?.source !== 'url'
    || runtime.selection?.effectiveBackend !== 'canvaskit'
    || runtime.selection?.selectionReason !== 'autoEligible') {
    blockers.push('autoSelectionMismatch');
  }
  if (runtime.selection?.preflight?.status !== 'eligible'
    || runtime.selection?.preflight?.complete !== true
    || runtime.selection?.preflight?.eligible !== true) {
    blockers.push('autoPreflightNotEligible');
  }
  if (typeof runtime.selection?.documentDigest !== 'string'
    || !runtime.selection.documentDigest.startsWith('blake3:')) {
    blockers.push('autoDocumentDigestMissing');
  }
  if (!Number.isSafeInteger(runtime.selection?.documentRevision)
    || !Number.isSafeInteger(runtime.selection?.resourceGeneration)) {
    blockers.push('autoDecisionGenerationMissing');
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
  } else if (comparison.status === 'identityMismatch') {
    blockers.push(`comparisonIdentityMismatch:${comparison.identityMismatches.join(',')}`);
  } else if (comparison.diff?.hasVisualBudget !== true) {
    blockers.push('visualThresholdMissing');
  } else if (comparison.diff?.passed !== true) {
    blockers.push('visualParityFailed');
  }
  if (performanceBudget === null) {
    blockers.push('performanceBudgetMissing');
  } else {
    if (typeof result.timings.documentLoadAndInitialRenderMs !== 'number'
      || !Number.isFinite(result.timings.documentLoadAndInitialRenderMs)) {
      blockers.push('performanceColdMissing');
    } else if (result.timings.documentLoadAndInitialRenderMs
      > performanceBudget.maxColdDocumentLoadAndInitialRenderMs) {
      blockers.push('performanceColdExceeded');
    }
    if (result.warmReplay?.rerendered !== true
      || typeof result.warmReplay?.replayMs !== 'number'
      || !Number.isFinite(result.warmReplay.replayMs)
      || result.warmReplay?.renderCountDelta !== 1) {
      blockers.push('warmReplayMissing');
    } else if (result.warmReplay.replayMs > performanceBudget.maxWarmReplayMs) {
      blockers.push('performanceWarmExceeded');
    }
    if (typeof result.warmReplay?.rendererDurationMs !== 'number') {
      blockers.push('warmRendererDurationMissing');
    } else if (result.warmReplay.rendererDurationMs
      > performanceBudget.maxWarmRendererDurationMs) {
      blockers.push('performanceRendererWarmExceeded');
    }
    if (typeof result.warmReplay?.imageCachePixels !== 'number') {
      blockers.push('imageCachePixelsMissing');
    } else if (result.warmReplay.imageCachePixels > performanceBudget.maxImageCachePixels) {
      blockers.push('imageCachePixelBudgetExceeded');
    }
  }
  if (readinessExpectations !== null) {
    for (const payloadKind of readinessExpectations.glyphOutlinePayloadKinds) {
      if ((result.layerFeatureProbe?.glyphOutlinePayloadCounts?.[payloadKind] ?? 0) < 1) {
        blockers.push(`glyphOutlinePayloadMissing:${payloadKind}`);
      }
    }
    if ((result.warmReplay?.imageCacheHitDelta ?? 0)
      < readinessExpectations.minWarmImageCacheHits) {
      blockers.push('warmImageCacheHitMissing');
    }
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
    selection: runtime.selection ?? null,
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
    performanceBudget,
    readinessExpectations,
    layerFeatureProbe: result.layerFeatureProbe ?? null,
    coldDocumentLoadAndInitialRenderMs: result.timings.documentLoadAndInitialRenderMs,
    warmReplay: result.warmReplay ?? null,
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
    targetBackend: 'canvaskit-default (explicit auto request)',
    canvaskitSurface: 'auto',
    requireActiveBackend: true,
    requireRuntimeReadiness: true,
    requireVisualParity: true,
    requireColdAndWarmPerformanceBudget: true,
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
      scope: options.scope,
      sampleCount: samples.length,
      profiles,
      canvaskitSurface: options.canvaskitSurface,
      results,
      browserBackendParity,
      canvaskitReplayDiagnostics,
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
if (hardGateViolations.length > 0) {
  for (const violation of hardGateViolations) {
    console.error(
      `[baseline] CanvasKit replay contract failed: ${violation.sampleId} `
        + `${violation.backend}@${violation.profile} ${violation.code}`,
    );
  }
  process.exitCode = 1;
}
if (captureError && !options.readinessOnly) {
  process.exitCode = 1;
}
