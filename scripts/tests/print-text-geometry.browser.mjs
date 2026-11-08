// 관계형: 실제 WASM의 glyph paint와 Screen/Print 메트릭 전환을 관측한다.
// 주석 갱신: 2026-11-04 15:18 KST. 합성 공급자·프로필 계약이다.
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

// 관계형: 미등록 앞 face의 결측을 추정하여 뒤 face의 윤곽으로 덮어쓰지 않는다.
// 주석 갱신: 2026-11-09 02:01 KST. MIT 합성 글꼴의 실제 WASM·Canvas 공급 소유 검사다.
// 실물 문서 좌표 핀이나 한컴 글꼴 설치에 의존하지 않으며 독립 PDF 일치율은 비해당이다.
test('Print TTF는 앞 후보의 coverage가 미확인일 때 실제 Canvas 글리프를 보존한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ...[
      ['primary', 'RHWPEquationLatinOnly.ttf'],
      ['untracked', 'RHWPEquationCJKLight.ttf'],
      ['fallback', 'RHWPEquationCJKRegular.ttf'],
    ].map(([key, name]) => [`/${key}.ttf`, ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts', name))]]),
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'print-untracked-coverage');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  const rows = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const programs = [];
    for (const key of ['primary', 'untracked', 'fallback']) {
      const family = `RHWP Coverage ${key}`;
      const bytes = new Uint8Array(await (await fetch(`/${key}.ttf`)).arrayBuffer());
      const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
      programs.push({key, family, bytes});
    }
    await document.fonts.ready;
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const family = programs.map(p => p.family).join(',');
    const source = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="${family}" Type="ttf"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/>
      </CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
      <TEXT CharShape="0"><CHAR>H가</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const rows = [];
    for (const supplied of ['none', 'partial', 'complete']) {
      const doc = new wasm.HwpDocument(source);
      try {
        doc.setHangul2024Compat(true);
        for (const p of programs) {
          if (supplied === 'complete' || (supplied === 'partial' && p.key !== 'untracked')) {
            doc.registerTextFontGlyphCoverage(p.family, p.bytes, 0);
          }
        }
        for (const outputProfile of ['screen', 'print']) {
          const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
          const calls = [], paths = [];
          const fillText = ctx.fillText.bind(ctx), fill = ctx.fill.bind(ctx);
          ctx.fillText = (text, ...args) => { calls.push(text); return fillText(text, ...args); };
          ctx.fill = (...args) => { paths.push(true); return fill(...args); };
          doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', outputProfile);
          rows.push({supplied, profile: outputProfile, calls, paths: paths.length, png: canvas.toDataURL(), tree: doc.getPageRenderTree(0)});
        }
      } finally { doc.free(); }
    }
    return rows;
  });
  for (const profile of ['screen', 'print']) {
    const reference = rows.find(r => r.supplied === 'none' && r.profile === profile);
    const partial = rows.find(r => r.supplied === 'partial' && r.profile === profile);
    assert.ok(reference.calls.includes('가'), '미공급 대조군은 실제 Canvas 글자를 그린다');
    assert.ok(partial.calls.includes('가'), '미확인 앞 후보가 있으면 뒤 후보 윤곽으로 덮어쓰지 않는다');
    assert.equal(partial.png, reference.png, 'coverage가 불완전해도 실제 선택한 글자의 잉크를 보존한다');
  }
  const complete = rows.find(r => r.supplied === 'complete' && r.profile === 'print');
  assert.ok(!complete.calls.includes('가'), '모든 앞 후보를 확인했으면 실제 공급자의 윤곽 재생을 유지한다');
  assert.ok(complete.paths > 0 && complete.calls.includes('H'), '윤곽 재생과 원 face의 문자 paint를 함께 확인한다');
  for (const row of rows) assert.equal(row.tree, rows[0].tree, 'coverage 공급은 조판·내용 소속을 바꾸지 않는다');
});

// 관계형: 기본 TTF가 공급하지 못한 글자만 검증된 다음 프로그램의 윤곽을 쓴다.
// 주석 갱신: 2026-11-04 16:26 KST. 좌표 핀이 아닌 합성 공급자·프로필 계약이다.
test('Print TTF 대체 글리프는 원 프로그램의 기하를 쓰고 Screen과 원 face는 보존한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/primary.ttf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPEquationLatinOnly.ttf'))]],
    // 기존 소유 fixture의 '가' 윤곽에 U+0628 cmap만 추가한 문맥 판정용 합성 face다.
    ['/fallback.ttf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/issue8533/RHWPPrintFallbackContext.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'ttf-fallback');
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  const rows = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const programs = [];
    for (const [family, url] of [['RHWP Print Primary', '/primary.ttf'], ['RHWP Print Fallback', '/fallback.ttf']]) {
      const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
      const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
      programs.push({family, bytes});
    }
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="RHWP Print Primary,RHWP Print Fallback" Type="ttf"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/>
      </CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
      <TEXT CharShape="0"><CHAR>H가ب</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const doc = new wasm.HwpDocument(source);
    try {
      doc.setHangul2024Compat(true);
      const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
      const calls = [], paths = [];
      const fillText = ctx.fillText.bind(ctx), fill = ctx.fill.bind(ctx);
      ctx.fillText = (text, ...args) => { calls.push(text); return fillText(text, ...args); };
      ctx.fill = (...args) => { paths.push(true); return fill(...args); };
      const rows = [];
      for (const supplied of [false, true]) {
        if (supplied) for (const {family, bytes} of programs) doc.registerTextFontGlyphCoverage(family, bytes, 0);
        for (const [profile, compat] of [['screen', true], ['print', true], ['print', false], ['high-quality', true]]) {
          doc.setHangul2024Compat(compat);
          calls.length = 0; paths.length = 0;
          doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
          rows.push({supplied, profile, compat, calls: [...calls], paths: paths.length, tree: doc.getPageRenderTree(0)});
        }
      }
      return rows;
    } finally { doc.free(); }
  });
  const printed = rows.find(row => row.supplied && row.profile === 'print' && row.compat);
  assert.ok(printed.calls.includes('H'), '원 face가 공급한 글자는 원 painter에 남는다');
  assert.ok(!printed.calls.includes('가'), '검증된 대체 TTF의 윤곽을 기하적으로 재생한다');
  assert.ok(printed.calls.includes('ب'), '단독 글자여도 문맥 shaping이 필요한 문자는 원 painter에 남긴다');
  const trees = new Map();
  for (const row of rows) {
    const tree = trees.get(row.compat) ?? row.tree;
    trees.set(row.compat, tree);
    assert.equal(row.tree, tree, '공급과 출력 프로필이 조판·소속을 바꾸지 않는다');
  }
  for (const row of rows.filter(row => row !== printed)) {
    assert.ok(row.calls.includes('가'), 'Screen 또는 미공급 프로그램에는 기존 painter를 쓴다');
  }
  assert.ok(printed.paths > rows[0].paths, '대체 글리프의 실제 path 재생이 있어야 한다');
});

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
      const originalRect = ctx.rect.bind(ctx); const rectangles = [];
      ctx.rect = (...args) => {
        const m = ctx.getTransform();
        rectangles.push({args, transform: [m.a, m.b, m.c, m.d, m.e, m.f]});
        return originalRect(...args);
      };
      ctx.fillText = (...args) => {
        const m = ctx.getTransform();
        calls.push({text: args[0], mode: ctx.textRendering, font: ctx.font,
          transform: [m.a, m.b, m.c, m.d, m.e, m.f]});
        return original(...args);
      };
      const rows = [];
      for (const [compat, profile] of [[true, 'screen'], [true, 'print'], [false, 'print'], [true, 'print'], [true, 'screen']]) {
        doc.setHangul2024Compat(compat);
        calls.length = 0;
        rectangles.length = 0;
        doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
        const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        let ink = 0;
        for (let i = 0; i < rgba.length; i += 4) {
          if (rgba[i + 3] > 0 && Math.min(rgba[i], rgba[i + 1], rgba[i + 2]) < 232) ink++;
        }
        rows.push({compat, profile, calls: [...calls], rectangles: [...rectangles], ink,
          layer: JSON.parse(doc.getPageLayerTreeWithProfile(0, profile, true, true)),
          tree: doc.getPageRenderTree(0)});
      }
      // 실제 WASM 경계에서도 화면 폭이 인쇄에 유출되면 거부해야 한다.
      // 공급자 폭은 합성 입력값이며 실물 문서의 좌표 정답이 아니다.
      const measured = new wasm.HwpDocument(new TextEncoder().encode(
        new TextDecoder().decode(bytes).replace('<CHAR>H</CHAR>', '<CHAR>😀A</CHAR>')));
      let metricProbe;
      try {
        const batch = JSON.parse(measured.collectCanvasMetricRequests(1, 1));
        if (!batch.requests.length) throw Error('보충 측정 반례가 수집되지 않았습니다.');
        const measureContext = document.createElement('canvas').getContext('2d');
        measured.registerCanvasMetricReplies(1, 1, batch.revision, JSON.stringify(batch.requests.map(r => {
          measureContext.font = r.font;
          return {...r, resolvedFont: measureContext.font, measuredAdvancePx: 31.25};
        })));
        measured.selectCanvasMetrics(true);
        const canvas = document.createElement('canvas');
        measured.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'screen');
        let printError = null;
        try { measured.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print'); }
        catch (error) { printError = String(error); }
        measured.selectCanvasMetrics(false);
        measured.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
        const portablePageCount = measured.pageCount();
        measured.selectCanvasMetrics(true);
        measured.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'screen');
        metricProbe = {printError, portablePageCount, restored: measured.canvasMetricsActive()};
      } finally { measured.free(); }
      return {rows, metricProbe};
    } finally { doc.free(); }
  });
  const firstByCompat = new Map();
  assert.match(result.metricProbe.printError ?? '', /select portable metrics before exporting/);
  assert.equal(result.metricProbe.portablePageCount, 1);
  assert.equal(result.metricProbe.restored, true);
  for (const row of result.rows) {
    const first = firstByCompat.get(row.compat) || row;
    firstByCompat.set(row.compat, first);
    assert.ok(row.ink > 0 && row.calls.some(call => call.text === 'H'), JSON.stringify(row));
    assert.equal(row.tree, first.tree, '프로필 전환으로 원본 배치·소속을 바꾸지 않는다');
    assert.deepEqual(row.calls.map(call => call.transform), first.calls.map(call => call.transform),
      '글리프 원점·장평을 모드 보정 수치로 바꾸지 않는다');
    const expected = row.profile === 'print' && row.compat ? 'geometricPrecision' : 'auto';
    assert.equal(row.layer.rasterClipCoverage, row.profile === 'print' && row.compat);
    const clips = [];
    const visit = node => {
      if (node.kind === 'clipRect' && node.clipKind === 'body') clips.push(node.clip);
      if (node.child) visit(node.child);
      for (const child of node.children || []) visit(child);
    };
    visit(row.layer.root);
    assert.ok(clips.length > 0, '본문 clip이 실제 재생 경로에 있어야 한다');
    for (const clip of clips) {
      const expectedRect = row.layer.rasterClipCoverage
        ? [Math.floor(clip.x), Math.floor(clip.y), Math.ceil(clip.x + clip.width) - Math.floor(clip.x), Math.ceil(clip.y + clip.height) - Math.floor(clip.y)]
        : [clip.x, clip.y, clip.width, clip.height];
      assert.ok(row.rectangles.some(rect => rect.transform.every((v, i) => v === [1, 0, 0, 1, 0, 0][i])
        && rect.args.every((v, i) => Math.abs(v - expectedRect[i]) < 0.002)),
      `본문 clip 재생이 세대 정책을 소비해야 한다: ${JSON.stringify({compat: row.compat, expectedRect, rectangles: row.rectangles})}`);
    }
    assert.ok(row.calls.every(call => call.mode === expected),
      `실제 glyph paint의 ${row.profile} 모드: ${JSON.stringify(row.calls)}`);
  }
});
