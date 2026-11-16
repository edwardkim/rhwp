// 관계형: 실제 WASM·FontFace·Canvas의 공급 프로그램 선택을 검사한다. 실물 raw 측정과 구분한다.
// 추가 관계 검증: 2026-11-15T17:38:44+09:00; 직전 파일 e29231c1f5cd86e6fc31445d672641423bcb1def (2026-11-15T12:51:50+09:00).
// source 8ad1c5c08a, 독립 Print/PDFium/96dpi/2px raw: 중첩 표 전 17쪽 Native 최저 98.63%, optimized fresh WASM 최저 98.65%.
// 11쪽 99.67/99.63%, 14쪽 99.42/99.35%; 좌표를 고정하지 않고 공급 형식·프레임·실제 Canvas 소비를 대조한다.
// 증거: mydocs/pr/assets/jinho-von-choi-20261114/supplied-ttf-frame/README.md.
// 주석 갱신: 2026-11-15T12:27:15+09:00. 직전 변경 fe6df8a2e5ac430482da7646a6dacad27030e93b
// (Git 커밋 시각 2026-11-09T21:28:12+09:00).
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

test('HFT 원문의 명시 자연 TTF는 Print에서 실제 등록 윤곽을 쓰고 화면·자동 치환은 보존한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/source.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/opensource/NotoSansKR-Regular.ttf'))]],
    ['/variable.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/redistributable/happiness-sans/HappinessSansVF.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'natural-hft-ttf');
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
  const rows = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const bytes = new Uint8Array(await (await fetch('/source.ttf')).arrayBuffer());
    const variable = new Uint8Array(await (await fetch('/variable.ttf')).arrayBuffer());
    const family = 'RHWP Explicit Natural TTF';
    const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = new TextEncoder().encode(`<HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE>
      <FACENAMELIST>${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}">
      <FONT Id="0" Name="RHWP Original HFT" Type="hft"/></FONTFACE>`).join('')}</FACENAMELIST>
      <CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="1100"/></CHARSHAPELIST>
      </MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0"><TEXT CharShape="0">
      <CHAR>가나</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const rows = [];
    for (const printer of [2022, 2024]) for (const profile of ['print', 'screen']) {
      for (const supply of ['static', 'missing', 'variable', 'cleared']) {
        for (const natural of [true, false]) {
          const doc = new wasm.HwpDocument(source);
          try {
            doc.setHangul2024Compat(printer === 2024);
            doc.setFontEnvironment(JSON.stringify({id: 'natural-hft-ttf-control',
              substitutions: {'RHWP Original HFT': family}, trueTypeFaces: [family],
              preserveSourceMetrics: !natural}));
            if (supply !== 'missing') doc.registerTextFontGlyphCoverage(family, supply === 'variable' ? variable : bytes, 0);
            if (supply === 'cleared') doc.clearTextFontFamily(family);
            const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
            const texts = [], fillText = ctx.fillText.bind(ctx), moveTo = ctx.moveTo.bind(ctx);
            let contours = 0;
            ctx.fillText = (text, ...args) => { texts.push(text); return fillText(text, ...args); };
            ctx.moveTo = (...args) => { contours++; return moveTo(...args); };
            doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
            rows.push({printer, profile, supply, natural, texts, contours});
          } finally { doc.free(); }
        }
      }
    }
    return rows;
  });
  for (const row of rows) {
    const outline = row.profile === 'print' && row.supply === 'static' && row.natural;
    assert.equal(row.texts.some(text => /[가나]/.test(text)), !outline, JSON.stringify(row));
    if (outline) assert.ok(row.contours > 0, '실제 선택 TTF 윤곽을 소비해야 한다');
  }
});

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

// 관계형: 실제 SFNT 형식 공급의 Print 프레임을 검사하며 원문 유형·전진·Screen/HQ를 보존한다.
test('supplied static TrueType keeps its Print frame after metric compaction', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/static.ttf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPEquationCJKRegular.ttf'))]],
    ['/cff.otf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPHftOutline.otf'))]],
    ['/variable.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/redistributable/happiness-sans/HappinessSansVF.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const userDataDir = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'supplied-ttf-frame');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(userDataDir, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  const rows = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const programs = {};
    for (const [name, url] of [['static', '/static.ttf'], ['cff', '/cff.otf'], ['variable', '/variable.ttf']]) {
      programs[name] = new Uint8Array(await (await fetch(url)).arrayBuffer());
    }
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = new TextEncoder().encode(`<HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE>
      <FACENAMELIST>${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="휴먼명조" Type="hft"/></FONTFACE>`).join('')}</FACENAMELIST>
      <CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="1100"/></CHARSHAPELIST>
      </MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0"><TEXT CharShape="0"><CHAR>가</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    function collect(value, found) {
      if (Array.isArray(value)) { for (const item of value) collect(item, found); }
      else if (value && typeof value === 'object') {
        if (value.type === 'textRun' && value.text.includes('가')) found.push(value.paintTransform ?? null);
        for (const [key, item] of Object.entries(value)) if (!['fontResources', 'resources', 'data'].includes(key)) collect(item, found);
      }
    }
    const rows = [];
    for (const printer of [2022, 2024]) for (const supply of ['missing', 'static', 'cff', 'variable']) {
      for (const natural of [false, true]) {
        const doc = new wasm.HwpDocument(source); const family = `RHWP Supplied Paint ${supply}`;
        try {
          doc.setHangul2024Compat(printer === 2024);
          doc.setFontEnvironment(JSON.stringify({id: 'supplied-ttf-frame-contract', substitutions: {'휴먼명조': family},
            trueTypeFaces: [family], preserveSourceMetrics: !natural}));
          let supplied = 0;
          if (supply !== 'missing') {
            const bytes = programs[supply]; const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
            doc.registerTextFontGlyphCoverage(family, bytes, 0);
            doc.prepareNativeOtfFonts(raw => {
              const request = JSON.parse(raw);
              if (!request.family.split(',').some(name => name.trim() === family)) return null;
              supplied++; return bytes;
            });
          }
          for (const profile of ['print', 'screen', 'high-quality']) {
            const frames = []; collect(JSON.parse(doc.getPageLayerTreeWithProfile(0, profile)), frames);
            const canvas = document.createElement('canvas'), context = canvas.getContext('2d'), observed = [];
            const transform = context.transform.bind(context);
            context.transform = (...matrix) => { observed.push(matrix); return transform(...matrix); };
            doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
            rows.push({printer, supply, natural, profile, supplied, frames, observed});
          }
        } finally { doc.free(); }
      }
    }
    return rows;
  });
  assert.equal(rows.length, 48);
  for (const row of rows) {
    assert.ok(row.frames.length > 0, '실제 본문이 paint까지 발행되어야 한다');
    assert.equal(row.supplied > 0, row.supply !== 'missing', '공급자 호출만이 아니라 실제 바이트 전달을 확인한다');
    const expected = row.supply === 'static' && !row.natural && row.profile === 'print';
    assert.ok(row.frames.every(frame => Boolean(frame) === expected), JSON.stringify(row));
    if (expected) for (const frame of row.frames) {
      const matrix = ['a', 'b', 'c', 'd', 'e', 'f'].map(key => frame[key]);
      assert.ok(row.observed.some(actual => actual.every((value, i) => Math.abs(value - matrix[i]) <= 1e-6)),
        'JSON 행렬의 소수 6자리 직렬화 정밀도 안에서 실제 Canvas가 같은 paint 프레임을 소비한다');
    }
  }
});
