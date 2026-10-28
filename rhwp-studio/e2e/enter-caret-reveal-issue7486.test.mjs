/** #7486: Enter로 새 쪽을 만든 뒤 추가 입력 없이 DOM 캐럿과 viewport를 갱신한다. */
import { runTest, createNewDocument, assert, screenshot } from './helpers.mjs';

const cases = [
  { spacing: 160, enters: 41 },
  { spacing: 200, enters: 33 },
  { spacing: 300, enters: 22 },
];

async function waitForMutationLayout(page, previousCount) {
  await page.waitForFunction((count) => window.__enterCaretLayoutCount > count,
    { timeout: 10_000 }, previousCount);
  // 완료 이벤트 이후의 실제 DOM/scroll 상태를 관찰한다. 입력·배율 변경은 하지 않는다.
  await page.evaluate(() => new Promise((resolve) =>
    requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function readState(page) {
  return page.evaluate(() => {
    const input = window.__inputHandler;
    const rect = input.cursor.getRect();
    const pos = input.cursor.getPosition();
    const zoom = input.viewportManager.getZoom();
    const container = document.getElementById('scroll-container');
    const content = document.getElementById('scroll-content');
    const caret = content.querySelector('.caret');
    const top = Number.parseFloat(caret.style.top);
    const left = Number.parseFloat(caret.style.left);
    const box = caret.getBoundingClientRect();
    const viewport = container.getBoundingClientRect();
    return {
      pos, rect, zoom,
      pages: window.__wasm.pageCount,
      virtualPages: input.virtualScroll.pageCount,
      layoutCount: window.__enterCaretLayoutCount,
      top, left,
      expectedTop: input.virtualScroll.getPageOffset(rect.pageIndex) + rect.y * zoom,
      expectedLeft: input.virtualScroll.getPageLeftResolved(rect.pageIndex, content.clientWidth) + rect.x * zoom,
      scrollTop: container.scrollTop,
      firstPageTop: input.virtualScroll.getPageOffset(0),
      secondPageTop: input.virtualScroll.getPageOffset(1),
      firstPageLeft: input.virtualScroll.getPageLeftResolved(0, content.clientWidth),
      secondPageLeft: input.virtualScroll.getPageLeftResolved(1, content.clientWidth),
      visible: box.top >= viewport.top + 19 && box.bottom <= viewport.bottom - 19
        && box.left >= viewport.left && box.right <= viewport.right,
    };
  });
}

function checkState(state, label, paragraph, owner, pages) {
  console.log(`${label}: ${JSON.stringify(state)}`);
  assert(state.pos.paragraphIndex === paragraph && state.pos.charOffset === 0,
    `${label}: 논리 커서 보존`);
  assert(state.pages === pages && state.virtualPages === pages && state.rect.pageIndex === owner,
    `${label}: 엔진 owner와 실제 쪽 배치 일치`);
  assert(Math.abs(state.top - state.expectedTop) < 1 && Math.abs(state.left - state.expectedLeft) < 1,
    `${label}: 캐럿 DOM이 완료된 쪽 원점에 투영`);
  assert(state.visible, `${label}: 캐럿이 viewport 안에 표시`);
}

runTest('Enter 새 쪽 캐럿·스크롤 및 Undo/Redo', async ({ page }) => {
  // 새 headless 세션의 스킨 안내만 닫는다. 실제 편집 keydown과 별개의 setup이다.
  await page.evaluate(() => {
    const card = document.querySelector('.skin-onboarding-card');
    if (card) card.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    const button = [...document.querySelectorAll('button.dialog-btn-primary')]
      .find((element) => element.offsetParent !== null);
    if (button) button.click();
  });
  const undoKey = await page.evaluate(() => /Mac/i.test(navigator.platform) ? 'Meta' : 'Control');

  for (const zoom of [1, 0.66]) {
    for (const { spacing, enters } of cases) {
      const label = `${spacing}% / 배율 ${Math.round(zoom * 100)}%`;
      await createNewDocument(page);
      const prepared = await page.evaluate(({ spacing, enters }) => {
        const wasm = window.__wasm;
        wasm.applyParaFormat(0, 0, JSON.stringify({ lineSpacing: spacing, lineSpacingType: 'Percent' }));
        // 경계 직전 fixture만 WASM API로 준비한다. 검출 대상인 마지막 Enter는 실키로 실행한다.
        for (let paragraph = 0; paragraph < enters - 1; paragraph++) {
          wasm.splitParagraph(0, paragraph, 0);
        }
        window.__enterCaretLayoutOff?.();
        window.__enterCaretLayoutCount = 0;
        window.__enterCaretLayoutOff = window.__eventBus.on('document-layout-refreshed', ({ source }) => {
          if (source === 'mutation') window.__enterCaretLayoutCount++;
        });
        window.__eventBus.emit('document-changed');
        return wasm.pageCount;
      }, { spacing, enters });
      assert(prepared === 1, `${label}: 경계 직전 한 쪽 fixture`);
      await waitForMutationLayout(page, 0);
      await page.evaluate(({ zoom, paragraph }) => {
        const input = window.__inputHandler;
        window.__eventBus.emit('page-view-settings-changed', {
          arrangement: { kind: zoom === 1 ? 'single' : 'double' },
          pageMovement: { direction: 'vertical', wheelHorizontal: false },
        });
        input.viewportManager.setZoom(zoom);
        input.moveCursorTo({ sectionIndex: 0, paragraphIndex: paragraph, charOffset: 0 });
        document.querySelector('[aria-label="문서 편집 입력"]').focus();
      }, { zoom, paragraph: enters - 1 });
      const before = await readState(page);

      await page.keyboard.press('Enter');
      await waitForMutationLayout(page, before.layoutCount);
      const after = await readState(page);
      checkState(after, `${label} Enter`, enters, 1, 2);
      if (zoom === 1) assert(after.scrollTop > before.scrollTop, `${label}: 새 쪽으로 스크롤 전진`);
      else assert(after.firstPageTop === after.secondPageTop && after.secondPageLeft > after.firstPageLeft,
        `${label}: 두 쪽 나란히 보기의 새 쪽 원점`);
      if (spacing === 200) await screenshot(page, `issue7486-enter-200-zoom${Math.round(zoom * 100)}`);

      await page.keyboard.down(undoKey);
      await page.keyboard.press('z');
      await page.keyboard.up(undoKey);
      await waitForMutationLayout(page, after.layoutCount);
      const undo = await readState(page);
      checkState(undo, `${label} Undo`, enters - 1, 0, 1);

      await page.keyboard.down(undoKey);
      await page.keyboard.down('Shift');
      await page.keyboard.press('z');
      await page.keyboard.up('Shift');
      await page.keyboard.up(undoKey);
      await waitForMutationLayout(page, undo.layoutCount);
      checkState(await readState(page), `${label} Redo`, enters, 1, 2);
    }
  }
  await page.evaluate(() => window.__enterCaretLayoutOff?.());
});
