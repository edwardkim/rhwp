import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2880] 표 객체 선택(isInTableObjectSelection) Ctrl+C/Ctrl+X 핸들러가
// 그림 개체 선택 핸들러(pictureCellPathJson 사용부)와 달리 cellPathJson 을
// wasm.copyControl/exportControlHtml 에 전달하지 않아, 셀 안 중첩 표를
// 복사·잘라내기하면 native 가 본문 표로 오인하는 회귀를 막는 source-guard.
//
// 이 파일은 무거운 WasmBridge/DOM mock 없이, "표 selection 블록의
// copyControl(...) 호출이 반드시 cellPathJson 인자를 포함한다"는 사실만
// 소스 텍스트 수준에서 검증한다 (getSelectedTableRef().cellPath 를 실제로
// native 호출에 흘려보내는지 확인).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('표 객체 선택 Ctrl+C/Ctrl+X 는 copyControl/exportControlHtml 에 cellPathJson 을 전달한다', () => {
  const src = source('src/engine/input-handler-keyboard.ts');

  const tableSelectionBlockStart = src.indexOf('this.cursor.isInTableObjectSelection()');
  assert.notEqual(tableSelectionBlockStart, -1, '표 객체 선택 모드 블록을 찾지 못함');

  const ctrlCStart = src.indexOf("e.key === 'c'", tableSelectionBlockStart);
  const ctrlXStart = src.indexOf("e.key === 'x'", tableSelectionBlockStart);
  assert.notEqual(ctrlCStart, -1, '표 Ctrl+C 핸들러를 찾지 못함');
  assert.notEqual(ctrlXStart, -1, '표 Ctrl+X 핸들러를 찾지 못함');

  const ctrlCBlock = src.slice(ctrlCStart, ctrlXStart);
  const ctrlVStart = src.indexOf("e.key === 'v'", ctrlXStart);
  const ctrlXBlock = src.slice(ctrlXStart, ctrlVStart === -1 ? undefined : ctrlVStart);

  for (const [label, block] of [['Ctrl+C', ctrlCBlock], ['Ctrl+X', ctrlXBlock]] as const) {
    assert.match(
      block,
      /const cellPathJson = pictureCellPathJson\(ref\);/,
      `표 ${label} 핸들러가 ref.cellPath 를 cellPathJson 으로 계산하지 않음 (회귀)`,
    );
    assert.match(
      block,
      /wasm\.copyControl\(ref\.sec, ref\.ppi, ref\.ci, cellPathJson\)/,
      `표 ${label} 핸들러의 copyControl 호출에 cellPathJson 인자가 없음 (회귀)`,
    );
    assert.match(
      block,
      /wasm\.exportControlHtml\(ref\.sec, ref\.ppi, ref\.ci, cellPathJson\)/,
      `표 ${label} 핸들러의 exportControlHtml 호출에 cellPathJson 인자가 없음 (회귀)`,
    );
  }
});
