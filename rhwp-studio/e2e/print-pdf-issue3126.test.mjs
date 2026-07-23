/**
 * Issue #3126 — 발견 가능한 PDF 저장 진입점과 same-origin iframe print pipeline.
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
import { runTest, loadHwpFile, assert } from './helpers.mjs';

const OUTPUT_DIR = '../output/e2e/issue-3126';

async function installPrintCapture(page) {
  await page.evaluate(() => {
    const sentinelHandle = { kind: 'file', name: 'issue-3126-sentinel.hwpx' };
    window.__wasm.currentFileHandle = sentinelHandle;
    window.__wasm.fileName = 'issue-3126-source.hwpx';
    window.__documentState.markDirty('issue-3126-e2e');
    window.__issue3126 = {
      sentinelHandle,
      sawPreparingFeedback: false,
      before: {
        fileName: window.__wasm.fileName,
        isDirty: window.__documentState.isDirty(),
      },
      capture: null,
    };

    const feedbackObserver = new MutationObserver(() => {
      const feedback = document.querySelector('[data-toast-kind="print-feedback"]');
      if ((feedback?.textContent || '').includes('PDF 준비 중')) {
        window.__issue3126.sawPreparingFeedback = true;
      }
    });
    feedbackObserver.observe(document.body, {
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
                sawPreparingFeedback: window.__issue3126.sawPreparingFeedback,
                printFeedbackVisibleAtPrint:
                  Boolean(document.querySelector('[data-toast-kind="print-feedback"]')),
                title: printDocument.title,
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
    window.__issue3126.feedbackObserver = feedbackObserver;
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
    state.feedbackObserver?.disconnect();
    return {
      before: state.before,
      capture: state.capture,
      after: {
        sameHandle: window.__wasm.currentFileHandle === state.sentinelHandle,
        fileName: window.__wasm.fileName,
        isDirty: window.__documentState.isDirty(),
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

function assertSharedPdfContract(menu, result) {
  const { before, capture, after } = result;
  assert(menu.ok, `PDF로 저장 메뉴 클릭 (${menu.reason || ''})`);
  assert(menu.label.includes('PDF로 저장'), '별도 PDF로 저장 메뉴 label');
  assert(
    menu.tooltip.includes('대상') && menu.tooltip.includes('PDF로 저장'),
    '남은 브라우저 단계 tooltip 안내',
  );
  assert(capture.frameOrigin === capture.hostOrigin, 'same-origin print iframe');
  assert(!capture.frameHref.startsWith('about:blank'), 'about:blank 비사용');
  assert(capture.frameHref.endsWith('/print.html'), '전용 print.html surface');
  assert(capture.printCallCount === 1, 'rhwp 한 번 클릭으로 print() 자동 1회 호출');
  assert(capture.sawPreparingFeedback, '인쇄창을 열기 전에 PDF 준비 피드백 표시');
  assert(!capture.printFeedbackVisibleAtPrint, '네이티브 인쇄창 호출 전에 안내 토스트 제거');
  assert(capture.statusAtPrint.includes('PDF 준비 완료'), 'print() 직전 PDF 준비 완료 상태');
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
  const load = await loadHwpFile(page, 'render-p35-font-native-bitmap.hwpx');
  await installPrintCapture(page);
  const menu = await clickPdfMenuItem(page);
  const result = await capturePrintDocument(page);
  assertSharedPdfContract(menu, result);

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
  const load = await loadHwpFile(page, 'hwpx/hwpx-02.hwpx');
  assert(load.pageCount > 1, '#2525 fixture는 다중 페이지');
  await installPrintCapture(page);
  const menu = await clickPdfMenuItem(page);
  const result = await capturePrintDocument(page);
  assertSharedPdfContract(menu, result);

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
