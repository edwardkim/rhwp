import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #3207] 서브모드 진입형 삽입·머리말/꼬리말 구조 조작의 히스토리 라우팅 소스 가드.
//
// 각주/미주/수식 삽입과 HF 생성/삭제/마당/감추기는 모두 문서 구조를 바꾸는데 미기록이라
// undo 불가 + 후속 undo 오프셋 오염이었다. 삽입류는 실행 후 노트 편집 서브모드·모달로
// 진입하지만, undo 시 서브모드 이탈에 별도 배선이 필요 없다 — SnapshotCommand 가
// editContext() 를 노출하지 않아 restoreEditContextAfterHistory 의 본문 분기를 타고
// 그 분기가 HF/FN 모드를 빠져나오기 때문이다. 이 전제까지 함께 핀한다.
// 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const insertSrc = src('src/command/commands/insert.ts');
const pageSrc = src('src/command/commands/page.ts');
const cmdSrc = src('src/engine/command.ts');
const ihSrc = src('src/engine/input-handler.ts');

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('설계 전제: SnapshotCommand 는 editContext 를 노출하지 않아 undo 가 서브모드를 이탈한다', () => {
  // SnapshotCommand 에 editContext 가 생기면 삽입 undo 가 서브모드에 남아 동작이 바뀐다.
  const snapshotCmd = slice(cmdSrc, 'export class SnapshotCommand', '\nexport ');
  assert.doesNotMatch(snapshotCmd, /editContext/,
    'SnapshotCommand 에 editContext 가 생기면 삽입 undo 의 서브모드 이탈 전제가 깨진다');
  // 본문 분기가 HF/FN 모드를 실제로 빠져나오는지.
  const restore = slice(ihSrc, 'private restoreEditContextAfterHistory', 'private resetDerivedStateAfterHistoryJump');
  assert.match(restore, /exitHeaderFooterMode\(\)/, '본문 분기가 HF 모드 이탈');
  assert.match(restore, /exitFootnoteMode\(\)/, '본문 분기가 각주 모드 이탈');
});

test('각주/미주/수식 삽입은 snapshot 으로 라우팅된다', () => {
  const note = slice(insertSrc, 'function insertNote', 'export const insertCommands');
  assert.match(note, /kind:\s*'snapshot'/, '노트 삽입 snapshot 라우팅');
  assert.match(note, /'insertFootnote'\s*:\s*'insertEndnote'/, '각주/미주 operationType 분기');
  assert.match(note, /if \(!result\.ok\) throw/, '실패 시 throw(no-op 엔트리 방지)');
  assert.match(note, /if \(result\) enterNoteEditing\(/, '기록 성공 시에만 노트 모드 진입');

  const eq = slice(insertSrc, "id: 'insert:equation'", "id: 'insert:field'");
  assert.match(eq, /operationType:\s*'insertEquation'/, '수식 삽입 snapshot 라우팅');

  // 미라우팅 흔적 금지.
  for (const m of ['insertFootnote', 'insertEndnote', 'insertEquation']) {
    assert.doesNotMatch(insertSrc, new RegExp(`services\\.wasm\\.${m}\\s*\\(`),
      `${m} 직접 호출 금지 — executeOperation 경유`);
  }
});

test('HF 구조 조작 3종이 snapshot 으로 라우팅된다', () => {
  for (const op of ['createHeaderFooter', 'applyHfTemplate', 'deleteHeaderFooter']) {
    assert.match(pageSrc, new RegExp(`operationType:\\s*'${op}'`), `${op} snapshot 라우팅`);
    assert.doesNotMatch(pageSrc, new RegExp(`services\\.wasm\\.${op}\\s*\\(`),
      `${op} 직접 호출 금지 — executeOperation 경유`);
  }
  // [보정 #3208] applyHfTemplate 은 삽입류처럼 ok:false 에 throw 해 before==after 무변
  // 스냅샷 엔트리(no-op undo)를 막는다. bridge 가 {ok} 를 반환하므로 검사 가능하다.
  const applyTpl = slice(pageSrc, "operationType: 'applyHfTemplate'", 'return bodyPos');
  assert.match(applyTpl, /if \(!r\.ok\) throw/, 'applyHfTemplate 실패 시 throw — 무변 스냅샷 엔트리 방지');
});

test('감추기 2종은 세션 상태라 의도적으로 기록하지 않는다', () => {
  // toggle_hide_header_footer_native 는 세션 집합(hidden_header_footer)과 렌더 캐시만 바꾸고
  // document 를 건드리지 않는다(레포 가드가 Exempt::SessionState/직렬화 비대상으로 분류).
  // 스냅샷은 document 만 담고 복원도 세션 집합을 되돌리지 않으므로, 라우팅하면 되돌아가는 것
  // 없이 redo 스택이 버려지고 스냅샷 예산만 소모돼 진짜 undo 이력이 축출된다.
  assert.doesNotMatch(pageSrc, /operationType:\s*'toggleHideHeaderFooter'/,
    '감추기를 snapshot 으로 라우팅하면 안 된다 — 세션 상태라 되돌아가지 않으면서 redo/예산만 파괴');
  assert.doesNotMatch(pageSrc, /operationType:\s*'hideCurrentPageHeaderFooter'/,
    '현재 쪽 감추기도 같은 이유로 라우팅 금지');
  // 다만 dirty 마킹 경로는 유지돼야 한다(afterEdit / document-changed).
  const hideCurrent = slice(pageSrc, "id: 'page:hide-current'", '\n  {');
  assert.match(hideCurrent, /emit\('document-changed'\)/, 'hide-current 의 dirty 마킹 emit 유지');
});
