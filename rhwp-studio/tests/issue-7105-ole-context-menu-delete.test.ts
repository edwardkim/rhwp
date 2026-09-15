import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { balancedFrom, functionBodyFrom } from './support/source-guard.ts';

// [#7105] 오른쪽 클릭 메뉴 "지우기(D)"(`insert:picture-delete`)가 OLE 개체를 그림 삭제 명령으로 보냈다.
//
// 선택된 개체의 종류가 `ole` 인데 이 명령의 분기는 `shape`·`line`·`group` 만 도형 삭제로 보내고
// 나머지를 전부 `deletePictureControl` 로 떨어뜨렸다. 코어의 그림 삭제는 `Control::Picture` 만
// 받으므로 OLE 는 "지정된 컨트롤이 그림이 아닙니다" 로 거부된다 — 신고 문서(`실험4 Transistor-MOSFET.hwp`)
// 의 OLE 그림 12개가 **선택 핸들은 뜨는데 메뉴로 지워지지 않았다**.
//
// 같은 개체를 키보드 Delete(`performDelete` → `deleteObjectControl`)나 잘라내기로 지우면
// `ole` 이 도형 삭제로 가서 성공한다. 신고자가 "잘라내기는 되는데 삭제만 안 된다" 고 한 것이 이 차이다.
// 두 경로가 같은 종류 집합을 도형 삭제로 보내는지 함께 잠근다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const insertSrc = readFileSync(join(rootDir, 'src/command/commands/insert.ts'), 'utf8');
const pictureSrc = readFileSync(join(rootDir, 'src/engine/input-handler-picture.ts'), 'utf8');

/** `if (<조건>) { … wasm.deleteShapeControl(` 의 조건식에서 `ref.type === '<종류>'` 들을 뽑는다. */
function shapeDeleteTypes(block: string, label: string): string[] {
  const m = block.match(/if\s*\(([^{]*?)\)\s*\{\s*(?:this\.)?wasm\.deleteShapeControl\(/);
  assert.ok(m, `${label}: 도형 삭제 분기를 찾아야 한다`);
  return [...m[1].matchAll(/ref\.type\s*===\s*'(\w+)'/g)].map((t) => t[1]).sort();
}

test('메뉴 "지우기" 는 OLE 개체를 도형 삭제 명령으로 보낸다', () => {
  const start = insertSrc.indexOf("id: 'insert:picture-delete'");
  assert.ok(start >= 0, 'insert:picture-delete 명령이 있어야 한다');
  const block = balancedFrom(insertSrc.slice(start), 'execute(services)', '{');
  const types = shapeDeleteTypes(block, 'insert:picture-delete');
  assert.ok(
    types.includes('ole'),
    `#7105: OLE 는 deleteShapeControl 로 가야 한다 — 그림 삭제는 OLE 를 거부한다. 현재 도형 삭제 대상: ${types}`,
  );
});

test('메뉴 삭제와 키보드 삭제가 같은 종류를 도형 삭제로 보낸다', () => {
  const start = insertSrc.indexOf("id: 'insert:picture-delete'");
  const menu = shapeDeleteTypes(
    balancedFrom(insertSrc.slice(start), 'execute(services)', '{'),
    'insert:picture-delete',
  );
  const keyboard = shapeDeleteTypes(
    functionBodyFrom(pictureSrc, 'export function deleteObjectControl'),
    'deleteObjectControl',
  );
  assert.deepEqual(
    menu,
    keyboard,
    '#7105: 메뉴 삭제(insert:picture-delete)와 키보드 삭제(deleteObjectControl)의 도형 삭제 대상이 갈리면 한쪽에서만 지워진다',
  );
});
