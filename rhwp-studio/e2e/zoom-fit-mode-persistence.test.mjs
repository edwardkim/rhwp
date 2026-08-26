/**
 * E2E 테스트 — 쪽 맞춤/폭 맞춤 선택을 저장하고 문서를 열 때 되돌린다
 *
 * 맞춤은 배율 수치가 아니라 규칙이다. 저장값은 `rhwp-settings.view.zoomFitMode` 이고,
 * 문서를 열 때 그 문서의 쪽 크기로 다시 계산해 적용한다. 휠·가로바 같은 수치 조작은
 * 맞춤을 푼다.
 *
 * 검증 항목:
 * 1. 폭 맞춤 단추가 선택을 저장한다
 * 2. 다른 문서를 열면 그 문서의 쪽 크기로 폭 맞춤을 다시 계산한다
 * 3. 쪽 맞춤은 앱을 다시 띄운 뒤 첫 문서에도 적용된다
 * 4. 배율 가로바로 수치를 바꾸면 맞춤이 풀린다
 */

import { runTest, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const DOC_A = '2010-01-06.hwp';
const DOC_B = '253E164F57A1BC6934-empty.hwp'; // A4 가로 — 쪽 크기가 A 와 다르다

const HORIZONTAL_FRAME_PADDING = 40;
const VERTICAL_FRAME_PADDING = 20;

async function openDocument(page, filename) {
  const result = await page.evaluate(async (fname) => {
    const resp = await fetch(`/samples/${encodeURIComponent(fname)}`);
    if (!resp.ok) return { error: `HTTP ${resp.status}` };
    const bytes = new Uint8Array(await resp.arrayBuffer());
    const requestId = `zoom-fit-${Math.random().toString(36).slice(2)}`;
    const done = new Promise((resolve) => {
      const off = window.__eventBus.on('open-document-bytes:done', (payload) => {
        if (payload.requestId !== requestId) return;
        off();
        resolve(payload);
      });
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes, fileName: fname, fileHandle: null, skipUnsavedGuard: true, requestId,
    });
    const outcome = await done;
    await new Promise((r) => setTimeout(r, 900));
    return outcome;
  }, filename);
  if (result.error || result.ok === false) {
    throw new Error(`문서 열기 실패 (${filename}): ${result.error ?? '알 수 없음'}`);
  }
}

const readZoomState = () => {
  const container = document.getElementById('scroll-container');
  const pageInfo = window.__wasm.getPageInfo(0);
  return {
    zoom: window.__canvasView.getViewportManager().getZoom(),
    fitMode: JSON.parse(localStorage.getItem('rhwp-settings') || '{}').view?.zoomFitMode ?? null,
    containerWidth: container.clientWidth,
    containerHeight: container.clientHeight,
    pageWidth: pageInfo.width,
    pageHeight: pageInfo.height,
  };
};

const fitWidthZoom = (s) => (s.containerWidth - HORIZONTAL_FRAME_PADDING) / s.pageWidth;
const fitPageZoom = (s) => Math.min(
  (s.containerWidth - HORIZONTAL_FRAME_PADDING) / s.pageWidth,
  (s.containerHeight - VERTICAL_FRAME_PADDING) / s.pageHeight,
);

async function startFresh(page) {
  // 첫 실행 스킨 안내 모달이 상태 표시줄 클릭을 삼킨다 — 선택을 마친 상태로 시작한다.
  await page.evaluate(() => {
    const stored = JSON.parse(localStorage.getItem('rhwp-settings') || '{}');
    localStorage.setItem('rhwp-settings', JSON.stringify({
      ...stored,
      version: 1,
      theme: { mode: 'system', skin: 'default', skinChosen: true },
    }));
  });
  await page.reload({ waitUntil: 'networkidle0' });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1500)));
}

runTest('쪽/폭 맞춤 저장과 복원', async ({ page }) => {
  await page.evaluate(() => localStorage.removeItem('rhwp-settings'));
  await startFresh(page);
  await openDocument(page, DOC_A);

  // ── TC1: 폭 맞춤 단추가 선택을 저장한다 ────────────────────
  await page.click('#sb-zoom-fit-width');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const fitWidth = await page.evaluate(readZoomState);
  assert(fitWidth.fitMode === 'fitWidth', `TC1: 폭 맞춤이 저장됨 (${fitWidth.fitMode})`);
  assert(Math.abs(fitWidth.zoom - fitWidthZoom(fitWidth)) < 0.005,
    `TC1: 배율이 폭 맞춤 (${fitWidth.zoom.toFixed(3)} vs ${fitWidthZoom(fitWidth).toFixed(3)})`);

  // ── TC2: 다른 쪽 크기의 문서에서 다시 계산한다 ──────────────
  await openDocument(page, DOC_B);
  const reopened = await page.evaluate(readZoomState);
  assert(reopened.fitMode === 'fitWidth', `TC2: 저장된 맞춤 유지 (${reopened.fitMode})`);
  assert(reopened.pageWidth !== fitWidth.pageWidth,
    `TC2: 두 문서의 쪽 폭이 실제로 다름 (${fitWidth.pageWidth} → ${reopened.pageWidth})`);
  assert(Math.abs(reopened.zoom - fitWidthZoom(reopened)) < 0.005,
    `TC2: 새 문서 쪽 폭으로 다시 계산 (${reopened.zoom.toFixed(3)} vs ${fitWidthZoom(reopened).toFixed(3)})`);

  // ── TC3: 쪽 맞춤은 앱을 다시 띄운 뒤에도 살아 있다 ──────────
  await page.click('#sb-zoom-fit');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const fitPage = await page.evaluate(readZoomState);
  assert(fitPage.fitMode === 'fitPage', `TC3: 쪽 맞춤이 저장됨 (${fitPage.fitMode})`);

  await startFresh(page);
  await openDocument(page, DOC_A);
  const restored = await page.evaluate(readZoomState);
  assert(restored.fitMode === 'fitPage', `TC3: 새 세션에서도 쪽 맞춤 유지 (${restored.fitMode})`);
  assert(Math.abs(restored.zoom - fitPageZoom(restored)) < 0.005,
    `TC3: 새 세션 첫 문서가 쪽 맞춤으로 열림 (${restored.zoom.toFixed(3)} vs ${fitPageZoom(restored).toFixed(3)})`);

  // ── TC4: 수치 배율은 맞춤을 푼다 ───────────────────────────
  await page.evaluate(() => {
    const range = document.getElementById('sb-zoom-range');
    range.value = String(Number(range.value) + 60);
    range.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const manual = await page.evaluate(readZoomState);
  assert(manual.fitMode === 'none', `TC4: 가로바로 배율을 바꾸면 맞춤이 풀림 (${manual.fitMode})`);
});
