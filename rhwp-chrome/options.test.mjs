import assert from 'node:assert/strict';
import test from 'node:test';

import { createChromeStorageMock } from './test/extension-storage-mock.mjs';
import { LOCAL_BACKUP_KEY, SETTINGS_SCHEMA_VERSION } from './sw/settings-store.js';
import { initializeOptionsPage } from './options.js';

const INPUT_IDS = ['autoOpen', 'showBadges', 'hoverPreview', 'disableExternalWebFonts'];

function createElement({ checked = false, disabled = false } = {}) {
  const listeners = new Map();
  const classes = new Set();
  return {
    checked,
    disabled,
    textContent: '',
    classList: {
      add(...names) {
        for (const name of names) classes.add(name);
      },
      remove(...names) {
        for (const name of names) classes.delete(name);
      },
      contains(name) {
        return classes.has(name);
      },
    },
    addEventListener(type, listener) {
      listeners.set(type, listener);
    },
    hasListener(type) {
      return listeners.has(type);
    },
    async dispatch(type) {
      return listeners.get(type)?.();
    },
  };
}

function createDocumentMock() {
  const elements = new Map();
  for (const id of [
    'title',
    'labelAutoOpen',
    'labelShowBadges',
    'labelHoverPreview',
    'labelDisableExternalWebFonts',
    'descDisableExternalWebFonts',
    'saved',
    'privacy',
    'version',
  ]) {
    elements.set(id, createElement());
  }
  for (const id of INPUT_IDS) elements.set(id, createElement({ disabled: true }));
  return {
    elements,
    documentRef: {
      getElementById(id) {
        return elements.get(id) ?? null;
      },
    },
  };
}

function addI18n(chrome) {
  chrome.i18n = {
    getMessage(key) {
      return {
        optionsSaved: 'Saved',
        optionsSaveError: 'Save failed',
        optionsLoadError: 'Load failed',
      }[key] ?? key;
    },
  };
}

function backup(settings, updatedAt = 1) {
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    updatedAt,
    settings,
  };
}

async function withoutExpectedConsoleErrors(run) {
  const originalError = console.error;
  console.error = () => {};
  try {
    return await run();
  } finally {
    console.error = originalError;
  }
}

test('inputs stay disabled and unbound until asynchronous loading finishes', async () => {
  const env = createChromeStorageMock();
  addI18n(env.chrome);
  const { documentRef, elements } = createDocumentMock();
  let resolveSync;
  env.chrome.storage.sync.get = () => new Promise((resolve) => {
    resolveSync = resolve;
  });

  const initializing = initializeOptionsPage({
    chromeApi: env.chrome,
    documentRef,
    setTimeoutFn() {},
  });

  assert.equal(elements.get('autoOpen').disabled, true);
  assert.equal(elements.get('autoOpen').hasListener('change'), false);

  resolveSync({ autoOpen: false, showBadges: true, hoverPreview: false });
  await initializing;

  assert.equal(elements.get('autoOpen').checked, false);
  assert.equal(elements.get('autoOpen').disabled, false);
  assert.equal(elements.get('autoOpen').hasListener('change'), true);
});

test('a successful change preserves other loaded settings and reports success', async () => {
  const initial = {
    autoOpen: false,
    showBadges: false,
    hoverPreview: true,
    disableExternalWebFonts: true,
  };
  const env = createChromeStorageMock({
    sync: initial,
    local: { [LOCAL_BACKUP_KEY]: backup(initial) },
  });
  addI18n(env.chrome);
  const { documentRef, elements } = createDocumentMock();

  await initializeOptionsPage({ chromeApi: env.chrome, documentRef, setTimeoutFn() {} });
  elements.get('autoOpen').checked = true;
  await elements.get('autoOpen').dispatch('change');

  assert.deepEqual(
    Object.fromEntries(INPUT_IDS.map((id) => [id, env.syncArea.dump()[id]])),
    { ...initial, autoOpen: true },
  );
  assert.equal(elements.get('saved').textContent, 'Saved');
  assert.equal(elements.get('saved').classList.contains('show'), true);
  assert.equal(elements.get('saved').classList.contains('error'), false);
});

test('sync save failure reports an error and restores the persisted value', async () => {
  const initial = {
    autoOpen: true,
    showBadges: true,
    hoverPreview: true,
    disableExternalWebFonts: false,
  };
  const env = createChromeStorageMock({
    sync: initial,
    local: { [LOCAL_BACKUP_KEY]: backup(initial) },
  });
  addI18n(env.chrome);
  const { documentRef, elements } = createDocumentMock();

  await initializeOptionsPage({ chromeApi: env.chrome, documentRef, setTimeoutFn() {} });
  env.syncArea.failNextSet(new Error('sync write failed'));
  elements.get('autoOpen').checked = false;
  await withoutExpectedConsoleErrors(() => elements.get('autoOpen').dispatch('change'));

  assert.equal(elements.get('autoOpen').checked, true);
  assert.equal(elements.get('saved').textContent, 'Save failed');
  assert.equal(elements.get('saved').classList.contains('error'), true);
});

test('load failure keeps inputs disabled and displays a load error', async () => {
  const env = createChromeStorageMock();
  addI18n(env.chrome);
  env.syncArea.failNextGet(new Error('sync failed'));
  env.localArea.failNextGet(new Error('local failed'));
  const { documentRef, elements } = createDocumentMock();

  await withoutExpectedConsoleErrors(() => initializeOptionsPage({
    chromeApi: env.chrome,
    documentRef,
    setTimeoutFn() {},
  }));

  assert.equal(elements.get('autoOpen').disabled, true);
  assert.equal(elements.get('saved').textContent, 'Load failed');
  assert.equal(elements.get('saved').classList.contains('error'), true);
});

// The packaged content script must wait for a valid settings reply before enabling UI.
async function contentSettingsProbe(reply, lastError) {
  const { readFile } = await import('node:fs/promises');
  const { runInNewContext } = await import('node:vm');
  const source = await readFile(new URL('./content-script.js', import.meta.url), 'utf8');
  const observed = { scans: 0, observers: 0, timers: 0, errorReads: 0 };
  let receive;
  runInNewContext(source, {
    chrome: { runtime: {
      sendMessage(message, callback) {
        assert.equal(message.type, 'get-settings');
        receive = callback;
      },
      get lastError() { observed.errorReads++; return lastError; },
    } },
    document: {
      head: { querySelector() { return null; } }, body: {},
      querySelectorAll() { observed.scans++; return []; },
    },
    MutationObserver: class { observe() { observed.observers++; } },
    setTimeout() { observed.timers++; }, URL, console,
  });
  assert.equal(typeof receive, 'function');
  assert.equal(observed.scans, 0, 'no scan before settings arrive');
  receive(reply);
  return observed;
}

test('content settings channel failure is handled without enabling default badges or prefetch', async () => {
  const result = await contentSettingsProbe(undefined, { message: 'message channel closed' });
  assert.equal(result.errorReads, 1, 'consume callback-scoped runtime.lastError');
  assert.equal(result.scans, 0);
  assert.equal(result.observers, 0);
  assert.equal(result.timers, 0);
});

test('content settings disabled reply preserves the user choice', async () => {
  const result = await contentSettingsProbe({ showBadges: false, hoverPreview: false }, undefined);
  assert.equal(result.scans, 0);
  assert.equal(result.observers, 0);
  assert.equal(result.timers, 0);
});

test('content settings enabled reply still starts the normal scan and observer', async () => {
  const result = await contentSettingsProbe({ showBadges: true, hoverPreview: false }, undefined);
  assert.equal(result.scans, 1);
  assert.equal(result.observers, 1);
});
