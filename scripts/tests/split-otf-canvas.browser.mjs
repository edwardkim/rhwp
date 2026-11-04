// 실제 Chrome의 동일 family 분할 FontFace 선택 계약이다. 합성 글꼴만 사용한다.
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
const original = readFileSync(join(root, 'tests/fixtures/fonts/RHWPHftOutline.otf'));
function table(bytes, name) {
  for (let i = 0; i < bytes.readUInt16BE(4); i++) {
    const record = 12 + i * 16;
    if (bytes.toString('ascii', record, record + 4) === name) return bytes.readUInt32BE(record + 8);
  }
  throw Error(`누락된 SFNT 표: ${name}`);
}
function remap(character, width) {
  const bytes = Buffer.from(original), cmap = table(bytes, 'cmap');
  const sub = cmap + bytes.readUInt32BE(cmap + 8);
  assert.equal(bytes.readUInt16BE(sub), 4);
  for (const pos of [sub + 14, sub + 20]) bytes.writeUInt16BE(character.charCodeAt(0), pos);
  bytes.writeUInt16BE((1 - character.charCodeAt(0)) & 0xffff, sub + 24);
  bytes.writeUInt16BE(width, table(bytes, 'hmtx'));
  return bytes;
}

test('Chrome은 역순 등록된 CFF의 원래 후보 순서와 글리프별 fallback을 유지한다', async t => {
  const files = new Map([
    ['/first.otf', ['application/octet-stream', remap('가', 900)]],
    ['/overlap.otf', ['application/octet-stream', remap('가', 400)]],
    ['/hangul.otf', ['application/octet-stream', remap('나', 700)]],
    ['/fallback.ttf', ['application/octet-stream', readFileSync(join(root, 'tests/fixtures/fonts/RHWPHostFixture-Regular.ttf'))]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'split-otf');
  let browser;
  t.after(async () => { if (browser) await browser.close(); rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100}); });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async () => {
    const programs = await Promise.all(['/first.otf', '/overlap.otf', '/hangul.otf', '/fallback.ttf'].map(async path => (await fetch(path)).arrayBuffer()));
    for (const [family, bytes] of [['ReferenceFirst', programs[0]], ['ReferenceOverlap', programs[1]], ['ReferenceHangul', programs[2]],
      ['ReferenceTtf', programs[3]], ['Split', programs[2]], ['Split', programs[1]], ['Split', programs[0]],
      ['Mixed', programs[3]], ['Mixed', programs[0]]]) {
      const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
    }
    await document.fonts.ready;
    const canvas = document.createElement('canvas'); const ctx = canvas.getContext('2d');
    const measure = (family, text) => { ctx.font = `40px ${family}`; return ctx.measureText(text).width; };
    return {first: measure('ReferenceFirst', '가'), overlap: measure('ReferenceOverlap', '가'),
      hangul: measure('ReferenceHangul', '나'), splitFirst: measure('Split', '가'), splitHangul: measure('Split', '나'),
      ttf: measure('ReferenceTtf', 'A'), mixedOtf: measure('Mixed', '가'), mixedTtf: measure('Mixed', 'A'),
      aliasFallback: measure('ReferenceFirst, ReferenceHangul', '나')};
  });
  assert.notEqual(result.first, result.overlap, '겹치는 글리프의 다른 폭으로 순서를 관측한다');
  assert.equal(result.splitFirst, result.first, JSON.stringify(result));
  assert.equal(result.splitHangul, result.hangul, JSON.stringify(result));
  assert.equal(result.mixedOtf, result.first, JSON.stringify(result));
  assert.equal(result.mixedTtf, result.ttf, JSON.stringify(result));
  assert.equal(result.aliasFallback, result.hangul, JSON.stringify(result));
});

test('fresh WASM은 단일 공급과 분할 OTF 배열을 모두 받아 누락 glyph의 전진을 갱신한다', async t => {
  const pkg = resolve(process.env.RHWP_SWEEP_TEST_WASM_PKG || join(root, 'pkg'));
  const files = new Map([
    ['/rhwp.js', ['application/javascript', readFileSync(join(pkg, 'rhwp.js'))]],
    ['/rhwp_bg.wasm', ['application/wasm', readFileSync(join(pkg, 'rhwp_bg.wasm'))]],
    ['/first.otf', ['application/octet-stream', remap('가', 900)]],
    ['/second.otf', ['application/octet-stream', remap('나', 700)]],
  ]);
  const server = createSweepServer(files, new Map());
  await new Promise((accept, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', accept); });
  t.after(async () => { server.closeAllConnections(); await new Promise(accept => server.close(accept)); });
  const profile = createSweepBrowserProfile(join(root, 'output/pr-review/browser-profile-tests'), 'split-otf-wasm');
  let browser;
  t.after(async () => { if (browser) await browser.close(); rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100}); });
  browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true, userDataDir: profile});
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async () => {
    const wasm = await import('/rhwp.js'); await wasm.default();
    const first = new Uint8Array(await (await fetch('/first.otf')).arrayBuffer());
    const second = new Uint8Array(await (await fetch('/second.otf')).arrayBuffer());
    const languages = ['Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User'];
    const source = new TextEncoder().encode(`<?xml version="1.0" encoding="UTF-8"?>
      <HWPML Version="2.91"><HEAD SecCnt="1"><MAPPINGTABLE><FACENAMELIST>
      ${languages.map(lang => `<FONTFACE Count="1" Lang="${lang}"><FONT Id="0" Name="RHWP HFT Outline Fixture" Type="hft"/></FONTFACE>`).join('')}
      </FACENAMELIST><CHARSHAPELIST Count="1"><CHARSHAPE Id="0" Height="2400"/></CHARSHAPELIST>
      </MAPPINGTABLE></HEAD><BODY><SECTION Id="0"><P ParaShape="0"><TEXT CharShape="0"><CHAR>가나</CHAR></TEXT></P></SECTION></BODY></HWPML>`);
    const doc = new wasm.HwpDocument(source);
    try {
      const requests = JSON.parse(doc.getNativeOtfFontRequests());
      const singleChanged = doc.prepareNativeOtfFonts(() => first);
      const single = doc.getPageRenderTree(0);
      const multiChanged = doc.prepareNativeOtfFonts(() => [first, second]);
      const multi = doc.getPageRenderTree(0);
      const outlineFirst = doc.registerHftOutlineFont('RHWP HFT Outline Fixture', first, 0);
      doc.clearTextFontFamily('RHWP HFT Outline Fixture');
      const outlineAfterClear = doc.registerHftOutlineFont('RHWP HFT Outline Fixture', first, 0);
      return {requests: requests.length, singleChanged, multiChanged, changedTree: single !== multi,
        outlineFirst, outlineAfterClear};
    } finally { doc.free(); }
  });
  assert.ok(result.requests > 0, JSON.stringify(result));
  assert.equal(result.singleChanged, true, JSON.stringify(result));
  assert.equal(result.multiChanged, true, JSON.stringify(result));
  assert.equal(result.changedTree, true, '두 번째 프로그램의 글리프 전진이 실제 조판에 적용돼야 한다');
  assert.equal(result.outlineFirst, true, JSON.stringify(result));
  assert.equal(result.outlineAfterClear, true, '이전 세대의 윤곽 공급을 지운 뒤 다시 등록한다');
});
