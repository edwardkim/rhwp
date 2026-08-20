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
  classList: { toggle(token: string, force: boolean): void };
  setAttribute(name: string, value: string): void;
}

function fakeItem(cmd: string): FakeItem {
  const item: FakeItem = {
    cmd,
    active: null,
    ariaChecked: null,
    classList: {
      toggle(token: string, force: boolean) {
        if (token === 'active') item.active = force;
      },
    },
    setAttribute(name: string, value: string) {
      if (name === 'aria-checked') item.ariaChecked = value;
    },
  };
  return item;
}

function fakeDom() {
  const dataset: Record<string, string | undefined> = {};
  const items = TOOLBOX_TARGETS.map(target => fakeItem(target.cmd));
  const dom: ToolboxDom = {
    documentElement: { dataset },
    querySelectorAll: (selector) => items.filter(item => selector === `[data-cmd="${item.cmd}"]`),
  };
  return { dom, dataset, items };
}

test('도구 상자 설정은 루트 표시 상태와 메뉴 체크 상태를 함께 맞춘다', () => {
  const { dom, dataset, items } = fakeDom();

  applyToolboxVisibility(dom, { basic: false, format: true });
  assert.equal(dataset.toolboxBasic, 'hidden');
  assert.equal(dataset.toolboxFormat, 'shown');
  assert.deepEqual(
    items.map(i => [i.cmd, i.active, i.ariaChecked]),
    [
      ['view:toolbox-basic', false, 'false'],
      ['view:toolbox-format', true, 'true'],
    ],
  );

  applyToolboxVisibility(dom, { basic: true, format: false });
  assert.equal(dataset.toolboxBasic, 'shown');
  assert.equal(dataset.toolboxFormat, 'hidden');
  assert.deepEqual(
    items.map(i => [i.cmd, i.active, i.ariaChecked]),
    [
      ['view:toolbox-basic', true, 'true'],
      ['view:toolbox-format', false, 'false'],
    ],
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
  assert.match(init, /view\.toolbarBasic === false/);
  assert.match(init, /view\.toolbarFormat === false/);
});
