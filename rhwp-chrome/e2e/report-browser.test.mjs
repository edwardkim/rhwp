import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import puppeteer from 'puppeteer';
import { generateReport } from './report.mjs';
import { isCompletePng } from './png-integrity.mjs';

// Explicit local check, never an automatic extension/CI gate. A missing browser
// fails this test; it must not silently skip the security assertions.
test('generated public and private HTML has no active content for hostile input', async t => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rhwp-report-browser-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const fixtures = path.join(directory, 'fixtures');
  execFileSync(process.execPath, ['rhwp-chrome/e2e/report-fixtures.mjs', '--output', fixtures]);
  const browser = await puppeteer.launch({ headless: true,
    executablePath: process.env.RHWP_REPORT_BROWSER || undefined, args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  // Test markup itself, so CSP cannot hide an escaping regression.
  await page.setBypassCSP(true);
  const input = path.join(fixtures, 'fail-run');
  const original = JSON.parse(await readFile(path.join(input, 'result.json')));
  const payloads = ['<script>globalThis.reportXss=true</script>', '<SCRIPT>globalThis.reportXss=true</SCRIPT>',
    '<ScRiPt>globalThis.reportXss=true</ScRiPt>', '<img src=x onerror="globalThis.reportXss=true">'];
  for (const [index, payload] of payloads.entries()) {
    const data = structuredClone(original);
    data.error = payload;
    data.limitations = [payload];
    data.candidate.version = payload;
    data.environment.chrome = payload;
    const diagnostic = JSON.stringify({ runId: data.runId, stage: payload, error: payload });
    await writeFile(path.join(input, 'smoke.json'), diagnostic);
    data.results[0].artifacts.find(a => a.path === 'smoke.json').sha256 = createHash('sha256').update(diagnostic).digest('hex');
    await writeFile(path.join(input, 'result.json'), JSON.stringify(data));
    const output = path.join(directory, `report-${index}`);
    await generateReport({ input, output });
    for (const name of ['report.html', 'private.html']) {
      await page.setContent(await readFile(path.join(output, name), 'utf8'));
      const dom = await page.evaluate(() => ({
        activeElements: document.querySelectorAll('script,iframe,object,embed,img,link').length,
        eventAttributes: [...document.querySelectorAll('*')].flatMap(element =>
          [...element.attributes].filter(attribute => attribute.name.toLowerCase().startsWith('on')).map(a => a.name)),
        executed: globalThis.reportXss === true,
      }));
      assert.deepEqual(dom, { activeElements: 0, eventAttributes: [], executed: false }, `${name}: ${payload}`);
    }
  }
  await page.setViewport({ width: 1200, height: 900 });
  await page.setContent(await readFile(path.join(fixtures, 'pass-report/report.html'), 'utf8'));
  assert.equal(isCompletePng(Buffer.from(await page.screenshot())), true, 'real Chromium screenshot');
  for (const fixture of ['pass', 'fail', 'error', 'not-run', 'incomplete']) {
    await page.setContent(await readFile(path.join(fixtures, `${fixture}-report/report.html`), 'utf8'));
    await page.setViewport({ width: 1200, height: 900 });
    const layout = await page.evaluate(() => {
      const details = document.querySelector('details');
      const table = document.querySelector('table');
      const summary = document.querySelector('[aria-label="결과 요약"]');
      return { collapsed: !details.open, resultsAboveMetadata: table.getBoundingClientRect().top < details.getBoundingClientRect().top,
        statusesVisible: summary.getBoundingClientRect().bottom <= innerHeight,
        resultsVisible: table.getBoundingClientRect().top < innerHeight };
    });
    assert.deepEqual(layout, { collapsed: true, resultsAboveMetadata: true, statusesVisible: true, resultsVisible: true }, fixture);
    for (const width of [1200, 390]) {
      await page.setViewport({ width, height: 900 });
      for (const open of [false, true]) {
        await page.$$eval('details', (items, open) => items.forEach(item => { item.open = open; }), open);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, `${fixture}: ${width}px, details=${open}`);
      }
    }
  }
  console.log(`Report browser validation: ${await browser.version()}`);
});
