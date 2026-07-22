import assert from 'node:assert/strict';
import test from 'node:test';

import { createChromeStorageMock } from '../test/extension-storage-mock.mjs';
import { LIFECYCLE_KEY, handleExtensionInstalled } from './extension-lifecycle.js';
import {
  LOCAL_BACKUP_KEY,
  loadSettingsForAutomaticActions,
} from './settings-store.js';

for (const reason of ['install', 'update', 'chrome_update']) {
  test(`${reason} records diagnostics without writing sync preferences`, async () => {
    const env = createChromeStorageMock({
      sync: { autoOpen: false },
      version: '0.2.9',
    });

    await handleExtensionInstalled(
      env.chrome,
      { reason, previousVersion: reason === 'update' ? '0.2.8' : undefined },
      { now: () => 400 },
    );

    assert.deepEqual(env.syncArea.dump(), { autoOpen: false });
    assert.deepEqual(env.syncArea.calls.set, []);
    assert.deepEqual(env.localArea.dump()[LIFECYCLE_KEY], {
      reason,
      previousVersion: reason === 'update' ? '0.2.8' : null,
      currentVersion: '0.2.9',
      recordedAt: 400,
    });
  });
}

test('update preserves autoOpen=false before a sync key is lost', async () => {
  const env = createChromeStorageMock({
    sync: { autoOpen: false, useRhwpViewer: true },
    version: '0.2.9',
  });

  await handleExtensionInstalled(
    env.chrome,
    { reason: 'update', previousVersion: '0.2.8' },
    { now: () => 401 },
  );
  assert.equal(env.localArea.dump()[LOCAL_BACKUP_KEY].settings.autoOpen, false);

  await env.syncArea.remove(['autoOpen']);
  const settings = await loadSettingsForAutomaticActions(env.chrome, { now: () => 402 });

  assert.equal(settings.autoOpen, false);
  assert.equal(env.localArea.dump()[LOCAL_BACKUP_KEY].settings.autoOpen, false);
});
