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
    // 확장 페이지의 실제 문맥을 검사하는 격리 브라우저에만 권한을 공급한다.
    env: { ...process.env, MOZ_REMOTE_ALLOW_SYSTEM_ACCESS: '1' },
    userDataDir: join(temp, 'profile'),
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
    // Firefox가 확장에서 연 탭은 Puppeteer의 페이지 캐시에 빠질 수 있다.
    // 실제 BiDi 문맥을 조회하여 자동으로 열린 바로 그 뷰어에서 DOM 명령을 실행한다.
    const { result: { contexts } } = await browser.connection.send('browsingContext.getTree', {});
    const context = contexts.find(context => context.url.includes('/viewer.html?'));
    if (!context) return;
    return {
      async evaluate(fn, ...args) {
        const functionDeclaration = `function(serialized) {
          return Promise.resolve((${fn.toString()})(...JSON.parse(serialized)))
            .then(value => JSON.stringify(value === undefined ? null : value));
        }`;
        const { result } = await browser.connection.send('script.callFunction', {
          functionDeclaration, target: { context: context.context }, awaitPromise: true,
          arguments: [{ type: 'string', value: JSON.stringify(args) }],
        });
        if (result.type !== 'success') {
          throw new Error(`Firefox 뷰어 실행 실패: ${JSON.stringify(result.exceptionDetails)}`);
        }
        return JSON.parse(result.result.value);
      },
    };
  }, 'auto-open viewer');
  // #1328/#1454 asks the user before checking installed fonts. Select the real
  // fallback button instead of bypassing consent; status text is now "1 / 1 쪽".
  await until(() => viewer.evaluate(expectedFilename => {
    const button = [...document.querySelectorAll('.modal-overlay button')].find(button =>
      /^(대체 글꼴로 보기|View with substitute fonts)$/.test(button.textContent.trim()));
    button?.click();
    return document.querySelector('textarea')
      && document.getElementById('sb-message')?.textContent.includes(expectedFilename);
  }, filename),
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
