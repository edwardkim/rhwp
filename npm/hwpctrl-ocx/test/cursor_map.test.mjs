/**
 * 좌표 변환기 검사 — 한글 `{list,para,pos}` ↔ studio `{sectionIndex,parentParaIndex,cellPath}`.
 *
 * 실물 문서로 판정한다. 변환이 **정확히 그 셀**을 지목했는지는 그 셀의 문단 길이가 삽입한
 * 글자 수만큼 늘었는지로 확인한다 — 좌표가 한 칸이라도 어긋나면 길이가 안 변한다.
 *
 * 픽스처: 단층 표(131셀) + 중첩 표(깊이 2, 198리스트). 중첩이 있어야 `cellParaIndex` 규칙이
 * 걸린다 — 그 자리를 0 으로 고정하면 단층은 통과하고 중첩만 깨진다.
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  listToStudio, studioToList, listDepth, indexLists, streamPosAtChar, caretToCursor,
} from '../src/cursor-map.mjs';
import { createHwpCtrl } from '../src/index.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '..', '..', '..');
const wasmPath = path.join(repoRoot, 'pkg', 'rhwp_bg.wasm');

const hasWasm = fs.existsSync(wasmPath);
const sample = (name) => path.join(repoRoot, 'samples', name);

async function openDoc(name) {
  const initRhwp = (await import(path.join(repoRoot, 'pkg', 'rhwp.js'))).default;
  const wasm = await import(path.join(repoRoot, 'pkg', 'rhwp.js'));
  await initRhwp({ module_or_path: fs.readFileSync(wasmPath) });
  const doc = new wasm.HwpDocument(new Uint8Array(fs.readFileSync(sample(name))));
  doc.convertToEditable?.();
  return doc;
}

const model = (doc) => JSON.parse(doc.getCursorModel());

test('본문 리스트는 셀 경로가 없다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const doc = await openDoc('table-001.hwp');
  const at = listToStudio(model(doc), 0);
  assert.deepEqual(at, { sectionIndex: 0, parentParaIndex: 0, cellPath: [] });
  assert.equal(listDepth(model(doc), 0), 0);
});

test('단층 표 셀 좌표가 그 셀을 지목한다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const doc = await openDoc('table-001.hwp');
  const m = model(doc);
  const cell = m.lists.find((l) => l.isCell);

  const at = listToStudio(m, cell.listId);
  assert.equal(at.cellPath.length, 1, '단층 표는 경로 길이 1');
  assert.equal(listDepth(m, cell.listId), 1);

  const pathJson = JSON.stringify(at.cellPath);
  const before = doc.getCellParagraphLengthByPath(at.sectionIndex, at.parentParaIndex, pathJson);
  doc.insertTextInCellByPath(at.sectionIndex, at.parentParaIndex, pathJson, 0, '표식');
  const after = doc.getCellParagraphLengthByPath(at.sectionIndex, at.parentParaIndex, pathJson);

  assert.equal(after - before, 2, `삽입한 2자만큼 늘어야 한다 (${before} → ${after})`);
});

test('중첩 표(깊이 2)도 같은 규칙으로 지목한다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const doc = await openDoc('issue1949_giant_cell_nested_tables_perf.hwp');
  const m = model(doc);
  const deepest = m.lists
    .map((l) => ({ l, d: listDepth(m, l.listId) }))
    .sort((a, b) => b.d - a.d)[0];
  assert.ok(deepest.d >= 2, `중첩 픽스처여야 한다 (깊이 ${deepest.d})`);

  const at = listToStudio(m, deepest.l.listId);
  assert.equal(at.cellPath.length, deepest.d);

  // 앞 칸의 cellParaIndex 는 **자식 표가 놓인 부모 셀 안의 문단 번호**여야 한다.
  const byId = indexLists(m);
  assert.equal(at.cellPath[0].cellParaIndex, byId.get(deepest.l.listId).hostPara);

  const pathJson = JSON.stringify(at.cellPath);
  const before = doc.getCellParagraphLengthByPath(at.sectionIndex, at.parentParaIndex, pathJson);
  doc.insertTextInCellByPath(at.sectionIndex, at.parentParaIndex, pathJson, 0, '표식');
  const after = doc.getCellParagraphLengthByPath(at.sectionIndex, at.parentParaIndex, pathJson);
  assert.equal(after - before, 2, `중첩 셀에도 삽입돼야 한다 (${before} → ${after})`);
});

test('cellParaIndex 를 0 으로 고정하면 중첩에서 깨진다 (규칙의 존재 이유)', {
  skip: !hasWasm && 'pkg WASM 없음',
}, async () => {
  const doc = await openDoc('issue1949_giant_cell_nested_tables_perf.hwp');
  const m = model(doc);
  const deepest = m.lists
    .map((l) => ({ l, d: listDepth(m, l.listId) }))
    .sort((a, b) => b.d - a.d)[0];

  const at = listToStudio(m, deepest.l.listId);
  const naive = at.cellPath.map((entry) => ({ ...entry, cellParaIndex: 0 }));

  assert.throws(
    () => doc.getCellParagraphLengthByPath(at.sectionIndex, at.parentParaIndex, JSON.stringify(naive)),
    '0 으로 고정한 경로는 그 셀을 못 찾아야 한다',
  );
});

test('역방향 변환이 원래 리스트로 돌아온다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const doc = await openDoc('table-001.hwp');
  const m = model(doc);
  for (const cell of m.lists.filter((l) => l.isCell).slice(0, 12)) {
    const at = listToStudio(m, cell.listId);
    assert.equal(studioToList(m, at), cell.listId, `list ${cell.listId} 왕복`);
  }
  assert.equal(studioToList(m, { sectionIndex: 0, parentParaIndex: 0, cellPath: [] }), 0);
});

// ── studio 캐럿 → 한글 커서 (syncCursorFromCaret 의 변환) ─────────────────

test('streamPosAtChar 는 그 글자로 모이는 가장 큰 자리를 고른다', () => {
  // 글자 2 앞에 누름틀 시작 코드(8칸)가 있는 문단: 자리 2(코드 앞)와 10(안내문 시작)이 글자 2로 모인다.
  const stops = [0, 1, 2, 10, 11, 20];
  const charOf = new Map([[0, 0], [1, 1], [2, 2], [10, 2], [11, 3], [20, 4]]);
  const charIndexAt = (pos) => charOf.get(pos);
  assert.equal(streamPosAtChar(stops, charIndexAt, 0), 0);
  assert.equal(streamPosAtChar(stops, charIndexAt, 1), 1);
  assert.equal(streamPosAtChar(stops, charIndexAt, 2), 10, '필드 안쪽 자리');
  assert.equal(streamPosAtChar(stops, charIndexAt, 3), 11);
  assert.equal(streamPosAtChar(stops, charIndexAt, 99), 20, '문단 끝을 넘으면 마지막 자리');
  assert.equal(streamPosAtChar([], charIndexAt, 3), 0);
});

test('여러 구역 본문의 누름틀 자리로 커서를 옮기면 그 누름틀 이름을 답한다', {
  skip: !hasWasm && 'pkg WASM 없음',
}, async () => {
  const doc = await openDoc('field-01.hwp');
  assert.ok(doc.getSectionCount() > 1, '다구역 픽스처여야 한다');
  const ctrl = createHwpCtrl({ doc });
  const bodyFields = JSON.parse(doc.getFieldList())
    .filter((f) => f.fieldType === 'clickhere' && !f.cellField && !(f.location.path || []).length);
  assert.ok(bodyFields.some((f) => f.location.sectionIndex > 0), '둘째 구역 이후의 누름틀도 검사한다');

  for (const f of bodyFields) {
    const caret = {
      sectionIndex: f.location.sectionIndex,
      parentParaIndex: f.location.paraIndex,
      cellPath: [],
      charOffset: f.startCharIdx,
    };
    const inside = caretToCursor(doc, { ...caret, field: { id: f.fieldId, at: 'inside' } });
    let rootPara = f.location.paraIndex;
    for (let s = 0; s < f.location.sectionIndex; s += 1) rootPara += doc.getParagraphCount(s);
    assert.deepEqual([inside.list, inside.para], [0, rootPara], `${f.name}: 본문 리스트·구역 누적 문단`);
    const back = JSON.parse(doc.getCharIndexAtStreamPos(inside.list, inside.para, inside.pos)).charIndex;
    assert.equal(back, f.startCharIdx, `${f.name}: 글자 번호 왕복`);
    const after = caretToCursor(doc, { ...caret, charOffset: f.endCharIdx, field: { id: f.fieldId, at: 'after' } });
    assert.ok(after.pos > inside.pos, `${f.name}: 끝 코드 뒤 자리가 안쪽보다 뒤 (${inside.pos} < ${after.pos})`);

    // `GetCurFieldName` 은 `getFieldList` 의 구역 안 문단 번호로 비교하므로 첫 구역에서만 대조한다.
    if (f.location.sectionIndex !== 0) continue;
    ctrl.SetPos(inside.list, inside.para, inside.pos);
    assert.equal(ctrl.GetCurFieldName(0), f.name, `${f.name}: 안쪽 캐럿은 그 누름틀 안`);
    // 같은 글자 번호라도 화면 캐럿이 끝 코드 뒤로 나갔으면 누름틀 밖이다.
    ctrl.SetPos(after.list, after.para, after.pos);
    assert.notEqual(ctrl.GetCurFieldName(0), f.name, `${f.name}: 끝 코드 뒤는 밖`);
  }
});

test('누름틀 시작 코드 앞으로 나간 캐럿은 시작 코드 앞 자리다', {
  skip: !hasWasm && 'pkg WASM 없음',
}, async () => {
  const doc = await openDoc('field-01.hwp');
  doc.setFieldValueByName('회사명', '가나다');
  const f = JSON.parse(doc.getFieldList()).find((x) => x.name === '회사명');
  assert.ok(f && f.endCharIdx - f.startCharIdx === 3, '내용이 있는 본문 누름틀');
  const caret = {
    sectionIndex: f.location.sectionIndex, parentParaIndex: f.location.paraIndex,
    cellPath: [], charOffset: f.startCharIdx,
  };
  const before = caretToCursor(doc, { ...caret, field: { id: f.fieldId, at: 'before' } });
  const inside = caretToCursor(doc, { ...caret, field: { id: f.fieldId, at: 'inside' } });
  assert.equal(before.pos, f.startPos - 8, '시작 코드(8칸) 앞');
  assert.equal(inside.pos, f.startPos, '내용 시작');

  const ctrl = createHwpCtrl({ doc });
  ctrl.SetPos(inside.list, inside.para, inside.pos);
  assert.equal(ctrl.GetCurFieldName(0), '회사명', '내용 시작은 안');
  const end = caretToCursor(doc, { ...caret, charOffset: f.endCharIdx, field: { id: f.fieldId, at: 'inside' } });
  assert.equal(end.pos, f.endPos, '안쪽 끝은 끝 코드 앞');
  ctrl.SetPos(end.list, end.para, end.pos);
  assert.equal(ctrl.GetCurFieldName(0), '회사명', '안쪽 끝도 안');
});

test('표 칸 캐럿은 그 칸의 리스트로 옮긴다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const doc = await openDoc('table-001.hwp');
  const m = model(doc);
  for (const cell of m.lists.filter((l) => l.isCell).slice(0, 12)) {
    const studio = listToStudio(m, cell.listId);
    const at = caretToCursor(doc, { ...studio, charOffset: 0 });
    assert.equal(at.list, cell.listId, `list ${cell.listId}`);
    assert.equal(at.para, 0);
    assert.equal(JSON.parse(doc.getCharIndexAtStreamPos(at.list, at.para, at.pos)).charIndex, 0);
  }
  assert.equal(caretToCursor(doc, {
    sectionIndex: 0, parentParaIndex: 0,
    cellPath: [{ controlIndex: 99, cellIndex: 0, cellParaIndex: 0 }], charOffset: 0,
  }), null, '없는 표 칸은 null');
});
