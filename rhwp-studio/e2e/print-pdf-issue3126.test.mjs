/**
 * Issue #3126 — PDF 안내/진행 모달, same-origin iframe PDF 경로,
 * same-origin 인쇄 미리보기 창.
 *
 * 자동화는 native 인쇄 대화상자 대신 iframe의 print()만 가로챈다. 구성된 인쇄
 * 문서는 별도 headless page에서 CDP printToPDF로 변환해 페이지 수와 검색 가능한
 * 텍스트를 검증한다. 실제 대화상자 자동 호출은 수동 Chrome/Edge 절차로 보완한다.
 *
 * 실행:
 *   PDFTOTEXT=/path/to/pdftotext \
 *   node e2e/print-pdf-issue3126.test.mjs --mode=headless
 */
import { execFileSync } from 'child_process';
import { mkdirSync } from 'fs';
import { runTest, loadApp, loadHwpFile, createNewDocument, assert } from './helpers.mjs';

const OUTPUT_DIR = '../output/e2e/issue-3126';

/**
 * PDF 안내 표시 설정을 localStorage 에 직접 심는다 (앱 재초기화 없음).
 *
 * [#3450] 이 설정은 `rhwp-settings` 에 영속되고 UserSettingsService 는 앱 초기화
 * 시점에 한 번만 읽는다. 안내 숨김 케이스가 값을 false 로 남기면 **다음 실행**의
 * 앞선 케이스들이 안내 없는 흐름(즉시 준비 모달)으로 시작해, 안내 모달 관련 단언만
 * 실패한다. host 모드는 사용자 Chrome 프로필에 붙으므로 그 값이 실행 사이에 남는다.
 */
async function writePdfGuidancePreference(page, show) {
  await page.evaluate((shouldShow) => {
    const raw = JSON.parse(localStorage.getItem('rhwp-settings') || '{}');
    raw.dialog = { ...(raw.dialog || {}), showPdfPrintGuidance: shouldShow };
    localStorage.setItem('rhwp-settings', JSON.stringify(raw));
  }, show);
}

/** 안내 표시 설정을 심고 앱을 다시 초기화해 그 값으로 시작하게 한다. */
async function applyPdfGuidancePreference(page, show) {
  await writePdfGuidancePreference(page, show);
  await loadApp(page);
}

async function installPrintCapture(page) {
  await page.evaluate(() => {
    const sentinelHandle = { kind: 'file', name: 'issue-3126-sentinel.hwpx' };
    window.__wasm.currentFileHandle = sentinelHandle;
    window.__wasm.fileName = 'issue-3126-source.hwpx';
    window.__documentState.markDirty('issue-3126-e2e');
    window.__issue3126 = {
      sentinelHandle,
      sawPdfConfirmDialog: false,
      sawPdfProgress: false,
      guidanceVisibleDuringProgress: false,
      before: {
        fileName: window.__wasm.fileName,
        isDirty: window.__documentState.isDirty(),
        hostTitle: document.title,
      },
      capture: null,
    };

    const dialogObserver = new MutationObserver(() => {
      const dialog = document.querySelector('[data-testid="pdf-print-dialog"]');
      const dialogText = dialog?.textContent || '';
      const guidance = dialog?.querySelector('.dialog-pdf-guidance');
      const guidanceVisible = guidance
        && getComputedStyle(guidance).display !== 'none'
        && getComputedStyle(guidance).visibility !== 'hidden';
      if (
        dialogText.includes('PDF로 저장')
        && dialogText.includes('인쇄 창')
        && guidanceVisible
      ) {
        window.__issue3126.sawPdfConfirmDialog = true;
      }
      const progress = document.querySelector('[data-testid="pdf-print-progress"]');
      if (progress && !progress.hidden && (progress.textContent || '').includes('PDF 준비 중')) {
        window.__issue3126.sawPdfProgress = true;
        window.__issue3126.guidanceVisibleDuringProgress ||= Boolean(guidanceVisible);
      }
    });
    dialogObserver.observe(document.body, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (!(node instanceof HTMLIFrameElement) || node.id !== 'rhwp-print-surface') {
            continue;
          }
          node.addEventListener('load', () => {
            const printWindow = node.contentWindow;
            const printDocument = node.contentDocument;
            if (!printWindow || !printDocument) return;
            printWindow.print = () => {
              const pages = [...printDocument.querySelectorAll('.page')].map((page) => ({
                className: page.className,
                width: getComputedStyle(page).width,
                height: getComputedStyle(page).height,
                svgWidth: page.querySelector('svg')?.getAttribute('width') || '',
                svgHeight: page.querySelector('svg')?.getAttribute('height') || '',
                text: page.textContent || '',
              }));
              window.__issue3126.capture = {
                frameHref: printWindow.location.href,
                frameOrigin: printWindow.location.origin,
                hostOrigin: window.location.origin,
                printCallCount: 1,
                statusAtPrint: document.getElementById('sb-message')?.textContent || '',
                sawPdfConfirmDialog: window.__issue3126.sawPdfConfirmDialog,
                sawPdfProgress: window.__issue3126.sawPdfProgress,
                guidanceVisibleDuringProgress:
                  window.__issue3126.guidanceVisibleDuringProgress,
                pdfDialogVisibleAtPrint:
                  Boolean(document.querySelector('[data-testid="pdf-print-dialog"]')),
                title: printDocument.title,
                hostTitleAtPrint: document.title,
                html: printDocument.documentElement.outerHTML,
                styleText: [...printDocument.querySelectorAll('style')]
                  .map((style) => style.textContent || '')
                  .join('\n'),
                textElementCount: printDocument.querySelectorAll('text').length,
                bodyText: printDocument.body.textContent || '',
                embeddedFontRuleCount: [...printDocument.querySelectorAll('style')]
                  .map((style) => style.textContent || '')
                  .join('\n')
                  .match(/@font-face\s*\{/g)?.length || 0,
                pages,
                stateAtPrint: {
                  sameHandle: window.__wasm.currentFileHandle === sentinelHandle,
                  fileName: window.__wasm.fileName,
                  isDirty: window.__documentState.isDirty(),
                },
              };
            };
          }, { once: true });
        }
      }
    });
    observer.observe(document.body, { childList: true });
    window.__issue3126.observer = observer;
    window.__issue3126.dialogObserver = dialogObserver;
  });
}

async function clickPdfMenuItem(page) {
  return page.evaluate(() => {
    const fileMenu = [...document.querySelectorAll('#menu-bar .menu-item')]
      .find((element) => (element.textContent || '').includes('파일'));
    const title = fileMenu?.querySelector('.menu-title');
    if (!title) return { ok: false, reason: '파일 메뉴 없음' };
    title.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));

    const item = document.querySelector('.md-item[data-cmd="file:print-to-pdf"]');
    if (!item) return { ok: false, reason: 'PDF로 저장 메뉴 없음' };
    const tooltip = item.getAttribute('title') || '';
    const label = item.textContent || '';
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    return { ok: true, tooltip, label };
  });
}

async function confirmPdfDialog(page, { hideFutureGuidance = false } = {}) {
  await page.waitForSelector('[data-testid="pdf-print-dialog"]', { timeout: 10_000 });
  return page.evaluate((shouldHideFutureGuidance) => {
    const dialog = document.querySelector('[data-testid="pdf-print-dialog"]');
    const primary = dialog?.closest('.dialog-wrap')?.querySelector('.dialog-btn-primary');
    const hideGuidanceCheck = dialog?.querySelector(
      '[data-testid="pdf-print-hide-guidance"]',
    );
    if (!(primary instanceof HTMLButtonElement)) {
      return { ok: false, reason: 'PDF 확인 버튼 없음', text: dialog?.textContent || '' };
    }
    if (shouldHideFutureGuidance && hideGuidanceCheck instanceof HTMLInputElement) {
      hideGuidanceCheck.click();
    }
    const result = {
      ok: true,
      reason: '',
      text: dialog?.textContent || '',
      primaryLabel: primary.textContent || '',
      hasHideFutureGuidance: hideGuidanceCheck instanceof HTMLInputElement,
      hideFutureGuidanceChecked:
        hideGuidanceCheck instanceof HTMLInputElement && hideGuidanceCheck.checked,
    };
    primary.click();
    return result;
  }, hideFutureGuidance);
}

async function capturePrintDocument(page) {
  await page.waitForFunction(() => window.__issue3126?.capture?.printCallCount === 1, {
    timeout: 60_000,
  });
  await page.waitForFunction(() => !document.getElementById('rhwp-print-surface'), {
    timeout: 10_000,
  });
  return page.evaluate(() => {
    const state = window.__issue3126;
    state.observer?.disconnect();
    state.dialogObserver?.disconnect();
    return {
      before: state.before,
      capture: state.capture,
      after: {
        sameHandle: window.__wasm.currentFileHandle === state.sentinelHandle,
        fileName: window.__wasm.fileName,
        isDirty: window.__documentState.isDirty(),
        hostTitle: document.title,
        surfaceRemoved: !document.getElementById('rhwp-print-surface'),
      },
    };
  });
}

async function renderCapturedPdf(browser, html, outputName) {
  mkdirSync(OUTPUT_DIR, { recursive: true });
  const printPage = await browser.newPage();
  await printPage.setContent(html, { waitUntil: 'load' });
  await printPage.evaluate(async () => {
    await document.fonts.ready;
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  });
  const outputPath = `${OUTPUT_DIR}/${outputName}.pdf`;
  const bytes = await printPage.pdf({
    path: outputPath,
    printBackground: true,
    preferCSSPageSize: true,
  });
  await printPage.close();
  assert(Buffer.from(bytes).subarray(0, 4).toString('ascii') === '%PDF', 'CDP PDF 매직');
  return outputPath;
}

function inspectPdf(outputPath) {
  const pdfinfo = process.env.PDFINFO || 'pdfinfo';
  const info = execFileSync(pdfinfo, [outputPath], { encoding: 'utf8' });
  const pageCount = Number(info.match(/^Pages:\s+(\d+)/m)?.[1] || 0);

  const textCandidates = [process.env.PDFTOTEXT, 'pdftotext'].filter(Boolean);
  let text = null;
  for (const command of textCandidates) {
    try {
      text = execFileSync(command, [outputPath, '-'], { encoding: 'utf8' });
      break;
    } catch {
      // 다음 후보를 시도한다. 도구가 없으면 DOM text 계약만 검증한다.
    }
  }
  return { pageCount, text };
}

function assertSharedPdfContract(menu, dialog, result) {
  const { before, capture, after } = result;
  assert(menu.ok, `PDF로 저장 메뉴 클릭 (${menu.reason || ''})`);
  assert(menu.label.includes('PDF로 저장'), '별도 PDF로 저장 메뉴 label');
  assert(
    menu.tooltip.includes('대상') && menu.tooltip.includes('PDF로 저장'),
    '남은 브라우저 단계 tooltip 안내',
  );
  assert(dialog.ok, `PDF 안내 모달 확인 (${dialog.reason || ''})`);
  assert(
    dialog.text.includes('대상') && dialog.text.includes('PDF로 저장'),
    'PDF 모달에서 브라우저 인쇄 대상 안내',
  );
  assert(dialog.hasHideFutureGuidance, '다음부터 안내를 숨기는 체크박스');
  assert(dialog.primaryLabel.includes('인쇄 창 열기'), 'PDF 모달의 명시적 확인 버튼');
  assert(capture.frameOrigin === capture.hostOrigin, 'same-origin print iframe');
  assert(!capture.frameHref.startsWith('about:blank'), 'about:blank 비사용');
  assert(capture.frameHref.endsWith('/print.html'), '전용 print.html surface');
  assert(capture.printCallCount === 1, '모달 확인 뒤 print() 자동 1회 호출');
  assert(capture.sawPdfConfirmDialog, 'PDF 안내 모달 표시');
  assert(capture.sawPdfProgress, '같은 모달에서 PDF 준비 진행률 표시');
  assert(!capture.guidanceVisibleDuringProgress, '준비 중에는 저장 방법 안내를 숨김');
  assert(!capture.pdfDialogVisibleAtPrint, '네이티브 인쇄창 호출 전에 PDF 모달 제거');
  assert(capture.statusAtPrint.includes('PDF 준비 완료'), 'print() 직전 PDF 준비 완료 상태');
  assert(capture.title === 'issue-3126-source', '인쇄 문서 제목에 원본 basename 보존');
  assert(capture.hostTitleAtPrint === 'issue-3126-source', 'Edge PDF 기본 이름용 host 제목');
  assert(after.hostTitle === before.hostTitle, 'print() 뒤 Studio 제목 복원');
  assert(capture.styleText.includes('@page rhwp-print-page-1'), '페이지별 named @page');
  assert(capture.textElementCount > 0, '검색 가능한 SVG text 요소 보존');
  assert(capture.bodyText.trim().length > 0, '인쇄 문서 텍스트 보존');
  assert(capture.stateAtPrint.sameHandle && after.sameHandle, 'file handle 불변');
  assert(capture.stateAtPrint.fileName === before.fileName, 'print 시점 파일명 불변');
  assert(after.fileName === before.fileName, '완료 후 파일명 불변');
  assert(capture.stateAtPrint.isDirty && after.isDirty, 'dirty 상태 불변');
  assert(after.surfaceRemoved, 'print 호출 뒤 iframe 정리');
}

await runTest('#3126 PDF 경로 — #2524 embedded bitmap/SVG font 회귀', async ({ page, browser }) => {
  await applyPdfGuidancePreference(page, true);
  const load = await loadHwpFile(page, 'render-p35-font-native-bitmap.hwpx');
  await installPrintCapture(page);
  const menu = await clickPdfMenuItem(page);
  mkdirSync(OUTPUT_DIR, { recursive: true });
  await page.screenshot({
    path: `${OUTPUT_DIR}/pdf-guidance-modal.png`,
    fullPage: false,
  });
  const dialog = await confirmPdfDialog(page);
  const result = await capturePrintDocument(page);
  assertSharedPdfContract(menu, dialog, result);

  assert(result.capture.pages.length === load.pageCount, '인쇄 페이지 수 = 문서 페이지 수');
  assert(result.capture.embeddedFontRuleCount > 0, '#2524 embedded font @font-face 보존');
  assert(
    result.capture.styleText.includes('data:font/'),
    '#2524 embedded font가 data URI로 포함됨',
  );

  const outputPath = await renderCapturedPdf(
    browser,
    result.capture.html,
    'font-native-bitmap-print',
  );
  const pdf = inspectPdf(outputPath);
  assert(pdf.pageCount === load.pageCount, 'CDP PDF 페이지 수 = 문서 페이지 수');
  if (pdf.text !== null) {
    assert(pdf.text.trim().length > 0, '생성 PDF에서 텍스트 추출 가능');
  }
});

await runTest('#3126 PDF 경로 — #2525 다중 페이지/검색 텍스트 회귀', async ({ page, browser }) => {
  await applyPdfGuidancePreference(page, true);
  const load = await loadHwpFile(page, 'hwpx/hwpx-02.hwpx');
  assert(load.pageCount > 1, '#2525 fixture는 다중 페이지');
  await installPrintCapture(page);
  const menu = await clickPdfMenuItem(page);
  const dialog = await confirmPdfDialog(page);
  const result = await capturePrintDocument(page);
  assertSharedPdfContract(menu, dialog, result);

  assert(result.capture.pages.length === load.pageCount, '모든 #2525 페이지가 인쇄 문서에 포함됨');
  for (let index = 0; index < load.pageCount; index++) {
    assert(
      result.capture.styleText.includes(`@page rhwp-print-page-${index + 1}`),
      `페이지 ${index + 1} named @page`,
    );
  }

  const outputPath = await renderCapturedPdf(browser, result.capture.html, 'hwpx-02-print');
  const pdf = inspectPdf(outputPath);
  assert(pdf.pageCount === load.pageCount, 'CDP PDF 다중 페이지 수 정합');
  if (pdf.text !== null) {
    assert(pdf.text.replace(/\s/g, '').length > 20, '생성 PDF 검색 텍스트가 비어 있지 않음');
  }
});

await runTest('#3126 PDF 경로 — 안내 숨김 저장과 복원 가능한 직접 준비 흐름', async ({ page }) => {
  await applyPdfGuidancePreference(page, true);
  await loadHwpFile(page, 'render-p35-font-native-bitmap.hwpx');
  await installPrintCapture(page);
  const firstMenu = await clickPdfMenuItem(page);
  const firstDialog = await confirmPdfDialog(page, { hideFutureGuidance: true });
  const firstResult = await capturePrintDocument(page);
  assertSharedPdfContract(firstMenu, firstDialog, firstResult);
  assert(firstDialog.hideFutureGuidanceChecked, '안내 숨김 체크 상태로 실행');

  const storedAfterFirst = await page.evaluate(() => {
    const settings = JSON.parse(localStorage.getItem('rhwp-settings') || '{}');
    return settings.dialog?.showPdfPrintGuidance;
  });
  assert(storedAfterFirst === false, '안내 숨김 설정을 rhwp-settings에 저장');

  await installPrintCapture(page);
  const secondMenu = await clickPdfMenuItem(page);
  const secondResult = await capturePrintDocument(page);
  assert(secondMenu.ok, '안내를 숨긴 뒤 PDF 메뉴 재실행');
  assert(!secondResult.capture.sawPdfConfirmDialog, '두 번째 실행은 저장 방법 안내를 생략');
  assert(secondResult.capture.sawPdfProgress, '두 번째 실행도 PDF 준비 진행률 표시');
  assert(
    !secondResult.capture.guidanceVisibleDuringProgress,
    '두 번째 실행의 진행 모달에도 저장 방법 안내가 보이지 않음',
  );
  assert(!secondResult.capture.pdfDialogVisibleAtPrint, 'print() 전에 직접 준비 모달 제거');
  assert(secondResult.after.surfaceRemoved, '두 번째 실행 뒤 iframe 정리');

  // [#3450] 이 케이스가 남긴 안내 숨김 설정을 프로필에서 원복한다. 각 케이스가
  // 시작 시 값을 심으므로 이중 안전장치지만, 실행 뒤 프로필을 처음 상태로 돌려
  // 다른 스위트·수동 확인이 이 실행에 오염되지 않게 한다.
  await writePdfGuidancePreference(page, true);
});

async function clickPrintMenuItem(page) {
  return page.evaluate(() => {
    const fileMenu = [...document.querySelectorAll('#menu-bar .menu-item')]
      .find((element) => (element.textContent || '').includes('파일'));
    const title = fileMenu?.querySelector('.menu-title');
    if (!title) return { ok: false, reason: '파일 메뉴 없음' };
    title.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));

    const item = document.querySelector('.md-item[data-cmd="file:print"]');
    if (!item) return { ok: false, reason: '인쇄 메뉴 없음' };
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
    return { ok: true, reason: '', label: item.textContent || '' };
  });
}

await runTest('#3126 인쇄 경로 — same-origin 미리보기와 수동 인쇄', async ({ page }) => {
  const load = await loadHwpFile(page, 'hwpx/hwpx-02.hwpx');
  const before = await page.evaluate(() => {
    const sentinelHandle = { kind: 'file', name: 'issue-3126-preview.hwpx' };
    window.__wasm.currentFileHandle = sentinelHandle;
    window.__wasm.fileName = 'issue-3126-preview.hwpx';
    window.__documentState.markDirty('issue-3126-preview-e2e');
    window.__issue3126Preview = { sentinelHandle };
    return {
      fileName: window.__wasm.fileName,
      isDirty: window.__documentState.isDirty(),
    };
  });

  const popupPromise = new Promise((resolve) => page.once('popup', resolve));
  const menu = await clickPrintMenuItem(page);
  assert(menu.ok, `기존 인쇄 메뉴 클릭 (${menu.reason || ''})`);
  const preview = await popupPromise;
  await preview.waitForSelector('#print-btn', { timeout: 60_000 });
  mkdirSync(OUTPUT_DIR, { recursive: true });
  await preview.screenshot({
    path: `${OUTPUT_DIR}/print-preview.png`,
    fullPage: false,
  });

  const previewState = await preview.evaluate(() => ({
    href: location.href,
    origin: location.origin,
    title: document.title,
    toolbarText: document.querySelector('.print-preview-bar')?.textContent || '',
    pageCount: document.querySelectorAll('.page').length,
    hasPrintButton: Boolean(document.getElementById('print-btn')),
    hasCloseButton: Boolean(document.getElementById('close-btn')),
  }));
  const hostOrigin = new URL(page.url()).origin;
  assert(previewState.origin === hostOrigin, '인쇄 미리보기 same-origin');
  assert(!previewState.href.startsWith('about:blank'), '인쇄 미리보기 about:blank 비사용');
  assert(previewState.href.endsWith('/print.html'), '인쇄 미리보기 전용 print.html');
  assert(previewState.title.includes('인쇄 미리보기'), '인쇄 미리보기 창 제목');
  assert(previewState.toolbarText.includes('인쇄'), '미리보기 인쇄 도구');
  assert(previewState.hasPrintButton && previewState.hasCloseButton, '인쇄/닫기 버튼');
  assert(previewState.pageCount === load.pageCount, '미리보기 페이지 수 = 문서 페이지 수');

  await preview.evaluate(() => {
    window.__issue3126PreviewPrintCalls = 0;
    window.print = () => {
      window.__issue3126PreviewPrintCalls += 1;
    };
  });
  await preview.click('#print-btn');
  const printCallCount = await preview.evaluate(() => window.__issue3126PreviewPrintCalls);
  assert(printCallCount === 1, '미리보기 인쇄 버튼이 print() 1회 호출');

  const after = await page.evaluate(() => ({
    sameHandle:
      window.__wasm.currentFileHandle === window.__issue3126Preview.sentinelHandle,
    fileName: window.__wasm.fileName,
    isDirty: window.__documentState.isDirty(),
  }));
  assert(after.sameHandle, '미리보기 전후 file handle 불변');
  assert(after.fileName === before.fileName, '미리보기 전후 파일명 불변');
  assert(before.isDirty && after.isDirty, '미리보기 전후 dirty 상태 불변');

  const closePromise = new Promise((resolve) => preview.once('close', resolve));
  await preview.click('#close-btn');
  await closePromise;
  assert(true, '미리보기 닫기 버튼으로 창 종료');
});

await runTest('#9333 메모 선택 인쇄 — 본문·표 셀·댓글과 별도 메모 목록', async ({ page, browser }) => {
  await applyPdfGuidancePreference(page, true);
  await createNewDocument(page);
  const memo = await page.evaluate(async () => {
    const wasm = window.__wasm;
    wasm.doc.insertText(0, 0, 0, '첫 쪽 본문');
    wasm.insertMemo({ section: 0, para: 0, cellPath: [] }, 0, 3, '검토자',
      '첫 쪽 검토 메모\n' + ('긴 메모에서 다음 내용을 확인합니다.\n').repeat(85) + '계속 쪽 끝 확인');
    const parent = wasm.getMemos()[0];
    wasm.insertMemoReply(parent.id, '회신자', '부모 메모에 대한 회신');
    wasm.insertPageBreak(0, 0, 6);
    const table = wasm.createTable(0, 1, 0, 1, 1);
    wasm.insertTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, '둘째 쪽의 표 셀');
    wasm.insertMemo({ section: 0, para: table.paraIdx, cellPath: [[table.controlIdx, 0, 0]] },
      0, 2, '검토자', '표 셀 검토 메모');
    await window.__canvasView.loadDocument();
    return { pages: wasm.pageCount, source: Array.from(wasm.exportHwpx()), dirty: window.__documentState.isDirty() };
  });
  assert(memo.pages === 2, '검증 문서의 두 원래 쪽');
  const popup = new Promise(resolve => page.once('popup', resolve));
  await clickPrintMenuItem(page);
  const preview = await popup;
  await preview.waitForSelector('#print-btn');
  const content = () => preview.$$eval('.page', els => els.map(e => e.textContent.replace(/\s/g, '')));
  const unannotated = await content();
  assert(unannotated.length === 2 && !unannotated.join('').includes('검토메모'), '미리보기 기본은 메모 제외');
  await preview.click('.print-preview-bar summary');
  await preview.click('[name="print-memo-include"]');
  assert(await preview.$eval('[name="print-memo-list"]', el => el.checked), '메모 포함 선택 시 별도 목록도 선택');
  const withList = await content();
  assert(withList.at(-1).includes('메모내용'), '별도 메모 내용 페이지');
  await preview.click('[name="print-memo-list"]');
  const withoutList = await content();
  assert(withoutList[0].includes('첫쪽검토메모'), '첫 쪽 대상 메모');
  assert(withoutList.at(-1).includes('표셀검토메모'), '나중 쪽의 셀 메모');
  assert(withoutList.slice(1, -1).join('').includes('계속쪽끝확인'), '긴 메모의 이어지는 쪽');
  assert(withoutList.join('').includes('부모메모에대한회신'), '부모와 댓글을 같은 검토 출력에 포함');
  await preview.click('[name="print-memo-include"]');
  assert(JSON.stringify(await content()) === JSON.stringify(unannotated), '옵션을 끄면 모든 원래 쪽으로 돌아감');
  await preview.close();

  await installPrintCapture(page);
  await clickPdfMenuItem(page);
  await page.waitForSelector('[data-testid="pdf-print-dialog"]');
  await page.click('[data-testid="pdf-print-dialog"] [name="print-memo-include"]');
  await page.click('[data-testid="pdf-print-dialog"] [name="print-memo-list"]');
  await confirmPdfDialog(page);
  const result = await capturePrintDocument(page);
  const pdfPages = result.capture.pages.map(p => p.text.replace(/\s/g, ''));
  assert(JSON.stringify(pdfPages) === JSON.stringify(withoutList), 'PDF 출력과 인쇄 미리보기가 같은 쪽과 메모를 표시');
  const outputPath = await renderCapturedPdf(browser, result.capture.html, 'memo-review-print');
  const pdf = inspectPdf(outputPath);
  assert(pdf.pageCount === withoutList.length, '검색 가능한 PDF 페이지가 미리보기와 같음');
  if (pdf.text !== null) {
    for (const phrase of ['첫 쪽 검토 메모', '계속 쪽 끝 확인', '부모 메모에 대한 회신', '표 셀 검토 메모']) {
      assert(pdf.text.replace(/\s/g, '').includes(phrase.replace(/\s/g, '')), 'PDF 내용 보존 ' + phrase);
    }
  }
  const source = await page.evaluate(() => Array.from(window.__wasm.exportHwpx()));
  assert(JSON.stringify(source) === JSON.stringify(memo.source), '인쇄 설정은 원본 문서에 저장되지 않음');
});

await runTest('#9336 짧은 용지의 이어지는 메모 인쇄', async ({ page, browser }) => {
  await applyPdfGuidancePreference(page, true);
  await createNewDocument(page);
  const original = await page.evaluate(async () => {
    const wasm = window.__wasm;
    wasm.setPageDef(0, {
      ...wasm.getPageDef(0), width: 30000, height: 7200,
      leftMargin: 300, rightMargin: 300, topMargin: 300, bottomMargin: 300,
      headerMargin: 0, footerMargin: 0,
    });
    wasm.doc.insertText(0, 0, 0, '메모가 달린 글자');
    wasm.insertMemo({ section: 0, para: 0, cellPath: [] }, 0, 2, '작성자',
      '검토 줄 1\n검토 줄 2\n검토 줄 3\n검토 줄 4\n검토 줄 5');
    await window.__canvasView.loadDocument();
    return wasm.getPageInfo(0);
  });
  const popup = new Promise(resolve => page.once('popup', resolve));
  await clickPrintMenuItem(page);
  const preview = await popup;
  await preview.waitForSelector('#print-btn');
  await preview.click('.print-preview-bar summary');
  await preview.click('[name="print-memo-include"]');
  await preview.click('[name="print-memo-list"]');
  const pages = await preview.$$eval('.page', nodes => nodes.map(node => ({
    text: node.textContent.replace(/\s/g, ''),
    height: Number(node.querySelector('svg').getAttribute('height')),
  })));
  assert(pages.length === 2, `높이 25.4mm에서 제목만 있는 쪽 없이 두 쪽 인쇄 (실제 ${pages.length}쪽)`);
  assert(pages[0].height === original.height && pages[1].height > original.height,
    '본문은 원래 용지, 이어지는 메모는 본문을 담을 수 있는 별도 용지');
  assert(pages[0].text.includes('검토줄1') && pages[1].text.includes('1쪽검토줄2검토줄3검토줄4검토줄5'),
    '이어지는 쪽에 원래 쪽 번호와 나머지 메모 줄을 순서대로 보존');
  await preview.close();

  await installPrintCapture(page);
  await clickPdfMenuItem(page);
  await page.waitForSelector('[data-testid="pdf-print-dialog"]');
  await page.click('[data-testid="pdf-print-dialog"] [name="print-memo-include"]');
  await page.click('[data-testid="pdf-print-dialog"] [name="print-memo-list"]');
  await confirmPdfDialog(page);
  const { capture } = await capturePrintDocument(page);
  const outputPath = await renderCapturedPdf(browser, capture.html, 'memo-short-paper');
  const pdf = inspectPdf(outputPath);
  assert(pdf.pageCount === pages.length, '실제 PDF의 페이지 수가 미리보기와 일치');
  assert(JSON.stringify(capture.pages.map(p => p.text.replace(/\s/g, '')))
    === JSON.stringify(pages.map(p => p.text)), '실제 PDF의 입력과 미리보기가 같은 쪽·내용을 구성');
  if (pdf.text !== null) {
    const [first, continued] = pdf.text.split('\f').map(text => text.replace(/\s/g, ''));
    assert(first.includes('검토줄1') && continued.includes('1쪽검토줄2검토줄3검토줄4검토줄5'),
      '실제 PDF 각 쪽에서 메모 줄을 읽을 수 있음');
  }
});
