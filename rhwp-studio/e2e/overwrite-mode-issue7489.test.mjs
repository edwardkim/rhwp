/**
 * #7489 수정 모드 — 실제 WASM 과 Studio 입력 경로로 scalar 오프셋과 IME 조각 수명을 확인한다.
 *
 * tests/support/overwrite-mode.runner.mjs 의 반례를 실제 코어에서 다시 본다.
 *  1. abcd, Home, Insert, 😀·X → 😀Xcd(캐럿 2). Ctrl+Z 한 번 → abcd, Ctrl+Y → 😀Xcd.
 *     😀 뒤 캐럿을 scalar 1 에 두고 친 X 도 한 묶음으로 되돌린다
 *  2. 삽입 모드 😀·X → 😀Xabcd, 한 번에 되돌리고 다시 실행한다
 *  3. IME 가 😀 를 확정한 뒤 하 → 😀하cd
 *  4. 덮은 IME 조합 중 새 문서·문서 열기 → 잡아 둔 문단 조각이 코어에 남지 않고 새 문서는 그대로다
 *
 * 실행: node e2e/run-with-vite.mjs -- node e2e/overwrite-mode-issue7489.test.mjs --mode=headless
 */
import { runTest, createNewDocument, assert } from './helpers.mjs';

const sleep = (page, ms) => page.evaluate((t) => new Promise((r) => setTimeout(r, t)), ms);

await runTest('#7489 수정 모드 scalar 오프셋과 IME 조각 수명', async ({ page }) => {
  const cdp = await page.createCDPSession();
  const state = () => page.evaluate(() => ({
    text: window.__wasm.getTextRange(0, 0, 0, 100),
    caret: window.__inputHandler.cursor.getPosition().charOffset,
    depth: window.__inputHandler.history.undoStack.length,
  }));
  const ctrl = async (key) => {
    await page.keyboard.down('Control');
    await page.keyboard.press(key);
    await page.keyboard.up('Control');
    await sleep(page, 300);
  };
  const compose = async (...updates) => {
    for (const text of updates) {
      await cdp.send('Input.imeSetComposition', { text, selectionStart: text.length, selectionEnd: text.length });
    }
  };
  /** 새 문서에 abcd 를 치고 캐럿을 문단 처음에 둔다. 수정 모드면 Insert 로 켠다. */
  const setup = async (overwrite) => {
    await createNewDocument(page);
    await page.evaluate(() => {
      const h = window.__inputHandler;
      h.moveCursorTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
      h.textarea.focus();
    });
    await page.keyboard.type('abcd');
    await page.keyboard.press('Home');
    if (await page.evaluate(() => window.__inputHandler.insertMode) === overwrite) await page.keyboard.press('Insert');
    await sleep(page, 400); // abcd 입력과 병합되지 않게 300ms 창을 넘긴다
    return (await state()).depth;
  };
  const show = (s) => JSON.stringify(s);

  await page.evaluate(() => {
    document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    const start = [...document.querySelectorAll('button.dialog-btn-primary')]
      .find((b) => b.offsetParent !== null && b.textContent?.trim() === '시작하기');
    start?.click();
  });

  // 1. 수정 모드 😀·X
  let depth = await setup(true);
  await page.keyboard.type('😀X', { delay: 30 });
  await sleep(page, 300);
  let s = await state();
  assert(s.text === '😀Xcd' && s.caret === 2, `수정 모드 😀·X → 😀Xcd, 캐럿 2 ${show(s)}`);
  assert(s.depth === depth + 1, `이어 친 😀·X 는 한 묶음으로 기록된다 ${show(s)}`);
  await ctrl('KeyZ');
  s = await state();
  assert(s.text === 'abcd' && s.caret === 0 && s.depth === depth, `Ctrl+Z 한 번 → abcd ${show(s)}`);
  await ctrl('KeyY');
  s = await state();
  assert(s.text === '😀Xcd' && s.caret === 2, `Ctrl+Y → 😀Xcd ${show(s)}`);

  // 1-2. 😀 뒤 캐럿을 scalar 1 에 두고 X. 300ms 안이면 앞 입력과 한 묶음이어야 한다.
  depth = await setup(true);
  await page.keyboard.type('😀');
  await page.evaluate(() => window.__inputHandler.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 1 }));
  await page.keyboard.type('X');
  await sleep(page, 300);
  s = await state();
  assert(s.text === '😀Xcd' && s.depth === depth + 1, `캐럿 1 에서 친 X 도 한 묶음이다 ${show(s)}`);
  await ctrl('KeyZ');
  s = await state();
  assert(s.text === 'abcd' && s.depth === depth, `Ctrl+Z 한 번 → abcd ${show(s)}`);
  await ctrl('KeyY');
  s = await state();
  assert(s.text === '😀Xcd' && s.caret === 2, `Ctrl+Y → 😀Xcd ${show(s)}`);

  // 2. 삽입 모드 😀·X
  depth = await setup(false);
  await page.keyboard.type('😀X', { delay: 30 });
  await sleep(page, 300);
  s = await state();
  assert(s.text === '😀Xabcd' && s.caret === 2 && s.depth === depth + 1, `삽입 모드 😀·X → 😀Xabcd ${show(s)}`);
  await ctrl('KeyZ');
  s = await state();
  assert(s.text === 'abcd' && s.depth === depth, `삽입 모드 Ctrl+Z 한 번 → abcd ${show(s)}`);
  await ctrl('KeyY');
  assert((await state()).text === '😀Xabcd', '삽입 모드 Ctrl+Y → 😀Xabcd');

  // 3. IME 😀 확정 뒤 하
  depth = await setup(true);
  await compose('😀');
  await cdp.send('Input.insertText', { text: '😀' });
  await compose('ㅎ', '하');
  await cdp.send('Input.insertText', { text: '하' });
  await sleep(page, 300);
  s = await state();
  assert(s.text === '😀하cd' && s.caret === 2, `IME 😀 확정 뒤 하 → 😀하cd ${show(s)}`);
  await ctrl('KeyZ');
  s = await state();
  assert(s.text === 'abcd' && s.depth === depth, `IME 입력도 Ctrl+Z 한 번 → abcd ${show(s)}`);

  // 4. 덮은 IME 조합 중 문서 교체. 조합이 기록되기 전에 deactivate 가 불렸는지도 함께 본다.
  const watchSession = () => page.evaluate(() => {
    const w = window.__wasm;
    const h = window.__inputHandler;
    window.__overwriteProbe = { captured: [], atDeactivate: null };
    const capture = w.captureDeleteRange.bind(w);
    w.captureDeleteRange = (...args) => {
      const id = capture(...args);
      window.__overwriteProbe.captured.push(id);
      return id;
    };
    const deactivate = h.deactivate;
    h.deactivate = function () {
      window.__overwriteProbe.atDeactivate = { composing: this.isComposing, fragment: this._compositionFragment };
      delete w.captureDeleteRange;
      delete h.deactivate;
      return deactivate.call(this);
    };
  });
  /** 조합이 잡아 둔 조각이 아직 코어에 있으면 되살아난다. 없으면 코어가 오류를 던진다. */
  const fragmentAlive = () => page.evaluate(() => {
    try {
      window.__wasm.restoreDeleteFragment(window.__overwriteProbe.captured[0]);
      return true;
    } catch {
      return false;
    }
  });
  const endComposition = async () => {
    await compose('');
    await sleep(page, 300);
  };

  // 4-1. 새 문서는 같은 코어를 다시 쓴다(createBlankDocument).
  await setup(true);
  await watchSession();
  await compose('ㅎ');
  s = await state();
  assert(s.text === 'ㅎbcd', `조합 ㅎ 이 a 를 덮는다 ${show(s)}`);
  await createNewDocument(page);
  let probe = await page.evaluate(() => window.__overwriteProbe);
  assert(probe.captured.length === 1 && probe.atDeactivate?.composing === true
    && probe.atDeactivate.fragment === probe.captured[0], `조합이 끝나기 전에 새 문서로 바뀐다 ${show(probe)}`);
  await endComposition();
  s = await state();
  assert(s.text === '' && s.depth === 0, `새 문서는 비어 있고 기록이 없다 ${show(s)}`);
  assert(await fragmentAlive() === false, '새 문서 뒤 옛 문단 조각이 코어에 남지 않는다');

  // 4-2. 문서 열기는 새 코어로 바꾼다. 옛 조각 해제가 새 문서를 건드리지 않아야 한다.
  await setup(true);
  await watchSession();
  await compose('ㅎ');
  const opened = await page.evaluate(async () => {
    const bytes = new Uint8Array(await (await fetch('/samples/para-001.hwp')).arrayBuffer());
    const bus = window.__eventBus;
    const requestId = 'overwrite-7489';
    const done = new Promise((resolve) => {
      bus.on('open-document-bytes:done', (r) => { if (r.requestId === requestId) resolve(r); });
    });
    bus.emit('open-document-bytes', { bytes, fileName: 'para-001.hwp', fileHandle: null, skipUnsavedGuard: true, requestId });
    return done;
  });
  await sleep(page, 1000);
  probe = await page.evaluate(() => window.__overwriteProbe);
  assert(opened.ok === true, `조합 중 문서 열기가 성공한다 ${show(opened)}`);
  assert(probe.atDeactivate?.composing === true && probe.atDeactivate.fragment === probe.captured[0],
    `조합이 끝나기 전에 연 문서로 바뀐다 ${show(probe)}`);
  const before = await page.evaluate(() => window.__wasm.getTextRange(0, 0, 0, 100));
  await endComposition();
  s = await state();
  assert(s.text === before && s.depth === 0, `연 문서는 그대로이고 기록이 없다 ${show(s)}`);
});
