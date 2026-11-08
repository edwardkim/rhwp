// #8614 비시각 계약: 실제 네트워크 미응답에서 초기화·문서 열기·후속 편집의 완료를 검사한다.
// 합성 문서와 대체 테스트 글꼴은 수명 계약용이며 독립 한컴 출력의 외형 정답지가 아니다.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import puppeteer from 'puppeteer-core';
import { createServer } from 'vite';

const root = resolve(import.meta.dirname, '..');
const output = resolve(root, '../output/pr-review/web-font-deadline/browser');
await mkdir(output, { recursive: true });
const profile = await mkdtemp(output + '/profile-');
// 외장 볼륨으로 라우팅된 pkg도 Vite가 같은 빌드로 제공하도록 실제 경로를 허용한다.
const wasmDirectory = await realpath(resolve(root, '../pkg'));
const server = await createServer({ root, cacheDir: output + '/vite-cache',
  server: { host: '127.0.0.1', port: 0, hmr: false, fs: { allow: [wasmDirectory] } } });
// 저장소에 라이선스 고지가 있는 테스트 공급. 원래 CDN 서체의 외형을 주장하지 않는다.
const fontBytes = await readFile(resolve(root, 'public/fonts/NotoSerifKR-Regular.woff2'));
const wasmHash = createHash('sha256').update(await readFile(resolve(root, '../pkg/rhwp_bg.wasm'))).digest('hex');
const results = [];
const scenarios = ['success', 'failure', 'network-pending', 'late-success', 'late-failure', 'disabled', 'local-pending'];
const selected = process.argv.find(arg => arg.startsWith('--scenario='))?.split('=')[1];
if (selected) assert.ok(scenarios.includes(selected), 'unknown scenario: ' + selected);
let browser;
try {
  await server.listen();
  browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH,
    headless: true, userDataDir: profile, args: ['--no-sandbox'] });
  const url = server.resolvedUrls.local[0];
  for (const mode of selected ? [selected] : scenarios) {
    const context = await browser.createBrowserContext(), page = await context.newPage();
    page.setDefaultTimeout(30_000);
    await page.setViewport({ width: 1280, height: 900, deviceScaleFactor: 1 });
    const held = [], errors = [], network = [], cancelled = new Set();
    page.on('requestfailed', request => cancelled.add(request.url()));
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => {
      if (message.text().includes('[FontLoader]') || message.type() === 'error') console.log(mode, message.text());
    });
    await page.setRequestInterception(true);
    page.on('request', request => {
      const external = request.url().startsWith('https://cdn.jsdelivr.net/');
      const local = request.url().endsWith('/fonts/RHWPOldHangul-Regular.woff2');
      if (!external && !(local && mode === 'local-pending')) { void request.continue(); return; }
      network.push(request.url());
      if (mode === 'network-pending' || mode === 'local-pending') held.push(request);
      else void request.respond({ status: mode === 'failure' ? 404 : 200,
        headers: { 'access-control-allow-origin': '*' }, contentType: 'font/woff2', body: fontBytes });
    });
    await page.evaluateOnNewDocument(mode => {
      window.__fontDeadlineFaces = [];
      window.__wasmResponseHashes = [];
      const fetch = window.fetch;
      window.fetch = async (...args) => {
        const response = await fetch(...args);
        if (new URL(response.url).pathname.endsWith('/rhwp_bg.wasm')) {
          // 초기화가 실제 소비하는 response의 복사본을 해시한다. 나중의 별도 fetch가 아니다.
          window.__wasmResponseHashes.push(response.clone().arrayBuffer()
            .then(bytes => crypto.subtle.digest('SHA-256', bytes))
            .then(bytes => Array.from(new Uint8Array(bytes), b => b.toString(16).padStart(2, '0')).join('')));
        }
        return response;
      };
      // OS 탐지 probe만 동일 폭으로 고정한다. 실제 Canvas/본문 측정은 그대로 둔다.
      const width = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetWidth');
      Object.defineProperty(HTMLElement.prototype, 'offsetWidth', { ...width, get() {
        if (this.tagName === 'SPAN' && this.textContent === 'mmmmmmmmmwwwwwwwWMWMWM한글글꼴측정0123456789') return 100;
        return width.get.call(this);
      } });
      document.fonts.check = () => false;
      const NativeFace = window.FontFace;
      window.FontFace = class extends NativeFace {
        constructor(family, source, descriptors) {
          super(family, source, descriptors);
          window.__fontDeadlineFaces.push(this);
          if (mode.startsWith('late-') && ['함초롬바탕', '함초롬돋움', 'Roboto'].includes(family)) {
            this.load = () => new Promise((resolve, reject) => { this.release = () => {
              if (mode === 'late-failure') { reject(new Error('late decode failure')); return; }
              return NativeFace.prototype.load.call(this).then(resolve, reject);
            }; });
          }
        }
      };
      if (mode === 'disabled' || mode === 'local-pending') {
        window.browser = { storage: { sync: { get: async () => ({ disableExternalWebFonts: true }) } } };
      }
    }, mode);
    const start = Date.now();
    await page.goto(url + '?renderer=canvas2d', { waitUntil: 'domcontentloaded' });
    try {
      await page.waitForFunction(() => window.__inputHandler?.isActive() && window.rhwpCommandErrors?.getState()?.status === 'ready');
    } catch (error) {
      console.log(mode, 'startup incomplete', await page.evaluate(() => ({
        text: document.body.innerText.slice(-1000), state: window.rhwpCommandErrors?.getState(),
        fontSetStatus: document.fonts.status, css: document.getElementById('rhwp-web-font-faces')?.textContent,
        faces: Array.from(document.fonts).map(face => ({ family: face.family, status: face.status })),
      })));
      throw error;
    }
    const startupMs = Date.now() - start;
    const opened = await page.evaluate(async () => {
      const w = window.__wasm, text = 'Font deadline 한글 문서';
      w.insertText(0, 0, 0, text);
      w.applyCharFormat(0, 0, 0, text.length, JSON.stringify({ fontId: w.findOrCreateFontId('Roboto') }));
      const data = w.exportHwp(), channel = new MessageChannel();
      const connected = new Promise(resolve => { channel.port1.onmessage = e => resolve(e.data); });
      channel.port1.start();
      window.postMessage({ type: 'rhwp-connect', version: 1, sessionId: 'font-deadline',
        capabilities: ['transferable-array-buffer'] }, location.origin, [channel.port2]);
      await connected;
      const before = window.__fontDeadlineFaces.length, start = performance.now();
      const loaded = new Promise((resolve, reject) => { channel.port1.onmessage = e => {
        if (e.data.id !== 1) return;
        if (e.data.error) reject(new Error(JSON.stringify(e.data.error))); else resolve(e.data.result);
      } });
      // 실제 공개 loadFile → loadBytes → initializeDocument 경로.
      channel.port1.postMessage({ type: 'rhwp-request', version: 1, sessionId: 'font-deadline', id: 1,
        method: 'loadFile', params: { data, fileName: 'font-deadline.hwp', skipUnsavedGuard: true, suppressDialogs: true } });
      const result = await loaded;
      channel.port1.close();
      return { elapsedMs: performance.now() - start, result, before, after: window.__fontDeadlineFaces.length,
        found: w.searchAllText(text, true).length, active: window.__inputHandler.isActive() };
    });
    assert.equal(opened.found, 1); assert.equal(opened.active, true); assert.ok(opened.result.pageCount > 0);
    if (!['disabled', 'local-pending', 'network-pending', 'failure'].includes(mode)) {
      assert.ok(opened.after > opened.before, 'document-only font is requested');
    }
    if (!['disabled', 'local-pending'].includes(mode)) assert.ok(network.some(url => url.includes('roboto')), 'document-only resource is requested');
    await page.waitForSelector('.document-page-canvas');
    const snapshot = async () => page.evaluate(async mode => {
      const loader = await import('/src/core/font-loader.ts');
      const count = window.__fontDeadlineFaces.length;
      await loader.loadWebFonts(['Roboto'], undefined, { disableExternalWebFonts: ['disabled', 'local-pending'].includes(mode) });
      await document.fonts.ready;
      const ctx = document.createElement('canvas').getContext('2d');
      ctx.font = '20px "Roboto", serif';
      return { count, repeated: window.__fontDeadlineFaces.length,
        supply: loader.getWebFontSupplySnapshot('Roboto').status,
        critical: loader.getWebFontSupplySnapshot('함초롬바탕').status,
        local: loader.getWebFontSupplySnapshot('RHWP Old Hangul').status,
        metric: ctx.measureText('Font deadline 한글 문서').width,
        added: window.__fontDeadlineFaces.filter(face => document.fonts.has(face)).map(face => face.family),
        tree: JSON.stringify(window.__wasm.getPageLayerTreeObject(0)) };
    }, mode);
    const before = await snapshot();
    assert.equal(before.count, before.repeated, 'failed files are not retried');
    assert.equal(before.supply, mode === 'success' ? 'loaded' : 'catalogued');
    assert.equal(before.critical, mode === 'success' ? 'loaded' : 'catalogued');
    assert.equal(before.local, mode === 'local-pending' ? 'catalogued' : 'loaded');
    const canvas = await page.$('.document-page-canvas');
    const clip = await canvas.evaluate(element => {
      const page = element.getBoundingClientRect(), viewport = document.getElementById('scroll-container').getBoundingClientRect();
      const x = Math.max(page.left, viewport.left, 0), y = Math.max(page.top, viewport.top, 0);
      return { x, y, width: Math.min(page.right, viewport.right, innerWidth) - x,
        height: Math.min(page.bottom, viewport.bottom, innerHeight) - y };
    });
    await page.screenshot({ path: output + '/' + mode + '-before.png', clip, captureBeyondViewport: false });
    const canvasBefore = await canvas.evaluate(element => element.toDataURL());
    if (mode === 'disabled') assert.equal(network.length, 0);
    if (mode === 'network-pending' || mode === 'local-pending') assert.ok(held.length > 0);
    for (const request of held) {
      assert.ok(cancelled.has(request.url()), 'deadline must cancel the actual network request');
    }
    // 원 load Promise가 정산된 뒤 늦은 등록·재측정·paint가 없는지 확인한다.
    await page.evaluate(async () => {
      await Promise.all(window.__fontDeadlineFaces.map(face => face.release?.()));
      await Promise.allSettled(window.__fontDeadlineFaces.map(face => face.loaded));
      await document.fonts.ready;
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    });
    const after = await snapshot();
    assert.deepEqual(after, before, 'late response must not change supply, metrics or layout');
    assert.equal(await canvas.evaluate(element => element.toDataURL()), canvasBefore, 'late response must not repaint content');
    await page.screenshot({ path: output + '/' + mode + '-after.png', clip, captureBeyondViewport: false });
    const edit = await page.evaluate(() => {
      const ih = window.__inputHandler, w = window.__wasm, bytes = Array.from(w.exportHwp());
      ih.executeOperation({ kind: 'snapshot', operationType: 'font-deadline', operation(wasm) {
        wasm.insertText(0, 0, 0, 'NEXT'); return { sectionIndex: 0, paragraphIndex: 0, charOffset: 4 };
      } });
      const inserted = w.searchAllText('NEXT', true).length;
      ih.handleUndo();
      return { inserted, restored: JSON.stringify(bytes) === JSON.stringify(Array.from(w.exportHwp())) };
    });
    assert.deepEqual(edit, { inserted: 1, restored: true });
    assert.deepEqual(errors, []);
    const responseHashes = await page.evaluate(() => Promise.all(window.__wasmResponseHashes));
    assert.ok(responseHashes.length > 0, 'observe the WASM loaded during startup');
    assert.ok(responseHashes.every(hash => hash === wasmHash), JSON.stringify(responseHashes));
    results.push({ mode, startupMs, opened, requests: network.length, held: held.length,
      before: { ...before, tree: undefined }, after: { ...after, tree: undefined }, edit, wasmHash,
      canvasHash: createHash('sha256').update(canvasBefore).digest('hex'),
      treeHash: createHash('sha256').update(before.tree).digest('hex') });
    console.log(JSON.stringify(results.at(-1)));
    await writeFile(output + '/results.json', JSON.stringify(results, null, 2));
    await context.close();
  }
} finally {
  await browser?.close();
  await server.close();
  await rm(profile, { recursive: true, force: true });
}
