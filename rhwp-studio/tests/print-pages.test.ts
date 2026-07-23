import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildPrintStyleText,
  createPrintPage,
  namespaceSvgReferenceValue,
  PDF_PRINT_GUIDANCE,
  printProgressText,
  printReadyText,
  pxToPrintMm,
} from '../src/command/print-pages.ts';

test('pxToPrintMm는 CSS px를 인쇄 mm로 변환한다', () => {
  assert.equal(pxToPrintMm(96), 25.4);
  assert.equal(pxToPrintMm(793.7008), 210);
});

test('createPrintPage는 페이지마다 독립된 named page와 크기를 만든다', () => {
  const portrait = createPrintPage('<svg />', { width: 793.7008, height: 1122.5197 }, 0);
  const landscape = createPrintPage('<svg />', { width: 1122.5197, height: 793.7008 }, 1);

  assert.equal(portrait.pageName, 'rhwp-print-page-1');
  assert.equal(landscape.pageName, 'rhwp-print-page-2');
  assert.equal(portrait.widthMm, 210);
  assert.equal(portrait.heightMm, 297);
  assert.equal(landscape.widthMm, 297);
  assert.equal(landscape.heightMm, 210);
});

test('buildPrintStyleText는 혼합 방향 페이지의 @page size를 페이지별로 보존한다', () => {
  const pages = [
    createPrintPage('<svg />', { width: 793.7008, height: 1122.5197 }, 0),
    createPrintPage('<svg />', { width: 1122.5197, height: 793.7008 }, 1),
    createPrintPage('<svg />', { width: 793.7008, height: 1122.5197 }, 2),
  ];

  const css = buildPrintStyleText(pages);

  assert.match(css, /@page rhwp-print-page-1 \{ size: 210mm 297mm; margin: 0; \}/);
  assert.match(css, /@page rhwp-print-page-2 \{ size: 297mm 210mm; margin: 0; \}/);
  assert.match(css, /\.rhwp-print-page-2 \{ page: rhwp-print-page-2; width: 297mm; height: 210mm; \}/);
  assert.match(css, /\.print-preview-bar/);
  assert.match(css, /@media print \{ \.print-preview-bar \{ display: none !important; \} \}/);
  assert.equal(css.includes('@page { size:'), false);
});

test('namespaceSvgReferenceValue는 SVG url/hash 참조를 페이지별 id로 바꾼다', () => {
  const idMap = new Map([
    ['body-clip-3', 'rhwp-print-page-2-body-clip-3'],
    ['grad.1', 'rhwp-print-page-2-grad.1'],
  ]);

  assert.equal(
    namespaceSvgReferenceValue('url(#body-clip-3)', idMap),
    'url(#rhwp-print-page-2-body-clip-3)',
  );
  assert.equal(
    namespaceSvgReferenceValue("clip-path: url('#body-clip-3'); fill: url(#grad.1)", idMap),
    'clip-path: url(#rhwp-print-page-2-body-clip-3); fill: url(#rhwp-print-page-2-grad.1)',
  );
  assert.equal(
    namespaceSvgReferenceValue('#body-clip-3', idMap),
    '#rhwp-print-page-2-body-clip-3',
  );
});

test('PDF 인쇄 의도는 준비 상태와 남은 브라우저 단계를 명확히 안내한다', () => {
  assert.equal(printProgressText('pdf', 2, 7), 'PDF 준비 중… (2/7)');
  assert.equal(
    printReadyText('pdf'),
    `PDF 준비 완료 — ${PDF_PRINT_GUIDANCE}`,
  );
  assert.match(PDF_PRINT_GUIDANCE, /대상 → PDF로 저장/);
});

test('일반 인쇄는 같은 진행 helper를 쓰되 PDF 안내를 표시하지 않는다', () => {
  assert.equal(printProgressText('print', 1, 3), '인쇄 준비 중… (1/3)');
  assert.equal(printReadyText('print'), '인쇄 미리보기 준비 완료');
});
