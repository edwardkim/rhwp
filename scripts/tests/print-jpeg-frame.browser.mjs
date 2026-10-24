// 실제 Canvas의 JPEG 재생을 독립 한컴 Print 영역과 비교한다. 설치 글꼴은 사용하지 않는다.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {readFileSync, rmSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {createSweepBrowserProfile, createSweepServer, preparePrintReadbackCanvas} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require = createRequire(join(root, 'rhwp-studio/package.json'));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

test('JPEG 인쇄 프레임은 실제 Canvas의 독립 Print 경계 픽셀을 보존한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const contract = JSON.parse(readFileSync(join(root, 'tests/fixtures/issue8160/hwp2022.json'), 'utf8'));
  const [region] = contract.browser_raster_regions;
  assert.equal(region.id, 'header-jpeg-print-frame');
  assert.equal(contract.dpi, 96);
  assert.equal(region.threshold, 232);
  assert.equal(region.symmetric_radius_pixels, 2);
  const input = readFileSync(join(root, contract.sample));
  const oracle = readFileSync(join(root, region.oracle));
  assert.equal(sha(input), contract.sample_sha256);
  assert.equal(sha(oracle), region.oracle_sha256);
  assert.equal(sha(readFileSync(join(root, contract.pdf))), contract.pdf_sha256);
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/input.hwp', ['application/octet-stream', input]],
    ['/oracle.png', ['image/png', oracle]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => {
    server.once('error', reject); server.listen(0, '127.0.0.1', accept);
  });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'jpeg-frame');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true,
    userDataDir: profile, protocolTimeout: 180000,
  });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  await page.$eval('#page', preparePrintReadbackCanvas);
  const result = await page.evaluate(async ({box, threshold, radius, pageIndex, compat2024}) => {
    const wasm = await import('/rhwp.js');
    await wasm.default({module_or_path: '/rhwp_bg.wasm'});
    const doc = new wasm.HwpDocument(new Uint8Array(await (await fetch('/input.hwp')).arrayBuffer()));
    const descriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
    const pending = new Set(); const errors = [];
    Object.defineProperty(HTMLImageElement.prototype, 'src', {
      ...descriptor, set(value) {
        const loaded = new Promise(accept => {
          this.addEventListener('load', accept, {once: true});
          this.addEventListener('error', () => { errors.push('image decode'); accept(); }, {once: true});
        });
        pending.add(loaded); loaded.finally(() => pending.delete(loaded));
        descriptor.set.call(this, value);
      },
    });
    try {
      doc.setHangul2024Compat(compat2024);
      const count = doc.pageCount();
      const tree = JSON.parse(doc.getPageLayerTreeWithProfile(pageIndex, 'print', true, true));
      const images = [];
      const collect = node => {
        if (!node || typeof node !== 'object') return;
        if (node.type === 'image') images.push({bbox: node.bbox, mime: node.mime, frame: node.imagePrintFrame});
        for (const value of Object.values(node)) if (value && typeof value === 'object') collect(value);
      };
      collect(tree); images.sort((a, b) => a.bbox.y - b.bbox.y);
      const canvas = document.getElementById('page');
      for (let attempt = 0; attempt < 4; attempt++) {
        doc.renderPageToCanvasFilteredWithProfile(pageIndex, canvas, 1, 'all', 'print');
        if (!pending.size) break;
        await Promise.all([...pending]);
        if (attempt === 3) throw Error('image resources not stable');
      }
      // 워밍업 context의 누적 상태를 캡처에 이어받지 않는다.
      const capture = document.createElement('canvas'); capture.id = 'page'; canvas.replaceWith(capture);
      capture.getContext('2d', {willReadFrequently: true});
      doc.renderPageToCanvasFilteredWithProfile(pageIndex, capture, 1, 'all', 'print');
      if (pending.size) throw Error('capture introduced unloaded image');
      const [x, y, right, bottom] = box, width = right - x, height = bottom - y;
      const actual = capture.getContext('2d').getImageData(x, y, width, height).data;
      const reference = new Image();
      await new Promise((accept, reject) => {
        reference.onload = accept; reference.onerror = reject; reference.src = '/oracle.png';
      });
      const expected = document.createElement('canvas'); expected.width = width; expected.height = height;
      expected.getContext('2d').drawImage(reference, 0, 0);
      const baseline = expected.getContext('2d').getImageData(0, 0, width, height).data;
      const mask = rgba => {
        const ink = new Uint8Array(width * height);
        for (let i = 0; i < ink.length; i++) {
          const alpha = rgba[4 * i + 3] / 255;
          let value = 255;
          for (let channel = 0; channel < 3; channel++) {
            value = Math.min(value, Math.round(rgba[4 * i + channel] * alpha + 255 * (1 - alpha)));
          }
          ink[i] = value < threshold ? 1 : 0;
        }
        return ink;
      };
      const unmatched = (left, right) => {
        let count = 0;
        for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
          if (!left[y * width + x]) continue;
          let found = false;
          for (let dy = -radius; dy <= radius && !found; dy++) for (let dx = -radius; dx <= radius; dx++) {
            const nx = x + dx, ny = y + dy;
            if (nx >= 0 && nx < width && ny >= 0 && ny < height && right[ny * width + nx]) {
              found = true; break;
            }
          }
          if (!found) count++;
        }
        return count;
      };
      const left = mask(actual), rightMask = mask(baseline);
      return {pageCount: count, images, errors, unmatched: [unmatched(left, rightMask), unmatched(rightMask, left)],
        width: reference.naturalWidth, height: reference.naturalHeight};
    } finally {
      Object.defineProperty(HTMLImageElement.prototype, 'src', descriptor); doc.free();
    }
  }, {box: region.box, threshold: region.threshold, radius: region.symmetric_radius_pixels,
    pageIndex: region.page - 1, compat2024: contract.compat2024});
  assert.equal(result.pageCount, contract.pages.length);
  assert.equal(result.images.length, 2, '원본 표지 그림의 누락·중복');
  assert.equal(result.images[0].mime, 'image/jpeg');
  assert.equal(result.images[0].frame.length, 8);
  assert.deepEqual(result.errors, []);
  assert.equal(result.width, region.box[2] - region.box[0]);
  assert.equal(result.height, region.box[3] - region.box[1]);
  assert.deepEqual(result.unmatched, [0, 0], '장치 프레임을 일반 비트맵 준비로 잃으면 로고 오른쪽에 여분 잉크가 생긴다');
});
