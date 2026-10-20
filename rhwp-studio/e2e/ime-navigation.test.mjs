/** Replay the native Arch/fcitx Process -> compositionend -> forwarded keydown trace. */
import assert from 'node:assert/strict';
import { runTest, createNewDocument, clickEditArea, typeText, getCursorPosition, getParaText, assert as reportAssert } from './helpers.mjs';

await runTest('IME navigation executes once per physical key', async ({ page }) => {
  async function setup() {
    await createNewDocument(page);
    await clickEditArea(page);
    await typeText(page, 'ABCDEF');
  }

  async function composeAndNavigate({ forwarded = true, releaseFirst = false, shiftKey = false, key = 'ArrowLeft' } = {}) {
    return page.evaluate(({ forwarded, releaseFirst, shiftKey, key }) => {
      const h = window.__inputHandler;
      const input = h.textarea;
      input.focus();
      input.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
      input.value = '한글';
      input.dispatchEvent(new InputEvent('input', {
        bubbles: true, inputType: 'insertCompositionText', data: '한글', isComposing: true,
      }));
      const options = { bubbles: true, cancelable: true, code: key, shiftKey };
      input.dispatchEvent(new KeyboardEvent('keydown', {
        ...options, key: 'Process', keyCode: 229, isComposing: true,
      }));
      input.dispatchEvent(new CompositionEvent('compositionupdate', { bubbles: true, data: '한글' }));
      input.dispatchEvent(new InputEvent('input', {
        bubbles: true, inputType: 'insertCompositionText', data: '한글', isComposing: true,
      }));
      if (releaseFirst) input.dispatchEvent(new KeyboardEvent('keyup', { ...options, key }));
      input.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: '한글' }));
      if (forwarded) input.dispatchEvent(new KeyboardEvent('keydown', { ...options, key }));
      input.dispatchEvent(new KeyboardEvent('keyup', { ...options, key }));
      return { position: h.getCursorPosition(), selection: h.getSelection() };
    }, { forwarded, releaseFirst, shiftKey, key });
  }

  for (const forwarded of [true, false]) {
    await setup();
    const state = await composeAndNavigate({ forwarded });
    assert.equal(state.position.charOffset, 7, 'one physical Left moves 8 -> 7');
    assert.equal(await getParaText(page, 0, 0, 100), 'ABCDEF한글');
    await page.keyboard.press('ArrowLeft');
    assert.equal((await getCursorPosition(page)).charOffset, 6, 'next physical press is independent');
    await page.keyboard.down('Control');
    await page.keyboard.press('KeyZ');
    await page.keyboard.up('Control');
    assert.equal(await getParaText(page, 0, 0, 100), 'ABCDEF', 'composition remains a single undoable edit');
  }

  await setup();
  const selected = await composeAndNavigate({ shiftKey: true });
  assert.equal(selected.position.charOffset, 7);
  assert.equal(selected.selection.start.charOffset, 7);
  assert.equal(selected.selection.end.charOffset, 8, 'Shift+Left selects one character');

  await setup();
  const released = await composeAndNavigate({ releaseFirst: true });
  assert.equal(released.position.charOffset, 6, 'keyup before commit preserves a new press');

  await setup();
  const enter = await composeAndNavigate({ key: 'Enter' });
  assert.equal(enter.position.paragraphIndex, 0, 'IME confirmation must not insert a paragraph');
  assert.equal(await getParaText(page, 0, 0, 100), 'ABCDEF한글');
  reportAssert(true, 'fcitx duplicate, single-dispatch control, subsequent keys, Shift, early keyup, Enter and Undo passed');
});
