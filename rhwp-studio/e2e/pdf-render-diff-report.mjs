/**
 * Report-only visual diff between browser Canvas output and SVG-derived PDF
 * export output rasterized back to PNG.
 *
 * This is intentionally non-gating. It writes JSON/Markdown summaries and PNG
 * artifacts so PDF export drift can be reviewed before it becomes a hard gate.
 */
import { createHash } from 'crypto';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  realpathSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'fs';
import { spawnSync } from 'node:child_process';
import {
  dirname,
  extname,
  isAbsolute,
  join,
  posix,
  relative,
  resolve,
  sep,
} from 'path';
import { fileURLToPath } from 'url';
import { PNG } from 'pngjs';
import { runTest, loadHwpFile, setTestCase } from './helpers.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const studioRoot = resolve(__dirname, '..');
const repoRoot = resolve(studioRoot, '..');
const ARTIFACT_DIR = join(__dirname, 'screenshots', 'render-diff');
const REPORT_PATH = join(ARTIFACT_DIR, 'pdf-results.json');
const SUMMARY_PATH = join(ARTIFACT_DIR, 'pdf-summary.md');
const SAMPLES_DIR = resolve(studioRoot, 'public', 'samples');
const REAL_SAMPLES_DIR = realpathSync(SAMPLES_DIR);
const VALID_FIXTURE_EXTENSIONS = new Set(['.hwp', '.hwpx']);
const MAX_CAPTURE_SIZE_DRIFT_PX = 1;

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

function optionalNumberFromEnv(name) {
  const raw = process.env[name];
  if (!raw) return null;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : null;
}

const COMMAND_TIMEOUT_MS = numberFromEnv('RHWP_RENDER_DIFF_COMMAND_TIMEOUT_MS', 120000);

function maxPagesFromEnv() {
  const raw = process.env.RHWP_RENDER_DIFF_MAX_PAGES;
  if (!raw) return 1;
  if (raw === 'all') return Number.POSITIVE_INFINITY;
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : 1;
}

function normalizeFixture(value) {
  const fixture = String(value).trim();
  if (!fixture) {
    throw new Error('PDF render diff fixture must not be empty');
  }
  if (fixture.includes('\0') || fixture.includes('\\') || fixture.includes('?') || fixture.includes('#')) {
    throw new Error(`invalid PDF render diff fixture path: ${fixture}`);
  }
  if (fixture.startsWith('/') || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(fixture)) {
    throw new Error(`PDF render diff fixture must be relative to public/samples: ${fixture}`);
  }
  let decoded = fixture;
  try {
    decoded = decodeURIComponent(fixture);
  } catch {
    throw new Error(`PDF render diff fixture must not contain malformed URL escapes: ${fixture}`);
  }
  if (decoded !== fixture) {
    throw new Error(`PDF render diff fixture must not be percent-encoded: ${fixture}`);
  }
  const normalized = posix.normalize(fixture);
  if (normalized !== fixture || normalized === '.' || normalized === '..' || normalized.startsWith('../')) {
    throw new Error(`PDF render diff fixture must stay under public/samples: ${fixture}`);
  }
  if (!VALID_FIXTURE_EXTENSIONS.has(extname(normalized).toLowerCase())) {
    throw new Error(`PDF render diff fixture must be a .hwp or .hwpx sample: ${fixture}`);
  }
  const resolved = resolve(SAMPLES_DIR, ...normalized.split('/'));
  const relativePath = relative(SAMPLES_DIR, resolved);
  if (relativePath === '' || relativePath === '..' || relativePath.startsWith(`..${sep}`) || isAbsolute(relativePath)) {
    throw new Error(`PDF render diff fixture resolved outside public/samples: ${fixture}`);
  }
  let current = REAL_SAMPLES_DIR;
  for (const part of normalized.split('/')) {
    current = resolve(current, part);
    let entry;
    try {
      entry = lstatSync(current);
    } catch {
      throw new Error(`PDF render diff fixture does not exist under public/samples: ${fixture}`);
    }
    if (entry.isSymbolicLink()) {
      throw new Error(`PDF render diff fixture must not use symlinks: ${fixture}`);
    }
  }
  const realResolved = realpathSync(resolved);
  const realRelativePath = relative(REAL_SAMPLES_DIR, realResolved);
  if (realRelativePath === '' || realRelativePath === '..' || realRelativePath.startsWith(`..${sep}`) || isAbsolute(realRelativePath)) {
    throw new Error(`PDF render diff fixture real path escaped public/samples: ${fixture}`);
  }
  if (!statSync(resolved).isFile()) {
    throw new Error(`PDF render diff fixture must be a regular file under public/samples: ${fixture}`);
  }
  return normalized;
}

function fixturesFromEnv() {
  const raw = process.env.RHWP_RENDER_DIFF_FILES;
  const fixtures = raw
    ? raw.split(',').map(s => s.trim()).filter(Boolean)
    : process.env.RHWP_RENDER_DIFF_ALL === '1' ? ALL_FIXTURES : DEFAULT_FIXTURES;
  return fixtures.map(normalizeFixture);
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

function runCommand(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd || repoRoot,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    timeout: COMMAND_TIMEOUT_MS,
  });

  if (result.error) {
    if (result.error.code === 'ETIMEDOUT') {
      throw new Error(`${command} ${args.join(' ')} timed out after ${COMMAND_TIMEOUT_MS}ms`);
    }
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error([
      `${command} ${args.join(' ')} exited with ${result.status}`,
      result.stdout?.trim(),
      result.stderr?.trim(),
    ].filter(Boolean).join('\n'));
  }

  return {
    stdout: result.stdout || '',
    stderr: result.stderr || '',
  };
}

function removeIfExists(path) {
  rmSync(path, { force: true });
}

function assertFreshPdf(path) {
  const stat = statSync(path);
  if (!stat.isFile() || stat.size === 0) {
    throw new Error(`PDF export did not create a non-empty PDF: ${path}`);
  }
  const header = readFileSync(path).subarray(0, 5).toString('ascii');
  if (header !== '%PDF-') {
    throw new Error(`PDF export did not create a PDF file: ${path}`);
  }
}

function rhwpCommandPrefix() {
  const envBin = process.env.RHWP_RENDER_DIFF_RHWP_BIN;
  const candidate = envBin
    ? isAbsolute(envBin) ? envBin : resolve(studioRoot, envBin)
    : resolve(repoRoot, 'target', 'debug', process.platform === 'win32' ? 'rhwp.exe' : 'rhwp');

  if (existsSync(candidate)) {
    return {
      command: candidate,
      args: [],
      mode: 'binary',
    };
  }

  return {
    command: 'cargo',
    args: ['run', '--quiet', '--bin', 'rhwp', '--'],
    mode: 'cargo-run',
  };
}

function flattenPngToWhiteCanvas(buffer, width, height) {
  const source = PNG.sync.read(buffer);
  const target = new PNG({ width, height });
  target.data.fill(255);

  for (let y = 0; y < Math.min(source.height, height); y += 1) {
    for (let x = 0; x < Math.min(source.width, width); x += 1) {
      const sourceOffset = (y * source.width + x) * 4;
      const targetOffset = (y * width + x) * 4;
      const alpha = source.data[sourceOffset + 3] / 255;
      target.data[targetOffset] = Math.round(source.data[sourceOffset] * alpha + 255 * (1 - alpha));
      target.data[targetOffset + 1] = Math.round(source.data[sourceOffset + 1] * alpha + 255 * (1 - alpha));
      target.data[targetOffset + 2] = Math.round(source.data[sourceOffset + 2] * alpha + 255 * (1 - alpha));
      target.data[targetOffset + 3] = 255;
    }
  }

  return target;
}

function comparePngPaths(referencePath, pdfPath, diffPath, channelTolerance) {
  const referenceSource = PNG.sync.read(readFileSync(referencePath));
  const pdfSource = PNG.sync.read(readFileSync(pdfPath));
  const widthDelta = Math.abs(referenceSource.width - pdfSource.width);
  const heightDelta = Math.abs(referenceSource.height - pdfSource.height);
  const sameSize = widthDelta === 0 && heightDelta === 0;
  const sizeNormalization = !sameSize && widthDelta <= MAX_CAPTURE_SIZE_DRIFT_PX && heightDelta <= MAX_CAPTURE_SIZE_DRIFT_PX
    ? {
      strategy: 'cropToCommonTopLeft',
      maxCaptureSizeDriftPx: MAX_CAPTURE_SIZE_DRIFT_PX,
      referenceSize: { width: referenceSource.width, height: referenceSource.height },
      pdfSize: { width: pdfSource.width, height: pdfSource.height },
      comparedSize: {
        width: Math.min(referenceSource.width, pdfSource.width),
        height: Math.min(referenceSource.height, pdfSource.height),
      },
    }
    : null;
  const width = sizeNormalization?.comparedSize.width ?? Math.max(referenceSource.width, pdfSource.width);
  const height = sizeNormalization?.comparedSize.height ?? Math.max(referenceSource.height, pdfSource.height);
  const reference = flattenPngToWhiteCanvas(readFileSync(referencePath), width, height);
  const pdf = flattenPngToWhiteCanvas(readFileSync(pdfPath), width, height);
  const diff = new PNG({ width, height });

  let diffPixels = 0;
  let maxChannelDelta = 0;
  let totalChannelDelta = 0;

  for (let i = 0; i < reference.data.length; i += 4) {
    const dr = Math.abs(reference.data[i] - pdf.data[i]);
    const dg = Math.abs(reference.data[i + 1] - pdf.data[i + 1]);
    const db = Math.abs(reference.data[i + 2] - pdf.data[i + 2]);
    const pixelDelta = Math.max(dr, dg, db);
    maxChannelDelta = Math.max(maxChannelDelta, pixelDelta);
    totalChannelDelta += dr + dg + db;

    if (pixelDelta > channelTolerance) {
      diffPixels += 1;
      diff.data[i] = 255;
      diff.data[i + 1] = 0;
      diff.data[i + 2] = 0;
      diff.data[i + 3] = 255;
    } else {
      diff.data[i] = 255;
      diff.data[i + 1] = 255;
      diff.data[i + 2] = 255;
      diff.data[i + 3] = 0;
    }
  }

  writeFileSync(diffPath, PNG.sync.write(diff));

  const totalPixels = width * height;
  return {
    referenceWidth: referenceSource.width,
    referenceHeight: referenceSource.height,
    pdfWidth: pdfSource.width,
    pdfHeight: pdfSource.height,
    width,
    height,
    sameSize,
    majorSizeMismatch: !sameSize && !sizeNormalization,
    sizeNormalization,
    diffPixels,
    totalPixels,
    diffRatio: totalPixels === 0 ? 1 : diffPixels / totalPixels,
    maxChannelDelta,
    averageChannelDelta: totalPixels === 0 ? 0 : totalChannelDelta / (totalPixels * 3),
  };
}

function renderMarkdownSummary(config, results) {
  const errors = results.filter(result => result.error);
  const warnings = results.filter(result => !result.error && result.exceedsReportThreshold);
  const lines = [
    '# PDF Visual Diff Report',
    '',
    `- status: report-only (does not gate CI)`,
    `- fixtures: ${config.fixtures.map(markdownCell).join(', ')}`,
    `- scale: ${config.scale}`,
    `- raster DPI: ${config.rasterDpi}`,
    `- max pages: ${formatPageLimit(config.maxPages)}`,
    `- channel tolerance: ${config.channelTolerance}`,
    `- command timeout: ${config.commandTimeoutMs}ms`,
    `- report threshold: ${config.maxDiffRatio == null ? 'disabled' : config.maxDiffRatio}`,
    `- compared pages: ${results.filter(result => !result.error).length}`,
    `- warning pages: ${warnings.length}`,
    `- error pages: ${errors.length}`,
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
    if (result.error) {
      lines.push([
        'error',
        markdownCell(result.fixture),
        result.pageIndex + 1,
        '-',
        '-',
        '-',
        '-',
        markdownCell(result.error),
      ].join(' | ').replace(/^/, '| ').replace(/$/, ' |'));
      continue;
    }

    const status = result.exceedsReportThreshold ? 'warn' : 'report';
    const size = result.sameSize
      ? `${result.width}x${result.height}`
      : `${result.referenceWidth}x${result.referenceHeight} vs ${result.pdfWidth}x${result.pdfHeight}`;
    const artifactText = Object.values(result.artifactFiles || {}).join('<br>');
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

  if (warnings.length > 0) {
    lines.push('');
    lines.push('## Warnings');
    lines.push('');
    for (const warning of warnings) {
      const percent = (warning.diffRatio * 100).toFixed(5);
      lines.push(`- ${markdownCell(warning.fixture)} page ${warning.pageIndex + 1}: ${percent}% differs`);
    }
  }

  if (errors.length > 0) {
    lines.push('');
    lines.push('## Errors');
    lines.push('');
    for (const error of errors) {
      lines.push(`- ${markdownCell(error.fixture)} page ${error.pageIndex + 1}: ${markdownCell(error.error)}`);
    }
  }

  lines.push('');
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
  channelTolerance: numberFromEnv('RHWP_RENDER_DIFF_PDF_CHANNEL_TOLERANCE', 8),
  maxDiffRatio: optionalNumberFromEnv('RHWP_RENDER_DIFF_PDF_MAX_RATIO'),
  commandTimeoutMs: COMMAND_TIMEOUT_MS,
  writeImages: process.env.RHWP_RENDER_DIFF_PDF_WRITE_IMAGES !== '0',
};
config.rasterDpi = Math.max(1, Math.round(72 * config.scale));

mkdirSync(ARTIFACT_DIR, { recursive: true });

runTest('PDF export visual diff report', async ({ page }) => {
  const results = [];
  const rhwp = rhwpCommandPrefix();

  try {
    for (const fixture of config.fixtures) {
      setTestCase(`pdf-render-diff ${fixture}`);
      let pageCount;
      try {
        ({ pageCount } = await loadHwpFile(page, fixture));
      } catch (error) {
        results.push({
          fixture,
          pageIndex: 0,
          error: error.message || String(error),
        });
        continue;
      }
      const pageLimit = Math.min(pageCount, config.maxPages);
      const samplePath = resolve(SAMPLES_DIR, ...fixture.split('/'));

      for (let pageIndex = 0; pageIndex < pageLimit; pageIndex += 1) {
        const baseName = `${safeName(fixture)}-p${String(pageIndex + 1).padStart(2, '0')}`;
        const referencePath = join(ARTIFACT_DIR, `${baseName}-pdf-reference.png`);
        const pdfPath = join(ARTIFACT_DIR, `${baseName}.pdf`);
        const pdfRasterPrefix = join(ARTIFACT_DIR, `${baseName}-pdf-raster`);
        const pdfRasterPath = `${pdfRasterPrefix}.png`;
        const diffPath = join(ARTIFACT_DIR, `${baseName}-pdf-diff.png`);

        try {
          for (const artifactPath of [referencePath, pdfPath, pdfRasterPath, diffPath]) {
            removeIfExists(artifactPath);
          }

          const reference = await page.evaluate((args) => {
            const doc = window.__wasm?.doc;
            if (!doc) throw new Error('window.__wasm.doc is not available');
            if (typeof doc.renderPageToCanvas !== 'function') {
              throw new Error('renderPageToCanvas is not available');
            }

            const canvas = document.createElement('canvas');
            doc.renderPageToCanvas(args.pageIndex, canvas, args.scale);
            return {
              width: canvas.width,
              height: canvas.height,
              dataUrl: canvas.toDataURL('image/png'),
            };
          }, { pageIndex, scale: config.scale });

          writeDataUrl(referencePath, reference.dataUrl);

          runCommand(rhwp.command, [
            ...rhwp.args,
            'export-pdf',
            samplePath,
            '-o',
            pdfPath,
            '-p',
            String(pageIndex),
          ]);
          assertFreshPdf(pdfPath);
          runCommand('pdftoppm', [
            '-r',
            String(config.rasterDpi),
            '-singlefile',
            '-png',
            pdfPath,
            pdfRasterPrefix,
          ]);

          const comparison = comparePngPaths(referencePath, pdfRasterPath, diffPath, config.channelTolerance);
          const result = {
            fixture,
            pageIndex,
            rhwpMode: rhwp.mode,
            ...comparison,
            exceedsReportThreshold: comparison.majorSizeMismatch
              || (config.maxDiffRatio != null && comparison.diffRatio > config.maxDiffRatio),
            artifactFiles: {
              reference: `e2e/screenshots/render-diff/${baseName}-pdf-reference.png`,
              pdf: `e2e/screenshots/render-diff/${baseName}.pdf`,
              pdfRaster: `e2e/screenshots/render-diff/${baseName}-pdf-raster.png`,
              diff: `e2e/screenshots/render-diff/${baseName}-pdf-diff.png`,
            },
          };

          if (!config.writeImages) {
            result.artifactFiles = {
              pdf: `e2e/screenshots/render-diff/${baseName}.pdf`,
              diff: `e2e/screenshots/render-diff/${baseName}-pdf-diff.png`,
            };
          }
          results.push(result);
        } catch (error) {
          results.push({
            fixture,
            pageIndex,
            error: error.message || String(error),
          });
        }
      }
    }
  } finally {
    writeReports(config, results);
  }

  const warnings = results.filter(result => result.exceedsReportThreshold).length;
  const errors = results.filter(result => result.error).length;
  console.log(`PDF visual diff report: ${results.length} page(s), ${warnings} warning(s), ${errors} error(s)`);
  console.log(`PDF visual diff summary: ${SUMMARY_PATH}`);
});
