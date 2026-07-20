import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#2548] IME 조합 경로의 삭제 count char 단위 가드.
//
// WASM 삭제/조회 count 는 Rust `Paragraph::delete_text_at` 의 char(Unicode scalar)
// 단위다(src/model/paragraph.rs). JS `String.length`(UTF-16 code unit)를 넘기면
// astral 문자(😀 등)에서 실제보다 많이 지워 인접 문자를 잃는다.
//
// [#2337-review] 는 undo/HF/FN 경로에 charCount() 를 적용했으나 IME 조합 경로
// (compositionLength / _iosLength)는 누락돼 있었다. 되돌아가지 않도록 핀한다.
//
// 주의: *커서 오프셋* (`anchor.charOffset + text.length`)은 studio 의 UTF-16 관례를
// 유지하므로 이 가드의 대상이 아니다 — tests/undo-delete-char-count.test.ts 와 동일 방침.
// 행위 증명(이모지 후보 조합 → 인접 문자 보존)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler-text.ts'), 'utf8');

test('IME 조합 길이는 charCount(scalar)로 계산한다', () => {
  assert.match(src, /this\.compositionLength = charCount\(text\)/,
    'compositionLength 는 deleteTextAt 의 삭제 count 로 쓰이므로 scalar 여야 함');
  assert.doesNotMatch(src, /this\.compositionLength = text\.length/,
    'UTF-16 length 를 삭제 count 로 쓰면 조합 중 astral 문자에서 뒤 문자를 잡아먹는다');
});

test('iOS 조합 폴백 길이도 charCount(scalar)로 계산한다', () => {
  assert.match(src, /this\._iosLength = charCount\(text\)/,
    '_iosLength 도 deleteTextAt 의 삭제 count 로 쓰이므로 scalar 여야 함');
  assert.doesNotMatch(src, /this\._iosLength = text\.length/,
    'iOS 폴백도 동일한 over-delete 결함을 갖는다');
});

test('삭제 count 로 전달되는 길이 변수에 UTF-16 length 가 남아있지 않다', () => {
  // deleteTextAt(pos, count) 는 count 를 그대로 wasm.deleteText* 에 넘긴다.
  // 그 count 의 출처인 두 변수만 검사한다(커서 오프셋은 대상 아님).
  const offenders = [...src.matchAll(/this\.(compositionLength|_iosLength) = ([^;]+);/g)]
    .filter((m) => /\.length\b/.test(m[2]) && !/charCount\(/.test(m[2]));
  assert.deepEqual(offenders.map((m) => m[0]), [],
    '삭제 count 변수에 UTF-16 length 대입이 남아있음');
});
