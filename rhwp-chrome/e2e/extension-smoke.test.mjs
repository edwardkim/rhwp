#!/usr/bin/env node

import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const EXTENSION_DIR = path.resolve(HERE, '..');
const ROOT = path.resolve(EXTENSION_DIR, '..');
const DIST_DIR = path.join(EXTENSION_DIR, 'dist');
const FIXTURE_FILE = path.join(ROOT, 'samples', 'hwp3-pagedef-1915.hwp');
const SURFACE_TIMEOUT_MS = positiveInteger(
  process.env.RHWP_EXTENSION_SMOKE_TIMEOUT_MS,
  60_000,
  'RHWP_EXTENSION_SMOKE_TIMEOUT_MS',
);
const REPEAT = positiveInteger(
  process.env.RHWP_EXTENSION_SMOKE_REPEAT,
  1,
  'RHWP_EXTENSION_SMOKE_REPEAT',
);
const MAX_PAGES = 1;
const SETTINGS = Object.freeze({
  autoOpen: false,
  showBadges: true,
  hoverPreview: false,
  disableExternalWebFonts: true,
});

await main();

async function main() {
  assert.ok(existsSync(path.join(DIST_DIR, 'manifest.json')), '먼저 rhwp-chrome dist를 빌드해야 합니다.');
  assert.ok(existsSync(FIXTURE_FILE), `smoke fixture가 없습니다: ${FIXTURE_FILE}`);

  for (let iteration = 1; iteration <= REPEAT; iteration += 1) {
    const prefix = REPEAT > 1 ? `[${iteration}/${REPEAT}] ` : '';
    process.stdout.write(`${prefix}Chrome extension packaged smoke 시작\n`);
    await runOnce(prefix);
    process.stdout.write(`${prefix}PASS: viewer/options/print/service worker/content script\n`);
  }
}

async function runOnce(prefix) {
  const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), 'rhwp-extension-smoke-'));
  const userDataDir = path.join(temporaryRoot, 'profile');
  const downloadDir = path.join(temporaryRoot, 'downloads');
  const diagnostics = createDiagnostics();
  let browser = null;
  let server = null;
  let workerSession = null;
  let failure = null;

  await mkdir(userDataDir, { recursive: true });
  await mkdir(downloadDir, { recursive: true });

  try {
    const fixture = await startFixtureServer(diagnostics);
    server = fixture.server;

    browser = await puppeteer.launch({
      headless: true,
      enableExtensions: [DIST_DIR],
      userDataDir,
      args: chromeArgs(fixture.origin),
    });

    const pages = await browser.pages();
    assert.ok(
      pages.length <= MAX_PAGES,
      `Chrome 시작 직후 예기치 않은 page가 열렸습니다: ${JSON.stringify(pages.map(page => page.url()))}`,
    );
    const page = pages[0] ?? await browser.newPage();
    page.setDefaultTimeout(SURFACE_TIMEOUT_MS);
    page.setDefaultNavigationTimeout(SURFACE_TIMEOUT_MS);
    attachPageDiagnostics(page, diagnostics, fixture.origin);
    attachPageBudget(browser, page, diagnostics);

    const browserSession = await browser.target().createCDPSession();
    await browserSession.send('Browser.setDownloadBehavior', {
      behavior: 'allow',
      downloadPath: downloadDir,
      eventsEnabled: true,
    });
    await browserSession.detach();

    diagnostics.surface = 'service-worker';
    const workerTarget = await browser.waitForTarget(
      target => target.type() === 'service_worker'
        && target.url().startsWith('chrome-extension://')
        && target.url().endsWith('/background.js'),
      { timeout: SURFACE_TIMEOUT_MS },
    );
    const extensionId = new URL(workerTarget.url()).hostname;
    assert.match(extensionId, /^[a-p]{32}$/, `잘못된 extension ID: ${extensionId}`);
    diagnostics.extensionId = extensionId;
    diagnostics.workerUrl = workerTarget.url();

    workerSession = await attachWorkerDiagnostics(
      workerTarget,
      diagnostics,
      fixture.origin,
      extensionId,
    );
    const worker = await workerTarget.worker();
    assert.ok(worker, 'MV3 service worker 실행 컨텍스트를 얻지 못했습니다.');
    const workerIdentity = await worker.evaluate(async (settings) => {
      await chrome.storage.sync.set(settings);
      return {
        id: chrome.runtime.id,
        manifestVersion: chrome.runtime.getManifest().manifest_version,
      };
    }, SETTINGS);
    assert.equal(workerIdentity.id, extensionId);
    assert.equal(workerIdentity.manifestVersion, 3);
    await assertPageBudget(browser, page, diagnostics);
    assertNoSurfaceErrors(diagnostics, 'service-worker');

    diagnostics.surface = 'viewer-dark';
    await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: 'dark' }]);
    const viewerUrl = new URL(`chrome-extension://${extensionId}/viewer.html`);
    viewerUrl.searchParams.set('url', `${fixture.origin}/smoke.hwp`);
    viewerUrl.searchParams.set('filename', 'extension-smoke.hwp');
    await page.goto(viewerUrl.href, { waitUntil: 'domcontentloaded' });
    await page.waitForSelector('#scroll-container canvas', { visible: true });
    await page.waitForFunction(() => {
      const message = document.getElementById('sb-message')?.textContent?.trim() ?? '';
      return /extension-smoke\.hwp/.test(message);
    });
    const viewerState = await page.evaluate(async (fixtureUrl) => {
      const root = document.documentElement;
      const rawIconUrl = getComputedStyle(root).getPropertyValue('--ui-icon-sprite-url').trim();
      const match = /^url\(["']?(.*?)["']?\)$/.exec(rawIconUrl);
      const iconUrl = match ? new URL(match[1], location.href) : null;
      const iconResponse = iconUrl ? await fetch(iconUrl) : null;
      const iconBytes = iconResponse ? (await iconResponse.arrayBuffer()).byteLength : 0;
      const workerProbe = await chrome.runtime.sendMessage({ type: 'fetch-file', url: fixtureUrl });
      return {
        origin: location.origin,
        theme: root.dataset.themeEffective,
        status: document.getElementById('sb-message')?.textContent?.trim() ?? '',
        canvasCount: document.querySelectorAll('#scroll-container canvas').length,
        iconUrl: iconUrl?.href ?? null,
        iconStatus: iconResponse?.status ?? null,
        iconBytes,
        workerProbeError: workerProbe?.error ?? null,
      };
    }, `${fixture.origin}/smoke.hwp`);
    assert.equal(viewerState.origin, `chrome-extension://${extensionId}`);
    assert.equal(viewerState.theme, 'dark');
    assert.ok(viewerState.canvasCount >= 1, 'viewer 문서 canvas가 생성되지 않았습니다.');
    assert.match(viewerState.status, /extension-smoke\.hwp/);
    assert.match(viewerState.iconUrl ?? '', /\/images\/icon_small_ko_dark\.svg$/);
    assert.equal(viewerState.iconStatus, 200);
    assert.ok(viewerState.iconBytes > 0, '다크 테마 아이콘 자산이 비어 있습니다.');
    assert.match(
      viewerState.workerProbeError ?? '',
      /로컬 또는 내부 네트워크 URL은 차단됩니다/,
      'service worker fetch-file 정책 응답을 받지 못했습니다.',
    );
    await assertPageBudget(browser, page, diagnostics);
    assertNoSurfaceErrors(diagnostics, 'viewer-dark');

    diagnostics.surface = 'print';
    const printState = await page.evaluate(async () => {
      const frame = document.createElement('iframe');
      frame.title = 'extension smoke print surface';
      frame.src = new URL('print.html', document.baseURI).href;
      frame.style.position = 'fixed';
      frame.style.width = '1px';
      frame.style.height = '1px';
      try {
        await new Promise((resolve, reject) => {
          const timeoutId = window.setTimeout(
            () => reject(new Error('print surface load timeout')),
            10_000,
          );
          frame.addEventListener('load', () => {
            window.clearTimeout(timeoutId);
            resolve();
          }, { once: true });
          frame.addEventListener('error', () => {
            window.clearTimeout(timeoutId);
            reject(new Error('print surface load error'));
          }, { once: true });
          document.body.appendChild(frame);
        });
        return {
          url: frame.contentWindow?.location.href ?? null,
          origin: frame.contentWindow?.location.origin ?? null,
          title: frame.contentDocument?.title ?? null,
          statusRole: frame.contentDocument?.querySelector('[role="status"]')?.getAttribute('role') ?? null,
          message: frame.contentDocument?.getElementById('print-loading-message')?.textContent?.trim() ?? null,
        };
      } finally {
        frame.remove();
      }
    });
    assert.equal(printState.url, `chrome-extension://${extensionId}/print.html`);
    assert.equal(printState.origin, `chrome-extension://${extensionId}`);
    assert.equal(printState.title, 'rhwp 인쇄 미리보기');
    assert.equal(printState.statusRole, 'status');
    assert.ok(printState.message?.length > 0, 'print surface 상태 문구가 없습니다.');
    await assertPageBudget(browser, page, diagnostics);
    assertNoSurfaceErrors(diagnostics, 'print');

    diagnostics.surface = 'options';
    await page.goto(`chrome-extension://${extensionId}/options.html`, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(() => {
      const ids = ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts'];
      return ids.every((id) => {
        const input = document.getElementById(id);
        return input instanceof HTMLInputElement && !input.disabled && input.offsetParent !== null;
      });
    });
    const optionsState = await page.evaluate(() => ({
      title: document.getElementById('title')?.textContent?.trim() ?? '',
      version: document.getElementById('version')?.textContent?.trim() ?? '',
      values: Object.fromEntries(
        ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts']
          .map(id => [id, document.getElementById(id)?.checked]),
      ),
    }));
    assert.ok(optionsState.title.length > 0, 'options i18n 제목이 비어 있습니다.');
    assert.match(optionsState.version, /^\d+\.\d+\.\d+$/);
    assert.deepEqual(optionsState.values, SETTINGS);
    await assertPageBudget(browser, page, diagnostics);
    assertNoSurfaceErrors(diagnostics, 'options');

    diagnostics.surface = 'content-script';
    await page.goto(`${fixture.origin}/fixture.html`, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(() => document.documentElement.dataset.hwpExtension === 'rhwp');
    await page.waitForSelector('.rhwp-badge');
    const contentState = await page.evaluate(() => ({
      marker: document.documentElement.dataset.hwpExtension,
      version: document.documentElement.dataset.hwpExtensionVersion,
      badgeCount: document.querySelectorAll('.rhwp-badge').length,
      processedCount: document.querySelectorAll('a[data-rhwp-processed="true"]').length,
    }));
    assert.equal(contentState.marker, 'rhwp');
    assert.match(contentState.version ?? '', /^\d+\.\d+\.\d+$/);
    assert.equal(contentState.badgeCount, 1);
    assert.equal(contentState.processedCount, 1);
    await assertPageBudget(browser, page, diagnostics);
    assertNoSurfaceErrors(diagnostics, 'content-script');

    assert.equal(diagnostics.errors.length, 0, formatDiagnostics(diagnostics));
    assert.equal(diagnostics.unexpectedPageTargets.length, 0, formatDiagnostics(diagnostics));
    process.stdout.write(`${prefix}extension=${extensionId} worker=${workerTarget.url()}\n`);
  } catch (error) {
    failure = new Error(`${error.message ?? error}\n${formatDiagnostics(diagnostics)}`, { cause: error });
  } finally {
    const cleanupErrors = [];
    if (workerSession) await workerSession.detach().catch(error => cleanupErrors.push(error));
    if (browser) await browser.close().catch(error => cleanupErrors.push(error));
    if (server) await closeServer(server).catch(error => cleanupErrors.push(error));
    await rm(temporaryRoot, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 })
      .catch(error => cleanupErrors.push(error));
    if (existsSync(temporaryRoot)) cleanupErrors.push(new Error(`임시 경로가 남았습니다: ${temporaryRoot}`));
    if (cleanupErrors.length > 0) {
      const cleanupFailure = new Error(`smoke 정리 실패:\n${cleanupErrors.map(String).join('\n')}`);
      failure = failure
        ? new AggregateError([failure, cleanupFailure], 'smoke 실행과 정리가 모두 실패했습니다.')
        : cleanupFailure;
    }
  }

  if (failure) throw failure;
}

function chromeArgs(fixtureOrigin) {
  const args = [
    '--disable-background-networking',
    '--disable-breakpad',
    '--disable-client-side-phishing-detection',
    '--disable-component-update',
    '--disable-default-apps',
    '--disable-domain-reliability',
    '--disable-features=AutofillServerCommunication,MediaRouter,OptimizationHints,Translate',
    '--disable-sync',
    '--metrics-recording-only',
    '--no-default-browser-check',
    '--no-first-run',
    `--proxy-server=${fixtureOrigin}`,
  ];
  if (typeof process.getuid === 'function' && process.getuid() === 0) {
    args.push('--no-sandbox', '--disable-setuid-sandbox');
  }
  return args;
}

async function startFixtureServer(diagnostics) {
  const fixtureBytes = await readFile(FIXTURE_FILE);
  const server = http.createServer((request, response) => {
    const host = request.headers.host ?? '';
    const isDirectLoopback = host.startsWith('127.0.0.1:') || host.startsWith('localhost:');
    if (!isDirectLoopback || /^https?:\/\//i.test(request.url ?? '')) {
      diagnostics.blockedProxyRequests.push(`${request.method} ${request.url}`);
      response.writeHead(502, { 'content-type': 'text/plain; charset=utf-8' });
      response.end('External network disabled by rhwp extension smoke.');
      return;
    }

    const requestUrl = new URL(request.url ?? '/', `http://${host}`);
    if (requestUrl.pathname === '/fixture.html') {
      response.writeHead(200, {
        'cache-control': 'no-store',
        'content-type': 'text/html; charset=utf-8',
      });
      response.end(`<!doctype html>
<html lang="ko">
<head><meta charset="utf-8"><link rel="icon" href="data:,"><title>rhwp extension smoke fixture</title></head>
<body><a id="document-link" href="/smoke.hwp">smoke.hwp</a></body>
</html>`);
      return;
    }
    if (requestUrl.pathname === '/smoke.hwp') {
      response.writeHead(200, {
        'access-control-allow-origin': '*',
        'cache-control': 'no-store',
        'content-length': fixtureBytes.byteLength,
        'content-type': 'application/x-hwp',
      });
      response.end(fixtureBytes);
      return;
    }
    if (requestUrl.pathname === '/favicon.ico') {
      response.writeHead(204, { 'cache-control': 'no-store' });
      response.end();
      return;
    }
    response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Not found');
  });
  server.on('connect', (request, socket) => {
    diagnostics.blockedProxyRequests.push(`CONNECT ${request.url}`);
    socket.end('HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n');
  });

  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      server.off('error', reject);
      resolve();
    });
  });
  const address = server.address();
  assert.ok(address && typeof address === 'object');
  return { server, origin: `http://127.0.0.1:${address.port}` };
}

function attachPageDiagnostics(page, diagnostics, fixtureOrigin) {
  page.on('console', message => {
    const entry = `[${diagnostics.surface}] ${message.type()}: ${message.text()}`;
    diagnostics.console.push(entry);
    if (message.type() === 'error') diagnostics.errors.push(entry);
  });
  page.on('pageerror', error => {
    diagnostics.errors.push(`[${diagnostics.surface}] pageerror: ${error.message ?? error}`);
  });
  page.on('dialog', async dialog => {
    diagnostics.errors.push(`[${diagnostics.surface}] unexpected dialog: ${dialog.type()} ${dialog.message()}`);
    await dialog.dismiss().catch(() => {});
  });
  page.on('request', request => {
    const url = request.url();
    if (isUnexpectedNetworkUrl(url, fixtureOrigin)) {
      diagnostics.errors.push(`[${diagnostics.surface}] unexpected network request: ${url}`);
    }
  });
  page.on('requestfailed', request => {
    const url = request.url();
    if (!isLocalBrowserUrl(url, fixtureOrigin, diagnostics.extensionId)) return;
    diagnostics.errors.push(
      `[${diagnostics.surface}] request failed: ${url} (${request.failure()?.errorText ?? 'unknown'})`,
    );
  });
  page.on('response', response => {
    const url = response.url();
    if (response.status() < 400 || !isLocalBrowserUrl(url, fixtureOrigin, diagnostics.extensionId)) return;
    diagnostics.errors.push(`[${diagnostics.surface}] HTTP ${response.status()}: ${url}`);
  });
}

function attachPageBudget(browser, ownedPage, diagnostics) {
  browser.on('targetcreated', target => {
    if (target.type() === 'page' && target !== ownedPage.target()) {
      diagnostics.unexpectedPageTargets.push(target.url() || '(blank page)');
    }
  });
}

async function attachWorkerDiagnostics(workerTarget, diagnostics, fixtureOrigin, extensionId) {
  const session = await workerTarget.createCDPSession();
  session.on('Runtime.exceptionThrown', event => {
    diagnostics.errors.push(
      `[service-worker] exception: ${event.exceptionDetails?.text ?? 'unknown exception'}`,
    );
  });
  session.on('Runtime.consoleAPICalled', event => {
    const message = event.args.map(arg => arg.value ?? arg.description ?? arg.type).join(' ');
    diagnostics.workerConsole.push(`${event.type}: ${message}`);
    if (event.type === 'error') diagnostics.errors.push(`[service-worker] error: ${message}`);
  });
  session.on('Log.entryAdded', event => {
    const entry = event.entry;
    diagnostics.workerConsole.push(`${entry.level}: ${entry.text}`);
    if (entry.level === 'error') diagnostics.errors.push(`[service-worker] error: ${entry.text}`);
  });
  session.on('Network.requestWillBeSent', event => {
    const url = event.request?.url ?? '';
    if (isUnexpectedNetworkUrl(url, fixtureOrigin)) {
      diagnostics.errors.push(`[service-worker] unexpected network request: ${url}`);
    }
  });
  session.on('Network.responseReceived', event => {
    const response = event.response;
    if (!response || response.status < 400) return;
    if (isLocalBrowserUrl(response.url, fixtureOrigin, extensionId)) {
      diagnostics.errors.push(`[service-worker] HTTP ${response.status}: ${response.url}`);
    }
  });
  await session.send('Runtime.enable');
  await session.send('Log.enable');
  await session.send('Network.enable');
  return session;
}

async function assertPageBudget(browser, ownedPage, diagnostics) {
  const currentPages = await browser.pages();
  const unexpected = currentPages.filter(page => page.target() !== ownedPage.target());
  assert.ok(currentPages.length <= MAX_PAGES, formatDiagnostics(diagnostics));
  assert.equal(unexpected.length, 0, formatDiagnostics(diagnostics));
  assert.equal(diagnostics.unexpectedPageTargets.length, 0, formatDiagnostics(diagnostics));
}

function assertNoSurfaceErrors(diagnostics, surface) {
  const errors = diagnostics.errors.filter(entry => entry.startsWith(`[${surface}]`));
  assert.equal(errors.length, 0, formatDiagnostics(diagnostics));
}

function isUnexpectedNetworkUrl(value, fixtureOrigin) {
  let url;
  try {
    url = new URL(value);
  } catch {
    return false;
  }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') return false;
  if (url.origin === fixtureOrigin) return false;
  return true;
}

function isLocalBrowserUrl(value, fixtureOrigin, extensionId) {
  let url;
  try {
    url = new URL(value);
  } catch {
    return false;
  }
  if (url.origin === fixtureOrigin) return true;
  return url.protocol === 'chrome-extension:' && (!extensionId || url.hostname === extensionId);
}

function createDiagnostics() {
  return {
    surface: 'launch',
    extensionId: null,
    workerUrl: null,
    console: [],
    workerConsole: [],
    errors: [],
    unexpectedPageTargets: [],
    blockedProxyRequests: [],
  };
}

function formatDiagnostics(diagnostics) {
  return [
    '--- extension smoke diagnostics ---',
    `surface=${diagnostics.surface}`,
    `extensionId=${diagnostics.extensionId ?? '(unknown)'}`,
    `worker=${diagnostics.workerUrl ?? '(unknown)'}`,
    `errors=${JSON.stringify(diagnostics.errors, null, 2)}`,
    `unexpectedPages=${JSON.stringify(diagnostics.unexpectedPageTargets, null, 2)}`,
    `blockedProxyRequests=${JSON.stringify(diagnostics.blockedProxyRequests, null, 2)}`,
    `workerConsole=${JSON.stringify(diagnostics.workerConsole.slice(-20), null, 2)}`,
    `pageConsole=${JSON.stringify(diagnostics.console.slice(-40), null, 2)}`,
  ].join('\n');
}

function positiveInteger(value, fallback, name) {
  if (value == null || value === '') return fallback;
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) throw new Error(`${name}은 양의 정수여야 합니다.`);
  return parsed;
}

async function closeServer(server) {
  await new Promise((resolve, reject) => {
    server.close(error => error ? reject(error) : resolve());
    server.closeAllConnections?.();
  });
}
