// 실제 문서·WASM·Canvas의 SVG crop 소비 계약. 최종 피델리티는 별도 Print Sweep으로 확인한다.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync, rmSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {createSweepBrowserProfile, createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require = createRequire(join(root, 'rhwp-studio/package.json'));

test('실제 편람 WMF의 저장 crop이 Canvas source rect로 소비된다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/input.hwp', ['application/octet-stream', readFileSync(join(root, 'samples/2025 행정업무운영 편람(최종).hwp'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => {
    server.once('error', reject); server.listen(0, '127.0.0.1', accept);
  });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'wmf-crop');
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
    const wasm = await import('/rhwp.js'); await wasm.default();
    const doc = new wasm.HwpDocument(new Uint8Array(await (await fetch('/input.hwp')).arrayBuffer()));
    const descriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
    const pending = new Set(); const errors = [];
    Object.defineProperty(HTMLImageElement.prototype, 'src', {
      ...descriptor, set(value) {
        const promise = new Promise(accept => {
          this.addEventListener('load', accept, {once: true});
          this.addEventListener('error', () => { errors.push('그림 decode 실패'); accept(); }, {once: true});
        });
        pending.add(promise); promise.finally(() => pending.delete(promise));
        descriptor.set.call(this, value);
      },
    });
    try {
      doc.setHangul2024Compat(true);
      const layer = JSON.parse(doc.getPageLayerTreeWithProfile(129, 'print', true, true));
      const images = [];
      const collect = node => {
        if (!node || typeof node !== 'object') return;
        if (node.type === 'image' && /:376:/.test(node.sourceImageKey || '')) images.push(node);
        for (const value of Object.values(node)) if (value && typeof value === 'object') collect(value);
      };
      collect(layer);
      if (images.length !== 1) throw Error(`원본 그림376의 중복·누락: ${images.length}`);
      const op = images[0];
      const bytes = doc.getSourceImageBytes(op.sourceImageKey);
      let binary = '';
      for (const byte of bytes) binary += String.fromCharCode(byte);
      const url = `data:${op.mime};base64,${btoa(binary)}`;
      const svg = new DOMParser().parseFromString(new TextDecoder().decode(bytes), 'image/svg+xml').documentElement;
      const viewport = svg.getAttribute('viewBox').trim().split(/\s+/).map(Number);
      const canvas = document.getElementById('page'); const ctx = canvas.getContext('2d');
      const draw = ctx.drawImage.bind(ctx); let calls = [];
      ctx.drawImage = (image, ...args) => {
        if (image.src === url) calls.push({args, width: image.naturalWidth, height: image.naturalHeight});
        return draw(image, ...args);
      };
      for (let attempt = 0; attempt < 4; attempt++) {
        calls = [];
        doc.renderPageToCanvasFilteredWithProfile(129, canvas, 1, 'all', 'print');
        if (!pending.size) break;
        await Promise.all([...pending]);
      }
      return {calls, errors, viewport, crop: op.crop, reference: op.originalSizeHu,
        intrinsicWidth: svg.getAttribute('width'), intrinsicHeight: svg.getAttribute('height')};
    } finally {
      Object.defineProperty(HTMLImageElement.prototype, 'src', descriptor);
      doc.free();
    }
  });
  assert.deepEqual(result.errors, []);
  const crops = result.calls.filter(call => call.args.length === 8);
  assert.equal(crops.length, 1,
    `SVG도 실제 Canvas의 crop source rect를 사용해야 한다: ${JSON.stringify(result.calls)}`);
  const [call] = crops;
  const [, , width, height] = result.viewport;
  assert.equal(Number(result.intrinsicWidth), width, 'SVG 픽셀 크기와 viewBox 좌표계를 일치시킨다');
  assert.equal(Number(result.intrinsicHeight), height);
  assert.equal(call.width, width); assert.equal(call.height, height);
  const {left, top, right, bottom} = result.crop;
  const [referenceWidth, referenceHeight] = result.reference;
  assert.ok(top > 0 && bottom > top, '실제 저장본의 위쪽 crop을 검사한다');
  const expected = [left / referenceWidth * width, top / referenceHeight * height,
    (right - left) / referenceWidth * width, (bottom - top) / referenceHeight * height];
  for (let index = 0; index < 4; index++) {
    assert.ok(Math.abs(call.args[index] - expected[index]) < 1e-6,
      `저장 원본 좌표를 한 번 환산해야 한다: ${JSON.stringify({actual: call.args.slice(0, 4), expected})}`);
  }
});
