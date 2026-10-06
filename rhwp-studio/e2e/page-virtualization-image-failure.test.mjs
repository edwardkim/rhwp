/**
 * #6042 Stage 5: 실제 브라우저 image decode 실패가 page scheduler 정착을 막지 않는지 확인한다.
 *
 * embedded data URL decode를 실패시켜 PageRenderer의 1.5s fallback 재렌더를 지난 뒤
 * 완료 서명·scheduler·surface가 정상 상태로 돌아오는 실제 수명 경로를 고정한다.
 */
import assert from 'node:assert/strict';
import { loadApp, loadHwpFile, runTest } from './helpers.mjs';

runTest('페이지 가상화 image decode 실패 fallback', async ({ page }) => {
  const pageErrors = [];
  page.on('pageerror', error => pageErrors.push(String(error)));

  await page.evaluateOnNewDocument(() => {
    const originalDecode = HTMLImageElement.prototype.decode;
    const fault = { attempts: 0, failures: 0 };
    Object.defineProperty(window, '__rhwpImageDecodeFault', { value: fault });
    HTMLImageElement.prototype.decode = function decodeWithSingleFailure() {
      const source = this.currentSrc || this.src;
      if (source.startsWith('data:image/')) {
        fault.attempts += 1;
        fault.failures += 1;
        return Promise.reject(new DOMException('injected image decode failure', 'EncodingError'));
      }
      return originalDecode.call(this);
    };
  });

  await loadApp(page, '?renderer=canvas2d&scrollProbe=1');
  await page.evaluate(() => {
    const renderer = window.__canvasView?.pageRenderer;
    const original = renderer?.reRenderPageCanvases;
    window.__rhwpImageFallbackRenders = 0;
    if (renderer && typeof original === 'function') {
      renderer.reRenderPageCanvases = function observedFallbackRender(...args) {
        window.__rhwpImageFallbackRenders += 1;
        return original.apply(this, args);
      };
    }
  });
  // 34%에서 문서를 처음 열어 첫 image prefetch 뒤에는 별도 zoom/resize가 끼지 않게 한다.
  await page.evaluate(() => {
    document.querySelector('#page-scroll-probe [data-zoom="34"]')?.click();
    return new Promise(resolve => setTimeout(resolve, 500));
  });
  await loadHwpFile(page, 'test-image.hwp');
  // PageRenderer fallback(1.5s)와 후속 decode/render가 모두 끝날 여유를 둔다.
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 3500)));

  const result = await page.evaluate(() => {
    document.querySelector('#page-scroll-probe [data-read]')?.click();
    const raw = document.querySelector('#page-scroll-probe pre')?.textContent || '{}';
    const trees = Array.from({ length: window.__wasm?.pageCount || 0 }, (_, page) => (
      window.__wasm?.getPageLayerTree(page) || ''
    ));
    return {
      fault: window.__rhwpImageDecodeFault,
      fallbackRenders: window.__rhwpImageFallbackRenders,
      prefetchedSignatures: window.__canvasView?.pageRenderer?.prefetchedImageSignatures?.size,
      probe: JSON.parse(raw),
      canvasCount: document.querySelectorAll('#scroll-container canvas').length,
      layerKinds: {
        image: trees.reduce((sum, tree) => sum + (tree.match(/\"type\":\"image\"/g)?.length || 0), 0),
        rawSvg: trees.reduce((sum, tree) => sum + (tree.match(/\"type\":\"rawSvg\"/g)?.length || 0), 0),
      },
    };
  });

  assert.ok(result.fault.failures > 0,
    `실제 embedded image decode 실패를 주입해야 한다: ${JSON.stringify(result.layerKinds)}`);
  assert.equal(result.fault.attempts, result.fault.failures,
    '이 검증에서는 모든 embedded image prefetch decode를 실패시켜야 한다');
  assert.ok(Math.abs(result.probe.zoom - 0.34) < 0.001, '문서는 34% 저배율에서 검증해야 한다');
  assert.ok(result.fault.attempts >= result.fault.failures,
    '주입한 실패가 실제 image decode 작업에 포함되어야 한다');
  assert.ok(result.fallbackRenders > 0, 'decode 실패 뒤 1.5초 fallback 재렌더가 실행되어야 한다');
  assert.equal(result.prefetchedSignatures, 0, '실패한 decode를 완료 서명으로 캐시하면 안 된다');
  assert.ok(result.canvasCount > 0, '실패 뒤에도 페이지 surface가 남아야 한다');
  assert.equal(result.probe.pendingImages, 0, 'fallback 뒤 image rerender job이 남으면 안 된다');
  assert.equal(result.probe.pendingPrefetch, 0, 'fallback 뒤 prefetch queue가 남으면 안 된다');
  assert.equal(result.probe.scheduler.visibleQueued, 0, 'visible scheduler queue가 정착해야 한다');
  assert.equal(result.probe.scheduler.prefetchQueued, 0, 'retained scheduler queue가 정착해야 한다');
  assert.deepEqual(result.probe.errors, [], '관찰 경계에서 제품 오류가 없어야 한다');
  assert.deepEqual(pageErrors, [], '브라우저 pageerror가 없어야 한다');
  assert.ok(result.probe.pages.every(entry => entry.flowImages === 'ready'),
    'fallback 뒤 연결된 flow image layer가 준비 상태여야 한다');

  console.log(`PAGE_VIRTUALIZATION_IMAGE_FAILURE_OK attempts=${result.fault.attempts} failures=${result.fault.failures} fallbackRenders=${result.fallbackRenders}`);
}, { skipLoadApp: true });
