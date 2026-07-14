import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';

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

function withUtf8Bom(data: Uint8Array): Uint8Array {
  return new Uint8Array([0xEF, 0xBB, 0xBF, ...data]);
}

function utf16(text: string, byteOrder: 'le' | 'be'): Uint8Array {
  const data = new Uint8Array(2 + text.length * 2);
  data.set(byteOrder === 'le' ? [0xFF, 0xFE] : [0xFE, 0xFF]);
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index);
    const offset = 2 + index * 2;
    data[offset] = byteOrder === 'le' ? code & 0xFF : code >> 8;
    data[offset + 1] = byteOrder === 'le' ? code >> 8 : code & 0xFF;
  }
  return data;
}

// "HWP Document File V3.00" — HWP3 고전 바이너리 매직 (코어 src/parser/hwp3/mod.rs).
const HWP3_HEADER = ascii('HWP Document File V3.00\x1a');
const HWP5_CFB = bytes(0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00, 0x00);
const HWPX_ZIP = bytes(0x50, 0x4B, 0x03, 0x04, 0x00, 0x00);
const HML_UTF8 = ascii(`<?xml version="1.0" encoding="UTF-8"?>
<HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core" Version="2.1">
  <HEAD/>
</HWPML>`);

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

test('HWPML signature는 잘못된 text/html Content-Type보다 우선한다', () => {
  assert.equal(detectDocumentByteKind(HML_UTF8, 'text/html'), 'hml');
  assert.doesNotThrow(() => assertRemoteDocumentBytes(HML_UTF8, 'text/html'));
});

test('UTF-8 BOM과 UTF-16LE/BE BOM HWPML signature를 인식한다', () => {
  const xml = `<?xml version="1.0"?><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"><HEAD/></HWPML>`;
  assert.equal(detectDocumentByteKind(withUtf8Bom(ascii(xml))), 'hml');
  assert.equal(detectDocumentByteKind(utf16(xml, 'le')), 'hml');
  assert.equal(detectDocumentByteKind(utf16(xml, 'be')), 'hml');
});

test('namespace 없는 HWPML 이름만으로는 HML로 오인하지 않는다', () => {
  assert.equal(detectDocumentByteKind(ascii('<?xml version="1.0"?><HWPML Version="2.1" />')), 'xml');
});

test('실물 legacy HWPML fixture는 namespace 없이도 HML로 인식한다', () => {
  const fixture = new Uint8Array(
    readFileSync(new URL('../../samples/hml/aligns.hml', import.meta.url)),
  );
  assert.equal(detectDocumentByteKind(fixture, 'text/html'), 'hml');
  assert.doesNotThrow(() => assertRemoteDocumentBytes(fixture, 'text/html'));
});

test('legacy HWPML은 HEAD/BODY가 64KiB 뒤에 있어도 root 속성으로 인식한다', () => {
  const xml = ascii(`<?xml version="1.0"?><HWPML Version="2.91" SubVersion="10.0.0.0" Style="embed">${' '.repeat(70 * 1024)}<HEAD/><BODY/></HWPML>`);
  assert.equal(detectDocumentByteKind(xml, 'application/xml'), 'hml');
});

test('legacy HWPML root 속성 계약이 불완전하면 일반 XML로 남긴다', () => {
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML Version="2.91" SubVersion="10.0.0.0"><HEAD/><BODY/></HWPML>',
  )), 'xml');
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML Version="2.91" Style="embed"><HEAD/><BODY/></HWPML>',
  )), 'xml');
});

test('속성이나 namespace만 둔 self-closing HWPML root는 signature로 인정하지 않는다', () => {
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML Version="2.91" SubVersion="10.0.0.0" Style="embed"/>',
  )), 'xml');
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"/>',
  )), 'xml');
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"></HWPML>',
  )), 'xml');
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"><!-- marker --></HWPML>',
  )), 'xml');
});

test('HWPML 태그가 중첩된 일반 XML은 HML로 오인하지 않는다', () => {
  const xml = ascii(`<?xml version="1.0"?>
<response><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core" /></response>`);
  assert.equal(detectDocumentByteKind(xml, 'application/xml'), 'xml');
});

test('XML 선언으로 시작하는 HTML 오류 페이지는 html로 분류한다', () => {
  const html = ascii('<?xml version="1.0"?><html><body>login required</body></html>');
  assert.equal(detectDocumentByteKind(html, 'text/html'), 'html');
  assert.throws(() => assertRemoteDocumentBytes(html, 'text/html'), /미리보기\/오류 페이지/);
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

test('악성 XML 주석 prefix를 제한 시간 안에 거부한다', () => {
  assert.equal(detectDocumentByteKind(ascii(
    '<?xml version="1.0"?>\n<!-- first --> <!-- second -->'
      + '<HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"><HEAD/></HWPML>',
  )), 'hml');
  assert.equal(detectDocumentByteKind(ascii(
    '<!-- marker --><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"></HWPML>',
  )), 'xml');
  assert.equal(detectDocumentByteKind(ascii(
    '<!-- marker --><HWPML xmlns="http://www.hancom.co.kr/hwpml/2011/core"><HEAD/></HWPML>',
  )), 'hml');
  assert.equal(detectDocumentByteKind(ascii('<!-- marker --><html><body/></html>')), 'html');

  const maliciousPrefix = ascii(`<!--${'--><!--'.repeat(24)}`);
  const startedAt = performance.now();

  assert.equal(detectDocumentByteKind(maliciousPrefix), 'unknown');
  assert.ok(
    performance.now() - startedAt < 100,
    '172-byte malicious comment prefix should be classified within 100ms',
  );
});
