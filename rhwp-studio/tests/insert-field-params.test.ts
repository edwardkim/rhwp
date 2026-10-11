import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  hasFieldInsertParams,
  parseFieldInsertParams,
} from '../src/command/field-insert-params.ts';

// `insert:field` 의 매개변수 경로 — 자동화가 대화상자 없이 캐럿 위치에 누름틀을 넣는다.
// 행위 증명(문서 반영·undo)은 e2e/automation-commands.test.mjs TC5c.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');
const LIMITS = { name: 250, guide: 250, memo: 1000 };

function slice(s: string, from: string, to: string): string {
  const a = s.indexOf(from);
  assert.notEqual(a, -1, `${from} not found`);
  const b = s.indexOf(to, a + from.length);
  return b === -1 ? s.slice(a) : s.slice(a, b);
}

test('필드 키가 없으면 null — 메뉴·툴바의 { anchorEl } 은 기존 대화상자 경로로 간다', () => {
  assert.equal(parseFieldInsertParams(undefined, LIMITS), null);
  assert.equal(parseFieldInsertParams({}, LIMITS), null);
  assert.equal(parseFieldInsertParams({ anchorEl: {} }, LIMITS), null);
  assert.equal(hasFieldInsertParams({ anchorEl: {} }), false);
  assert.equal(hasFieldInsertParams({ name: '' }), true);
});

test('매개변수로 누름틀 속성을 만들고 생략값은 대화상자와 같은 의미로 채운다', () => {
  assert.deepEqual(
    parseFieldInsertParams({ name: '현황.매출액', guide: '매출액 합계', memo: 'value:현황.매출액|sum' }, LIMITS),
    { name: '현황.매출액', guide: '매출액 합계', memo: 'value:현황.매출액|sum', editable: true },
  );
  assert.deepEqual(
    parseFieldInsertParams({ name: 'a', editable: false, type: 'clickhere' }, LIMITS),
    { name: 'a', guide: '', memo: '', editable: false },
  );
});

test('형식 오류·알 수 없는 키·상한 초과는 던진다', () => {
  assert.throws(() => parseFieldInsertParams({ name: 1 }, LIMITS), /name must be a string/);
  assert.throws(() => parseFieldInsertParams({ name: 'a', editable: 'yes' }, LIMITS), /editable must be a boolean/);
  assert.throws(() => parseFieldInsertParams({ name: 'a', type: 'date' }, LIMITS), /type must be "clickhere"/);
  assert.throws(() => parseFieldInsertParams({ name: 'a', guidetext: 'x' }, LIMITS), /unknown param: guidetext/);
  assert.throws(() => parseFieldInsertParams({ name: 'x'.repeat(251) }, LIMITS), /name exceeds 250/);
  assert.throws(() => parseFieldInsertParams({ memo: 'x'.repeat(1001) }, LIMITS), /memo exceeds 1000/);
});

test('insert:field 는 매개변수 경로를 대화상자보다 먼저 판정하고 같은 snapshot 경로를 쓴다', () => {
  const block = slice(src('src/command/commands/insert.ts'), "id: 'insert:field'", 'fieldInsertDialog.show()');
  assert.match(block, /runsWithoutDialog:\s*\(params\)\s*=>\s*hasFieldInsertParams\(params\)/);
  assert.match(block, /parseFieldInsertParams\(params,\s*\{\s*name:\s*MAX_FIELD_NAME_LEN,\s*guide:\s*MAX_FIELD_GUIDE_LEN,\s*memo:\s*MAX_FIELD_MEMO_LEN/);
  const paramsAt = block.indexOf('if (fromParams)');
  const dialogAt = block.indexOf('new FieldInsertDialog()');
  assert.ok(paramsAt !== -1 && paramsAt < dialogAt, '매개변수 판정이 대화상자 생성보다 앞선다');
  assert.match(block, /if \(fromParams\) \{\s*insertAtCaret\(fromParams\);\s*return;/, '매개변수 경로는 던짐을 삼키지 않는다');
  assert.match(block, /onApply = \(props\) => \{\s*try \{\s*insertAtCaret\(props\);/, '대화상자 경로는 같은 삽입 함수를 쓴다');
});

test('자동화는 runsWithoutDialog 가 참이면 needs-dialog 로 거절하지 않는다', () => {
  const host = src('src/automation/host.ts');
  assert.match(
    host,
    /!options\.allowDialog && def\?\.opensDialog && !def\.runsWithoutDialog\?\.\(params\)/,
  );
});
