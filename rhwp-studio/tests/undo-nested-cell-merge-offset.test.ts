import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 중첩 표 셀 문단 커맨드의 "축 일치" 가드 — 두 축 모두.
//
// (축 1) 인덱스 축: hit-test 는 flat 필드(controlIndex/cellIndex/cellParaIndex)를
//   cellPath[0], 즉 **최외곽** 엔트리에서 채운다(cursor_rect.rs 의 `outer = &ctx.path[0]`).
//   따라서 중첩 셀에서 pos.cellParaIndex 는 바깥 셀의 문단 인덱스이고, 안쪽 셀의 값은
//   cellPath[last].cellParaIndex 다. 셀 문단 커맨드가 ...ByPath API 를 부르면서 인덱스만
//   flat 에서 가져오면, 올바른 셀의 **엉뚱한 문단**을 병합/분할한다.
//
// (축 2) 병합 위치는 같은 뮤테이션에서 반환한 논리 커서 위치다. 수식은 한 자리를 차지한다.
//
//   중첩 표 안쪽 셀의 2번째 문단에서 Enter → Ctrl+Z
//   기대: 방금 분할한 문단이 다시 합쳐짐
//   실제(축 1 어긋남): 바깥 축 인덱스로 계산돼 무관한 두 문단이 합쳐짐
//
// 기준 선례: cursor.ts:399, input-handler-text.ts:307 의 useCellPath 분기.
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

const CELL_PARA_COMMANDS = [
  'SplitParagraphInCellCommand',
  'MergeParagraphInCellCommand',
  'MergeNextParagraphInCellCommand',
];

test('셀 문단 인덱스는 단일 헬퍼로 cellPath 축에서 읽는다', () => {
  assert.match(commandSrc, /function cellParaIndexOf\s*\(/,
    '인덱스 축 유도가 여러 곳에 복제되면 한쪽만 고쳐지는 회귀가 재발한다');

  // 헬퍼는 마지막(가장 안쪽) 엔트리를 봐야 한다.
  assert.match(commandSrc, /path!\[path!\.length - 1\]\.cellParaIndex/,
    'cellPath 의 마지막 엔트리가 안쪽 셀의 문단 인덱스다');
});

test('셀 문단 커맨드가 flat cellParaIndex 를 직접 쓰지 않는다', () => {
  for (const name of CELL_PARA_COMMANDS) {
    const block = classBlock(name);
    assert.doesNotMatch(block, /const cpi = pos\.cellParaIndex!/,
      `${name}: 중첩 셀에서 pos.cellParaIndex 는 바깥 셀 값이라 ...ByPath 인덱스로 쓸 수 없다`);
    assert.match(block, /const cpi = cellParaIndexOf\(pos\)/,
      `${name}: 인덱스 축을 헬퍼로 통일해야 함`);
  }
});

test('셀 문단 병합 커서와 실행 취소 분할은 같은 논리 위치를 쓴다', () => {
  const block = classBlock('MergeParagraphInCellCommand');
  assert.match(block, /mergeParagraphInCellByPath\(/);
  assert.match(block, /wasm\.mergeParagraphInCell\(/);
  assert.doesNotMatch(block, /getCellParagraphLength/);
  assert.match(block, /this\.mergePointOffset = result\.charOffset/);
  assert.match(block, /splitParagraphInCellByPath\([^;]+this\.mergePointOffset/);
});
