import test from 'node:test';
import assert from 'node:assert/strict';

import {
  createDocumentForBytes,
  isPdfBytes,
  normalizePdfImportLayout,
} from '../src/core/pdf-import-layout.ts';
import { userSettings } from '../src/core/user-settings.ts';

const pdf = new TextEncoder().encode('%PDF-1.7\n%âã\n1 0 obj');
const hwp = new Uint8Array([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

test('PDF bytes are recognised the way the engine detects the format', () => {
  assert.equal(isPdfBytes(pdf), true);
  assert.equal(isPdfBytes(hwp), false);
  // The engine only accepts a header at offset 0; leading bytes are not a PDF.
  assert.equal(isPdfBytes(new TextEncoder().encode('\n%PDF-1.7')), false);
  assert.equal(isPdfBytes(new TextEncoder().encode('%PDF')), false);
  assert.equal(isPdfBytes(new Uint8Array()), false);
});

test('unknown layout values fall back to the original-look default', () => {
  assert.equal(normalizePdfImportLayout('editable'), 'editable');
  assert.equal(normalizePdfImportLayout('fidelity'), 'fidelity');
  for (const value of [undefined, null, '', 'Editable', 1, {}]) {
    assert.equal(normalizePdfImportLayout(value), 'fidelity');
  }
});

/** Records which creation path a load took. */
function factory(withImport = true) {
  const calls: { path: 'constructor' | 'importPdf'; options?: unknown }[] = [];
  class Doc {
    data: Uint8Array;
    constructor(data: Uint8Array) {
      this.data = data;
      calls.push({ path: 'constructor' });
    }
  }
  const api = Doc as unknown as {
    new (data: Uint8Array): Doc;
    importPdf?: (data: Uint8Array, options: unknown) => Doc;
  };
  if (withImport) {
    api.importPdf = (data, options) => {
      calls.push({ path: 'importPdf', options });
      return Object.create(Doc.prototype, { data: { value: data } }) as Doc;
    };
  }
  return { api, calls };
}

test('only an editable-layout PDF opens through importPdf with that layout', () => {
  const cases: [Uint8Array, 'fidelity' | 'editable', string, unknown][] = [
    [pdf, 'editable', 'importPdf', { layout: 'editable' }],
    [pdf, 'fidelity', 'constructor', undefined],
    [hwp, 'editable', 'constructor', undefined],
    [hwp, 'fidelity', 'constructor', undefined],
  ];
  for (const [data, layout, path, options] of cases) {
    const { api, calls } = factory();
    const doc = createDocumentForBytes(api, data, layout);
    assert.equal(doc.data, data);
    assert.deepEqual(calls, [{ path, ...(options === undefined ? {} : { options }) }]);
  }
});

test('a WASM build without importPdf still opens the PDF with the default layout', () => {
  const { api, calls } = factory(false);
  createDocumentForBytes(api, pdf, 'editable');
  assert.deepEqual(calls, [{ path: 'constructor' }]);
});

test('the PDF import layout is stored in rhwp-settings and can be switched back', () => {
  const originalStorage = (globalThis as { localStorage?: Storage }).localStorage;
  const store = new Map<string, string>();
  (globalThis as { localStorage?: Storage }).localStorage = {
    get length() { return store.size; },
    clear() { store.clear(); },
    getItem(key: string) { return store.get(key) ?? null; },
    key(index: number) { return Array.from(store.keys())[index] ?? null; },
    removeItem(key: string) { store.delete(key); },
    setItem(key: string, value: string) { store.set(key, value); },
  } as Storage;
  try {
    assert.equal(userSettings.getPdfImportLayout(), 'fidelity');
    userSettings.setPdfImportLayout('editable');
    assert.equal(userSettings.getPdfImportLayout(), 'editable');
    assert.equal(JSON.parse(store.get('rhwp-settings') ?? '{}').import.pdfLayout, 'editable');
    // A value outside the two layouts is stored as the default.
    userSettings.setPdfImportLayout('columns' as never);
    assert.equal(userSettings.getPdfImportLayout(), 'fidelity');
    assert.equal(JSON.parse(store.get('rhwp-settings') ?? '{}').import.pdfLayout, 'fidelity');
  } finally {
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});
