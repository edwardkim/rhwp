import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// ApplyCharFormatCommand 의 중첩 셀 좌표 축 가드.
//
// flat controlIndex/cellIndex 는 hit-test 가 cellPath[0](최외곽)에서 채운다. 중첩 표 셀
// 선택에 이를 그대로 applyCharFormatInCell/getCellCharPropertiesAt/setCharShapeIdInCell(flat)에
// 넘기면 **바깥 셀**에 서식을 적용/복원한다. execute·undo 모두 최내곽 셀 대상 ...ByPath 로
// 라우팅하고, 셀 문단 인덱스는 cellParaIndexOf(=cellPath[last])에서 읽어야 한다.
//
// 기준 선례: cursor.ts:399 의 useCellPath 분기. 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

function classBlock(name: string): string {
  const start = commandSrc.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = commandSrc.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? commandSrc.slice(start) : commandSrc.slice(start, start + 1 + rel);
}

test('ApplyCharFormatCommand 셀 서식 적용/복원이 최내곽 셀 대상 ...ByPath 로 라우팅한다', () => {
  const block = classBlock('ApplyCharFormatCommand');
  assert.match(block, /applyCharFormatInCellByPath\(/, 'execute 는 applyCharFormatInCellByPath 로 최내곽 셀 적용');
  assert.match(block, /getCellCharPropertiesAtByPath\(/, 'before/after charShapeId 도 ...ByPath 로 조회');
  assert.match(block, /setCharShapeIdInCellByPath\(/, 'undo(restoreCharShapeIds)도 ...ByPath 로 복원');
});

test('ApplyCharFormatCommand 가 flat 셀 축 API/좌표를 쓰지 않는다', () => {
  const block = classBlock('ApplyCharFormatCommand');
  assert.doesNotMatch(block, /wasm\.applyCharFormatInCell\(/, 'flat applyCharFormatInCell 은 바깥 셀에 적용된다');
  assert.doesNotMatch(block, /wasm\.getCellCharPropertiesAt\(/, 'flat getCellCharPropertiesAt 금지');
  assert.doesNotMatch(block, /wasm\.setCharShapeIdInCell\(/, 'flat setCharShapeIdInCell 은 undo 를 바깥 셀에 복원한다');
  assert.doesNotMatch(block, /wasm\.getCellParagraphLength\(/, 'flat getCellParagraphLength 금지');
  assert.doesNotMatch(block, /start\.cellParaIndex!/, '중첩 셀에서 start.cellParaIndex 는 바깥 셀 값 (cellParaIndexOf 사용)');
  assert.doesNotMatch(block, /end\.cellParaIndex!/, '중첩 셀에서 end.cellParaIndex 는 바깥 셀 값 (cellParaIndexOf 사용)');
});
