// CI artifact contract: no document coordinates or rendering baseline expectations.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { verifyCanvasWasm } from '../frontend-canvas-gates.mjs';

function fixture() {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-canvas-artifact-'));
  mkdirSync(path.join(root, 'pkg'));
  const artifacts = {};
  for (const [name, bytes] of [['rhwp.js', 'test loader'], ['rhwp_bg.wasm', 'test wasm']]) {
    writeFileSync(path.join(root, 'pkg', name), bytes);
    artifacts[name] = { sha256: createHash('sha256').update(bytes).digest('hex') };
  }
  const manifestPath = path.join(root, 'build.json');
  const manifest = { success: true, profile: 'release', build_configuration: { build_mode: 'standard' },
    tool_commands: { 'wasm-opt': ['wasm-opt', '-Oz'] }, artifacts };
  const save = () => writeFileSync(manifestPath, JSON.stringify(manifest));
  save();
  return { root, manifestPath, manifest, save, cleanup: () => rmSync(root, { recursive: true, force: true }) };
}

test('accepts the exact successful optimized JS/WASM pair', () => {
  const f = fixture();
  try { assert.deepEqual(verifyCanvasWasm(f.manifestPath, f.root), f.manifest); }
  finally { f.cleanup(); }
});

test('rejects dev, failed and unoptimized artifacts before starting the browser', () => {
  for (const change of [m => { m.profile = 'dev'; }, m => { m.success = false; },
    m => { m.build_configuration.build_mode = 'cdylib'; },
    m => { delete m.tool_commands; }, m => { m.tool_commands['wasm-opt'] = []; }]) {
    const f = fixture();
    try {
      change(f.manifest); f.save();
      assert.throws(() => verifyCanvasWasm(f.manifestPath, f.root), /optimized release/);
    } finally { f.cleanup(); }
  }
});

test('rejects stale loader and stale WASM independently', () => {
  for (const name of ['rhwp.js', 'rhwp_bg.wasm']) {
    const f = fixture();
    try {
      writeFileSync(path.join(f.root, 'pkg', name), 'different bytes');
      assert.throws(() => verifyCanvasWasm(f.manifestPath, f.root), /artifact mismatch/);
    } finally { f.cleanup(); }
  }
});

test('rejects an absent artifact digest', () => {
  const f = fixture();
  try {
    delete f.manifest.artifacts['rhwp.js']; f.save();
    assert.throws(() => verifyCanvasWasm(f.manifestPath, f.root), /artifact mismatch/);
  } finally { f.cleanup(); }
});
