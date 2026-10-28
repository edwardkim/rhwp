/**
 * E2E 회귀 — #6109 사용자 배율 검증과 원자 보기 설정 transaction
 *
 * 검증 항목:
 * 1. invalid 사용자 배율은 대화상자 안에서 설명하고 어떤 보기 이벤트도 내지 않는다
 * 2. 교정한 값을 Enter로 제출하면 배치·이동·배율을 한 transaction으로 적용한다
 * 3. 표준 zoom 이벤트는 유지하면서 CanvasView는 최종 상태로 recalcLayout을 한 번만 수행한다
 * 4. Escape와 취소는 상태를 바꾸지 않는다
 * 5. 10%와 500% 경계값을 실제 브라우저에서 정확히 적용한다
 */

import {
  assert,
  createNewDocument,
  loadApp,
  runTest,
  screenshot,
  setTestCase,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const delay = (page, milliseconds = 100) => page.evaluate(
  (duration) => new Promise(resolve => setTimeout(resolve, duration)),
  milliseconds,
);

async function openZoomDialog(page) {
  await page.click('#sb-zoom-display');
  await page.waitForSelector('.zoom-dialog', { visible: true });
}

async function selectCustomZoom(page, rawValue) {
  await page.click('input[name="zoom-choice"][value="custom"]');
  await page.evaluate((value) => {
    const input = document.querySelector('input[aria-label="사용자 정의 배율"]');
    input.value = value;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.focus();
  }, rawValue);
}

async function confirmZoomDialog(page) {
  await page.click('.zoom-dialog .dialog-btn-primary');
}

async function installTransactionProbe(page) {
  await page.evaluate(() => {
    const view = window.__canvasView;
    const eventBus = window.__eventBus;
    if (!view || !eventBus) throw new Error('CanvasView 또는 EventBus가 준비되지 않음');
    const probe = {
      pageViewEvents: [],
      zoomEvents: [],
      recalcCount: 0,
      recalcSnapshots: [],
    };
    const originalRecalc = view.recalcLayout;
    view.recalcLayout = function issue6109RecalcProbe(...args) {
      probe.recalcCount += 1;
      probe.recalcSnapshots.push({
        zoom: this.getViewportManager().getZoom(),
        arrangement: structuredClone(this.pageArrangement),
        pageMovement: structuredClone(this.pageMovement),
      });
      return originalRecalc.apply(this, args);
    };
    eventBus.on('page-view-settings-changed', payload => {
      probe.pageViewEvents.push(structuredClone(payload));
    });
    eventBus.on('zoom-changed', payload => {
      probe.zoomEvents.push(structuredClone(payload));
    });
    window.__issue6109Probe = probe;
  });
}

async function resetProbe(page) {
  await page.evaluate(() => {
    const probe = window.__issue6109Probe;
    probe.pageViewEvents.length = 0;
    probe.zoomEvents.length = 0;
    probe.recalcCount = 0;
    probe.recalcSnapshots.length = 0;
  });
}

async function readState(page) {
  return page.evaluate(() => {
    const probe = window.__issue6109Probe;
    const settings = JSON.parse(localStorage.getItem('rhwp-settings') || '{}').view || {};
    return {
      dialogVisible: !!document.querySelector('.zoom-dialog'),
      zoom: window.__canvasView.getViewportManager().getZoom(),
      settings,
      pageViewEvents: structuredClone(probe.pageViewEvents),
      zoomEvents: structuredClone(probe.zoomEvents),
      recalcCount: probe.recalcCount,
      recalcSnapshots: structuredClone(probe.recalcSnapshots),
    };
  });
}

async function assertInvalidSubmission(page, rawValue, label, expectedZoom) {
  await resetProbe(page);
  await openZoomDialog(page);
  await selectCustomZoom(page, rawValue);
  await confirmZoomDialog(page);
  await delay(page);
  const state = await page.evaluate(() => {
    const input = document.querySelector('input[aria-label="사용자 정의 배율"]');
    const error = document.getElementById('zoom-dialog-custom-error');
    const probe = window.__issue6109Probe;
    return {
      dialogVisible: !!document.querySelector('.zoom-dialog'),
      ariaInvalid: input?.getAttribute('aria-invalid'),
      describedBy: input?.getAttribute('aria-describedby'),
      inputFocused: document.activeElement === input,
      errorVisible: !!error && !error.hidden,
      errorText: error?.textContent || '',
      zoom: window.__canvasView.getViewportManager().getZoom(),
      pageViewEventCount: probe.pageViewEvents.length,
      zoomEventCount: probe.zoomEvents.length,
      recalcCount: probe.recalcCount,
    };
  });
  assert(state.dialogVisible, `${label}: invalid 제출 뒤 대화상자를 유지함`);
  assert(state.ariaInvalid === 'true', `${label}: 입력에 aria-invalid=true를 설정함`);
  assert(state.describedBy === 'zoom-dialog-custom-error', `${label}: 입력과 안정된 오류 ID를 연결함`);
  assert(state.inputFocused, `${label}: 오류 입력으로 focus를 복원함`);
  assert(state.errorVisible && state.errorText.length > 0, `${label}: role=alert 오류 설명을 표시함`);
  assert(Math.abs(state.zoom - expectedZoom) < 1e-9, `${label}: 기존 배율을 유지함`);
  assert(
    state.pageViewEventCount === 0 && state.zoomEventCount === 0 && state.recalcCount === 0,
    `${label}: 보기 이벤트와 레이아웃을 실행하지 않음`,
  );
}

runTest('Issue #6109 사용자 배율·보기 설정 transaction', async ({ page }) => {
  await page.evaluateOnNewDocument(() => {
    localStorage.setItem('rhwp-settings', JSON.stringify({
      version: 1,
      theme: { mode: 'system', skin: 'default', skinChosen: true },
    }));
  });
  await loadApp(page);
  await createNewDocument(page);
  await installTransactionProbe(page);
  const initialZoom = await page.evaluate(
    () => window.__canvasView.getViewportManager().getZoom(),
  );

  setTestCase('TC1 invalid 사용자 배율 차단');
  await assertInvalidSubmission(page, '', '빈 값', initialZoom);
  await page.keyboard.press('Escape');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  await assertInvalidSubmission(page, '9', '최솟값 미만', initialZoom);
  await page.keyboard.press('Escape');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  await assertInvalidSubmission(page, '501', '최댓값 초과', initialZoom);
  await screenshot(page, 'issue-6109-invalid-custom-zoom');
  for (const mode of ['light', 'dark']) {
    const colors = await page.evaluate((themeMode) => {
      window.__theme.setThemeMode(themeMode);
      const input = document.querySelector('input[aria-label="사용자 정의 배율"]');
      const error = document.getElementById('zoom-dialog-custom-error');
      const probe = document.createElement('span');
      probe.style.color = 'var(--ui-danger-strong)';
      document.body.appendChild(probe);
      const tokenColor = getComputedStyle(probe).color;
      probe.remove();
      return {
        tokenColor,
        inputBorderColor: getComputedStyle(input).borderColor,
        errorColor: getComputedStyle(error).color,
      };
    }, mode);
    assert(
      colors.inputBorderColor === colors.tokenColor && colors.errorColor === colors.tokenColor,
      `${mode === 'light' ? '밝은' : '어두운'} 테마의 입력·오류 문구가 --ui-danger-strong 계산값을 공유함`,
    );
  }

  setTestCase('TC2 교정 후 Enter 원자 적용');
  await selectCustomZoom(page, '137');
  const corrected = await page.evaluate(() => {
    const input = document.querySelector('input[aria-label="사용자 정의 배율"]');
    const error = document.getElementById('zoom-dialog-custom-error');
    return {
      ariaInvalid: input?.getAttribute('aria-invalid'),
      errorHidden: error?.hidden,
      errorText: error?.textContent || '',
    };
  });
  assert(
    corrected.ariaInvalid === 'false' && corrected.errorHidden && corrected.errorText === '',
    '유효한 값으로 교정하면 오류와 ARIA invalid 상태를 해제함',
  );
  await page.click('input[name="page-arrangement"][value="double"]');
  await page.click('input[aria-label="마우스 휠을 사용하여 좌우로 스크롤하기"]');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  await delay(page, 150);
  const committed = await readState(page);
  const pageEvent = committed.pageViewEvents[0];
  const zoomEvent = committed.zoomEvents[0];
  const recalc = committed.recalcSnapshots[0];
  assert(committed.pageViewEvents.length === 1, 'page-view-settings-changed를 한 번 발행함');
  assert(committed.zoomEvents.length === 1, '표준 zoom-changed를 한 번 유지함');
  assert(committed.recalcCount === 1, 'CanvasView recalcLayout을 한 번만 실행함');
  assert(
    pageEvent?.arrangement?.kind === 'double'
      && pageEvent?.pageMovement?.direction === 'vertical'
      && pageEvent?.pageMovement?.wheelHorizontal === true
      && Math.abs(pageEvent?.zoom?.value - 1.37) < 1e-9,
    '단일 transaction payload가 최종 배치·이동·배율을 포함함',
  );
  assert(
    Math.abs(zoomEvent?.zoom - 1.37) < 1e-9 || Math.abs(zoomEvent - 1.37) < 1e-9,
    'zoom-changed가 최종 137%를 전달함',
  );
  assert(
    recalc?.arrangement?.kind === 'double'
      && recalc?.pageMovement?.wheelHorizontal === true
      && Math.abs(recalc?.zoom - 1.37) < 1e-9,
    '유일한 recalcLayout이 이미 최종 배치·이동·배율 상태에서 실행됨',
  );
  assert(
    committed.settings.pageArrangement?.kind === 'double'
      && committed.settings.pageMovement?.wheelHorizontal === true
      && committed.settings.zoomFitMode === 'none'
      && Math.abs(committed.zoom - 1.37) < 1e-9,
    '저장 상태와 화면 배율이 최종 transaction과 일치함',
  );

  setTestCase('TC3 Escape·취소 무변경');
  await resetProbe(page);
  await openZoomDialog(page);
  await selectCustomZoom(page, '200');
  await page.keyboard.press('Escape');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  const escaped = await readState(page);
  assert(
    Math.abs(escaped.zoom - 1.37) < 1e-9
      && escaped.pageViewEvents.length === 0
      && escaped.zoomEvents.length === 0
      && escaped.recalcCount === 0,
    'Escape는 배율·보기 이벤트·레이아웃을 변경하지 않음',
  );
  await openZoomDialog(page);
  await selectCustomZoom(page, '250');
  await page.click('.zoom-dialog .dialog-footer .dialog-btn:not(.dialog-btn-primary)');
  await page.waitForSelector('.zoom-dialog', { hidden: true });
  const cancelled = await readState(page);
  assert(
    Math.abs(cancelled.zoom - 1.37) < 1e-9
      && cancelled.pageViewEvents.length === 0
      && cancelled.zoomEvents.length === 0
      && cancelled.recalcCount === 0,
    '취소 버튼은 배율·보기 이벤트·레이아웃을 변경하지 않음',
  );

  setTestCase('TC4 유효 경계 10%·500%');
  for (const percent of [10, 500]) {
    await resetProbe(page);
    await openZoomDialog(page);
    await selectCustomZoom(page, String(percent));
    await confirmZoomDialog(page);
    await page.waitForSelector('.zoom-dialog', { hidden: true });
    await delay(page, 150);
    const boundary = await readState(page);
    assert(
      Math.abs(boundary.zoom - percent / 100) < 1e-9,
      `${percent}% 경계값을 정확히 적용함`,
    );
    assert(
      boundary.pageViewEvents.length === 1
        && boundary.zoomEvents.length === 1
        && boundary.recalcCount === 1,
      `${percent}% 적용도 단일 이벤트·레이아웃 transaction을 유지함`,
    );
  }
}, { skipLoadApp: true });
