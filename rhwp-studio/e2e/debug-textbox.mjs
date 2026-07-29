/**
 * E2E 디버그: 글상자 삽입 후 텍스트 위치 확인
 */
import { runTest, createNewDocument } from './helpers.mjs';

runTest('디버그: 글상자 위치', async ({ page }) => {
  await createNewDocument(page);

  const result = await page.evaluate(() => {
    const w = window.__wasm;
    w.doc.insertText(0, 0, 0, 'Before');
    w.doc.splitParagraph(0, 0, 6);
    w.doc.splitParagraph(0, 1, 0);
    w.doc.insertText(0, 2, 0, 'After');

    JSON.parse(w.doc.createShapeControl(JSON.stringify({
      sectionIdx: 0, paraIdx: 1, charOffset: 0,
      width: 21600, height: 7200,
      shapeType: 'textbox', textWrap: 'TopAndBottom',
    })));

    const svg = w.doc.renderPageSvg(0);
    const textElements = svg.match(/<text[^>]+y="([^"]+)"[^>]*>([^<])<\/text>/g) || [];
    const positions = textElements.map(t => {
      const yM = t.match(/y="([^"]+)"/);
      const chM = t.match(/>([^<])</);
      return { y: yM?.[1], ch: chM?.[1] };
    });
    const rects = svg.match(/<rect[^/]*\/>/g) || [];
    return { positions: positions.slice(0, 20), rects: rects.slice(0, 5), svgLen: svg.length };
  });
  console.log(JSON.stringify(result, null, 2));
});
