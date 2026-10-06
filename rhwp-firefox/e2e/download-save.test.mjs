/**
 * Packaged Firefox download → edit → save/save-as regression (#6964).
 * Build rhwp-firefox first; reuse Puppeteer from `npm ci --prefix rhwp-chrome`.
 * FIREFOX_EXECUTABLE_PATH=/path/to/firefox npm --prefix rhwp-firefox run test:e2e:download
 * RHWP_FIREFOX_DIST optionally selects a combined-fix package.
 * RHWP_EXPECT_BASENAME=1 additionally checks #6961 when testing both fixes together.
 * Uses an isolated profile, loopback fixture, and automatic download directory;
 * exercises production DOM commands, not the native OS file picker.
 */
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtemp, readFile, mkdir, rm, readdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import http from 'node:http';

const root = fileURLToPath(new URL('../../', import.meta.url));
const require = createRequire(new URL('../../rhwp-chrome/package.json', import.meta.url));
const { default: puppeteer } = require('puppeteer');
const executablePath = process.env.FIREFOX_EXECUTABLE_PATH;
assert.ok(executablePath, 'Set FIREFOX_EXECUTABLE_PATH to the Firefox binary');
const dist = resolve(process.env.RHWP_FIREFOX_DIST || join(root, 'rhwp-firefox/dist'));
const temp = await mkdtemp(join(tmpdir(), 'rhwp-firefox-save-'));
const downloads = join(temp, 'downloads');
await mkdir(downloads);
const filename = '신청서(SW) (개인)_1.hwp';
const marker = 'ISSUE6964_EDIT';
const bytes = await readFile(join(root, 'samples/re-font-dotum-empty-hancom.hwp'));
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(check, message) {
  const deadline = Date.now() + 30_000;
  do {
    const result = await check();
    if (result) return result;
    await delay(100);
  } while (Date.now() < deadline);
  throw new Error(`Timed out: ${message}`);
}
const server = http.createServer((req, res) => {
  if (req.url === '/document.hwp') {
    res.writeHead(200, {
      'content-type': 'application/x-hwp',
      'content-disposition': `attachment; filename*=UTF-8''${encodeURIComponent(filename)}`,
    });
    res.end(bytes);
  } else {
    res.writeHead(200, { 'content-type': 'text/html;charset=utf-8' });
    res.end('<a id="download" href="/document.hwp">Download</a>');
  }
});
let browser;
try {
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => { server.off('error', reject); resolve(); });
  });
  browser = await puppeteer.launch({
    browser: 'firefox', executablePath, headless: true,
    args: ['--remote-allow-system-access'], userDataDir: join(temp, 'profile'),
    extraPrefsFirefox: {
      'browser.download.folderList': 2,
      'browser.download.dir': downloads,
      'browser.helperApps.neverAsk.saveToDisk': 'application/x-hwp',
      'browser.download.useDownloadDir': true,
    },
  });
  console.log('Firefox:', await browser.version());
  await browser.installExtension(dist);
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  // Installation can resolve before the module background registers observers.
  // A badge appears only after content-script's get-settings round trip succeeds.
  await until(() => page.evaluate(() => Boolean(document.querySelector('.rhwp-badge'))), 'extension ready');
  await page.click('#download');
  const viewer = await until(async () => {
    for (const candidate of await browser.pages()) {
      // BiDi Page.url() can report about:blank for privileged extension pages.
      const url = await candidate.evaluate(() => location.href);
      if (url.includes('/viewer.html?')) return candidate;
    }
  }, 'auto-open viewer');
  await until(() => viewer.evaluate(() =>
    document.querySelector('textarea') && document.body.innerText.includes('1페이지')),
  'loaded fixture');
  const getDownloads = () => viewer.evaluate(() => browser.downloads.search({}));
  async function assertDownloadAndTabs(count) {
    await until(async () => {
      const items = await getDownloads();
      return items.length === count && items.every(item => item.state === 'complete');
    }, `${count} completed downloads`);
    // Let the asynchronous onChanged observer finish after download completion.
    await delay(1000);
    const tabs = await viewer.evaluate(() => browser.tabs.query({}));
    assert.equal(tabs.filter(tab => tab.url?.includes('/viewer.html?')).length, 1,
      `expected one viewer after ${count} downloads`);
    return getDownloads();
  }
  await assertDownloadAndTabs(1);
  if (process.env.RHWP_EXPECT_BASENAME === '1') {
    assert.equal(await viewer.evaluate(() => new URL(location.href).searchParams.get('filename')), filename);
  }
  await viewer.evaluate(value => {
    const input = document.querySelector('textarea');
    input.focus(); input.value = value;
    input.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: value }));
  }, marker);
  async function command(name) {
    await viewer.evaluate(name => {
      document.querySelector('[data-menu="file"] .menu-title')
        .dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      document.querySelector(`[data-cmd="${name}"]`).click();
    }, name);
  }
  await command('file:save');
  await assertDownloadAndTabs(2);
  await command('file:save-as-hwp');
  // Firefox currently has the format-name dialog and download fallback dialog.
  for (let index = 0; index < 2; index++) {
    await until(() => viewer.evaluate(() => Boolean(document.querySelector('.modal-overlay input'))), 'save-as name');
    if (process.env.RHWP_EXPECT_BASENAME === '1') {
      assert.equal(await viewer.evaluate(() => document.querySelector('.modal-overlay input').value), filename.slice(0, -4));
    }
    await viewer.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary').click());
    await delay(200);
  }
  const items = await assertDownloadAndTabs(3);
  const outputItems = items.filter(item => item.url.startsWith('blob:moz-extension://'));
  assert.equal(outputItems.length, 2);
  const { default: init, HwpDocument } = await import(new URL('../../pkg/rhwp.js', import.meta.url));
  await init({ module_or_path: await readFile(join(root, 'pkg/rhwp_bg.wasm')) });
  for (const item of outputItems) {
    if (process.env.RHWP_EXPECT_BASENAME === '1') {
      assert.match(basename(item.filename), /^신청서\(SW\) \(개인\)_1(?:\(\d+\))?\.hwp$/);
    }
    const doc = new HwpDocument(await readFile(item.filename));
    try { assert.ok(doc.getTextFileText().includes(marker), 'saved HWP retains edit'); }
    finally { doc.free(); }
  }
  console.log(JSON.stringify({ viewerTabs: 1, completedDownloads: items.length,
    savedNames: outputItems.map(item => basename(item.filename)), editPreserved: true }));
} catch (error) {
  console.error('Download files:', await readdir(downloads));
  if (browser) {
    for (const page of await browser.pages()) {
      console.error('Page:', await page.evaluate(() => ({ url: location.href, text: document.body?.innerText.slice(-600) })));
      if ((await page.evaluate(() => location.href)).startsWith('moz-extension:')) {
        console.error('STATE', await page.evaluate(() => browser.storage.session.get(null)));
        console.error('DOWNLOADS', await page.evaluate(() => browser.downloads.search({})));
      }
    }
  }
  throw error;
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
  await rm(temp, { recursive: true, force: true });
}
