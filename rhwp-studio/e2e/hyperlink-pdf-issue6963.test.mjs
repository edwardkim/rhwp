/**
 * #6963 — 실제 Studio 링크 dialog → 저장 명령 → file input 재열기 → PDF.
 * OS 파일 picker/write와 native print()만 캡처한다. 캡처한 실제 인쇄 DOM을
 * Chromium printToPDF에 전달하며 SVG 링크가 /Link 주석이 되는지 별도로 검사한다.
 * CHROME_PATH=/path/to/chrome VITE_URL=http://127.0.0.1:7763 \
 *   node e2e/hyperlink-pdf-issue6963.test.mjs --mode=headless
 * PDF 주석 검사에는 PYTHON으로 지정한 Python의 pypdf 패키지가 필요하다.
 */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { strict as assert } from 'node:assert';
import { runTest, createNewDocument, loadHwpFile } from './helpers.mjs';

const output = resolve('../output/pdf/issue6963-stage5');
const uri = 'https://example.com/한글?q=1&lang=ko#부분';
const text = '한컴 링크 😀';

async function fileMenu(page, command) {
  await page.evaluate((cmd) => {
    const title = [...document.querySelectorAll('#menu-bar .menu-title')]
      .find((node) => node.textContent.includes('파일'));
    assertElement(title).dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    assertElement(document.querySelector(`.md-item[data-cmd="${cmd}"]`)).click();
    function assertElement(node) {
      if (!node) throw new Error(`메뉴 없음: ${cmd}`);
      return node;
    }
  }, command);
}

async function capturePdf(page, browser, name) {
  await page.evaluate(() => {
    window.__hyperlinkPrint = null;
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) for (const node of mutation.addedNodes) {
        if (!(node instanceof HTMLIFrameElement) || node.id !== 'rhwp-print-surface') continue;
        node.addEventListener('load', () => {
          node.contentWindow.print = () => {
            window.__hyperlinkPrint = node.contentDocument.documentElement.outerHTML;
          };
        }, { once: true });
      }
    });
    observer.observe(document.body, { childList: true });
    window.__hyperlinkPrintObserver = observer;
  });
  await fileMenu(page, 'file:print-to-pdf');
  await page.waitForSelector('[data-testid="pdf-print-dialog"]');
  await page.click('.dialog-btn-primary');
  await page.waitForFunction(() => window.__hyperlinkPrint !== null, { timeout: 60000 });
  const html = await page.evaluate(() => {
    window.__hyperlinkPrintObserver.disconnect();
    return window.__hyperlinkPrint;
  });
  writeFileSync(`${output}/${name}.html`, html);
  const print = await browser.newPage();
  try {
    await print.setContent(html, { waitUntil: 'load' });
    await print.emulateMediaType('print');
    await print.evaluate(() => document.fonts.ready);
    const links = await print.evaluate(() => [...document.querySelectorAll('a[href]')].map((a) => {
      const r = a.getBoundingClientRect();
      const page = a.closest('.page');
      const p = page.getBoundingClientRect();
      return { uri: a.getAttribute('href'), page: [...document.querySelectorAll('.page')].indexOf(page),
        x: r.x - p.x, y: r.y - p.y, width: r.width, height: r.height };
    }));
    await print.pdf({ path: `${output}/${name}.pdf`, printBackground: true, preferCSSPageSize: true });
    if (name === 'studio-hwp' || name === 'textmail') {
      await print.evaluate(() => {
        for (const group of document.querySelectorAll('[data-rhwp-hyperlinks]')) group.remove();
      });
      await print.pdf({ path: `${output}/${name}-without-links.pdf`, printBackground: true, preferCSSPageSize: true });
    }
    return links;
  } finally { await print.close(); }
}

async function clickInChromePdfViewer(browser) {
  const server = createServer((_request, response) => {
    response.writeHead(200, { 'Content-Type': 'application/pdf' });
    response.end(readFileSync(`${output}/studio-hwp.pdf`));
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const viewer = await browser.newPage();
  try {
    await viewer.setViewport({ width: 800, height: 600 });
    await viewer.setRequestInterception(true);
    viewer.on('request', request => request.url().startsWith('https://example.com/')
      ? request.respond({ status: 200, contentType: 'text/html', body: '<h1>Hyperlink clicked</h1>' })
      : request.continue());
    await viewer.goto(`http://127.0.0.1:${server.address().port}/studio-link.pdf`);
    const frame = await viewer.waitForFrame(frame => frame.url().startsWith('chrome-extension:'));
    await frame.waitForSelector('pdf-viewer');
    // The fixed single-page fixture at 800x600 opens fit-width with the sidebar.
    // The screenshot verifies this concrete hit point inside its first text link.
    await new Promise(resolve => setTimeout(resolve, 1500));
    await viewer.screenshot({ path: `${output}/chrome-pdf-viewer.png` });
    await viewer.mouse.click(390, 142);
    await viewer.waitForFunction(() => document.querySelector('h1')?.textContent === 'Hyperlink clicked');
    assert.equal(viewer.url(), encodeURI(uri));
    return { viewer: 'Chrome built-in PDF viewer', url: viewer.url(), x: 390, y: 142,
      externalRequest: 'intercepted with local test response' };
  } finally {
    await viewer.close();
    await new Promise(resolve => server.close(resolve));
  }
}

await runTest('#6963 Studio hyperlink save/reopen and browser PDF', async ({ page, browser }) => {
  mkdirSync(output, { recursive: true });
  await createNewDocument(page);
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('#hyperlink-text');
  await page.type('#hyperlink-text', '수정하기 전 링크 문자열');
  await page.type('#hyperlink-uri', uri);
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-text', { hidden: true });
  await page.evaluate(() => { window.__inputHandler.cursor.clearSelection(); window.__inputHandler.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 1 }); });
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('[role="alertdialog"]');
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-text');
  await page.$eval('#hyperlink-text', (el, value) => { el.value = value; el.dispatchEvent(new Event('input', { bubbles: true })); }, text);
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-text', { hidden: true });
  assert.equal(await page.evaluate(() => window.__documentState.isDirty()), true);
  const context = () => page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }));
  const before = await context();
  assert.equal(before.links[0].uri, uri);
  assert.equal(before.text, text);
  const evidence = { browser: await browser.version(), uri, text, before, formats: {}, pdfs: {}, oracles: {} };
  for (const format of ['hwp', 'hwpx']) {
    await page.evaluate(() => {
      window.__hyperlinkSaved = null;
      window.showSaveFilePicker = async (options) => ({
        name: options.suggestedName,
        async createWritable() { return {
          async write(blob) { window.__hyperlinkSaved = [...new Uint8Array(await blob.arrayBuffer())]; },
          async close() {},
        }; },
      });
    });
    await fileMenu(page, `file:save-as-${format}`);
    await page.waitForSelector('.dialog-btn-primary');
    await page.click('.dialog-btn-primary');
    await page.waitForFunction(() => window.__hyperlinkSaved !== null);
    const bytes = Buffer.from(await page.evaluate(() => window.__hyperlinkSaved));
    assert.equal(bytes.subarray(0, format === 'hwp' ? 4 : 2).toString('hex'), format === 'hwp' ? 'd0cf11e0' : '504b');
    const path = `${output}/studio-link.${format}`;
    writeFileSync(path, bytes);
    await page.waitForFunction(() => !window.__documentState.isDirty());
    await (await page.$('#file-input')).uploadFile(path);
    await page.waitForFunction((name) => window.__wasm.fileName === name, {}, `studio-link.${format}`);
    await page.waitForFunction(() => !!window.__inputHandler);
    const reopened = await context();
    assert.deepEqual(reopened, before);
    evidence.formats[format] = { bytes: bytes.length, reopened };
    evidence.pdfs[`studio-${format}`] = await capturePdf(page, browser, `studio-${format}`);
    assert.equal(evidence.pdfs[`studio-${format}`].length, 1);
    assert.equal(evidence.pdfs[`studio-${format}`][0].uri, encodeURI(uri));
    console.log(`${format}: save/reopen + Chromium PDF captured`);
  }
  evidence.viewerClick = await clickInChromePdfViewer(browser);
  console.log('Chrome PDF viewer: real hit-area click navigated to the preserved URI');
  await createNewDocument(page);
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('#hyperlink-text');
  await page.$eval('#hyperlink-text', (input) => {
    input.value = 'alpha beta gamma delta '.repeat(240);
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await page.type('#hyperlink-uri', 'https://example.com/multiline');
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-text', { hidden: true });
  evidence.pdfs.multiline = await capturePdf(page, browser, 'multiline');
  assert(new Set(evidence.pdfs.multiline.map(link => link.page)).size > 1);
  assert(evidence.pdfs.multiline.length > 2);
  console.log('Multiline link: multiple PDF pages captured');
  for (const sample of [
    { name: 'textmail', source: 'basic/Textmail.hwp', pdf: 'pdf/basic/Textmail-2022.pdf', uris: ['http://www.hancom.co.kr'] },
    { name: 'lh', source: 'hwpx_sample2.hwpx', pdf: 'pdf/hwpx_sample2-hwpx-2020.pdf', uris: ['https://apply.lh.or.kr/', 'https://apply.lh.or.kr/LH/index.html#MN::CLCC_MN_0010:'] },
  ]) {
    await loadHwpFile(page, sample.source);
    evidence.pdfs[sample.name] = await capturePdf(page, browser, sample.name);
    evidence.oracles[sample.name] = { pdf: sample.pdf, uris: sample.uris };
    assert(evidence.pdfs[sample.name].length > 0);
    console.log(`${sample.name}: Hancom sample → Chromium PDF captured`);
  }
  writeFileSync(`${output}/browser-evidence.json`, JSON.stringify(evidence, null, 2) + '\n');
  console.log(execFileSync(process.env.PYTHON || 'python3',
    ['tools/verify_studio_hyperlink_pdf.py', output], { cwd: resolve('..'), encoding: 'utf8' }));
});
