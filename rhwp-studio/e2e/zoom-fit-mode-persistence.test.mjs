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
 * 5. 두 쪽 폭 맞춤은 두 쪽과 사이 간격을 한 행으로 계산한다
 * 6. 여러 쪽은 비율 선택을 잠그고 전체 블록을 쪽 맞춤한다
 */

import {
  assert,
  runTest,
  screenshot,
  setTestCase,
} from './helpers.mjs';

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
    arrangement: JSON.parse(localStorage.getItem('rhwp-settings') || '{}').view?.pageArrangement ?? null,
    containerWidth: container.clientWidth,
    containerHeight: container.clientHeight,
    pageWidth: pageInfo.width,
    pageHeight: pageInfo.height,
  };
};

const pageGrid = (arrangement) => {
  if (arrangement?.kind === 'double' || arrangement?.kind === 'facing') {
    return { columns: 2, rows: 1 };
  }
  if (arrangement?.kind === 'multiple') {
    return { columns: arrangement.columns, rows: arrangement.rows };
  }
  return { columns: 1, rows: 1 };
};

const fitWidthZoom = (s) => {
  const { columns } = pageGrid(s.arrangement);
  return (
    s.containerWidth - HORIZONTAL_FRAME_PADDING - 10 * (columns - 1)
  ) / (s.pageWidth * columns);
};
const fitPageZoom = (s) => Math.min(
  fitWidthZoom(s),
  (
    s.containerHeight
    - VERTICAL_FRAME_PADDING
    - 10 * (pageGrid(s.arrangement).rows - 1)
  ) / (s.pageHeight * pageGrid(s.arrangement).rows),
);

async function openZoomDialog(page) {
  await page.click('#sb-zoom-display');
  await page.waitForSelector('.zoom-dialog', { visible: true });
}

async function confirmZoomDialog(page) {
  await page.click('.zoom-dialog .dialog-btn-primary');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
}

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
  setTestCase('TC1 자동 배치 폭 맞춤');
  await page.click('#sb-zoom-fit-width');
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const fitWidth = await page.evaluate(readZoomState);
  assert(fitWidth.fitMode === 'fitWidth', `TC1: 폭 맞춤이 저장됨 (${fitWidth.fitMode})`);
  assert(Math.abs(fitWidth.zoom - fitWidthZoom(fitWidth)) < 0.005,
    `TC1: 배율이 폭 맞춤 (${fitWidth.zoom.toFixed(3)} vs ${fitWidthZoom(fitWidth).toFixed(3)})`);

  // ── TC2: 다른 쪽 크기의 문서에서 다시 계산한다 ──────────────
  setTestCase('TC2 문서별 폭 맞춤 복원');
  await openDocument(page, DOC_B);
  const reopened = await page.evaluate(readZoomState);
  assert(reopened.fitMode === 'fitWidth', `TC2: 저장된 맞춤 유지 (${reopened.fitMode})`);
  assert(reopened.pageWidth !== fitWidth.pageWidth,
    `TC2: 두 문서의 쪽 폭이 실제로 다름 (${fitWidth.pageWidth} → ${reopened.pageWidth})`);
  assert(Math.abs(reopened.zoom - fitWidthZoom(reopened)) < 0.005,
    `TC2: 새 문서 쪽 폭으로 다시 계산 (${reopened.zoom.toFixed(3)} vs ${fitWidthZoom(reopened).toFixed(3)})`);

  // ── TC3: 쪽 맞춤은 앱을 다시 띄운 뒤에도 살아 있다 ──────────
  setTestCase('TC3 자동 배치 쪽 맞춤 복원');
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
  setTestCase('TC4 수치 배율 전환');
  await page.evaluate(() => {
    const range = document.getElementById('sb-zoom-range');
    range.value = String(Number(range.value) + 60);
    range.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  const manual = await page.evaluate(readZoomState);
  assert(manual.fitMode === 'none', `TC4: 가로바로 배율을 바꾸면 맞춤이 풀림 (${manual.fitMode})`);

  // ── TC5: 고정 쪽 배치별 폭·쪽 맞춤을 비교한다 ───────────────
  for (const [kind, label] of [
    ['single', '한 쪽'],
    ['double', '두 쪽'],
    ['facing', '맞쪽'],
  ]) {
    setTestCase(`TC5 ${label} 배치 맞춤`);
    await openZoomDialog(page);
    await page.click(`input[name="page-arrangement"][value="${kind}"]`);
    await page.click('input[name="zoom-choice"][value="fitWidth"]');
    await confirmZoomDialog(page);
    const widthFit = await page.evaluate(readZoomState);
    await screenshot(page, `issue-6108-${kind}-fit-width`);
    assert(widthFit.arrangement?.kind === kind,
      `TC5: ${label} 배치가 저장됨 (${widthFit.arrangement?.kind})`);
    assert(widthFit.fitMode === 'fitWidth',
      `TC5: ${label} 폭 맞춤 규칙이 저장됨 (${widthFit.fitMode})`);
    assert(Math.abs(widthFit.zoom - fitWidthZoom(widthFit)) < 0.005,
      `TC5: ${label} 행 전체를 폭 맞춤 (${widthFit.zoom.toFixed(3)} vs ${fitWidthZoom(widthFit).toFixed(3)})`);

    await openZoomDialog(page);
    await page.click('input[name="zoom-choice"][value="fitPage"]');
    await confirmZoomDialog(page);
    const pageFit = await page.evaluate(readZoomState);
    await screenshot(page, `issue-6108-${kind}-fit-page`);
    assert(pageFit.fitMode === 'fitPage',
      `TC5: ${label} 쪽 맞춤 규칙이 저장됨 (${pageFit.fitMode})`);
    assert(Math.abs(pageFit.zoom - fitPageZoom(pageFit)) < 0.005,
      `TC5: ${label} 블록 전체를 쪽 맞춤 (${pageFit.zoom.toFixed(3)} vs ${fitPageZoom(pageFit).toFixed(3)})`);
  }

  // ── TC6: 여러 쪽은 비율 선택을 잠그고 전체 블록을 맞춘다 ────
  setTestCase('TC6 여러 쪽 2×2 입력 계약');
  await openZoomDialog(page);
  await page.click('input[name="page-arrangement"][value="multiple"]');
  await page.evaluate(() => {
    for (const ariaLabel of ['여러 쪽 가로 쪽 수', '여러 쪽 세로 쪽 수']) {
      const input = document.querySelector(`input[aria-label="${ariaLabel}"]`);
      input.value = '2';
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new Event('change', { bubbles: true }));
    }
  });
  const multipleControls = await page.evaluate(() => ({
    allZoomChoicesDisabled: [...document.querySelectorAll('input[name="zoom-choice"]')]
      .every((input) => input.disabled),
    customDisabled: document.querySelector('input[aria-label="사용자 정의 배율"]')?.disabled,
    columnsEnabled: !document.querySelector('input[aria-label="여러 쪽 가로 쪽 수"]')?.disabled,
    rowsEnabled: !document.querySelector('input[aria-label="여러 쪽 세로 쪽 수"]')?.disabled,
  }));
  await screenshot(page, 'issue-6108-multiple-disabled-ratios');
  assert(multipleControls.allZoomChoicesDisabled && multipleControls.customDisabled,
    'TC6: 여러 쪽에서 모든 비율 선택과 사용자 정의 입력이 비활성화됨');
  assert(multipleControls.columnsEnabled && multipleControls.rowsEnabled,
    'TC6: 여러 쪽의 가로·세로 쪽 수 입력은 활성화됨');
  await confirmZoomDialog(page);
  const multipleFit = await page.evaluate(readZoomState);
  setTestCase('TC6 여러 쪽 2×2 전체 맞춤');
  await screenshot(page, 'issue-6108-multiple-2x2-fit');
  assert(
    multipleFit.arrangement?.kind === 'multiple'
      && multipleFit.arrangement.columns === 2
      && multipleFit.arrangement.rows === 2,
    `TC6: 여러 쪽 2×2 배치가 저장됨 (${JSON.stringify(multipleFit.arrangement)})`,
  );
  assert(Math.abs(multipleFit.zoom - fitPageZoom(multipleFit)) < 0.005,
    `TC6: 여러 쪽 2×2 전체 블록을 쪽 맞춤 (${multipleFit.zoom.toFixed(3)} vs ${fitPageZoom(multipleFit).toFixed(3)})`);
});
