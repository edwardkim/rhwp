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

test('문단부호 표시 설정은 rhwp-settings에 저장된다', () => {
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
    userSettings.setShowParagraphMarks(true);
    assert.equal(userSettings.getViewSettings().showParagraphMarks, true);
    let stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.showParagraphMarks, true);

    userSettings.setShowControlCodes(true);
    assert.equal(userSettings.getViewSettings().showControlCodes, true);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.showControlCodes, true);

    userSettings.setShowParagraphMarks(false);
    assert.equal(userSettings.getViewSettings().showParagraphMarks, false);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.showParagraphMarks, false);
  } finally {
    userSettings.setShowControlCodes(false);
    userSettings.setShowParagraphMarks(false);
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('짤림보기(clipView) 설정은 rhwp-settings에 저장되고 기본값은 켜짐이다', () => {
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
    // 기본값: 짤림보기 켜짐(오버플로 표시)
    assert.equal(userSettings.getViewSettings().clipView, true);

    userSettings.setClipView(false);
    assert.equal(userSettings.getViewSettings().clipView, false);
    let stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.clipView, false);

    userSettings.setClipView(true);
    assert.equal(userSettings.getViewSettings().clipView, true);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.clipView, true);
  } finally {
    userSettings.setClipView(true);
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('복구용 자동저장 설정은 rhwp-settings에 저장된다', () => {
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
    userSettings.updateAutosaveSettings({
      recoveryEnabled: false,
      recoveryIntervalMinutes: 30,
      idleSaveEnabled: true,
      idleDelaySeconds: 45,
    });

    const settings = userSettings.getAutosaveSettings();
    assert.equal(settings.recoveryEnabled, false);
    assert.equal(settings.recoveryIntervalMinutes, 30);
    assert.equal(settings.idleSaveEnabled, true);
    assert.equal(settings.idleDelaySeconds, 45);

    const stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.deepEqual(stored.autosave, {
      recoveryEnabled: false,
      recoveryIntervalMinutes: 30,
      idleSaveEnabled: true,
      idleDelaySeconds: 45,
    });
  } finally {
    userSettings.updateAutosaveSettings({
      recoveryEnabled: true,
      recoveryIntervalMinutes: 10,
      idleSaveEnabled: true,
      idleDelaySeconds: 10,
    });
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});
