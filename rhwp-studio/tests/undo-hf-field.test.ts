import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #3212] 머리말/꼬리말 필드 삽입의 히스토리 기록 소스 가드.
//
// 필드 삽입은 HF 모드 '내부' 편집이라 snapshot 으로 기록하면 undo 가
// restoreEditContextAfterHistory 의 본문 분기를 타 HF 밖으로 튕겨나간다(#3207 이 삽입류에서
// 활용한 바로 그 성질이 여기서는 오답이 된다). 그래서 editContext 를 노출하는 역연산 명령을
// kind:'record' 로 기록해 undo/redo 가 HF 모드와 오프셋을 유지하게 한다.
// 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const cmdSrc = src('src/engine/command.ts');
const pageSrc = src('src/command/commands/page.ts');

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('InsertFieldInHeaderFooterCommand 는 마커 삭제로 역연산하고 HF editContext 를 노출한다', () => {
  const block = slice(cmdSrc, 'export class InsertFieldInHeaderFooterCommand', '\nexport class DeleteTextInHeaderFooterCommand');
  assert.match(block, /execute\(wasm[\s\S]*?wasm\.insertFieldInHf\(/, 'execute 는 필드 삽입');
  assert.match(block, /undo\(wasm[\s\S]*?wasm\.deleteTextInHeaderFooter\([\s\S]*?markerLength\)/,
    'undo 는 마커 길이만큼 삭제(역연산)');
  // HF 모드 유지의 근거 — editContext 노출 + 오프셋 갱신.
  assert.match(block, /editContext\(\): EditContext \{ return this\.lastContext; \}/, 'editContext 노출');
  assert.match(block, /hfEditContext\(this\.target, this\.paraIdx, this\.charOffset \+ this\.markerLength\)/,
    'redo 후 커서는 마커 뒤');
  assert.match(block, /hfEditContext\(this\.target, this\.paraIdx, this\.charOffset\)/, 'undo 후 커서는 삽입 지점');
});

test('insertHfField 는 모델 offset에서 마커 길이를 실측해 record 로 기록한다', () => {
  const block = slice(pageSrc, 'function insertHfField', 'function navigateHeaderFooter');
  assert.match(block, /kind:\s*'record'/, "record 경로(뮤테이션 선적용 후 기록 — #2337 HF 커맨드 동형)");
  assert.match(block, /new InsertFieldInHeaderFooterCommand\(/, '역연산 명령으로 기록');
  // 파일명/두 자리 쪽번호처럼 marker가 표시 문자열로 확장돼도 mutation은 모델 char
  // index에서 해야 한다. 이 정규화가 없으면 undo가 문단 끝 밖 offset을 삭제해 no-op이 된다.
  assert.match(block, /getHeaderFooterParaInfo\(/, '삽입 전 모델 문단 길이 조회');
  assert.match(block, /Math\.min\(renderedCharOffset, paraInfo\.charCount\)/,
    '렌더 offset을 모델 범위로 정규화');
  // 하드코딩된 길이가 아니라 삽입 결과에서 실측해야 필드 종류가 늘어도 어긋나지 않는다.
  assert.match(block, /const markerLength = result\.charOffset - charOffset/,
    '마커 길이는 모델 오프셋 기준 결과 차이로 실측');
  assert.match(block, /if \(markerLength <= 0\)/, '비정상 삽입 결과는 history에 기록하지 않음');
  // 성공했을 때만 기록(no-op 엔트리 방지).
  assert.match(block, /if \(result\.ok && result\.charOffset !== undefined\)/, '성공 시에만 기록');
});
