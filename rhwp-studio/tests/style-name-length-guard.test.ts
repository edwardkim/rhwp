import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// #2866 (#2851/#2862 재발 사례): 스타일 이름/영문 이름 입력 길이 미검증 → Rust 직렬화기
// (src/serializer/doc_info.rs의 serialize_style → src/serializer/byte_writer.rs의
// write_hwp_string)의 `utf16.len() as u16` 캐스팅 랩어라운드로 STYLE 레코드가 손상될 수
// 있었다. `.rs`를 수정하지 않는 범위에서, 프런트엔드 다이얼로그가 wasm 호출 전에 이름 길이를
// 안전한 상한(MAX_STYLE_NAME_LEN)으로 막는지를 소스 가드로 검증한다.

const dir = path.dirname(fileURLToPath(import.meta.url));
const src = readFileSync(path.join(dir, '../src/ui/style-edit-dialog.ts'), 'utf8');

test('스타일 이름 상한은 u16 캐스팅이 랩어라운드되는 65536보다 충분히 작다', () => {
  const m = src.match(/MAX_STYLE_NAME_LEN\s*=\s*(\d+)/);
  assert.ok(m, 'MAX_STYLE_NAME_LEN 상수를 찾을 수 없음');
  const limit = Number(m![1]);
  assert.ok(limit > 0 && limit < 65536, `한도(${limit})가 u16 랩어라운드 지점보다 작아야 함`);
});
