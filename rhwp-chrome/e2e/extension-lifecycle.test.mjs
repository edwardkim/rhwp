#!/usr/bin/env node
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, mkdtemp, readFile, rm, statfs, writeFile } from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer';
import { chromeArgs, rejectProxyConnect } from './extension-smoke.test.mjs';
import { monitorTabs } from './tab-monitor.mjs';
import { recordConsoleMessage } from './failure-diagnostics.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const DIST = path.resolve(process.env.RHWP_EXTENSION_DIST_DIR ?? path.join(ROOT, 'rhwp-chrome/dist'));
const TIMEOUT = 30_000;
const QUIET_MS = 1_500;
const repeat = Number(process.env.RHWP_EXTENSION_LIFECYCLE_REPEAT ?? 1);
assert.ok(Number.isSafeInteger(repeat) && repeat > 0, 'repeat must be a positive integer');
const cases = [
  ...['reenter', 'worker', 'restart'].map(transition => ({ id: `off-${transition}`, transition })),
  ...['hwp', 'hwpx'].flatMap(format => [
    { id: `off-download-${format}`, format, autoOpen: false },
    { id: `on-download-${format}`, format, autoOpen: true },
    { id: `wake-download-${format}`, format, autoOpen: true, wake: true },
  ]),
  { id: 'past-downloads', history: true },
];
const selected = process.env.RHWP_EXTENSION_LIFECYCLE_CASE
  ? cases.filter(item => item.id === process.env.RHWP_EXTENSION_LIFECYCLE_CASE) : cases;
assert.ok(selected.length > 0, 'unknown lifecycle case');
const fixtures = {
  hwp: await readFile(path.join(ROOT, 'samples/hwp3-pagedef-1915.hwp')),
  hwpx: await readFile(path.join(ROOT, 'samples/hwpx_sample2.hwpx')),
};
const results = [];
for (let iteration = 1; iteration <= repeat; iteration++) {
  for (const scenario of selected) {
    process.stdout.write(`START ${iteration}/${repeat} ${scenario.id}\n`);
    results.push(await runScenario(scenario, iteration));
    process.stdout.write(`PASS ${iteration}/${repeat} ${scenario.id}\n`);
  }
}
process.stdout.write(`PASS: ${results.length} lifecycle scenarios, no retries\n`);

async function runScenario(scenario, iteration) {
  const temp = await mkdtemp(path.join(os.tmpdir(), 'rhwp-lifecycle-'));
  const profile = path.join(temp, 'profile');
  const downloadsDir = path.join(temp, 'downloads');
  await mkdir(downloadsDir);
  const diagnostic = {
    scenario: scenario.id, iteration, stage: 'launch', events: [], console: [], errors: [],
    blockedProxyRequests: [], proxyClientAborts: [], downloads: [],
  };
  const fixture = await fixtureServer(diagnostic);
  let browser, page, session, monitor, extensionId;
  let failure;
  const abort = new AbortController();
  const poll = check => pollUntil(check, abort.signal);
  const started = Date.now();
  const guard = operation => within(monitor ? monitor.guard(operation) : operation);
  const observe = expected => {
    monitor?.detach();
    monitor = monitorTabs(browser, page.target(), expected, diagnostic.events);
  };

  async function launch(withExtension = true) {
    browser = await puppeteer.launch({
      headless: true, enableExtensions: withExtension ? [DIST] : undefined,
      userDataDir: profile, args: chromeArgs(fixture.origin),
    });
    diagnostic.browserVersion = await browser.version();
    const initial = await browser.pages();
    assert.equal(initial.length, 1, 'only the owned browser page may exist on startup');
    [page] = initial;
    page.setDefaultTimeout(TIMEOUT);
    page.setDefaultNavigationTimeout(TIMEOUT);
    observe(0);
    page.on('console', msg => recordConsoleMessage(diagnostic, msg, page.url()));
    page.on('pageerror', error => diagnostic.errors.push(error.message));
    page.on('request', request => {
      if (/^https?:/.test(request.url()) && new URL(request.url()).origin !== fixture.origin) {
        diagnostic.errors.push(`unexpected request: ${request.url()}`);
      }
    });
    browser.on('targetcreated', target => {
      if (target.type() === 'service_worker') diagnostic.events.push({ event: 'worker-start', url: target.url() });
    });
    browser.on('targetdestroyed', target => {
      if (target.type() === 'service_worker') diagnostic.events.push({ event: 'worker-stop', url: target.url() });
    });
    session = await browser.target().createCDPSession();
    await session.send('Browser.setDownloadBehavior', {
      behavior: 'allow', downloadPath: downloadsDir, eventsEnabled: true,
    });
    session.on('Browser.downloadWillBegin', event => diagnostic.downloads.push({ ...event, progress: [] }));
    session.on('Browser.downloadProgress', event => {
      diagnostic.downloads.find(item => item.guid === event.guid)?.progress.push(event);
    });
    if (withExtension) {
      const target = await guard(browser.waitForTarget(isWorker));
      const detected = new URL(target.url()).hostname;
      if (extensionId) assert.equal(detected, extensionId, 'extension identity survives profile restart');
      extensionId = detected;
      diagnostic.extensionId = extensionId;
    }
  }

  async function options() {
    await guard(page.goto(`chrome-extension://${extensionId}/options.html`));
    await guard(page.waitForSelector('#autoOpen:not(:disabled)', { visible: true }));
  }
  async function setOption(id, value) {
    if (await page.$eval(`#${id}`, input => input.checked) === value) return;
    await guard(page.click(`#${id}`));
    await guard(page.waitForFunction((key, expected) => {
      const input = document.getElementById(key);
      return !input.disabled && input.checked === expected
        && document.querySelector('#saved.show:not(.error)') !== null;
    }, {}, id, value));
  }
  async function configure(autoOpen) {
    diagnostic.stage = 'options-save';
    await options();
    await setOption('disableExternalWebFonts', true);
    await setOption('showBadges', false);
    await setOption('hoverPreview', false);
    // Exercise an actual save even when the desired value matches the default.
    await setOption('autoOpen', !autoOpen);
    await setOption('autoOpen', autoOpen);
    diagnostic.savedAutoOpen = autoOpen;
  }
  async function stopWorker() {
    const target = await guard(browser.waitForTarget(isWorker));
    const worker = await guard(target.worker());
    await guard(worker.close());
    await guard(poll(() => !browser.targets().includes(target)));
    assert.ok(!browser.targets().some(isWorker), 'worker is stopped before the user action');
    return target;
  }
  async function download(format, expected) {
    const name = `${scenario.id}-${randomUUID()}.${format}`;
    fixture.files.set(name, fixtures[format]);
    await guard(page.goto(`${fixture.origin}/fixture.html?file=${name}`));
    // A content script can wake a worker; terminate only after the fixture page is ready.
    let oldTarget;
    if (scenario.wake) {
      diagnostic.stage = 'worker-stop';
      oldTarget = await stopWorker();
    }
    diagnostic.stage = `download-${format}`;
    observe(expected);
    await guard(page.click('#download'));
    const record = await guard(poll(() => diagnostic.downloads.find(item => item.suggestedFilename === name)));
    await guard(poll(() => {
      assert.ok(!record.progress.some(event => event.state === 'canceled'),
        `Chrome canceled download ${record.guid}: ${record.suggestedFilename}`);
      return record.progress.some(event => event.state === 'completed');
    }));
    assert.deepEqual(await readFile(path.join(downloadsDir, name)), fixtures[format]);
    if (expected) await guard(browser.waitForTarget(target => target.type() === 'page'
      && target.url().startsWith(`chrome-extension://${extensionId}/viewer.html?`)));
    await guard(delay(QUIET_MS));
    monitor.assertComplete(extensionId);
    if (scenario.wake) {
      const next = await guard(browser.waitForTarget(isWorker));
      assert.notEqual(next, oldTarget, 'download started a new worker');
    }
    diagnostic.events.push({ event: 'download-assertion', format, expected, observed: monitor.targets().length });
    for (const target of monitor.targets()) {
      assert.equal(new URL(target.url()).searchParams.get('filename'), name);
      await guard((await target.page()).close());
    }
    observe(0);
    return name;
  }
  async function restart() {
    // Close the owned tab explicitly so Chrome session restore does not add a
    // second about:blank alongside Puppeteer's startup tab on relaunch.
    monitor.assertComplete(extensionId);
    await guard(page.close());
    monitor.detach();
    monitor = null;
    await within(browser.close());
    browser = null;
    await launch();
  }

  try {
    await launch(!scenario.history);
    if (scenario.history) {
      diagnostic.stage = 'prepare-history-without-extension';
      const names = [];
      for (const format of ['hwp', 'hwpx']) names.push(await download(format, 0));
      // Make the already completed history older than the production freshness grace.
      await guard(delay(6_000));
      await restart();
      diagnostic.stage = 'verify-persisted-history';
      await guard(page.goto('chrome://downloads/'));
      await guard(page.waitForFunction(expected => {
        const visibleNames = [];
        function visit(root) {
          for (const element of root.querySelectorAll('*')) {
            if (element.tagName === 'A' && element.getClientRects().length) visibleNames.push(element.textContent);
            if (element.shadowRoot) visit(element.shadowRoot);
          }
        }
        visit(document);
        return expected.every(name => visibleNames.some(text => text.includes(name)));
      }, {}, names));
      await configure(true);
      await guard(delay(QUIET_MS));
      monitor.assertComplete(extensionId);
    } else if (scenario.transition) {
      await configure(false);
      diagnostic.stage = `transition-${scenario.transition}`;
      await guard(page.goto('about:blank'));
      if (scenario.transition === 'worker') await stopWorker();
      if (scenario.transition === 'restart') await restart();
      await options();
      assert.equal(await page.$eval('#autoOpen', input => input.checked), false);
      await guard(delay(QUIET_MS));
      monitor.assertComplete(extensionId);
    } else {
      await configure(scenario.autoOpen);
      await download(scenario.format, scenario.autoOpen ? 1 : 0);
    }
    assert.deepEqual(diagnostic.errors, [], 'page errors');
    if (diagnostic.browserUiErrors?.length) {
      process.stdout.write(`${JSON.stringify({ scenario: scenario.id, browserUiErrors: diagnostic.browserUiErrors })}\n`);
    }
  } catch (error) {
    failure = error;
    diagnostic.error = error.stack;
    diagnostic.openTabs = browser?.targets().filter(target => target.type() === 'page').map(target => target.url());
    diagnostic.workers = browser?.targets().filter(isWorker).map(target => target.url());
    // Reading extension internals is reserved for failure diagnostics.
    const disk = await statfs(downloadsDir).catch(() => null);
    if (disk) diagnostic.downloadDiskFreeBytes = disk.bavail * disk.bsize;
    const workerTarget = browser?.targets().find(isWorker);
    if (workerTarget) {
      // Query only URLs served by this scenario's own loopback fixture. Retain
      // error codes and IDs, never a browser-wide download list or preferences.
      const urls = diagnostic.downloads.map(item => item.url).filter(url => new URL(url).origin === fixture.origin);
      diagnostic.fixtureDownloadErrors = await within((async () => {
        const worker = await workerTarget.worker();
        return worker.evaluate(async fixtureUrls => {
          const result = [];
          for (const url of fixtureUrls) {
            const items = await chrome.downloads.search({ url });
            result.push(...items.map(item => ({ id: item.id, state: item.state, error: item.error || null })));
          }
          return result;
        }, urls);
      })(), 2_000).catch(String);
    }
    if (page?.url().startsWith('chrome-extension://')) {
      diagnostic.settings = await within(page.evaluate(() => chrome.storage.sync.get('autoOpen')), 2_000).catch(String);
    }
    const output = process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR;
    if (output) {
      await mkdir(output, { recursive: true });
      await within(page?.screenshot({ path: path.join(output, `${scenario.id}-${iteration}.png`) }), 2_000).catch(() => {});
      await writeFile(path.join(output, `${scenario.id}-${iteration}.json`), JSON.stringify(diagnostic, null, 2));
    }
  } finally {
    abort.abort();
    monitor?.detach();
    const cleanupErrors = [];
    if (browser) await within(browser.close(), 5_000).catch(error => cleanupErrors.push(error));
    await within(new Promise((resolve, reject) => {
      fixture.server.close(error => error ? reject(error) : resolve());
      fixture.server.closeAllConnections();
    }), 5_000).catch(error => cleanupErrors.push(error));
    await rm(temp, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 }).catch(error => cleanupErrors.push(error));
    if (cleanupErrors.length) failure = new AggregateError([...(failure ? [failure] : []), ...cleanupErrors], 'scenario cleanup failed');
  }
  if (failure) throw new Error(`${scenario.id}: ${failure.message}\n${JSON.stringify(diagnostic)}`, { cause: failure });
  return { scenario: scenario.id, iteration, durationMs: Date.now() - started, browser: diagnostic.browserVersion };
}

function isWorker(target) {
  return target.type() === 'service_worker' && /^chrome-extension:\/\/[^/]+\/background.js$/.test(target.url());
}
function delay(ms) { return new Promise(resolve => setTimeout(resolve, ms)); }
async function pollUntil(check, signal) {
  const deadline = Date.now() + TIMEOUT;
  while (Date.now() < deadline) {
    signal.throwIfAborted();
    const result = await check();
    if (result) return result;
    await delay(25);
  }
  throw new Error('condition timed out');
}
async function within(operation, timeout = TIMEOUT) {
  let timer;
  try {
    return await Promise.race([operation, new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error(`operation timed out after ${timeout}ms`)), timeout);
    })]);
  } finally { clearTimeout(timer); }
}
async function fixtureServer(diagnostic) {
  const files = new Map();
  const server = http.createServer((request, response) => {
    if (!request.headers.host?.startsWith('127.0.0.1:') || /^https?:/.test(request.url)) {
      diagnostic.blockedProxyRequests.push(`${request.method} ${request.url}`);
      response.writeHead(502).end();
      return;
    }
    const url = new URL(request.url, `http://${request.headers.host}`);
    const name = url.pathname.slice(1);
    if (url.pathname === '/fixture.html') {
      // Render only the server-owned name, never reflect the request value.
      const file = [...files.keys()].find(registered => registered === url.searchParams.get('file'));
      if (!file) { response.writeHead(404).end(); return; }
      const encodedFile = encodeURIComponent(file);
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(`<!doctype html><meta charset="utf-8"><link rel="icon" href="data:,"><a id="download" href="/${encodedFile}" download="${encodedFile}">Download fixture</a>`);
    } else if (files.has(name)) {
      const bytes = files.get(name);
      response.writeHead(200, {
        'content-type': name.endsWith('.hwpx') ? 'application/hwp+zip' : 'application/x-hwp',
        'content-disposition': `attachment; filename="${name}"`,
        'content-length': bytes.length, 'access-control-allow-origin': '*', 'cache-control': 'no-store',
      });
      response.end(bytes);
    } else response.writeHead(url.pathname === '/favicon.ico' ? 204 : 404).end();
  });
  server.on('connect', (request, socket) => rejectProxyConnect(request.url, socket, diagnostic));
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  return { server, files, origin: `http://127.0.0.1:${server.address().port}` };
}
