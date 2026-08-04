import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// #2883 (#2851/#2862/#2866/#2878 형제 필드 재발): 개체 설명문(picture-props-dialog.ts)과
// 수식 스크립트(equation-editor-dialog.ts) 입력 길이 미검증 → Rust 직렬화기
// (src/serializer/control.rs)가 각각 CommonObjAttr.description / EQEDIT script를
// write_hwp_string()(src/serializer/byte_writer.rs)로 기록할 때 UTF-16 코드 유닛 수를
// `as u16`으로 캐스팅해 65536 이상이면 랩어라운드되어 레코드가 손상될 수 있었다.
// `.rs`를 수정하지 않는 범위에서, 프런트엔드 다이얼로그가 wasm 호출 전에 각 필드 길이를
// 안전한 상한(MAX_OBJECT_DESCRIPTION_LEN/MAX_EQUATION_SCRIPT_LEN)으로 막는지를
// 소스 가드로 검증한다.

const dir = path.dirname(fileURLToPath(import.meta.url));
const pictureSrc = readFileSync(path.join(dir, '../src/ui/picture-props-dialog.ts'), 'utf8');
const equationSrc = readFileSync(path.join(dir, '../src/ui/equation-editor-dialog.ts'), 'utf8');

test('개체 설명문/수식 스크립트 상한은 u16 캐스팅이 랩어라운드되는 65536보다 충분히 작다', () => {
  const descMatch = pictureSrc.match(/MAX_OBJECT_DESCRIPTION_LEN\s*=\s*(\d+)/);
  const scriptMatch = equationSrc.match(/MAX_EQUATION_SCRIPT_LEN\s*=\s*(\d+)/);
  assert.ok(descMatch, 'MAX_OBJECT_DESCRIPTION_LEN 상수를 찾을 수 없음');
  assert.ok(scriptMatch, 'MAX_EQUATION_SCRIPT_LEN 상수를 찾을 수 없음');

  const descLimit = Number(descMatch![1]);
  const scriptLimit = Number(scriptMatch![1]);

  assert.ok(descLimit > 0 && descLimit < 65536, `개체 설명문 상한(${descLimit})이 u16 랩어라운드 지점보다 작아야 함`);
  assert.ok(scriptLimit > 0 && scriptLimit < 65536, `수식 스크립트 상한(${scriptLimit})이 u16 랩어라운드 지점보다 작아야 함`);
});

test('handleOk()가 상한 초과 시 저장을 거부한다', () => {
  assert.match(pictureSrc, /if \(this\.descInput\.value\.length > MAX_OBJECT_DESCRIPTION_LEN\)/);
  assert.match(equationSrc, /if \(script\.length > MAX_EQUATION_SCRIPT_LEN\)/);
});
