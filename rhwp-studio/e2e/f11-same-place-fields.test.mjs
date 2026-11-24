/**
 * E2E: F11로 같은 글자 자리의 빈 누름틀을 하나씩 고르고, 고른 누름틀에 입력한다.
 *
 * 한/글 12.30(Mac)은 "앞 뒤끝"의 글자 자리 2에 차례로 넣은 빈 누름틀 가·나를 ⌃F11로 나, 가 차례로 고르고,
 * 고른 누름틀에 친 글이 그 누름틀에 든다. 셀 안에서는 그 셀의 누름틀, 그 표, 표 앞 본문 누름틀 차례다.
 *
 * 시나리오 (실제 F11 키):
 *  (a) 문단 끝에서 F11 → 나, 다시 F11 → 가, 다시 F11 → 고를 것 없음
 *  (b) F11 한 번 뒤 입력 → 글이 나에 든다
 *  (c) F11 두 번 뒤 입력 → 글이 가에 든다
 *  (d) 셀 안에서 F11 → 셀 누름틀, 표, 본문 누름틀
 */
import { runTest, createNewDocument, getParaText, assert } from './helpers.mjs';

const settle = (page) => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));

const f11 = async (page) => {
  await page.keyboard.press('F11');
  await settle(page);
};

/** 본문이나 1×1 표의 같은 글자 자리에 빈 누름틀 두 개를 만든다. */
async function buildSamePlace(page, area = 'body') {
  await createNewDocument(page);
  await page.evaluate((area) => {
    const w = window.__wasm;
    const ih = window.__inputHandler;
    const position = { sectionIndex: 0, paragraphIndex: 0, charOffset: 4 };
    const table = area === 'cell' ? w.createTable(0, 0, 0, 1, 1) : null;
    if (table) {
      if (!table.ok) throw new Error(JSON.stringify(table));
      w.doc.insertTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, '앞 뒤끝');
      Object.assign(position, { parentParaIndex: table.paraIdx, controlIndex: table.controlIdx, cellIndex: 0, cellParaIndex: 0 });
    } else {
      w.doc.insertText(0, 0, 0, '앞 뒤끝');
    }
    for (const name of ['가', '나']) {
      const result = table
        ? w.doc.insertClickHereFieldInCell(0, table.paraIdx, table.controlIdx, 0, 0, 2, false, name + '안내', '', name, true)
        : w.doc.insertClickHereField(0, 0, 2, name + '안내', '', name, true);
      if (!JSON.parse(result).ok) throw new Error(result);
    }
    window.__canvasView.loadDocument();
    ih.cursor.moveTo(position);
    ih.updateCaret();
    ih.focus();
  }, area);
}

async function typeHangul(page) {
  const cdp = await page.createCDPSession();
  try {
    for (const [initial, syllable] of [['하', '한'], ['그', '글']]) {
      for (const text of [initial, syllable]) {
        await cdp.send('Input.imeSetComposition', { text, selectionStart: 1, selectionEnd: 1 });
      }
      await cdp.send('Input.insertText', { text: syllable });
    }
  } finally {
    await cdp.detach();
  }
}

/** 지금 F11이 고른 것: 누름틀이면 그 이름, 표면 'table', 없으면 null. */
const picked = (page) => page.evaluate(() => {
  const ih = window.__inputHandler;
  if (ih.cursor.isInTableObjectSelection()) return 'table';
  const pin = ih.f11Field;
  if (!pin) return null;
  const list = JSON.parse(window.__wasm.doc.getFieldList());
  return list.find((f) => f.controlIndex === pin.controlIndex && f.fieldId === pin.info.fieldId)?.name ?? '?';
});

const savedValues = (page) => page.evaluate(() => {
  const doc = window.__wasm.doc;
  const values = (d) => Object.fromEntries(JSON.parse(d.getFieldList()).map((f) => [f.name, f.value]));
  const snapshots = { live: values(doc) };
  for (const [format, bytes] of [['HWP', doc.exportHwp()], ['HWPX', doc.exportHwpx()]]) {
    const reopened = new doc.constructor(bytes);
    try { snapshots[format] = values(reopened); }
    finally { reopened.free(); }
  }
  return snapshots;
});

runTest('F11 같은 자리 누름틀', async ({ page }) => {
  await buildSamePlace(page);
  await f11(page);
  assert(await picked(page) === '나', 'F11 한 번: 나');
  await f11(page);
  assert(await picked(page) === '가', 'F11 두 번: 가');
  await f11(page);
  assert(await picked(page) === null, 'F11 세 번: 고를 것 없음');

  for (const area of ['body', 'cell']) {
    for (const chosen of ['나', '가']) {
      for (const input of ['xy', '한글']) {
        await buildSamePlace(page, area);
        await f11(page);
        if (chosen === '가') await f11(page);
        assert(await picked(page) === chosen, `${area} F11 선택 ${chosen}`);
        if (input === '한글') await typeHangul(page);
        else await page.keyboard.type(input);
        await settle(page);
        const peer = chosen === '가' ? '나' : '가';
        for (const [format, fields] of Object.entries(await savedValues(page))) {
          assert(fields[chosen] === input && fields[peer] === '',
            `${area} ${format} 선택 ${chosen}, 입력 ${input}: ${JSON.stringify(fields)}`);
        }
      }
    }
  }

  // 한 누름틀에 입력한 다음 다시 F11을 눌러 이웃을 선택하고, 문단 끝에서 필드를 빠져나간다.
  await buildSamePlace(page);
  await f11(page);
  await page.keyboard.type('xy');
  await f11(page);
  assert(await picked(page) === '가', '입력 후 F11로 이전 누름틀 선택');
  await page.keyboard.type('ab');
  await page.keyboard.press('End');
  await page.keyboard.type('!');
  await settle(page);
  for (const [format, fields] of Object.entries(await savedValues(page))) {
    assert(fields['나'] === 'xy' && fields['가'] === 'ab', `${format} 선택 이동과 이탈: ${JSON.stringify(fields)}`);
  }
  assert((await getParaText(page, 0, 0)).endsWith('뒤끝!'), 'End 뒤 입력은 누름틀 밖의 문단 끝에 남는다');

  // 첫 입력이 예약한 화면 갱신보다 두 번째 입력 이벤트가 먼저 도착하는 순서를 재현한다.
  await buildSamePlace(page);
  await f11(page);
  await f11(page);
  await page.evaluate(() => {
    const input = window.__inputHandler.textarea;
    for (const text of ['x', 'y']) {
      input.value = text;
      input.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: text }));
    }
  });
  for (const [format, fields] of Object.entries(await savedValues(page))) {
    assert(fields['가'] === 'xy' && fields['나'] === '', `${format} 화면 갱신 전 연속 입력: ${JSON.stringify(fields)}`);
  }

  // 본문 누름틀 '본문' 뒤 1x1 표, 그 셀의 "셀 끝" 글자 자리 2에 누름틀 '셀'.
  await createNewDocument(page);
  await page.evaluate(() => {
    const w = window.__wasm;
    const ih = window.__inputHandler;
    w.doc.insertText(0, 0, 0, '앞');
    JSON.parse(w.doc.insertClickHereField(0, 0, 0, '본문안내', '', '본문', true));
    const t = w.createTable(0, 0, 1, 1, 1);
    if (!t?.ok) throw new Error(`createTable: ${JSON.stringify(t)}`);
    w.doc.insertTextInCell(0, t.paraIdx, t.controlIdx, 0, 0, 0, '셀 끝');
    const r = JSON.parse(w.doc.insertClickHereFieldInCell(0, t.paraIdx, t.controlIdx, 0, 0, 2, false, '셀안내', '', '셀', true));
    if (!r.ok) throw new Error(`insertClickHereFieldInCell: ${JSON.stringify(r)}`);
    window.__canvasView?.loadDocument?.();
    ih.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 3,
      parentParaIndex: t.paraIdx, controlIndex: t.controlIdx, cellIndex: 0, cellParaIndex: 0,
    });
    ih.updateCaret();
    ih.focus();
  });
  await f11(page);
  assert(await picked(page) === '셀', '셀 안 F11: 셀 누름틀');
  await f11(page);
  assert(await picked(page) === 'table', '셀 안 F11 두 번: 그 표');
  await f11(page);
  assert(await picked(page) === '본문', '셀 안 F11 세 번: 표 앞 본문 누름틀');
});
