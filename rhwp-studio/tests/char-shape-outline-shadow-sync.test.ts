import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// CharShapeDialog.populateFromProps() 의 외곽선/그림자 토글 버튼 동기화 가드.
//
// collectMods()는 attrBtns['outline']/['shadow']의 현재 활성 상태를 initialProps의
// outlineType/shadowType(>0)과 비교해 변경분을 만든다. populateFromProps()가 문서 값으로
// 이 버튼들을 먼저 채우지 않으면 항상 꺼짐 상태로 시작해, 다른 값만 바꿔도 기존 외곽선/
// 그림자 서식이 조용히 꺼진다. #2908(table-cell-props-dialog)/#2915(para-shape-dialog)와
// 동일한 populate 미동기화 버그 패턴. 이슈 #2928.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/char-shape-dialog.ts'), 'utf8');

function methodBlock(name: string): string {
  const start = src.indexOf(`private ${name}(`);
  assert.notEqual(start, -1, `${name} 메서드 not found`);
  const rel = src.slice(start + 1).indexOf('\n  private ');
  return rel === -1 ? src.slice(start) : src.slice(start, start + 1 + rel);
}

test('populateFromProps 가 outline/shadow 토글 버튼을 문서 값으로 동기화한다', () => {
  const block = methodBlock('populateFromProps');
  assert.match(
    block,
    /setAttrBtn\('outline',\s*\(p\.outlineType \|\| 0\) > 0\)/,
    'outline 버튼은 p.outlineType > 0 으로 초기화되어야 한다',
  );
  assert.match(
    block,
    /setAttrBtn\('shadow',\s*\(p\.shadowType \|\| 0\) > 0\)/,
    'shadow 버튼은 p.shadowType > 0 으로 초기화되어야 한다',
  );
});
