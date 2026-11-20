/** #9011 실제 메모 명령/대화상자와 snapshot undo, 저장 재열기 계약. */
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runTest, createNewDocument } from './helpers.mjs';

const output = resolve('../output/issue9011');
const state = page => page.evaluate(() => window.__wasm.getMemos().map(({
  id, text, author, sectionIndex, paraIndex, startCharIdx, endCharIdx, path,
}) => ({ id, text, author, sectionIndex, paraIndex, startCharIdx, endCharIdx, path })));
const dirty = page => page.evaluate(() => window.__documentState.isDirty());
const contentAndAnchor = memos => memos.map(({ id, ...memo }) => memo)
  .sort((a, b) => a.text.localeCompare(b.text));
const selectMemo = (page, id) => page.select('select[aria-label="문서의 메모"]', String(id));
const deleteMemo = page => page.click('[data-memo-dialog] .dialog-body button');
const open = async page => {
  await page.click('#menu-bar .menu-item[data-menu="insert"] .menu-title');
  await page.click('#menu-bar .md-item[data-cmd="insert:comment"]');
  await page.waitForSelector('#memo-text');
};
const close = async page => {
  await page.keyboard.press('Escape');
  await page.waitForSelector('#memo-text', { hidden: true });
};
const fill = (page, text) => page.$eval('#memo-text', (el, value) => { el.value = value; }, text);

async function saveAndReopen(page, format, pass) {
  const name = `studio-memo-${pass}.${format}`;
  // Studio의 저장 명령을 실행하고 OS 파일 선택기에 넘긴 바이트를 파일로 다시 연다.
  await page.evaluate(() => {
    window.__memoSaved = null;
    window.showSaveFilePicker = async options => ({
      name: options.suggestedName,
      async createWritable() {
        return {
          async write(blob) { window.__memoSaved = [...new Uint8Array(await blob.arrayBuffer())]; },
          async close() {},
        };
      },
    });
  });
  await page.evaluate(format => {
    document.querySelector('#menu-bar .menu-item[data-menu="file"] .menu-title')
      .dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    document.querySelector(`.md-item[data-cmd="file:save-as-${format}"]`).click();
  }, format);
  await page.waitForSelector('.dialog-btn-primary');
  await page.click('.dialog-btn-primary');
  await page.waitForFunction(() => window.__memoSaved !== null && !window.__documentState.isDirty());
  const bytes = Buffer.from(await page.evaluate(() => window.__memoSaved));
  assert.equal(bytes.subarray(0, format === 'hwp' ? 4 : 2).toString('hex'), format === 'hwp' ? 'd0cf11e0' : '504b');
  const path = resolve(output, name);
  writeFileSync(path, bytes);
  await (await page.$('#file-input')).uploadFile(path);
  await page.waitForFunction(name => window.__wasm.fileName === name && !!window.__inputHandler, {}, name);
  await page.waitForFunction(() => !window.__documentState.isDirty());
}

await runTest('#9011 메모 UI CRUD·undo·redo·저장', async ({page}) => {
  mkdirSync(output, { recursive: true });
  await createNewDocument(page);
  await page.evaluate(async () => {
    window.__wasm.doc.insertText(0,0,0,'메모 기준 글자');
    await window.__canvasView.loadDocument();
    const ih = window.__inputHandler;
    ih.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:0});
    ih.cursor.setAnchor();
    ih.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:2});
    ih.updateCaret();
  });
  assert.equal(await dirty(page), false);
  await open(page);
  await fill(page,'취소한 초안');
  await close(page);
  assert.equal((await state(page)).length,0,'닫기는 반영하지 않은 입력을 저장하지 않는다');
  assert.equal(await dirty(page), false);
  await open(page);
  await page.click('.dialog-btn-primary');
  assert.equal((await state(page)).length, 0, '빈 본문 거부');
  assert.match(await page.$eval('[role="alert"]', e=>e.textContent), /내용/);
  assert.equal(await dirty(page), false);
  await page.type('#memo-author', '테스트');
  await fill(page, '<script>메모</script>\n둘째 줄');
  await page.click('.dialog-btn-primary');
  const inserted = await state(page);
  assert.equal(inserted.length,1);
  assert.equal(inserted[0].text,'<script>메모</script>\n둘째 줄');
  assert.deepEqual(contentAndAnchor(inserted), [{
    text: '<script>메모</script>\n둘째 줄', author: '테스트', sectionIndex: 0, paraIndex: 0,
    startCharIdx: 0, endCharIdx: 2, path: null,
  }]);
  assert.equal(await dirty(page), true);
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.equal((await state(page)).length,0);
  await page.evaluate(()=>window.__inputHandler.performRedo());
  assert.deepEqual(await state(page),inserted);

  await open(page);
  await selectMemo(page, inserted[0].id);
  await page.screenshot({ path: resolve(output, 'memo-dialog.png') });
  await fill(page,'수정한 메모');
  await page.select('select[aria-label="문서의 메모"]','');
  await selectMemo(page, inserted[0].id);
  assert.equal(await page.$eval('#memo-text',e=>e.value),'수정한 메모','목록 전환은 미반영 입력을 보존한다');
  await page.click('.dialog-btn-primary');
  assert.equal((await state(page))[0].text,'수정한 메모');
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.deepEqual(await state(page),inserted);
  await page.evaluate(()=>window.__inputHandler.performRedo());

  await open(page);
  await selectMemo(page, inserted[0].id);
  await deleteMemo(page);
  assert.equal((await state(page)).length,0);
  await close(page);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.equal((await state(page))[0].text, '수정한 메모');
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.equal((await state(page)).length, 0);
  await page.evaluate(() => window.__inputHandler.performUndo());
  await page.evaluate(()=>{
    const ih=window.__inputHandler; ih.cursor.clearSelection();
    ih.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:8}); ih.updateCaret();
  });
  await open(page); await fill(page,'두 번째 메모'); await page.click('.dialog-btn-primary');
  const multiple=await state(page); assert.equal(multiple.length,2);
  const second=multiple.find(m=>m.text==='두 번째 메모');
  assert.deepEqual([second.startCharIdx, second.endCharIdx], [8, 8]);
  await selectMemo(page, second.id);
  await deleteMemo(page);
  assert.equal((await state(page)).length,1); await close(page);
  await open(page);
  await page.evaluate(()=>window.__wasm.doc.insertText(0,0,0,'외부 변경'));
  await fill(page,'오래된 범위');
  await page.click('.dialog-btn-primary');
  assert.match(await page.$eval('[role="alert"]',e=>e.textContent),/바뀌었습니다/);
  assert.equal((await state(page)).length,1);
  await close(page);
  await open(page);
  await page.evaluate(()=>window.__wasm.createNewDocument());
  await fill(page,'이전 문서의 초안');
  await page.click('.dialog-btn-primary');
  assert.match(await page.$eval('[role="alert"]',e=>e.textContent),/바뀌었습니다/);
  assert.equal((await state(page)).length,0);
  await close(page);

  await createNewDocument(page);
  const table = await page.evaluate(async () => {
    const wasm = window.__wasm;
    wasm.doc.insertText(0, 0, 0, '본문 기준 글자');
    wasm.splitParagraph(0, 0, 8);
    const table = wasm.createTable(0, 1, 0, 1, 2);
    if (!table.ok) throw new Error('createTable failed');
    wasm.insertTextInCell(0, table.paraIdx, table.controlIdx, 1, 0, 0, '셀 기준 글자');
    await window.__canvasView.loadDocument();
    return table;
  });

  const position = async (start, end = start) => {
    await page.evaluate(({ start, end }) => {
      const input = window.__inputHandler;
      input.cursor.clearSelection();
      input.cursor.moveTo(start);
      if (start !== end) {
        input.cursor.setAnchor();
        input.cursor.moveTo(end);
      }
      input.updateCaret();
      input.focus();
    }, { start, end });
  };
  const body = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
  await position(body, { ...body, paragraphIndex: 1 });
  await page.click('#menu-bar .menu-item[data-menu="insert"] .menu-title');
  await page.click('#menu-bar .md-item[data-cmd="insert:comment"]');
  await page.waitForFunction(() => document.querySelector('#rhwp-toast-container')?.textContent.includes('한 문단'));
  assert.equal(await page.$('#memo-text'), null, '여러 문단 선택 거절');
  assert.deepEqual(await state(page), []);
  assert.equal(await dirty(page), false);

  const cell = (cellIndex, charOffset) => ({
    sectionIndex: 0, paragraphIndex: 0, charOffset, parentParaIndex: table.paraIdx,
    controlIndex: table.controlIdx, cellIndex, cellParaIndex: 0,
    cellPath: [{ controlIndex: table.controlIdx, cellIndex, cellParaIndex: 0 }],
  });
  const add = async (text) => {
    await open(page);
    await fill(page, text);
    await page.click('.dialog-btn-primary');
    await page.waitForFunction(() => document.querySelector('[data-memo-dialog] [role="alert"]')
      ?.textContent.includes('반영했습니다'));
    await close(page);
  };
  await position(body, { ...body, charOffset: 2 });
  await add('본문 선택 메모');
  await position(cell(1, 1), cell(1, 4));
  await add('둘째 셀 선택 메모');
  await position(cell(0, 0));
  await add('빈 셀 커서 메모');

  const expected = [
    { text: '본문 선택 메모', author: '', sectionIndex: 0, paraIndex: 0,
      startCharIdx: 0, endCharIdx: 2, path: null },
    ...[[1, 1, 4, '둘째 셀 선택 메모'], [0, 0, 0, '빈 셀 커서 메모']].map(([cellIndex, start, end, text]) => ({
      text, author: '', sectionIndex: 0, paraIndex: table.paraIdx, startCharIdx: start,
      endCharIdx: end, path: [{ type: 'cell', controlIndex: table.controlIdx, cellIndex, paraIndex: 0 }],
    })),
  ].sort((a, b) => a.text.localeCompare(b.text));
  assert.deepEqual(contentAndAnchor(await state(page)), expected);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.deepEqual(contentAndAnchor(await state(page)), expected.filter(memo => memo.text !== '빈 셀 커서 메모'));
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.deepEqual(contentAndAnchor(await state(page)), expected);

  for (const format of ['hwp', 'hwpx']) {
    for (const pass of [1, 2]) {
      await saveAndReopen(page, format, pass);
      const reopened = await state(page);
      assert.deepEqual(contentAndAnchor(reopened), expected, `${format} 저장·재열기 ${pass}회차`);
      await open(page);
      const memo = reopened.find(m => m.text === '둘째 셀 선택 메모');
      await selectMemo(page, memo.id);
      assert.equal(await page.$eval('#memo-text', el => el.value), memo.text);
      assert.match(await page.$eval('select[aria-label="문서의 메모"]', el => el.selectedOptions[0].textContent), /표 셀 2/);
      await close(page);
    }
  }
});
