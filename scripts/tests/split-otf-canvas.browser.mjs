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
// 실제 Bold 프로그램과 보통 프로그램을 같은 윤곽으로 분리해 선택 계약만 관측한다.
function boldProgram(bytes) {
  const bold = Buffer.from(bytes), os2 = table(bold, 'OS/2'), head = table(bold, 'head');
  bold.writeUInt16BE(700, os2 + 4);
  bold.writeUInt16BE((bold.readUInt16BE(os2 + 62) | 0x20) & ~0x40, os2 + 62);
  bold.writeUInt16BE(bold.readUInt16BE(head + 44) | 1, head + 44);
  return bold;
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
    ['/first-bold.otf', ['application/octet-stream', boldProgram(remap('가', 900))]],
    ['/second-bold.otf', ['application/octet-stream', boldProgram(remap('나', 700))]],
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
      doc.setFontEnvironment(JSON.stringify({id: 'split-hft-actual-owner',
        substitutions: {'RHWP HFT Outline Fixture': 'Selected Split'}}));
      for (const bytes of [second, first]) {
        const face = new FontFace('Selected Split', bytes); await face.load(); document.fonts.add(face);
      }
      const canvas = document.createElement('canvas'), ctx = canvas.getContext('2d');
      const calls = [], paths = [];
      const fillText = ctx.fillText.bind(ctx), fill = ctx.fill.bind(ctx);
      ctx.fillText = (text, ...args) => { calls.push(text); return fillText(text, ...args); };
      ctx.fill = (...args) => { paths.push(true); return fill(...args); };
      const paint = () => {
        calls.length = 0; paths.length = 0;
        doc.renderPageToCanvasFilteredWithProfile(0, canvas, 1, 'all', 'print');
        return {calls: [...calls], paths: paths.length};
      };
      const selectedFirst = doc.registerHftOutlineFont('Selected Split', first, 0);
      const singlePaint = paint();
      const candidatesChanged = doc.registerHftOutlineFontCandidates('Selected Split', [first, second]);
      const splitPaint = paint();
      doc.clearTextFontFamily('Selected Split');
      const clearedPaint = paint();
      // 유형 미상의 원본에도 실제 프로그램 신원과 스타일별 선택을 보존한다.
      const untypedSource = new TextEncoder().encode(new TextDecoder().decode(source)
        .replaceAll('Type="hft"', 'Type="unknown"'));
      const untyped = new wasm.HwpDocument(untypedSource);
      let verifiedCff;
      try {
        untyped.applyCharFormat(0, 0, 0, 2, JSON.stringify({bold: true}));
        const digest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', first))]
          .map(byte => byte.toString(16).padStart(2, '0')).join('');
        untyped.setFontEnvironment(JSON.stringify({id: 'verified-cff-synthetic-bold',
          substitutions: {'RHWP HFT Outline Fixture': 'RHWP HFT Outline Fixture'},
          selectedPrograms: {'RHWP HFT Outline Fixture': {query: 'RHWP HFT Outline Fixture', family: 'RHWP HFT Outline Fixture',
            trueType: false, openTypeCff: true, programId: `sweep-${digest}`}}}));
        untyped.prepareNativeOtfFonts(() => [first, second]);
        for (const family of ['RHWP HFT Outline Fixture']) {
          for (const bytes of [second, first]) {
            const face = new FontFace(family, bytes); await face.load(); document.fonts.add(face);
          }
        }
        const tree = untyped.getPageRenderTree(0);
        const registrations = ['RHWP HFT Outline Fixture'].map(
          family => untyped.registerHftOutlineFontCandidates(family, [first, second]));
        const target = document.createElement('canvas'), context = target.getContext('2d');
        const seen = [], fills = [], strokes = [];
        for (const [name, into] of [['fillText', seen], ['fill', fills], ['stroke', strokes]]) {
          const original = context[name].bind(context);
          context[name] = (...args) => { into.push(name === 'fillText' ? args[0] : context.lineWidth); return original(...args); };
        }
        const draw = profile => {
          seen.length = 0; fills.length = 0; strokes.length = 0;
          untyped.renderPageToCanvasFilteredWithProfile(0, target, 1, 'all', profile);
          return {text: seen.join(''), fills: fills.length, strokes: [...strokes]};
        };
        const unchanged = tree === untyped.getPageRenderTree(0);
        untyped.setHangul2024Compat(false);
        const printed = draw('print');
        untyped.setHangul2024Compat(true);
        const printed2024 = draw('print'), screen = draw('screen');
        const altType = JSON.parse(untyped.getFontDecisionTrace(0, '{"maxCharacters":2}')).records[0].document.altType;
        untyped.applyCharFormat(0, 0, 0, 2, JSON.stringify({bold: false}));
        const regular = draw('print');
        // 보통 윤곽 등록이 남아 있어도 스타일별 공급자가 실제 Bold를 고르면 합성하지 않는다.
        const boldPrograms = await Promise.all(['/first-bold.otf', '/second-bold.otf'].map(
          async path => new Uint8Array(await (await fetch(path)).arrayBuffer())));
        for (const family of ['RHWP HFT Outline Fixture']) {
          for (const bytes of [...boldPrograms].reverse()) {
            const face = new FontFace(family, bytes, {weight: '700'});
            await face.load(); document.fonts.add(face);
          }
        }
        untyped.applyCharFormat(0, 0, 0, 2, JSON.stringify({bold: true}));
        const realBoldRequests = JSON.parse(untyped.getNativeOtfFontRequests());
        const realBoldChanged = untyped.prepareNativeOtfFonts(() => boldPrograms);
        const realBold = draw('print');
        verifiedCff = {registrations, printed, printed2024, screen, regular, realBoldRequests, realBoldChanged, realBold, unchanged, altType};
      } finally { untyped.free(); }
      return {requests: requests.length, singleChanged, multiChanged, changedTree: single !== multi,
        outlineFirst, outlineAfterClear, selectedFirst, candidatesChanged, singlePaint, splitPaint, clearedPaint, verifiedCff};
    } finally { doc.free(); }
  });
  assert.ok(result.requests > 0, JSON.stringify(result));
  assert.equal(result.singleChanged, true, JSON.stringify(result));
  assert.equal(result.multiChanged, true, JSON.stringify(result));
  assert.equal(result.changedTree, true, '두 번째 프로그램의 글리프 전진이 실제 조판에 적용돼야 한다');
  assert.equal(result.outlineFirst, true, JSON.stringify(result));
  assert.equal(result.outlineAfterClear, true, '이전 세대의 윤곽 공급을 지운 뒤 다시 등록한다');
  assert.equal(result.selectedFirst, true, '실제 표시 family도 원 HFT 소유를 유지한다');
  assert.equal(result.candidatesChanged, true);
  assert.deepEqual(result.singlePaint.calls, ['나'], '첫 조각에 없는 글자만 Canvas가 그린다');
  assert.deepEqual(result.splitPaint.calls, [], '분할 프로그램 두 글자 모두 실제 윤곽을 그린다');
  assert.equal(result.splitPaint.paths, result.singlePaint.paths + 1);
  assert.deepEqual(result.clearedPaint.calls, ['가', '나'], '세대 교체는 두 조각의 윤곽을 모두 폐기한다');
  assert.deepEqual(result.verifiedCff.registrations, [true], '실제 선택 신원이 같은 family만 등록한다');
  assert.equal(result.verifiedCff.printed.text, '', JSON.stringify(result.verifiedCff));
  assert.equal(result.verifiedCff.printed.fills, 2);
  assert.deepEqual(result.verifiedCff.printed.strokes, [1, 1], '0.02em과 기존 Print 장치 최소 폭을 함께 적용한다');
  assert.deepEqual(result.verifiedCff.printed2024, result.verifiedCff.printed, '합성 굵기 계약은 두 Print 세대에 적용한다');
  assert.equal(result.verifiedCff.screen.text, '가나', '화면의 기존 글꼴 선택을 보존한다');
  assert.equal(result.verifiedCff.regular.text, '가나', '미정 유형의 보통 글자를 HFT로 재분류하지 않는다');
  assert.equal(result.verifiedCff.realBoldChanged, true, JSON.stringify(result.verifiedCff));
  assert.deepEqual(result.verifiedCff.realBold, {text: '가나', fills: 0, strokes: []}, '실제 Bold에 보통 CFF 합성 굵기를 덧씌우지 않는다');
  assert.equal(result.verifiedCff.unchanged, true, 'paint 등록은 조판 트리를 바꾸지 않는다');
  assert.equal(result.verifiedCff.altType, 0, '원본 유형을 보존한다');
});
