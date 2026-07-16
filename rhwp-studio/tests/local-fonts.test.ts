import test from 'node:test';
import assert from 'node:assert/strict';

import {
  clearStoredLocalFonts,
  detectLocalFonts,
  getDetectedLocalFonts,
  getLocalFontRecords,
  getLocalFontState,
  getLocalFonts,
  localFontFaceKey,
  loadLocalFontBytes,
  loadLocalFontBytesFor,
  loadStoredLocalFonts,
  resetLocalFontsForTests,
  resolveLocalFont,
  type LocalFontSnapshot,
} from '../src/core/local-fonts.ts';
import { analyzeDocumentFonts } from '../src/core/document-font-status.ts';
import { fontFamilyChainForDisplay } from '../src/core/font-substitution.ts';

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

function utf16Be(value: string): Uint8Array {
  const bytes = new Uint8Array(value.length * 2);
  for (let index = 0; index < value.length; index += 1) {
    const codeUnit = value.charCodeAt(index);
    bytes[index * 2] = codeUnit >> 8;
    bytes[index * 2 + 1] = codeUnit & 0xff;
  }
  return bytes;
}

function createSfntWithNameRecords(entries: ReadonlyArray<{
  nameId: number;
  value: string;
  platformId?: number;
  bytes?: Uint8Array;
}>): Uint8Array {
  const encoded = entries.map(entry => ({ ...entry, bytes: entry.bytes ?? utf16Be(entry.value) }));
  const nameTableOffset = 12 + 16;
  const stringsOffset = 6 + encoded.length * 12;
  const nameTableLength = stringsOffset + encoded.reduce((sum, entry) => sum + entry.bytes.length, 0);
  const bytes = new Uint8Array(nameTableOffset + nameTableLength);
  const view = new DataView(bytes.buffer);

  view.setUint32(0, 0x00010000, false);
  view.setUint16(4, 1, false);
  bytes.set([0x6e, 0x61, 0x6d, 0x65], 12);
  view.setUint32(20, nameTableOffset, false);
  view.setUint32(24, nameTableLength, false);

  view.setUint16(nameTableOffset, 0, false);
  view.setUint16(nameTableOffset + 2, encoded.length, false);
  view.setUint16(nameTableOffset + 4, stringsOffset, false);
  let stringCursor = 0;
  encoded.forEach((entry, index) => {
    const recordOffset = nameTableOffset + 6 + index * 12;
    view.setUint16(recordOffset, entry.platformId ?? 3, false);
    view.setUint16(recordOffset + 2, 1, false);
    view.setUint16(recordOffset + 4, 0x0412, false);
    view.setUint16(recordOffset + 6, entry.nameId, false);
    view.setUint16(recordOffset + 8, entry.bytes.length, false);
    view.setUint16(recordOffset + 10, stringCursor, false);
    bytes.set(entry.bytes, nameTableOffset + stringsOffset + stringCursor);
    stringCursor += entry.bytes.length;
  });
  return bytes;
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

test('저장된 v2 snapshot의 반복 별칭 해석은 전체 face를 다시 정규화하지 않는다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const records = Array.from({ length: 256 }, (_, index) => ({
    family: `Family ${index}`,
    fullName: `Family ${index} Regular`,
    postscriptName: `Family${index}-Regular`,
    style: 'Regular',
    displayName: `글꼴 ${index}`,
    aliases: [`Family ${index}`, `Family ${index} Regular`, `별칭 ${index}`],
  }));
  const snapshot: LocalFontSnapshot = {
    version: 2,
    detectedAt: '2026-07-12T00:00:00.000Z',
    families: records.map(record => record.family),
    fontRecords: records,
    source: 'local-font-access',
  };
  const originalNormalize = String.prototype.normalize;
  let normalizeCalls = 0;

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = createStorage({ [STORAGE_KEY]: JSON.stringify(snapshot) });
  g.queryLocalFonts = undefined;

  try {
    await loadStoredLocalFonts();
    String.prototype.normalize = function(this: string, form?: string): string {
      normalizeCalls++;
      return originalNormalize.call(this, form);
    };

    for (let index = 0; index < 40; index++) {
      assert.equal(resolveLocalFont('별칭 255')?.postscriptName, 'Family255-Regular');
      assert.equal(resolveLocalFont('Family 128 Regular')?.family, 'Family 128');
    }
    assert.equal(normalizeCalls, 80);
  } finally {
    String.prototype.normalize = originalNormalize;
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

test('SFNT 지역화 이름을 보존해 HWP 한글 full name을 영문 family로 해석한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const storage = createStorage();

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = storage;
  g.queryLocalFonts = async () => [{
    family: '08SeoulHangang',
    fullName: '08SeoulHangang M',
    postscriptName: 'SeoulHangangM',
    style: 'M',
    blob: async () => new Blob([createSfntWithNameRecords([
      { nameId: 1, value: '08서울한강체' },
      { nameId: 2, value: 'M' },
      { nameId: 4, value: '08서울한강체 M' },
      { nameId: 6, value: 'SeoulHangangM' },
    ])]),
  }];

  try {
    const fonts = await detectLocalFonts({ force: true, includeRegistered: true });
    const record = resolveLocalFont('08서울한강체 M');
    const englishRecord = resolveLocalFont('08SeoulHangang M');
    const report = analyzeDocumentFonts(['08서울한강체 M']);
    const cssChain = fontFamilyChainForDisplay('08서울한강체 M');
    const stored = JSON.parse(storage.getItem(STORAGE_KEY) ?? '{}') as LocalFontSnapshot;

    assert.deepEqual(fonts, ['08서울한강체 M']);
    assert.equal(record?.family, '08SeoulHangang');
    assert.equal(englishRecord?.postscriptName, 'SeoulHangangM');
    assert.equal(record?.postscriptName, 'SeoulHangangM');
    assert.ok(record?.aliases.includes('08서울한강체 M'));
    assert.deepEqual(getLocalFontRecords().map(item => item.displayName), ['08서울한강체 M']);
    assert.deepEqual(report.fonts.map(item => [item.fontName, item.status, item.source]), [
      ['08서울한강체 M', 'available', 'local'],
    ]);
    assert.match(cssChain, /^"08SeoulHangang"/);
    assert.equal(stored.version, 2);
    assert.equal('blob' in (stored.fontRecords?.[0] ?? {}), false);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('legacy Macintosh name record는 표시 이름에 섞지 않는다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const storage = createStorage();
  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = storage;
  g.queryLocalFonts = async () => [{
    family: 'Browser Font',
    fullName: 'Browser Font Regular',
    postscriptName: 'BrowserFont-Regular',
    style: 'Regular',
    blob: async () => new Blob([createSfntWithNameRecords([
      { nameId: 1, value: '!legacy', platformId: 1, bytes: new TextEncoder().encode('!legacy') },
      { nameId: 4, value: '!legacy display', platformId: 1, bytes: new TextEncoder().encode('!legacy display') },
    ])]),
  }];

  try {
    assert.deepEqual(await detectLocalFonts({ force: true }), ['Browser Font Regular']);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('저장된 깨진 표시 이름은 브라우저 family 이름으로 복구한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const snapshot: LocalFontSnapshot = {
    version: 2,
    detectedAt: '2026-07-12T00:00:00.000Z',
    families: ['Browser Font'],
    fontRecords: [{
      family: 'Browser Font',
      fullName: 'Browser Font Regular',
      postscriptName: 'BrowserFont-Regular',
      style: 'Regular',
      displayName: '»Ã깨진 이름',
      aliases: ['Browser Font', 'Browser Font Regular', '»Ã깨진 이름'],
    }],
    source: 'local-font-access',
  };
  const storage = createStorage({ [STORAGE_KEY]: JSON.stringify(snapshot) });
  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = storage;
  g.queryLocalFonts = undefined;

  try {
    await loadStoredLocalFonts();
    assert.deepEqual(getLocalFonts(), ['Browser Font Regular']);
  } finally {
    await clearStoredLocalFonts();
    resetLocalFontsForTests();
    restoreGlobals(originals);
  }
});

test('CanvasKit용 SFNT 바이트는 여러 PostScript face를 일괄 조회하고 동시 요청만 공유한다', async () => {
  const g = globalThis as TestGlobals;
  const originals = {
    browser: g.browser,
    chrome: g.chrome,
    document: g.document,
    localStorage: g.localStorage,
    queryLocalFonts: g.queryLocalFonts,
  };
  const storage = createStorage();
  const nameTableBytes = createSfntWithNameRecords([
    { nameId: 1, value: '08서울한강체' },
    { nameId: 2, value: 'M' },
    { nameId: 4, value: '08서울한강체 M' },
    { nameId: 6, value: 'SeoulHangangM' },
  ]);
  const secondNameTableBytes = createSfntWithNameRecords([
    { nameId: 1, value: '08서울한강체' },
    { nameId: 2, value: 'L' },
    { nameId: 4, value: '08서울한강체 L' },
    { nameId: 6, value: 'SeoulHangangL' },
  ]);
  const fontBytes = new Uint8Array(nameTableBytes.length + 4);
  fontBytes.set(nameTableBytes);
  fontBytes.set([0xde, 0xad, 0xbe, 0xef], nameTableBytes.length);
  const secondFontBytes = new Uint8Array(secondNameTableBytes.length + 4);
  secondFontBytes.set(secondNameTableBytes);
  secondFontBytes.set([0xca, 0xfe, 0xba, 0xbe], secondNameTableBytes.length);
  const queryCalls: Array<string[] | undefined> = [];

  resetLocalFontsForTests();
  g.browser = undefined;
  g.chrome = undefined;
  g.localStorage = storage;
  g.queryLocalFonts = async (options?: { postscriptNames?: string[] }) => {
    queryCalls.push(options?.postscriptNames);
    return [
      {
        family: '08SeoulHangang',
        fullName: '08SeoulHangang M',
        postscriptName: 'SeoulHangangM',
        style: 'M',
        blob: async () => new Blob([options ? fontBytes : nameTableBytes]),
      },
      {
        family: '08SeoulHangang',
        fullName: '08SeoulHangang L',
        postscriptName: 'SeoulHangangL',
        style: 'L',
        blob: async () => new Blob([options ? secondFontBytes : secondNameTableBytes]),
      },
    ];
  };

  try {
    await detectLocalFonts({ force: true, includeRegistered: true });
    const [all, first] = await Promise.all([
      loadLocalFontBytesFor(['08서울한강체 M', '08서울한강체 L']),
      loadLocalFontBytes('08서울한강체 M'),
    ]);
    const hangangRecord = resolveLocalFont('08서울한강체 M');
    const hangangLightRecord = resolveLocalFont('08서울한강체 L');

    assert.deepEqual(new Uint8Array(all.get(localFontFaceKey(hangangRecord!)) ?? new ArrayBuffer(0)), fontBytes);
    assert.deepEqual(new Uint8Array(all.get(localFontFaceKey(hangangLightRecord!)) ?? new ArrayBuffer(0)), secondFontBytes);
    assert.deepEqual(new Uint8Array(first ?? new ArrayBuffer(0)), fontBytes);
    assert.equal(resolveLocalFont('08SeoulHangang'), null);
    assert.equal(queryCalls.length, 2);
    assert.equal(queryCalls[0], undefined);
    assert.deepEqual(new Set(queryCalls[1]), new Set(['SeoulHangangM', 'SeoulHangangL']));
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
