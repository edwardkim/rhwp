import {
  runTest, setTestCase, createNewDocument, loadHwpFile, clickEditArea, screenshot, assert,
} from './helpers.mjs';

const pageErrors = [];

const key = (page, opts) => page.evaluate((o) => {
  const t = window.__inputHandler.textarea;
  t.dispatchEvent(new KeyboardEvent('keydown', {
    key: o.key, code: o.code || o.key,
    ctrlKey: !!o.ctrl, metaKey: !!o.meta, altKey: !!o.alt, shiftKey: !!o.shift,
    bubbles: true, cancelable: true,
  }));
}, opts);

const settle = () => new Promise(r => setTimeout(r, 350));

// 셀 선택 모드는 createNewDocument 를 넘어 남는다 — 케이스마다 해제.
const resetModes = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  const cur = ih.cursor;
  cur.clearSelection();
  try { if (cur.isInCellSelectionMode?.()) cur.exitCellSelectionMode(); } catch {}
  try { if (cur.isInTableObjectSelection?.()) cur.exitTableObjectSelection?.(); } catch {}
  try { if (cur.isInPictureObjectSelection?.()) cur.exitPictureObjectSelection?.(); } catch {}
  try { if (cur.isInBlockSelectionMode?.()) cur.exitBlockSelectionMode(); } catch {}
  ih.cellSelectionRenderer?.clear?.();
});

const buildDoc = (page) => createNewDocument(page)
  .then(() => resetModes(page))
  .then(() => page.evaluate(async () => {
    const w = window.__wasm;
    const nf = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    w.doc.insertText(0, 0, 0, 'BEFORE');
    w.doc.splitParagraph(0, 0, 6);
    const created = w.createTable(0, 1, 0, 3, 3);
    if (!created?.ok) throw new Error(`createTable: ${JSON.stringify(created)}`);
    w.doc.insertParagraph(0, created.paraIdx + 1);
    w.doc.insertText(0, created.paraIdx + 1, 0, 'AFTER');
    for (const b of w.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0) || []) {
      w.doc.insertTextInCell(0, created.paraIdx, created.controlIdx, b.cellIdx, 0, 0, `${'ABC'[b.col]}${b.row + 1}`);
    }
    window.__canvasView?.loadDocument?.();
    await nf(); await nf();
    return { paraIdx: created.paraIdx, controlIdx: created.controlIdx };
  }));

const caretInBody = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  ih.cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 2 });
  ih.updateCaret(); ih.focus();
});

const caretInCell = (page, doc, row, col) => page.evaluate(({ paraIdx, controlIdx, row, col }) => {
  const w = window.__wasm;
  const ih = window.__inputHandler;
  const b = (w.getTableCellBboxes(0, paraIdx, controlIdx, 0) || []).find(x => x.row === row && x.col === col);
  if (!b) throw new Error(`cell (${row},${col}) 없음`);
  ih.cursor.moveTo({
    sectionIndex: 0, paragraphIndex: 0, charOffset: 1,
    parentParaIndex: paraIdx, controlIndex: controlIdx,
    cellIndex: b.cellIdx, cellParaIndex: 0,
    cellPath: [{ controlIndex: controlIdx, cellIndex: b.cellIdx, cellParaIndex: 0 }],
  });
  ih.updateCaret(); ih.focus();
}, { ...doc, row, col });

const selState = (page) => page.evaluate(() => {
  const c = window.__inputHandler.cursor;
  const ordered = c.getSelectionOrdered?.();
  return {
    hasSel: c.hasSelection(),
    start: ordered ? { ...ordered.start } : null,
    end: ordered ? { ...ordered.end } : null,
    pos: c.getPosition(),
    cellSel: !!c.isInCellSelectionMode(),
    inCell: c.isInCell(),
  };
});

const cellText = (page, doc, row, col, cellParaIndex = 0) => page.evaluate(({ paraIdx, controlIdx, row, col, cellParaIndex }) => {
  const w = window.__wasm;
  const b = (w.getTableCellBboxes(0, paraIdx, controlIdx, 0) || []).find(x => x.row === row && x.col === col);
  if (!b) return null;
  return w.getTextInCell(0, paraIdx, controlIdx, b.cellIdx, cellParaIndex, 0, 100);
}, { ...doc, row, col, cellParaIndex });

// 마지막 selectionRenderer.render 에 전달된 rect 목록을 캡처하는 spy.
const installRectSpy = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  const r = ih.selectionRenderer;
  if (r && !r.__spyPatched) {
    r.__spyPatched = true;
    const orig = r.render.bind(r);
    r.render = (rects, zoom) => { window.__lastSelRects = rects; return orig(rects, zoom); };
  }
});

await runTest('⌘A 전체 선택 — 셀/글상자 범위 + 표 하이라이트 + 포커스 밖', async ({ page }) => {
  page.on('pageerror', (err) => pageErrors.push(String(err)));
  await clickEditArea(page);
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // ── (a) 본문 캐럿 → 문서 전체 + 표 영역도 하이라이트 ──
  {
    setTestCase('a-body');
    const doc = await buildDoc(page);
    await caretInBody(page);
    await installRectSpy(page);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel === true, 'a: 선택 있음' + ` (실제 ${s.hasSel})`);
    assert(s.start.paragraphIndex === 0, 'a: 시작은 문서 첫 문단' + ` (실제 ${s.start.paragraphIndex})`);
    assert(s.start.parentParaIndex === undefined, 'a: 시작은 본문' + ` (실제 ${s.start.parentParaIndex})`);
    const paraCount = await page.evaluate(() => window.__wasm.getParagraphCount(0));
    assert(s.end.paragraphIndex === paraCount - 1, 'a: 끝은 문서 마지막 문단' + ` (실제 ${s.end.paragraphIndex})`);
    // 표 bbox 와 겹치는 하이라이트 rect 가 있어야 한다
    const { rects, unions } = await page.evaluate(async (d) => {
      const w = window.__wasm;
      const bs = w.getTableCellBboxes(0, d.paraIdx, d.controlIdx) || [];
      const byPage = new Map();
      for (const b of bs) {
        const u = byPage.get(b.pageIndex) ?? { x0: 1e9, y0: 1e9, x1: -1e9, y1: -1e9 };
        u.x0 = Math.min(u.x0, b.x); u.y0 = Math.min(u.y0, b.y);
        u.x1 = Math.max(u.x1, b.x + b.w); u.y1 = Math.max(u.y1, b.y + b.h);
        byPage.set(b.pageIndex, u);
      }
      return {
        rects: window.__lastSelRects || [],
        unions: [...byPage].map(([pageIndex, u]) => ({ pageIndex, ...u })),
      };
    }, doc);
    const overlaps = unions.some(u => rects.some(r => r.pageIndex === u.pageIndex
      && r.x < u.x1 && r.x + r.width > u.x0 && r.y < u.y1 && r.y + r.height > u.y0));
    assert(overlaps, `a: 표 영역이 하이라이트에 포함 (rects=${JSON.stringify(rects)}, table=${JSON.stringify(unions)})`);
    await screenshot(page, 'pr2-a-body-cmdA');
  }

  // ── (b) 셀 안 캐럿 → 그 셀 내용만 ──
  {
    setTestCase('b-cell');
    const doc = await buildDoc(page);
    await caretInCell(page, doc, 1, 1); // B2
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel === true, 'b: 선택 있음' + ` (실제 ${s.hasSel})`);
    assert(s.start.parentParaIndex === doc.paraIdx, 'b: 선택은 셀 안' + ` (실제 ${s.start.parentParaIndex})`);
    assert((s.start.cellParaIndex ?? s.start.cellPath?.at(-1)?.cellParaIndex) === 0, 'b: 셀 첫 문단');
    assert(s.start.charOffset === 0, 'b: 셀 첫 문단 시작' + ` (실제 ${s.start.charOffset})`);
    assert(s.pos.parentParaIndex === doc.paraIdx, 'b: 캐럿이 셀을 떠나지 않음' + ` (실제 ${s.pos.parentParaIndex})`);
    await screenshot(page, 'pr2-b-cell-cmdA');
    await key(page, { key: 'Backspace' });
    await page.evaluate(settle);
    assert(await cellText(page, doc, 1, 1) === '', 'b: B2 만 지워짐');
    assert(await cellText(page, doc, 0, 0) === 'A1', 'b: 다른 셀 보존');
    assert(await cellText(page, doc, 2, 2) === 'C3', 'b: 다른 셀 보존');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    await page.evaluate(settle);
    assert(await cellText(page, doc, 1, 1) === 'B2', 'b: 편집 뒤 undo가 B2 원문을 복원');
  }

  // ── (b2) 셀의 부분 선택 뒤 ⌘A → 기존 anchor 대신 셀 처음부터 ──
  {
    setTestCase('b2-cell-partial-selection');
    const doc = await buildDoc(page);
    await caretInCell(page, doc, 1, 1); // B2의 offset 1
    await key(page, { key: 'ArrowLeft', shift: true });
    const partial = await selState(page);
    assert(partial.hasSel && partial.start.charOffset === 0 && partial.end.charOffset === 1,
      `b2: 부분 선택 사전 조건 (실제 ${JSON.stringify(partial)})`);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    const s = await selState(page);
    assert(s.start.parentParaIndex === doc.paraIdx && s.end.parentParaIndex === doc.paraIdx,
      'b2: 선택 범위는 B2 셀 내부');
    assert(s.start.cellParaIndex === 0 && s.start.charOffset === 0
      && s.end.cellParaIndex === 0 && s.end.charOffset === 2,
    `b2: 기존 anchor 대신 B2 전체 선택 (실제 ${JSON.stringify(s)})`);
  }

  // ── (a2) 본문 부분 선택 뒤 ⌘A → 문서 첫 위치부터 ──
  {
    setTestCase('a2-body-partial-selection');
    await buildDoc(page);
    await caretInBody(page); // BEFORE의 offset 2
    await key(page, { key: 'ArrowRight', shift: true });
    const partial = await selState(page);
    assert(partial.hasSel && partial.start.charOffset === 2 && partial.end.charOffset === 3,
      `a2: 본문 부분 선택 사전 조건 (실제 ${JSON.stringify(partial)})`);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    const s = await selState(page);
    assert(s.hasSel && s.start.sectionIndex === 0 && s.start.paragraphIndex === 0
      && s.start.charOffset === 0 && s.start.parentParaIndex === undefined,
    `a2: 기존 anchor 대신 문서 첫 위치부터 선택 (실제 ${JSON.stringify(s)})`);
  }

  // ── (c) 여러 문단 셀 → 첫 문단 시작부터 마지막 문단 끝까지 ──
  {
    setTestCase('c-multi-paragraph-cell');
    const doc = await buildDoc(page);
    await page.evaluate(async (d) => {
      const w = window.__wasm;
      const b = (w.getTableCellBboxes(0, d.paraIdx, d.controlIdx, 0) || [])
        .find(x => x.row === 1 && x.col === 1);
      if (!b) throw new Error('B2 셀 없음');
      w.doc.splitParagraphInCell(0, d.paraIdx, d.controlIdx, b.cellIdx, 0, 2);
      w.doc.insertTextInCell(0, d.paraIdx, d.controlIdx, b.cellIdx, 1, 0, 'SECOND');
      await window.__canvasView?.loadDocument?.();
    }, doc);
    await caretInCell(page, doc, 1, 1);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel, 'c: 여러 문단 셀 선택 있음');
    assert(s.start.parentParaIndex === doc.paraIdx && s.end.parentParaIndex === doc.paraIdx,
      'c: 범위 양끝이 같은 셀 내부');
    assert(s.start.cellParaIndex === 0 && s.start.charOffset === 0,
      `c: 첫 문단 시작 (실제 ${JSON.stringify(s.start)})`);
    assert(s.end.cellParaIndex === 1 && s.end.charOffset === 6,
      `c: 마지막 문단 끝 (실제 ${JSON.stringify(s.end)})`);
    assert(await cellText(page, doc, 1, 1, 0) === 'B2', 'c: 첫 문단 원문');
    assert(await cellText(page, doc, 1, 1, 1) === 'SECOND', 'c: 둘째 문단 원문');
    await key(page, { key: 'Backspace' });
    await page.evaluate(settle);
    const edited = await page.evaluate(({ paraIdx, controlIdx }) => {
      const w = window.__wasm;
      const b = (w.getTableCellBboxes(0, paraIdx, controlIdx, 0) || [])
        .find(x => x.row === 1 && x.col === 1);
      return {
        count: w.getCellParagraphCount(0, paraIdx, controlIdx, b.cellIdx),
        first: w.getTextInCell(0, paraIdx, controlIdx, b.cellIdx, 0, 0, 100),
      };
    }, doc);
    assert(edited.count === 1 && edited.first === '',
      `c: B2와 SECOND가 삭제되고 빈 문단 하나만 남음 (실제 ${JSON.stringify(edited)})`);
    assert(await cellText(page, doc, 0, 0) === 'A1', 'c: 다른 셀 보존');
    assert(await cellText(page, doc, 2, 2) === 'C3', 'c: 다른 셀 보존');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    await page.evaluate(settle);
    assert(await cellText(page, doc, 1, 1, 0) === 'B2', 'c: undo가 첫 문단 B2 복원');
    assert(await cellText(page, doc, 1, 1, 1) === 'SECOND', 'c: undo가 둘째 문단 SECOND 복원');
  }

  // ── (n) 런타임 HTML 붙여넣기로 만든 중첩 표의 안쪽 셀만 선택 ──
  {
    setTestCase('n-nested-table-cell');
    await createNewDocument(page);
    await resetModes(page);
    const nested = await page.evaluate(async () => {
      const w = window.__wasm;
      const result = JSON.parse(w.pasteHtml(0, 0, 0,
        '<table><tr><td>OUTER<table><tr><td>INNER</td><td>PEER</td></tr></table></td><td>OUTERPEER</td></tr></table>'));
      if (!result.ok) throw new Error(`중첩 표 pasteHtml 실패: ${JSON.stringify(result)}`);
      const controls = w.getControls();
      const outer = controls.find(c => c.ctrlId === 'tbl' && c.list === 0);
      const inner = controls.find(c => c.ctrlId === 'tbl' && c.list !== 0);
      if (!outer || !inner) throw new Error(`중첩 표 컨트롤 없음: ${JSON.stringify(controls)}`);
      const path = [
        { controlIndex: outer.controlIndex, cellIndex: 0, cellParaIndex: inner.para },
        { controlIndex: inner.controlIndex, cellIndex: 0, cellParaIndex: 0 },
      ];
      const text = w.getTextInCellByPath(0, outer.para, JSON.stringify(path), 0, 100);
      if (text !== 'INNER') throw new Error(`안쪽 셀 내용 불일치: ${text}`);
      await window.__canvasView?.loadDocument?.();
      return { paraIdx: outer.para, path };
    });
    await page.evaluate(({ paraIdx, path }) => {
      const ih = window.__inputHandler;
      ih.cursor.moveTo({
        sectionIndex: 0, paragraphIndex: 0, charOffset: 2,
        parentParaIndex: paraIdx,
        controlIndex: path[0].controlIndex, cellIndex: path[0].cellIndex,
        cellParaIndex: path[1].cellParaIndex, cellPath: path,
      });
      ih.updateCaret(); ih.focus();
    }, nested);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel, 'n: 안쪽 셀 선택 있음');
    assert(s.start.parentParaIndex === nested.paraIdx && s.end.parentParaIndex === nested.paraIdx,
      'n: 범위 양끝이 표 내부');
    assert(JSON.stringify(s.start.cellPath) === JSON.stringify(nested.path)
      && JSON.stringify(s.end.cellPath) === JSON.stringify(nested.path),
    `n: 범위가 안쪽 셀 깊이 2에 한정 (실제 ${JSON.stringify(s)})`);
    assert(s.start.charOffset === 0 && s.end.charOffset === 5,
      `n: INNER 텍스트 전체 범위 (실제 ${JSON.stringify(s)})`);
    const texts = () => page.evaluate(({ paraIdx, path }) => {
      const w = window.__wasm;
      const read = (p) => w.getTextInCellByPath(0, paraIdx, JSON.stringify(p), 0, 100);
      return {
        inner: read(path),
        peer: read(path.map((entry, i) => i === 1 ? { ...entry, cellIndex: 1 } : entry)),
        outerPeer: w.getTextInCell(0, paraIdx, path[0].controlIndex, 1, 0, 0, 100),
      };
    }, nested);
    await key(page, { key: 'Backspace' });
    await page.evaluate(settle);
    assert(JSON.stringify(await texts()) === JSON.stringify({ inner: '', peer: 'PEER', outerPeer: 'OUTERPEER' }),
      'n: 편집은 안쪽 셀에만 적용');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    await page.evaluate(settle);
    assert(JSON.stringify(await texts()) === JSON.stringify({ inner: 'INNER', peer: 'PEER', outerPeer: 'OUTERPEER' }),
      'n: undo가 안쪽 셀 원문을 복원');
  }

  // ── (h) 글상자 안 캐럿 → 글상자 텍스트만 선택 ──
  {
    setTestCase('h-textbox');
    await resetModes(page);
    // loadHwpFile는 제품의 문서 전환 수명주기를 거치지 않으므로 이전 문서의 undo를 비운다.
    await page.evaluate(() => {
      const ih = window.__inputHandler;
      ih.history.clear(ih.wasm);
    });
    await loadHwpFile(page, 'hml/formatting_table.hml');
    const box = await page.evaluate(() => {
      const w = window.__wasm;
      const shape = w.getControls().find(c => c.ctrlId === 'gso' && c.list === 0);
      if (!shape) throw new Error('샘플 글상자 없음');
      const paraIdx = shape.para;
      const controlIdx = shape.controlIndex;
      const text = w.getTextInCell(0, paraIdx, controlIdx, 0, 0, 0, 100);
      if (text !== 'textbox') throw new Error(`샘플 글상자 내용 불일치: ${text}`);
      return { paraIdx, controlIdx, text };
    });
    const bodyBefore = await page.evaluate(() => {
      const w = window.__wasm;
      return Array.from({ length: w.getParagraphCount(0) }, (_, p) => w.getTextRange(0, p, 0, 1000));
    });
    await page.evaluate(({ paraIdx, controlIdx }) => {
      const ih = window.__inputHandler;
      ih.cursor.moveTo({
        sectionIndex: 0, paragraphIndex: 0, charOffset: 2,
        parentParaIndex: paraIdx, controlIndex: controlIdx,
        cellIndex: 0, cellParaIndex: 0, isTextBox: true,
        cellPath: [{ controlIndex: controlIdx, cellIndex: 0, cellParaIndex: 0 }],
      });
      ih.updateCaret(); ih.focus();
    }, box);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel, 'h: 글상자 안 선택 있음');
    assert(s.start.parentParaIndex === box.paraIdx && s.end.parentParaIndex === box.paraIdx,
      'h: 범위 양끝이 글상자 안');
    assert(s.start.isTextBox === true && s.end.isTextBox === true, 'h: 글상자 문맥 유지');
    assert(s.start.charOffset === 0 && s.end.charOffset === box.text.length,
      `h: 글상자 텍스트 전체 범위 (실제 ${JSON.stringify(s)})`);
    assert(s.start.cellIndex === 0 && s.end.cellIndex === 0, 'h: 선택은 샘플 글상자 내부');
    await key(page, { key: 'Backspace' });
    await page.evaluate(settle);
    const after = await page.evaluate(({ paraIdx, controlIdx }) => {
      const w = window.__wasm;
      return {
        text: w.getTextInCell(0, paraIdx, controlIdx, 0, 0, 0, 100),
        body: Array.from({ length: w.getParagraphCount(0) }, (_, p) => w.getTextRange(0, p, 0, 1000)),
      };
    }, box);
    assert(after.text === '', `h: 글상자 텍스트 삭제 (실제 ${after.text})`);
    assert(JSON.stringify(after.body) === JSON.stringify(bodyBefore), 'h: 본문 보존');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    await page.evaluate(settle);
    assert(await page.evaluate(({ paraIdx, controlIdx }) =>
      window.__wasm.getTextInCell(0, paraIdx, controlIdx, 0, 0, 0, 100), box) === box.text,
    'h: undo가 글상자 원문 복원');
  }

  // ── (d) 셀 블록(F5) → 블록 해제 + 현재 셀 선택 ──
  {
    setTestCase('d-cellblock');
    const doc = await buildDoc(page);
    await caretInCell(page, doc, 0, 0);
    await key(page, { key: 'F5', code: 'F5' });
    await page.evaluate(settle);
    const before = await selState(page);
    assert(before.cellSel === true, 'd: 셀 블록 진입 확인' + ` (실제 ${before.cellSel})`);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.cellSel === false, 'd: 블록 해제' + ` (실제 ${s.cellSel})`);
    assert(s.hasSel === true, 'd: 선택 있음' + ` (실제 ${s.hasSel})`);
    assert(s.start.parentParaIndex === doc.paraIdx, 'd: 셀 안 선택' + ` (실제 ${s.start.parentParaIndex})`);
    await screenshot(page, 'pr2-d-cellblock-cmdA');
  }

  // ── (e) 툴바 버튼 포커스에서 실제 CDP ⌘A → 편집기 전체 선택 + 포커스 복귀 ──
  {
    setTestCase('e-toolbar');
    const doc = await buildDoc(page);
    await caretInBody(page);
    await page.evaluate(() => {
      const btn = document.querySelector('button');
      btn?.focus();
    });
    const active = await page.evaluate(() => document.activeElement?.tagName);
    assert(active === 'BUTTON', 'e: 포커스가 버튼에 있음을 확인' + ` (실제 ${active})`);
    await page.keyboard.down('Meta');
    await page.keyboard.press('a');
    await page.keyboard.up('Meta');
    await page.evaluate(settle);
    const s = await selState(page);
    const after = await page.evaluate(() => document.activeElement?.tagName);
    assert(s.hasSel === true, 'e: textarea 밖 포커스에서도 전체 선택' + ` (실제 ${s.hasSel})`);
    assert(after === 'TEXTAREA', `e: 포커스가 편집기 textarea로 복귀 (실제 ${after})`);
  }

  // ── (e2) 네이티브 입력란·선택 목록은 문서 ⌘A에 빼앗기지 않는다 ──
  {
    setTestCase('e2-input-and-select');
    await page.evaluate(() => {
      const ih = window.__inputHandler;
      ih.cursor.clearSelection();
      window.__selectAllPrevented = null;
      document.addEventListener('keydown', (event) => {
        if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'a') {
          window.__selectAllPrevented = event.defaultPrevented;
        }
      });
      const input = document.createElement('input');
      input.id = 'select-all-native-input';
      input.value = 'NATIVE INPUT';
      document.body.appendChild(input);
      input.focus();
    });
    await page.keyboard.down('Meta');
    await page.keyboard.press('a');
    await page.keyboard.up('Meta');
    const inputState = await page.evaluate(() => {
      return {
        focused: document.activeElement?.id === 'select-all-native-input',
        prevented: window.__selectAllPrevented,
        documentSelected: window.__inputHandler.cursor.hasSelection(),
      };
    });
    assert(inputState.focused && inputState.prevented === false && !inputState.documentSelected,
      `e2: 입력란의 ⌘A는 문서가 가로채지 않음 (실제 ${JSON.stringify(inputState)})`);

    await page.evaluate(() => {
      const select = document.createElement('select');
      select.id = 'select-all-native-select';
      select.innerHTML = '<option>ONE</option><option>TWO</option>';
      document.body.appendChild(select);
      select.focus();
    });
    await page.keyboard.down('Meta');
    await page.keyboard.press('a');
    await page.keyboard.up('Meta');
    const selectState = await page.evaluate(() => ({
      focused: document.activeElement?.id === 'select-all-native-select',
      prevented: window.__selectAllPrevented,
      documentSelected: window.__inputHandler.cursor.hasSelection(),
    }));
    assert(selectState.focused && selectState.prevented === false && !selectState.documentSelected,
      `e2: 선택 목록의 ⌘A를 문서가 가로채지 않음 (실제 ${JSON.stringify(selectState)})`);
    await page.evaluate(() => {
      document.getElementById('select-all-native-input')?.remove();
      document.getElementById('select-all-native-select')?.remove();
    });
  }

  // ── (f) 한글 IME 스타일 ⌘A (key 'ㅁ', code 'KeyA') ──
  {
    setTestCase('f-ime');
    await buildDoc(page);
    await caretInBody(page);
    await key(page, { key: 'ㅁ', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    assert(s.hasSel === true, 'f: IME ⌘A 전체 선택' + ` (실제 ${s.hasSel})`);
  }

  // ── (g) ⌘A 뒤 Backspace → 표 포함 전체 삭제 (회귀) ──
  {
    setTestCase('g-delete-all');
    const doc = await buildDoc(page);
    await caretInBody(page);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await key(page, { key: 'Backspace' });
    await page.evaluate(settle);
    const after = await page.evaluate((d) => {
      const w = window.__wasm;
      const n = w.getParagraphCount(0);
      const body = [];
      for (let p = 0; p < n; p++) body.push(w.getTextRange(0, p, 0, 50));
      const bboxes = w.getTableCellBboxes(0, d.paraIdx, d.controlIdx) || [];
      return { n, body, tableLeft: bboxes.length };
    }, doc);
    assert(after.tableLeft === 0, 'g: 표도 삭제됨' + ` (실제 ${after.tableLeft})`);
    assert(after.n === 1, `g: 문단 1개만 남음 (실제 ${after.n})`);
    assert((after.body[0] ?? '') === '', 'g: 본문도 비었음');
  }

  // ── (s) 여러 쪽 문서에서 ⌘A는 스크롤 위치를 바꾸지 않는다 (맨 아래로 튀는 결함 회귀) ──
  {
    setTestCase('s-no-scroll-jump');
    await createNewDocument(page);
    await resetModes(page);
    const pages = await page.evaluate(async () => {
      const w = window.__wasm;
      const nf = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
      for (let i = 0; i < 150; i++) {
        w.doc.insertText(0, i, 0, `LINE ${i}`);
        w.doc.splitParagraph(0, i, String(`LINE ${i}`).length);
      }
      window.__canvasView?.loadDocument?.();
      await nf(); await nf();
      return w.pageCount ?? w.getPageCount?.();
    });
    await caretInBody(page);
    const before = await page.evaluate(async () => {
      const c = document.getElementById('scroll-container');
      c.scrollTop = 0;
      await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
      return { top: c.scrollTop, max: c.scrollHeight - c.clientHeight };
    });
    assert(before.max > 500, `s: 스크롤할 만큼 긴 문서 (max ${before.max}, pages ${pages})`);
    await key(page, { key: 'a', code: 'KeyA', meta: true });
    await page.evaluate(settle);
    const s = await selState(page);
    const top = await page.evaluate(() => document.getElementById('scroll-container').scrollTop);
    assert(s.hasSel === true, 's: 전체 선택됨');
    assert(s.start.paragraphIndex === 0 && s.start.charOffset === 0, `s: 문서 시작부터 (실제 ${JSON.stringify(s.start)})`);
    assert(s.end.paragraphIndex >= 150, `s: 문서 끝까지 (실제 ${JSON.stringify(s.end)})`);
    assert(top === before.top, `s: 스크롤 위치 유지 (전 ${before.top}, 후 ${top})`);
  }

  assert(pageErrors.length === 0, `pageerror 0건 (실제: ${JSON.stringify(pageErrors)})`);
});
