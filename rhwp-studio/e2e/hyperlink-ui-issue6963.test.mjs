/** #6963 한컴 대화상자·기존 링크 확인·우클릭·실제 포인터 열기 및 서식 회귀. */
import { strict as assert } from 'node:assert';
import { mkdirSync } from 'node:fs';
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
  assert.match(await page.$eval('#scroll-container', el => el.title), /https:\/\/example.com/);
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
  await page.$eval('#hyperlink-uri', el => { el.value = 'https://example.org/updated'; el.dispatchEvent(new Event('input', { bubbles: true })); });
  await page.click('.dialog-btn-primary');
  const links = () => page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }).links);
  assert.equal((await links())[0].uri, 'https://example.org/updated');
  await page.mouse.click(point.x, point.y, { button: 'right' });
  await page.click('.context-menu [data-cmd="hyperlink:remove"]');
  assert.equal((await links()).length, 0);
  assert.equal((await props()).underline, false);
  await page.evaluate(() => window.__inputHandler.performUndo());
  assert.equal((await links()).length, 1);
  await page.evaluate(() => window.__inputHandler.performRedo());
  assert.equal((await links()).length, 0);
  assert.equal(await page.evaluate(() => window.__wasm.getHyperlinkContext({ section: 0, para: 0, cellPath: [] }).text), '한컴 링크 테스트');
  // 삭제한 글자 또는 빈 페이지 우클릭에는 링크 명령이 없다.
  await page.mouse.click(point.x + 300, point.y + 100, { button: 'right' });
  assert.equal(await page.$('.context-menu [data-cmd="hyperlink:remove"]'), null);
  console.log('PASS: insert style, click/visited, existing confirmation/cancel, context edit/remove, undo/redo, non-link context');
});
