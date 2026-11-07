/**
 * 메모 기준 글자를 실제 Backspace·Delete 로 지우고 Ctrl+Z·Ctrl+Y 로 되돌린다.
 *
 * 코어는 기준 글자가 모두 지워진 메모를 같은 편집에서 지운다. 글자 재삽입만 하는 undo 는
 * 메모를 되살리지 못하고, 기준 바로 앞·뒤 글자의 재삽입은 메모 범위에 붙는다. 메모 준비는
 * 기존 e2e 관례대로 원시 WASM 호출로 하고(undo 스택 밖), 편집과 undo/redo 는 실제 키로 한다.
 */
import assert from 'node:assert/strict';
import { runTest, createNewDocument } from './helpers.mjs';

const sleep = (page, ms) => page.evaluate(t => new Promise(r => setTimeout(r, t)), ms);

async function press(page, key, times = 1) {
  for (let i = 0; i < times; i++) {
    await page.keyboard.press(key);
    await sleep(page, 80);
  }
}

async function chord(page, key, times = 1) {
  for (let i = 0; i < times; i++) {
    await page.keyboard.down('Control');
    await page.keyboard.press(key);
    await page.keyboard.up('Control');
    await sleep(page, 120);
  }
}

/** 새 문서에 `ab memo cd` 를 쓰고 `memo`(3..7)에 메모를 단다. 커서는 `offset` 에 둔다. */
async function prepare(page, offset) {
  await createNewDocument(page);
  await page.evaluate(async (caret) => {
    const doc = window.__wasm.doc;
    doc.insertText(0, 0, 0, 'ab memo cd');
    const inserted = JSON.parse(doc.insertMemo(0, 0, 3, 7, JSON.stringify({ author: 'e2e', text: '메모 본문' })));
    if (!inserted.ok) throw new Error('insertMemo failed');
    await window.__canvasView.loadDocument();
    const input = window.__inputHandler;
    input.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: caret });
    input.updateCaret();
    input.focus();
  }, offset);
  await sleep(page, 200);
}

const state = page => page.evaluate(() => ({
  text: window.__wasm.getTextRange(0, 0, 0, window.__wasm.getParagraphLength(0, 0)),
  memos: JSON.parse(window.__wasm.doc.getMemoList())
    .map(({ text, startCharIdx, endCharIdx }) => ({ text, startCharIdx, endCharIdx })),
}));

const initial = { text: 'ab memo cd', memos: [{ text: '메모 본문', startCharIdx: 3, endCharIdx: 7 }] };

runTest('메모 기준 글자의 Backspace·Delete 삭제와 Ctrl+Z·Ctrl+Y', async ({ page }) => {
  // Backspace 로 기준 끝에서 한 글자씩 지운다. 메모에 닿는 삭제는 한 글자마다 undo 한 단계다.
  await prepare(page, 7);
  assert.deepEqual(await state(page), initial);
  await press(page, 'Backspace', 4);
  assert.deepEqual(await state(page), { text: 'ab  cd', memos: [] }, '기준 전체를 지우면 메모도 사라진다');
  await chord(page, 'KeyZ');
  assert.deepEqual(await state(page), {
    text: 'ab m cd', memos: [{ text: '메모 본문', startCharIdx: 3, endCharIdx: 4 }],
  }, '마지막 글자 삭제의 undo 는 메모를 되살린다');
  await chord(page, 'KeyZ', 3);
  assert.deepEqual(await state(page), initial);
  await chord(page, 'KeyY', 4);
  assert.deepEqual(await state(page), { text: 'ab  cd', memos: [] });
  await chord(page, 'KeyZ', 4);
  assert.deepEqual(await state(page), initial);

  // Delete 로 기준 시작에서 앞쪽 글자부터 지운다.
  await prepare(page, 3);
  await press(page, 'Delete', 4);
  assert.deepEqual(await state(page), { text: 'ab  cd', memos: [] });
  await chord(page, 'KeyZ', 4);
  assert.deepEqual(await state(page), initial);

  // 기준 바로 앞·뒤 글자 삭제의 undo 는 메모 범위를 넓히지 않는다.
  await prepare(page, 3);
  await press(page, 'Backspace');
  await chord(page, 'KeyZ');
  assert.deepEqual(await state(page), initial, '기준 바로 앞 글자');
  await prepare(page, 7);
  await press(page, 'Delete');
  await chord(page, 'KeyZ');
  assert.deepEqual(await state(page), initial, '기준 바로 뒤 글자');

  // 메모에서 떨어진 글자 삭제는 기존처럼 연속 삭제를 한 단계로 합친다.
  await prepare(page, 2);
  await press(page, 'Backspace', 2);
  assert.deepEqual(await state(page), {
    text: ' memo cd', memos: [{ text: '메모 본문', startCharIdx: 1, endCharIdx: 5 }],
  });
  await chord(page, 'KeyZ');
  assert.deepEqual(await state(page), initial, '병합된 일반 삭제는 한 번에 되돌린다');
});
