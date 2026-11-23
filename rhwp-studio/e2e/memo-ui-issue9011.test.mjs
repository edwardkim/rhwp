/** #9011 실제 메모 명령/대화상자와 snapshot undo, 저장 재열기 계약. */
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runTest, createNewDocument, loadApp } from './helpers.mjs';

const output = resolve('../output/issue9011');
const state = page => page.evaluate(() => window.__wasm.getMemos().map(({
  id, text, author, sectionIndex, paraIndex, startCharIdx, endCharIdx, path,
}) => ({ id, text, author, sectionIndex, paraIndex, startCharIdx, endCharIdx, path })));
const dirty = page => page.evaluate(() => window.__documentState.isDirty());
const contentAndAnchor = memos => memos.map(({ id, ...memo }) => memo)
  .sort((a, b) => a.text.localeCompare(b.text));
const selectMemo = (page, id) => page.click('[data-memo-dialog] [data-memo-choice="' + id + '"]');
const activeMemo = '[data-memo-dialog] [role="option"][aria-selected="true"]';
const newMemo = '[data-memo-dialog] [data-memo-choice=""]';
const deleteMemo = page => page.click('[data-memo-dialog] .memo-delete');
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
  await page.waitForFunction(name => {
    const fallback = [...document.querySelectorAll('.modal-overlay .dialog-footer button')]
      .find(button => ['대체 글꼴로 보기', 'View with substitute fonts'].includes(button.textContent));
    if (fallback) fallback.click();
    return window.__wasm.fileName === name && !!window.__inputHandler
      && document.querySelector('#file-input').value === ''
      && !document.documentElement.classList.contains('rhwp-busy')
      && !window.__documentState.isDirty();
  }, {}, name);
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
  await page.waitForSelector('.memo-margin', { hidden: true });
  await open(page);
  const focused = () => page.evaluate(() => document.activeElement.id || document.activeElement.textContent);
  assert.equal(await focused(), 'memo-text');
  await page.keyboard.press('Tab');
  assert.equal(await focused(), '반영');
  await page.keyboard.press('Tab');
  assert.equal(await focused(), '닫기');
  await page.keyboard.down('Shift');
  await page.keyboard.press('Tab');
  assert.equal(await focused(), '반영');
  await page.keyboard.press('Tab');
  assert.equal(await focused(), 'memo-text');
  await page.keyboard.up('Shift');
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

  await page.click('#menu-bar .menu-item[data-menu="view"] .menu-title');
  await page.$eval('[data-cmd="view:memo-list"]', item => item.click());
  await page.waitForSelector('#memo-text');
  await selectMemo(page, inserted[0].id);
  assert.equal(await page.$eval(activeMemo + ' .memo-list-preview', el => el.textContent), inserted[0].text);
  assert.match(await page.$eval(activeMemo + ' .memo-list-meta', el => el.textContent), /테스트.*1쪽 · 본문/);
  assert.equal(await page.$eval('.memo-dialog-location', el => el.open), false);
  assert.equal(await page.$(activeMemo + ' script'), null, '메모 미리보기를 HTML로 해석하지 않는다');
  await page.keyboard.press('Home');
  assert.equal(await page.$eval(activeMemo, el => el.dataset.memoChoice), '');
  await page.keyboard.press('End');
  assert.equal(await page.$eval(activeMemo, el => el.dataset.memoChoice), String(inserted[0].id));
  await page.screenshot({ path: resolve(output, 'memo-dialog.png') });
  await fill(page,'수정한 메모');
  await selectMemo(page, '');
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
  assert.deepEqual([second.startCharIdx, second.endCharIdx], [6, 8]);
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
  const toggleMemoView = cmd => page.evaluate(id => {
    document.querySelector('#menu-bar .menu-item[data-menu="view"] .menu-title')
      .dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    document.querySelector('.md-item[data-cmd="' + id + '"]').click();
  }, cmd);
  const body = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
  await position(body, { ...body, paragraphIndex: 1 });
  await open(page);
  assert.equal(await page.$eval(newMemo, option => option.disabled), true);
  assert.equal(await page.$eval('[data-memo-dialog] .dialog-btn-primary', button => button.disabled), true);
  assert.match(await page.$eval('[data-memo-dialog] .dialog-body p', el => el.textContent), /한 문단/);
  assert.deepEqual(await state(page), []);
  assert.equal(await dirty(page), false);
  await close(page);

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
  const [bodyMemo] = await state(page);
  await position(body, { ...body, paragraphIndex: 1 });
  await open(page);
  assert.equal(await page.$eval(newMemo, option => option.disabled), true);
  assert.equal(await page.$eval('#memo-text', el => el.value), bodyMemo.text);
  await fill(page, '여러 문단 선택 중 수정한 메모');
  await page.click('[data-memo-dialog] .dialog-btn-primary');
  assert.deepEqual(await state(page), [{ ...bodyMemo, text: '여러 문단 선택 중 수정한 메모' }]);
  await page.focus('#memo-text');
  await page.keyboard.press('Tab');
  assert.equal(await focused(), '선택한 메모 삭제');
  await page.keyboard.press('Enter');
  assert.deepEqual(await state(page), []);
  await close(page);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.deepEqual(await state(page), [{ ...bodyMemo, text: '여러 문단 선택 중 수정한 메모' }]);
  await position(cell(1, 1), cell(1, 4));
  await add('둘째 셀 선택 메모');
  await position(cell(0, 0));
  await add('빈 셀 커서 메모');

  const expected = [
    { text: '여러 문단 선택 중 수정한 메모', author: '', sectionIndex: 0, paraIndex: 0,
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
      assert.match(await page.$eval(activeMemo + ' .memo-list-meta', el => el.textContent), /표 셀$/);
      assert.doesNotMatch(await page.$eval(activeMemo, el => el.textContent), /문단|개체/);
      await page.click('.memo-dialog-location summary');
      assert.equal(await page.$eval('.memo-dialog-location', el => el.open), true);
      assert.match(await page.$eval('.memo-dialog-location span', el => el.textContent), /표 셀 2.*글자 2–4/);
      await page.click('.memo-dialog-location summary');
      const expectedOffset = memo.endCharIdx;
      await page.focus('.memo-go-to');
      await page.keyboard.press('Enter');
      await page.waitForSelector('#memo-text', { hidden: true });
      const focus = await page.evaluate(() => window.__inputHandler.getCursorPosition());
      assert.equal(focus.charOffset, expectedOffset);
      assert.equal(focus.cellPath.at(-1).cellIndex, 1);
      assert.equal(await dirty(page), false, '메모 이동은 저장 상태를 바꾸지 않는다');
      const card = `.memo-card[data-memo-id="${memo.id}"]`;
      await page.waitForSelector(card, { visible: true });
      await page.click(card + ' .memo-card-open');
      assert.equal(await page.$('.memo-marker-layer'), null, '쪽 안쪽에 중복 메모 버튼을 만들지 않는다');
      assert.equal(await page.$eval('#memo-text', el => el.value), memo.text);
      assert.equal(await dirty(page), false, '표시에서 메모를 열어도 문서는 바뀌지 않는다');
      await close(page);
      console.log(`  ${format.toUpperCase()} ${pass}회차: 본문과 두 셀의 메모 및 대상 위치 보존`);
    }
  }

  const beforeMargin = await state(page);
  const saveBytes = () => page.evaluate(() => [...window.__wasm.exportHwpx()]);
  const beforePreview = await saveBytes();
  await page.waitForSelector('.memo-margin', { visible: true });
  await page.waitForFunction(() => document.querySelectorAll('.memo-card').length === 3);
  assert.equal(await page.$eval('[data-cmd="view:memo-guides"]', item => item.getAttribute('aria-pressed')), 'true');
  await page.waitForFunction(() => document.querySelectorAll('.memo-margin-guides polyline').length === 3);
  const geometry = () => page.evaluate(() => {
    const viewport = document.querySelector('#scroll-container').getBoundingClientRect();
    const cards = [...document.querySelectorAll('.memo-card')]
      .map(button => button.getBoundingClientRect()).sort((a, b) => a.top - b.top);
    return cards.every((card, index) => card.left >= viewport.right &&
      (index === 0 || card.top >= cards[index - 1].bottom));
  });
  assert.equal(await geometry(), true, '여백 상자는 본문과 다른 메모를 덮지 않는다');
  await page.evaluate(() => window.__canvasView.getViewportManager().setZoom(1.2));
  await page.waitForFunction(() => window.__canvasView.getViewportManager().getZoom() === 1.2
    && document.querySelector('.memo-margin-guides polyline'));
  assert.equal(await geometry(), true, '확대 후에도 여백 상자는 분리된다');
  await page.click('.memo-card-open');
  const awaitMemoText = await page.$eval('#memo-text', el => el.value);
  assert.ok(beforeMargin.some(memo => memo.text === awaitMemoText), '카드에서 해당 메모를 연다');
  await close(page);
  await toggleMemoView('view:memo-guides');
  await page.waitForFunction(() => !document.querySelector('.memo-margin-guides polyline'));
  await toggleMemoView('view:memo-margin');
  await page.waitForSelector('.memo-margin', { hidden: true });
  assert.equal(await page.$('.memo-marker-layer'), null);
  await toggleMemoView('view:memo-list');
  await page.waitForSelector('#memo-text');
  assert.equal(await page.$$('[data-memo-choice]').then(choices => choices.length), beforeMargin.length + 1);
  await close(page);
  assert.deepEqual(await state(page), beforeMargin);
  assert.deepEqual(await saveBytes(), beforePreview, '보기 상태는 저장 내용에 기록되지 않는다');
  assert.equal(await dirty(page), false);

  const family = () => page.evaluate(() => window.__wasm.getMemos().map(memo => ({
    id: memo.id, parentId: memo.parentId, text: memo.text, author: memo.author,
  })));
  const parentText = '둘째 셀 선택 메모';
  const parent = (await family()).find(memo => memo.text === parentText);
  await open(page);
  await selectMemo(page, parent.id);
  await page.click('.memo-add-reply');
  await fill(page, '작성 중인 댓글');
  await page.click('.memo-cancel-reply');
  assert.equal((await family()).length, 3);
  assert.equal(await dirty(page), false);
  assert.equal(await page.$eval('#memo-text', el => el.value), parentText);
  await page.click('.memo-add-reply');
  await page.type('#memo-author', '검토자');
  await fill(page, '첫 댓글');
  await page.click('[data-memo-dialog] .dialog-btn-primary');
  const reply = (await family()).find(memo => memo.parentId === parent.id);
  assert.ok(reply);
  assert.equal(reply.author, '검토자');
  assert.equal(await page.$eval(activeMemo + ' .memo-list-preview', el => el.textContent), '첫 댓글');
  assert.match(await page.$eval(activeMemo + ' .memo-list-meta', el => el.textContent), /댓글 · 검토자/);
  assert.match(await page.$eval('[data-memo-dialog] .memo-dialog-body p:not([hidden])', el => el.textContent),
    new RegExp(parentText));
  assert.equal(await page.$eval('.memo-add-reply', el => el.disabled), true);
  await fill(page, '수정한 댓글');
  await page.click('[data-memo-dialog] .dialog-btn-primary');
  assert.equal((await family()).find(memo => memo.id === reply.id).text, '수정한 댓글');
  await deleteMemo(page);
  assert.equal((await family()).find(memo => memo.id === reply.id), undefined);
  await close(page);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.equal((await family()).find(memo => memo.id === reply.id).text, '수정한 댓글');
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.equal((await family()).length, 3);
  await page.evaluate(() => window.__inputHandler.performUndo());
  await open(page);
  await selectMemo(page, parent.id);
  await page.click('.memo-add-reply');
  await fill(page, '둘째 댓글');
  await page.click('[data-memo-dialog] .dialog-btn-primary');
  await close(page);
  const relationships = async () => {
    const memos = await family();
    return memos.filter(memo => memo.parentId !== null)
      .map(memo => [memo.text, memos.find(parent => parent.id === memo.parentId)?.text])
      .sort();
  };
  const expectedFamily = [['둘째 댓글', parentText], ['수정한 댓글', parentText]];
  for (const format of ['hwp', 'hwpx']) {
    await saveAndReopen(page, format, 'reply');
    assert.deepEqual(await relationships(), expectedFamily, format + ' 부모 관계와 댓글 내용');
  }

  const beforeDeleting = await family();
  const reopenedParent = beforeDeleting.find(memo => memo.text === parentText);
  await open(page);
  await selectMemo(page, reopenedParent.id);
  await deleteMemo(page);
  const confirmation = '.modal-overlay:last-child .dialog-wrap';
  await page.waitForFunction(selector => document.querySelector(selector)?.textContent.includes('댓글 2개'), {}, confirmation);
  await page.keyboard.press('Escape');
  await page.waitForFunction(() => document.querySelectorAll('.modal-overlay').length === 1);
  assert.deepEqual(await family(), beforeDeleting);
  assert.equal(await dirty(page), false);
  await deleteMemo(page);
  await page.click(confirmation + ' .dialog-btn-primary');
  await page.waitForFunction(() => window.__wasm.getMemos().length === 2);
  await close(page);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.deepEqual(await family(), beforeDeleting);
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.deepEqual(await relationships(), []);
  console.log('  댓글 추가·수정·삭제, 부모 확인·취소, undo/redo와 HWP/HWPX 부모 관계 보존');

  await createNewDocument(page);
  const pageMemoOffset = await page.evaluate(async () => {
    const text = '긴 문단의 메모 대상 문자열입니다. '.repeat(160);
    window.__wasm.doc.insertText(0, 0, 0, text);
    await window.__canvasView.loadDocument();
    return text.length - 12;
  });
  const later = { ...body, charOffset: pageMemoOffset };
  const actualPage = await page.evaluate(offset => window.__wasm.getCursorRect(0, 0, offset).pageIndex + 1, pageMemoOffset);
  assert.ok(actualPage > 1, '기준 글자가 문단의 시작 쪽에 있지 않다');
  await position(later, { ...later, charOffset: pageMemoOffset + 4 });
  await add('뒤쪽 기준 글자의 메모');
  await open(page);
  const [laterMemo] = await state(page);
  await selectMemo(page, laterMemo.id);
  const option = await page.$eval(activeMemo + ' .memo-list-meta', el => el.textContent);
  assert.ok(option.endsWith(`${actualPage}쪽 · 본문`), option);
  await close(page);

  await createNewDocument(page);
  // 저장된 한컴 웹한글 V3 InsertFieldMemo에서 대조한 Unicode scalar 범위.
  const phrase = '한글ABC 테스트, 다음';
  const unicode = 'e\u0301lan 👩‍💻한글';
  const wordCases = [
    ['혼용 단어 중간', phrase, 1, 0, 5],
    ['한영 경계', phrase, 2, 0, 5],
    ['단어 끝 공백', phrase, 5, 0, 5],
    ['다음 단어 시작', phrase, 6, 6, 10],
    ['문장부호', phrase, 9, 6, 10],
    ['문단 끝', phrase, 13, 11, 13],
    ['빈 문단', '', 0, 0, 0],
    ['결합 문자', unicode, 2, 0, 5],
    ['이모지와 한글', unicode, 9, 6, 11],
    ['선행 공백', '  하나 둘', 1, 2, 4],
    ['연속 공백 안', '하나   둘', 3, 5, 6],
    ['후행 공백', '하나  ', 4, 0, 2],
    ['공백뿐인 문단', '   ', 2, 2, 2],
    ['줄바꿈 없는 공백', '가　나 다', 3, 2, 5],
  ];
  const wordTable = await page.evaluate(async cases => {
    const wasm = window.__wasm;
    for (const [index, [, text]] of cases.entries()) {
      if (text) wasm.doc.insertText(0, index, 0, text);
      wasm.splitParagraph(0, index, Array.from(text).length);
    }
    const result = wasm.createTable(0, cases.length, 0, cases.length, 1);
    if (!result.ok) throw new Error('createTable failed');
    for (const [index, [, text]] of cases.entries()) {
      if (text) wasm.insertTextInCell(0, result.paraIdx, result.controlIdx, index, 0, 0, text);
    }
    await window.__canvasView.loadDocument();
    return result;
  }, wordCases);
  const targets = [];
  for (const isCell of [false, true]) {
    for (const [index, [label, , caret, start, end]] of wordCases.entries()) {
      const pos = isCell ? {
        ...body, paragraphIndex: 0, charOffset: caret, parentParaIndex: wordTable.paraIdx,
        controlIndex: wordTable.controlIdx, cellIndex: index, cellParaIndex: 0,
        cellPath: [{ controlIndex: wordTable.controlIdx, cellIndex: index, cellParaIndex: 0 }],
      } : { ...body, paragraphIndex: index, charOffset: caret };
      const name = (isCell ? '셀 ' : '본문 ') + label;
      await position(pos);
      await add(name);
      const actual = (await state(page)).find(memo => memo.text === name);
      assert.deepEqual([actual.startCharIdx, actual.endCharIdx], [start, end], name);
      assert.equal(actual.path?.at(-1).cellIndex ?? null, isCell ? index : null, name);
      targets.push(actual);
    }
  }
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.equal((await state(page)).length, targets.length - 1);
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.deepEqual((await state(page)).sort((a, b) => a.id - b.id), targets);
  for (const format of ['hwp', 'hwpx']) {
    await saveAndReopen(page, format, 'words');
    assert.deepEqual(contentAndAnchor(await state(page)), contentAndAnchor(targets));
  }
  console.log('  한컴 웹한글을 대조한 단어 범위 — 본문·표 셀, Unicode, HWP/HWPX 저장·재열기');

  await createNewDocument(page);
  const grouped = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const boxes = [];
    for (const [i, text] of ['첫째 글상자', '둘째 글상자'].entries()) {
      const box = wasm.createShapeControl({
        sectionIdx: 0, paraIdx: 0, charOffset: i, shapeType: 'textbox',
        width: 10000, height: 5000, horzOffset: i * 12000, vertOffset: 0,
      });
      if (!box.ok) throw new Error('글상자 생성 실패');
      wasm.insertTextInCell(0, box.paraIdx, box.controlIdx, 0, 0, 0, text);
      boxes.push({ paraIdx: box.paraIdx, controlIdx: box.controlIdx });
    }
    const group = wasm.groupShapes(0, boxes);
    if (!group.ok) throw new Error('글상자 묶기 실패');
    const target = { section: 0, para: group.paraIdx, cellPath: [[group.controlIdx, 1, 0]] };
    if (wasm.getHyperlinkContext(target).text !== '둘째 글상자') throw new Error('묶음 내부 주소가 잘못되었습니다.');
    await window.__canvasView.loadDocument();
    return group;
  });
  const groupedBox = {
    sectionIndex: 0, paragraphIndex: 0, charOffset: 0, isTextBox: true,
    parentParaIndex: grouped.paraIdx, controlIndex: grouped.controlIdx, cellIndex: 1, cellParaIndex: 0,
    cellPath: [{ controlIndex: grouped.controlIdx, cellIndex: 1, cellParaIndex: 0 }],
  };
  await position(groupedBox);
  const hwpBytes = () => page.evaluate(() => Array.from(window.__wasm.exportHwp()));
  const before = await hwpBytes();
  await open(page);
  await fill(page, '묶음 안에 남아서는 안 되는 메모');
  await page.click('.dialog-btn-primary');
  assert.match(await page.$eval('[data-memo-dialog] [role="alert"]', el => el.textContent), /이 위치에는/);
  assert.deepEqual(await hwpBytes(), before, '거부된 메모는 저장 바이트를 바꾸지 않는다');
  assert.deepEqual(await state(page), []);
  assert.equal(await dirty(page), false);
  await close(page);
  console.log('  묶음 안 글상자: 오류 표시, 저장 바이트·dirty 유지');

  await createNewDocument(page);
  const quick = await page.evaluate(async () => {
    const w = window.__wasm, cv = window.__canvasView;
    w.doc.insertText(0, 0, 0, '가나다라 별도 단어');
    w.splitParagraph(0, 0, 10);
    const table = w.createTable(0, 1, 0, 1, 1);
    w.insertTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, '가운데 글자');
    await cv.loadDocument();
    return table;
  });
  const clickAt = async rect => {
    const point = await page.evaluate(rect => {
      const view = window.__canvasView;
      const v = view.getVirtualScroll(), zoom = view.getViewportManager().getZoom();
      const c = document.querySelector('#scroll-content'), b = c.getBoundingClientRect();
      return { x: b.left + v.getPageLeftResolved(rect.pageIndex, c.clientWidth) + rect.x * zoom + 1,
        y: b.top + v.getPageOffset(rect.pageIndex) + (rect.y + rect.height / 2) * zoom };
    }, rect);
    await page.mouse.click(point.x, point.y, { button: 'right' });
    await page.waitForSelector('.context-menu [data-cmd="insert:comment"]', { visible: true });
  };
  const compose = async text => {
    await page.waitForSelector('.memo-inline-editor textarea:focus');
    await page.type('.memo-inline-editor textarea', text);
    await page.click('.memo-inline-editor button[type="submit"]');
    await page.waitForSelector('.memo-inline-editor', { hidden: true });
  };
  await position({ ...body, charOffset: 1 }, { ...body, charOffset: 3 });
  await toggleMemoView('view:memo-guides');
  await clickAt(await page.evaluate(() => window.__wasm.getCursorRect(0, 0, 2)));
  await page.click('.context-menu [data-cmd="insert:comment"]');
  await page.waitForSelector('.memo-margin-guides .is-draft');
  await page.waitForSelector('.memo-highlight-layer .is-draft');
  assert.deepEqual(await state(page), [], '안내선과 범위 표시만으로 메모를 만들지 않는다');
  assert.equal(await dirty(page), false);
  await compose('빠른 메뉴 선택 메모');
  await page.waitForSelector('.memo-margin-guides .is-draft', { hidden: true });
  assert.deepEqual((await state(page)).map(m => [m.startCharIdx, m.endCharIdx]), [[1, 3]]);

  // 본문에 캐럿을 둔 채 셀을 우클릭하면 이전 캐럿 대신 클릭한 셀에 붙는다.
  await position({ ...body, charOffset: 0 });
  await clickAt(await page.evaluate(({ paraIdx, controlIdx }) =>
    window.__wasm.getCursorRectByPath(0, paraIdx, JSON.stringify([{ controlIndex: controlIdx, cellIndex: 0, cellParaIndex: 0 }]), 1), quick));
  await page.click('.context-menu [data-cmd="insert:comment"]');
  await page.waitForSelector('.memo-margin-guides .is-draft');
  assert.equal(await page.evaluate(() => {
    const ink = document.querySelector('.memo-highlight-layer .is-draft').getBoundingClientRect();
    const overlay = document.querySelector('.memo-margin-guides').getBoundingClientRect();
    const origin = document.querySelector('.memo-margin-guides .is-draft').points.getItem(0);
    return Math.abs(ink.left - overlay.left - origin.x) < 2
      && Math.abs(ink.bottom - overlay.top - origin.y) < 2;
  }), true, '셀 초안의 안내선은 표시된 기준 글자에서 시작한다');
  await page.keyboard.press('Escape');
  await page.waitForSelector('.memo-margin-guides .is-draft', { hidden: true });
  assert.equal((await state(page)).length, 1);
  await clickAt(await page.evaluate(({ paraIdx, controlIdx }) =>
    window.__wasm.getCursorRectByPath(0, paraIdx, JSON.stringify([{ controlIndex: controlIdx, cellIndex: 0, cellParaIndex: 0 }]), 1), quick));
  await page.click('.context-menu [data-cmd="insert:comment"]');
  await compose('빠른 메뉴 셀 메모');
  const cellMemo = (await state(page)).find(m => m.text === '빠른 메뉴 셀 메모');
  assert.deepEqual([cellMemo.startCharIdx, cellMemo.endCharIdx, cellMemo.path.at(-1).cellIndex], [0, 3, 0]);
  const card = '.memo-card[data-memo-id="' + cellMemo.id + '"]';
  await page.waitForSelector(card);
  assert.equal(await page.$('.memo-marker-layer'), null);
  await page.hover(card);
  await page.waitForSelector(card + '.is-active');
  assert.ok(await page.$('.memo-highlight-layer .is-active'));
  await page.click(card + ' .memo-card-actions button:last-child');
  await compose('부모 카드 안의 회신');
  await page.waitForSelector(card + ' details summary');
  assert.equal(await page.$eval(card + ' details summary', e => e.textContent), '댓글 1개');
  await page.click(card + ' details summary');
  assert.equal(await page.$eval(card + ' .memo-reply-open', e => e.textContent), '부모 카드 안의 회신');
  await page.click(card + ' header', { button: 'right' });
  await page.waitForSelector('.context-menu .md-item', { visible: true });
  await page.click('.context-menu .md-item:last-child');
  await page.waitForFunction(() => [...document.querySelectorAll('.modal-overlay')].some(e => e.textContent.includes('댓글 1개')));
  await page.keyboard.press('Escape');
  assert.equal((await state(page)).length, 3);

  await toggleMemoView('view:memo-visibility');
  await page.waitForSelector('.memo-margin', { hidden: true });
  await page.waitForFunction(() => !document.querySelector('.memo-highlight-layer span'));
  await toggleMemoView('view:memo-visibility');
  await page.waitForSelector(card, { visible: true });
  await position({ ...body, charOffset: 7 });
  await page.click('#menu-bar .menu-item[data-menu="insert"] .menu-title');
  const insertItem = '#menu-bar .md-item[data-cmd="insert:memo-here"]';
  assert.equal(await page.$eval(insertItem, el => el.classList.contains('disabled')), false, '입력 메뉴의 메모 넣기 활성');
  await page.$eval(insertItem, el => el.click());
  await page.waitForSelector('.memo-inline-editor textarea:focus');
  await page.keyboard.press('Escape');
  assert.equal((await state(page)).length, 3);
  console.log('  빠른 메뉴 본문·셀 위치, 인라인 회신 및 강조·숨기기');

  await toggleMemoView('view:memo-guides');
  await createNewDocument(page);
  await loadApp(page, '?lang=en');
  await createNewDocument(page);
  assert.equal(await page.$eval('[data-cmd="view:memo-guides"]', item => item.getAttribute('aria-pressed')), 'false');
  await open(page);
  assert.equal(await page.$eval('[data-cmd="insert:comment"] .md-label', el => el.textContent), 'Memo list');
  assert.equal(await page.$eval('[data-memo-dialog]', el => el.getAttribute('aria-label')), 'Memo');
  assert.equal(await page.$eval('[data-memo-dialog] [role="listbox"]', el => el.getAttribute('aria-label')), 'Document memos');
  assert.equal(await page.$eval('[data-memo-dialog] .dialog-btn-primary', el => el.textContent), 'Apply');
  await page.click('[data-memo-dialog] .dialog-btn-primary');
  assert.equal(await page.$eval('[data-memo-dialog] [role="alert"]', el => el.textContent), 'Enter memo text.');
  await close(page);
});
