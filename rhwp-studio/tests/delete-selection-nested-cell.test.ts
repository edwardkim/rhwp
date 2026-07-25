import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// DeleteSelectionCommand 의 중첩 셀 좌표 축 가드.
//
// flat controlIndex/cellIndex 는 hit-test 가 cellPath[0](최외곽)에서 채운다. 중첩 표
// 셀에서 이를 그대로 deleteRangeInCell(flat)에 넘기면 **바깥 셀**의 선택을 삭제한다.
// 최내곽 셀을 대상으로 ...ByPath 로 라우팅하고, 셀 문단 인덱스는 cellPath[last]
// (cellParaIndexOf)에서 읽어야 한다.
//
// [#2418 이후] undo 는 문서 스냅샷 복원이라 좌표를 쓰지 않는다 — execute/undo 축이
// 어긋날 여지 자체가 없어졌고, 축 정합은 삭제 호출 한 곳에만 걸린다. 예전에는 undo 가
// 저장 텍스트를 다시 넣었으므로 캡처 축(getTextInCellByPath)도 함께 맞춰야 했다.
//
// 기준 선례: cursor.ts:399, input-handler-text.ts 의 useCellPath 분기.
// 행위 증명(중첩 표 왕복)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const commandSrc = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

/** `export class NAME ...` 부터 다음 `export class` 전까지 클래스 본문을 추출. */
function classBlock(name: string): string {
  const start = commandSrc.indexOf(`export class ${name}`);
  assert.notEqual(start, -1, `${name} 클래스 not found`);
  const rel = commandSrc.slice(start + 1).indexOf('\nexport class ');
  return rel === -1 ? commandSrc.slice(start) : commandSrc.slice(start, start + 1 + rel);
}

test('DeleteSelectionCommand 셀 삭제가 최내곽 셀 대상 ...ByPath 로 라우팅한다', () => {
  const block = classBlock('DeleteSelectionCommand');
  assert.match(block, /deleteRangeInCellByPath\(/,
    '중첩 셀 선택 삭제는 deleteRangeInCellByPath 로 최내곽 셀을 대상으로 해야 한다');
  assert.match(block, /cellParaIndexOf\(start\)/,
    '셀 문단 인덱스는 cellPath[last] 에서 읽어야 한다');
  assert.match(block, /cellParaIndexOf\(end\)/,
    '선택 끝 문단 인덱스도 cellPath[last] 기준이어야 한다');
});

test('DeleteSelectionCommand 가 flat 셀 축 API/좌표를 쓰지 않는다', () => {
  const block = classBlock('DeleteSelectionCommand');
  assert.doesNotMatch(block, /wasm\.deleteRangeInCell\(/,
    'flat deleteRangeInCell 은 cellPath[0](최외곽) 좌표라 중첩 셀에서 바깥 셀을 삭제한다');
  assert.doesNotMatch(block, /wasm\.getTextInCell\(/,
    'flat getTextInCell 대신 getTextInCellByPath 를 써야 한다');
  assert.doesNotMatch(block, /wasm\.getCellParagraphLength\(/,
    'flat getCellParagraphLength 는 외부 표 기준 레거시 좌표를 쓴다');
  assert.doesNotMatch(block, /start\.cellParaIndex!/,
    '중첩 셀에서 start.cellParaIndex 는 바깥 셀 문단 인덱스다 (cellParaIndexOf 사용)');
  assert.doesNotMatch(block, /end\.cellParaIndex!/,
    '중첩 셀에서 end.cellParaIndex 는 바깥 셀 문단 인덱스다 (cellParaIndexOf 사용)');
});
