// 관계형: 실제 WASM·FontFace·Canvas의 font-only 계약. 실물 문서 정확도 증거로 사용하지 않는다.
// 주석 갱신: 2026-11-07 14:17 +0900. 직전 변경 84764fa21c96cd5f8a0e158658e4162a4fb3ce9e
// (2026-10-16T11:32:36+09:00). 저장소 소유 합성 CFF와 공개 편집 API만 사용한다.
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

// 관계형: 같은 실제 CFF의 보통 윤곽을 합성 굵기의 fill·stroke도 소비해야 한다.
// 저장소 소유 합성 글꼴의 계약 검사이며 실물 페이지의 위치 핀이나 시각 gate를 대신하지 않는다.
test('실제 HFT CFF의 합성 굵기도 등록된 원 윤곽을 소비한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source.otf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPHftOutlineJamo.otf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'hft-bold');
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
    const bytes = new Uint8Array(await (await fetch('/source.otf')).arrayBuffer());
    const face = new FontFace('RHWP HFT Outline Fixture', bytes); await face.load(); document.fonts.add(face);
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="RHWP HFT Outline Fixture" Type="hft"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400">
      </CHARSHAPE></CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
      <TEXT CharShape="0"><CHAR>가</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const doc = new wasm.HwpDocument(source);
    try {
      // 현재 HML reader의 미지원 BOLD 태그에 의존하지 않고 실제 공개 편집 경로로 설정한다.
      doc.applyCharFormat(0, 0, 0, 1, JSON.stringify({bold: true}));
      const layer = JSON.parse(doc.getPageLayerTreeWithProfile(0, 'print', true, true));
      const runs = [];
      const walk = value => {
        if (!value || typeof value !== 'object') return;
        if (value.type === 'textRun' && value.text === '가') runs.push(value);
        if (Array.isArray(value)) value.forEach(walk); else Object.values(value).forEach(walk);
      };
      walk(layer);
      const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
      const texts = [], fillText = ctx.fillText.bind(ctx), moveTo = ctx.moveTo.bind(ctx);
      let contours = 0;
      ctx.fillText = (text, ...args) => { texts.push(text); return fillText(text, ...args); };
      ctx.moveTo = (...args) => { contours++; return moveTo(...args); };
      const registered = doc.registerHftOutlineFont('RHWP HFT Outline Fixture', bytes, 0);
      doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
      const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
      let ink = 0;
      for (let i = 0; i < rgba.length; i += 4) if (rgba[i + 3] > 200 && Math.min(rgba[i], rgba[i + 1], rgba[i + 2]) < 32) ink++;
      return {bold: runs.length === 1 && runs[0].paintStyle.bold, registered, texts, contours, ink};
    } finally { doc.free(); }
  });
  assert.equal(result.bold, true, '합성 입력이 실제 굵은 글자 모양으로 해석되어야 한다');
  assert.equal(result.registered, true);
  assert.equal(result.texts.includes('가'), false, '확인한 원 CFF 윤곽을 브라우저의 대체 glyph로 덮지 않는다');
  assert.ok(result.contours > 0 && result.ink > 0, '실제 path와 Canvas 잉크를 모두 소비한다');
});

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
      for (const bold of [false, true]) for (const text of ['가', '\ua960']) {
        const doc = new wasm.HwpDocument(source(text));
        try {
          if (bold) doc.applyCharFormat(0, 0, 0, 1, JSON.stringify({bold: true}));
          if (typeof doc.registerHftOutlineFont !== 'function') throw Error('같은 source의 fresh WASM을 RHWP_SWEEP_TEST_WASM_PKG로 지정해야 합니다.');
          const canvas = document.createElement('canvas'); const ctx = canvas.getContext('2d');
          const fillText = ctx.fillText.bind(ctx); const calls = [];
          ctx.fillText = (...args) => { calls.push({text: args[0], font: ctx.font}); return fillText(...args); };
          const paint = () => { calls.length = 0; doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile); return {png: canvas.toDataURL(), calls: [...calls]}; };
          const before = paint(); const registered = doc.registerHftOutlineFont('RHWP HFT Outline Fixture', original, 0);
          const after = paint(); rows.push({profile, bold, text, registered, identical: before.png === after.png, before: before.calls, after: after.calls});
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
