import assert from 'node:assert/strict';
import test from 'node:test';
import { RhwpEditor } from '../index.js';
import { EditorTransport } from '../transport.js';

const request = {
  schemaVersion: 1, format: 'hwpx', password: 'fixture-secret-only', expectedDocumentEpoch: 2, expectedChangeSeq: 3,
};
const receipt = {
  schemaVersion: 1, exportId: 'a'.repeat(32), format: 'hwpx',
  protection: 'password-protected', documentEpoch: 2, changeSeq: 3,
};
const artifact = {
  ...receipt, bytes: new Uint8Array([1, 2]), byteCount: 2,
  contentLoss: { schemaVersion: 1, outputFormat: 'hwpx', count: 0, losses: [] },
};

test('new SDK keeps password RPCs off an old Studio and the legacy fallback', async t => {
  for (const mode of ['old', 'legacy']) {
    const requests = [];
    const transport = new EditorTransport({
      contentWindow: {
        postMessage(data, _origin, [port]) {
          requests.push(data);
          t.after(() => port?.close());
          if (mode === 'old') {
            port.onmessage = ({ data: request }) => { requests.push(request); };
            port.postMessage({
              type: 'rhwp-connected', version: 1, sessionId: data.sessionId,
              capabilities: ['transferable-array-buffer'],
            });
          }
        },
      },
    }, 'https://studio.example', {
      window: { addEventListener() {}, removeEventListener() {} }, handshakeTimeoutMs: 100,
    });
    t.after(() => transport.destroy());
    await transport.connect();
    const editor = new RhwpEditor({}, transport);
    await assert.rejects(editor.getSaveProtectionState(), { code: 'UNSUPPORTED_CAPABILITY' });
    await assert.rejects(editor.exportPasswordProtectedDocument(request), { code: 'UNSUPPORTED_CAPABILITY' });
    await assert.rejects(editor.acknowledgeHostSave(receipt), { code: 'UNSUPPORTED_CAPABILITY' });
    await assert.rejects(transport.request('exportPasswordProtectedDocument', request), { code: 'UNSUPPORTED_CAPABILITY' });
    assert.deepEqual(requests.map(data => data.type), ['rhwp-connect']);
    assert.ok(requests[0].capabilities.includes('password-save-v1'));
    assert.ok(!JSON.stringify(requests).includes(request.password));
  }
});

test('SDK validates password-save responses and does not forward peer error text', async () => {
  const unsent = Symbol('unsent');
  let result = unsent;
  let count = 0;
  const editor = new RhwpEditor({}, {
    supports: capability => capability === 'password-save-v1',
    request: async (method, params) => {
      count++;
      if (result === unsent) throw Object.assign(new Error(request.password), { code: 'INVALID_REQUEST' });
      if (method === 'exportPasswordProtectedDocument') assert.deepEqual(params, request);
      return result;
    },
  });
  await assert.rejects(editor.exportPasswordProtectedDocument({ ...request, password: 12 }),
    { code: 'INVALID_REQUEST' });
  assert.equal(count, 0);
  await assert.rejects(editor.exportPasswordProtectedDocument(request), {
    code: 'INVALID_REQUEST', message: 'Invalid password-save request.',
  });
  for (const malformed of [
    { byteCount: 3 }, { changeSeq: 4 }, { format: 'hwp' }, { bytes: [1, 2] }, { password: request.password },
    { contentLoss: { ...artifact.contentLoss, outputFormat: 'hwp' } },
    { contentLoss: { ...artifact.contentLoss, count: 1, losses: [{ code: 'unknown' }] } },
    { contentLoss: { ...artifact.contentLoss, extra: true } },
  ]) {
    result = { ...artifact, ...malformed };
    await assert.rejects(editor.exportPasswordProtectedDocument(request), { code: 'INVALID_RESPONSE' });
  }
  result = artifact;
  assert.deepEqual(await editor.exportPasswordProtectedDocument(request), artifact);
  result = { schemaVersion: 1, persistedProtection: 'password-protected', loadedFormat: 'hwpx',
    documentEpoch: 2, changeSeq: 3, requiresPasswordForSave: false };
  await assert.rejects(editor.getSaveProtectionState(), { code: 'INVALID_RESPONSE' });
  result = { ...result, requiresPasswordForSave: true };
  assert.deepEqual(await editor.getSaveProtectionState(), result);
  result = { ok: true, wasDirty: 'yes' };
  await assert.rejects(editor.acknowledgeHostSave(receipt), { code: 'INVALID_RESPONSE' });
  result = { ok: true, wasDirty: true };
  assert.deepEqual(await editor.acknowledgeHostSave(receipt), result);
});
