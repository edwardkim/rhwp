// 관계형: 실제 WASM·FontFace·Canvas의 공급 프로그램 선택을 검사한다. 실물 raw 측정과 구분한다.
// 주석 갱신: 2026-11-08T08:10:55.558137+09:00. 직전 변경 631ba67cee528e4bd65c23042d9071b29622dfba
// (Git 커밋 시각 2026-11-08T07:11:47+09:00).
// 저장소 소유 CFF·OFL TTF를 직접 공급하며 시스템 글꼴 설치나 상용 프로그램 배포에 의존하지 않는다.
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
test('HFT·TTF 선언의 합성 굵기가 실제 CFF를 쓰고 공급된 TTF는 보존한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source.otf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPHftOutlineJamo.otf'))]],
    ['/source.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/opensource/NotoSansKR-Regular.ttf'))]],
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
  const results = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const bytes = new Uint8Array(await (await fetch('/source.otf')).arrayBuffer());
    // 합성 굵기 계약이 명시된 source family에 소유 fixture를 실제 FontFace로 연결한다.
    // 상용 글꼴 프로그램을 사용하거나 그 글리프의 시각 일치를 주장하는 검사가 아니다.
    const family = '한양신명조';
    const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const results = [];
    for (const sourceType of ['hft', 'ttf', 'unknown']) {
      const source = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
        <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
        ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="${family}" Type="${lang === 'Hangul' ? sourceType : 'hft'}"/></FONTFACE>`).join('')}
        </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400">
        </CHARSHAPE></CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
        <TEXT CharShape="0"><CHAR>가</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
      const doc = new wasm.HwpDocument(source);
      try {
        // 현재 HML reader의 미지원 BOLD 태그에 의존하지 않고 실제 공개 편집 경로로 설정한다.
        doc.applyCharFormat(0, 0, 0, 1, JSON.stringify({bold: true}));
        const svg = doc.renderPageSvgWithProfile(0, 'print');
        const layer = JSON.parse(doc.getPageLayerTreeWithProfile(0, 'print', true, true));
        const runs = [];
        const walk = value => {
          if (!value || typeof value !== 'object') return;
          if (value.type === 'textRun' && value.text === '가') runs.push(value);
          if (Array.isArray(value)) value.forEach(walk); else Object.values(value).forEach(walk);
        };
        walk(layer);
        const registered = doc.registerHftOutlineFont(family, bytes, 0);
        const rows = [];
        for (const profile of ['screen', 'print', 'high-quality']) {
          const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
          const texts = [], fillText = ctx.fillText.bind(ctx), moveTo = ctx.moveTo.bind(ctx);
          let contours = 0;
          ctx.fillText = (text, ...args) => { texts.push(text); return fillText(text, ...args); };
          ctx.moveTo = (...args) => { contours++; return moveTo(...args); };
          doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
          const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
          let ink = 0;
          for (let i = 0; i < rgba.length; i += 4) if (rgba[i + 3] > 200 && Math.min(rgba[i], rgba[i + 1], rgba[i + 2]) < 32) ink++;
          rows.push({profile, texts, contours, ink});
        }
        const fauxStroke = Number(/<text\b[^>]*stroke-width="([\d.]+)"[^>]*>가<\/text>/.exec(svg)?.[1]);
        let trueTypeAliasWins = null;
        if (sourceType === 'ttf') {
          const alias = `__rhwp_ttf__${family}`;
          const ttf = new Uint8Array(await (await fetch('/source.ttf')).arrayBuffer());
          const font = new FontFace(alias, ttf); await font.load(); document.fonts.add(font);
          doc.registerTextFontGlyphCoverage(alias, ttf, 0);
          const paint = () => {
            const canvas = document.createElement('canvas');
            doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
            return canvas.toDataURL();
          };
          doc.clearTextFontFamily(family);
          const before = paint();
          doc.registerHftOutlineFont(family, bytes, 0);
          trueTypeAliasWins = before === paint();
        }
        let missingGlyphControl = null;
        if (sourceType === 'hft') {
          // 분리 CFF에 없는 숫자는 같은 family의 TTF 동반 face가 소유한다.
          // 불완전한 coverage에서 뒤 family를 고른 결과로 이를 덮어쓰면 안 된다.
          const ttf = new Uint8Array(await (await fetch('/source.ttf')).arrayBuffer());
          const companion = new FontFace(family, ttf); await companion.load(); document.fonts.add(companion);
          const preferred = new FontFace(family, bytes); await preferred.load(); document.fonts.add(preferred);
          for (const fallback of ['Haansoft Batang', 'HYMyeongJo-Extra', 'HY견명조', 'HCR Batang']) {
            const font = new FontFace(fallback, ttf); await font.load(); document.fonts.add(font);
            doc.registerTextFontGlyphCoverage(fallback, ttf, 0);
          }
          doc.insertText(0, 0, 1, '0');
          doc.applyCharFormat(0, 0, 1, 2, JSON.stringify({bold: true}));
          const profiles = [];
          for (const profile of ['screen', 'print']) {
            const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
            const texts = [], fillText = ctx.fillText.bind(ctx);
            ctx.fillText = (text, ...args) => { texts.push(text); return fillText(text, ...args); };
            doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
            profiles.push({profile, texts});
          }
          missingGlyphControl = {profiles};
        }
        results.push({sourceType, bold: runs.length === 1 && runs[0].paintStyle.bold, fauxBold: fauxStroke > 0, registered, rows, trueTypeAliasWins, missingGlyphControl});
      } finally { doc.free(); }
    }
    return results;
  });
  for (const result of results) {
    assert.equal(result.bold, true, '합성 입력이 실제 굵은 글자 모양으로 해석되어야 한다');
    assert.equal(result.fauxBold, true, '보통 bold face 요청이 아닌 실제 합성 굵기 경계에 진입한다');
    assert.equal(result.registered, true);
    if (result.sourceType === 'ttf') {
      assert.equal(result.trueTypeAliasWins, true, '실제 TTF 원 프로그램이 있으면 CFF 등록으로 덮어쓰지 않는다');
    }
    if (result.missingGlyphControl) {
      for (const control of result.missingGlyphControl.profiles) {
        assert.equal(control.texts.includes('0'), true,
          `${control.profile}: 분리 CFF의 누락 글리프는 동반 face를 아는 Canvas에 맡겨야 한다`);
        if (control.profile === 'print') assert.equal(control.texts.includes('가'), false,
          'CFF가 소유한 인쇄 글리프의 기존 윤곽 경로는 유지한다');
      }
    }
    for (const row of result.rows) {
      assert.equal(row.texts.includes('가'), row.profile === 'screen',
        `${result.sourceType}: 실제 TTF가 없으면 Print는 등록된 CFF를 쓰고 Screen은 보존한다: ${JSON.stringify(row)}`);
      assert.ok(row.ink > 0, '실제 Canvas 잉크를 소비한다');
      if (row.profile !== 'screen') assert.ok(row.contours > 0, '원 CFF의 실제 path를 소비한다');
    }
  }
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
      for (const bold of [false, true]) for (const text of (bold ? ['\ua960'] : ['가', '\ua960'])) {
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
