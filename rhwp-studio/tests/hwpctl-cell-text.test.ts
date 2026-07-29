import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// hwpctl 표 셀 텍스트 API(SetCellText / GetCellText) 소스 가드.
//
// 1) SetCellText 는 Set 의미다. delete 없이 offset 0 에 insert 하면 기존 셀 텍스트 앞에
//    붙어 누적된다(같은 셀 두 번 호출 시 "2010"). #2344/#2367 과 동일한 결함 형태다.
// 2) GetCellText 는 getTextInCellByPath 를 단일 문자열 path 하나로 호출했는데, 해당 WASM
//    API 는 (sec, parentPara, pathJson, charOffset, count) 5 인자다. 인자 수가 맞지 않아
//    항상 예외로 떨어져 '' 를 반환했다.
//
// 행위 증명(왕복 read-back)은 브라우저 왕복(PR 검증). 여기서는 호출 형태를 정적으로 핀한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
// CRLF 체크아웃에서도 동일하게 동작하도록 개행을 정규화한다.
const src = readFileSync(join(rootDir, 'src/hwpctl/index.ts'), 'utf8').replace(/\r\n/g, '\n');

/** 메서드 본문만 잘라낸다(다른 메서드의 호출이 섞이지 않도록). */
function methodBody(name: string): string {
  const start = src.indexOf(`  ${name}(`);
  assert.ok(start >= 0, `${name} 메서드가 존재해야 함`);
  const rest = src.slice(start + 1);
  const end = rest.indexOf('\n  }\n');
  assert.ok(end >= 0, `${name} 본문 경계를 찾아야 함`);
  return rest.slice(0, end);
}

test('SetCellText 는 기존 셀 텍스트를 지운 뒤 삽입한다', () => {
  const body = methodBody('SetCellText');

  assert.match(body, /getCellParagraphLength\s*\(/,
    '지울 길이를 getCellParagraphLength 로 구해야 함');
  assert.match(body, /deleteTextInCell\s*\(/,
    'delete 없이 insert 하면 기존 텍스트 위에 누적된다');
  assert.match(body, /insertTextInCell\s*\(/, 'insert 호출 유지');

  const deleteAt = body.search(/deleteTextInCell\s*\(/);
  const insertAt = body.search(/insertTextInCell\s*\(/);
  assert.ok(deleteAt < insertAt, 'deleteTextInCell 이 insertTextInCell 보다 먼저여야 함');
});

test('GetCellText 는 인자 수가 맞는 WASM 셀 텍스트 API 를 호출한다', () => {
  const body = methodBody('GetCellText');

  // 단일 문자열 path 를 만들어 넘기던 형태가 되살아나면 다시 항상 '' 를 반환한다.
  assert.doesNotMatch(body, /`s\$\{/,
    '어떤 WASM 시그니처와도 맞지 않는 "s0:p1:c0:cell2:p0" 형태 path 문자열 금지');
  assert.doesNotMatch(body, /getTextInCellByPath\s*\(\s*path\s*\)/,
    'getTextInCellByPath 를 단일 인자로 호출 금지 — 실제 시그니처는 5 인자');

  assert.match(body, /getTextInCell\s*\(/, '인덱스 기반 getTextInCell 사용');
  assert.match(body, /getCellParagraphLength\s*\(/,
    '읽을 길이를 getCellParagraphLength 로 구해야 함');
});
