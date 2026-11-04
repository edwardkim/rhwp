import test from 'node:test';
import assert from 'node:assert/strict';
import { preparePdfForEditor, isPdfInput, publishPreparedPdf } from '../src/core/pdf-open.ts';
import type { PdfImportSession } from '../../rhwp-shared/pdf/import-session.js';

test('PDF reading is bounded before the input supplier is called', async () => {
  let reads = 0, disposed = 0;
  const session = { dispose() { disposed++; } } as PdfImportSession;
  await assert.rejects(preparePdfForEditor({ byteLength: 9, read: async () => { reads++; return new Uint8Array(9); } },
    {}, { backend: { inputBytes: 8 } }, { signal: new AbortController().signal, timeoutMs: 1000 }, session), /PDF_RESOURCE_LIMIT/);
  assert.equal(reads, 0); assert.equal(disposed, 1);
});

test('candidate conversion and editable export share cancellation/deadline and always retire the Worker', async () => {
  let disposed = 0, exported = 0;
  const controller = new AbortController();
  const session = { async importPdf() { return { pageCount: 1, diagnostics: { resourcePolicy: 'constrained' } }; },
    async exportDocument() { exported++; controller.abort(); return new Uint8Array([7]); },
    dispose() { disposed++; },
  } as unknown as PdfImportSession;
  await assert.rejects(preparePdfForEditor({ byteLength: 1, read: async () => new Uint8Array([1]) },
    { resourceProfile: 'constrained' }, { backend: { inputBytes: 1 } }, { signal: controller.signal, timeoutMs: 1000 }, session), /PDF_IMPORT_CANCELLED/);
  assert.equal(exported, 1); assert.equal(disposed, 1);
  assert.equal(isPdfInput(new TextEncoder().encode('%PDF-1.7'), 'renamed.bin'), true);
});


test('a pending input supplier is retired on cancellation or deadline', async () => {
  for (const cancel of [true, false]) {
    let disposed = 0, imported = 0;
    const controller = new AbortController();
    const session = { dispose() { disposed++; }, importPdf() { imported++; } } as unknown as PdfImportSession;
    const promise = preparePdfForEditor({ byteLength: 1, read: () => new Promise(() => {}) }, {},
      { backend: { inputBytes: 1 } }, { signal: controller.signal, timeoutMs: cancel ? 1000 : 10 }, session);
    if (cancel) setTimeout(() => controller.abort(), 0);
    await assert.rejects(promise, new RegExp(cancel ? 'CANCELLED' : 'TIMEOUT'));
    assert.equal(imported, 0); assert.equal(disposed, 1);
  }
});

test('editor handoff carries the actual post-export remaining allowance', async () => {
  let exported = false, disposed = 0;
  const session = { async importPdf() { return { pageCount: 1, diagnostics: {} }; },
    async exportDocument() { exported = true; return new Uint8Array([1]); },
    getEditorReopenLimits() { assert.ok(exported); return { inputBytes: 8, decodedBytes: 123, options: '{}' }; },
    dispose() { disposed++; },
  } as unknown as PdfImportSession;
  const prepared = await preparePdfForEditor({ byteLength: 1, read: async () => new Uint8Array([1]) }, {},
    { backend: { inputBytes: 8 } }, { signal: new AbortController().signal, timeoutMs: 1000 }, session);
  assert.deepEqual(prepared.reopen, { inputBytes: 8, decodedBytes: 123, options: '{}' });
  assert.ok(Object.isFrozen(prepared.reopen)); assert.equal(disposed, 1);
});

test('candidate publication observes queued cancel, expiry and a competing document after synchronous reopen', async () => {
  for (const failure of ['cancel', 'timeout', 'superseded', 'prepare', 'create', 'none']) {
    const controller = new AbortController(); let current = true, destroyed = 0, published = 0;
    const result = { input: new Uint8Array([1, 2]), editable: new Uint8Array([3]), deadline: performance.now() + 1000,
      pageCount: 1, diagnostics: {}, reopen: { inputBytes: 10, decodedBytes: 100, options: '{}' } };
    const run = publishPreparedPdf(result, controller.signal, () => current, {
      create() { if (failure === 'create') throw new Error('create'); return { live: true }; },
      prepare() {
        if (failure === 'prepare') throw new Error('prepare');
        setTimeout(() => {
          if (failure === 'cancel') controller.abort();
          if (failure === 'timeout') result.deadline = 0;
          if (failure === 'superseded') current = false;
        }, 0);
      },
      publish(candidate, digest) { assert.ok(candidate.live); assert.match(digest, /^blake3:[a-f0-9]{64}$/); published++; },
      destroy(candidate) { assert.ok(candidate.live); candidate.live = false; destroyed++; },
    });
    if (failure === 'none') { await run; assert.equal(published, 1); assert.equal(destroyed, 0); }
    else { await assert.rejects(run); assert.equal(published, 0); assert.equal(destroyed, failure === 'create' ? 0 : 1); }
  }
});
