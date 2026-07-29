import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2367] 계산식 쉼표 옵션 겹침 회귀 소스 가드.
//
// evaluateTableFormula(write_result=true) 가 원시 결과를 셀에 먼저 기록하므로, 쉼표 포맷
// 문자열을 지우지 않고 offset 0 에 삽입하면 두 값이 겹쳐 남는다("6,912" + "6912" →
// "6,9126912"). #2344 계열과 동형으로 delete 후 insert 해야 한다. 행위 증명(셀 텍스트
// 회귀 게이트)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const dialogSrc = readFileSync(join(rootDir, 'src/ui/formula-dialog.ts'), 'utf8');

test('쉼표 포맷은 기존 셀 텍스트를 지운 뒤 삽입한다', () => {
  assert.match(dialogSrc, /\bthis\.wasm\.deleteTextInCell\s*\(/,
    'delete 없이 insert 하면 원시 결과 위에 겹쳐 기록된다');
  assert.match(dialogSrc, /\bthis\.wasm\.getCellParagraphLength\s*\(/,
    '지울 길이는 getCellParagraphLength 로 산출해야 함');

  // 순서 보장: delete 가 insert 보다 앞서야 겹침이 발생하지 않는다.
  const deleteAt = dialogSrc.search(/\bthis\.wasm\.deleteTextInCell\s*\(/);
  const insertAt = dialogSrc.search(/\bthis\.wasm\.insertTextInCell\s*\(/);
  assert.ok(deleteAt >= 0 && insertAt >= 0, 'delete/insert 호출이 모두 존재해야 함');
  assert.ok(deleteAt < insertAt, 'deleteTextInCell 이 insertTextInCell 보다 먼저 호출돼야 함');
});

test('쉼표 기록은 commit() snapshot 안에서 원자화된다', () => {
  // 라우팅이 사라지면 셀 문자 수 변화가 미기록되어 후속 undo 오프셋이 오염된다(#2344).
  assert.match(dialogSrc, /operationType:\s*'tableFormula'/,
    'tableFormula snapshot 라우팅 유지');

  // delete/insert 는 commit 클로저 안에 있어야 한다 — executeOperation 호출보다 앞선 위치.
  const commitAt = dialogSrc.search(/const commit\s*=\s*\(\)\s*=>/);
  const execAt = dialogSrc.search(/executeOperation\s*\(/);
  const deleteAt = dialogSrc.search(/\bthis\.wasm\.deleteTextInCell\s*\(/);
  assert.ok(commitAt >= 0 && execAt >= 0, 'commit 클로저와 executeOperation 이 존재해야 함');
  assert.ok(deleteAt > commitAt && deleteAt < execAt,
    'delete 는 commit() 클로저 안에 있어야 함');
});
