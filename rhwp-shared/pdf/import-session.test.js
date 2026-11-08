import test from 'node:test';
import assert from 'node:assert/strict';
import { PdfImportSession } from './import-session.js';

const policy = { schemaVersion: 1, backend: { inputBytes: 16 * 1024 * 1024 } };
const ready = () => new Promise(resolve => setImmediate(resolve));

function fixture(productPolicy = policy, limits = { outputBytes: 128 * 1024 * 1024, diagnosticsBytes: 32 * 1024 * 1024 }) {
  const workers = [];
  const session = new PdfImportSession({ workerFactory: () => {
    const worker = {
      messages: [], terminated: false,
      postMessage(message) {
        this.messages.push(message);
        if (message.method === 'policy') queueMicrotask(() => this.reply(typeof productPolicy === 'function' ? productPolicy(message.options) : productPolicy));
        if (message.method === 'reserveInput') queueMicrotask(() => this.reply(true));
      },
      terminate() { this.terminated = true; },
      reply(result, error) {
        const message = this.messages.at(-1);
        const json = !error && ['importPdf', 'inspect'].includes(message.method);
        if (message.method === 'policy' && result?.backend) {
          result = { ...result, backend: { decodedBytes: 2 ** 31, ...result.backend } };
        }
        this.onmessage?.({ data: { id: message.id, error, remainingDecodedBytes: 2 ** 30,
          result: json ? JSON.stringify({ diagnostics: {}, ...result }) : result,
          ...(json ? { encoding: 'json' } : {}),
          ...(message.method === 'policy' ? { limits } : {}),
        } });
      },
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
  await ready();
  assert.notEqual(f.workers[0].messages.at(-1).bytes.buffer, bytes.buffer);
  f.workers[0].reply({ pageCount: 3 });
  await loaded;
}

test('failed candidate leaves active session usable; next import reuses candidate and replaces only on success', async () => {
  const f = fixture();
  await active(f);
  const failed = f.session.importPdf(bytes);
  await ready();
  f.workers[1].reply(null, { code: 'PDF_PARSE_FAILED', message: 'invalid PDF' });
  await assert.rejects(failed, rejected('PDF_PARSE_FAILED'));
  const inspect = f.session.inspect();
  f.workers[0].reply({ pageCount: 3 });
  assert.deepEqual(await inspect, { pageCount: 3, diagnostics: {} });
  assert.equal(f.workers[0].terminated, false);
  const next = f.session.importPdf(bytes, { sourcePages: [1] });
  await ready();
  assert.equal(f.workers.length, 2);
  f.workers[1].reply({ pageCount: 1 });
  await next;
  assert.equal(f.workers[0].terminated, true);
  assert.equal(f.workers[1].terminated, false);
  assert.equal(bytes.byteLength, 5);
  f.session.dispose();
});

test('product policy is read before copying input; views copy only visible bytes', async () => {
  const f = fixture();
  await assert.rejects(f.session.importPdf(new Uint8Array(16 * 1024 * 1024 + 1)), rejected('PDF_RESOURCE_LIMIT'));
  assert.equal(f.workers.length, 1);
  assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy']);
  const backing = new Uint8Array(4096);
  backing.set(bytes, 100);
  const loaded = f.session.importPdf(backing.subarray(100, 105));
  await ready();
  assert.equal(f.workers[0].messages.at(-1).bytes.buffer.byteLength, 5);
  assert.deepEqual(f.workers[0].messages.at(-1).bytes, bytes);
  assert.equal(f.workers[0].messages.filter(m => m.method === 'policy').length, 2);
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
  await ready();
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
  await ready();
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
  assert.equal(f.workers.length, 0);
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
  await ready();
  f.workers[1].reply({ pageCount: 1 });
  await second;
  f.workers[1].postMessage = () => { throw new Error('clone'); };
  await assert.rejects(f.session.renderPageSvg(0), rejected('PDF_WORKER_PROTOCOL'));
  f.session.dispose();
});

test('a lower product input ceiling is enforced without transferring candidate bytes', async () => {
  const f = fixture({ schemaVersion: 1, backend: { inputBytes: bytes.length - 1 } });
  await assert.rejects(f.session.importPdf(bytes), rejected('PDF_RESOURCE_LIMIT'));
  assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy']);
  f.session.dispose();
});

test('malformed product policies fail closed before transferring input', async () => {
  for (const productPolicy of [null, {}, { schemaVersion: 2, backend: { inputBytes: 1024 } },
    { schemaVersion: 1, backend: { inputBytes: 0 } },
    { schemaVersion: 1, backend: { inputBytes: Number.MAX_SAFE_INTEGER + 1 } }]) {
    const f = fixture(productPolicy);
    await assert.rejects(f.session.importPdf(bytes), rejected('PDF_WORKER_PROTOCOL'));
    assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy']);
    assert.equal(f.workers[0].terminated, true);
    f.session.dispose();
  }
});

test('a higher product ceiling does not retain the former JavaScript 16 MiB limit', async () => {
  const input = new Uint8Array(16 * 1024 * 1024 + 1);
  const f = fixture({ schemaVersion: 1, backend: { inputBytes: input.length } });
  const pending = f.session.importPdf(input);
  await ready();
  assert.equal(f.workers[0].messages.at(-1).method, 'importPdf');
  assert.equal(f.workers[0].messages.at(-1).bytes.length, input.length);
  f.workers[0].reply({ pageCount: 1 });
  await pending;
  assert.equal(input.length, 16 * 1024 * 1024 + 1);
  f.session.dispose();
});

test('each request resolves its own profile and lower caps before transfer on a reused candidate', async () => {
  const f = fixture(options => ({ schemaVersion: 1, backend: { inputBytes: options.resourceLimits.backend.inputBytes } }));
  await assert.rejects(f.session.importPdf(bytes, { resourceProfile: 'constrained', resourceLimits: { backend: { inputBytes: 4 } } }), rejected('PDF_RESOURCE_LIMIT'));
  assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy']);
  const next = f.session.importPdf(bytes, { resourceProfile: 'standard', resourceLimits: { backend: { inputBytes: 5 } } });
  await ready();
  assert.equal(f.workers.length, 1);
  assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy', 'policy', 'reserveInput', 'importPdf']);
  assert.equal(f.workers[0].messages.at(-1).options.resourceLimits.backend.inputBytes, 5);
  f.workers[0].reply({ pageCount: 1 }); await next; f.session.dispose();
});

test('caller mutations cannot change options between policy lookup and import', async () => {
  const f = fixture();
  const options = { sourcePages: [0], layout: 'editable', resourceProfile: 'constrained', resourceLimits: { backend: { inputBytes: 8 } } };
  const next = f.session.importPdf(bytes, options);
  options.sourcePages[0] = 99; options.layout = 'fidelity'; options.resourceProfile = 'standard'; options.resourceLimits.backend.inputBytes = 1;
  await ready();
  const messages = f.workers[0].messages;
  const imported = messages.find(message => message.method === 'importPdf');
  assert.deepEqual(messages[0].options, imported.options);
  assert.deepEqual(imported.options, { sourcePages: [0], layout: 'editable', resourceProfile: 'constrained', resourceLimits: { backend: { inputBytes: 8 } } });
  f.workers[0].reply({ pageCount: 1 }); await next; f.session.dispose();
});

test('memory failure retires only candidate and a fresh worker can recover', async () => {
  const f = fixture(); await active(f);
  const pending = f.session.importPdf(bytes); await ready();
  f.workers[1].reply(null, { code: 'PDF_WORKER_MEMORY', message: 'allocation failed', fatal: true });
  await assert.rejects(pending, rejected('PDF_WORKER_MEMORY'));
  assert.equal(f.workers[0].terminated, false); assert.equal(f.workers[1].terminated, true);
  const next = f.session.importPdf(bytes); await ready();
  assert.equal(f.workers.length, 3);
  f.workers[2].reply({ pageCount: 1 }); await next; f.session.dispose();
});

test('input copy allocation failure preserves active and retires candidate before transfer', async () => {
  const f = fixture(); await active(f);
  const input = bytes.slice(), original = globalThis.Uint8Array;
  globalThis.Uint8Array = new Proxy(original, { construct() { throw new RangeError('allocation failed'); } });
  try {
    await assert.rejects(f.session.importPdf(input), rejected('PDF_WORKER_MEMORY'));
    assert.equal(f.workers[0].terminated, false); assert.equal(f.workers[1].terminated, true);
    assert.deepEqual(f.workers[1].messages.map(m => m.method), ['policy', 'reserveInput']);
  } finally { globalThis.Uint8Array = original; f.session.dispose(); }
});


test('progress notifications cannot publish a candidate and cancellation from a phase callback preserves active', async () => {
  const f = fixture(); await active(f);
  const controller = new AbortController(); const phases = [];
  const pending = f.session.importPdf(bytes, {}, { signal: controller.signal, onProgress: phase => { phases.push(phase); controller.abort(); } });
  await ready();
  const worker = f.workers[1];
  worker.onmessage({ data: { id: worker.messages.at(-1).id, phase: 'importing' } });
  await assert.rejects(pending, rejected('PDF_IMPORT_CANCELLED'));
  assert.deepEqual(phases, ['importing']); assert.equal(worker.terminated, true); assert.equal(f.workers[0].terminated, false);
  f.session.dispose();
});

test('export lower cap reaches the Worker; invalid caps do not send requests', async () => {
  const f = fixture();
  await active(f);
  const count = f.workers[0].messages.length;
  for (const outputBytes of [0, -1, null, 1.5, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    await assert.rejects(f.session.exportDocument('hwpx', { outputBytes }), rejected('PDF_RESOURCE_POLICY'));
  }
  assert.equal(f.workers[0].messages.length, count);
  const exported = f.session.exportDocument('hwpx', { outputBytes: 32 });
  assert.equal(f.workers[0].messages.at(-1).maximumBytes, 32);
  f.workers[0].reply(new Uint8Array([80, 75]));
  assert.deepEqual(await exported, new Uint8Array([80, 75]));
  f.session.dispose();
});


test('unknown or nested option payloads are rejected before cloning or creating a Worker', async () => {
  const original = globalThis.structuredClone;
  let copies = 0;
  globalThis.structuredClone = (...args) => { copies++; return original(...args); };
  try {
    for (const options of [{ layout: 'unknown' }, { layout: {} }, { extra: 'x'.repeat(100000) }, { resourceLimits: { backend: { objects: new Uint8Array(100000) } } }]) {
      const f = fixture();
      await assert.rejects(f.session.importPdf(bytes, options), error => ['PDF_IMPORT_OPTIONS', 'PDF_RESOURCE_POLICY'].includes(error.code));
      assert.equal(f.workers.length, 0);
      f.session.dispose();
    }
    assert.equal(copies, 0);
  } finally { globalThis.structuredClone = original; }
});

test('selection metadata is bounded and accessors are not evaluated before policy transfer', async () => {
  let getters = 0;
  const accessor = [0]; Object.defineProperty(accessor, '0', { get() { getters++; return 0; } });
  for (const sourcePages of [accessor, new Array(1025).fill(0), [0, 0], [Infinity]]) {
    const f = fixture();
    await assert.rejects(f.session.importPdf(bytes, { sourcePages }), rejected('PDF_PAGE_SELECTION'));
    assert.equal(f.workers.length, 0); f.session.dispose();
  }
  assert.equal(getters, 0);
});

test('the product reserves the private input copy before its owner allocates or transfers it', async () => {
  const f = fixture();
  let copies = 0;
  const input = bytes.slice(), original = globalThis.Uint8Array;
  globalThis.Uint8Array = new Proxy(original, { construct(target, args) {
    if (args[0] === input.length) copies++;
    return Reflect.construct(target, args);
  } });
  try {
    const pending = f.session.importPdf(input);
    const worker = f.workers[0]; const post = worker.postMessage;
    worker.postMessage = function(message) {
      if (message.method === 'reserveInput') {
        this.messages.push(message);
        queueMicrotask(() => this.reply(null, { code: 'PDF_RESOURCE_LIMIT', message: 'copy budget', fatal: true }));
      } else post.call(this, message);
    };
    await assert.rejects(pending, rejected('PDF_RESOURCE_LIMIT'));
    assert.equal(copies, 0);
    assert.deepEqual(worker.messages.map(m => m.method), ['policy', 'reserveInput']);
    assert.equal(worker.terminated, true);
  } finally { globalThis.Uint8Array = original; f.session.dispose(); }
});

test('a caller slice override cannot alias the transferred private backing', async () => {
  const f = fixture();
  const input = Buffer.from([1, 2, 3]), expected = Array.from(input);
  let slices = 0; input.slice = () => { slices++; return input; };
  const pending = f.session.importPdf(input); await ready();
  const transfer = f.workers[0].messages.at(-1).bytes;
  assert.notEqual(transfer.buffer, input.buffer);
  assert.deepEqual(Array.from(transfer), expected);
  assert.equal(slices, 0); assert.deepEqual(Array.from(input), expected);
  f.workers[0].reply({ pageCount: 1 }); await pending; f.session.dispose();
});

test('invalid JSON diagnostics cannot publish a candidate or discard the active document', async () => {
  for (const result of ['x'.repeat(129), 'not json', JSON.stringify({ pageCount: 0, diagnostics: {} })]) {
    const f = fixture(policy, { outputBytes: 1024, diagnosticsBytes: 128 }); await active(f);
    const pending = f.session.importPdf(bytes); await ready();
    const worker = f.workers[1];
    worker.onmessage({ data: { id: worker.messages.at(-1).id, result, encoding: 'json', remainingDecodedBytes: 2 ** 30 } });
    await assert.rejects(pending, rejected('PDF_WORKER_PROTOCOL'));
    assert.equal(worker.terminated, true); assert.equal(f.workers[0].terminated, false);
    f.session.dispose();
  }
});

test('render byte caps count UTF-8 without allocating an encoded response copy', async () => {
  const f = fixture(policy, { outputBytes: 4, diagnosticsBytes: 1024 }); await active(f);
  const pending = f.session.renderPageSvg(0);
  f.workers[0].reply('가가');
  await assert.rejects(pending, rejected('PDF_WORKER_PROTOCOL'));
  assert.equal(f.workers[0].terminated, true); f.session.dispose();
});

test('responses delivered after the absolute deadline cannot replace a document before the timer runs', async () => {
  const f = fixture(); await active(f);
  const pending = f.session.importPdf(bytes, {}, { timeoutMs: 100 }); await ready();
  const until = performance.now() + 120;
  while (performance.now() < until) { /* keep the owner event loop occupied */ }
  f.workers[1].reply({ pageCount: 1 });
  await assert.rejects(pending, rejected('PDF_IMPORT_TIMEOUT'));
  assert.equal(f.workers[1].terminated, true); assert.equal(f.workers[0].terminated, false); f.session.dispose();
});

test('a transport failure retires the uncertain Worker and a new import recovers', async () => {
  const f = fixture(); await active(f);
  f.workers[0].postMessage = () => { throw new Error('transfer failed'); };
  await assert.rejects(f.session.renderPageSvg(0), rejected('PDF_WORKER_PROTOCOL'));
  assert.equal(f.workers[0].terminated, true);
  const next = f.session.importPdf(bytes); await ready();
  f.workers[1].reply({ pageCount: 1 }); await next;
  f.session.dispose();
});


test('a view detached while policy is pending cannot be copied or transferred as a different input', async () => {
  const f = fixture(), input = new Uint8Array([1, 2, 3]);
  const pending = f.session.importPdf(input);
  structuredClone(input.buffer, { transfer: [input.buffer] });
  await assert.rejects(pending, rejected('PDF_INVALID_INPUT'));
  assert.deepEqual(f.workers[0].messages.map(m => m.method), ['policy']);
  f.session.dispose();
});

test('diagnostic decoding cannot fulfill a request after its deadline', async () => {
  const f = fixture(); await active(f);
  const original = JSON.parse;
  try {
    const pending = f.session.inspect({ timeoutMs: 100 });
    JSON.parse = (...args) => {
      const until = performance.now() + 120;
      while (performance.now() < until) { /* model bounded but expensive decoding */ }
      return original(...args);
    };
    f.workers[0].reply({ pageCount: 3 });
    await assert.rejects(pending, rejected('PDF_IMPORT_TIMEOUT'));
    assert.equal(f.workers[0].terminated, true);
  } finally { JSON.parse = original; f.session.dispose(); }
});

test('opening password is snapshotted, sent to worker and omitted from retained reopen options', async () => {
  const f = fixture();
  const options = { password: 'synthetic opening password' };
  const imported = f.session.importPdf(bytes, options);
  options.password = 'changed by caller';
  await ready();
  assert.equal(f.workers[0].messages.at(-1).options.password, 'synthetic opening password');
  f.workers[0].reply({ pageCount: 1 }); await imported;
  assert.equal(f.session._active.options.password, undefined);
  assert.equal(JSON.stringify(f.session.getEditorReopenLimits()).includes('synthetic opening password'), false);
  f.session.dispose();
});

test('password errors allow retry and invalid credential metadata fails before worker creation', async () => {
  for (const password of [null, 17, 'x'.repeat(1025), '가'.repeat(342)]) {
    const f = fixture();
    await assert.rejects(f.session.importPdf(bytes, { password }), rejected('PDF_IMPORT_OPTIONS'));
    assert.equal(f.workers.length, 0); f.session.dispose();
  }
  const f = fixture();
  const first = f.session.importPdf(bytes); await ready();
  f.workers[0].reply(null, { code: 'PDF_PASSWORD_REQUIRED', message: 'opening password required' });
  await assert.rejects(first, rejected('PDF_PASSWORD_REQUIRED'));
  const second = f.session.importPdf(bytes, { password: 'wrong' }); await ready();
  f.workers[0].reply(null, { code: 'PDF_PASSWORD_INVALID', message: 'incorrect opening password' });
  await assert.rejects(second, rejected('PDF_PASSWORD_INVALID'));
  const third = f.session.importPdf(bytes, { password: 'correct' }); await ready();
  f.workers[0].reply({ pageCount: 1 }); await third;
  assert.equal(f.session._active.options.password, undefined); f.session.dispose();
});

test('actual page progress reaches observers without completing the request', async () => {
  const f = fixture(), updates = [];
  const pending = f.session.importPdf(bytes, {}, {onProgress: (phase, p) => updates.push([phase, p])});
  await ready(); const worker = f.workers[0], id = worker.messages.at(-1).id;
  for (const [sourcePage, completedPages] of [[2, 0], [2, 1], [5, 1], [5, 2]]) {
    worker.onmessage({data: {id, phase: 'importing', progress: {sourcePage, sourcePageCount: 5, completedPages, totalPages: 2}}});
  }
  assert.deepEqual(updates.map(([, p]) => p.completedPages), [0, 1, 1, 2]);
  assert.equal(f.session._active, null);
  worker.reply({pageCount: 2}); await pending;
  f.session.dispose();
});

test('invalid or regressing page progress retires the worker', async () => {
  for (const change of [{completedPages: 3}, {totalPages: 0}, {sourcePage: 6}, {completedPages: NaN}, {completedPages: 0}]) {
    const f = fixture(); const pending = f.session.importPdf(bytes);
    const rejectedResult = assert.rejects(pending, rejected('PDF_WORKER_PROTOCOL'));
    await ready(); const worker = f.workers[0], id = worker.messages.at(-1).id;
    const progress = {sourcePage: 2, sourcePageCount: 5, completedPages: 1, totalPages: 2};
    worker.onmessage({data: {id, phase: 'importing', progress}});
    worker.onmessage({data: {id, phase: 'importing', progress: {...progress, ...change}}});
    await rejectedResult; assert.equal(worker.terminated, true); f.session.dispose();
  }
});

test('remaining work above 4 GiB reaches editor reopen without 32-bit truncation', async () => {
  const f = fixture({ ...policy, backend: { ...policy.backend, decodedBytes: 8 * 2 ** 30 } });
  const pending = f.session.importPdf(bytes); await ready();
  const worker = f.workers[0];
  worker.onmessage({ data: {
    id: worker.messages.at(-1).id, encoding: 'json',
    result: JSON.stringify({ pageCount: 1, diagnostics: {} }),
    remainingDecodedBytes: 5 * 2 ** 30 + 17,
  } });
  await pending;
  assert.equal(f.session.getEditorReopenLimits().decodedBytes, 5 * 2 ** 30 + 17);
  const inspect = f.session.inspect();
  worker.onmessage({ data: {
    id: worker.messages.at(-1).id, encoding: 'json',
    result: JSON.stringify({ pageCount: 1, diagnostics: {} }),
    remainingDecodedBytes: 8 * 2 ** 30 + 1,
  } });
  await assert.rejects(inspect, rejected('PDF_WORKER_PROTOCOL'));
  f.session.dispose();
});
