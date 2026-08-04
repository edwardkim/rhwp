import assert from 'node:assert/strict';
import test from 'node:test';

import { createChromeStorageMock } from '../test/extension-storage-mock.mjs';
import {
  DEFAULT_SETTINGS,
  LOCAL_BACKUP_KEY,
  SETTINGS_SCHEMA_VERSION,
  SYNC_META_KEY,
  loadSettings,
  loadSettingsForAutomaticActions,
  saveSettings,
} from './settings-store.js';

function snapshot(settings, updatedAt = 100) {
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    updatedAt,
    settings,
  };
}

test('clean storage returns defaults and creates only a local backup', async () => {
  const env = createChromeStorageMock();

  const settings = await loadSettings(env.chrome, { now: () => 101 });

  assert.deepEqual(settings, DEFAULT_SETTINGS);
  assert.deepEqual(env.syncArea.dump(), {});
  assert.deepEqual(env.localArea.dump()[LOCAL_BACKUP_KEY], snapshot(DEFAULT_SETTINGS, 101));
});

test('legacy flat sync values are preserved and backed up locally', async () => {
  const env = createChromeStorageMock({
    sync: { autoOpen: false, showBadges: false, hoverPreview: true },
  });

  const settings = await loadSettings(env.chrome, { now: () => 102 });

  assert.deepEqual(settings, {
    ...DEFAULT_SETTINGS,
    autoOpen: false,
    showBadges: false,
  });
  assert.equal(env.localArea.dump()[LOCAL_BACKUP_KEY].settings.autoOpen, false);
});

test('missing sync keys recover per-key values from the local backup', async () => {
  const localSettings = {
    autoOpen: false,
    showBadges: false,
    hoverPreview: false,
    disableExternalWebFonts: true,
  };
  const env = createChromeStorageMock({
    sync: { showBadges: true },
    local: { [LOCAL_BACKUP_KEY]: snapshot(localSettings) },
  });

  const settings = await loadSettings(env.chrome, { now: () => 103 });

  assert.deepEqual(settings, { ...localSettings, showBadges: true });
});

test('sync read failure falls back to the local backup', async () => {
  const localSettings = { ...DEFAULT_SETTINGS, autoOpen: false };
  const env = createChromeStorageMock({
    local: { [LOCAL_BACKUP_KEY]: snapshot(localSettings) },
  });
  env.syncArea.failNextGet(new Error('sync unavailable'));

  const settings = await loadSettings(env.chrome, { now: () => 104 });

  assert.equal(settings.autoOpen, false);
});

test('automatic actions fail closed when sync cannot confirm a local true value', async () => {
  const env = createChromeStorageMock({
    local: { [LOCAL_BACKUP_KEY]: snapshot(DEFAULT_SETTINGS) },
  });
  env.syncArea.failNextGet(new Error('sync unavailable'));

  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 105 });

  assert.equal(settings.autoOpen, false);
});

test('automatic actions keep the clean-install autoOpen default', async () => {
  const env = createChromeStorageMock();

  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 106 });

  assert.equal(settings.autoOpen, true);
});

test('automatic actions fail closed for partial sync without a local backup', async () => {
  const env = createChromeStorageMock({
    sync: { useRhwpViewer: true },
  });

  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 107 });

  assert.equal(settings.autoOpen, false);
  assert.equal(env.localArea.dump()[LOCAL_BACKUP_KEY], undefined);
});

test('automatic actions fail closed when only sync metadata survives', async () => {
  const env = createChromeStorageMock({
    sync: {
      [SYNC_META_KEY]: { schemaVersion: SETTINGS_SCHEMA_VERSION, updatedAt: 50 },
    },
  });

  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 108 });

  assert.equal(settings.autoOpen, false);
  assert.equal(env.localArea.dump()[LOCAL_BACKUP_KEY], undefined);
});

test('automatic actions recover a missing sync key from a valid local backup', async () => {
  const env = createChromeStorageMock({
    sync: { showBadges: false },
    local: { [LOCAL_BACKUP_KEY]: snapshot(DEFAULT_SETTINGS) },
  });

  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 109 });

  assert.equal(settings.autoOpen, true);
});

test('load fails when neither storage area can provide a trustworthy value', async () => {
  const env = createChromeStorageMock();
  env.syncArea.failNextGet(new Error('sync unavailable'));
  env.localArea.failNextGet(new Error('local unavailable'));

  await assert.rejects(loadSettings(env.chrome), /설정을 불러오지 못했습니다/);
});

test('sync read failure does not silently enable defaults without a local backup', async () => {
  const env = createChromeStorageMock();
  env.syncArea.failNextGet(new Error('sync unavailable'));

  await assert.rejects(loadSettings(env.chrome), /설정을 불러오지 못했습니다/);
  assert.deepEqual(env.localArea.dump(), {});
});

test('save writes a versioned local snapshot and compatible flat sync keys', async () => {
  const env = createChromeStorageMock();
  const next = { ...DEFAULT_SETTINGS, autoOpen: false, disableExternalWebFonts: true };

  const saved = await saveSettings(env.chrome, next, { now: () => 200 });

  assert.deepEqual(saved, next);
  assert.deepEqual(env.localArea.dump()[LOCAL_BACKUP_KEY], snapshot(next, 200));
  assert.deepEqual(env.syncArea.dump(), {
    ...next,
    [SYNC_META_KEY]: { schemaVersion: SETTINGS_SCHEMA_VERSION, updatedAt: 200 },
  });
});

test('save rejects instead of reporting success when sync persistence fails', async () => {
  const env = createChromeStorageMock();
  env.syncArea.failNextSet(new Error('sync write failed'));

  await assert.rejects(
    saveSettings(env.chrome, { ...DEFAULT_SETTINGS, autoOpen: false }),
    /sync write failed/,
  );
});

test('local backup failure does not prevent authoritative sync persistence', async () => {
  const env = createChromeStorageMock();
  const next = { ...DEFAULT_SETTINGS, autoOpen: false };
  env.localArea.failNextSet(new Error('local backup failed'));

  const originalWarn = console.warn;
  console.warn = () => {};
  try {
    const saved = await saveSettings(env.chrome, next, { now: () => 201 });
    assert.deepEqual(saved, next);
  } finally {
    console.warn = originalWarn;
  }

  assert.deepEqual(env.syncArea.dump(), {
    ...next,
    [SYNC_META_KEY]: { schemaVersion: SETTINGS_SCHEMA_VERSION, updatedAt: 201 },
  });
});

test('invalid stored values never replace boolean settings', async () => {
  const env = createChromeStorageMock({
    sync: { autoOpen: 'false', showBadges: 0 },
    local: {
      [LOCAL_BACKUP_KEY]: snapshot({ ...DEFAULT_SETTINGS, autoOpen: false, showBadges: false }),
    },
  });

  const settings = await loadSettings(env.chrome, { now: () => 300 });

  assert.equal(settings.autoOpen, false);
  assert.equal(settings.showBadges, false);
});
