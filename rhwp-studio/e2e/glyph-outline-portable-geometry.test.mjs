/** #536: 실제 Chromium CanvasKit raster에서 strict geometry 거절과 전체 fallback을 검사한다.
 * CHROMIUM_PATH=/path/to/chromium npm run e2e:glyph-outline-portable-geometry
 */
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import puppeteer from 'puppeteer-core';

const executablePath = process.env.CHROMIUM_PATH || process.env.CHROME_PATH || [
  '/usr/bin/chromium', '/usr/bin/chromium-browser',
].find(candidate => fs.existsSync(candidate));
assert.ok(executablePath, 'CHROMIUM_PATH에 Chromium 실행 파일을 지정하세요');
const browserVersion = execFileSync(executablePath, ['--version'], { encoding: 'utf8' }).trim();

const browser = await puppeteer.launch({ executablePath, headless: true, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage();
  const pageErrors = [];
  page.on('pageerror', error => pageErrors.push(error.message));
  // 빈 문서에서 production renderer만 로드한다. Studio editor/WASM boot는 이 소비단 검사의 대상이 아니다.
  const harnessUrl = new URL('/__glyph_geometry__', process.env.VITE_URL || 'http://127.0.0.1:7700').href;
  await page.setRequestInterception(true);
  page.on('request', request => {
    if (request.isNavigationRequest() && request.url() === harnessUrl) {
      void request.respond({ status: 200, contentType: 'text/html', body: '<!doctype html><title>Glyph geometry raster</title>' });
    } else {
      void request.continue();
    }
  });
  await page.goto(harnessUrl, { waitUntil: 'domcontentloaded' });
  const result = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const renderer = await CanvasKitLayerRenderer.create('default', 'software');
    const bounds = { x: 0, y: 0, width: 96, height: 96 };
    const group = 'portable-glyph-geometry';
    const fallback = {
      type: 'textRun', bbox: { x: 12, y: 12, width: 60, height: 32 },
      text: 'AB', baseline: 28, style: { fontSize: 28, color: '#000000' },
      variant: { equivalenceGroup: group, variantId: 'fallback', variantKind: 'textRun', isDefaultFallback: true },
    };
    const outline = (partIndex = 0, partCount = 1) => ({
      type: 'glyphOutline', payloadKind: 'monochromeFill', bbox: bounds,
      variant: { equivalenceGroup: group, variantId: 'outline', variantKind: 'glyphOutline', partIndex, partCount },
      paintStyle: { color: partIndex === 0 ? '#ff0000' : '#0000ff' },
      placement: { baselineY: 0, runToPage: { a: 1, b: 0, c: 0, d: 1, e: 32 + partIndex * 32, f: 64 } },
      paths: [{ fillRule: 'nonzero', commands: [
        { type: 'moveTo', x: -12, y: -16 },
        { type: 'lineTo', x: 12, y: -16 },
        { type: 'lineTo', x: 12, y: 8 },
        { type: 'lineTo', x: -12, y: 8 },
        { type: 'closePath' },
      ] }],
      diagnostics: { strictVisualEligible: true, quality: 'exact', replayEligibility: 'portable' },
    });
    const raster = ops => {
      const target = document.createElement('canvas');
      target.width = bounds.width;
      target.height = bounds.height;
      const output = renderer.renderPage({ width: bounds.width, height: bounds.height, profile: 'screen',
        root: { kind: 'leaf', bounds, ops } }, target, 1);
      const copy = document.createElement('canvas');
      copy.width = bounds.width;
      copy.height = bounds.height;
      const context = copy.getContext('2d');
      context.drawImage(output, 0, 0);
      const pixels = context.getImageData(0, 0, bounds.width, bounds.height).data;
      const diagnostics = renderer.diagnostics();
      if (!diagnostics.lastRenderCompleted || diagnostics.lastRenderError
        || diagnostics.lastUnexpectedUnsupportedOps.length > 0) {
        throw new Error(`CanvasKit replay 실패: ${JSON.stringify(diagnostics)}`);
      }
      return { pixels, png: copy.toDataURL(), surface: diagnostics.surfaceBackend };
    };
    const differentPixels = (left, right) => {
      let count = 0;
      for (let offset = 0; offset < left.length; offset += 4) {
        if (left[offset] !== right[offset] || left[offset + 1] !== right[offset + 1]
          || left[offset + 2] !== right[offset + 2] || left[offset + 3] !== right[offset + 3]) count += 1;
      }
      return count;
    };
    try {
      const blank = raster([]);
      const baseline = raster([fallback]);
      const positives = [];
      const cases = [];
      const screenshots = { fallback: baseline.png };
      for (const partCount of [1, 2]) {
        const parts = Array.from({ length: partCount }, (_, index) => outline(index, partCount));
        const direct = raster(parts.map(({ variant: _variant, ...op }) => op));
        const selected = raster([fallback, ...parts]);
        positives.push({ partCount, directInk: differentPixels(direct.pixels, blank.pixels),
          fallbackDifference: differentPixels(direct.pixels, baseline.pixels),
          selectionDifference: differentPixels(selected.pixels, direct.pixels) });
        screenshots[`strict-${partCount}-part`] = selected.png;
        for (const value of [-1e100, 1e100]) {
          for (const field of ['path', 'baseline', 'affine']) {
            for (let partIndex = 0; partIndex < partCount; partIndex += 1) {
              const damaged = structuredClone(parts);
              const part = damaged[partIndex];
              if (field === 'path') part.paths[0].commands[0].x = value;
              if (field === 'baseline') part.placement.baselineY = value;
              if (field === 'affine') part.placement.runToPage.e = value;
              const actual = raster([fallback, ...damaged]);
              const name = `${partCount}-part:${partIndex}:${field}:${value}`;
              cases.push({ name, fallbackDifference: differentPixels(actual.pixels, baseline.pixels) });
              if (partCount === 2 && partIndex === 1 && field === 'path' && value > 0) {
                screenshots['overflow-fallback'] = actual.png;
              }
            }
          }
        }
      }
      return { surface: baseline.surface, fallbackInk: differentPixels(baseline.pixels, blank.pixels),
        positives, cases, screenshots };
    } finally {
      renderer.dispose();
    }
  });
  assert.deepEqual(pageErrors, [], '브라우저 예외가 없어야 합니다');
  assert.equal(result.surface, 'software');
  assert.ok(result.fallbackInk > 0, 'TextRun 대조군이 실제 글자 픽셀을 출력해야 합니다');
  for (const positive of result.positives) {
    assert.ok(positive.directInk > 0, `${positive.partCount}-part: 정상 outline 잉크`);
    assert.ok(positive.fallbackDifference > 0, `${positive.partCount}-part: 정상 outline과 fallback은 다른 출력`);
    assert.equal(positive.selectionDifference, 0, `${positive.partCount}-part: 정상 strict 선택은 direct outline과 동일`);
  }
  for (const item of result.cases) {
    assert.equal(item.fallbackDifference, 0, `${item.name}: 전체 TextRun fallback 보존`);
  }
  const { screenshots, ...summary } = result;
  if (process.env.EVIDENCE_DIR) {
    fs.mkdirSync(process.env.EVIDENCE_DIR, { recursive: true });
    for (const [name, png] of Object.entries(screenshots)) {
      fs.writeFileSync(path.join(process.env.EVIDENCE_DIR, `glyph-geometry-${name}.png`), Buffer.from(png.split(',')[1], 'base64'));
    }
    fs.writeFileSync(path.join(process.env.EVIDENCE_DIR, 'glyph-geometry-raster.json'), `${JSON.stringify({ browserVersion, ...summary }, null, 2)}\n`);
  }
  console.log(`PASS ${browserVersion}: 정상 strict ${result.positives.length}건, overflow 전체 fallback ${result.cases.length}건; TextRun 잉크 ${result.fallbackInk}px`);
} finally {
  await browser.close();
}
