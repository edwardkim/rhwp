import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// Issue #2037: 찾아 바꾸기/모두 바꾸기는 문서를 mutate 하므로 반드시 편집 라우터
// (executeOperation)를 통과해 undo 스택에 기록되어야 한다 (#1320 계약).
// 기존에는 wasm.replaceText/replaceAll 직접 호출 + document-changed emit 만
// 수행되어, 대량 치환이 Ctrl+Z 로 복구되지 않았다.

function methodBlock(src: string, methodName: string): string {
  const start = src.indexOf(`private ${methodName}`);
  assert.notEqual(start, -1, `${methodName} not found`);
  const next = src.indexOf('\n  private ', start + 1);
  return src.slice(start, next === -1 ? undefined : next);
}

test('바꾸기(doReplace)는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/find-dialog.ts');
  const block = methodBlock(dialog, 'doReplace');

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'replaceText'/,
    '바꾸기는 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});

test('모두 바꾸기(doReplaceAll)는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/find-dialog.ts');
  const block = methodBlock(dialog, 'doReplaceAll');

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'replaceAll'/,
    '모두 바꾸기는 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});

test('Find hit count는 추가 전체 검색 없이 탐색 결과를 사용한다', () => {
  const dialog = source('src/ui/find-dialog.ts');

  assert.match(dialog, /result\.totalMatchCount/, 'searchText 결과의 count를 UI에 표시해야 함');
  assert.doesNotMatch(dialog, /searchAllText\s*\(/, '더 넓은 searchAllText 의미를 사용하면 안 됨');
  assert.match(dialog, /find-dialog-match-count/, '일시 상태와 분리된 count 라벨이어야 함');
  assert.match(dialog, /showMatchCount\(0\)/, '검색 실패는 0건으로 표시해야 함');
});

test('Find hit count는 한국어와 영어 locale에 등록된다', () => {
  const ko = source('src/i18n/locales/ko.ts');
  const en = source('src/i18n/locales/en.ts');

  assert.match(ko, /"dialog\.find\.matchCountLabel\.text": "검색 결과 \{p1\}개"/);
  assert.match(en, /"dialog\.find\.matchCountLabel\.text": "\{p1\} matches"/);
});
