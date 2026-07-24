import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// #2851: 필드 이름 입력 길이 미검증 → Rust 직렬화기(src/serializer/control.rs)의
// `nlen as u16` 캐스팅 랩어라운드로 CTRL_DATA 레코드가 손상될 수 있었다.
// `.rs`를 수정하지 않는 범위에서, 프런트엔드 다이얼로그가 wasm 호출 전에
// 이름 길이를 안전한 상한(MAX_FIELD_NAME_LEN)으로 막는지를 소스 가드로 검증한다.

const dir = path.dirname(fileURLToPath(import.meta.url));
const editSrc = readFileSync(path.join(dir, '../src/ui/field-edit-dialog.ts'), 'utf8');
const insertSrc = readFileSync(path.join(dir, '../src/ui/field-insert-dialog.ts'), 'utf8');

test('필드 이름 상한은 u16 캐스팅이 랩어라운드되는 65536보다 충분히 작다', () => {
  const m = editSrc.match(/MAX_FIELD_NAME_LEN\s*=\s*(\d+)/);
  assert.ok(m, 'MAX_FIELD_NAME_LEN 상수를 찾을 수 없음');
  const limit = Number(m![1]);
  assert.ok(limit > 0 && limit < 65536, `한도(${limit})가 u16 랩어라운드 지점보다 작아야 함`);
});

test('FieldEditDialog는 이름 길이 초과 시 onConfirm이 false를 반환해 다이얼로그를 닫지 않는다', () => {
  assert.match(
    editSrc,
    /nameInput\.value\.length > MAX_FIELD_NAME_LEN[\s\S]{0,160}return false;/,
  );
});

test('FieldInsertDialog도 동일한 이름 길이 가드를 갖는다', () => {
  assert.match(
    insertSrc,
    /nameInput\.value\.length > MAX_FIELD_NAME_LEN[\s\S]{0,160}return false;/,
  );
  assert.match(insertSrc, /MAX_FIELD_NAME_LEN/);
});
