/** #8103: 복원된 선택을 확정된 쪽 배치에 투영한다. */
import { runTest, createNewDocument, assert, screenshot, setTestCase } from './helpers.mjs';

const frames = (page) => page.evaluate(() => new Promise(resolve =>
  requestAnimationFrame(() => requestAnimationFrame(resolve))));

async function key(page, name, modifier) {
  if (modifier) await page.keyboard.down(modifier);
  await page.keyboard.press(name);
  if (modifier) await page.keyboard.up(modifier);
}

async function layout(page, count) {
  await page.waitForFunction(n => window.__selectionLayoutCount > n, { timeout: 30_000 }, count);
  await frames(page);
}

async function state(page) {
  return page.evaluate(() => {
    const ih = window.__inputHandler;
    const content = document.getElementById('scroll-content');
    const container = document.getElementById('scroll-container');
    const selection = ih.cursor.getSelection();
    const ordered = ih.cursor.getSelectionOrdered();
    const rects = ordered ? window.__wasm.getSelectionRects(
      ordered.start.sectionIndex, ordered.start.paragraphIndex, ordered.start.charOffset,
      ordered.end.paragraphIndex, ordered.end.charOffset,
    ) : [];
    const highlights = [...content.querySelectorAll('.selection-highlight')]
      .filter(el => el.style.display !== 'none')
      .map(el => ['left', 'top', 'width', 'height'].map(k => parseFloat(el.style[k])));
    const zoom = ih.viewportManager.getZoom();
    const projected = rects.map(r => [
      ih.virtualScroll.getPageLeftResolved(r.pageIndex, content.clientWidth) + r.x * zoom,
      ih.virtualScroll.getPageOffset(r.pageIndex) + r.y * zoom,
      r.width * zoom, r.height * zoom,
    ]);
    return {
      selection, highlights, projected, owners: rects.map(r => r.pageIndex),
      pages: window.__wasm.pageCount, virtualPages: ih.virtualScroll.pageCount,
      paragraphs: window.__wasm.getParagraphCount(0),
      texts: Array.from({ length: window.__wasm.getParagraphCount(0) }, (_, p) =>
        window.__wasm.getTextRange(0, p, 0, 100)),
      scroll: [container.scrollLeft, container.scrollTop], zoom,
      count: window.__selectionLayoutCount,
    };
  });
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const near = (a, b) => a.length === b.length && a.every((r, i) =>
  r.every((v, j) => Math.abs(v - b[i][j]) < 0.02));

function checkProjection(s, label) {
  assert(s.pages === s.virtualPages, `${label}: 엔진과 VirtualScroll 쪽 수 일치`);
  assert(near(s.highlights, s.projected), `${label}: 모든 음영이 확정 쪽 원점에 투영`);
  for (const owner of [0, Math.floor(s.pages / 2), s.pages - 1]) {
    const indexes = s.owners.flatMap((p, i) => p === owner ? [i] : []);
    assert(indexes.length > 0 && indexes.every(i => s.highlights[i]
      && near([s.highlights[i]], [s.projected[i]])), `${label}: 쪽 ${owner + 1} 선택 좌표`);
  }
}

await runTest('선택 삭제 Undo의 확정 layout·스크롤·줌·stale fencing', async ({ page }) => {
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.evaluate(() => {
    document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    [...document.querySelectorAll('button.dialog-btn-primary')].find(el => el.offsetParent !== null)?.click();
  });
  const modifier = await page.evaluate(() => /Mac/i.test(navigator.platform) ? 'Meta' : 'Control');
  const text = Array.from({ length: 400 }, (_, i) => `P${String(i).padStart(3, '0')} selection sample`).join('\n');
  await createNewDocument(page);
  await page.evaluate(() => {
    window.__selectionLayoutCount = 0;
    window.__eventBus.on('document-layout-refreshed', () => window.__selectionLayoutCount++);
    window.__eventBus.emit('page-view-settings-changed', {
      arrangement: { kind: 'single' }, pageMovement: { direction: 'vertical', wheelHorizontal: false },
    });
    window.__inputHandler.viewportManager.setZoom(1);
    window.__inputHandler.focus();
  });
  await page.evaluate(value => {
    const clipboardData = new DataTransfer();
    clipboardData.setData('text/plain', value);
    window.__inputHandler.textarea.dispatchEvent(new ClipboardEvent('paste', { clipboardData, bubbles: true, cancelable: true }));
  }, text);
  await layout(page, 0);
  await key(page, 'Home', modifier);
  await key(page, 'a', modifier);
  const original = await state(page);
  assert(original.paragraphs === 400 && original.pages > 2, '400문단 여러 쪽 fixture');
  const expectedTexts = original.texts;

  for (const zoom of [1, 0.66]) {
    setTestCase(`zoom-${zoom}`);
    await page.evaluate(z => window.__inputHandler.viewportManager.setZoom(z), zoom);
    await frames(page);
    const before = await state(page);
    checkProjection(before, `${zoom} zoom`);
    await key(page, 'Backspace');
    await layout(page, before.count);
    const deleted = await state(page);
    assert(deleted.pages === 1 && deleted.paragraphs === 1 && deleted.texts[0] === '', '전체 삭제: 빈 한 쪽');
    await key(page, 'z', modifier);
    await layout(page, deleted.count);
    const restored = await state(page);
    console.log(JSON.stringify({ zoom, restoredPages: restored.pages,
      mismatchedHighlights: restored.projected.filter((r, i) =>
        !restored.highlights[i] || !near([r], [restored.highlights[i]])).length }));
    checkProjection(restored, `${zoom} Undo`);
    assert(same(restored.texts, expectedTexts), 'Undo: 원문 400문단 순서·내용 보존');
    assert(same(restored.selection, before.selection), 'Undo: anchor/focus 보존');

    for (const owner of [0, Math.floor(restored.pages / 2), restored.pages - 1]) {
      await page.evaluate(p => {
        const ih = window.__inputHandler;
        ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(p));
      }, owner);
      await frames(page);
      const scrolled = await state(page);
      checkProjection(scrolled, `${zoom} scroll p${owner + 1}`);
      assert(same(scrolled.selection, restored.selection), '스크롤: anchor/focus 유지');
      await screenshot(page, `issue8103-zoom${zoom}-page${owner + 1}`);
    }
    const scrolled = await state(page);
    await key(page, 'a', modifier);
    const fresh = await state(page);
    assert(near(scrolled.highlights, fresh.highlights), '같은 확정 layout의 새 선택 대조군과 전체 좌표 일치');
    assert(same(scrolled.scroll, fresh.scroll), '재투영은 스크롤을 이동하지 않음');

    // Redo는 선택을 해제하고 삭제하며, 다시 Undo하면 원래 선택을 되살린다.
    await page.keyboard.down(modifier);
    await page.keyboard.down('Shift');
    await page.keyboard.press('z');
    await page.keyboard.up('Shift');
    await page.keyboard.up(modifier);
    await layout(page, fresh.count);
    const redone = await state(page);
    assert(redone.pages === 1 && redone.texts[0] === '' && redone.selection === null
      && redone.highlights.length === 0, 'Redo: 실제 전체 삭제 범위·선택 해제 일치');
    await key(page, 'z', modifier);
    await layout(page, redone.count);
    const repeated = await state(page);
    checkProjection(repeated, `${zoom} repeat Undo`);
    // 새 선택 대조군을 만들지 않은 복원 선택에서, 마지막 쪽을 보며 실제로 삭제한다.
    await page.evaluate(() => {
      const ih = window.__inputHandler;
      ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(window.__wasm.pageCount - 1));
    });
    await frames(page);
    await key(page, 'Backspace');
    await layout(page, repeated.count);
    const removed = await state(page);
    assert(removed.pages === 1 && removed.paragraphs === 1 && removed.texts[0] === ''
      && removed.selection === null && removed.highlights.length === 0,
    '재선택 없는 복원 선택: 마지막 쪽에서 Backspace가 전체 음영 범위를 삭제');
    await key(page, 'z', modifier);
    await layout(page, removed.count);
    const recovered = await state(page);
    assert(same(recovered.texts, expectedTexts), '실제 Backspace의 Undo가 400문단 원문을 복원');
    checkProjection(recovered, `${zoom} Backspace Undo`);
  }

  setTestCase('pending-layout-new-selection');
  await page.evaluate(() => {
    const w = window.__wasm;
    const prepare = w.prepareCanvasMetrics.bind(w);
    window.__releaseSelectionLayout = null;
    w.prepareCanvasMetrics = async (...args) => {
      w.prepareCanvasMetrics = prepare;
      await new Promise(resolve => { window.__releaseSelectionLayout = resolve; });
      return prepare(...args);
    };
    window.__eventBus.emit('document-changed');
  });
  const pending = await state(page);
  const newer = await page.evaluate(() => {
    const ih = window.__inputHandler;
    // 방향이 역순인 새 선택을 layout 대기 중에 만든다.
    ih.cursor.selectRange({ sectionIndex: 0, paragraphIndex: 220, charOffset: 7 },
      { sectionIndex: 0, paragraphIndex: 200, charOffset: 2 });
    ih.updateCaret(true);
    ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(3));
    ih.viewportManager.setZoom(0.8);
    return ih.cursor.getSelection();
  });
  await frames(page);
  const waiting = await state(page);
  await page.evaluate(() => window.__releaseSelectionLayout());
  await layout(page, pending.count);
  const settled = await state(page);
  assert(same(settled.selection, newer), '지연 layout은 새 역방향 anchor/focus를 보존');
  assert(same(settled.scroll, waiting.scroll), '지연 layout 완료는 현재 스크롤을 보존');
  assert(near(settled.highlights, settled.projected), '지연 완료는 현재 선택·배율로 재투영');

  setTestCase('pending-layout-cleared-selection');
  await page.evaluate(() => {
    const w = window.__wasm;
    const prepare = w.prepareCanvasMetrics.bind(w);
    w.prepareCanvasMetrics = async (...args) => {
      w.prepareCanvasMetrics = prepare;
      await new Promise(resolve => { window.__releaseSelectionLayout = resolve; });
      return prepare(...args);
    };
    window.__eventBus.emit('document-changed');
  });
  const clearing = await state(page);
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.clearSelection();
    ih.updateCaret(true);
    window.__releaseSelectionLayout();
  });
  await layout(page, clearing.count);
  const cleared = await state(page);
  assert(cleared.selection === null && cleared.highlights.length === 0, '지연 완료가 해제된 선택을 되살리지 않음');
  assert(errors.length === 0, `pageerror 0건: ${JSON.stringify(errors)}`);
  console.log(JSON.stringify({ pages: original.pages, paragraphs: original.paragraphs,
    highlights: original.highlights.length, staleSelectionPreserved: same(settled.selection, newer) }));
});
