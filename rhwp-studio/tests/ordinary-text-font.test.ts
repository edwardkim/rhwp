import test from 'node:test';
import assert from 'node:assert/strict';
import { ordinaryTextFontFaceData } from '../src/core/ordinary-text-font.ts';

test('일반 텍스트 사본은 원본 공급 바이트와 WASM 반환 view의 수명에서 독립적이다', () => {
  const source = Uint8Array.of(1, 2, 3).buffer;
  const memory = Uint8Array.of(99, 7, 8, 99);
  let calls = 0;
  const selected = ordinaryTextFontFaceData(source, 0, {
    ordinaryTextFontProgram(bytes, index) {
      calls++;
      assert.deepEqual([...bytes], [1, 2, 3]);
      assert.equal(index, 0);
      return memory.subarray(1, 3);
    },
  });
  memory.fill(0);
  assert.equal(calls, 1);
  assert.deepEqual([...new Uint8Array(selected!)], [7, 8]);
  assert.deepEqual([...new Uint8Array(source)], [1, 2, 3]);
});

test('공통 처리가 비적용을 반환하면 선택된 일반 프로그램을 그대로 보존한다', () => {
  const source = Uint8Array.of(1, 2, 3).buffer;
  assert.deepEqual(new Uint8Array(ordinaryTextFontFaceData(source, 0, {
    ordinaryTextFontProgram() { return undefined; },
  })!), new Uint8Array(source));
});

test('무효 collection 선택은 공통 프로그램 호출 전에 거절한다', () => {
  let called = false;
  const result = ordinaryTextFontFaceData(Uint8Array.of(1, 2, 3).buffer, 1, {
    ordinaryTextFontProgram() { called = true; return Uint8Array.of(7); },
  });
  assert.equal(result, null);
  assert.equal(called, false);
});
