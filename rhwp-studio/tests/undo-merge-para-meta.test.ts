import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2342] 문단 병합 undo 의 메타데이터 복원 배선 소스 가드.
//
// split_at 은 새 문단을 앞 문단에서 파생시키므로(Enter 분할 시맨틱) 병합의 역연산으로
// 쓰이면 사라진 문단의 문단모양/스타일/단나누기/번호시작/PARA_HEADER tail/탭확장을
// 재현하지 못한다. 병합 결과의 removedParaMeta 를 undo 분할에 되돌려야 왕복이 닫힌다.
//
// 행위 계약(7 필드 왕복)은 Rust 쪽 회귀 테스트가 고정한다:
//   text_editing   merge_paragraph_undo_restores_removed_paragraph_meta
//   header_footer  merge_paragraph_in_header_undo_restores_removed_paragraph_meta
//   footnote       merge_paragraph_in_footnote_undo_restores_removed_paragraph_meta
// 이 파일은 스튜디오 배선이 그 값을 실어 나르는지만 정적으로 고정한다 — 뮤테이션 원장
// (mutation-routing-guard)은 호출 수만 세므로 인자 유실을 잡지 못한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const commandSrc = src('src/engine/command.ts');
const textSrc = src('src/engine/input-handler-text.ts');
const keyboardSrc = src('src/engine/input-handler-keyboard.ts');

function classBody(s: string, className: string): string {
  const a = s.indexOf(`export class ${className} `);
  assert.notEqual(a, -1, `${className} not found`);
  const b = s.indexOf('\nexport class ', a + 1);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('본문 병합 커맨드는 removedParaMeta 를 캡처해 undo 분할에 되돌린다', () => {
  for (const className of ['MergeParagraphCommand', 'MergeNextParagraphCommand']) {
    const body = classBody(commandSrc, className);
    assert.match(body, /this\.removedParaMeta = JSON\.parse\(wasm\.mergeParagraph\(/, `${className} 캡처`);
    assert.match(body, /wasm\.splitParagraph\([^)]*this\.removedParaMeta\)/, `${className} 복원`);
  }
});

test('HF/FN 병합 커맨드는 인라인 결과의 removedParaMeta 를 받아 undo 분할에 되돌린다', () => {
  const hf = classBody(commandSrc, 'MergeParagraphInHeaderFooterCommand');
  assert.match(hf, /private removedParaMeta\?: RemovedParaMeta/, 'HF 생성자 인자');
  assert.match(hf, /wasm\.splitParagraphInHeaderFooter\([^)]*this\.removedParaMeta\)/, 'HF 복원');

  const fn = classBody(commandSrc, 'MergeParagraphInFootnoteCommand');
  assert.match(fn, /private removedParaMeta\?: RemovedParaMeta/, 'FN 생성자 인자');
  assert.match(fn, /wasm\.splitParagraphInFootnote\([^)]*this\.removedParaMeta\)/, 'FN 복원');
});

test('HF/FN 인라인 편집부는 병합 결과의 removedParaMeta 를 커맨드에 넘긴다', () => {
  // 최초 적용은 kind:'record' 라 execute() 가 돌지 않는다 — 인라인 결과를 넘기지 않으면
  // 첫 undo 에서 메타가 유실된다.
  const hfSites = textSrc.match(/new MergeParagraphInHeaderFooterCommand\([^)]*\)/g) ?? [];
  assert.equal(hfSites.length, 2, 'HF 병합 커맨드 생성 지점은 Backspace/Delete 두 곳');
  for (const site of hfSites) {
    assert.match(site, /result\.removedParaMeta\)$/, `HF 인라인 전달: ${site}`);
  }

  const fnSites = keyboardSrc.match(/new MergeParagraphInFootnoteCommand\([^)]*\)/g) ?? [];
  assert.equal(fnSites.length, 1, 'FN 병합 커맨드 생성 지점');
  for (const site of fnSites) {
    assert.match(site, /result\.removedParaMeta\)$/, `FN 인라인 전달: ${site}`);
  }
});
