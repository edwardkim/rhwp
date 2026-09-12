/** #6963 한컴 대화상자·기존 링크 확인·우클릭·실제 포인터 열기 및 서식 회귀. */
import { strict as assert } from 'node:assert';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runTest, createNewDocument } from './helpers.mjs';

const output = resolve('../output/issue6963-stage7');
await runTest('#6963 hyperlink editor UI', async ({ page }) => {
  mkdirSync(output, { recursive: true });
  await createNewDocument(page);
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('#hyperlink-text');
  assert.equal(await page.$eval('.dialog-btn-primary', el => el.textContent), '넣기');
  assert.equal(await page.$eval('.dialog-btn-primary', el => el.disabled), true);
  assert.equal(await page.$$eval('[role="tab"]', tabs => tabs.map(t => t.textContent).join(',')), '웹 주소');
  assert.equal(await page.$eval('#hyperlink-preview', el => el.disabled), true);
  await page.type('#hyperlink-text', '한컴 링크 테스트');
  await page.type('#hyperlink-uri', 'https://example.com/한글?q=1#부분');
  const beforePreview = await page.evaluate(() => ({ context: window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }), dirty: window.__documentState.isDirty() }));
  await page.evaluate(() => {
    window.__previewRecord = {};
    window.open = (url, target) => {
      Object.assign(window.__previewRecord, { url, target });
      return { set opener(value) { window.__previewRecord.opener = value; }, location: { replace(value) { window.__previewRecord.destination = value; } } };
    };
  });
  await page.click('#hyperlink-preview');
  assert.equal(await page.evaluate(() => window.__previewRecord.destination), 'https://example.com/%ED%95%9C%EA%B8%80?q=1#%EB%B6%80%EB%B6%84');
  assert.equal(await page.evaluate(() => window.__previewRecord.opener), null);
  assert.deepEqual(await page.evaluate(() => ({ context: window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }), dirty: window.__documentState.isDirty() })), beforePreview);
  assert.ok(await page.$('#hyperlink-text'), 'preview keeps dialog open');
  await page.screenshot({ path: resolve(output, 'insert-dialog.png') });
  await page.evaluate(() => document.documentElement.dataset.themeEffective = 'light');
  await page.screenshot({ path: resolve(output, 'insert-dialog-light.png') });
  await page.evaluate(() => document.documentElement.dataset.themeEffective = 'dark');
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-text', { hidden: true });
  const props = () => page.evaluate(() => window.__wasm.getCharPropertiesAt(0, 0, 0));
  assert.equal((await props()).textColor.toLowerCase(), '#0000ff');
  assert.equal((await props()).underline, true);
  // 등록 직후 끝 캐럿의 실제 키 입력은 링크/색/밑줄을 이어받지 않는다.
  await page.keyboard.type('XYZ');
  const appended = await page.evaluate(() => ({
    context: window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }),
    props: window.__wasm.getCharPropertiesAt(0, 0, 9),
  }));
  assert.equal(appended.context.text, '한컴 링크 테스트XYZ');
  assert.equal(appended.context.links[0].text, '한컴 링크 테스트');
  assert.equal(appended.context.links[0].end, 9);
  assert.equal(appended.props.textColor.toLowerCase(), '#000000');
  assert.equal(appended.props.underline, false);
  await page.screenshot({ path: resolve(output, 'typing-after-link.png') });
  // 이어 쓴 텍스트만 지워 기존 링크 편집·방문·undo 시나리오를 계속한다.
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  // 양방향 삭제 키는 링크 전체 삭제를 확인하고 한 번의 Undo로 주소·서식을 복구한다.
  const deletionContext = () => page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }));
  const beforeDeletion = await deletionContext();
  for (const [navigation, key] of [['End', 'Backspace'], ['Home', 'Delete']]) {
    await page.keyboard.press(navigation);
    await page.keyboard.press(key);
    await page.waitForSelector('[role="alertdialog"][aria-label="지우기"]');
    assert.equal(await page.$eval('.dialog-body', el => el.textContent), '[하이퍼링크]를 지울까요?');
    assert.deepEqual(await deletionContext(), beforeDeletion);
    await page.keyboard.press('Escape');
    assert.deepEqual(await deletionContext(), beforeDeletion);
    await page.keyboard.press(key);
    await page.click('.dialog-btn-primary');
    assert.equal((await deletionContext()).text, '');
    assert.equal((await deletionContext()).links.length, 0);
    await page.evaluate(() => window.__inputHandler.performUndo());
    assert.deepEqual(await deletionContext(), beforeDeletion);
    assert.equal((await props()).textColor.toLowerCase(), '#0000ff');
    assert.equal((await props()).underline, true);
  }
  const point = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const r = window.__wasm.getSelectionRects(0, 0, 0, 0, 2)[0];
    const content = document.querySelector('#scroll-content');
    const cr = content.getBoundingClientRect();
    const vs = ih.virtualScroll, zoom = ih.viewportManager.getZoom();
    return { x: cr.left + vs.getPageLeftResolved(r.pageIndex, content.clientWidth) + (r.x + r.width / 2) * zoom,
      y: cr.top + vs.getPageOffset(r.pageIndex) + (r.y + r.height / 2) * zoom };
  });
  // 별도 탭의 opener 단절 및 목적지 지정은 가짜 탭 핸들에 기록해 외부 통신 없이 확인한다.
  await page.evaluate(() => {
    window.__openedLinks = [];
    window.open = (url, target) => {
      const record = { url, target, opener: 'original', destination: null };
      window.__openedLinks.push(record);
      return { set opener(value) { record.opener = value; }, location: { replace(value) { record.destination = value; } } };
    };
  });
  await page.mouse.move(point.x, point.y);
  assert.equal(await page.$eval('#scroll-container', el => el.title), 'https://example.com/한글?q=1#부분');
  await page.mouse.click(point.x, point.y);
  await page.waitForFunction(() => window.__openedLinks.length === 1);
  assert.equal((await props()).textColor.toLowerCase(), '#800080');
  const opened = await page.evaluate(() => window.__openedLinks[0]);
  assert.equal(opened.opener, null);
  assert.equal(opened.destination, 'https://example.com/%ED%95%9C%EA%B8%80?q=1#%EB%B6%80%EB%B6%84');
  await page.screenshot({ path: resolve(output, 'visited-link.png') });
  await page.mouse.move(point.x, point.y);
  await page.mouse.down();
  await page.mouse.move(point.x + 70, point.y, { steps: 8 });
  await page.mouse.up();
  assert.equal(await page.evaluate(() => window.__openedLinks.length), 1, 'drag selection must not open URL');
  await page.evaluate(() => { window.__inputHandler.cursor.clearSelection(); window.__inputHandler.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 1 }); });
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('[role="alertdialog"]');
  await page.screenshot({ path: resolve(output, 'existing-link-confirm.png') });
  await page.keyboard.press('Escape');
  assert.equal(await page.$('#hyperlink-text'), null);
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-uri');
  assert.equal(await page.$eval('.dialog-btn-primary', el => el.textContent), '고치기');
  await page.keyboard.press('Escape');
  await page.mouse.click(point.x, point.y, { button: 'right' });
  await page.waitForSelector('.context-menu [data-cmd="hyperlink:edit"]');
  await page.screenshot({ path: resolve(output, 'context-menu.png') });
  await page.click('.context-menu [data-cmd="hyperlink:edit"]');
  await page.waitForSelector('#hyperlink-uri');
  assert.equal(await page.$('[role="alertdialog"]'), null);
  assert.equal(await page.$eval('#hyperlink-text', el => el.readOnly), false);
  await page.$eval('#hyperlink-text', el => { el.value = '바뀐 표시 문자열 😀'; el.dispatchEvent(new Event('input', { bubbles: true })); });
  await page.$eval('#hyperlink-uri', el => { el.value = 'https://example.org/updated'; el.dispatchEvent(new Event('input', { bubbles: true })); });
  await page.click('.dialog-btn-primary');
  const links = () => page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }).links);
  assert.equal((await links())[0].uri, 'https://example.org/updated');
  assert.equal((await links())[0].text, '바뀐 표시 문자열 😀');
  assert.equal((await props()).textColor.toLowerCase(), '#800080');
  await page.mouse.click(point.x, point.y, { button: 'right' });
  await page.click('.context-menu [data-cmd="hyperlink:remove"]');
  assert.equal((await links()).length, 0);
  assert.equal((await props()).underline, false);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.equal((await links()).length, 1);
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.equal((await links()).length, 0);
  assert.equal(await page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }).text), '바뀐 표시 문자열 😀');
  // 삭제한 글자 또는 빈 페이지 우클릭에는 링크 명령이 없다.
  await page.mouse.click(point.x + 300, point.y + 100, { button: 'right' });
  assert.equal(await page.$('.context-menu [data-cmd="hyperlink:remove"]'), null);
  // 색/밑줄 fixture만 엔진에서 준비하고, 링크 삽입/해제는 실제 UI로 실행한다.
  await page.keyboard.press('Escape');
  await createNewDocument(page);
  await page.evaluate(async () => {
    const w = window.__wasm;
    w.insertText(0, 0, 0, '빨간밑줄 초록글자');
    w.applyCharFormat(0, 0, 0, 9, JSON.stringify({ fontSize: 2400, bold: true }));
    w.applyCharFormat(0, 0, 0, 4, JSON.stringify({ textColor: '#ff0000', underlineType: 'Bottom', underlineColor: '#ff0000' }));
    w.applyCharFormat(0, 0, 4, 9, JSON.stringify({ textColor: '#008000', underlineType: 'None', underlineColor: '#008000' }));
    await window.__canvasView.loadDocument();
    const c = window.__inputHandler.cursor;
    c.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 }); c.setAnchor();
    c.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 9 });
  });
  await page.click('button[data-cmd="insert:hyperlink"]');
  await page.waitForSelector('#hyperlink-uri');
  await page.type('#hyperlink-uri', 'https://example.com/original-format');
  await page.click('.dialog-btn-primary');
  await page.waitForSelector('#hyperlink-uri', { hidden: true });
  assert.equal((await props()).textColor.toLowerCase(), '#0000ff');
  await page.screenshot({ path: resolve(output, 'unlink-before.jpg'), type: 'jpeg', quality: 90 });
  for (const format of ['Hwp', 'Hwpx']) {
    const bytes = await page.evaluate(format => Array.from(window.__wasm['export' + format]()), format);
    writeFileSync(resolve(output, 'unlink-before.' + format.toLowerCase()), Buffer.from(bytes));
  }
  const restorePoint = await page.evaluate(() => {
    const ih=window.__inputHandler, vs=ih.virtualScroll;
    const r=window.__wasm.getSelectionRects(0,0,0,0,9)[0];
    const content=document.getElementById('scroll-content'), cr=content.getBoundingClientRect();
    const zoom=ih.viewportManager.getZoom();
    return {x:cr.left+vs.getPageLeftResolved(r.pageIndex,content.clientWidth)+(r.x+5)*zoom,
      y:cr.top+vs.getPageOffset(r.pageIndex)+(r.y+r.height/2)*zoom};
  });
  await page.mouse.click(restorePoint.x, restorePoint.y, { button: 'right' });
  await page.waitForSelector('.context-menu [data-cmd="hyperlink:remove"]');
  await page.click('.context-menu [data-cmd="hyperlink:remove"]');
  const restored = await page.evaluate(() => Array.from({length:9},(_,i)=>window.__wasm.getCharPropertiesAt(0,0,i)));
  for(let i=0;i<9;i++) {
    assert.equal(restored[i].textColor.toLowerCase(),i<4?'#ff0000':'#008000');
    assert.equal(restored[i].underline,i<4);
    assert.equal(restored[i].bold,true);
  }
  assert.equal((await links()).length,0);
  await page.screenshot({ path: resolve(output, 'unlink-restored.jpg'), type: 'jpeg', quality: 90 });
  for (const format of ['Hwp', 'Hwpx']) {
    const bytes = await page.evaluate(format => Array.from(window.__wasm['export' + format]()), format);
    writeFileSync(resolve(output, 'unlink-restored.' + format.toLowerCase()), Buffer.from(bytes));
  }
  console.log('PASS: mixed original color/underline restored through real insert and context-remove UI');
  console.log('PASS: insert style, click/visited, existing confirmation/cancel, context edit/remove, undo/redo, non-link context');
});
