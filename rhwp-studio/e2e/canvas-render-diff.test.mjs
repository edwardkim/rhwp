/**
 * Browser canvas visual diff between the legacy PageRenderTree path and the
 * default PageLayerTree replay path.
 *
 * Run from rhwp-studio after building ../pkg with wasm-pack:
 *   npm run e2e:render-diff
 *
 * To start/stop the Vite server automatically:
 *   npm run e2e:render-diff:ci
 *
 * Useful knobs:
 *   RHWP_RENDER_DIFF_FILES=basic/KTX.hwp,biz_plan.hwp
 *   RHWP_RENDER_DIFF_MAX_PAGES=1|all
 *   RHWP_RENDER_DIFF_ALL=1
 *   RHWP_RENDER_DIFF_WRITE_IMAGES=1
 */
import { createHash } from 'crypto';
import { existsSync, mkdirSync, writeFileSync } from 'fs';
import { dirname, extname, isAbsolute, join, posix, relative, resolve, sep } from 'path';
import { fileURLToPath } from 'url';
import { runTest, loadHwpFile, assert, setTestCase } from './helpers.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ARTIFACT_DIR = join(__dirname, 'screenshots', 'render-diff');
const REPORT_PATH = join(ARTIFACT_DIR, 'results.json');
const SUMMARY_PATH = join(ARTIFACT_DIR, 'summary.md');
const SAMPLES_DIR = resolve(__dirname, '..', 'public', 'samples');
const VALID_FIXTURE_EXTENSIONS = new Set(['.hwp', '.hwpx']);

const DEFAULT_FIXTURES = [
  'basic/KTX.hwp',
  'biz_plan.hwp',
  'tac-case-001.hwp',
];

const ALL_FIXTURES = [
  'BlogForm_BookReview.hwp',
  'basic/KTX.hwp',
  'biz_plan.hwp',
  'footnote-01.hwp',
  'form-002.hwpx',
  'kps-ai.hwp',
  'number-bullet.hwp',
  'oullim-01.hwp',
  'para-head-num-2.hwp',
  'shift-return.hwp',
  'tac-case-001.hwp',
];

function numberFromEnv(name, fallback) {
  const raw = process.env[name];
  if (!raw) return fallback;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function maxPagesFromEnv() {
  const raw = process.env.RHWP_RENDER_DIFF_MAX_PAGES;
  if (!raw) return 1;
  if (raw === 'all') return Number.POSITIVE_INFINITY;
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : 1;
}

function fixturesFromEnv() {
  const raw = process.env.RHWP_RENDER_DIFF_FILES;
  const fixtures = raw
    ? raw.split(',').map(s => s.trim()).filter(Boolean)
    : process.env.RHWP_RENDER_DIFF_ALL === '1' ? ALL_FIXTURES : DEFAULT_FIXTURES;
  return fixtures.map(normalizeFixture);
}

function normalizeFixture(value) {
  const fixture = String(value).trim();
  if (!fixture) {
    throw new Error('render diff fixture must not be empty');
  }
  if (fixture.includes('\0') || fixture.includes('\\') || fixture.includes('?') || fixture.includes('#')) {
    throw new Error(`invalid render diff fixture path: ${fixture}`);
  }
  if (fixture.startsWith('/') || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(fixture)) {
    throw new Error(`render diff fixture must be relative to public/samples: ${fixture}`);
  }
  let decoded = fixture;
  try {
    decoded = decodeURIComponent(fixture);
  } catch {
    throw new Error(`render diff fixture must not contain malformed URL escapes: ${fixture}`);
  }
  if (decoded !== fixture) {
    throw new Error(`render diff fixture must not be percent-encoded: ${fixture}`);
  }
  const normalized = posix.normalize(fixture);
  if (normalized !== fixture || normalized === '.' || normalized === '..' || normalized.startsWith('../')) {
    throw new Error(`render diff fixture must stay under public/samples: ${fixture}`);
  }
  if (!VALID_FIXTURE_EXTENSIONS.has(extname(normalized).toLowerCase())) {
    throw new Error(`render diff fixture must be a .hwp or .hwpx sample: ${fixture}`);
  }
  const resolved = resolve(SAMPLES_DIR, ...normalized.split('/'));
  const relativePath = relative(SAMPLES_DIR, resolved);
  if (relativePath === '' || relativePath === '..' || relativePath.startsWith(`..${sep}`) || isAbsolute(relativePath)) {
    throw new Error(`render diff fixture resolved outside public/samples: ${fixture}`);
  }
  if (!existsSync(resolved)) {
    throw new Error(`render diff fixture does not exist under public/samples: ${fixture}`);
  }
  return normalized;
}

function safeName(value) {
  const sanitized = value.replace(/[^a-z0-9_.-]+/gi, '_').replace(/^_+|_+$/g, '');
  const hash = createHash('sha256').update(value).digest('hex').slice(0, 8);
  return `${sanitized || 'fixture'}-${hash}`;
}

function writeDataUrl(path, dataUrl) {
  const match = /^data:image\/png;base64,([A-Za-z0-9+/=]+)$/.exec(dataUrl);
  if (!match) {
    throw new Error(`expected PNG data URL for ${path}`);
  }
  writeFileSync(path, Buffer.from(match[1], 'base64'));
}

function markdownCell(value) {
  return String(value)
    .replace(/\\/g, '\\\\')
    .replace(/\|/g, '\\|')
    .replace(/\[/g, '\\[')
    .replace(/\]/g, '\\]')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\n/g, '<br>');
}

function formatPageLimit(maxPages) {
  return maxPages === Number.POSITIVE_INFINITY ? 'all' : String(maxPages);
}

function resultLine(result) {
  const percent = (result.diffRatio * 100).toFixed(5);
  const size = result.sameSize
    ? `${result.width}x${result.height}`
    : `${result.legacyWidth}x${result.legacyHeight} vs ${result.layerWidth}x${result.layerHeight}`;
  return `${result.fixture} page ${result.pageIndex + 1}: ${result.diffPixels}/${result.totalPixels} pixels differ (${percent}%), max channel delta ${result.maxChannelDelta}, size ${size}`;
}

function renderMarkdownSummary(config, results) {
  const failures = results.filter(result => !result.pass);
  const lines = [
    '# Canvas Visual Diff',
    '',
    `- fixtures: ${config.fixtures.map(markdownCell).join(', ')}`,
    `- scale: ${config.scale}`,
    `- max pages: ${formatPageLimit(config.maxPages)}`,
    `- channel tolerance: ${config.channelTolerance}`,
    `- max diff ratio: ${config.maxDiffRatio}`,
    `- compared pages: ${results.length}`,
    `- failed pages: ${failures.length}`,
    '',
  ];

  if (results.length === 0) {
    lines.push('No pages were compared. Check the e2e error screenshot and Vite log artifacts.');
    lines.push('');
    return lines.join('\n');
  }

  lines.push('| Status | Fixture | Page | Size | Diff pixels | Diff ratio | Max channel delta | Artifacts |');
  lines.push('| --- | --- | ---: | --- | ---: | ---: | ---: | --- |');
  for (const result of results) {
    const status = result.pass ? 'pass' : 'fail';
    const size = result.sameSize
      ? `${result.width}x${result.height}`
      : `${result.legacyWidth}x${result.legacyHeight} vs ${result.layerWidth}x${result.layerHeight}`;
    const artifactText = result.artifactFiles
      ? Object.values(result.artifactFiles).join('<br>')
      : '';
    lines.push([
      status,
      markdownCell(result.fixture),
      result.pageIndex + 1,
      size,
      result.diffPixels,
      result.diffRatio.toFixed(8),
      result.maxChannelDelta,
      markdownCell(artifactText),
    ].join(' | ').replace(/^/, '| ').replace(/$/, ' |'));
  }

  lines.push('');
  if (failures.length > 0) {
    lines.push('## Failures');
    lines.push('');
    for (const failure of failures) {
      lines.push(`- ${resultLine(failure)}`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

function writeReports(config, results) {
  writeFileSync(REPORT_PATH, JSON.stringify({
    config: {
      ...config,
      maxPages: formatPageLimit(config.maxPages),
    },
    results,
  }, null, 2));
  writeFileSync(SUMMARY_PATH, renderMarkdownSummary(config, results));
}

const config = {
  fixtures: fixturesFromEnv(),
  scale: numberFromEnv('RHWP_RENDER_DIFF_SCALE', 1),
  maxPages: maxPagesFromEnv(),
  channelTolerance: numberFromEnv('RHWP_RENDER_DIFF_CHANNEL_TOLERANCE', 1),
  maxDiffRatio: numberFromEnv('RHWP_RENDER_DIFF_MAX_RATIO', 0.0005),
  writeImages: process.env.RHWP_RENDER_DIFF_WRITE_IMAGES === '1',
};

mkdirSync(ARTIFACT_DIR, { recursive: true });

runTest('Canvas legacy/layer visual diff', async ({ page }) => {
  const results = [];

  try {
    for (const fixture of config.fixtures) {
      setTestCase(`render-diff ${fixture}`);
      const { pageCount } = await loadHwpFile(page, fixture);
      const pageLimit = Math.min(pageCount, config.maxPages);

      for (let pageIndex = 0; pageIndex < pageLimit; pageIndex++) {
        const result = await page.evaluate((args) => {
          const doc = window.__wasm?.doc;
          if (!doc) throw new Error('window.__wasm.doc is not available');
          if (typeof doc.renderPageToCanvasLegacy !== 'function') {
            throw new Error('renderPageToCanvasLegacy is not available; rebuild the WASM package');
          }
          if (typeof doc.renderPageToCanvas !== 'function') {
            throw new Error('renderPageToCanvas is not available');
          }

          const legacyCanvas = document.createElement('canvas');
          const layerCanvas = document.createElement('canvas');
          doc.renderPageToCanvasLegacy(args.pageIndex, legacyCanvas, args.scale);
          doc.renderPageToCanvas(args.pageIndex, layerCanvas, args.scale);

          const width = Math.max(legacyCanvas.width, layerCanvas.width);
          const height = Math.max(legacyCanvas.height, layerCanvas.height);
          const sameSize = legacyCanvas.width === layerCanvas.width
            && legacyCanvas.height === layerCanvas.height;

          const normalize = (canvas) => {
            if (canvas.width === width && canvas.height === height) return canvas;
            const normalized = document.createElement('canvas');
            normalized.width = width;
            normalized.height = height;
            normalized.getContext('2d').drawImage(canvas, 0, 0);
            return normalized;
          };

          const legacy = normalize(legacyCanvas);
          const layer = normalize(layerCanvas);
          const legacyData = legacy.getContext('2d', { willReadFrequently: true })
            .getImageData(0, 0, width, height);
          const layerData = layer.getContext('2d', { willReadFrequently: true })
            .getImageData(0, 0, width, height);
          const diffCanvas = document.createElement('canvas');
          diffCanvas.width = width;
          diffCanvas.height = height;
          const diffCtx = diffCanvas.getContext('2d');
          const diffData = diffCtx.createImageData(width, height);

          let diffPixels = 0;
          let maxChannelDelta = 0;
          let totalChannelDelta = 0;

          for (let i = 0; i < legacyData.data.length; i += 4) {
            const dr = Math.abs(legacyData.data[i] - layerData.data[i]);
            const dg = Math.abs(legacyData.data[i + 1] - layerData.data[i + 1]);
            const db = Math.abs(legacyData.data[i + 2] - layerData.data[i + 2]);
            const da = Math.abs(legacyData.data[i + 3] - layerData.data[i + 3]);
            const pixelDelta = Math.max(dr, dg, db, da);
            maxChannelDelta = Math.max(maxChannelDelta, pixelDelta);
            totalChannelDelta += dr + dg + db + da;

            if (pixelDelta > args.channelTolerance) {
              diffPixels += 1;
              diffData.data[i] = 255;
              diffData.data[i + 1] = 0;
              diffData.data[i + 2] = 0;
              diffData.data[i + 3] = 255;
            } else {
              diffData.data[i] = 255;
              diffData.data[i + 1] = 255;
              diffData.data[i + 2] = 255;
              diffData.data[i + 3] = 0;
            }
          }

          diffCtx.putImageData(diffData, 0, 0);

          const totalPixels = width * height;
          const diffRatio = totalPixels === 0 ? 1 : diffPixels / totalPixels;
          const pass = sameSize && diffRatio <= args.maxDiffRatio;
          const includeImages = args.writeImages || !pass;

          return {
            pageIndex: args.pageIndex,
            legacyWidth: legacyCanvas.width,
            legacyHeight: legacyCanvas.height,
            layerWidth: layerCanvas.width,
            layerHeight: layerCanvas.height,
            width,
            height,
            sameSize,
            diffPixels,
            totalPixels,
            diffRatio,
            maxChannelDelta,
            averageChannelDelta: totalPixels === 0 ? 0 : totalChannelDelta / (totalPixels * 4),
            pass,
            images: includeImages ? {
              legacy: legacyCanvas.toDataURL('image/png'),
              layer: layerCanvas.toDataURL('image/png'),
              diff: diffCanvas.toDataURL('image/png'),
            } : null,
          };
        }, {
          pageIndex,
          scale: config.scale,
          channelTolerance: config.channelTolerance,
          maxDiffRatio: config.maxDiffRatio,
          writeImages: config.writeImages,
        });

        const baseName = `${safeName(fixture)}-p${String(pageIndex + 1).padStart(2, '0')}`;
        if (result.images) {
          const artifactFiles = {
            legacy: `e2e/screenshots/render-diff/${baseName}-legacy.png`,
            layer: `e2e/screenshots/render-diff/${baseName}-layer.png`,
            diff: `e2e/screenshots/render-diff/${baseName}-diff.png`,
          };
          writeDataUrl(join(ARTIFACT_DIR, `${baseName}-legacy.png`), result.images.legacy);
          writeDataUrl(join(ARTIFACT_DIR, `${baseName}-layer.png`), result.images.layer);
          writeDataUrl(join(ARTIFACT_DIR, `${baseName}-diff.png`), result.images.diff);
          result.artifactFiles = artifactFiles;
        }

        delete result.images;
        result.fixture = fixture;
        results.push(result);

        assert(result.pass, resultLine(result));
      }
    }
  } finally {
    writeReports(config, results);
  }

  const failures = results.filter(result => !result.pass);
  if (failures.length > 0) {
    throw new Error([
      `${failures.length} canvas visual diff case(s) exceeded tolerance:`,
      ...failures.map(result => `- ${resultLine(result)}`),
      `See ${SUMMARY_PATH} and ${ARTIFACT_DIR} for details.`,
    ].join('\n'));
  }
});
