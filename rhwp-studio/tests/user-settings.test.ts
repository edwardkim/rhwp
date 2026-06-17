import test from 'node:test';
import assert from 'node:assert/strict';

import { userSettings } from '../src/core/user-settings.ts';

test('개체 속성 비율 유지 설정은 rhwp-settings에 저장된다', () => {
  const originalStorage = (globalThis as { localStorage?: Storage }).localStorage;
  const store = new Map<string, string>();
  const mockStorage = {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  } as Storage;

  (globalThis as { localStorage?: Storage }).localStorage = mockStorage;
  try {
    userSettings.setPicturePropsKeepRatio(false);
    assert.equal(userSettings.getPicturePropsKeepRatio(), false);
    let stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.dialog.picturePropsKeepRatio, false);

    userSettings.setPicturePropsKeepRatio(true);
    assert.equal(userSettings.getPicturePropsKeepRatio(), true);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.dialog.picturePropsKeepRatio, true);
  } finally {
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});
