import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {createSweepServer, registerCanvasFont} from '../export-wasm-png-for-sweep.mjs';
import {findChrome} from '../rasterize-svg-webfonts.mjs';

test('공급 face의 원본 advance를 이름·공백·한국어 별칭으로 사용한다', async () => {
  const require = createRequire(new URL('../../rhwp-studio/package.json', import.meta.url));
  const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf', import.meta.url));
  const server = createSweepServer(new Map([
    ['/supply.js', ['application/javascript', 'export ' + registerCanvasFont.toString()]],
    ['/fixture.ttf', ['application/octet-stream', font]],
  ]), new Map());
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await require('puppeteer-core').launch({executablePath: findChrome(process.env.VISUAL_SWEEP_CHROME), headless: true});
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    const result = await page.evaluate(async () => {
      const {registerCanvasFont} = await import('/supply.js');
      const bytes = await (await fetch('/fixture.ttf')).arrayBuffer();
      const names = ['RHWP Supply Probe', '공급글꼴검사'];
      for (const family of names) await registerCanvasFont(family, bytes, 400, 'normal');
      await document.fonts.ready;
      const ctx = document.getElementById('page').getContext('2d');
      return names.map(family => {
        ctx.font = `20px "${family}"`;
        return {family, ratio: ctx.measureText('A').width / 20};
      });
    });
    // fixture의 hmtx advance와 head unitsPerEm에서 정한 독립적인 글꼴 계약이다.
    for (const item of result) assert.ok(Math.abs(item.ratio - 0.9) < 0.001, `${item.family}: ${item.ratio}`);
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
});
