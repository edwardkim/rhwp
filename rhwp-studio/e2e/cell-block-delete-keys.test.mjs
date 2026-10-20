/**
 * E2E: 셀 블록 지우기 키 — macOS 정합.
 *
 * 한컴 Mac 단축키표: 셀 내용 지우기 = Delete(=브라우저 Backspace)·fn+Delete(=Delete),
 * 지우기(셀 삭제) = ⌘E. ⌘⌫·⌘Delete 는 같은 지우기 경로로 매핑한다.
 *
 * 시나리오 (3x3 표, 셀 텍스트 A1..C3, 앞뒤 본문 BEFORE/AFTER):
 *  (a) 2x2 부분 블록 + Backspace  → 4칸 내용만, 블록 유지, 대화상자 없음
 *  (b) 부분 블록 + Meta+Backspace → 동일 (묻지 않음)
 *  (c) 부분 블록 + Meta+E        → 동일
 *  (d) 부분 블록 + Meta+Delete   → 동일
 *  (e) 전체 줄 블록 + Meta+Backspace + [남김] → 내용만 지움, 3줄 유지
 *  (f) 전체 줄 블록 + Meta+Backspace + [지우기] → 줄 구조 삭제, Meta+Z 로 복원
 *  (g) 전체 표 블록 + Meta+Backspace + [지우기] → 표 삭제, 본문 보존
 *  (h) 전체 줄 블록 + [취소] → 아무 변화 없음
 * 모든 케이스에서 pageerror 0건을 요구한다.
 */
import { runTest, createNewDocument, clickEditArea, screenshot, assert } from './helpers.mjs';

const pageErrors = [];

const key = (page, init) => page.evaluate((i) => {
  const ih = window.__inputHandler;
  ih.textarea.dispatchEvent(new KeyboardEvent('keydown', {
    key: i.key, code: i.code || '', shiftKey: !!i.shift, ctrlKey: !!i.ctrl,
    metaKey: !!i.meta, altKey: !!i.alt, bubbles: true, cancelable: true,
  }));
}, init).then(() => page.evaluate(() => new Promise(r => setTimeout(r, 250))));

// 셀 선택/개체 선택 모드는 createNewDocument 를 넘어 남는다 — 케이스마다 명시 해제.
const resetModes = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  const cur = ih.cursor;
  try { if (cur.isInCellSelectionMode?.()) cur.exitCellSelectionMode(); } catch {}
  try { if (cur.isInTableObjectSelection?.()) cur.exitTableObjectSelection?.(); } catch {}
  try { if (cur.isInPictureObjectSelection?.()) cur.exitPictureObjectSelection?.(); } catch {}
  try { if (cur.isInBlockSelectionMode?.()) cur.exitBlockSelectionMode(); } catch {}
  ih.cellSelectionRenderer?.clear?.();
  ih.updateCaret?.();
  // 이전 케이스가 남긴 모달은 취소로 닫아 pending Promise 를 해소한다.
  document.querySelectorAll('.modal-overlay').forEach((ov) => {
    ov.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    ov.remove();
  });
});

const buildDoc = (page) => createNewDocument(page)
  .then(() => resetModes(page))
  .then(() => page.evaluate(async () => {
  const w = window.__wasm;
  const nextFrame = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
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
  await nextFrame(); await nextFrame();
  return { paraIdx: created.paraIdx, controlIdx: created.controlIdx };
}));

const caretInCell = (page, doc, row, col) => page.evaluate(({ paraIdx, controlIdx, row, col }) => {
  const w = window.__wasm;
  const ih = window.__inputHandler;
  const b = (w.getTableCellBboxes(0, paraIdx, controlIdx, 0) || []).find(x => x.row === row && x.col === col);
  if (!b) throw new Error(`cell (${row},${col}) 없음`);
  ih.cursor.moveToCellByIndex(0, paraIdx, controlIdx,
    [{ controlIndex: controlIdx, cellIndex: b.cellIdx, cellParaIndex: 0 }], b.cellIdx, 'end');
  ih.updateCaret(); ih.focus();
}, { ...doc, row, col });

const snapshot = (page, doc) => page.evaluate(({ paraIdx, controlIdx }) => {
  const w = window.__wasm;
  const ih = window.__inputHandler;
  let dims = null;
  const cells = {};
  try {
    const bboxes = w.getTableCellBboxes(0, paraIdx, controlIdx, 0) || [];
    const rows = new Set(); const cols = new Set();
    for (const b of bboxes) {
      rows.add(b.row); cols.add(b.col);
      const n = w.getCellParagraphCount(0, paraIdx, controlIdx, b.cellIdx);
      const parts = [];
      for (let p = 0; p < n; p++) parts.push(w.getTextInCell(0, paraIdx, controlIdx, b.cellIdx, p, 0, 100));
      cells[`${'ABC'[b.col]}${b.row + 1}`] = parts.join('¶');
    }
    dims = bboxes.length ? { rows: rows.size, cols: cols.size } : null;
  } catch { dims = null; }
  const n = w.getParagraphCount(0);
  const body = [];
  for (let p = 0; p < n; p++) body.push(w.getTextRange(0, p, 0, 50));
  return { dims, cells, body, cellSel: !!ih.cursor.isInCellSelectionMode(), phase: ih.cursor.getCellSelectionPhase() };
}, doc);

const dialogOpen = (page) => page.evaluate(() => !!document.querySelector('.modal-overlay .dialog-wrap'));
const clickDialogButton = (page, label) => page.evaluate((want) => {
  const btn = [...document.querySelectorAll('.modal-overlay .dialog-footer button')]
    .find(b => b.textContent?.trim() === want);
  if (!btn) return false;
  btn.click();
  return true;
}, label).then(ok => page.evaluate(() => new Promise(r => setTimeout(r, 400))).then(() => ok));

const select2x2 = async (page, doc) => {
  await caretInCell(page, doc, 0, 0);
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'ArrowDown', code: 'ArrowDown' });
  await key(page, { key: 'ArrowRight', code: 'ArrowRight' });
};
const selectRow1 = async (page, doc) => {
  await caretInCell(page, doc, 1, 0);
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'ArrowRight', code: 'ArrowRight' });
  await key(page, { key: 'ArrowRight', code: 'ArrowRight' });
};
const selectAll = async (page, doc) => {
  await caretInCell(page, doc, 0, 0);
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'F5', code: 'F5' });
  await key(page, { key: 'F5', code: 'F5' });
};

const FULL = { A1: 'A1', B1: 'B1', C1: 'C1', A2: 'A2', B2: 'B2', C2: 'C2', A3: 'A3', B3: 'B3', C3: 'C3' };

await runTest('셀 블록 지우기 키 — Backspace/Delete/⌘⌫/⌘E/⌘Delete', async ({ page }) => {
  page.on('pageerror', (err) => pageErrors.push(String(err)));
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // ── (a)~(d): 부분 블록 + 내용 지우기 키들 ──
  const partial = [
    ['a-Backspace', { key: 'Backspace' }],
    ['b-Meta+Backspace', { key: 'Backspace', meta: true }],
    ['c-Meta+E', { key: 'e', code: 'KeyE', meta: true }],
    ['d-Meta+Delete', { key: 'Delete', meta: true }],
  ];
  for (const [name, k] of partial) {
    const doc = await buildDoc(page);
    await select2x2(page, doc);
    await key(page, k);
    const s = await snapshot(page, doc);
    assert(s.cells.A1 === '' && s.cells.B1 === '' && s.cells.A2 === '' && s.cells.B2 === ''
      && s.cells.C1 === 'C1' && s.cells.C2 === 'C2' && s.cells.A3 === 'A3' && s.cells.B3 === 'B3' && s.cells.C3 === 'C3',
      `${name}: 2x2 블록 내용만 지워짐 (실제 ${JSON.stringify(s.cells)})`);
    assert(s.cellSel === true, `${name}: 셀 블록 유지`);
    assert(s.dims?.rows === 3 && s.dims?.cols === 3, `${name}: 표 차수 3x3 유지`);
    assert(await dialogOpen(page) === false, `${name}: 대화상자가 열리지 않음`);
  }

  // ── (e) 전체 줄 + Meta+Backspace + 남김 ──
  {
    const doc = await buildDoc(page);
    await selectRow1(page, doc);
    await key(page, { key: 'Backspace', meta: true });
    assert(await dialogOpen(page), 'e: 구조 삭제 확인 대화상자 표시');
    assert(await clickDialogButton(page, '남김'), 'e: [남김] 클릭');
    const s = await snapshot(page, doc);
    assert(s.cells.A2 === '' && s.cells.B2 === '' && s.cells.C2 === '' && s.cells.A1 === 'A1',
      `e: 남김 — 줄 내용만 지움 (실제 ${JSON.stringify(s.cells)})`);
    assert(s.dims?.rows === 3, `e: 남김 — 줄 수 3 유지 (실제 ${s.dims?.rows})`);
    assert(s.cellSel === true, 'e: 남김 — 셀 블록 유지');
  }

  // ── (f) 전체 줄 + Meta+Backspace + 지우기 → undo ──
  {
    const doc = await buildDoc(page);
    await selectRow1(page, doc);
    await key(page, { key: 'Backspace', meta: true });
    assert(await dialogOpen(page), 'f: 대화상자 표시');
    assert(await clickDialogButton(page, '지우기'), 'f: [지우기] 클릭');
    const s = await snapshot(page, doc);
    assert(s.dims?.rows === 2, `f: 지우기 — 줄 수 2 (실제 ${JSON.stringify(s.dims)})`);
    assert(s.cells.A1 === 'A1' && s.cells.A2 === 'A3' && s.cells.C2 === 'C3',
      `f: 남은 줄 텍스트 (실제 ${JSON.stringify(s.cells)})`);
    assert(s.cellSel === false, 'f: 구조 삭제 뒤 블록 해제');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    const u = await snapshot(page, doc);
    assert(u.dims?.rows === 3 && u.cells.A2 === 'A2' && u.cells.C2 === 'C2',
      `f: ⌘Z 복원 — 3줄+텍스트 (실제 ${JSON.stringify(u.cells)})`);
    assert(u.cellSel === true, 'f: undo 뒤 셀 블록 복원');
  }

  // ── (g) 전체 표 + Meta+Backspace + 지우기 ──
  {
    const doc = await buildDoc(page);
    await selectAll(page, doc);
    await key(page, { key: 'Backspace', meta: true });
    assert(await dialogOpen(page), 'g: 대화상자 표시');
    assert(await clickDialogButton(page, '지우기'), 'g: [지우기] 클릭');
    const s = await snapshot(page, doc);
    assert(s.dims === null, `g: 표 삭제됨 (실제 ${JSON.stringify(s.dims)})`);
    assert(s.body.includes('BEFORE') && s.body.includes('AFTER'),
      `g: 앞뒤 본문 보존 (실제 ${JSON.stringify(s.body)})`);
  }

  // ── (h) 취소 ──
  {
    const doc = await buildDoc(page);
    await selectRow1(page, doc);
    await key(page, { key: 'Backspace', meta: true });
    assert(await dialogOpen(page), 'h: 대화상자 표시');
    assert(await clickDialogButton(page, '취소'), 'h: [취소] 클릭');
    const s = await snapshot(page, doc);
    assert(JSON.stringify(s.cells) === JSON.stringify(FULL) && s.dims?.rows === 3,
      `h: 취소 — 아무 변화 없음 (실제 ${JSON.stringify(s.cells)})`);
    assert(s.cellSel === true, 'h: 취소 — 셀 블록 유지');
  }

  // ── (i) 대화상자 포커스 순환·설명 연결·Escape 취소 ──
  {
    const doc = await buildDoc(page);
    await selectRow1(page, doc);
    await key(page, { key: 'e', code: 'KeyE', meta: true });
    assert(await dialogOpen(page), 'i: ⌘E 대화상자 표시');
    const dialogState = () => page.evaluate(() => {
      const dialog = document.querySelector('.modal-overlay .dialog-wrap');
      const describedBy = dialog?.getAttribute('aria-describedby');
      return {
        focused: document.activeElement?.textContent?.trim(),
        describedBy,
        description: describedBy && document.getElementById(describedBy)?.textContent?.trim(),
      };
    });
    const initial = await dialogState();
    assert(initial.focused === '지우기', `i: 초기 포커스 [지우기] (실제 ${initial.focused})`);
    assert(initial.description?.length > 0, 'i: aria-describedby가 본문 id를 가리킴');
    for (const label of ['남김', '취소', '×', '지우기']) {
      await page.keyboard.press('Tab');
      const state = await dialogState();
      assert(state.focused === label, `i: Tab → ${label} (실제 ${state.focused})`);
    }
    for (const label of ['×', '취소', '남김', '지우기']) {
      await page.keyboard.down('Shift');
      await page.keyboard.press('Tab');
      await page.keyboard.up('Shift');
      const state = await dialogState();
      assert(state.focused === label, `i: Shift+Tab → ${label} (실제 ${state.focused})`);
    }
    await page.keyboard.press('Escape');
    assert(await dialogOpen(page) === false, 'i: Escape가 대화상자를 닫음');
    const s = await snapshot(page, doc);
    assert(JSON.stringify(s.cells) === JSON.stringify(FULL) && s.dims?.rows === 3,
      'i: Escape=취소, 내용과 행 수 유지');
  }

  // 실제 키 입력으로 포커스된 버튼의 Enter/Space 활성화를 확인한다.
  for (const activation of ['Enter', 'Space']) {
    for (const [label, tabs] of [['지우기', 0], ['남김', 1], ['취소', 2], ['×', 3]]) {
      const doc = await buildDoc(page);
      await selectRow1(page, doc);
      await key(page, { key: 'e', code: 'KeyE', meta: true });
      for (let i = 0; i < tabs; i++) await page.keyboard.press('Tab');
      assert(await page.evaluate(() => document.activeElement?.textContent?.trim()) === label,
        `${activation}: ${label} 포커스`);
      await page.keyboard.press(activation);
      await page.waitForFunction(() => !document.querySelector('.modal-overlay'));
      const s = await snapshot(page, doc);
      assert(s.dims?.rows === (label === '지우기' ? 2 : 3), `${activation}: ${label} 행 수`);
      if (label === '남김') {
        assert(s.cells.A2 === '' && s.cells.C2 === '' && s.cells.A3 === 'A3',
          `${activation}: 남김은 선택 내용만 지움`);
      } else if (label === '취소' || label === '×') {
        assert(JSON.stringify(s.cells) === JSON.stringify(FULL), `${activation}: ${label} 내용 보존`);
      }
    }
  }

  // 마지막 행/열 삭제 뒤 경로 기반 커서, 이동·입력, Undo/Redo를 검사한다.
  for (const axis of ['row', 'column']) {
    const doc = await buildDoc(page);
    await page.evaluate(({ paraIdx, controlIdx }) => {
      const w = window.__wasm;
      const b = w.getTableCellBboxes(0, paraIdx, controlIdx, 0).find(x => x.row === 2 && x.col === 2);
      w.doc.splitParagraphInCell(0, paraIdx, controlIdx, b.cellIdx, 0, 2);
      w.doc.insertTextInCell(0, paraIdx, controlIdx, b.cellIdx, 1, 0, 'TAIL');
      window.__canvasView.loadDocument();
    }, doc);
    await caretInCell(page, doc, 2, 2);
    await key(page, { key: 'F5', code: 'F5' });
    await key(page, { key: 'F5', code: 'F5' });
    for (let i = 0; i < 2; i++) await key(page, {
      key: axis === 'row' ? 'ArrowLeft' : 'ArrowUp',
    });
    await key(page, { key: 'e', code: 'KeyE', meta: true });
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => !document.querySelector('.modal-overlay'));
    const cursorState = () => page.evaluate(() => {
      const cursor = window.__inputHandler.cursor;
      return { pos: cursor.getPosition(), rect: cursor.getRect() };
    });
    const checkCursor = async (stage) => {
      const { pos, rect } = await cursorState();
      assert(pos.cellPath?.length === 1 && pos.cellPath[0].cellIndex === pos.cellIndex,
        `${axis}/${stage}: flat 셀과 경로 일치`);
      assert(pos.paragraphIndex === pos.cellParaIndex && pos.cellParaIndex === pos.cellPath[0].cellParaIndex,
        `${axis}/${stage}: 문단 좌표 일치`);
      assert(rect !== null, `${axis}/${stage}: 커서 rect 존재`);
    };
    const deleted = await snapshot(page, doc);
    assert(deleted.dims?.[axis === 'row' ? 'rows' : 'cols'] === 2, `${axis}: 마지막 구조 삭제`);
    await checkCursor('delete');
    await key(page, { key: 'z', code: 'KeyZ', meta: true });
    const restored = await snapshot(page, doc);
    assert(restored.dims?.rows === 3 && restored.dims?.cols === 3 && restored.cells.C3 === 'C3¶TAIL',
      `${axis}: Undo 내용과 구조 복원`);
    assert(restored.cellSel, `${axis}: Undo 블록 복원`);
    await key(page, { key: 'z', code: 'KeyZ', meta: true, shift: true });
    assert((await snapshot(page, doc)).dims?.[axis === 'row' ? 'rows' : 'cols'] === 2, `${axis}: Redo 구조 삭제`);
    await checkCursor('redo');
    await key(page, { key: 'ArrowRight', code: 'ArrowRight' });
    await page.keyboard.type('X');
    await checkCursor('input');
    assert(Object.values((await snapshot(page, doc)).cells).some(text => text.includes('X')),
      `${axis}: 커서 이동 뒤 입력이 남은 셀에 반영`);
  }

  await screenshot(page, 'cell-block-delete-keys');
  assert(pageErrors.length === 0, `pageerror 0건 (실제: ${JSON.stringify(pageErrors.slice(0, 3))})`);
  await page.close();
});
