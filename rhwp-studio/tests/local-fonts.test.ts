import test from 'node:test';
import assert from 'node:assert/strict';

import {
  clearStoredLocalFonts,
  detectLocalFonts,
  getDetectedLocalFonts,
  getLocalFontState,
  getLocalFonts,
  loadStoredLocalFonts,
  resetLocalFontsForTests,
  type LocalFontSnapshot,
} from '../src/core/local-fonts.ts';

const STORAGE_KEY = 'rhwp-local-fonts';

type TestGlobals = typeof globalThis & {
  browser?: unknown;
  chrome?: unknown;
  document?: unknown;
  localStorage?: Storage;
  queryLocalFonts?: unknown;
};

function createStorage(initial: Record<string, string> = {}): Storage {
  const store = new Map(Object.entries(initial));
  return {
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
}

function sortedKo(values: string[]): string[] {
  return [...values].sort((a, b) => a.localeCompare(b, 'ko'));
}

function restoreGlobals(originals: {
  browser: unknown;
  chrome: unknown;
  document: unknown;
  localStorage: Storage | undefined;
  queryLocalFonts: unknown;
}): void {
  const g = globalThis as TestGlobals;
  if (originals.browser === undefined) {
    delete g.browser;
  } else {
    g.browser = originals.browser;
  }
  if (originals.chrome === undefined) {
    delete g.chrome;
  } else {
    g.chrome = originals.chrome;
  }
  if (originals.document === undefined) {
    delete g.document;
  } else {
    g.document = originals.document;
  }
  if (originals.localStorage === undefined) {
    delete g.localStorage;
  } else {
    g.localStorage = originals.localStorage;
  }
  if (originals.queryLocalFonts === undefined) {
    delete g.queryLocalFonts;
  } else {
    g.queryLocalFonts = originals.queryLocalFonts;
  }
}

function firstQuotedFontFamily(value: string): string | undefined {
  const start = value.indexOf('"');
  if (start < 0) return undefined;

  let result = '';
  for (let index = start + 1; index < value.length; index += 1) {
    const char = value[index];
    if (char === '"') return result;
    if (char === '\\' && index + 1 < value.length) {
      result += value[index + 1];
      index += 1;
      continue;
    }
    result += char;
  }
  return undefined;
}

function createProbeDocument(installedFamilies: readonly string[]): unknown {
  const installed = new Set(installedFamilies);
  const context = {
    font: '',
    measureText(text: string) {
      const fallback = this.font.split(',').pop()?.trim() ?? 'sans-serif';
      const target = firstQuotedFontFamily(this.font);
      const fallbackWidth = fallback.includes('monospace') ? 12 : fallback.includes('serif') ? 10 : 11;
      const width = target && installed.has(target) ? fallbackWidth + 3 : fallbackWidth;
      return { width: text.length * width };
    },
  };
  return {
    createElement(tagName: string) {
      if (tagName !== 'canvas') return {};
      return {
        getContext(contextId: string) {
          return contextId === '2d' ? context : null;
        },
      };
    },
  };
}

test('저장된 localStorage snapshot 로드는 queryLocalFonts를 호출하지 않는다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const snapshot: LocalFontSnapshot = {
    version: 1,
    detectedAt: '2026-06-21T00:00:00.000Z',
    families: ['로컬B', '함초롬바탕', '로컬A', '로컬A'],
    source: 'local-font-access',
  };
  let queryCount = 0;

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = createStorage({ [STORAGE_KEY]: JSON.stringify(snapshot) });
  g.queryLocalFonts = async () => {
    queryCount++;
    return [];
  };

  try {
    const loaded = await loadStoredLocalFonts();
    assert.deepEqual(loaded?.families, sortedKo(['로컬A', '로컬B', '함초롬바탕']));
    assert.equal(queryCount, 0);
    assert.deepEqual(getLocalFonts(), sortedKo(['로컬A', '로컬B']));
    assert.deepEqual(getDetectedLocalFonts(), sortedKo(['로컬A', '로컬B', '함초롬바탕']));
    assert.deepEqual(getLocalFontState(), {
      supported: true,
      method: 'local-font-access',
      loaded: true,
      stored: true,
      source: 'local-font-access',
      complete: true,
      storage: 'local-storage',
      count: 3,
      checkedFamilies: sortedKo(['로컬A', '로컬B', '함초롬바탕']),
      detectedAt: '2026-06-21T00:00:00.000Z',
      lastError: null,
    });
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('Chrome 확장 컨텍스트에서는 chrome.storage.local snapshot을 우선 사용한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const snapshot: LocalFontSnapshot = {
    version: 1,
    detectedAt: '2026-06-21T01:00:00.000Z',
    families: ['확장로컬'],
    source: 'local-font-access',
  };
  let localStorageRead = 0;

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = {
    storage: {
      local: {
        get: (_key: string, callback: (items: Record<string, unknown>) => void) => {
          callback({ [STORAGE_KEY]: snapshot });
        },
        set: (_items: Record<string, unknown>, callback: () => void) => {
          callback();
        },
        remove: (_key: string, callback: () => void) => {
          callback();
        },
      },
    },
  };
  g.localStorage = {
    get length() {
      return 0;
    },
    clear() {},
    getItem() {
      localStorageRead++;
      throw new Error('localStorage should not be read');
    },
    key() {
      return null;
    },
    removeItem() {},
    setItem() {},
  } as Storage;
  g.queryLocalFonts = async () => {
    throw new Error('queryLocalFonts should not be called');
  };

  try {
    const loaded = await loadStoredLocalFonts();
    assert.deepEqual(loaded?.families, ['확장로컬']);
    assert.equal(localStorageRead, 0);
    assert.equal(getLocalFontState().storage, 'chrome-storage-local');
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('Firefox 확장 컨텍스트에서는 browser.storage.local snapshot을 사용한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const snapshot: LocalFontSnapshot = {
    version: 1,
    detectedAt: '2026-06-21T01:30:00.000Z',
    families: ['파이어폭스로컬'],
    source: 'font-presence-probe',
    checkedFamilies: ['파이어폭스로컬', '없는글꼴'],
  };

  resetLocalFontsForTests();
  g.chrome = undefined;
  g.browser = {
    storage: {
      local: {
        get: async () => ({ [STORAGE_KEY]: snapshot }),
        set: async () => {},
        remove: async () => {},
      },
    },
  };
  g.localStorage = createStorage();
  g.queryLocalFonts = undefined;
  g.document = createProbeDocument([]);

  try {
    const loaded = await loadStoredLocalFonts();
    const state = getLocalFontState();
    assert.deepEqual(loaded?.families, ['파이어폭스로컬']);
    assert.deepEqual(loaded?.checkedFamilies, sortedKo(['파이어폭스로컬', '없는글꼴']));
    assert.equal(state.storage, 'browser-storage-local');
    assert.equal(state.source, 'font-presence-probe');
    assert.equal(state.complete, false);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('detectLocalFonts는 전체 snapshot을 저장하고 기본 반환은 웹 등록 글꼴을 제외한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  let storedSnapshot: LocalFontSnapshot | null = null;
  let queryCount = 0;

  resetLocalFontsForTests();
  g.browser = undefined;
  g.localStorage = undefined;
  g.chrome = {
    storage: {
      local: {
        get: (_key: string, callback: (items: Record<string, unknown>) => void) => {
          callback({});
        },
        set: (items: Record<string, unknown>, callback: () => void) => {
          storedSnapshot = items[STORAGE_KEY] as LocalFontSnapshot;
          callback();
        },
        remove: (_key: string, callback: () => void) => {
          storedSnapshot = null;
          callback();
        },
      },
    },
  };
  g.queryLocalFonts = async () => {
    queryCount++;
    return [
      { family: '내 로컬', fullName: '내 로컬', postscriptName: 'LocalPS', style: 'Regular' },
      { family: '함초롬바탕', fullName: '함초롬바탕', postscriptName: 'HCRBatang', style: 'Regular' },
      { family: '내 로컬', fullName: '내 로컬', postscriptName: 'LocalPS', style: 'Regular' },
    ];
  };

  try {
    const filtered = await detectLocalFonts({ force: true });
    assert.equal(queryCount, 1);
    assert.deepEqual(filtered, ['내 로컬']);
    assert.deepEqual(getDetectedLocalFonts(), sortedKo(['내 로컬', '함초롬바탕']));
    assert.deepEqual(storedSnapshot?.families, sortedKo(['내 로컬', '함초롬바탕']));
    assert.equal(storedSnapshot?.source, 'local-font-access');
    assert.equal(storedSnapshot?.checkedFamilies, undefined);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('저장소 읽기 실패는 빈 상태로 처리하고 오류 상태만 기록한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = {
    get length() {
      return 0;
    },
    clear() {},
    getItem() {
      throw new Error('storage blocked');
    },
    key() {
      return null;
    },
    removeItem() {},
    setItem() {},
  } as Storage;
  g.queryLocalFonts = undefined;

  try {
    const loaded = await loadStoredLocalFonts();
    const state = getLocalFontState();
    assert.equal(loaded, null);
    assert.equal(state.loaded, true);
    assert.equal(state.stored, false);
    assert.equal(state.storage, 'local-storage');
    assert.equal(state.method, null);
    assert.equal(state.source, null);
    assert.equal(state.complete, false);
    assert.deepEqual(state.checkedFamilies, []);
    assert.equal(state.lastError, 'storage blocked');
    assert.deepEqual(getLocalFonts(), []);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('clearStoredLocalFonts는 저장된 snapshot과 런타임 상태를 초기화한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const snapshot: LocalFontSnapshot = {
    version: 1,
    detectedAt: '2026-06-21T02:00:00.000Z',
    families: ['초기화대상'],
    source: 'local-font-access',
  };
  const storage = createStorage({ [STORAGE_KEY]: JSON.stringify(snapshot) });

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = storage;
  g.queryLocalFonts = async () => {
    throw new Error('queryLocalFonts should not be called');
  };

  try {
    await loadStoredLocalFonts();
    assert.equal(getLocalFontState().stored, true);
    assert.deepEqual(getDetectedLocalFonts(), ['초기화대상']);

    await clearStoredLocalFonts();
    const state = getLocalFontState();
    assert.equal(storage.getItem(STORAGE_KEY), null);
    assert.equal(state.loaded, true);
    assert.equal(state.stored, false);
    assert.equal(state.count, 0);
    assert.equal(state.detectedAt, null);
    assert.deepEqual(state.checkedFamilies, []);
    assert.deepEqual(getDetectedLocalFonts(), []);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('Local Font Access API가 없으면 문서 후보 글꼴만 probe snapshot으로 저장한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  let storedSnapshot: LocalFontSnapshot | null = null;

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = {
    get length() {
      return storedSnapshot ? 1 : 0;
    },
    clear() {
      storedSnapshot = null;
    },
    getItem() {
      return null;
    },
    key() {
      return storedSnapshot ? STORAGE_KEY : null;
    },
    removeItem() {
      storedSnapshot = null;
    },
    setItem(_key: string, value: string) {
      storedSnapshot = JSON.parse(value) as LocalFontSnapshot;
    },
  } as Storage;
  g.queryLocalFonts = undefined;
  g.document = createProbeDocument(['문서로컬']);

  try {
    const fonts = await detectLocalFonts({
      force: true,
      includeRegistered: true,
      candidateFamilies: ['문서로컬', '없는글꼴', '함초롬바탕'],
    });
    const state = getLocalFontState();
    assert.deepEqual(fonts, ['문서로컬']);
    assert.equal(storedSnapshot?.source, 'font-presence-probe');
    assert.deepEqual(storedSnapshot?.families, ['문서로컬']);
    assert.deepEqual(storedSnapshot?.checkedFamilies, sortedKo(['문서로컬', '없는글꼴', '함초롬바탕']));
    assert.equal(state.method, 'font-presence-probe');
    assert.equal(state.source, 'font-presence-probe');
    assert.equal(state.complete, false);
    assert.deepEqual(state.checkedFamilies, sortedKo(['문서로컬', '없는글꼴', '함초롬바탕']));
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});
