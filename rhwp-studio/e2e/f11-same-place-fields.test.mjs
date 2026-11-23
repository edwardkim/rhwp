/**
 * E2E: F11로 같은 글자 자리의 빈 누름틀을 하나씩 고르고, 고른 누름틀에 입력한다.
 *
 * 한/글 12.30(Mac)은 "앞 뒤끝"의 글자 자리 2에 차례로 넣은 빈 누름틀 가·나를 ⌃F11로 나, 가 차례로 고르고,
 * 고른 누름틀에 친 글이 그 누름틀에 든다. 셀 안에서는 그 셀의 누름틀, 그 표, 표 앞 본문 누름틀 차례다.
 *
 * 시나리오 (실제 F11 키):
 *  (a) 문단 끝에서 F11 → 나, 다시 F11 → 가, 다시 F11 → 고를 것 없음
 *  (b) F11 한 번 뒤 입력 → 글이 나에 든다
 *  (c) F11 두 번 뒤 입력 → 글이 가에 든다
 *  (d) 셀 안에서 F11 → 셀 누름틀, 표, 본문 누름틀
 */
import { runTest, createNewDocument, assert } from './helpers.mjs';

const settle = (page) => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));

const f11 = async (page) => {
  await page.keyboard.press('F11');
  await settle(page);
};

/** "앞 뒤끝" 글자 자리 2에 빈 누름틀 가·나를 차례로 넣고 캐럿을 문단 끝에 둔다. */
const buildSamePlace = (page) => createNewDocument(page).then(() => page.evaluate(() => {
  const w = window.__wasm;
  const ih = window.__inputHandler;
  w.doc.insertText(0, 0, 0, '앞 뒤끝');
  for (const name of ['가', '나']) {
    const r = JSON.parse(w.doc.insertClickHereField(0, 0, 2, `${name}안내`, '', name, true));
    if (!r.ok) throw new Error(`insertClickHereField ${name}: ${JSON.stringify(r)}`);
  }
  window.__canvasView?.loadDocument?.();
  ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 4 });
  ih.updateCaret();
  ih.focus();
}));

/** 지금 F11이 고른 것: 누름틀이면 그 이름, 표면 'table', 없으면 null. */
const picked = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  if (ih.cursor.isInTableObjectSelection()) return 'table';
  const pin = ih.f11Field;
  if (!pin) return null;
  const list = JSON.parse(window.__wasm.doc.getFieldList());
  return list.find((f) => f.controlIndex === pin.controlIndex && f.fieldId === pin.info.fieldId)?.name ?? '?';
});

const values = (page) => page.evaluate(() => Object.fromEntries(
  JSON.parse(window.__wasm.doc.getFieldList()).map((f) => [f.name, f.value]),
));

runTest('F11 같은 자리 누름틀', async ({ page }) => {
  await buildSamePlace(page);
  await f11(page);
  assert(await picked(page) === '나', 'F11 한 번: 나');
  await f11(page);
  assert(await picked(page) === '가', 'F11 두 번: 가');
  await f11(page);
  assert(await picked(page) === null, 'F11 세 번: 고를 것 없음');

  await buildSamePlace(page);
  await f11(page);
  await page.keyboard.type('xy');
  await settle(page);
  const one = await values(page);
  assert(one['나'] === 'xy' && one['가'] === '', `F11 한 번 뒤 입력은 나에: ${JSON.stringify(one)}`);

  await buildSamePlace(page);
  await f11(page);
  await f11(page);
  await page.keyboard.type('z');
  await settle(page);
  const two = await values(page);
  assert(two['가'] === 'z' && two['나'] === '', `F11 두 번 뒤 입력은 가에: ${JSON.stringify(two)}`);

  // 본문 누름틀 '본문' 뒤 1x1 표, 그 셀의 "셀 끝" 글자 자리 2에 누름틀 '셀'.
  await createNewDocument(page);
  await page.evaluate(() => {
    const w = window.__wasm;
    const ih = window.__inputHandler;
    w.doc.insertText(0, 0, 0, '앞');
    JSON.parse(w.doc.insertClickHereField(0, 0, 0, '본문안내', '', '본문', true));
    const t = w.createTable(0, 0, 1, 1, 1);
    if (!t?.ok) throw new Error(`createTable: ${JSON.stringify(t)}`);
    w.doc.insertTextInCell(0, t.paraIdx, t.controlIdx, 0, 0, 0, '셀 끝');
    const r = JSON.parse(w.doc.insertClickHereFieldInCell(0, t.paraIdx, t.controlIdx, 0, 0, 2, false, '셀안내', '', '셀', true));
    if (!r.ok) throw new Error(`insertClickHereFieldInCell: ${JSON.stringify(r)}`);
    window.__canvasView?.loadDocument?.();
    ih.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 3,
      parentParaIndex: t.paraIdx, controlIndex: t.controlIdx, cellIndex: 0, cellParaIndex: 0,
    });
    ih.updateCaret();
    ih.focus();
  });
  await f11(page);
  assert(await picked(page) === '셀', '셀 안 F11: 셀 누름틀');
  await f11(page);
  assert(await picked(page) === 'table', '셀 안 F11 두 번: 그 표');
  await f11(page);
  assert(await picked(page) === '본문', '셀 안 F11 세 번: 표 앞 본문 누름틀');
});
