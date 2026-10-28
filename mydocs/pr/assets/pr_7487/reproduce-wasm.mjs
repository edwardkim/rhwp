// PR #7487의 합성 빈 문단 경계와 저장 문서 대조군을 직접 WASM API로 재실행한다.
// 브라우저의 입력·스크롤·캐럿 DOM 시험을 대신하지 않는다.
// 사용: node mydocs/pr/assets/pr_7487/reproduce-wasm.mjs
// 선행: 저장소 루트 wrapper로 검토 source의 pkg/를 새로 빌드한다.
import fs from 'node:fs/promises';
import init, { HwpDocument } from '../../../../pkg/rhwp.js';
await init({module_or_path: await fs.readFile(new URL('../../../../pkg/rhwp_bg.wasm', import.meta.url))});
const engine = [];
for (const [spacing, boundary] of [[200, 33], [300, 22]]) {
  const doc = HwpDocument.createEmpty();
  doc.createBlankDocument();
  doc.applyParaFormat(0, 0, JSON.stringify({lineSpacing: spacing, lineSpacingType: 'Percent'}));
  const rows = [];
  for (let i = 1; i <= boundary + 1; i++) {
    const split = JSON.parse(doc.splitParagraph(0, i - 1, 0, undefined));
    if (i < boundary - 1) continue;
    let cursor;
    try { cursor = JSON.parse(doc.getCursorRect(0, i, 0)); }
    catch (error) { cursor = {error: String(error)}; }
    rows.push({enter: i, split, pages: doc.pageCount(), cursor});
  }
  engine.push({spacing, rows});
  doc.free();
}
const control = new HwpDocument(await fs.readFile(new URL('../../../../samples/p122.hwp', import.meta.url)));
const out = new URL('../../../../output/pr-review/pr7487-review-20261001/wasm-replay/', import.meta.url);
await fs.mkdir(out, {recursive: true});
for (let i = 0; i < control.pageCount(); i++) {
  await fs.writeFile(new URL(`render_tree_${String(i + 1).padStart(3, '0')}.json`, out), control.getPageRenderTree(i));
}
await fs.writeFile(new URL('rhwp_002.svg', out), control.renderPageSvg(1));
console.log(JSON.stringify({engine, p122PageCount: control.pageCount(), outputDirectory: out.pathname}, null, 2));
control.free();
