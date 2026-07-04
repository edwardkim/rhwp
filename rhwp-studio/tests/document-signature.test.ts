import test from 'node:test';
import assert from 'node:assert/strict';

import {
  detectDocumentByteKind,
  assertRemoteDocumentBytes,
} from '../src/core/document-signature.ts';

function bytes(...nums: number[]): Uint8Array {
  return new Uint8Array(nums);
}

function ascii(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}

// "HWP Document File V3.00" — HWP3 고전 바이너리 매직 (코어 src/parser/hwp3/mod.rs).
const HWP3_HEADER = ascii('HWP Document File V3.00\x1a');
const HWP5_CFB = bytes(0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00);
const HWPX_ZIP = bytes(0x50, 0x4B, 0x03, 0x04, 0x00, 0x00);

test('HWP3 매직을 hwp 로 인식한다 (#1657)', () => {
  assert.equal(detectDocumentByteKind(HWP3_HEADER, null), 'hwp');
});

test('서버가 text/html 404 를 보내도 HWP3 본문 매직이 우선한다 (pps.go.kr 케이스)', () => {
  // 조달청은 HWP3 본문에 Content-Type: text/html, HTTP 404 를 보낸다.
  assert.equal(detectDocumentByteKind(HWP3_HEADER, 'text/html'), 'hwp');
  assert.doesNotThrow(() => assertRemoteDocumentBytes(HWP3_HEADER, 'text/html'));
});

test('HWP5(CFB) / HWPX(ZIP) 회귀 없음', () => {
  assert.equal(detectDocumentByteKind(HWP5_CFB, null), 'hwp');
  assert.equal(detectDocumentByteKind(HWPX_ZIP, null), 'hwpx');
  assert.doesNotThrow(() => assertRemoteDocumentBytes(HWP5_CFB, null));
  assert.doesNotThrow(() => assertRemoteDocumentBytes(HWPX_ZIP, null));
});

test('실제 HTML 오류/미리보기 페이지는 계속 거부한다', () => {
  const html = ascii('<!DOCTYPE html><html><body>error</body></html>');
  assert.equal(detectDocumentByteKind(html, 'text/html'), 'html');
  assert.throws(
    () => assertRemoteDocumentBytes(html, 'text/html'),
    /미리보기\/오류 페이지/,
  );
});

test('매직 없는 알 수 없는 바이트는 거부한다', () => {
  const junk = bytes(0x01, 0x02, 0x03, 0x04);
  assert.equal(detectDocumentByteKind(junk, null), 'unknown');
  assert.throws(
    () => assertRemoteDocumentBytes(junk, null),
    /시그니처를 확인할 수 없습니다/,
  );
});

test('HWP3 매직보다 짧은 바이트는 오인식하지 않는다', () => {
  // "HWP " 까지만 — 전체 매직 미달이면 hwp 가 아니다.
  assert.notEqual(detectDocumentByteKind(ascii('HWP '), null), 'hwp');
});
