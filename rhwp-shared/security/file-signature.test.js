// 실행: node --test rhwp-shared/security/file-signature.test.js

'use strict';

import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';
import vm from 'node:vm';

const source = fs.readFileSync(new URL('./file-signature.js', import.meta.url), 'utf8');
const context = { ArrayBuffer, TextDecoder, Uint8Array };
vm.runInNewContext(
  `${source}\nglobalThis.__rhwpFileSignature = { verifyDocumentSignature, verifyHwpSignature };`,
  context,
  { filename: 'file-signature.js' },
);

const {
  verifyDocumentSignature,
  verifyHwpSignature,
} = context.__rhwpFileSignature;

const repositoryRoot = new URL('../../', import.meta.url);
const safariManifest = JSON.parse(
  fs.readFileSync(new URL('rhwp-safari/src/manifest.json', repositoryRoot), 'utf8'),
);
const safariBackground = fs.readFileSync(
  new URL('rhwp-safari/src/background.js', repositoryRoot),
  'utf8',
);
const safariBuildScript = fs.readFileSync(
  new URL('rhwp-safari/build.sh', repositoryRoot),
  'utf8',
);

function assertSignature(actual, expected) {
  assert.equal(actual.isDocument ?? actual.isHwp, expected.isDocument ?? expected.isHwp);
  assert.equal(actual.format, expected.format);
}

function utf16Bytes(text, littleEndian) {
  const bytes = [];
  for (const unit of text) {
    const code = unit.charCodeAt(0);
    if (littleEndian) {
      bytes.push(code & 0xFF, code >> 8);
    } else {
      bytes.push(code >> 8, code & 0xFF);
    }
  }
  return Uint8Array.from(bytes);
}

test('HWP와 HWPX 매직 넘버를 기존 형식으로 판정한다', () => {
  assertSignature(verifyHwpSignature(Uint8Array.from([0xD0, 0xCF, 0x11, 0xE0])), {
    isHwp: true,
    format: 'hwp',
  });
  assertSignature(verifyHwpSignature(Uint8Array.from([0x50, 0x4B, 0x03, 0x04])), {
    isHwp: true,
    format: 'hwpx',
  });
});

test('UTF-8 HML의 XML 선언과 HWPML Version 루트를 판정한다', () => {
  const hml = new TextEncoder().encode('\uFEFF<?xml version="1.0"?><HWPML Version="2.91"></HWPML>');

  assertSignature(verifyDocumentSignature(hml), { isDocument: true, format: 'hml' });
  assertSignature(verifyHwpSignature(hml), { isHwp: false, format: null });
});

test('UTF-16 LE와 BE HML을 판정한다', () => {
  const xml = '<?xml version="1.0"?><HWPML Version="2.91"></HWPML>';
  const le = Uint8Array.from([0xFF, 0xFE, ...utf16Bytes(xml, true)]);
  const be = Uint8Array.from([0xFE, 0xFF, ...utf16Bytes(xml, false)]);

  assertSignature(verifyDocumentSignature(le), { isDocument: true, format: 'hml' });
  assertSignature(verifyDocumentSignature(be), { isDocument: true, format: 'hml' });
});

test('저장소의 실제 HML 샘플을 판정한다', () => {
  for (const sample of ['samples/hml/aligns.hml', 'samples/hml/formatting_table.hml']) {
    const bytes = fs.readFileSync(new URL(sample, repositoryRoot));
    assertSignature(verifyDocumentSignature(bytes), { isDocument: true, format: 'hml' });
  }
});

test('HTML, JSON, 빈 Version HWPML은 HML로 허용하지 않는다', () => {
  for (const source of [
    '<!doctype html><html><body>not hml</body></html>',
    '{"document":"not hml"}',
    '<HWPML Version=""></HWPML>',
    '<HWPMLVersion="2.91"></HWPMLVersion>',
  ]) {
    assertSignature(
      verifyDocumentSignature(new TextEncoder().encode(source)),
      { isDocument: false, format: null },
    );
  }
});

test('Safari가 공용 문서 판별기를 background보다 먼저 적재하고 dist로 복사한다', () => {
  assert.deepEqual(safariManifest.background.scripts, ['file-signature.js', 'background.js']);
  assert.match(safariBuildScript, /cp "\$ROOT\/rhwp-shared\/security\/file-signature\.js" "\$DIST\/file-signature\.js"/);
  assert.match(safariBackground, /verifyDocumentSignature\(buf\)/);

  const listeners = {};
  const safariContext = {
    ArrayBuffer,
    TextDecoder,
    URL,
    Uint8Array,
    browser: {
      action: { onClicked: { addListener: (listener) => { listeners.action = listener; } } },
      runtime: {
        onInstalled: { addListener: (listener) => { listeners.installed = listener; } },
        onMessage: { addListener: (listener) => { listeners.message = listener; } },
      },
      contextMenus: { onClicked: { addListener: (listener) => { listeners.contextMenu = listener; } } },
      storage: { local: { get: async () => ({}), set: async () => {} } },
    },
  };
  vm.createContext(safariContext);
  vm.runInContext(source, safariContext, { filename: 'file-signature.js' });
  vm.runInContext(safariBackground, safariContext, { filename: 'background.js' });

  assert.equal(typeof listeners.message, 'function');
  assert.equal(typeof listeners.installed, 'function');
  assert.equal(typeof listeners.action, 'function');
  assert.equal(typeof listeners.contextMenu, 'function');
});
