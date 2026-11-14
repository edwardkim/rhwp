// 관계형: 실제 FontFace 주 프로그램의 Print 기하·후보 소유·Screen 보존을 검사한다.
// 주석 갱신: 2026-11-15 05:42:03 KST. 공개 OFL TTF·MIT CFF 자원만 공급하며 실물 px 핀·시스템 글꼴을 요구하지 않는다.
// 실물 보정 근거: mydocs/pr/assets/issue_9060/print-primary-glyph-geometry-analysis.md.
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

async function setup(t, key) {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ...[
      ['latin', 'RHWPEquationLatinOnly.ttf'],
      ['light', 'RHWPEquationCJKLight.ttf'],
      ['regular', 'RHWPEquationCJKRegular.ttf'],
      ['cff', 'RHWPHftOutline.otf'],
    ].map(([name, file]) => [`/${name}`, ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts', file))]]),
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), key);
  let browser;
  t.after(async () => {
    if (browser) await browser.close();
    rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
  });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default(); globalThis.geometryWasm = wasm;
    globalThis.geometryFont = async (family, name) => {
      const bytes = new Uint8Array(await (await fetch(`/${name}`)).arrayBuffer());
      const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
      return bytes;
    };
    globalThis.geometryDocument = (family, type, text) => {
      const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
      return new wasm.HwpDocument(new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
        <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
        ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="${family}" Type="${type}"/></FONTFACE>`).join('')}
        </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/>
        </CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0">
        <TEXT CharShape="0"><CHAR>${text}</CHAR></TEXT></P></SECTION></BODY></HWPML>`));
    };
    globalThis.geometryPaint = (doc, profile, compat = true) => {
      doc.setHangul2024Compat(compat);
      const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
      const texts = []; let paths = 0;
      const fillText = ctx.fillText.bind(ctx), fill = ctx.fill.bind(ctx);
      ctx.fillText = (text, ...args) => { texts.push(text); return fillText(text, ...args); };
      ctx.fill = (...args) => { paths++; return fill(...args); };
      doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', profile);
      const rgba = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
      let ink = 0;
      for (let i = 0; i < rgba.length; i += 4) if (rgba[i + 3] && Math.min(rgba[i], rgba[i + 1], rgba[i + 2]) < 200) ink++;
      return {texts, paths, ink, png: canvas.toDataURL(), tree: doc.getPageRenderTree(0)};
    };
  });
  return page;
}

test('확인된 주 TTF는 원본 선언과 관계없이 Print에서만 같은 프로그램의 윤곽을 쓴다', async t => {
  const page = await setup(t, 'print-primary-normal');
  const rows = await page.evaluate(async () => {
    const family = 'RHWP Primary Geometry';
    const bytes = await geometryFont(family, 'latin');
    await geometryFont(`__rhwp_ttf__${family}`, 'latin');
    const rows = [];
    for (const type of ['ttf', 'hft', 'unknown']) {
      const doc = geometryDocument(family, type, 'H');
      try {
        for (const supplied of [false, true]) {
          if (supplied) for (const name of [family, `__rhwp_ttf__${family}`]) doc.registerPrintFontPrimary?.(name, bytes, 0, true);
          for (const [profile, compat] of [['screen', true], ['print', true], ['print', false], ['high-quality', true]]) {
            rows.push({type, supplied, profile, compat, ...geometryPaint(doc, profile, compat)});
          }
        }
        for (const name of [family, `__rhwp_ttf__${family}`]) doc.clearPrintFontPrimary?.(name);
        rows.push({type, supplied: false, profile: 'print-after-clear', compat: true, ...geometryPaint(doc, 'print')});
      } finally { doc.free(); }
    }
    return rows;
  });
  for (const type of ['ttf', 'hft', 'unknown']) {
    const selected = rows.filter(row => row.type === type), trees = new Map();
    for (const row of selected) {
      const tree = trees.get(row.compat) ?? row.tree; trees.set(row.compat, tree);
      assert.equal(row.tree, tree, '실제 paint 공급은 charX·조판·내용 소속을 바꾸지 않는다');
      assert.ok(row.ink > 0, '빈 출력은 기하 통과가 아니다');
      if (row.supplied && row.profile === 'print') {
        assert.ok(!row.texts.includes('H'), `${type}/${row.compat}: 확인된 주 프로그램의 실제 path 재생`);
      } else {
        assert.ok(row.texts.includes('H'), '미공급·Screen·HighQuality·해제 뒤는 기존 painter를 유지한다');
      }
    }
    const before = selected.find(row => !row.supplied && row.profile === 'screen');
    const after = selected.find(row => row.supplied && row.profile === 'screen');
    assert.equal(after.png, before.png, 'Screen의 실제 glyph 잉크를 바꾸지 않는다');
  }
});

test('주 TTF가 결측이어도 같은 family의 미확인 동반 프로그램을 건너뛰지 않는다', async t => {
  const page = await setup(t, 'print-primary-incomplete');
  const rows = await page.evaluate(async () => {
    const family = 'RHWP Primary Companion', fallback = 'RHWP Other Fallback';
    await geometryFont(family, 'light');
    const primary = await geometryFont(family, 'latin');
    const other = await geometryFont(fallback, 'regular');
    const doc = geometryDocument(`${family},${fallback}`, 'hft', '가');
    try {
      const before = geometryPaint(doc, 'print');
      doc.registerPrintFontPrimary?.(family, primary, 0, false);
      doc.registerPrintFontPrimary?.(fallback, other, 0, true);
      return [before, geometryPaint(doc, 'print')];
    } finally { doc.free(); }
  });
  assert.ok(rows[1].texts.includes('가'), '원 family의 동반 glyph는 실제 Canvas가 고른다');
  assert.equal(rows[1].png, rows[0].png, '뒤 family의 다른 모양으로 원 glyph를 덮지 않는다');
  assert.equal(rows[1].tree, rows[0].tree);
});

test('완전한 CFF의 결측만 확인된 다음 TTF의 Print 윤곽으로 이어간다', async t => {
  const page = await setup(t, 'print-primary-cff-missing');
  const rows = await page.evaluate(async () => {
    const family = '한양신명조', fallback = 'RHWP Known Print Fallback';
    const cff = await geometryFont(family, 'cff'), ttf = await geometryFont(fallback, 'latin');
    const doc = geometryDocument(family, 'hft', 'H');
    try {
      doc.setFontEnvironment(JSON.stringify({id: 'print-primary-contract', substitutions: {}, glyphFallbacks: {[family]: [fallback]}}));
      const registered = doc.registerHftOutlineFont(family, cff, 0);
      const rows = [];
      for (const complete of [false, true]) {
        doc.registerPrintFontPrimary?.(family, cff, 0, complete);
        doc.registerPrintFontPrimary?.(fallback, ttf, 0, true);
        rows.push({complete, registered, ...geometryPaint(doc, 'print')});
      }
      return rows;
    } finally { doc.free(); }
  });
  assert.ok(rows.every(row => row.registered), 'CFF source 등록의 실제 경로를 확인한다');
  assert.ok(rows[0].texts.includes('H'), '불완전한 앞 family를 건너뛰지 않는다');
  assert.ok(!rows[1].texts.includes('H'), '모든 CFF의 결측이 확인됐을 때만 다음 실제 프로그램을 재생한다');
  assert.ok(rows[1].paths > rows[0].paths && rows[1].ink > 0);
  assert.equal(rows[1].tree, rows[0].tree);
});
