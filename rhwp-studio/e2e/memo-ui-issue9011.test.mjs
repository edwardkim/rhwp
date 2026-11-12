/** #9011 실제 메모 명령/대화상자와 snapshot undo, 저장 재열기 계약. */
import assert from 'node:assert/strict';
import {mkdirSync} from 'node:fs';
import { runTest, createNewDocument } from './helpers.mjs';

const state = page => page.evaluate(() => window.__wasm.getMemos().map(({id,text,author}) => ({id,text,author})));
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

await runTest('#9011 메모 UI CRUD·undo·redo·저장', async ({page}) => {
  mkdirSync('../output/issue9011', {recursive:true});
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
  await open(page);
  await fill(page,'취소한 초안');
  await close(page);
  assert.equal((await state(page)).length,0,'닫기는 반영하지 않은 입력을 저장하지 않는다');
  await open(page);
  await page.click('.dialog-btn-primary');
  assert.equal((await state(page)).length, 0, '빈 본문 거부');
  assert.match(await page.$eval('[role="alert"]', e=>e.textContent), /내용/);
  await page.type('#memo-author', '테스트');
  await fill(page, '<script>메모</script>\n둘째 줄');
  await page.click('.dialog-btn-primary');
  const inserted = await state(page);
  assert.equal(inserted.length,1);
  assert.equal(inserted[0].text,'<script>메모</script>\n둘째 줄');
  assert.equal(await page.evaluate(()=>window.__documentState.isDirty()),true);
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.equal((await state(page)).length,0);
  await page.evaluate(()=>window.__inputHandler.performRedo());
  assert.deepEqual(await state(page),inserted);

  await open(page);
  await page.select('select[aria-label="문서의 메모"]',String(inserted[0].id));
  await page.screenshot({path:'../output/issue9011/memo-dialog.png'});
  await fill(page,'수정한 메모');
  await page.select('select[aria-label="문서의 메모"]','');
  await page.select('select[aria-label="문서의 메모"]',String(inserted[0].id));
  assert.equal(await page.$eval('#memo-text',e=>e.value),'수정한 메모','목록 전환은 미반영 입력을 보존한다');
  await page.click('.dialog-btn-primary');
  assert.equal((await state(page))[0].text,'수정한 메모');
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.deepEqual(await state(page),inserted);
  await page.evaluate(()=>window.__inputHandler.performRedo());

  for(const format of ['Hwp','Hwpx']) {
    // 별도 WASM 인스턴스로 재열어 현재 편집 세션을 바꾸지 않는다.
    const roundtrip=await page.evaluate(format=>{
      const doc=window.__wasm.doc;
      const bytes=window.__wasm['export'+format]();
      const reopened=new doc.constructor(bytes);
      try {
        const again=new doc.constructor(reopened['export'+format]());
        try { return [JSON.parse(reopened.getMemoList()).map(m=>m.text),
          JSON.parse(again.getMemoList()).map(m=>m.text)]; } finally {again.free();}
      } finally {reopened.free();}
    },format);
    assert.deepEqual(roundtrip,[['수정한 메모'],['수정한 메모']],`${format} 저장/재열기`);
  }
  await open(page);
  await page.select('select[aria-label="문서의 메모"]',String(inserted[0].id));
  await page.$$eval('.dialog-body button',buttons=>buttons.find(b=>b.textContent==='선택한 메모 삭제').click());
  assert.equal((await state(page)).length,0);
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.equal((await state(page))[0].text,'수정한 메모');
  await page.evaluate(()=>{
    const ih=window.__inputHandler; ih.cursor.clearSelection();
    ih.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:8}); ih.updateCaret();
  });
  await open(page); await fill(page,'두 번째 메모'); await page.click('.dialog-btn-primary');
  const multiple=await state(page); assert.equal(multiple.length,2);
  const second=multiple.find(m=>m.text==='두 번째 메모');
  await page.select('select[aria-label="문서의 메모"]',String(second.id));
  await page.$$eval('.dialog-body button',buttons=>buttons.find(b=>b.textContent==='선택한 메모 삭제').click());
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
  await page.evaluate(async () => {
    const result=window.__wasm.createTable(0,0,0,1,1);
    if(!result.ok) throw new Error('createTable failed');
    await window.__canvasView.loadDocument();
    const ih=window.__inputHandler;
    ih.cursor.moveTo({sectionIndex:0,paragraphIndex:0,charOffset:0,
      parentParaIndex:result.paraIdx,controlIndex:result.controlIdx,cellIndex:0,cellParaIndex:0,
      cellPath:[{controlIndex:result.controlIdx,cellIndex:0,cellParaIndex:0}]});
    ih.updateCaret(); ih.focus();
  });
  await open(page);
  await fill(page,'빈 셀의 메모');
  await page.click('.dialog-btn-primary');
  assert.equal((await state(page))[0].text,'빈 셀의 메모');
  assert.equal(await page.evaluate(()=>window.__wasm.getMemos()[0].path[0].type),'cell');
  await close(page);
  await page.evaluate(()=>window.__inputHandler.performUndo());
  assert.equal((await state(page)).length,0);
  await page.evaluate(()=>window.__inputHandler.performRedo());
  assert.equal((await state(page))[0].text,'빈 셀의 메모');

});
