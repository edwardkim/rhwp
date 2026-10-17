import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const read = (path: string) => readFileSync(join(root, path), 'utf8');

test('cell block delete shortcuts preserve the selection and reach delete handling', () => {
  const keyboard = read('src/engine/input-handler-keyboard.ts');
  const edit = read('src/command/commands/edit.ts');
  const handler = read('src/engine/input-handler.ts');
  assert.match(keyboard, /'edit:delete',\s*\]\);/);
  assert.match(keyboard, /e\.key === 'Delete' \|\| e\.key === 'Backspace'/);
  assert.match(keyboard, /e\.metaKey && !e\.ctrlKey && !e\.altKey[\s\S]{0,300}dispatch\('edit:delete'\)/);
  assert.match(edit, /id: 'edit:delete'[\s\S]*?canExecute:[^\n]*ctx\.inCellSelectionMode/);
  assert.match(handler, /performDelete\(\): void[\s\S]*?this\.deleteSelectedCellBlock\(\)/);
});
