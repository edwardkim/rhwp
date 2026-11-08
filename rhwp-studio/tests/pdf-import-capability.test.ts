import test from 'node:test';
import assert from 'node:assert/strict';
import { hasPdfImportCapability } from '../src/core/pdf-import-capability.ts';

test('PDF capability is confirmed by the runtime, not an importPdf export', () => {
  for (const api of [null, {}, { importPdf() {} }, { supportsPdfImport: false },
    { supportsPdfImport: () => false }, { supportsPdfImport: () => 'true' }]) {
    assert.equal(hasPdfImportCapability(api), false);
  }
  assert.equal(hasPdfImportCapability({ supportsPdfImport: () => true }), true);
});
