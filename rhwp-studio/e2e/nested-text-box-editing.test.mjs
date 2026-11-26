// 관계형 검사 — #9398·#9399 포인터가 고른 글상자와 편집·실행 취소가 가리키는 문단을 비교한다.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { runTest, waitForCanvas } from './helpers.mjs';

const entry = (controlIndex, cellIndex, cellParaIndex) => ({ controlIndex, cellIndex, cellParaIndex });
const settle = page => page.evaluate(() => new Promise(done => requestAnimationFrame(() => requestAnimationFrame(done))));

async function openFixture(page, name) {
  const bytes = await readFile(new URL('../../tests/fixtures/group_textbox_cell/' + name, import.meta.url));
  const waitForOpen = page.evaluate(({ name, bytes }) => new Promise((resolve, reject) => {
    const bus = window.__eventBus;
    const timeout = setTimeout(() => { off(); reject(new Error('문서 열기 시간 초과: ' + name)); }, 30000);
    const off = bus.on('open-document-bytes:done', result => {
      if (result.requestId !== name) return;
      off();
      clearTimeout(timeout);
      result.ok ? resolve() : reject(new Error(result.error));
    });
    bus.emit('open-document-bytes', {
      fileName: name, bytes: Uint8Array.from(bytes), requestId: name, skipUnsavedGuard: true,
    });
  }), { name, bytes: [...bytes] });
  const dialogWait = new AbortController();
  const fontPrompt = page.waitForFunction(() => [...document.querySelectorAll('button')].find(
    button => button.offsetWidth && ['대체 글꼴로 보기', 'View with substitute fonts'].includes(button.textContent.trim()),
  ), { timeout: 12000, signal: dialogWait.signal }).then(button => button.asElement()?.click()).catch(() => {});
  try {
    await Promise.race([waitForOpen, fontPrompt]);
    await waitForOpen;
  } finally {
    dialogWait.abort();
  }
  await waitForCanvas(page);
  await settle(page);
}

async function clickPage(page, coordinates) {
  const point = await page.evaluate(({ x, y, pageIndex }) => {
    const input = window.__inputHandler;
    const sc = input.container.querySelector('#scroll-content');
    const frame = sc.getBoundingClientRect();
    const zoom = input.viewportManager.getZoom();
    return {
      x: frame.left + input.virtualScroll.getPageLeftResolved(pageIndex, sc.clientWidth) + x * zoom,
      y: frame.top + input.virtualScroll.getPageOffset(pageIndex) + y * zoom,
    };
  }, coordinates);
  await page.mouse.click(point.x, point.y);
  await settle(page);
}

async function clickCaret(page, parent, path, offset, dx) {
  const rect = await page.evaluate(({ parent, path, offset }) =>
    window.__wasm.getCursorRectByPath(0, parent, JSON.stringify(path), offset), { parent, path, offset });
  await clickPage(page, { pageIndex: rect.pageIndex, x: rect.x + dx, y: rect.y + rect.height / 2 });
}

async function paragraphs(page, parent, path) {
  return page.evaluate(({ parent, path }) => {
    const wasm = window.__wasm;
    const count = wasm.getCellParagraphCountByPath(0, parent, JSON.stringify(path));
    return Array.from({ length: count }, (_, index) => {
      const paragraphPath = [...path.slice(0, -1), { ...path.at(-1), cellParaIndex: index }];
      return wasm.getTextInCellByPath(0, parent, JSON.stringify(paragraphPath), 0, 100);
    });
  }, { parent, path });
}

async function controlKey(page, key) {
  await page.keyboard.down('Control');
  try { await page.keyboard.press(key); } finally { await page.keyboard.up('Control'); }
  await settle(page);
}

runTest('#9398·#9399 중첩 글상자 실제 클릭과 편집', async ({ page }) => {
  await openFixture(page, 'cell-group-textbox.hwp');
  const left = [entry(0, 0, 1), entry(0, 0, 0)];
  const right = [entry(0, 0, 1), entry(0, 1, 0)];
  await clickCaret(page, 1, right, 5, 5);
  const cursor = () => page.evaluate(() => window.__inputHandler.cursor.getPosition());
  const selected = () => page.evaluate(() => window.__inputHandler.cursor.getSelectedPictureRef() ?? null);
  assert.deepEqual((await cursor()).cellPath, right);
  assert.equal((await cursor()).charOffset, 5);
  assert.equal(await selected(), null);
  await page.keyboard.type('K');
  assert.deepEqual(await paragraphs(page, 1, right), ['둘째 상자K']);
  assert.deepEqual(await paragraphs(page, 1, left), ['첫 상자']);
  await controlKey(page, 'z');
  assert.deepEqual(await paragraphs(page, 1, right), ['둘째 상자']);

  const group = await page.evaluate(() => window.__wasm.getPageControlLayout(0).controls.find(c => c.type === 'group'));
  await clickPage(page, { pageIndex: 0, x: group.x + group.w / 2, y: group.y + group.h / 2 });
  assert.equal((await selected())?.type, 'group');
  await page.keyboard.press('Escape');
  await clickPage(page, { pageIndex: 0, x: group.x + 1, y: group.y + group.h / 3 });
  assert.equal((await selected())?.type, 'group');
  assert.deepEqual((await selected()).cellPath, left.slice(0, -1));
  await clickCaret(page, 1, left, 4, 5);
  assert.equal(await selected(), null);
  assert.deepEqual((await cursor()).cellPath, left);
  await page.keyboard.type('J');
  assert.deepEqual(await paragraphs(page, 1, left), ['첫 상자J']);

  await openFixture(page, 'table-inline-equation.hwpx');
  const cell = [entry(2, 0, 0), entry(0, 0, 0)];
  await clickCaret(page, 0, cell, 2, 0.25);
  await page.keyboard.press('Enter');
  assert.deepEqual(await paragraphs(page, 0, cell), ['B', 'OX']);
  await page.keyboard.press('Backspace');
  assert.equal((await cursor()).charOffset, 2);
  assert.deepEqual(await paragraphs(page, 0, cell), ['BOX']);
  await controlKey(page, 'z');
  assert.deepEqual(await paragraphs(page, 0, cell), ['B', 'OX']);
  await controlKey(page, 'y');
  assert.equal((await cursor()).charOffset, 2);
  await page.keyboard.type('K');
  assert.deepEqual(await paragraphs(page, 0, cell), ['BKOX']);
  await controlKey(page, 'z');
  assert.deepEqual(await paragraphs(page, 0, cell), ['BOX']);
});
