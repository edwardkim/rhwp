import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #3206] 머리말/꼬리말 진입의 존재 확인 소스 가드.
//
// 존재 확인은 생성 네이티브가 중복을 거부하는 조건과 같은 범위(구역·종류·applyTo)로 해야
// 한다. `navigateHeaderFooterByPage` 는 `current_page + direction` 부터 훑는 쪽 이동용이라
// 현재 쪽을 건너뛴다 — 존재 확인에 쓰면 1쪽 문서에서 어느 방향으로도 대상이 없어, 이미 있는
// 머리말을 없다고 판단하고 생성으로 넘어가 "이미 존재" 오류로 진입 자체가 실패했다.
//
// 쪽 이동 명령(page:headerfooter-next/prev)은 이 함수의 본래 용도이므로 그대로 둔다 —
// 현재 쪽까지 포함하도록 바꾸면 "다음 머리말로 이동"이 같은 쪽에서 멈춘다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const pageSrc = readFileSync(join(rootDir, 'src/command/commands/page.ts'), 'utf8');

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('머리말 진입은 구역 범위 존재 확인으로 판단하고, 쪽 이동 함수를 쓰지 않는다', () => {
  const block = slice(pageSrc, 'function ensureHeaderFooter(', 'function insertHfField(');
  assert.match(block, /wasm\.getHeaderFooter\(sectionIndex, isHeader, applyTo\)/, '생성과 같은 범위의 존재 확인');
  assert.match(block, /operationType:\s*'createHeaderFooter'/, '생성은 snapshot 라우팅 유지(#3207)');
  assert.doesNotMatch(block, /navigateHeaderFooterByPage/, '진입 경로에서 쪽 이동 함수 사용 금지');
});

test('진입 대상(구역·applyTo)은 실제 렌더되는 컨트롤에서 얻는다', () => {
  // `양 쪽` 으로 고정하면 홀수/짝수 전용 머리말이 있는 쪽에서 캐럿이 찍힌 컨트롤과 다른
  // 것을 편집하게 된다 — 입력이 화면에 안 나타나고 반대 홀짝 쪽에 들어간다.
  const block = slice(pageSrc, 'function enterHeaderFooterEditing(', 'function insertHfField(');
  assert.match(block, /getHeaderFooterEditTarget\(currentPage, isHeader\)/, '쪽 기준 대상 조회');
  assert.match(block, /enterHeaderFooterMode\(isHeader, target\.sectionIndex, target\.applyTo/, '조회한 좌표로 진입');
  assert.doesNotMatch(block, /applyTo:\s*number/, 'applyTo 를 인자로 받아 고정하지 않는다');

  // 툴바 두 커맨드는 종류만 넘긴다 — `양 쪽` 하드코딩이 되살아나면 회귀.
  assert.match(pageSrc, /enterHeaderFooterEditing\(services, true\);/, '머리말 커맨드');
  assert.match(pageSrc, /enterHeaderFooterEditing\(services, false\);/, '꼬리말 커맨드');
});

test('마당 적용 후 재진입은 존재 확인 없이 적용 좌표로 들어간다', () => {
  // apply_hf_template_native 가 기존 HF 삭제 → 재생성이라 적용 좌표의 존재는 보장된다.
  const block = slice(pageSrc, 'function applyHfTemplate(', 'export const pageCommands');
  assert.doesNotMatch(block, /navigateHeaderFooterByPage/, '마당 재진입에서 쪽 이동 함수 사용 금지');
  assert.match(block, /enterHeaderFooterMode\(isHeader, sectionIdx, applyTo/, '적용 좌표로 직접 진입');
});

test('쪽 이동 함수는 이동 명령에만 남는다', () => {
  const uses = pageSrc.match(/wasm\.navigateHeaderFooterByPage\(/g) ?? [];
  assert.equal(uses.length, 1, `쪽 이동 함수 호출은 이동 명령 1곳뿐이어야 한다 (현재 ${uses.length})`);
  const nav = slice(pageSrc, 'function navigateHeaderFooter', 'function applyHfTemplate(');
  assert.match(nav, /navigateHeaderFooterByPage\(currentPage, isHeader, direction\)/, '이동 명령은 현재 쪽 기준으로 앞뒤 이동');
});
