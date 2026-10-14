import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {join, resolve} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require = createRequire(join(root, 'rhwp-studio/package.json'));

test('공급한 family가 실제 Canvas advance와 글리프를 결정한다', async t => {
  const path = join(root, 'tests/fixtures/fonts/RHWPHostFixture-Regular.ttf');
  const bytes = readFileSync(path);
  const server = createSweepServer(new Map([
    ['/font-registration.mjs', ['application/javascript', readFileSync(join(root, 'scripts/visual-sweep-font-registration.mjs'))]],
  ]), new Map([['/font/0', {path, sha256: createHash('sha256').update(bytes).digest('hex')}]]));
  await new Promise((accept, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', accept);
  });
  t.after(async () => {
    server.closeAllConnections();
    await new Promise(accept => server.close(accept));
  });
  const browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true,
  });
  t.after(() => browser.close());
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const results = await page.evaluate(async () => {
    const {installCanvasFont} = await import('/font-registration.mjs');
    const bytes = await (await fetch('/font/0')).arrayBuffer();
    const ctx = document.getElementById('page').getContext('2d');
    const results = [];
    for (const family of ['RHWP Sweep Absent Alias', 'RHWP 검증 글꼴']) {
      const font = await installCanvasFont(family, bytes, {weight: '400', style: 'normal'});
      ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
      ctx.font = `40px ${JSON.stringify(family)}`;
      ctx.fillText('A', 10, 50);
      const pixels = ctx.getImageData(0, 0, ctx.canvas.width, ctx.canvas.height);
      let left = Infinity, right = -1, top = Infinity, bottom = -1;
      for (let y = 0; y < pixels.height; y++) {
        for (let x = 0; x < pixels.width; x++) {
          if (!pixels.data[(y * pixels.width + x) * 4 + 3]) continue;
          left = Math.min(left, x); right = Math.max(right, x);
          top = Math.min(top, y); bottom = Math.max(bottom, y);
        }
      }
      results.push({family, status: font.status, advance: ctx.measureText('A').width,
                    inkWidth: right - left + 1, inkHeight: bottom - top + 1});
    }
    return results;
  });
  for (const result of results) {
    assert.equal(result.status, 'loaded');
    // 기존 fixture generator: 1000 UPM, 900 advance, 300×700 직사각형.
    assert.ok(Math.abs(result.advance - 36) < 0.01, JSON.stringify(result));
    assert.equal(result.inkWidth, 12, JSON.stringify(result));
    assert.equal(result.inkHeight, 28, JSON.stringify(result));
  }
});
