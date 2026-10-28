import assert from 'node:assert/strict';
import test from 'node:test';

import { probeEmbedding } from '../kerning_r4e_wasm_embedding_probe.mjs';

const inputs = {
  noto: Buffer.from('noto-font-payload'),
  smoke: Buffer.from('smoke-font-payload'),
  fixture: Buffer.from('runtime-fixture-payload'),
};

test('embedding probe accepts a code-only WASM payload', () => {
  const result = probeEmbedding(Buffer.from('wasm-code-only'), inputs);
  assert.equal(result.status, 'pass');
  assert.equal(result.probeCount, 9);
  assert.ok(result.probes.every((probe) => !probe.present));
});

test('embedding probe rejects tracked payload and private identity markers', () => {
  assert.throws(
    () => probeEmbedding(Buffer.concat([Buffer.from('wasm'), inputs.smoke]), inputs),
    /full-smoke-face-bytes/,
  );
  assert.throws(
    () => probeEmbedding(Buffer.from('prefix /home/edward/private suffix'), inputs),
    /private-home-marker/,
  );
});
