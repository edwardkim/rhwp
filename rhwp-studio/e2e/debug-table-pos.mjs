/**
 * E2E 디버그: 표 삽입 후 텍스트 위치 확인
 */
import { runTest, createNewDocument } from './helpers.mjs';

runTest('디버그: 표 위치', async ({ page }) => {
  await createNewDocument(page);

  const result = await page.evaluate(() => {
    const w = window.__wasm;
    w.doc.insertText(0, 0, 0, 'Before table');
    w.doc.splitParagraph(0, 0, 12);
    w.doc.splitParagraph(0, 1, 0);
    w.doc.insertText(0, 2, 0, 'After table');

    const tr = JSON.parse(w.doc.createTable(0, 1, 0, 2, 2));
    w.doc.insertTextInCell(0, tr.paraIdx, tr.controlIdx, 0, 0, 0, 'Cell');
    window.__eventBus?.emit('document-changed');

    const svg = w.doc.renderPageSvg(0);
    const textElements = svg.match(/<text[^>]+y="([^"]+)"[^>]*>([^<])<\/text>/g) || [];
    const positions = textElements.map(t => {
      const yM = t.match(/y="([^"]+)"/);
      const chM = t.match(/>([^<])</);
      return { y: yM?.[1], ch: chM?.[1] };
    });
    const rects = svg.match(/<rect[^/]*\/>/g) || [];
    const tableRects = rects.filter(r => r.includes('stroke'));
    return { positions: positions.slice(0, 20), tableRects: tableRects.slice(0, 5), svgLen: svg.length };
  });
  console.log(JSON.stringify(result, null, 2));
});
