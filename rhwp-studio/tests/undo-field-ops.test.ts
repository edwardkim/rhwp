import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2377] 누름틀 삽입/고치기/지우기 히스토리 라우팅 소스 가드.
//
// 세 op 모두 안내문 텍스트를 넣고/바꾸고/지워 문자 수를 바꾼다 — 미기록 시 undo 불가 +
// 후속 undo 오프셋 오염. 삽입/고치기는 일반 모드 전용 커맨드라 순수 snapshot, 지우기는
// 키보드 경계 삭제로 양식 모드에서도 도달하므로(양식 모드 게이트가 snapshot 을 드롭해
// 무언 폐기 회귀가 됨 — #2361 리뷰 계급) 일반 모드만 snapshot 라우팅하고 양식 모드는
// 기존 직접 경로를 유지한다. 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const insertSrc = src('src/command/commands/insert.ts');
const editSrc = src('src/command/commands/edit.ts');
const ihSrc = src('src/engine/input-handler.ts');

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('누름틀 삽입(insert:field)은 snapshot 으로 라우팅되고 실패 시 엔트리를 만들지 않는다', () => {
  const block = slice(insertSrc, "id: 'insert:field'", 'fieldInsertDialog.show()');
  assert.match(block, /operationType:\s*'insertField'/, 'insertField snapshot 라우팅');
  assert.match(block, /\bwasm\.insertClickHereField\(/, '뮤테이션은 operation 콜백 안');
  assert.doesNotMatch(block, /services\.wasm\.insertClickHereField/, '직접 호출 금지');
  assert.match(block, /if \(!result\.ok\) throw/, '실패 시 throw(no-op 엔트리 방지)');
});

test('누름틀 고치기(field:edit)는 snapshot 으로 라우팅된다', () => {
  const block = slice(editSrc, "id: 'field:edit'", "id: 'field:remove'");
  assert.match(block, /operationType:\s*'updateFieldProps'/, 'updateFieldProps snapshot 라우팅');
  assert.match(block, /\bwasm\.updateClickHereProps\(/, '뮤테이션은 operation 콜백 안');
  assert.doesNotMatch(block, /services\.wasm\.updateClickHereProps/, '직접 호출 금지');
});

test('누름틀 지우기(removeCurrentField)는 일반=snapshot / 양식=직접(비기록) 분기다', () => {
  const block = slice(ihSrc, 'removeCurrentField(posOverride', 'confirmRemoveCurrentField');
  // 양식 모드: 게이트가 snapshot 을 드롭하므로 직접 경로 유지(무언 폐기 회귀 방지).
  assert.match(block, /this\.editMode === 'form'/, '양식 모드 분기 존재');
  // 일반 모드: snapshot 라우팅.
  assert.match(block, /operationType:\s*'removeField'/, 'removeField snapshot 라우팅');
  assert.match(block, /if \(!result\.ok\) throw/, '실패 시 throw(no-op 엔트리 방지)');
});
