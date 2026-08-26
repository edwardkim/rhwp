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

import { listToStudio, studioToList, listDepth, indexLists } from '../src/cursor-map.mjs';

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
