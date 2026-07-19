import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2374] 양식 컨트롤 값 변경 히스토리 기록 소스 가드.
//
// 체크박스/라디오/콤보/편집 필드의 setFormValue(InCell) 직접 쓰기는 미기록이라, 이후
// 스냅샷 undo 가 값 변경 이전 문서를 복원해 양식 값을 무언 파괴한다(#2337 계급). 양식
// 모드에서는 snapshot 이 게이트에서 드롭되므로 kind:'record' + 역연산(SetFormValueCommand)
// 이 유일한 기록 경로다. 4곳 배선 + 원자화(라디오 배치) + no-op 필터를 정적으로 핀한다.
// 행위 증명은 브라우저 왕복(samples/form-01.hwp, PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const ih = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');
const cmd = readFileSync(join(rootDir, 'src/engine/command.ts'), 'utf8');

/** 메서드/블록 소스를 from~to 사이에서 추출. */
function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('SetFormValueCommand 는 배치 역연산(execute=after, undo=before 역순)이다', () => {
  const block = slice(cmd, 'export class SetFormValueCommand', '// ─── 스냅샷 기반 명령');
  assert.match(block, /implements EditCommand/, 'EditCommand 구현');
  assert.match(block, /readonly type = 'setFormValue'/, 'type 식별자');
  // execute 는 afterJson 순방향, undo 는 beforeJson 역순(라디오 그룹 원자 왕복).
  assert.match(block, /execute\(wasm[\s\S]*?afterJson/, 'execute → after 적용');
  assert.match(block, /undo\(wasm[\s\S]*?length - 1[\s\S]*?beforeJson/, 'undo → before 역순 적용');
  assert.match(block, /setFormValueInCell/, '셀 내 컨트롤 locator 지원');
});

test('recordFormValueChanges 는 kind:record + no-op 필터로 기록한다', () => {
  const block = slice(ih, 'private recordFormValueChanges', 'handleFormObjectClick');
  assert.match(block, /beforeJson !== t\.afterJson/, 'no-op(before==after) 제외 — 유령 엔트리 방지');
  assert.match(block, /kind:\s*'record'/, "record 경로(양식 모드에서 snapshot 은 드롭됨)");
  assert.match(block, /new SetFormValueCommand\(/, '역연산 커맨드로 기록');
});

test('양식 값 쓰기 4곳(체크박스/라디오/콤보/편집)이 record 로 배선된다', () => {
  // 체크박스: handleFormObjectClick CheckBox 분기.
  const checkbox = slice(ih, "case 'CheckBox': {", 'break;');
  assert.match(checkbox, /recordFormValueChanges\(/, '체크박스 토글 기록');
  // 라디오: 그룹 해제+선택을 changes 배열로 모아 1회 기록(원자화).
  const radio = slice(ih, 'private handleRadioButtonClick', 'formBboxToOverlayRect');
  assert.match(radio, /const changes: FormValueTarget\[\]/, '라디오 배치 수집');
  const radioRecords = radio.match(/recordFormValueChanges\(/g) ?? [];
  assert.equal(radioRecords.length, 1, '라디오는 1회 기록(그룹 원자화)');
  // 콤보 + 편집 필드.
  const combo = slice(ih, 'private showComboBoxOverlay', 'private showEditOverlay');
  assert.match(combo, /recordFormValueChanges\(/, '콤보 선택 기록');
  const edit = ih.slice(ih.indexOf('private showEditOverlay'));
  assert.match(edit, /recordFormValueChanges\(/, '편집 필드 커밋 기록');
  assert.match(edit, /if \(committed\) return/, '이중 커밋(Enter→blur) 가드');
});
