export interface PrintPageInfo {
  width: number;
  height: number;
}

export interface PrintPage {
  svg: string;
  widthMm: number;
  heightMm: number;
  pageName: string;
  className: string;
}

export type PrintIntent = 'print' | 'pdf';

export const PDF_PRINT_GUIDANCE =
  '브라우저 인쇄 창에서 ‘대상 → PDF로 저장’을 선택합니다.';

export function pdfPrintTitle(fileName: string): string {
  const baseName = fileName.trim().replace(/\.(hwp|hwpx|hml)$/i, '').trim();
  return baseName || '문서';
}

export function printProgressText(
  intent: PrintIntent,
  currentPage: number,
  pageCount: number,
): string {
  const label = intent === 'pdf' ? 'PDF 준비 중…' : '인쇄 준비 중…';
  return `${label} (${currentPage}/${pageCount})`;
}

export function printReadyText(intent: PrintIntent): string {
  return intent === 'pdf'
    ? `PDF 준비 완료 — ${PDF_PRINT_GUIDANCE}`
    : '인쇄 미리보기 준비 완료';
}

export function pxToPrintMm(px: number): number {
  return Math.round((px * 25.4 / 96) * 1000) / 1000;
}

function formatMm(mm: number): string {
  return Number.isInteger(mm)
    ? String(mm)
    : mm.toFixed(3).replace(/0+$/, '').replace(/\.$/, '');
}

export function createPrintPage(svg: string, pageInfo: PrintPageInfo, pageIndex: number): PrintPage {
  return {
    svg,
    widthMm: pxToPrintMm(pageInfo.width),
    heightMm: pxToPrintMm(pageInfo.height),
    pageName: `rhwp-print-page-${pageIndex + 1}`,
    className: `rhwp-print-page-${pageIndex + 1}`,
  };
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function namespaceSvgReferenceValue(value: string, idMap: Map<string, string>): string {
  let next = value;
  for (const [oldId, newId] of idMap) {
    const escapedOldId = escapeRegExp(oldId);
    next = next.replace(
      new RegExp(`url\\((['"]?)#${escapedOldId}\\1\\)`, 'g'),
      () => `url(#${newId})`,
    );
    if (next === `#${oldId}`) {
      next = `#${newId}`;
    }
  }
  return next;
}

function namespaceSvgIds(root: Element, namespace: string): void {
  const idElements = [
    ...(root.hasAttribute('id') ? [root] : []),
    ...Array.from(root.querySelectorAll('[id]')),
  ];
  const idMap = new Map<string, string>();

  for (const element of idElements) {
    const id = element.getAttribute('id');
    if (id) {
      idMap.set(id, `${namespace}-${id}`);
    }
  }
  if (idMap.size === 0) return;

  const allElements = [root, ...Array.from(root.querySelectorAll('*'))];
  for (const element of allElements) {
    for (const attr of Array.from(element.attributes)) {
      if (attr.name === 'id') {
        const nextId = idMap.get(attr.value);
        if (nextId) element.setAttribute(attr.name, nextId);
        continue;
      }

      const nextValue = namespaceSvgReferenceValue(attr.value, idMap);
      if (nextValue !== attr.value) {
        element.setAttribute(attr.name, nextValue);
      }
    }
  }
}

export function buildPrintStyleText(pages: PrintPage[]): string {
  const pageRules = pages
    .map((page) => `@page ${page.pageName} { size: ${formatMm(page.widthMm)}mm ${formatMm(page.heightMm)}mm; margin: 0; }`)
    .join('\n');
  const pageSizeRules = pages
    .map((page) => `.${page.className} { page: ${page.pageName}; width: ${formatMm(page.widthMm)}mm; height: ${formatMm(page.heightMm)}mm; }`)
    .join('\n');

  return `
${pageRules}
* { margin: 0; padding: 0; }
body { background: #fff; }
.page { break-after: page; page-break-after: always; overflow: hidden; }
${pageSizeRules}
.page:last-child { break-after: auto; page-break-after: auto; }
.page svg { width: 100%; height: 100%; }
@media screen {
  body { background: #e5e7eb; display: flex; flex-direction: column; align-items: center; gap: 16px; padding: 16px; }
  .page { background: #fff; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
  body.rhwp-print-preview { padding-top: 72px; }
  .print-preview-bar {
    position: fixed;
    inset: 0 0 auto 0;
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 48px;
    padding: 8px 16px;
    background: #1e293b;
    color: #f8fafc;
    box-shadow: 0 2px 8px rgba(15,23,42,0.28);
    font: 14px/1.4 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }
  .print-preview-bar button {
    min-width: 72px;
    height: 32px;
    padding: 0 14px;
    border: 1px solid #64748b;
    border-radius: 5px;
    background: #475569;
    color: #fff;
    cursor: pointer;
    font: inherit;
  }
  .print-preview-bar button:hover { background: #64748b; }
  .print-preview-bar button:focus-visible { outline: 2px solid #93c5fd; outline-offset: 2px; }
  .print-preview-bar .print-preview-primary { background: #2563eb; border-color: #60a5fa; }
  .print-preview-bar .print-preview-primary:hover { background: #1d4ed8; }
  .print-preview-title {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #e2e8f0;
  }
}
@media print { .print-preview-bar { display: none !important; } }
`;
}

export function appendPrintStyle(doc: Document, pages: PrintPage[]): void {
  const style = doc.createElement('style');
  style.textContent = buildPrintStyleText(pages);
  doc.head.appendChild(style);
}

export function appendSvgPage(doc: Document, container: HTMLElement, printPage: PrintPage): void {
  const page = doc.createElement('div');
  page.className = `page ${printPage.className}`;

  const parsed = new DOMParser().parseFromString(printPage.svg, 'image/svg+xml');
  const parseError = parsed.querySelector('parsererror');
  if (parseError) {
    throw new Error(`인쇄용 SVG 파싱 실패: ${parseError.textContent || 'parsererror'}`);
  }

  namespaceSvgIds(parsed.documentElement, printPage.pageName);
  page.appendChild(doc.importNode(parsed.documentElement, true));
  container.appendChild(page);
}
