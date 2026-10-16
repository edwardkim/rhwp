// 실제 WASM·FontFace·Canvas의 font-only 계약. 한컴 문서 정확도 증거로 사용하지 않는다.
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

test('실제 HFT 윤곽 replay가 옛한글 우선 face와 화면 경로를 보존한다', async t => {
  // Frontend package CI는 먼저 같은 source의 WASM을 pkg/에 빌드한다.
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/original.otf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPHftOutlineJamo.otf'))]],
    ['/old-hangul.otf', ['application/octet-stream', readFileSync(join(root, 'ttfs/opensource/RHWPOldHangul-Regular.otf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'hft-selection');
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
    const original = new Uint8Array(await (await fetch('/original.otf')).arrayBuffer());
    const old = await (await fetch('/old-hangul.otf')).arrayBuffer();
    for (const [family, bytes] of [['RHWP HFT Outline Fixture', original], ['RHWP Old Hangul', old]]) {
      const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
    }
    await document.fonts.ready;
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = text => new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="RHWP HFT Outline Fixture" Type="hft"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/>
      </CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
      <TEXT CharShape="0"><CHAR>${text}</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const rows = [];
    for (const profile of ['screen', 'print', 'high-quality']) {
      for (const text of ['가', '\ua960']) {
        const doc = new wasm.HwpDocument(source(text));
        try {
          if (typeof doc.registerHftOutlineFont !== 'function') throw Error('같은 source의 fresh WASM을 RHWP_SWEEP_TEST_WASM_PKG로 지정해야 합니다.');
          const canvas = document.createElement('canvas'); const ctx = canvas.getContext('2d');
          const fillText = ctx.fillText.bind(ctx); const calls = [];
          ctx.fillText = (...args) => { calls.push({text: args[0], font: ctx.font}); return fillText(...args); };
          const paint = () => { calls.length = 0; doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile); return {png: canvas.toDataURL(), calls: [...calls]}; };
          const before = paint(); const registered = doc.registerHftOutlineFont('RHWP HFT Outline Fixture', original, 0);
          const after = paint(); rows.push({profile, text, registered, identical: before.png === after.png, before: before.calls, after: after.calls});
        } finally { doc.free(); }
      }
    }
    return rows;
  });
  for (const row of result) {
    assert.equal(row.registered, true, JSON.stringify(row));
    assert.ok(row.before.some(call => call.text === row.text), JSON.stringify(row));
    if (row.text === '\ua960' || row.profile === 'screen') {
      assert.equal(row.identical, true, `선택된 face의 실제 PNG를 보존해야 한다: ${JSON.stringify(row)}`);
      assert.ok(row.after.some(call => call.text === row.text), JSON.stringify(row));
      if (row.text === '\ua960') assert.ok(row.after.some(call => call.font.includes('RHWP Old Hangul')), JSON.stringify(row));
    } else {
      assert.equal(row.after.some(call => call.text === row.text), false,
        `일반 HFT 윤곽 경로를 전체 비활성화해 통과하면 안 된다: ${JSON.stringify(row)}`);
    }
  }
});
