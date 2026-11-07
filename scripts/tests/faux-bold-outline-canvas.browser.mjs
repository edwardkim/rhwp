// 관계형·고정형: 등록·미등록 경로가 같은 profile별 stroke를 소비한다. Print 장치 1px·Screen SVG 폭.
// 주석 갱신: 2026-11-08T08:55:00.690538+09:00. 직전 파일 변경: 62dd9174efa7518e2a422be2d097ad9ae936fad6 2026-11-08T08:53:25+09:00.
// 합성 빈 문서(휴먼명조 굵게 '국민권익' + 보통 '가나', 24pt)의 실제 WASM·FontFace·Canvas paint
// 계약이다. 한컴 문서의 시각 출력을 검증하지 않으므로 다른 원본의 raw 일치율을 이 검사의 측정값으로
// 적지 않는다. 같은 변경의 30396 권고문 25쪽 Native/fresh WASM TSV와 대표 PNG는
// mydocs/pr/assets/issue_8430/README.md 에 있다.
// 실행: RHWP_SWEEP_TEST_WASM_PKG=<같은 source의 fresh pkg> node --test scripts/tests/faux-bold-outline-canvas.browser.mjs
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

test('합성 굵기는 등록한 실제 face의 윤곽 fill+stroke로 그리고 미등록 face는 기존 Canvas 경로를 쓴다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    // 저장소의 OFL 글꼴. 시스템 글꼴 설치에 의존하지 않는다.
    ['/batang.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/opensource/RHWPPublicBatang-Regular.ttf'))]],
    // 재배포 허가 가변 글꼴: 기본 인스턴스 윤곽이 브라우저 paint와 같다고 확정할 수 없다.
    ['/variable.ttf', ['application/octet-stream', readFileSync(join(root, 'ttfs/redistributable/happiness-sans/HappinessSansVF.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'faux-bold-outline');
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
    const bytes = new Uint8Array(await (await fetch('/batang.ttf')).arrayBuffer());
    const variable = new Uint8Array(await (await fetch('/variable.ttf')).arrayBuffer());
    const build = () => {
      const doc = wasm.HwpDocument.createEmpty();
      doc.createBlankDocument();
      doc.insertText(0, 0, 0, '국민권익 가나');
      // 앞 네 글자만 굵게. 뒤 '가나'는 같은 face의 보통 굵기 대조군이다.
      // 굵은 face가 없는 휴먼명조(metric DB의 bold_fallback)를 모든 글자에 지정한다.
      const fontId = doc.findOrCreateFontId('휴먼명조');
      doc.applyCharFormat(0, 0, 0, 4, JSON.stringify({fontSize: 2400, bold: true, fontId}));
      doc.applyCharFormat(0, 0, 5, 7, JSON.stringify({fontSize: 2400, fontId}));
      return doc;
    };
    const probe = build();
    const svg = probe.renderPageSvgWithProfile(0, 'print');
    const family = /font-family="([^"]+)"/.exec(svg)?.[1]?.split(',')[0].replace(/&apos;|'/g, '').trim();
    const fauxStroke = Number(/stroke-width="([\d.]+)"[^>]*>국/.exec(svg)?.[1] ?? NaN);
    probe.free();
    if (!family) throw Error('빈 문서의 본문 face를 SVG에서 읽지 못했습니다.');
    const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face); await document.fonts.ready;

    const ctxCalls = canvas => {
      const ctx = canvas.getContext('2d', {willReadFrequently: true});
      const calls = [];
      for (const name of ['fillText', 'strokeText', 'fill', 'stroke']) {
        const original = ctx[name].bind(ctx);
        ctx[name] = (...args) => {
          calls.push({name, text: typeof args[0] === 'string' ? args[0] : null, lineWidth: ctx.lineWidth,
            transform: [ctx.getTransform().a, ctx.getTransform().d]});
          return original(...args);
        };
      }
      return {ctx, calls};
    };
    const ink = (canvas, x0, x1) => {
      const data = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
      let sum = 0;
      for (let y = 0; y < canvas.height; y++) for (let x = x0; x < x1; x++) sum += 255 - data[(y * canvas.width + x) * 4];
      return sum / 255;
    };
    const rows = [];
    for (const profile of ['print', 'screen']) {
      for (const registered of [false, true]) {
        const doc = build();
        try {
          if (registered) doc.registerTextFontGlyphCoverage(family, bytes, 0);
          const canvas = document.createElement('canvas');
          const {calls} = ctxCalls(canvas);
          doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
          const bold = calls.filter(call => call.text && /[국민권익]/.test(call.text));
          const plain = calls.filter(call => call.text && /[가나]/.test(call.text));
          const inkLeft = ink(canvas, 0, Math.floor(canvas.width / 2));
          doc.applyCharFormat(0, 0, 0, 4, JSON.stringify({bold: false}));
          const normalCanvas = document.createElement('canvas');
          doc.renderPageToCanvasFilteredWithProfile(0, normalCanvas, 1, 'all', profile);
          rows.push({profile, registered, bold, plain,
            pathFill: calls.filter(call => call.name === 'fill').length,
            pathStroke: calls.filter(call => call.name === 'stroke').map(call => call.lineWidth),
            png: canvas.toDataURL(), inkLeft,
            normalInk: ink(normalCanvas, 0, Math.floor(normalCanvas.width / 2))});
        } finally { doc.free(); }
      }
    }
    // 반례: 가변 face 등록, 등록 뒤 family 해제는 기존 Canvas 글자 paint로 돌아간다.
    const fallback = {};
    for (const [name, prepare] of [
      ['variable', doc => doc.registerTextFontGlyphCoverage(family, variable, 0)],
      ['untracked-primary', doc => doc.registerTextFontGlyphCoverage('Haansoft Batang', bytes, 0)],
      ['cleared', doc => { doc.registerTextFontGlyphCoverage(family, bytes, 0); doc.clearTextFontFamily(family); }],
    ]) {
      const doc = build();
      try {
        prepare(doc);
        const canvas = document.createElement('canvas');
        const {calls} = ctxCalls(canvas);
        doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
        fallback[name] = calls.filter(call => call.text && /[국민권익]/.test(call.text)).map(call => call.name);
      } finally { doc.free(); }
    }
    return {family, fauxStroke, rows, fallback};
  });

  // 선행 조건: 빈 문서 face가 굵은 face 없는 합성 굵기 대상이어야 이 계약이 의미가 있다.
  assert.ok(result.fauxStroke > 0 && result.fauxStroke < 1, `합성 굵기 대상 face가 아님: ${result.family} ${result.fauxStroke}`);
  for (const profile of ['print', 'screen']) {
    const before = result.rows.find(row => row.profile === profile && !row.registered);
    const after = result.rows.find(row => row.profile === profile && row.registered);
    // 미등록은 실제 FontFace의 글자 API를 유지하며 장치 폭은 같은 profile 규칙을 따른다.
    assert.ok(before.bold.some(call => call.name === 'strokeText'), JSON.stringify(before.bold));
    // 등록: 같은 글자를 글자 API로 그리지 않고 윤곽 path fill+stroke로 그린다.
    assert.equal(after.bold.length, 0, `굵은 글자가 아직 글자 API로 그려짐: ${JSON.stringify(after.bold)}`);
    assert.ok(after.pathFill >= 4 && after.pathStroke.length >= 4, JSON.stringify(after));
    // 보통 굵기 대조군은 계속 fillText만 사용한다.
    assert.ok(after.plain.length > 0 && after.plain.every(call => call.name === 'fillText'), JSON.stringify(after.plain));
    const widths = [...new Set(after.pathStroke.map(width => width.toFixed(3)))];
    const fallbackWidths = [...new Set(before.bold.filter(call => call.name === 'strokeText').map(call => call.lineWidth.toFixed(3)))];
    assert.deepEqual(fallbackWidths, widths, `${profile}: 원 프로그램 미확인 경로도 같은 장치 선 굵기를 소비한다`);
    if (profile === 'print') {
      // 인쇄 래스터는 선과 같은 PDFium 최소 한 장치 픽셀 규칙(raster_stroke)을 따른다.
      assert.deepEqual(widths, ['1.000'], JSON.stringify(after.pathStroke));
    } else {
      // 화면 프로필은 SVG·PDF와 같은 0.02em을 그대로 쓴다.
      assert.deepEqual(widths, [result.fauxStroke.toFixed(3)], JSON.stringify(after.pathStroke));
    }
    for (const row of [before, after]) {
      assert.ok(row.inkLeft > row.normalInk,
        `${profile}, 등록=${row.registered}: 같은 원문의 보통 굵기보다 합성 굵기가 진해야 한다`);
    }
  }
  for (const [name, calls] of Object.entries(result.fallback)) {
    assert.ok(calls.includes('strokeText'), `${name}: 기존 Canvas 합성 굵기 경로를 유지해야 한다 ${JSON.stringify(calls)}`);
  }
});
