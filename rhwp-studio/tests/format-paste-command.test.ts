import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { canExecuteFormatPaste } from '../src/command/format-paste-availability.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function ctx(overrides: Partial<Parameters<typeof canExecuteFormatPaste>[0]> = {}) {
  return {
    hasDocument: true,
    hasCopiedFormat: true,
    isFormMode: false,
    hasSelection: true,
    inCellSelectionMode: false,
    ...overrides,
  };
}

test('모양 붙여넣기는 복사 상태와 적용 대상이 있을 때만 활성화된다', () => {
  assert.equal(canExecuteFormatPaste(ctx()), true);
  assert.equal(canExecuteFormatPaste(ctx({ hasSelection: false, inCellSelectionMode: true })), true);
  assert.equal(canExecuteFormatPaste(ctx({ hasDocument: false })), false);
  assert.equal(canExecuteFormatPaste(ctx({ hasCopiedFormat: false })), false);
  assert.equal(canExecuteFormatPaste(ctx({ isFormMode: true })), false);
  assert.equal(canExecuteFormatPaste(ctx({ hasSelection: false, inCellSelectionMode: false })), false);
});

test('edit:format-paste 커맨드는 붙여넣기 전용 경로에 연결되어야 한다', () => {
  const edit = source('src/command/commands/edit.ts');

  assert.match(edit, /id:\s*'edit:format-paste'/);
  assert.match(edit, /label:\s*'모양 붙여넣기'/);
  assert.match(edit, /canExecute:\s*canExecuteFormatPaste/);
  assert.match(edit, /performFormatPaste\(\)/);
});

test('모양 붙여넣기 UI와 컨텍스트 상태 경로가 유지되어야 한다', () => {
  const html = source('index.html');
  const types = source('src/command/types.ts');
  const main = source('src/main.ts');
  const inputHandler = source('src/engine/input-handler.ts');
  const commandPalette = source('src/ui/command-palette.ts');

  assert.match(html, /data-cmd="edit:format-paste"/);
  assert.match(types, /hasCopiedFormat:\s*boolean/);
  assert.match(main, /hasCopiedFormat:\s*inputHandler\?\.hasCopiedFormat\(\)\s*\?\?\s*false/);
  assert.match(inputHandler, /hasCopiedFormat\(\):\s*boolean/);
  assert.match(inputHandler, /performFormatPaste\(\):\s*void/);
  assert.equal((inputHandler.match(/commandId:\s*'edit:format-paste'/g) ?? []).length, 2);
  assert.match(commandPalette, /hasCopiedFormat:\s*false/);
});
