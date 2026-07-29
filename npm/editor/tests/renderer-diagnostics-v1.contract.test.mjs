import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const declarations = readFileSync(fileURLToPath(new URL('../index.d.ts', import.meta.url)), 'utf8');

function interfaceSource(name) {
  const match = declarations.match(new RegExp(`export interface ${name} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `${name} declaration must exist`);
  return match[1];
}

test('renderer-diagnostics-v1 keeps its original backend enum', () => {
  const diagnostics = interfaceSource('RendererDiagnosticsV1');
  const diagnosticsBackend = diagnostics
    .split('\n')
    .find((line) => line.trim().startsWith('backend: { backend:'));
  assert.match(diagnosticsBackend ?? '', /backend: 'canvas2d' \| 'canvaskit'/);
  assert.doesNotMatch(diagnosticsBackend ?? '', /'auto'/);
  assert.match(diagnostics, /selection\?: RendererSelectionV1 \| null/);
});

test('additive renderer selection represents auto requests', () => {
  const selection = interfaceSource('RendererSelectionV1');
  const selectionRequest = selection
    .split('\n')
    .find((line) => line.trim().startsWith('request: { backend:'));
  assert.match(selectionRequest ?? '', /backend: 'auto' \| 'canvas2d' \| 'canvaskit'/);
  assert.match(selection, /requestedBackend: 'auto' \| 'canvas2d' \| 'canvaskit'/);
});
