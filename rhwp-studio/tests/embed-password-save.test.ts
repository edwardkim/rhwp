import assert from 'node:assert/strict';
import { once } from 'node:events';
import test, { type TestContext } from 'node:test';
import type { PasswordProtectedDocumentRequest } from '../../npm/editor/index.js';
import { HostPasswordSave } from '../src/embed/host-password-save.ts';
import type { EmbedRpcHandlers } from '../src/embed/rpc-router.ts';
import { installEmbedRuntime } from '../src/embed/runtime.ts';

const secret = 'fixture-secret-only';
const request: PasswordProtectedDocumentRequest = {
  schemaVersion: 1, format: 'hwp', password: secret, expectedDocumentEpoch: 2, expectedChangeSeq: 3,
};
const methods = ['getSaveProtectionState', 'exportPasswordProtectedDocument', 'acknowledgeHostSave'];

function fixture() {
  const revision = { documentEpoch: 2, changeSeq: 3 };
  const bytes = new Uint8Array([7, 8, 9]);
  const calls: string[] = [];
  let mutateDuringExport = () => {};
  const state = { dirty: true, draft: true, fileName: 'original.hwp' };
  function artifact(format: 'hwp' | 'hwpx') {
    calls.push(format);
    mutateDuringExport();
    return {
      bytes,
      contentLoss: {
        schemaVersion: 1 as const, outputFormat: format, count: 1,
        losses: [{ code: 'controlOmitted' as const, subject: 'control' as const,
          path: 'sections[0]', reason: 'unsupportedByOutputFormat' as const }],
      },
    };
  }
  const wasm = {
    getLoadedDocumentFormat: () => 'hwp5',
    requiresPasswordForSave: false,
    exportHwpWithPasswordAndReport: () => artifact('hwp'),
    exportHwpxWithPasswordAndReport: () => artifact('hwpx'),
  };
  const save = new HostPasswordSave({
    wasm,
    revision: () => ({ ...revision }),
    async completeHostSave(fileName) {
      const wasDirty = state.dirty;
      state.dirty = state.draft = false;
      if (fileName) state.fileName = fileName;
      return { ok: true, wasDirty };
    },
  });
  const handlers = {
    passwordSave: {
      async getState() { return save.getState(); },
      async export(params) { return save.export(params); },
      async acknowledge(params) { return save.acknowledge(params); },
    },
  } as EmbedRpcHandlers;
  return { save, wasm, state, revision, calls, bytes, handlers,
    beforeArtifact: (callback: () => void) => { mutateDuringExport = callback; } };
}

function install(t: TestContext, handlers: EmbedRpcHandlers, topLevel = false) {
  let listener: (event: MessageEvent) => void;
  const replies: Array<{ error: { code: string; message: string } }> = [];
  const host = {
    addEventListener(_type: string, next: (event: MessageEvent) => void) { listener = next; },
    removeEventListener() {},
    postMessage(response: typeof replies[number]) { replies.push(response); },
  };
  const parent = topLevel ? host : { postMessage: host.postMessage };
  const cleanup = installEmbedRuntime({
    hostWindow: host as unknown as Window, parentWindow: parent as unknown as Window, handlers,
  });
  t.after(cleanup);
  const dispatch = (data: unknown, origin: string, ports: MessagePort[] = [], source = parent) => {
    listener({ data, origin, ports, source } as unknown as MessageEvent);
  };
  return { dispatch, replies };
}

async function connect(t: TestContext, runtime: ReturnType<typeof install>,
  offered: string[], origin = 'https://host.example') {
  const channel = new MessageChannel();
  t.after(() => channel.port1.close());
  const transfers: Array<{ embedded: boolean; transferred: boolean }> = [];
  const post = channel.port2.postMessage.bind(channel.port2);
  channel.port2.postMessage = (message, transfer = []) => {
    if (message.result?.bytes) {
      transfers.push({ embedded: true, transferred: transfer[0] === message.result.bytes.buffer });
    }
    post(message, transfer);
  };
  const handshake = once(channel.port1, 'message');
  runtime.dispatch({
    type: 'rhwp-connect', version: 1, sessionId: 'save-session',
    capabilities: ['transferable-array-buffer', ...offered],
  }, origin, [channel.port2]);
  const [connected] = await handshake;
  let id = 0;
  const rpc = async (method: string, params: unknown = {}) => {
    const reply = once(channel.port1, 'message');
    channel.port1.postMessage({ type: 'rhwp-request', version: 1, sessionId: 'save-session',
      id: ++id, method, params });
    return (await reply)[0];
  };
  return { connected, rpc, transfers };
}

test('password-save-v1 stays off an unnegotiated port and all legacy requests', { timeout: 3000 }, async t => {
  const { handlers, calls } = fixture();
  const runtime = install(t, handlers);
  const { connected, rpc } = await connect(t, runtime, []);
  assert.ok(!connected.capabilities.includes('password-save-v1'));
  for (const method of methods) {
    assert.equal((await rpc(method, request)).error.code, 'UNSUPPORTED_CAPABILITY');
    runtime.dispatch({ type: 'rhwp-request', id: 5, method, params: request }, 'https://host.example');
    assert.equal(runtime.replies.at(-1)?.error.code, 'UNSUPPORTED_TRANSPORT');
  }
  assert.deepEqual(calls, []);
  assert.ok(!JSON.stringify(runtime.replies).includes(secret));
});

test('only same-window identity allows a custom-scheme MessagePort', { timeout: 3000 }, async t => {
  const { handlers } = fixture();
  const embedded = install(t, handlers);
  const blockedPort = new MessageChannel();
  t.after(() => blockedPort.port1.close());
  const closed = once(blockedPort.port1, 'close');
  // A close listener alone does not keep Node's event loop alive.
  blockedPort.port1.ref();
  embedded.dispatch({ type: 'rhwp-connect', version: 1, sessionId: 'opaque',
    capabilities: ['transferable-array-buffer', 'password-save-v1'] }, 'null', [blockedPort.port2]);
  await closed;

  const top = install(t, handlers, true);
  const { connected, rpc } = await connect(t, top, ['password-save-v1'], 'null');
  assert.ok(connected.capabilities.includes('password-save-v1'));
  assert.equal((await rpc('getSaveProtectionState')).result.loadedFormat, 'hwp5');
});

test('validate before serializing and transfer a copy of the reported artifact', { timeout: 3000 }, async t => {
  const { handlers, calls, bytes, revision, state, wasm } = fixture();
  const { rpc, transfers } = await connect(t, install(t, handlers), ['password-save-v1']);
  for (const malformed of [
    { schemaVersion: 2 }, { format: 'hwp3' }, { password: 123 }, { password: 'abcd' },
    { password: '한'.repeat(1366) }, { expectedDocumentEpoch: -1 }, { expectedChangeSeq: 1.5 },
    { [secret]: true },
  ]) {
    const result = await rpc('exportPasswordProtectedDocument', { ...request, ...malformed });
    assert.equal(result.error.code, 'INVALID_REQUEST');
    assert.ok(!JSON.stringify(result).includes(secret));
  }
  assert.equal((await rpc('getSaveProtectionState', [])).error.code, 'INVALID_REQUEST');
  for (const mismatch of [{ expectedChangeSeq: 2 }, { expectedDocumentEpoch: 1 }]) {
    assert.equal((await rpc('exportPasswordProtectedDocument', { ...request, ...mismatch })).error.code,
      'STALE_SAVE_REVISION');
  }
  assert.deepEqual(calls, []);
  const { result } = await rpc('exportPasswordProtectedDocument', request);
  assert.deepEqual(calls, ['hwp']);
  assert.deepEqual(transfers, [{ embedded: true, transferred: true }]);
  assert.deepEqual(result.bytes, bytes);
  assert.equal(result.byteCount, bytes.length);
  assert.equal(result.contentLoss.losses[0].path, 'sections[0]');
  assert.ok(!JSON.stringify(result).includes(secret));
  const { bytes: _bytes, byteCount: _byteCount, contentLoss: _contentLoss, ...receipt } = result;
  assert.ok(state.dirty && state.draft && !wasm.requiresPasswordForSave);
  assert.equal((await rpc('acknowledgeHostSave', { ...receipt, format: 'hwpx' })).error.code,
    'INVALID_SAVE_EXPORT');
  assert.equal((await rpc('acknowledgeHostSave', { ...receipt, fileName: 12 })).error.code,
    'INVALID_REQUEST');
  revision.changeSeq++;
  assert.equal((await rpc('acknowledgeHostSave', receipt)).error.code, 'STALE_SAVE_REVISION');
  assert.ok(state.dirty && state.draft && !wasm.requiresPasswordForSave);
});

test('an edit during export and document replacement invalidate the receipt', { timeout: 3000 }, async t => {
  const { handlers, state, wasm, revision, calls, beforeArtifact } = fixture();
  const { rpc } = await connect(t, install(t, handlers), ['password-save-v1']);
  beforeArtifact(() => { revision.changeSeq += 1; });
  const disturbed = await rpc('exportPasswordProtectedDocument', request);
  assert.equal(disturbed.error.code, 'STALE_SAVE_REVISION');
  assert.deepEqual(calls, ['hwp']);
  assert.ok(state.dirty && state.draft && !wasm.requiresPasswordForSave);

  beforeArtifact(() => { throw new Error(secret); });
  const failure = await rpc('exportPasswordProtectedDocument', { ...request, expectedChangeSeq: 4 });
  assert.deepEqual(failure.error, {
    code: 'PASSWORD_SAVE_FAILED', message: 'The password-save operation failed.',
  });
  beforeArtifact(() => {});
  const { result } = await rpc('exportPasswordProtectedDocument', { ...request, format: 'hwpx', expectedChangeSeq: 4 });
  const { bytes: _bytes, byteCount: _byteCount, contentLoss, ...receipt } = result;
  assert.equal(contentLoss.outputFormat, 'hwpx');
  revision.documentEpoch++;
  assert.equal((await rpc('acknowledgeHostSave', receipt)).error.code, 'STALE_SAVE_REVISION');
  assert.ok(state.dirty && state.draft && !wasm.requiresPasswordForSave);
});

test('a matching one-use acknowledgment publishes protection and clears the saved revision', async () => {
  const { save, state, wasm } = fixture();
  const { bytes: _bytes, byteCount: _byteCount, contentLoss: _contentLoss, ...older } = save.export(request);
  const { bytes: _other, byteCount: _count, contentLoss: _report, ...current } = save.export(request);
  assert.throws(() => save.acknowledge(older), { code: 'INVALID_SAVE_EXPORT' });
  assert.ok(state.dirty && state.draft && !wasm.requiresPasswordForSave);
  assert.deepEqual(await save.acknowledge({ ...current, fileName: 'durable.hwp' }), { ok: true, wasDirty: true });
  assert.equal(state.fileName, 'durable.hwp');
  assert.ok(!state.dirty && !state.draft && wasm.requiresPasswordForSave);
  assert.equal(save.getState().persistedProtection, 'password-protected');
  assert.throws(() => save.acknowledge(current), { code: 'INVALID_SAVE_EXPORT' });
});
