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
