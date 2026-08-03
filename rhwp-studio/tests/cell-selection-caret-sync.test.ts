import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const studioRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const workDir = mkdtempSync(path.join(tmpdir(), 'rhwp-cell-caret-'));
const driverPath = path.join(workDir, 'driver.mjs');

writeFileSync(driverPath, `
import { registerHooks } from 'node:module';
import { pathToFileURL } from 'node:url';

const srcRoot = ${JSON.stringify(pathToFileURL(path.join(studioRoot, 'src') + path.sep).href)};
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) return nextResolve(srcRoot + specifier.slice(2) + '.ts', context);
    if (/^\\.{1,2}\\//.test(specifier) && !/\\.[a-z]+$/.test(specifier)) {
      return nextResolve(specifier + '.ts', context);
    }
    return nextResolve(specifier, context);
  },
});

const { CursorState } = await import(srcRoot + 'engine/cursor.ts');
const bboxes = [
  { cellIdx: 0, row: 0, col: 0, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 0, y: 0, w: 40, h: 20 },
  { cellIdx: 1, row: 0, col: 1, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 40, y: 0, w: 40, h: 20 },
  { cellIdx: 2, row: 1, col: 0, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 0, y: 20, w: 40, h: 20 },
  { cellIdx: 3, row: 1, col: 1, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 40, y: 20, w: 40, h: 20 },
];
const rect = (cellIndex) => ({
  pageIndex: 0, x: cellIndex * 40, y: 0, height: 12,
  cellBounds: { x: cellIndex * 40, y: 0, w: 40, h: 20 }, cellOverflowed: false,
});
const info = (cellIndex) => ({ row: bboxes[cellIndex].row, col: bboxes[cellIndex].col });
const dimensions = { rowCount: 2, colCount: 2 };

function flatPosition(cellIndex) {
  return {
    sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
    parentParaIndex: 5, controlIndex: 3, cellIndex, cellParaIndex: 0,
    cursorRect: rect(cellIndex),
  };
}
const flatWasm = {
  getCellInfo(_sec, _ppi, _ci, cellIndex) { return info(cellIndex); },
  getTableDimensions() { return dimensions; },
  getTableCellBboxes() { return bboxes; },
  getCursorRectInCell(_sec, _ppi, _ci, cellIndex) { return rect(cellIndex); },
};
const flat = new CursorState(flatWasm);
flat.moveToHit(flatPosition(0));
flat.enterCellSelectionMode();
flat.moveCellSelection(0, 1);

const initialPath = [
  { controlIndex: 3, cellIndex: 0, cellParaIndex: 0 },
  { controlIndex: 7, cellIndex: 0, cellParaIndex: 0 },
];
const nestedWasm = {
  getCellInfoByPath() { return info(0); },
  getTableDimensionsByPath() { return dimensions; },
  getTableCellBboxesByPath() { return bboxes; },
  getCursorRectByPathNear(_sec, _ppi, _path, _offset, hint) { return rect(hint ?? 0); },
  getCursorRectByPath() { return rect(0); },
};
const nested = new CursorState(nestedWasm);
nested.moveToHit({
  sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
  parentParaIndex: 5, controlIndex: 3, cellIndex: 0, cellParaIndex: 0,
  cellPath: initialPath, cursorRect: rect(0),
});
nested.enterCellSelectionMode();
nested.moveCellSelection(1, 0);

process.stdout.write('###' + JSON.stringify({
  flat: { position: flat.getPosition(), range: flat.getSelectedCellRange(), rect: flat.getRect() },
  nested: { position: nested.getPosition(), range: nested.getSelectedCellRange(), rect: nested.getRect() },
}) + '###');
`);

const run = spawnSync(
  process.execPath,
  ['--experimental-transform-types', '--no-warnings', driverPath],
  { cwd: studioRoot, encoding: 'utf8' },
);
rmSync(workDir, { recursive: true, force: true });

assert.equal(run.status, 0, `F5 셀 선택 캐럿 드라이버 실패:\n${run.stdout}\n${run.stderr}`);
const captured = /###([\s\S]*)###/.exec(run.stdout);
assert.ok(captured, `F5 셀 선택 캐럿 결과가 없습니다:\n${run.stdout}`);
const result = JSON.parse(captured[1]);

assert.deepEqual(result.flat.range, { startRow: 0, startCol: 1, endRow: 0, endCol: 1 });
assert.equal(result.flat.position.cellIndex, 1);
assert.equal(result.flat.position.cellParaIndex, 0);
assert.equal(result.flat.position.charOffset, 0);
assert.equal(result.flat.rect.x, 40);

assert.deepEqual(result.nested.range, { startRow: 1, startCol: 0, endRow: 1, endCol: 0 });
assert.equal(result.nested.position.cellPath.at(-1).cellIndex, 2);
assert.equal(result.nested.position.cellPath.at(-1).cellParaIndex, 0);
assert.equal(result.nested.position.charOffset, 0);

test('F5 단일 셀 선택 이동은 평면·중첩 표의 문서 위치를 동기화한다', () => {});

test('F5 단일 셀 선택 이동은 화면 캐럿을 숨긴다', () => {
  const keyboardPath = path.join(studioRoot, 'src/engine/input-handler-keyboard.ts');
  const keyboard = readFileSync(keyboardPath, 'utf8');
  const start = keyboard.indexOf('// phase 1: 단일 셀 이동');
  const end = keyboard.indexOf('this.updateCellSelection();', start);

  assert.notEqual(start, -1, 'F5 phase 1 셀 이동 블록을 찾을 수 없습니다');
  assert.notEqual(end, -1, 'F5 phase 1 뒤의 셀 선택 렌더 갱신을 찾을 수 없습니다');

  const phaseOne = keyboard.slice(start, end);
  assert.match(phaseOne, /this\.cursor\.moveCellSelection\(dr, dc\);\s*[\s\S]*this\.caret\.hide\(\);/);
  assert.doesNotMatch(phaseOne, /this\.updateCaret\(\)/);
});

test('F5 셀 선택에서 Escape는 마지막 셀의 편집 캐럿으로 복귀한다', () => {
  const keyboardPath = path.join(studioRoot, 'src/engine/input-handler-keyboard.ts');
  const keyboard = readFileSync(keyboardPath, 'utf8');
  const modeStart = keyboard.indexOf('// ─── 셀 선택 모드 중 키 처리');
  const escapeStart = keyboard.indexOf("if (e.key === 'Escape')", modeStart);
  const escapeEnd = keyboard.indexOf('// 셀 크기 조절', escapeStart);

  assert.notEqual(modeStart, -1, 'F5 셀 선택 키 처리 블록을 찾을 수 없습니다');
  assert.notEqual(escapeStart, -1, 'F5 셀 선택 Escape 처리 블록을 찾을 수 없습니다');
  assert.notEqual(escapeEnd, -1, 'F5 셀 선택 Escape 처리 블록의 끝을 찾을 수 없습니다');

  const escapeBlock = keyboard.slice(escapeStart, escapeEnd);
  assert.match(escapeBlock, /this\.cursor\.exitCellSelectionMode\(\);/);
  assert.match(escapeBlock, /this\.cellSelectionRenderer\?\.clear\(\);/);
  assert.match(escapeBlock, /this\.updateCaret\(\);/);
  assert.doesNotMatch(escapeBlock, /enterTableObjectSelection|renderTableObjectSelection|table-object-selection-changed/);
});
