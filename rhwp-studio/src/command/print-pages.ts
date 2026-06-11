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
  .print-bar { position: fixed; top: 0; left: 0; right: 0; background: #1e293b; color: #fff; padding: 8px 16px; display: flex; align-items: center; gap: 12px; font: 14px sans-serif; z-index: 100; }
  .print-bar button { padding: 6px 16px; background: #2563eb; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }
  .print-bar button:hover { background: #1d4ed8; }
  body { padding-top: 56px; }
}
@media print { .print-bar { display: none; } }
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
