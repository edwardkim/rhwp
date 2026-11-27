import { test } from 'node:test';
import assert from 'node:assert/strict';
import { requireMemoSuccess, memoTargetPath, validateMemoEdit } from '../src/core/memo-editor.ts';

test('메모 실패 JSON은 성공으로 처리하지 않는다', () => {
  requireMemoSuccess('{"ok":true}');
  assert.throws(() => requireMemoSuccess('{"ok":false,"code":"memo_overlap","message":"내부 메시지"}'), /이미 메모/);
  assert.throws(() => requireMemoSuccess('{}'));
  assert.throws(() => requireMemoSuccess('not json'));
});
test('셀 경로의 모든 깊이를 API 필드로 전달한다', () => {
  assert.deepEqual(JSON.parse(memoTargetPath({ section: 1, para: 2, cellPath: [[3,4,5],[6,7,8]] })),
    [{ controlIndex: 3, cellIndex: 4, cellParaIndex: 5 }, { controlIndex: 6, cellIndex: 7, cellParaIndex: 8 }]);
});
test('빈 본문과 사라진 메모, 읽기 전용 메모를 거부한다', () => {
  assert.throws(() => validateMemoEdit({ kind: 'insert', author: '', text: ' ' }, []));
  assert.throws(() => validateMemoEdit({ kind: 'remove', id: 2 }, []));
  assert.throws(() => validateMemoEdit({ kind: 'remove', id: 2 }, [{ id: 2, editable: false }] as any));
  validateMemoEdit({ kind: 'insert', author: '', text: '내용' }, []);
});

test('부모 메모 삭제로 댓글을 조용히 함께 지우지 않는다', () => {
  assert.throws(() => validateMemoEdit({ kind: 'remove', id: 2 },
    [{ id: 2, editable: true }, { id: 3, parentId: 2, editable: true }] as any), /댓글/);
});
