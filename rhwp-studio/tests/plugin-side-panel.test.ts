import test from 'node:test';
import assert from 'node:assert/strict';

import { PluginHostRegistry, type PluginHostDeps } from '../src/plugin/host.ts';
import { PLUGIN_API_VERSION, type PluginHost, type StudioPlugin } from '../src/plugin/types.ts';
import type { SidePanelRegistry } from '../src/view/side-panel-host.ts';
import type { PluginSidePanelSpec } from '../src/plugin/types.ts';

function fakePanels() {
  const registered = new Map<string, PluginSidePanelSpec>();
  let open: string | null = null;
  const panels: SidePanelRegistry = {
    register(spec) {
      registered.set(spec.id, spec);
      return () => { registered.delete(spec.id); if (open === spec.id) open = null; };
    },
    toggle(id, want) {
      const next = want ?? open !== id;
      open = next ? id : (open === id ? null : open);
      return next;
    },
    isOpen: (id) => open === id,
  };
  return { panels, registered };
}

function fakeAutomation() {
  const commands = new Set<string>();
  const menuItems = new Set<string>();
  return {
    commands, menuItems,
    automation: {
      listCommands: () => [], getMenuModel: () => [], isEnabled: () => true,
      execute: () => ({ ok: true }), getContext: () => ({}),
      registerCommand: (def: { id: string }) => { commands.add(def.id); },
      unregisterCommand: (id: string) => { commands.delete(id); },
      addMenuItem: (spec: { commandId: string }) => { menuItems.add(spec.commandId); },
      removeMenuItem: (id: string) => { menuItems.delete(id); },
    },
  };
}

function registry(plugin: StudioPlugin, sidePanels?: SidePanelRegistry) {
  const { automation } = fakeAutomation();
  const deps = {
    wasm: { borrowDocumentHandle: () => null, documentGeneration: 1 },
    automation,
    eventBus: { on: () => () => {} },
    getInputHandler: () => null,
    loadDocument: async () => {},
    createBlankDocument: () => {},
    resolve: async () => plugin,
    sidePanels,
  } as unknown as PluginHostDeps;
  return new PluginHostRegistry(deps);
}

function panelPlugin(capture: (host: PluginHost) => void): StudioPlugin {
  return {
    id: 'probe',
    apiVersion: PLUGIN_API_VERSION,
    activate(host) {
      host.ui.registerSidePanel({ id: 'chat', title: 'Chat', mount: () => {} });
      capture(host);
      return {};
    },
  };
}

test('패널 id 는 플러그인 id 로 감싸고 토글·열림 조회도 같은 이름을 쓴다', async () => {
  const { panels, registered } = fakePanels();
  let host!: PluginHost;
  await registry(panelPlugin((h) => { host = h; }), panels).load('probe');

  assert.deepEqual([...registered.keys()], ['probe:chat']);
  assert.equal(host.ui.toggleSidePanel('chat', true), true);
  assert.equal(host.ui.isSidePanelOpen('chat'), true);
  assert.equal(panels.isOpen('probe:chat'), true);
});

test('unload 는 플러그인이 등록한 패널을 원장으로 걷는다', async () => {
  const { panels, registered } = fakePanels();
  const plugins = registry(panelPlugin(() => {}), panels);
  await plugins.load('probe');
  await plugins.unload('probe');
  assert.equal(registered.size, 0);
});

test('패널 자리가 없는 구성에서는 등록이 거절되고 활성화 실패로 돌아온다', async () => {
  await assert.rejects(
    registry(panelPlugin(() => {})).load('probe'),
    (error: Error & { code?: string }) => error.code === 'PLUGIN_ACTIVATE_FAILED'
      && /패널 자리가 없습니다/.test(error.message),
  );
});

test('패널 자리가 없으면 토글·열림 조회는 false 다', async () => {
  let host!: PluginHost;
  const plugin: StudioPlugin = {
    id: 'probe', apiVersion: PLUGIN_API_VERSION,
    activate(h) { host = h; return {}; },
  };
  await registry(plugin).load('probe');
  assert.equal(host.ui.toggleSidePanel('chat', true), false);
  assert.equal(host.ui.isSidePanelOpen('chat'), false);
});

test('문서가 없으면 선택 평문은 null 이다', async () => {
  let host!: PluginHost;
  const plugin: StudioPlugin = {
    id: 'probe', apiVersion: PLUGIN_API_VERSION,
    activate(h) { host = h; return {}; },
  };
  await registry(plugin).load('probe');
  assert.equal(host.getSelectionText(), null);
});
