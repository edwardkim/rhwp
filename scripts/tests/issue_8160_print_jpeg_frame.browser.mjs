// 관계형·고정형: 그림 테두리의 연속 잉크와 JPEG의 독립 Print 영역을 실제 Canvas에서 검사한다.
// 주석 갱신: 2026-11-15T00:21:44+09:00. 직전 파일 변경:
// 96827b3accd98ad0b80f76260b42fba24eded504, 2026-11-04T18:33:15+09:00.
// pic-crop-01 전 1쪽: 2026-11-15, 96 DPI·PDFium·2px raw, Native 100.00% / 최적화 fresh WASM 100.00%.
// 측정 근거: mydocs/pr/assets/issue_9060/canvas-image-stage/README.md. 설치 글꼴은 사용하지 않는다.
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

// 고정형: 추가한 편람 140쪽의 수정 전 Native 100%, fresh WASM 99.98238%.
// 주석 갱신: 2026-11-04 15:59 KST. 독립 Print 영역의 기존 2px 내용 경계를 고정한다.
for (const [label, contractPath, regionId] of [
  ['issue_8160 표지', 'tests/fixtures/issue8160/hwp2022.json', 'header-jpeg-print-frame'],
  ['issue_8533 편람 140쪽', 'tests/fixtures/issue8533/handbook-print-frame.json', 'handbook-login-jpeg-print-frame'],
]) test(`${label}: JPEG 인쇄 프레임은 실제 Canvas의 독립 Print 경계 픽셀을 보존한다`, async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const contract = JSON.parse(readFileSync(join(root, contractPath), 'utf8'));
  const [region] = contract.browser_raster_regions;
  assert.equal(region.id, regionId);
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
  assert.equal(result.pageCount, contract.page_count ?? contract.pages.length);
  assert.equal(result.images.length, contract.image_count ?? 2, '원본 그림의 누락·중복');
  const target = result.images[contract.frame_image_index ?? 0];
  assert.equal(target.mime, 'image/jpeg');
  assert.equal(target.frame.length, 8);
  assert.deepEqual(result.errors, []);
  assert.equal(result.width, region.box[2] - region.box[0]);
  assert.equal(result.height, region.box[3] - region.box[1]);
  assert.deepEqual(result.unmatched, [0, 0], '장치 프레임을 일반 비트맵 준비로 잃으면 로고 오른쪽에 여분 잉크가 생긴다');
});

// 관계형: 원본 그림 칸의 위 테두리는 crop 그림 뒤에서도 연속 잉크로 남아야 한다.
// 주석 갱신: 2026-11-14T23:59:25+09:00. 직전 파일 변경:
// 96827b3accd98ad0b80f76260b42fba24eded504, 2026-11-04T18:33:15+09:00.
// 2026-11-14, 96 DPI·2px raw, 전 1쪽 Native 100.00% / 기존 최적화 WASM 94.65%.
// 독립 PDF·진단·제품 재검증은 mydocs/pr/assets/issue_9060/canvas-image-coverage-plan.md를 따른다.
test('잘린 그림의 소수 목적 영역이 아래에 그린 위 테두리를 지우지 않는다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const input = readFileSync(join(root, 'samples/pic-crop-01.hwp'));
  assert.equal(sha(input), '3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c');
  assert.equal(sha(readFileSync(join(root, 'pdf/pic-crop-01-2022.pdf'))),
    '4ae5f6f8bc308bedf16fbca139de448c42d29b42bbba22221f4a1acc9694c4d3');
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/input.hwp', ['application/octet-stream', input]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => {
    server.once('error', reject); server.listen(0, '127.0.0.1', accept);
  });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'cropped-edge');
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
  const result = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js');
    await wasm.default({module_or_path: '/rhwp_bg.wasm'});
    const doc = new wasm.HwpDocument(new Uint8Array(await (await fetch('/input.hwp')).arrayBuffer()));
    const descriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
    const pending = new Set(), errors = [];
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
      doc.setHangul2024Compat(false);
      const tree = JSON.parse(doc.getPageLayerTreeWithProfile(0, 'print', true, true));
      const images = [];
      const collect = node => {
        if (!node || typeof node !== 'object') return;
        if (node.type === 'image') images.push(node);
        for (const value of Object.values(node)) if (value && typeof value === 'object') collect(value);
      };
      collect(tree); images.sort((a, b) => a.bbox.y - b.bbox.y);
      const canvas = document.getElementById('page');
      prepare(canvas);
      for (let attempt = 0; attempt < 4; attempt++) {
        doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
        if (!pending.size) break;
        await Promise.all([...pending]);
        if (attempt === 3) throw Error('image resources not stable');
      }
      const capture = document.createElement('canvas'); canvas.replaceWith(capture); prepare(capture);
      doc.renderPageToCanvasFilteredWithProfile(0, capture, 1, 'all', 'print');
      if (pending.size) throw Error('capture introduced unloaded image');
      const context = capture.getContext('2d');
      // 위치는 실제 그림 칸에서 가져온다. 기대값은 독립 Print에서 확인한
      // 위 테두리의 연속 잉크이며 특정 문서의 절대 좌표를 고정하지 않는다.
      const borders = images.map(({bbox: b}) => {
        const left = Math.ceil(b.x + b.width / 20), right = Math.floor(b.x + b.width * 19 / 20);
        const top = Math.floor(b.y) - 2, bottom = Math.ceil(b.y) + 2;
        const width = right - left;
        const pixels = context.getImageData(left, top, width, bottom - top).data;
        let longest = 0;
        for (let y = 0; y < bottom - top; y++) {
          let run = 0;
          for (let x = 0; x < width; x++) {
            const i = (y * width + x) * 4;
            const ink = Math.min(...pixels.slice(i, i + 3)) < 232 && pixels[i + 3] > 0;
            run = ink ? run + 1 : 0; longest = Math.max(longest, run);
          }
        }
        return {longest, width};
      });
      return {pages: doc.pageCount(), borders, errors};
    } finally {
      Object.defineProperty(HTMLImageElement.prototype, 'src', descriptor); doc.free();
    }
    function prepare(canvas) { canvas.getContext('2d', {willReadFrequently: true}); }
  });
  assert.equal(result.pages, 1);
  assert.equal(result.borders.length, 2, '원본의 두 crop 그림을 모두 검사한다');
  assert.deepEqual(result.errors, []);
  for (const [index, border] of result.borders.entries()) {
    assert.ok(border.longest > border.width / 2,
      `그림 ${index + 1}: 위 테두리 연속 잉크 ${border.longest}/${border.width}`);
  }
});
