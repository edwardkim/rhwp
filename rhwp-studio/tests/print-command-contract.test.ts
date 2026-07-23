import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const commandSource = readFileSync(
  new URL('../src/command/commands/file.ts', import.meta.url),
  'utf8',
);
const indexHtml = readFileSync(
  new URL('../index.html', import.meta.url),
  'utf8',
);
const printSurfaceSource = readFileSync(
  new URL('../src/command/print-surface.ts', import.meta.url),
  'utf8',
);
const toolbarCss = readFileSync(
  new URL('../src/styles/toolbar.css', import.meta.url),
  'utf8',
);
const pdfDialogSource = readFileSync(
  new URL('../src/ui/pdf-print-dialog.ts', import.meta.url),
  'utf8',
);
const optionsDialogSource = readFileSync(
  new URL('../src/ui/options-dialog.ts', import.meta.url),
  'utf8',
);
const printHtml = readFileSync(
  new URL('../public/print.html', import.meta.url),
  'utf8',
);

test('PDF와 인쇄 미리보기는 페이지 준비 pipeline을 공유하고 surface만 분리한다', () => {
  assert.match(commandSource, /runPdfPrint\(services\)/);
  assert.match(commandSource, /runPrintPreview\(services\)/);
  assert.equal(commandSource.match(/preparePrintPages\(services,/g)?.length, 2);
  assert.match(commandSource, /createPrintSurface\(\)/);
  assert.match(commandSource, /createPrintPreviewSurface\(\)/);
  assert.match(commandSource, /surface\.window\.print\(\)/);
  assert.match(printSurfaceSource, /hostWindow\.open\(surfaceUrl, '_blank'\)/);
  assert.doesNotMatch(printSurfaceSource, /about:blank/);
});

test('print pipeline은 명시적인 print profile SVG만 사용한다', () => {
  assert.match(commandSource, /renderPageSvgWithProfile\(i, 'print'\)/);
  assert.doesNotMatch(commandSource, /wasm\.renderPageSvg\(i\)/);
});

test('파일 메뉴는 별도 PDF 진입점과 브라우저의 남은 단계를 노출한다', () => {
  assert.match(indexHtml, /data-cmd="file:print-to-pdf"/);
  assert.match(indexHtml, />PDF로 저장…</);
  assert.match(indexHtml, /대상 → PDF로 저장/);
  assert.match(
    indexHtml,
    /data-cmd="file:print-to-pdf"[^>]*>.*class="md-icon icon-pdf"/,
  );
  assert.match(
    toolbarCss,
    /\.icon-pdf\s*\{\s*background-position:\s*calc\(-40px \* 3\) calc\(-40px \* 0\);\s*\}/,
  );
  assert.match(indexHtml, /data-cmd="file:print"/);
});

test('PDF 경로는 안내·진행 모달을 닫은 뒤 native 인쇄창을 호출한다', () => {
  assert.match(pdfDialogSource, /PDF_PRINT_GUIDANCE/);
  assert.match(pdfDialogSource, /인쇄 창 열기/);
  assert.match(pdfDialogSource, /다음부터 이 안내를 표시하지 않기/);
  assert.match(pdfDialogSource, /printProgressText\('pdf'/);
  assert.match(commandSource, /getShowPdfPrintGuidance\(\)/);
  assert.match(commandSource, /setShowPdfPrintGuidance\(false\)/);
  assert.match(optionsDialogSource, /PDF로 저장할 때 저장 방법 안내 표시/);
  assert.match(commandSource, /dialog\.closeBeforePrint\(\)/);
  assert.match(commandSource, /await waitForHostPaint\(\)/);
  assert.doesNotMatch(commandSource, /data\.toastKind|PDF_FEEDBACK_MIN_VISIBLE_MS/);
});

test('인쇄 전용 문서는 same-origin 미리보기 loading surface를 제공한다', () => {
  assert.match(printHtml, /인쇄 미리보기를 준비하고 있습니다/);
  assert.match(commandSource, /appendPrintPreviewBar/);
  assert.match(commandSource, /id = 'print-btn'/);
  assert.match(commandSource, /id = 'close-btn'/);
});

test('print pipeline은 저장 handle·파일명·dirty 상태를 변경하지 않는다', () => {
  const printSection = commandSource.slice(
    commandSource.indexOf('async function preparePrintPages'),
    commandSource.indexOf('export const fileCommands'),
  );
  assert.doesNotMatch(printSection, /\.fileName\s*=/);
  assert.doesNotMatch(printSection, /\.currentFileHandle\s*=/);
  assert.doesNotMatch(printSection, /documentState\.(markDirty|markClean)\(/);
});
