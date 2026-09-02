#!/usr/bin/env node

import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer';

const CURRENT_FILE = fileURLToPath(import.meta.url);
const HERE = path.dirname(CURRENT_FILE);
const EXTENSION_DIR = path.resolve(HERE, '..');
const ROOT = path.resolve(EXTENSION_DIR, '..');
const DIST_DIR = path.join(EXTENSION_DIR, 'dist');
const HWP_FIXTURE_FILE = path.join(ROOT, 'samples', 'hwp3-pagedef-1915.hwp');
const TIMEOUT_MS = positiveInteger(
  process.env.RHWP_EXTENSION_DOWNLOAD_TIMEOUT_MS,
  30_000,
  'RHWP_EXTENSION_DOWNLOAD_TIMEOUT_MS',
);
const SETTINGS = Object.freeze({
  autoOpen: true,
  showBadges: false,
  hoverPreview: false,
  disableExternalWebFonts: true,
});

const ALL_CASES = Object.freeze([
  {
    id: 'normal-xlsx',
    pathname: '/normal.xlsx',
    suggestedFilename: 'normal.xlsx',
    expectedViewerCount: 0,
  },
  {
    id: 'misleading-hwp-url',
    pathname: '/misleading.hwp',
    suggestedFilename: 'public-report.xlsx',
    expectedViewerCount: 0,
  },
  {
    id: 'confirmed-hwp',
    pathname: '/document.hwp',
    suggestedFilename: 'document.hwp',
    expectedViewerCount: 1,
  },
  {
    id: 'extensionless-hwp',
    pathname: '/download?id=extensionless',
    suggestedFilename: 'extensionless',
    expectedViewerCount: 1,
  },
]);
const CASES = selectCases(ALL_CASES, process.env.RHWP_EXTENSION_DOWNLOAD_CASE);

if (process.argv[1] && path.resolve(process.argv[1]) === CURRENT_FILE) {
  await main();
}

async function main() {
  assert.ok(existsSync(path.join(DIST_DIR, 'manifest.json')), '먼저 rhwp-chrome dist를 빌드해야 합니다.');
  assert.ok(existsSync(HWP_FIXTURE_FILE), `HWP fixture가 없습니다: ${HWP_FIXTURE_FILE}`);

  const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), 'rhwp-download-e2e-'));
  const userDataDir = path.join(temporaryRoot, 'profile');
  const downloadDir = path.join(temporaryRoot, 'downloads');
  const downloads = createDownloadDiagnostics();
  let browser = null;
  let browserSession = null;
  let server = null;
  let failure = null;

  await mkdir(userDataDir, { recursive: true });
  await mkdir(downloadDir, { recursive: true });

  try {
    const fixture = await startFixtureServer();
    server = fixture.server;

    browser = await puppeteer.launch({
      headless: true,
      enableExtensions: [DIST_DIR],
      userDataDir,
      args: chromeArgs(fixture.origin),
    });

    const pages = await browser.pages();
    const fixturePage = pages[0] ?? await browser.newPage();
    fixturePage.setDefaultTimeout(TIMEOUT_MS);
    fixturePage.setDefaultNavigationTimeout(TIMEOUT_MS);

    browserSession = await browser.target().createCDPSession();
    browserSession.on('Browser.downloadWillBegin', event => {
      downloads.begun.push(event);
    });
    browserSession.on('Browser.downloadProgress', event => {
      downloads.progress.set(event.guid, event);
    });
    await browserSession.send('Browser.setDownloadBehavior', {
      behavior: 'allow',
      downloadPath: downloadDir,
      eventsEnabled: true,
    });

    const workerTarget = await browser.waitForTarget(
      target => target.type() === 'service_worker'
        && target.url().startsWith('chrome-extension://')
        && target.url().endsWith('/background.js'),
      { timeout: TIMEOUT_MS },
    );
    const extensionId = new URL(workerTarget.url()).hostname;
    assert.match(extensionId, /^[a-p]{32}$/, `잘못된 extension ID: ${extensionId}`);
    const worker = await workerTarget.worker();
    assert.ok(worker, 'MV3 service worker 실행 컨텍스트를 얻지 못했습니다.');
    await worker.evaluate(async settings => chrome.storage.sync.set(settings), SETTINGS);

    await fixturePage.goto(`${fixture.origin}/fixture.html`, { waitUntil: 'domcontentloaded' });

    const results = [];
    for (const testCase of CASES) {
      process.stdout.write(`START ${testCase.id}\n`);
      const result = await runDownloadCase({
        browser,
        downloads,
        downloadDir,
        extensionId,
        fixtureOrigin: fixture.origin,
        fixturePage,
        testCase,
        worker,
      });
      results.push(result);
      process.stdout.write(
        `PASS ${testCase.id}: guid=${result.guid} file=${result.suggestedFilename} viewerTabs=${result.viewerCount}\n`,
      );
    }

    assert.deepEqual(
      results.map(result => result.viewerCount),
      CASES.map(testCase => testCase.expectedViewerCount),
    );
    if (CASES.length === ALL_CASES.length) {
      assert.equal(
        results.reduce((sum, result) => sum + result.viewerCount, 0),
        2,
        '네 다운로드에서 HWP 두 건에만 viewer 탭이 생성되어야 합니다.',
      );
      process.stdout.write('PASS: XLSX 2건 탭 0, HWP 2건 download id별 탭 1\n');
    }
  } catch (error) {
    failure = new Error(`${error.message ?? error}\n${formatDownloadDiagnostics(downloads)}`, { cause: error });
  } finally {
    const cleanupErrors = [];
    if (browserSession) {
      await cleanupWithin(browserSession.detach(), 'CDP session detach').catch(error => cleanupErrors.push(error));
    }
    if (browser) {
      await cleanupWithin(browser.close(), 'Chrome close').catch(error => cleanupErrors.push(error));
    }
    if (server) {
      await cleanupWithin(closeServer(server), 'fixture server close').catch(error => cleanupErrors.push(error));
    }
    await rm(temporaryRoot, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 })
      .catch(error => cleanupErrors.push(error));
    if (existsSync(temporaryRoot)) cleanupErrors.push(new Error(`임시 경로가 남았습니다: ${temporaryRoot}`));
    if (cleanupErrors.length > 0) {
      const cleanupFailure = new Error(`download E2E 정리 실패:\n${cleanupErrors.map(String).join('\n')}`);
      failure = failure
        ? new AggregateError([failure, cleanupFailure], 'download E2E 실행과 정리가 모두 실패했습니다.')
        : cleanupFailure;
    }
  }

  if (failure) throw failure;
}

async function runDownloadCase({
  browser,
  downloads,
  downloadDir,
  extensionId,
  fixtureOrigin,
  fixturePage,
  testCase,
  worker,
}) {
  const endpoint = new URL(testCase.pathname, fixtureOrigin).href;
  const beforeBegun = downloads.begun.length;
  const beforeViewerUrls = getViewerUrls(browser, extensionId);

  await fixturePage.bringToFront();
  await operationWithin(fixturePage.click(`#${testCase.id}`), `${testCase.id} click`);

  const begun = await waitUntil(() => downloads.begun
    .slice(beforeBegun)
    .find(event => event.url === endpoint));
  process.stdout.write(
    `BEGIN ${testCase.id}: guid=${begun.guid} suggestedFilename=${begun.suggestedFilename}\n`,
  );
  assert.equal(begun.suggestedFilename, testCase.suggestedFilename);

  await waitUntil(() => downloads.progress.get(begun.guid)?.state === 'completed');
  const downloadedPath = path.join(downloadDir, begun.suggestedFilename);
  const downloadedStat = await waitUntil(async () => {
    try {
      return await stat(downloadedPath);
    } catch (error) {
      if (error?.code === 'ENOENT') return null;
      throw error;
    }
  });
  assert.ok(downloadedStat.isFile());
  assert.ok(downloadedStat.size > 0, `다운로드 파일이 비어 있습니다: ${begun.suggestedFilename}`);

  if (testCase.expectedViewerCount > 0) {
    await waitUntil(() => viewerUrlsForEndpoint(browser, extensionId, endpoint).length
      === testCase.expectedViewerCount);
  }
  await delay(750);

  const afterViewerUrls = getViewerUrls(browser, extensionId);
  const newViewerUrls = afterViewerUrls.filter(url => !beforeViewerUrls.includes(url));
  const matchingViewerUrls = viewerUrlsForEndpoint(browser, extensionId, endpoint);
  assert.equal(
    matchingViewerUrls.length,
    testCase.expectedViewerCount,
    `${testCase.id}의 viewer 탭 수가 예상과 다릅니다: ${JSON.stringify(afterViewerUrls)}`,
  );
  assert.equal(
    newViewerUrls.length,
    testCase.expectedViewerCount,
    `${testCase.id}가 다른 viewer 탭을 만들었습니다: ${JSON.stringify(newViewerUrls)}`,
  );

  const downloadItems = await worker.evaluate(async url => {
    const items = await chrome.downloads.search({});
    return items.filter(item => item.url === url).map(item => ({
      id: item.id,
      exists: item.exists,
      filename: item.filename,
      state: item.state,
      url: item.url,
    }));
  }, endpoint);
  assert.equal(downloadItems.length, 1, `${testCase.id}의 Chrome download id가 하나여야 합니다.`);
  assert.equal(downloadItems[0].state, 'complete');
  assert.equal(downloadItems[0].exists, true);

  return {
    downloadId: downloadItems[0].id,
    guid: begun.guid,
    suggestedFilename: begun.suggestedFilename,
    viewerCount: matchingViewerUrls.length,
  };
}

function getViewerUrls(browser, extensionId) {
  const prefix = `chrome-extension://${extensionId}/viewer.html`;
  return browser.targets()
    .filter(target => target.type() === 'page' && target.url().startsWith(prefix))
    .map(target => target.url());
}

function viewerUrlsForEndpoint(browser, extensionId, endpoint) {
  return getViewerUrls(browser, extensionId).filter(value => {
    const viewerUrl = new URL(value);
    return viewerUrl.searchParams.get('url') === endpoint;
  });
}

async function startFixtureServer() {
  const hwpBytes = await readFile(HWP_FIXTURE_FILE);
  const xlsxBytes = Buffer.from('PK\x03\x04rhwp-xlsx-download-fixture', 'latin1');
  const server = http.createServer((request, response) => {
    const host = request.headers.host ?? '';
    const isDirectLoopback = host.startsWith('127.0.0.1:') || host.startsWith('localhost:');
    if (!isDirectLoopback || /^https?:\/\//i.test(request.url ?? '')) {
      response.writeHead(502, { 'content-type': 'text/plain; charset=utf-8' });
      response.end('External network disabled by rhwp download E2E.');
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
<head><meta charset="utf-8"><link rel="icon" href="data:,"><title>#6534 download fixture</title></head>
<body>
  <a id="normal-xlsx" href="/normal.xlsx">normal XLSX</a>
  <a id="misleading-hwp-url" href="/misleading.hwp">misleading HWP URL</a>
  <a id="confirmed-hwp" href="/document.hwp">confirmed HWP</a>
  <a id="extensionless-hwp" href="/download?id=extensionless">extensionless HWP</a>
</body>
</html>`);
      return;
    }

    const responseSpec = responseFor(requestUrl, hwpBytes, xlsxBytes);
    if (responseSpec) {
      response.writeHead(200, {
        'cache-control': 'no-store',
        'content-disposition': `attachment; filename="${responseSpec.filename}"`,
        'content-length': responseSpec.body.byteLength,
        'content-type': responseSpec.contentType,
      });
      response.end(responseSpec.body);
      return;
    }

    response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Not found');
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

function responseFor(requestUrl, hwpBytes, xlsxBytes) {
  if (requestUrl.pathname === '/normal.xlsx') {
    return {
      body: xlsxBytes,
      contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      filename: 'normal.xlsx',
    };
  }
  if (requestUrl.pathname === '/misleading.hwp') {
    return {
      body: xlsxBytes,
      contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      filename: 'public-report.xlsx',
    };
  }
  if (requestUrl.pathname === '/document.hwp') {
    return { body: hwpBytes, contentType: 'application/x-hwp', filename: 'document.hwp' };
  }
  if (requestUrl.pathname === '/download' && requestUrl.searchParams.get('id') === 'extensionless') {
    return { body: hwpBytes, contentType: 'application/x-hwp', filename: 'extensionless' };
  }
  return null;
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

function createDownloadDiagnostics() {
  return { begun: [], progress: new Map() };
}

function formatDownloadDiagnostics(downloads) {
  return [
    '--- download E2E diagnostics ---',
    `begun=${JSON.stringify(downloads.begun, null, 2)}`,
    `progress=${JSON.stringify([...downloads.progress.values()], null, 2)}`,
  ].join('\n');
}

async function waitUntil(probe) {
  const deadline = Date.now() + TIMEOUT_MS;
  let lastError = null;
  while (Date.now() < deadline) {
    try {
      const value = await probe();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await delay(50);
  }
  if (lastError) throw lastError;
  throw new Error(`조건이 ${TIMEOUT_MS}ms 안에 충족되지 않았습니다.`);
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function positiveInteger(value, fallback, name) {
  if (value == null || value === '') return fallback;
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) throw new Error(`${name}은 양의 정수여야 합니다.`);
  return parsed;
}

function selectCases(cases, selectedId) {
  if (!selectedId) return cases;
  const selected = cases.filter(testCase => testCase.id === selectedId);
  if (selected.length !== 1) {
    throw new Error(`RHWP_EXTENSION_DOWNLOAD_CASE가 알려진 사례여야 합니다: ${selectedId}`);
  }
  return Object.freeze(selected);
}

async function cleanupWithin(operation, label) {
  return operationWithin(operation, label, 5_000);
}

async function operationWithin(operation, label, timeoutMs = TIMEOUT_MS) {
  let timeoutId;
  try {
    return await Promise.race([
      operation,
      new Promise((_, reject) => {
        timeoutId = setTimeout(() => reject(new Error(`${label} timeout`)), timeoutMs);
      }),
    ]);
  } finally {
    clearTimeout(timeoutId);
  }
}

async function closeServer(server) {
  await new Promise((resolve, reject) => {
    server.close(error => error ? reject(error) : resolve());
    server.closeAllConnections?.();
  });
}
