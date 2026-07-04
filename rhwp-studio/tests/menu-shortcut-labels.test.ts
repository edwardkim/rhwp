import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { CommandRegistry } from '../src/command/registry.ts';
import { syncMenuShortcutLabels } from '../src/ui/menu-shortcut-labels.ts';
import type { CommandDef } from '../src/command/types.ts';
import type { PlatformKind } from '../src/engine/navigation-keymap.ts';

type TestGlobal = typeof globalThis & {
  __rhwpTestPlatformKind?: PlatformKind;
  document?: unknown;
};

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function assertCommandShortcut(src: string, commandId: string, shortcutLabel: string): void {
  const start = src.indexOf(`id: '${commandId}'`);
  assert.notEqual(start, -1, `${commandId} command not found`);
  const next = src.indexOf('\n  {', start + 1);
  const block = src.slice(start, next === -1 ? undefined : next);
  assert.match(block, new RegExp(`shortcutLabel:\\s*'${shortcutLabel.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}'`));
}

class FakeSpan {
  className = '';
  textContent = '';
}

class FakeMenuItem {
  dataset: { cmd?: string };
  shortcut: FakeSpan | null;

  constructor(cmd: string, shortcutText?: string) {
    this.dataset = { cmd };
    this.shortcut = shortcutText === undefined ? null : new FakeSpan();
    if (this.shortcut) this.shortcut.textContent = shortcutText;
  }

  querySelector(selector: string): FakeSpan | null {
    if (selector === '.md-shortcut') return this.shortcut;
    return null;
  }

  appendChild(child: FakeSpan): FakeSpan {
    this.shortcut = child;
    return child;
  }
}

class FakeContainer {
  readonly items: FakeMenuItem[];

  constructor(items: FakeMenuItem[]) {
    this.items = items;
  }

  querySelectorAll(selector: string): FakeMenuItem[] {
    assert.equal(selector, '.md-item[data-cmd]');
    return this.items;
  }
}

function registryWith(command: Pick<CommandDef, 'id' | 'shortcutLabel'>): CommandRegistry {
  const registry = new CommandRegistry();
  registry.register({
    id: command.id,
    label: command.id,
    shortcutLabel: command.shortcutLabel,
    execute: () => {},
  });
  return registry;
}

function withPlatform(platform: PlatformKind, run: () => void): void {
  const globalForTest = globalThis as TestGlobal;
  const previousPlatform = globalForTest.__rhwpTestPlatformKind;
  const previousDocument = globalForTest.document;
  globalForTest.__rhwpTestPlatformKind = platform;
  globalForTest.document = {
    createElement(tagName: string) {
      assert.equal(tagName, 'span');
      return new FakeSpan();
    },
  };
  try {
    run();
  } finally {
    if (previousPlatform === undefined) {
      delete globalForTest.__rhwpTestPlatformKind;
    } else {
      globalForTest.__rhwpTestPlatformKind = previousPlatform;
    }
    if (previousDocument === undefined) {
      delete globalForTest.document;
    } else {
      globalForTest.document = previousDocument;
    }
  }
}

test('상단 메뉴 단축키는 macOS에서 ⌘ 기호로 변환된다', () => {
  withPlatform('mac', () => {
    const item = new FakeMenuItem('file:save', 'Ctrl+S');
    syncMenuShortcutLabels(
      new FakeContainer([item]) as unknown as HTMLElement,
      registryWith({ id: 'file:save', shortcutLabel: 'Ctrl+S' }),
    );

    assert.equal(item.shortcut?.textContent, '⌘S');
  });
});

test('상단 메뉴 단축키는 macOS에서 ⌥ 기호로 변환된다', () => {
  withPlatform('mac', () => {
    const item = new FakeMenuItem('file:new-doc', 'Alt+N');
    syncMenuShortcutLabels(
      new FakeContainer([item]) as unknown as HTMLElement,
      registryWith({ id: 'file:new-doc', shortcutLabel: 'Alt+N' }),
    );

    assert.equal(item.shortcut?.textContent, '⌥N');
  });
});

test('상단 메뉴 단축키는 Windows/Linux 계열에서 Ctrl 표기를 유지한다', () => {
  withPlatform('other', () => {
    const item = new FakeMenuItem('file:save', 'Ctrl+S');
    syncMenuShortcutLabels(
      new FakeContainer([item]) as unknown as HTMLElement,
      registryWith({ id: 'file:save', shortcutLabel: 'Ctrl+S' }),
    );

    assert.equal(item.shortcut?.textContent, 'Ctrl+S');
  });
});

test('상단 메뉴 단축키는 하드코딩 값보다 CommandRegistry 정의를 우선한다', () => {
  withPlatform('mac', () => {
    const item = new FakeMenuItem('file:save-as', 'Ctrl+OLD');
    syncMenuShortcutLabels(
      new FakeContainer([item]) as unknown as HTMLElement,
      registryWith({ id: 'file:save-as', shortcutLabel: 'Ctrl+Shift+S' }),
    );

    assert.equal(item.shortcut?.textContent, '⌘⇧S');
  });
});

test('상단 메뉴 항목에 md-shortcut이 없으면 registry 값을 기준으로 생성한다', () => {
  withPlatform('mac', () => {
    const item = new FakeMenuItem('file:print');
    syncMenuShortcutLabels(
      new FakeContainer([item]) as unknown as HTMLElement,
      registryWith({ id: 'file:print', shortcutLabel: 'Ctrl+P' }),
    );

    assert.equal(item.shortcut?.className, 'md-shortcut');
    assert.equal(item.shortcut?.textContent, '⌘P');
  });
});

test('상단 메뉴 하드코딩 단축키와 registry shortcutLabel의 누락 항목을 고정한다', () => {
  const view = source('src/command/commands/view.ts');
  const format = source('src/command/commands/format.ts');

  assertCommandShortcut(view, 'view:zoom-fit-page', 'Ctrl+G,P');
  assertCommandShortcut(view, 'view:zoom-fit-width', 'Ctrl+G,W');
  assert.match(view, /zoomLevel\(100,\s*'Ctrl\+G,Q'\)/);
  assertCommandShortcut(view, 'view:para-mark', 'Ctrl+G,T');
  assertCommandShortcut(view, 'view:border-transparent', 'Alt+V,T');

  assertCommandShortcut(format, 'format:font-size-increase', 'Alt+Shift+E');
  assertCommandShortcut(format, 'format:font-size-decrease', 'Alt+Shift+R');
  assertCommandShortcut(format, 'format:align-left', 'Ctrl+Shift+L');
  assertCommandShortcut(format, 'format:align-center', 'Alt+Shift+C');
  assertCommandShortcut(format, 'format:align-right', 'Alt+Shift+H');
  assertCommandShortcut(format, 'format:align-justify', 'Ctrl+Shift+M');
  assertCommandShortcut(format, 'format:align-distribute', 'Alt+Shift+D');
  assertCommandShortcut(format, 'format:line-spacing-increase', 'Alt+Shift+Z');
  assertCommandShortcut(format, 'format:line-spacing-decrease', 'Alt+Shift+A');
});

test('표 줄/칸 추가·지우기 대표 메뉴에 한컴 단축키를 표시한다', () => {
  const table = source('src/command/commands/table.ts');
  const html = source('index.html');
  const inputHandler = source('src/engine/input-handler.ts');
  const dialog = source('src/ui/dialog.ts');

  assertCommandShortcut(table, 'table:insert-row-col', 'Alt+Enter');
  assertCommandShortcut(table, 'table:delete-row-col', 'Alt+Delete');
  assert.match(table, /id: 'table:insert-row-col'[\s\S]*?label: '줄\/칸 추가하기\(I\)\.\.\.'/);
  assert.match(table, /id: 'table:delete-row-col'[\s\S]*?label: '줄\/칸 지우기\(E\)\.\.\.'/);
  assert.match(html, /data-cmd="table:insert-row-col"[\s\S]*?<span class="md-label">줄\/칸 추가하기\(I\)\.\.\.<\/span>[\s\S]*?<span class="md-shortcut">Alt\+Enter<\/span>/);
  assert.match(html, /data-cmd="table:delete-row-col"[\s\S]*?<span class="md-label">줄\/칸 지우기\(E\)\.\.\.<\/span>[\s\S]*?<span class="md-shortcut">Alt\+Delete<\/span>/);
  assert.match(inputHandler, /commandId: 'table:insert-row-col'/);
  assert.match(inputHandler, /commandId: 'table:delete-row-col'/);
  assert.match(table, /id: 'table:transpose-copy'[\s\S]*?label: '행\/열 바꿈 복사'/);
  assert.match(table, /id: 'table:transpose-paste'[\s\S]*?label: '행\/열 바꿈 붙여넣기'/);
  assert.doesNotMatch(table, /id: 'table:transpose-copy'[\s\S]*?ih\.exitCellSelectionMode\(\)/);
  assert.match(table, /id: 'table:transpose-paste'[\s\S]*?isInCellSelectionMode/);
  assert.match(table, /id: 'table:transpose-paste'[\s\S]*?transposeTableCellsInPlace/);
  assert.match(table, /id: 'table:transpose-paste'[\s\S]*?range\.startRow[\s\S]*?range\.startCol/);
  assert.match(html, /data-cmd="table:transpose-copy"[\s\S]*?<span class="md-label">행\/열 바꿈 복사<\/span>/);
  assert.match(html, /data-cmd="table:transpose-paste"[\s\S]*?<span class="md-label">행\/열 바꿈 붙여넣기<\/span>/);
  assert.match(inputHandler, /commandId: 'table:transpose-copy'/);
  assert.match(inputHandler, /commandId: 'table:transpose-paste'/);
  assert.match(inputHandler, /getDefaultContextMenuItems[\s\S]*?commandId: 'table:transpose-paste'/);
  assert.match(dialog, /afterClose\?\.\(\)/);
  assert.match(table, /id: 'table:insert-row-col'[\s\S]*?dialog\.afterClose = \(\) => restoreEditorFocus\(ih\)/);
  assert.match(table, /id: 'table:delete-row-col'[\s\S]*?dialog\.afterClose = \(\) => restoreEditorFocus\(ih\)/);

  assert.doesNotMatch(html, /data-cmd="table:insert-row-above"/);
  assert.doesNotMatch(html, /data-cmd="table:insert-row-below"/);
  assert.doesNotMatch(html, /data-cmd="table:insert-col-left"/);
  assert.doesNotMatch(html, /data-cmd="table:insert-col-right"/);
  assert.doesNotMatch(html, /data-cmd="table:delete-row"/);
  assert.doesNotMatch(html, /data-cmd="table:delete-col"/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:insert-row-above'/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:insert-row-below'/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:insert-col-left'/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:insert-col-right'/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:delete-row'/);
  assert.doesNotMatch(inputHandler, /commandId: 'table:delete-col'/);
});

test('표 줄/칸 메뉴는 macOS에서 Option 기호로 표시한다', () => {
  withPlatform('mac', () => {
    const insertItem = new FakeMenuItem('table:insert-row-col', 'Alt+OLD');
    const deleteItem = new FakeMenuItem('table:delete-row-col', 'Alt+OLD');
    const registry = new CommandRegistry();
    registry.register({ id: 'table:insert-row-col', label: '줄/칸 추가하기(I)...', shortcutLabel: 'Alt+Enter', execute: () => {} });
    registry.register({ id: 'table:delete-row-col', label: '줄/칸 지우기(E)...', shortcutLabel: 'Alt+Delete', execute: () => {} });

    syncMenuShortcutLabels(
      new FakeContainer([insertItem, deleteItem]) as unknown as HTMLElement,
      registry,
    );

    assert.equal(insertItem.shortcut?.textContent, '⌥Enter');
    assert.equal(deleteItem.shortcut?.textContent, '⌥Delete');
  });
});
