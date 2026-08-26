import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import {
  TOOLBOX_TARGETS,
  applyToolboxVisibility,
  type ToolboxDom,
} from '../src/view/toolbox-visibility.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

interface FakeItem {
  cmd: string;
  active: boolean | null;
  ariaChecked: string | null;
  ariaExpanded: string | null;
  ariaLabel: string | null;
  title: string | null;
  attributes: Record<string, string>;
  classList: { toggle(token: string, force: boolean): void };
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
}

function fakeItem(cmd: string, attributes: Record<string, string>): FakeItem {
  const item: FakeItem = {
    cmd,
    active: null,
    ariaChecked: null,
    ariaExpanded: null,
    ariaLabel: null,
    title: null,
    attributes: { ...attributes },
    classList: {
      toggle(token: string, force: boolean) {
        if (token === 'active') item.active = force;
      },
    },
    getAttribute(name: string) {
      return item.attributes[name] ?? null;
    },
    setAttribute(name: string, value: string) {
      item.attributes[name] = value;
      if (name === 'aria-checked') item.ariaChecked = value;
      if (name === 'aria-expanded') item.ariaExpanded = value;
      if (name === 'aria-label') item.ariaLabel = value;
      if (name === 'title') item.title = value;
    },
  };
  return item;
}

function fakeDom() {
  const dataset: Record<string, string | undefined> = {};
  const items = TOOLBOX_TARGETS.map(target => fakeItem(target.cmd, { role: 'menuitemcheckbox' }));
  const button = fakeItem('view:toolbox-basic', { 'aria-controls': 'icon-toolbar' });
  items.push(button);
  const dom: ToolboxDom = {
    documentElement: { dataset },
    querySelectorAll: (selector) => items.filter(item => selector === `[data-cmd="${item.cmd}"]`),
  };
  return { dom, dataset, items, button };
}

test('도구 상자 설정은 루트 표시 상태와 메뉴 체크 상태를 함께 맞춘다', () => {
  const { dom, dataset, items, button } = fakeDom();

  applyToolboxVisibility(dom, { basic: false, format: true });
  assert.equal(dataset.toolboxBasic, 'hidden');
  assert.equal(dataset.toolboxFormat, 'shown');
  assert.deepEqual(
    items.slice(0, 2).map(i => [i.cmd, i.active, i.ariaChecked]),
    [
      ['view:toolbox-basic', false, 'false'],
      ['view:toolbox-format', true, 'true'],
    ],
  );
  assert.deepEqual(
    [button.active, button.ariaChecked, button.ariaExpanded, button.ariaLabel, button.title],
    [false, null, 'false', '기본 도구 상자 펴기', '기본 도구 상자 펴기 (Ctrl+F1)'],
  );

  applyToolboxVisibility(dom, { basic: true, format: false });
  assert.equal(dataset.toolboxBasic, 'shown');
  assert.equal(dataset.toolboxFormat, 'hidden');
  assert.deepEqual(
    items.slice(0, 2).map(i => [i.cmd, i.active, i.ariaChecked]),
    [
      ['view:toolbox-basic', true, 'true'],
      ['view:toolbox-format', false, 'false'],
    ],
  );
  assert.deepEqual(
    [button.active, button.ariaChecked, button.ariaExpanded, button.ariaLabel, button.title],
    [true, null, 'true', '기본 도구 상자 접기', '기본 도구 상자 접기 (Ctrl+F1)'],
  );
});

test('도구 상자 메뉴 항목은 체크 상태를 가진 활성 항목이다', () => {
  const html = source('index.html');
  for (const target of TOOLBOX_TARGETS) {
    const line = html.split('\n').find(l => l.includes(`data-cmd="${target.cmd}"`));
    assert.ok(line, `메뉴 항목이 있어야 한다: ${target.cmd}`);
    assert.match(line!, /role="menuitemcheckbox"/, `체크형 항목이어야 한다: ${target.cmd}`);
    assert.doesNotMatch(line!, /md-item disabled/, `비활성으로 두면 안 된다: ${target.cmd}`);
  }
});

test('메뉴바 우측 버튼은 기본 도구 상자 커맨드와 접근성 상태를 공유한다', () => {
  const html = source('index.html');
  const menuBar = source('src/ui/menu-bar.ts');
  const commands = source('src/command/commands/view.ts');
  const main = source('src/main.ts');

  assert.match(
    html,
    /<button[^>]*id="toolbox-basic-toggle"[^>]*class="menu-command menu-toolbox-toggle"[^>]*data-cmd="view:toolbox-basic"[^>]*aria-controls="icon-toolbar"[^>]*aria-expanded="false"/,
  );
  assert.match(
    menuBar,
    /closest\(\s*'\.md-item\[data-cmd\], \.menu-command\[data-cmd\]'\s*,?\s*\)/,
  );
  assert.match(commands, /id: 'view:toolbox-basic',[\s\S]*?shortcutLabel: 'Ctrl\+F1'/);
  assert.match(main, /GLOBAL_VIEW_SHORTCUTS[\s\S]*?'view:toolbox-basic'/);
});

test('숨김 규칙과 첫 페인트 초기화는 같은 data 속성을 쓴다', () => {
  const css = source('src/style.css');
  const init = source('public/theme-init.js');
  for (const target of TOOLBOX_TARGETS) {
    assert.match(
      css,
      new RegExp(`\\[${target.attribute}="hidden"\\]\\s*#${target.elementId}\\s*\\{[^}]*display:\\s*none`),
      `숨김 규칙이 있어야 한다: ${target.attribute}`,
    );
    assert.match(
      init,
      new RegExp(`root\\.dataset\\.${target.datasetKey}\\s*=`),
      `첫 페인트 전 초기화가 있어야 한다: ${target.datasetKey}`,
    );
  }
  // 저장 키(user-settings 의 view.toolbar*)를 theme-init 이 그대로 읽어야 복원이 맞다.
  assert.match(init, /view\.toolbarBasic === true/);
  assert.match(init, /view\.toolbarFormat === false/);
});
