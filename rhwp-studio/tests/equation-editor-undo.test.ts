import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const editorSource = readFileSync(
  new URL('../src/ui/equation-editor-dialog.ts', import.meta.url),
  'utf8',
);
const insertSource = readFileSync(
  new URL('../src/command/commands/insert.ts', import.meta.url),
  'utf8',
);

test('existing equation editor applies changes through snapshot history', () => {
  assert.match(editorSource, /private services\?: CommandServices/);
  assert.match(
    editorSource,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'equationEdit'/,
  );
  assert.match(
    insertSource,
    /new EquationEditorDialog\(services\.wasm, services\.eventBus, services\)/,
  );
});
