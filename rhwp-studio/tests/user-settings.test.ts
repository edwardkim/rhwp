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

test('PDF 저장 안내 표시 설정은 rhwp-settings에 저장되고 다시 켤 수 있다', () => {
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
    userSettings.setShowPdfPrintGuidance(false);
    assert.equal(userSettings.getShowPdfPrintGuidance(), false);
    let stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.dialog.showPdfPrintGuidance, false);

    userSettings.setShowPdfPrintGuidance(true);
    assert.equal(userSettings.getShowPdfPrintGuidance(), true);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.dialog.showPdfPrintGuidance, true);
  } finally {
    userSettings.setShowPdfPrintGuidance(true);
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

test('도구 상자 표시 설정은 저장되고 기본 도구 상자만 처음에 접혀 있다', () => {
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
    assert.equal(userSettings.getViewSettings().toolbarBasic, false);
    assert.equal(userSettings.getViewSettings().toolbarFormat, true);

    userSettings.setToolbarBasic(false);
    userSettings.setToolbarFormat(false);
    let stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.toolbarBasic, false);
    assert.equal(stored.view.toolbarFormat, false);

    userSettings.setToolbarFormat(true);
    stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.view.toolbarBasic, false);
    assert.equal(stored.view.toolbarFormat, true);
  } finally {
    userSettings.setToolbarBasic(false);
    userSettings.setToolbarFormat(true);
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('저장된 도구 상자 설정은 다시 시작해도 복원된다', async () => {
  const originalStorage = (globalThis as { localStorage?: Storage }).localStorage;
  const store = new Map<string, string>([
    ['rhwp-settings', JSON.stringify({ view: { toolbarBasic: false } })],
  ]);
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
    // 새 모듈 인스턴스 = 새 실행. 생성자의 load() 가 저장값을 읽는 경로를 그대로 태운다.
    const fresh = await import('../src/core/user-settings.ts?restart=toolbox');
    const view = fresh.userSettings.getViewSettings();
    assert.equal(view.toolbarBasic, false);
    // 저장값에 없는 항목은 기본값(보임)으로 채운다.
    assert.equal(view.toolbarFormat, true);
  } finally {
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('쪽 배치는 자동이 기본이며 여러 쪽 설정을 정규화해 저장·복원한다', async () => {
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
    userSettings.setPageArrangement({ kind: 'multiple', columns: 12, rows: 0 });
    assert.deepEqual(userSettings.getViewSettings().pageArrangement, {
      kind: 'multiple',
      columns: 8,
      rows: 1,
    });
    const stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.deepEqual(stored.view.pageArrangement, {
      kind: 'multiple',
      columns: 8,
      rows: 1,
    });

    store.set('rhwp-settings', JSON.stringify({ view: {} }));
    const fresh = await import('../src/core/user-settings.ts?restart=page-arrangement');
    assert.deepEqual(fresh.userSettings.getViewSettings().pageArrangement, { kind: 'auto' });
  } finally {
    userSettings.setPageArrangement({ kind: 'auto' });
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('쪽 이동은 세로가 기본이며 가로 휠 설정을 저장·복원한다', async () => {
  const originalStorage = (globalThis as { localStorage?: Storage }).localStorage;
  const store = new Map<string, string>();
  const mockStorage = {
    get length() { return store.size; },
    clear() { store.clear(); },
    getItem(key: string) { return store.get(key) ?? null; },
    key(index: number) { return Array.from(store.keys())[index] ?? null; },
    removeItem(key: string) { store.delete(key); },
    setItem(key: string, value: string) { store.set(key, value); },
  } as Storage;

  (globalThis as { localStorage?: Storage }).localStorage = mockStorage;
  try {
    userSettings.setPageMovement({ direction: 'horizontal', wheelHorizontal: false });
    assert.deepEqual(userSettings.getViewSettings().pageMovement, {
      direction: 'horizontal',
      wheelHorizontal: false,
    });
    const stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.deepEqual(stored.view.pageMovement, {
      direction: 'horizontal',
      wheelHorizontal: false,
    });

    store.set('rhwp-settings', JSON.stringify({ view: {} }));
    const fresh = await import('../src/core/user-settings.ts?restart=page-movement');
    assert.deepEqual(fresh.userSettings.getViewSettings().pageMovement, {
      direction: 'vertical',
      wheelHorizontal: true,
    });
  } finally {
    userSettings.setPageMovement({ direction: 'vertical', wheelHorizontal: true });
    userSettings.setPageArrangement({ kind: 'auto' });
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

test('스킨 설정은 rhwp-settings에 저장되고 잘못된 값은 default로 정규화된다', () => {
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
    userSettings.setThemeSkin('flat');
    assert.equal(userSettings.getThemeSettings().skin, 'flat');
    // 직접 선택은 첫 실행 안내 플래그를 확정한다.
    assert.equal(userSettings.getThemeSettings().skinChosen, true);
    const stored = JSON.parse(store.get('rhwp-settings') ?? '{}');
    assert.equal(stored.theme.skin, 'flat');
    assert.equal(stored.theme.skinChosen, true);

    userSettings.setThemeSkin('oldschool');
    assert.equal(userSettings.getThemeSettings().skin, 'oldschool');

    // 저장소에 없던 잘못된 값은 setter 정규화로 default가 된다.
    userSettings.setThemeSkin('neon' as never);
    assert.equal(userSettings.getThemeSettings().skin, 'default');
  } finally {
    userSettings.setThemeSkin('default');
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});

test('skinChosen 도입 이전에 스킨을 고른 사용자는 첫 실행 안내 대상이 아니다', async () => {
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
    const { normalizeThemeSettings } = await import('../src/core/user-settings.ts');
    // skinChosen 키가 없던 구버전 저장값
    assert.equal(normalizeThemeSettings({ skin: 'flat' }).skinChosen, true);
    assert.equal(normalizeThemeSettings({ skin: 'oldschool' }).skinChosen, true);
    // 기본 스킨이면 아직 고르지 않은 것으로 본다
    assert.equal(normalizeThemeSettings({ skin: 'default' }).skinChosen, false);
    assert.equal(normalizeThemeSettings({}).skinChosen, false);
    // 명시적으로 저장된 값이 있으면 그대로 존중한다
    assert.equal(normalizeThemeSettings({ skin: 'flat', skinChosen: false }).skinChosen, false);
  } finally {
    (globalThis as { localStorage?: Storage }).localStorage = originalStorage;
  }
});
