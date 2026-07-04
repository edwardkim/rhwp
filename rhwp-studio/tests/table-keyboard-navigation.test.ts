import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function tabCaseBlock(): string {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const start = keyboard.indexOf("case 'Tab': {");
  const end = keyboard.indexOf("case 'Insert': {", start);
  assert.notEqual(start, -1, 'Tab case not found');
  assert.notEqual(end, -1, 'Insert case after Tab not found');
  return keyboard.slice(start, end);
}

function exitTableBlock(): string {
  const cursor = source('src/engine/cursor.ts');
  const start = cursor.indexOf('private exitTable(delta: number): void {');
  const end = cursor.indexOf('// ─── 캐럿 좌표 갱신', start);
  assert.notEqual(start, -1, 'exitTable method not found');
  assert.notEqual(end, -1, 'exitTable method end not found');
  return cursor.slice(start, end);
}

test('Tab in a table cell uses cell navigation before inserting a tab character', () => {
  const tabCase = tabCaseBlock();

  assert.match(tabCase, /this\.cursor\.isInCell\(\)\s*&&\s*!this\.cursor\.isInTextBox\(\)/);
  assert.match(tabCase, /e\.shiftKey[\s\S]*this\.cursor\.moveToCellPrev\(\)[\s\S]*insertRowAfterLastTableCellByTab\.call\(this\)[\s\S]*this\.cursor\.moveToCellNext\(\)/);
  assert.match(tabCase, /this\.updateCaret\(\)/);
  assert.ok(tabCase.indexOf('moveToCellNext') < tabCase.indexOf('new InsertTabCommand'));
});

test('Tab in the last table cell inserts a row before exiting the table', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const tabCase = tabCaseBlock();
  const start = keyboard.indexOf('function insertRowAfterLastTableCellByTab');
  const end = keyboard.indexOf('type PictureDeleteRef', start);
  assert.notEqual(start, -1, 'last-cell Tab row insertion helper not found');
  assert.notEqual(end, -1, 'last-cell Tab row insertion helper end not found');
  const helper = keyboard.slice(start, end);

  assert.match(helper, /uniqueCellsInReadingOrder\(this\.wasm\.getTableCellBboxes\(sec,\s*ppi,\s*ci\)\)/);
  assert.match(helper, /order\[order\.length - 1\]\.cellIdx !== currentCellIdx/);
  assert.match(helper, /wasm\.insertTableRow\(sec,\s*ppi,\s*ci,\s*insertAfterRow,\s*true\)/);
  assert.match(helper, /insertedRow = insertAfterRow \+ 1/);
  assert.match(helper, /tableCellStartPosition\(pos,\s*nextCell\?\.cellIdx \?\? currentCellIdx\)/);
  assert.ok(
    tabCase.indexOf('insertRowAfterLastTableCellByTab.call(this)') < tabCase.indexOf('this.cursor.moveToCellNext()'),
    'last-cell row insertion must run before normal next-cell navigation',
  );
});

test('Backward table exit lands on the table paragraph start mark', () => {
  const exitTable = exitTableBlock();

  assert.match(exitTable, /paragraphIndex:\s*ppi!,\s*charOffset:\s*0/);
  assert.doesNotMatch(exitTable, /getParagraphLength\(sec,\s*ppi!\s*-\s*1\)/);
});
