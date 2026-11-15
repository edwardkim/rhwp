// 관계형 CSS/FontFace 공급 계약. 한컴 문서의 조판 좌표나 외형을 고정하지 않는다.
// 2026-11-15 KST: 실제 Studio setter의 측정·paint를 원시 Canvas 대조군과 비교한다.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runTest, loadApp, screenshot } from './helpers.mjs';

const programs = Object.fromEntries(['Regular', 'Italic'].map(style => [style,
  readFileSync(new URL(`../../tests/fixtures/fonts/RHWPHostFixture-${style}.ttf`, import.meta.url)).toString('base64'),
]));

await runTest('#9077 Canvas 예약 별칭의 실제 face 선택', async ({ page }) => {
  await loadApp(page, '?renderer=canvas2d');
  const result = await page.evaluate(async programs => {
    const { setRawCanvasFont } = await import('/src/core/canvas-font-raw.ts');
    const { registerLoadedLocalFontFace, clearLoadedLocalFontFaces } = await import('/src/core/font-loader.ts');
    const { parseCssFontFamilyList } = await import('/src/core/font-substitution.ts');
    const decode = style => Uint8Array.from(atob(programs[style]), c => c.charCodeAt(0));
    const supplied = [];
    async function supply(family, style = 'Regular', local = false) {
      const face = new FontFace(family, decode(style));
      await face.load();
      if (local) registerLoadedLocalFontFace(face);
      else document.fonts.add(face);
      supplied.push(face);
    }
    function paint(font, raw = false) {
      const canvas = document.createElement('canvas');
      canvas.width = 360; canvas.height = 110;
      const context = canvas.getContext('2d');
      context.fillStyle = 'white'; context.fillRect(0, 0, canvas.width, canvas.height);
      if (raw) setRawCanvasFont(context, font);
      else context.font = font;
      context.fillStyle = 'black';
      const width = context.measureText('A가').width;
      context.fillText('A가', 12, 80);
      return { font: context.font, width, pixels: canvas.toDataURL() };
    }
    const checks = [];
    try {
      // 라이선스가 확인된 작은 테스트 프로그램을 공급한다. 실제 궁서/Arial의 외형 주장이 아니다.
      for (const [family, local] of [['함초롬바탕', false], ['RHWP 9077 Exact Face', true]]) {
        for (const face of document.fonts) if (face.family.replaceAll('"', '') === family) document.fonts.delete(face);
        await supply(family, 'Regular', local);
        const control = paint(`64px "${family}"`, true);
        const fallback = paint('64px serif', true);
        for (const aliases of [[], [`__rhwp_ttf__${family}`], [`__rhwp_intrinsic__${family}`],
          [`__rhwp_intrinsic__${family}`, `__rhwp_ttf__${family}`]]) {
          const font = '64px ' + [...aliases, family, 'Batang', 'Malgun Gothic', 'serif']
            .map(name => `"${name}"`).join(', ');
          const actual = paint(font);
          const names = parseCssFontFamilyList(actual.font.split('px ')[1]);
          checks.push({ family, aliases, font: actual.font, width: actual.width,
            expectedWidth: control.width, samePixels: actual.pixels === control.pixels,
            distinctFallback: control.pixels !== fallback.pixels, leading: names.slice(0, aliases.length + 1) });
        }
        // 공급된 예약 별칭은 일반 face보다 먼저 실제 별도 프로그램을 선택한다.
        for (const prefix of ['__rhwp_ttf__', '__rhwp_intrinsic__']) {
          const alias = `${prefix}${family}`;
          await supply(alias, 'Italic');
          const actual = paint(`64px "${alias}", "${family}", serif`);
          const expected = paint(`64px "${alias}"`, true);
          checks.push({ family, alias, prepared: true, samePixels: actual.pixels === expected.pixels,
            distinctOriginal: actual.pixels !== control.pixels, width: actual.width, expectedWidth: expected.width });
        }
      }
      const sheet = document.createElement('canvas'); sheet.width = 360; sheet.height = 110;
      const context = sheet.getContext('2d');
      context.fillStyle = 'white'; context.fillRect(0, 0, 360, 110);
      context.font = '64px "__rhwp_ttf__RHWP 9077 Exact Face", "RHWP 9077 Exact Face", serif';
      context.fillStyle = 'black'; context.fillText('A가', 12, 80);
      sheet.id = 'issue9077-preview';
      sheet.style.cssText = 'position:fixed;bottom:0;right:0;z-index:2147483647;border:1px solid black';
      document.body.append(sheet);
      return { checks, backend: window.__renderBackend, browser: navigator.userAgent };
    } finally {
      clearLoadedLocalFontFaces();
      for (const face of supplied) document.fonts.delete(face);
    }
  }, programs);
  assert.equal(result.backend, 'canvas2d');
  for (const check of result.checks) {
    assert.equal(check.samePixels, true, JSON.stringify(check));
    assert.equal(check.width, check.expectedWidth, JSON.stringify(check));
    if (check.prepared) assert.equal(check.distinctOriginal, true, JSON.stringify(check));
    else {
      assert.equal(check.distinctFallback, true, JSON.stringify(check));
      assert.deepEqual(check.leading, [...check.aliases, check.family]);
    }
  }
  console.log(JSON.stringify(result, null, 2));
  await screenshot(page, 'issue9077-canvas-font-alias');
}, { skipLoadApp: true });
