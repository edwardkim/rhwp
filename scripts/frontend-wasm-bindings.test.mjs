import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));

test('generated WASM declarations contain every explicit js_name export', () => {
  const source = readFileSync(path.join(ROOT, 'src/wasm_api.rs'), 'utf8');
  const declarations = readFileSync(path.join(ROOT, 'pkg/rhwp.d.ts'), 'utf8');
  const exportNames = new Set(
    [...source.matchAll(/#\[wasm_bindgen\(js_name\s*=\s*([A-Za-z0-9_]+)\)\]/g)]
      .map((match) => match[1]),
  );
  const missing = [...exportNames]
    .filter((name) => !new RegExp(`\\b${name}\\b`).test(declarations))
    .sort();

  assert.deepEqual(
    missing,
    [],
    `pkg/rhwp.d.ts is stale; rebuild WASM before frontend gates: ${missing.join(', ')}`,
  );
});
