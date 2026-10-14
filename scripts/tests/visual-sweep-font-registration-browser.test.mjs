import assert from 'node:assert/strict';
import {execFile} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {isAbsolute, join, resolve, sep} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {promisify} from 'node:util';
import {createSweepBrowserProfile, createSweepServer} from '../export-wasm-png-for-sweep.mjs';
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
  const profileRoot = join(root, 'output/pr-review/browser-profile-tests');
  const userDataDir = createSweepBrowserProfile(profileRoot, 0);
  assert.ok(isAbsolute(userDataDir));
  assert.ok(userDataDir.startsWith(resolve(profileRoot) + sep));
  let browser;
  t.after(async () => {
    if (browser) {
      await browser.close();
      // 실제 Chrome도 host와 같은 경로를 썼는지 확인한다(Snap /tmp 누수 방지).
      assert.ok(existsSync(join(userDataDir, 'Default/Preferences')));
    }
    rmSync(userDataDir, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
    assert.equal(existsSync(userDataDir), false);
  });
  browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir,
  });
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

test('Chrome 시작 실패 후에도 output 전용 프로필을 정리한다', async t => {
  const work = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'launch-fault');
  t.after(() => rmSync(work, {recursive: true, force: true, maxRetries: 5, retryDelay: 100}));
  const pkg = join(work, 'pkg'), output = join(work, 'out');
  mkdirSync(pkg);
  // 브라우저 시작 단계의 fault injection이다. 문서 파싱·조판은 실행하지 않는다.
  writeFileSync(join(pkg, 'rhwp.js'), '');
  writeFileSync(join(pkg, 'rhwp_bg.wasm'), Buffer.alloc(0));
  writeFileSync(join(work, 'source'), Buffer.alloc(0));
  writeFileSync(join(work, 'fonts.json'), JSON.stringify({faces: []}));
  writeFileSync(join(work, 'options.json'), JSON.stringify({compat: '2022', scale: 1, workers: 1, pages: [1]}));
  await assert.rejects(promisify(execFile)(process.execPath, [
    join(root, 'scripts/export-wasm-png-for-sweep.mjs'), '--pkg', pkg,
    '--input', join(work, 'source'), '--out', output,
    '--fonts', join(work, 'fonts.json'), '--options', join(work, 'options.json'),
  ], {
    // Node는 존재하지만 Chrome 전용 인자를 받지 못하므로 실제 launch가 실패한다.
    env: {...process.env, VISUAL_SWEEP_CHROME: process.execPath}, timeout: 15_000,
  }), error => !error.killed && /Failed to launch/.test(error.stderr));
  assert.deepEqual(readdirSync(output).filter(name => name.startsWith('chrome-worker-')), []);
});
