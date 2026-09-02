import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildPrintStyleText,
  createPrintPage,
  namespaceSvgReferenceValue,
  PDF_PRINT_GUIDANCE,
  pdfPrintTitle,
  printProgressText,
  printReadyText,
  pxToPrintMm,
  snapToStandardPaperMm,
} from '../src/command/print-pages.ts';

test('pxToPrintMm는 CSS px를 인쇄 mm로 변환한다', () => {
  assert.equal(pxToPrintMm(96), 25.4);
  assert.equal(pxToPrintMm(793.7008), 210);
});

test('pxToPrintMm는 실제 wire 값에서도 표준 판형 mm를 복원한다 (#6561)', () => {
  // getPageInfo 는 px 를 소수 1자리로 직렬화한다. A4 는 정수 HWPUNIT
  // 양자화(84188=296.9966mm)까지 겹쳐 1122.5px 로 들어온다 — 이 값이
  // 296.995mm 가 아니라 297mm 로 나와야 판형 정체성이 산다.
  assert.equal(pxToPrintMm(1122.5), 297);
  assert.equal(pxToPrintMm(793.7), 210);
  // Letter(8.5×11in)는 유닛이 정확히 떨어진다 — 스냅이 정확한 값을 흔들면 안 된다
  assert.equal(pxToPrintMm(816), 215.9);
  assert.equal(pxToPrintMm(1056), 279.4);
});

test('snapToStandardPaperMm는 허용 오차 밖의 크기를 보존한다 (#6561)', () => {
  // 사용자가 입력 가능한 0.1mm 단위 비표준 크기는 스냅되지 않는다
  assert.equal(snapToStandardPaperMm(296.9), 296.9);
  assert.equal(snapToStandardPaperMm(210.1), 210.1);
  assert.equal(snapToStandardPaperMm(500), 500);
  // 허용 오차(0.05mm) 안쪽 잡음만 흡수한다
  assert.equal(snapToStandardPaperMm(296.995), 297);
  assert.equal(snapToStandardPaperMm(297.005), 297);
});

test('createPrintPage는 wire 절단 px에서도 @page를 표준 A4로 만든다 (#6561)', () => {
  const page = createPrintPage('<svg />', { width: 793.7, height: 1122.5 }, 0);
  assert.equal(page.widthMm, 210);
  assert.equal(page.heightMm, 297);
  const css = buildPrintStyleText([page]);
  assert.match(css, /@page rhwp-print-page-1 \{ size: 210mm 297mm; margin: 0; \}/);
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

test('인쇄 CSS는 분할을 막는 페이지 상자 규율을 유지한다 (#6561)', () => {
  // headless Chrome 실측: 이 세 규칙이 있으면 용지 강제(A4/Letter)·여백
  // 강제·정수 px 반올림 어느 조건에서도 1쪽 조판이 2쪽으로 갈라지지 않고,
  // 규칙 없이 SVG 를 나열만 하면 A4 강제에서 2쪽으로 갈라진다.
  const css = buildPrintStyleText([
    createPrintPage('<svg />', { width: 793.7, height: 1122.5 }, 0),
  ]);
  assert.match(css, /\.page \{ break-after: page; page-break-after: always; overflow: hidden; \}/);
  assert.match(css, /@page rhwp-print-page-1 \{ size: [^}]*margin: 0; \}/);
  assert.match(css, /\.page svg \{ width: 100%; height: 100%; \}/);
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

test('PDF 기본 파일명은 원본 문서 이름을 보존하고 HWP 계열 확장자를 제거한다', () => {
  assert.equal(pdfPrintTitle('복학원서.hwp'), '복학원서');
  assert.equal(pdfPrintTitle('회의록.HWPX'), '회의록');
  assert.equal(pdfPrintTitle('양식.hml'), '양식');
  assert.equal(pdfPrintTitle('확장자 없는 문서'), '확장자 없는 문서');
  assert.equal(pdfPrintTitle(' .hwp '), '문서');
});

test('일반 인쇄는 같은 진행 helper를 쓰되 PDF 안내를 표시하지 않는다', () => {
  assert.equal(printProgressText('print', 1, 3), '인쇄 준비 중… (1/3)');
  assert.equal(printReadyText('print'), '인쇄 미리보기 준비 완료');
});
