// 합성 API 계약: 실제 WASM의 glyph paint와 Screen/Print 전환을 관측한다.
// 실물 한컴 원본의 Visual Sweep을 대신하지 않는다.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync, rmSync} from 'node:fs';
import {join, resolve} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {createSweepBrowserProfile, createSweepServer} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const require = createRequire(join(root, 'rhwp-studio/package.json'));

test('실제 Canvas Print 글리프는 기하 모드를 소비하고 Screen으로 복귀한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/font.ttf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPEquationLatinOnly.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'print-geometry');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({
    executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile,
  });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const font = new FontFace('RHWP Equation Contract', await (await fetch('/font.ttf')).arrayBuffer());
    await font.load(); document.fonts.add(font); await document.fonts.ready;
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const bytes = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="RHWP Equation Contract" Type="ttf"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/>
      </CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
      <TEXT CharShape="0"><CHAR>H</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const doc = new wasm.HwpDocument(bytes);
    try {
      const canvas = document.createElement('canvas'); const ctx = canvas.getContext('2d');
      if (!('textRendering' in ctx)) throw Error('기하 모드를 실제 지원하는 브라우저가 필요합니다.');
      const original = ctx.fillText.bind(ctx); const calls = [];
      ctx.fillText = (...args) => {
        const m = ctx.getTransform();
        calls.push({text: args[0], mode: ctx.textRendering, font: ctx.font,
          transform: [m.a, m.b, m.c, m.d, m.e, m.f]});
        return original(...args);
      };
      const rows = [];
      for (const profile of ['screen', 'print', 'screen', 'print']) {
        calls.length = 0;
        doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
        const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        let ink = 0;
        for (let i = 0; i < rgba.length; i += 4) {
          if (rgba[i + 3] > 0 && Math.min(rgba[i], rgba[i + 1], rgba[i + 2]) < 232) ink++;
        }
        rows.push({profile, calls: [...calls], ink, tree: doc.getPageRenderTree(0)});
      }
      return rows;
    } finally { doc.free(); }
  });
  const first = result[0];
  for (const row of result) {
    assert.ok(row.ink > 0 && row.calls.some(call => call.text === 'H'), JSON.stringify(row));
    assert.equal(row.tree, first.tree, '프로필 전환으로 원본 배치·소속을 바꾸지 않는다');
    assert.deepEqual(row.calls.map(call => call.transform), first.calls.map(call => call.transform),
      '글리프 원점·장평을 모드 보정 수치로 바꾸지 않는다');
    const expected = row.profile === 'print' ? 'geometricPrecision' : 'auto';
    assert.ok(row.calls.every(call => call.mode === expected),
      `실제 glyph paint의 ${row.profile} 모드: ${JSON.stringify(row.calls)}`);
  }
});
