import test from 'node:test';
import assert from 'node:assert/strict';

import {
  parseLocalBodyTextReplaceResult,
} from '../src/core/local-text-replace-result.ts';

test('stable local result는 pending page-local effect로 정규화된다', () => {
  assert.deepEqual(parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":false}',
  ), {
    ok: true,
    charOffset: 4,
    documentPaginationPending: true,
    flowChanged: false,
  });
});

test('완료된 flow boundary result는 pending 없이 허용된다', () => {
  assert.deepEqual(parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":false,"flowChanged":true}',
  ), {
    ok: true,
    charOffset: 4,
    documentPaginationPending: false,
    flowChanged: true,
  });
});

test('모순되거나 불완전한 local result는 거부한다', () => {
  assert.throws(() => parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":true}',
  ));
  assert.throws(() => parseLocalBodyTextReplaceResult('{"ok":true}'));
});

test('stable local result의 focusedPagePatch 를 정규화한다', () => {
  assert.deepEqual(parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":false,'
    + '"focusedPagePatch":{"pageIndex":0,"x":10,"y":20,"width":30,"height":12}}',
  ), {
    ok: true,
    charOffset: 4,
    documentPaginationPending: true,
    flowChanged: false,
    focusedPagePatch: { pageIndex: 0, x: 10, y: 20, width: 30, height: 12 },
  });
});

test('깨진 focusedPagePatch 는 무시하고 국소 조판 신호는 유지한다', () => {
  assert.deepEqual(parseLocalBodyTextReplaceResult(
    '{"ok":true,"charOffset":4,"documentPaginationPending":true,"flowChanged":false,'
    + '"focusedPagePatch":{"pageIndex":0,"x":10,"y":20,"width":0,"height":12}}',
  ), {
    ok: true,
    charOffset: 4,
    documentPaginationPending: true,
    flowChanged: false,
  });
});
