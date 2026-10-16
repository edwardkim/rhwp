import test from 'node:test';
import assert from 'node:assert/strict';
import { PdfImportSession } from './import-session.js';

function fixture() {
  const workers = [];
  const session = new PdfImportSession({ workerFactory: () => {
    const worker = {
      messages: [], terminated: false,
      postMessage(message) { this.messages.push(message); },
      terminate() { this.terminated = true; },
      reply(result, error) { this.onmessage?.({ data: { id: this.messages.at(-1).id, result, error } }); },
    };
    workers.push(worker);
    return worker;
  } });
  return { session, workers };
}
const bytes = new Uint8Array([37, 80, 68, 70, 45]);
const rejected = code => error => error.code === code;

async function active(f) {
  const loaded = f.session.importPdf(bytes);
  assert.notEqual(f.workers[0].messages[0].bytes.buffer, bytes.buffer);
  f.workers[0].reply({ pageCount: 3 });
  await loaded;
}

test('failed candidate leaves active session usable; next import reuses candidate and replaces only on success', async () => {
  const f = fixture();
  await active(f);
  const failed = f.session.importPdf(bytes);
  f.workers[1].reply(null, { code: 'PDF_PARSE_FAILED', message: 'invalid PDF' });
  await assert.rejects(failed, rejected('PDF_PARSE_FAILED'));
  const inspect = f.session.inspect();
  f.workers[0].reply({ pageCount: 3 });
  assert.deepEqual(await inspect, { pageCount: 3 });
  assert.equal(f.workers[0].terminated, false);
  const next = f.session.importPdf(bytes, { sourcePages: [1] });
  assert.equal(f.workers.length, 2);
  f.workers[1].reply({ pageCount: 1 });
  await next;
  assert.equal(f.workers[0].terminated, true);
  assert.equal(f.workers[1].terminated, false);
  assert.equal(bytes.byteLength, 5);
  f.session.dispose();
});

test('input views copy only visible bytes and oversized input is rejected before Worker creation', async () => {
  const f = fixture();
  await assert.rejects(f.session.importPdf(new Uint8Array(16 * 1024 * 1024 + 1)), rejected('PDF_RESOURCE_LIMIT'));
  assert.equal(f.workers.length, 0);
  const backing = new Uint8Array(4096);
  backing.set(bytes, 100);
  const loaded = f.session.importPdf(backing.subarray(100, 105));
  assert.equal(f.workers[0].messages[0].bytes.buffer.byteLength, 5);
  assert.deepEqual(f.workers[0].messages[0].bytes, bytes);
  f.workers[0].reply({ pageCount: 1 });
  await loaded;
  f.session.dispose();
});

test('abort terminates candidate, ignores late completion and recreates; active survives', async () => {
  const f = fixture();
  await active(f);
  const controller = new AbortController();
  const pending = f.session.importPdf(bytes, {}, { signal: controller.signal });
  const late = f.workers[1].onmessage;
  controller.abort();
  late({ data: { id: 1, result: { pageCount: 99 } } });
  await assert.rejects(pending, rejected('PDF_IMPORT_CANCELLED'));
  assert.equal(f.workers[1].terminated, true);
  assert.equal(f.workers[0].terminated, false);
  const next = f.session.importPdf(bytes);
  assert.equal(f.workers.length, 3);
  f.workers[2].reply({ pageCount: 2 });
  await next;
  f.session.dispose();
});

test('deadline and fatal Worker errors reject candidate and preserve active', async () => {
  const f = fixture();
  await active(f);
  await assert.rejects(f.session.importPdf(bytes, {}, { timeoutMs: 5 }), rejected('PDF_IMPORT_TIMEOUT'));
  assert.equal(f.workers[1].terminated, true);
  const pending = f.session.importPdf(bytes);
  f.workers[2].reply(null, { code: 'PDF_WORKER_FAILED', message: 'trap', fatal: true });
  await assert.rejects(pending, rejected('PDF_WORKER_FAILED'));
  assert.equal(f.workers[2].terminated, true);
  assert.equal(f.workers[0].terminated, false);
  f.session.dispose();
});

test('pre-abort, invalid deadline, concurrent import and dispose cannot publish a candidate', async () => {
  const f = fixture();
  const abort = new AbortController(); abort.abort();
  await assert.rejects(f.session.importPdf(bytes, {}, { signal: abort.signal }), rejected('PDF_IMPORT_CANCELLED'));
  assert.equal(f.workers[0].messages.length, 0);
  await assert.rejects(f.session.importPdf(bytes, {}, { timeoutMs: NaN }), rejected('PDF_DEADLINE_INVALID'));
  const pending = f.session.importPdf(bytes);
  await assert.rejects(f.session.importPdf(bytes), rejected('PDF_IMPORT_BUSY'));
  f.session.dispose();
  await assert.rejects(pending, rejected('PDF_SESSION_DISPOSED'));
  await assert.rejects(f.session.importPdf(bytes), rejected('PDF_SESSION_DISPOSED'));
});

test('clone errors, worker load errors and missing document have explicit failures', async () => {
  const f = fixture();
  await assert.rejects(f.session.inspect(), rejected('PDF_NO_DOCUMENT'));
  const first = f.session.importPdf(bytes);
  f.workers[0].onerror({});
  await assert.rejects(first, rejected('PDF_WORKER_FAILED'));
  const second = f.session.importPdf(bytes);
  f.workers[1].reply({ pageCount: 1 });
  await second;
  f.workers[1].postMessage = () => { throw new Error('clone'); };
  await assert.rejects(f.session.renderPageSvg(0), rejected('PDF_WORKER_PROTOCOL'));
  f.session.dispose();
});
