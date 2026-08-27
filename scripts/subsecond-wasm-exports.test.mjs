import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const artifact = process.env.RHWP_SUBSECOND_WASM
  ?? 'target/wasm32-unknown-unknown/debug/rhwp-subsecond.wasm';

test('the built Subsecond module exports its complete trace lifecycle', () => {
  const module = new WebAssembly.Module(readFileSync(artifact));
  const exports = new Map(WebAssembly.Module.exports(module).map(entry => [entry.name, entry.kind]));
  for (const name of [
    'beginSubsecondTrace',
    'activateSubsecondTrace',
    'deactivateSubsecondTrace',
    'endSubsecondTrace',
  ]) assert.equal(exports.get(name), 'function', `${name} is not a function in ${artifact}`);
});
