/** #7235: 실제 CanvasKit replay에서 crop 뒤 비율과 letterbox를 검사한다.
 * node e2e/run-with-vite.mjs -- node e2e/canvaskit-cropped-contain.test.mjs
 */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import puppeteer from 'puppeteer-core';

const executablePath = process.env.CHROME_PATH || [
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  '/usr/bin/google-chrome', '/usr/bin/chromium',
].find(p => fs.existsSync(p));
const browser = await puppeteer.launch({ executablePath, headless: true, args: ['--no-sandbox'] });
try {
  const page = await browser.newPage();
  await page.goto(`${process.env.VITE_URL || 'http://127.0.0.1:7700'}/`, { waitUntil: 'domcontentloaded' });
  const results = await page.evaluate(async () => {
    const { CanvasKitLayerRenderer } = await import('/src/view/canvaskit-renderer.ts');
    const renderer = await CanvasKitLayerRenderer.create('default', 'software');
    const source = document.createElement('canvas');
    source.width = source.height = 40;
    const ctx = source.getContext('2d');
    ctx.fillStyle = 'red'; ctx.fillRect(0, 0, 40, 20);
    ctx.fillStyle = 'blue'; ctx.fillRect(0, 20, 40, 20);
    const base64 = source.toDataURL().split(',')[1];
    const results = [];
    try {
      for (const fillMode of ['none', 'zoom', 'fitToSize', 'total']) {
        const bbox = { x: 0, y: 0, width: 80, height: 80 };
        const tree = { width: 80, height: 80, profile: 'screen', root: {
          kind: 'leaf', bounds: bbox, ops: [{
            type: 'image', bbox, fillMode, base64, mimeType: 'image/png',
            crop: { left: 0, top: 20, right: 40, bottom: 40 }, originalSizeHu: [40, 40],
          }],
        }};
        const target = document.createElement('canvas');
        target.width = target.height = 80;
        const output = renderer.renderPage(tree, target, 1);
        const copy = document.createElement('canvas');
        copy.width = copy.height = 80;
        const pixels = copy.getContext('2d'); pixels.drawImage(output, 0, 0);
        const sample = y => Array.from(pixels.getImageData(40, y, 1, 1).data);
        const all = pixels.getImageData(0, 0, 80, 80).data;
        let red = 0;
        for (let i = 0; i < all.length; i += 4) if (all[i] > 128 && all[i + 1] < 64 && all[i + 2] < 64) red++;
        results.push({ fillMode, top: sample(10), middle: sample(40), bottom: sample(70), red, png: copy.toDataURL() });
      }
    } finally { renderer.dispose(); }
    return results;
  });
  for (const result of results) {
    assert.deepEqual(result.middle, [0, 0, 255, 255], `${result.fillMode}: crop 영역`);
    const edge = ['none', 'zoom'].includes(result.fillMode) ? [255, 255, 255, 255] : [0, 0, 255, 255];
    assert.deepEqual(result.top, edge, `${result.fillMode}: 위 여백`);
    assert.deepEqual(result.bottom, edge, `${result.fillMode}: 아래 여백`);
    assert.equal(result.red, 0, `${result.fillMode}: 제거된 crop 내용`);
    if (process.env.EVIDENCE_DIR) {
      fs.mkdirSync(process.env.EVIDENCE_DIR, { recursive: true });
      fs.writeFileSync(path.join(process.env.EVIDENCE_DIR, `canvaskit_crop_${result.fillMode}.png`), Buffer.from(result.png.split(',')[1], 'base64'));
    }
    console.log(`PASS ${result.fillMode}: 실제 CanvasKit crop raster`);
  }
} finally { await browser.close(); }
