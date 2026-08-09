import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tableObjectClipboardTarget } from '../src/engine/table-object-clipboard-target.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const nestedPath = [
  { controlIndex: 1, cellIndex: 0, cellParaIndex: 0 },
  { controlIndex: 2, cellIndex: 0, cellParaIndex: 12 },
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
];

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('#4272 중첩 표 객체는 마지막 엔트리를 control, prefix를 owner path로 사용한다', () => {
  assert.deepEqual(
    tableObjectClipboardTarget({ ci: 1, cellPath: nestedPath }),
    {
      controlIndex: 0,
      ownerCellPathJson: JSON.stringify(nestedPath.slice(0, -1)),
    },
  );
});

test('#4272 본문 표 객체는 기존 ref.ci와 빈 owner path를 유지한다', () => {
  assert.deepEqual(
    tableObjectClipboardTarget({ ci: 3 }),
    { controlIndex: 3, ownerCellPathJson: '' },
  );
});

test('#4272 키보드와 command 복사는 같은 표 객체 주소 변환을 사용한다', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const handler = source('src/engine/input-handler.ts');

  const tableSelectionStart = keyboard.indexOf('// Ctrl+C → 표 복사');
  const ctrlCStart = keyboard.indexOf("e.key === 'c'", tableSelectionStart);
  const ctrlXStart = keyboard.indexOf("e.key === 'x'", ctrlCStart);
  assert.notEqual(ctrlCStart, -1, '표 Ctrl+C 핸들러를 찾지 못함');
  assert.notEqual(ctrlXStart, -1, '표 Ctrl+X 경계를 찾지 못함');
  const ctrlCBlock = keyboard.slice(ctrlCStart, ctrlXStart);
  assert.match(ctrlCBlock, /const target = tableObjectClipboardTarget\(ref\);/);
  assert.match(
    ctrlCBlock,
    /wasm\.copyControl\([\s\S]*?target\.controlIndex, target\.ownerCellPathJson/,
  );
  assert.match(
    ctrlCBlock,
    /wasm\.exportControlHtml\([\s\S]*?target\.controlIndex, target\.ownerCellPathJson/,
  );

  const performCopyStart = handler.indexOf('performCopy(): void');
  const performCopyEnd = handler.indexOf('\n  /**', performCopyStart + 1);
  const performCopyBlock = handler.slice(
    performCopyStart,
    performCopyEnd === -1 ? undefined : performCopyEnd,
  );
  const tableBranchStart = performCopyBlock.indexOf('this.cursor.isInTableObjectSelection()');
  const tableBranch = performCopyBlock.slice(tableBranchStart);
  assert.notEqual(tableBranchStart, -1, 'performCopy 표 객체 분기를 찾지 못함');
  assert.match(tableBranch, /const target = tableObjectClipboardTarget\(ref\);/);
  assert.match(
    tableBranch,
    /wasm\.copyControl\([\s\S]*?target\.controlIndex, target\.ownerCellPathJson/,
  );
  assert.match(
    tableBranch,
    /wasm\.exportControlHtml\([\s\S]*?target\.controlIndex, target\.ownerCellPathJson/,
  );
});
