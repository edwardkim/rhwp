import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { loadApp, runTest } from './helpers.mjs';

const a = await readFile(new URL('../../samples/issue8639/trailing_space_overflow_synthetic-hancom-2024.hwp', import.meta.url));
const b = await readFile(new URL('../../samples/issue7418/condense_cell_synthetic.hwpx', import.meta.url));

async function open(page, name, bytes, mode = 'ordinary') {
  await page.evaluate(({ name, data, mode }) => {
    const probe = window.openProbe;
    probe.mode = mode;
    probe.entered = false;
    if (mode === 'held') probe.obsoleteFinished = false;
    const files = new DataTransfer();
    files.items.add(new File([Uint8Array.from(atob(data), c => c.charCodeAt(0))], name));
    const input = document.querySelector('#file-input');
    input.files = files.files;
    input.dispatchEvent(new Event('change', { bubbles: true }));
  }, { name, data: bytes.toString('base64'), mode });
}

async function success(page, name) {
  await page.waitForFunction(name => window.openProbe.results.some(r => r.name === name), {}, name);
  const result = await page.evaluate(name => window.openProbe.results.find(r => r.name === name), name);
  assert.equal(result.document, name);
  assert.equal(result.loaded, 1, '현재 문서의 첫 화면 완료 이벤트가 성공 표시보다 먼저 온다');
  assert.ok(result.ink > 0, '성공 시점에 첫 페이지 내용이 보인다');
  return result;
}

await runTest('최초 페이지 렌더링과 글꼴 준비 경합 (#9164)', async ({ page }) => {
  await loadApp(page, '?renderer=canvas2d');
  await page.evaluate(async () => {
    const view = window.__canvasView;
    const wasm = window.__wasm;
    const { setHostFontProvider } = await import('/src/core/local-fonts.ts');
    const probe = window.openProbe = { mode: 'ordinary', entered: false, results: [], loaded: new Map() };
    window.__eventBus.on('document-view-loaded', () => {
      const key = wasm.documentGeneration;
      probe.loaded.set(key, (probe.loaded.get(key) ?? 0) + 1);
    });
    const originalLoad = view.loadDocument;
    view.loadDocument = function (...args) {
      const mode = probe.mode;
      probe.mode = 'ordinary';
      const prepare = wasm.prepareCanvasMetrics;
      wasm.prepareCanvasMetrics = async function (...args) {
        wasm.prepareCanvasMetrics = prepare;
        probe.entered = true;
        if (mode === 'failure') throw new Error('합성 metrics 준비 실패');
        if (mode === 'network') {
          const face = new FontFace('Issue9164', 'url(/fonts/GowunDodum-Regular.woff2?issue=9164)');
          document.fonts.add(face);
          document.fonts.addEventListener('loadingdone', event => {
            if (event.isTrusted && event.fontfaces.includes(face)) probe.fontDelivered = true;
          }, { once: true });
          await face.load();
        }
        if (mode === 'host') {
          void setHostFontProvider({ getSnapshot: async () => ({ revision: 'test', faces: [] }),
            subscribe: () => () => {}, readFace: async () => { throw Error('빈 글꼴 목록'); } });
        }
        if (mode === 'font' || mode === 'burst') {
          queueMicrotask(() => document.fonts.dispatchEvent(new Event('loadingdone')));
          if (mode === 'burst') queueMicrotask(() => document.fonts.dispatchEvent(new Event('loadingdone')));
        }
        if (mode === 'held' || mode === 'burst' || mode === 'network') {
          await new Promise(resolve => { probe.release = resolve; });
        }
        try {
          return await prepare.apply(this, args);
        } finally {
          if (mode === 'held') probe.obsoleteFinished = true;
        }
      };
      return originalLoad.apply(this, args);
    };
    probe.firstPage = () => {
      const canvas = document.querySelector('.document-page-canvas');
      let ink = 0;
      let hash = 0;
      if (canvas) {
        const pixels = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
        for (let i = 0; i < pixels.length; i += 4) {
          if (pixels[i + 3] && pixels[i] < 160) {
            ink++;
            hash = (Math.imul(hash, 31) + i) >>> 0;
          }
        }
      }
      return { ink, hash };
    };
    const status = document.querySelector('#sb-message');
    new MutationObserver(() => {
      const name = status.textContent.match(/^(case-[\w-]+\.hwpx?) — /)?.[1];
      if (!name) return;
      probe.results.push({ name, document: wasm.fileName, loaded: probe.loaded.get(wasm.documentGeneration) ?? 0,
        fallback: window.__renderBackendFallbackReason, ...probe.firstPage() });
    }).observe(status, { childList: true, subtree: true });
    // 합성 문서도 설치 글꼴 확인을 제안할 수 있다. 웹 대체 글꼴을 선택한다.
    new MutationObserver(() => {
      document.querySelector('.modal-overlay details')?.closest('.dialog-wrap')
        ?.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)')?.click();
    }).observe(document.body, { subtree: true, childList: true });
  });
  await open(page, 'case-a.hwp', a);
  const normal = await success(page, 'case-a.hwp');
  await page.setRequestInterception(true);
  const interceptedFont = Promise.withResolvers();
  page.on('request', request => {
    if (request.url().endsWith('GowunDodum-Regular.woff2?issue=9164')) interceptedFont.resolve(request);
    else void request.continue();
  });
  for (const mode of ['font', 'burst', 'network', 'host', 'failure']) {
    const name = 'case-' + mode + '.hwp';
    await open(page, name, a, mode);
    if (mode === 'network') {
      await page.waitForFunction(() => document.fonts.status === 'loading' && window.openProbe.entered);
      const request = await interceptedFont.promise;
      assert.equal(await page.evaluate(() => window.openProbe.results.some(r => r.name === 'case-network.hwp')), false);
      await request.continue();
    }
    const result = await success(page, name);
    if (mode === 'failure') assert.equal(result.fallback, 'rendererPreparationFailed', '준비 실패는 portable 렌더러로 복구한다');
    else assert.equal(result.hash, normal.hash, '재열기의 첫 페이지는 정상 대조군과 일치한다');
    if (mode === 'network') assert.equal(await page.evaluate(() => window.openProbe.fontDelivered), true);
    if (mode === 'burst' || mode === 'network') await page.evaluate(() => window.openProbe.release());
    console.log(mode, result.ink, result.fallback);
  }
  // 같은 이름·같은 바이트를 다시 열어도 새 문서 세대에서 첫 화면을 완성한다.
  await page.evaluate(() => { window.openProbe.results.length = 0; });
  await open(page, 'case-a.hwp', a);
  assert.equal((await success(page, 'case-a.hwp')).hash, normal.hash);
  await open(page, 'case-reference.hwpx', b);
  const reference = await success(page, 'case-reference.hwpx');
  assert.notEqual(reference.hash, normal.hash);
  await open(page, 'case-obsolete.hwp', a, 'held');
  await page.waitForFunction(() => window.openProbe.entered);
  await open(page, 'case-current.hwpx', b);
  const replacement = await success(page, 'case-current.hwpx');
  await page.evaluate(() => window.openProbe.release());
  await page.waitForFunction(() => window.openProbe.obsoleteFinished && !document.documentElement.classList.contains('rhwp-busy'));
  assert.equal(replacement.hash, reference.hash);
  assert.equal(await page.evaluate(() => window.openProbe.results.some(r => r.name === 'case-obsolete.hwp')), false);
  const afterRelease = await page.evaluate(() => ({
    document: window.__wasm.fileName, hash: window.openProbe.firstPage().hash,
    loaded: window.openProbe.loaded.get(window.__wasm.documentGeneration),
  }));
  assert.deepEqual(afterRelease, { document: 'case-current.hwpx', loaded: 1, hash: reference.hash });
  assert.match(await page.$eval('#sb-message', e => e.textContent), /^case-current\.hwpx/);
  await open(page, 'case-retained.hwp', a, 'held');
  await page.waitForFunction(() => window.openProbe.entered);
  await open(page, 'case-invalid.hwp', Buffer.from('invalid synthetic document'));
  await page.waitForFunction(() => document.querySelector('#sb-message').textContent.includes('UNSUPPORTED_FILE_FORMAT'));
  const failureStatus = await page.$eval('#sb-message', e => e.textContent);
  await page.evaluate(() => window.openProbe.release());
  await page.waitForFunction(() => window.openProbe.obsoleteFinished
    && !document.documentElement.classList.contains('rhwp-busy')
    && window.openProbe.loaded.has(window.__wasm.documentGeneration));
  assert.equal(await page.$eval('#sb-message', e => e.textContent), failureStatus);
  assert.equal(await page.evaluate(() => window.__wasm.fileName), 'case-retained.hwp');
  assert.equal(await page.evaluate(() => window.openProbe.firstPage().hash), normal.hash);
  assert.equal(await page.evaluate(() => window.__inputHandler.isActive()), true, '복원한 문서의 편집 상태');
  await page.keyboard.type('recovered-input');
  assert.ok(await page.evaluate(() => window.__wasm.doc.getTextFileText().includes('recovered-input')));
  await page.evaluate(() => { document.querySelector('#file-input').dataset.skipUnsavedGuard = 'true'; });
  await open(page, 'case-fresh.hwpx', b);
  await success(page, 'case-fresh.hwpx');
  // A 복구의 글꼴 준비 도중 C를 열면 뒤늦은 복구가 C를 덮지 않는다.
  await open(page, 'case-interrupted.hwp', a, 'held');
  await page.waitForFunction(() => window.openProbe.entered);
  await page.evaluate(() => { window.openProbe.releaseOriginal = window.openProbe.release; });
  await open(page, 'case-broken.hwp', Buffer.from('invalid synthetic document'), 'held');
  await page.waitForFunction(() => window.openProbe.entered && window.openProbe.release !== window.openProbe.releaseOriginal);
  await open(page, 'case-third.hwpx', b);
  assert.equal((await success(page, 'case-third.hwpx')).hash, reference.hash);
  await page.evaluate(() => { window.openProbe.releaseOriginal(); window.openProbe.release(); });
  await page.waitForFunction(() => !document.documentElement.classList.contains('rhwp-busy'));
  assert.equal(await page.evaluate(() => window.__wasm.fileName), 'case-third.hwpx');
  assert.match(await page.$eval('#sb-message', e => e.textContent), /^case-third\.hwpx/);
  // 복구할 수 없는 paint 실패는 성공으로 보고되지 않아야 한다.
  await page.evaluate(() => {
    window.__canvasView.pageRenderer.renderPage = () => { throw Error('합성 paint 실패'); };
  });
  await open(page, 'case-no-paint.hwp', a);
  await page.waitForFunction(() => document.querySelector('#sb-message').textContent.includes('첫 화면의 페이지를 그리지 못했습니다.'));
  assert.equal(await page.evaluate(() => window.openProbe.results.some(r => r.name === 'case-no-paint.hwp')), false);
}, { skipLoadApp: true });
