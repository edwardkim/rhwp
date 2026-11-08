import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { EventEmitter } from 'node:events';
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { observeWasmArtifact } from '../e2e/wasm-artifact-check.mjs';

const bytes = Buffer.from('fresh wasm bytes');
const sha256 = createHash('sha256').update(bytes).digest('hex');

for (const mode of ['match', 'stale', 'missing', 'failed-response', 'unreadable', 'mixed']) {
  test(`browser WASM provenance: ${mode}`, async () => {
    const directory = mkdtempSync(path.join(tmpdir(), 'rhwp-wasm-provenance-'));
    try {
      const manifest = path.join(directory, 'build.json');
      writeFileSync(manifest, JSON.stringify({ success: true, source_sha: 'test-sha',
        profile: 'release', artifacts: { 'rhwp_bg.wasm': { sha256 } } }));
      const session = Object.assign(new EventEmitter(), {
        send: async (method, params) => {
          if (method === 'Network.enable') {
            assert.ok(params.maxResourceBufferSize >= 64 * 1024 * 1024);
            return {};
          }
          if (mode === 'unreadable') throw new Error('body unavailable');
          const stale = mode === 'stale' || params.requestId === 'stale';
          return { body: (stale ? Buffer.from('old wasm') : bytes).toString('base64'), base64Encoded: true };
        },
        detach: async () => session.removeAllListeners(),
      });
      const page = { createCDPSession: async () => session, url: () => 'http://localhost:7700/' };
      const observer = await observeWasmArtifact(page, manifest);
      const response = (requestId) => ({ requestId, response: {
        url: 'http://localhost:7700/@fs/pkg/rhwp_bg.wasm?import',
        status: mode === 'failed-response' ? 404 : 200,
      } });
      if (mode !== 'missing') session.emit('Network.responseReceived', response('fresh'));
      if (mode === 'mixed') session.emit('Network.responseReceived', response('stale'));
      if (mode === 'match') await observer.finish();
      else await assert.rejects(observer.finish(), /mismatch or missing/);
      assert.equal(session.listenerCount('Network.responseReceived'), 0);
      const records = readdirSync(path.join(directory, 'consumption'));
      assert.equal(records.length, 1);
      const proof = JSON.parse(readFileSync(path.join(directory, 'consumption', records[0]), 'utf8'));
      assert.equal(proof.matched, mode === 'match');
      assert.equal(proof.expected_sha256, sha256);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });
}

test('local checks without a manifest remain opt-in; invalid manifests fail closed', async () => {
  const page = new EventEmitter();
  await (await observeWasmArtifact(page, '')).finish();
  assert.equal(page.listenerCount('response'), 0);
  const directory = mkdtempSync(path.join(tmpdir(), 'rhwp-wasm-manifest-'));
  try {
    const manifest = path.join(directory, 'build.json');
    for (const contents of [{}, { success: false }, { success: true, artifacts: {} }]) {
      writeFileSync(manifest, JSON.stringify(contents));
      await assert.rejects(observeWasmArtifact(page, manifest), /incomplete or unsuccessful/);
    }
    writeFileSync(manifest, '{invalid');
    await assert.rejects(observeWasmArtifact(page, manifest), SyntaxError);
    await assert.rejects(observeWasmArtifact(page, path.join(directory, 'missing.json')), /ENOENT/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
