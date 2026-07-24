import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// #2878 (#2851 형제 필드 재발): 누름틀(ClickHere) 안내문(guide)/메모(memo) 입력 길이
// 미검증 → Rust 직렬화기(src/serializer/control.rs)가 guide/memo를 포함한 command
// 문자열 전체 길이를 `cmd_len as u16`으로 기록할 때 랩어라운드되어 CTRL_DATA 레코드가
// 손상될 수 있었다. `.rs`를 수정하지 않는 범위에서, 프런트엔드 다이얼로그가 wasm 호출
// 전에 guide/memo 길이를 안전한 상한(MAX_FIELD_GUIDE_LEN/MAX_FIELD_MEMO_LEN)으로
// 막는지를 소스 가드로 검증한다.

const dir = path.dirname(fileURLToPath(import.meta.url));
const src = readFileSync(path.join(dir, '../src/ui/field-edit-dialog.ts'), 'utf8');

test('누름틀 안내문/메모 상한은 u16 캐스팅이 랩어라운드되는 65536보다 충분히 작다', () => {
  const guideMatch = src.match(/MAX_FIELD_GUIDE_LEN\s*=\s*(\d+)/);
  const memoMatch = src.match(/MAX_FIELD_MEMO_LEN\s*=\s*(\d+)/);
  assert.ok(guideMatch, 'MAX_FIELD_GUIDE_LEN 상수를 찾을 수 없음');
  assert.ok(memoMatch, 'MAX_FIELD_MEMO_LEN 상수를 찾을 수 없음');

  const guideLimit = Number(guideMatch![1]);
  const memoLimit = Number(memoMatch![1]);

  assert.ok(
    guideLimit > 0 && memoLimit > 0 && guideLimit + memoLimit < 65536,
    `합산 한도(${guideLimit + memoLimit})가 u16 랩어라운드 지점보다 작아야 함`,
  );
});
