import { test } from 'node:test';
import { strict as assert } from 'node:assert';

import { LOCAL_BACKUP_KEY, SETTINGS_SCHEMA_VERSION } from './settings-store.js';

let importSerial = 0;

function createChromeMock(options = {}) {
  const listeners = {
    onCreated: [],
    onChanged: [],
    onDeterminingFilename: [],
  };
  const calls = {
    cancel: [],
    erase: [],
    search: [],
    sessionGet: [],
    sessionRemove: [],
    sessionSet: [],
    tabsCreate: [],
  };
  const searchItems = new Map();
  const sessionItems = new Map(Object.entries(options.session || {}));
  const syncItems = new Map(Object.entries(options.settings || {}));
  const localItems = new Map(Object.entries(options.localSettings || {}));

  async function waitForSettingsRead() {
    if (options.settingsReadDelayMs > 0) {
      await new Promise((resolve) => setTimeout(resolve, options.settingsReadDelayMs));
    }
  }

  function getStorageValues(items, query) {
    if (query == null) return Object.fromEntries(items);
    if (typeof query === 'string') {
      return items.has(query) ? { [query]: items.get(query) } : {};
    }
    if (Array.isArray(query)) {
      return Object.fromEntries(
        query.filter((key) => items.has(key)).map((key) => [key, items.get(key)]),
      );
    }
    if (typeof query === 'object') {
      return Object.fromEntries(
        Object.entries(query).map(([key, fallback]) => [
          key,
          items.has(key) ? items.get(key) : fallback,
        ]),
      );
    }
    return {};
  }

  const chrome = {
    downloads: {
      onCreated: {
        addListener(listener) {
          listeners.onCreated.push(listener);
        },
      },
      onChanged: {
        addListener(listener) {
          listeners.onChanged.push(listener);
        },
      },
      onDeterminingFilename: {
        addListener(listener) {
          listeners.onDeterminingFilename.push(listener);
        },
      },
      async search(query) {
        const searchIndex = calls.search.length;
        calls.search.push(query);
        const delayMs = options.searchDelaysMs?.[searchIndex] ?? 0;
        if (delayMs > 0) {
          await new Promise((resolve) => setTimeout(resolve, delayMs));
        }
        const item = searchItems.get(query.id);
        return item ? [item] : [];
      },
      async cancel(id) {
        calls.cancel.push(id);
      },
      async erase(query) {
        calls.erase.push(query);
      },
    },
    runtime: {
      getURL(path) {
        return `chrome-extension://rhwp/${path}`;
      },
    },
    storage: {
      session: {
        async get(query) {
          calls.sessionGet.push(query);
          return getStorageValues(sessionItems, query);
        },
        async set(items) {
          const setIndex = calls.sessionSet.length;
          calls.sessionSet.push(items);
          const delayMs = options.sessionSetDelaysMs?.[setIndex] ?? 0;
          if (delayMs > 0) {
            await new Promise((resolve) => setTimeout(resolve, delayMs));
          }
          for (const [key, value] of Object.entries(items)) {
            sessionItems.set(key, value);
          }
        },
        async remove(query) {
          calls.sessionRemove.push(query);
          const keys = Array.isArray(query) ? query : [query];
          for (const key of keys) {
            sessionItems.delete(key);
          }
        },
      },
      sync: {
        async get(query) {
          await waitForSettingsRead();
          if (options.syncGetError) throw options.syncGetError;
          return getStorageValues(syncItems, query);
        },
        async set(items) {
          for (const [key, value] of Object.entries(items)) syncItems.set(key, value);
        },
      },
      local: {
        async get(query) {
          await waitForSettingsRead();
          return getStorageValues(localItems, query);
        },
        async set(items) {
          for (const [key, value] of Object.entries(items)) localItems.set(key, value);
        },
      },
    },
    tabs: {
      create(options) {
        calls.tabsCreate.push(options);
        return { id: calls.tabsCreate.length };
      },
    },
  };

  return { chrome, listeners, calls, searchItems, sessionItems };
}

async function importFreshInterceptor() {
  importSerial += 1;
  return import(`./download-interceptor.js?test=${Date.now()}-${importSerial}`);
}

async function withChromeMock(env, run) {
  const originalChrome = globalThis.chrome;
  globalThis.chrome = env.chrome;
  try {
    const module = await importFreshInterceptor();
    module.setupDownloadInterceptor();
    await run(env);
  } finally {
    if (originalChrome === undefined) {
      delete globalThis.chrome;
    } else {
      globalThis.chrome = originalChrome;
    }
  }
}

async function flushAsyncWork() {
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
}

function lastListener(list) {
  return list[list.length - 1];
}

test('Chrome interceptor registers observers, not onDeterminingFilename', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners }) => {
    assert.equal(listeners.onCreated.length, 1);
    assert.equal(listeners.onChanged.length, 1);
    assert.equal(listeners.onDeterminingFilename.length, 0);
  });
});

test('non-HWP blob PDF download is ignored', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 101,
      url: 'blob:chrome-extension://other/11111111-1111-4111-8111-111111111111',
      filename: '11111111-1111-4111-8111-111111111111.pdf',
      mime: 'application/pdf',
    });
    await flushAsyncWork();

    searchItems.set(101, {
      id: 101,
      url: 'blob:chrome-extension://other/11111111-1111-4111-8111-111111111111',
      filename: '11111111-1111-4111-8111-111111111111.pdf',
      mime: 'application/pdf',
    });
    await listeners.onChanged[0]({
      id: 101,
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, []);
    assert.deepEqual(calls.cancel, []);
    assert.deepEqual(calls.erase, []);
  });
});

test('HWP download opens viewer once', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 201,
      url: 'https://example.com/sample.hwp',
      filename: 'sample.hwp',
      mime: 'application/x-hwp',
      fileSize: 1024,
    });
    await flushAsyncWork();

    await listeners.onChanged[0]({
      id: 201,
      filename: { current: '/Users/melee/Downloads/sample.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.match(calls.tabsCreate[0].url, /^chrome-extension:\/\/rhwp\/viewer\.html\?/);
    assert.match(calls.tabsCreate[0].url, /filename=sample\.hwp/);
    assert.deepEqual(calls.search, []);
  });
});

test('autoOpen=false does not open viewer', async () => {
  const env = createChromeMock({ settings: { autoOpen: false } });

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 301,
      url: 'https://example.com/sample.hwpx',
      filename: 'sample.hwpx',
      mime: 'application/hwp+zip',
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('filename finalized in onChanged is rechecked with downloads.search', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 401,
      url: 'https://example.com/download?id=401',
      filename: 'download',
      mime: 'application/octet-stream',
    });
    await flushAsyncWork();

    searchItems.set(401, {
      id: 401,
      url: 'https://example.com/download?id=401',
      filename: 'sample.hwp',
      mime: 'application/octet-stream',
    });
    await listeners.onChanged[0]({
      id: 401,
      filename: { current: '/Users/melee/Downloads/sample.hwp' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 401 }]);
    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('local file HWP is opened and suppressed best-effort', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 501,
      url: 'file:///Users/melee/Downloads/local.hwp',
      filename: 'local.hwp',
      mime: 'application/x-hwp',
    });
    await flushAsyncWork();
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.deepEqual(calls.cancel, [501]);
    assert.deepEqual(calls.erase, [{ id: 501 }]);
  });
});

// #1498: onChanged 단독(= onCreated 미관측, 과거 다운로드 기록)으로는 뷰어를 열지 않는다.
test('past download (onChanged only, no onCreated) does not open the viewer', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    // service worker 재기동 후 과거 HWP 다운로드 항목에 onChanged 만 발화하는 상황.
    searchItems.set(900, {
      id: 900,
      url: 'https://example.com/old.hwp',
      filename: 'old.hwp',
      mime: 'application/x-hwp',
    });
    await listeners.onChanged[0]({
      id: 900,
      filename: { current: '/Users/melee/Downloads/old.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    // seen 에 없으므로 재조회/오픈 모두 일어나지 않아야 한다.
    assert.deepEqual(calls.search, [], 'onChanged 단독은 downloads.search 를 호출하지 않아야 함');
    assert.deepEqual(calls.tabsCreate, [], 'onChanged 단독은 뷰어를 열지 않아야 함');
  });
});

// #1498 v2: Chrome 이 과거 완료 항목을 onCreated 로 전달해도 뷰어를 열지 않는다.
test('past completed download delivered through onCreated does not open the viewer', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 902,
      url: 'https://example.com/old-created.hwp',
      finalUrl: 'https://example.com/old-created.hwp',
      filename: 'old-created.hwp',
      mime: 'application/x-hwp',
      state: 'complete',
      startTime: '2000-01-01T00:00:00.000Z',
      endTime: '2000-01-01T00:00:01.000Z',
      fileSize: 1024,
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, [], '과거 완료 항목 onCreated 는 뷰어를 열지 않아야 함');
    assert.deepEqual(calls.cancel, []);
    assert.deepEqual(calls.erase, []);
  });
});

// #1498 v2: onChanged 재조회 결과가 과거 항목이면 seen 에 있어도 뷰어를 열지 않는다.
test('past download returned from onChanged search does not open the viewer', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 903,
      url: 'https://example.com/download?id=903',
      filename: 'download',
      mime: 'application/octet-stream',
    });
    await flushAsyncWork();

    searchItems.set(903, {
      id: 903,
      url: 'https://example.com/old-search.hwp',
      filename: 'old-search.hwp',
      mime: 'application/x-hwp',
      state: 'complete',
      startTime: '2000-01-01T00:00:00.000Z',
      endTime: '2000-01-01T00:00:01.000Z',
    });
    await listeners.onChanged[0]({
      id: 903,
      filename: { current: '/Users/melee/Downloads/old-search.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 903 }]);
    assert.deepEqual(calls.tabsCreate, [], '재조회된 과거 항목은 뷰어를 열지 않아야 함');
  });
});

// #1498: onCreated 로 관측한 새 다운로드는 onChanged 재판정으로 정상 오픈된다.
test('new download seen via onCreated is opened on onChanged recheck', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 901,
      url: 'https://example.com/download?id=901',
      filename: 'download',
      mime: 'application/octet-stream',
    });
    await flushAsyncWork();

    searchItems.set(901, {
      id: 901,
      url: 'https://example.com/download?id=901',
      filename: 'fresh.hwp',
      mime: 'application/octet-stream',
    });
    await listeners.onChanged[0]({
      id: 901,
      filename: { current: '/Users/melee/Downloads/fresh.hwp' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 901 }]);
    assert.equal(calls.tabsCreate.length, 1, '새 다운로드는 정상 오픈되어야 함');
  });
});

test('download tracked before service worker restart opens on onChanged recheck', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 904,
      url: 'https://example.com/download?id=904',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 0);
    assert.equal(calls.sessionSet.length, 1);
  });

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    searchItems.set(904, {
      id: 904,
      url: 'https://example.com/download?id=904',
      filename: 'restart-fresh.hwp',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await lastListener(listeners.onChanged)({
      id: 904,
      filename: { current: '/Users/melee/Downloads/restart-fresh.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1, '재시작 후 filename 확정 항목은 1회 열려야 함');
    assert.match(calls.tabsCreate[0].url, /filename=restart-fresh\.hwp/);
  });
});

test('handled state in storage prevents duplicate open after restart', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 905,
      url: 'https://example.com/already-opened.hwp',
      filename: 'already-opened.hwp',
      mime: 'application/x-hwp',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
  });

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    searchItems.set(905, {
      id: 905,
      url: 'https://example.com/already-opened.hwp',
      filename: 'already-opened.hwp',
      mime: 'application/x-hwp',
      startTime: new Date().toISOString(),
    });
    await lastListener(listeners.onChanged)({
      id: 905,
      filename: { current: '/Users/melee/Downloads/already-opened.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1, 'handled 상태는 재시작 후에도 중복 open을 막아야 함');
  });
});

test('same download id with multiple changed events opens once', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 906,
      url: 'https://example.com/download?id=906',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    searchItems.set(906, {
      id: 906,
      url: 'https://example.com/download?id=906',
      finalUrl: 'https://cdn.example.com/fresh-multi.hwp',
      filename: 'fresh-multi.hwp',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });

    await listeners.onChanged[0]({
      id: 906,
      filename: { current: '/Users/melee/Downloads/fresh-multi.hwp' },
    });
    await flushAsyncWork();
    await listeners.onChanged[0]({
      id: 906,
      finalUrl: { current: 'https://cdn.example.com/fresh-multi.hwp' },
    });
    await flushAsyncWork();
    await listeners.onChanged[0]({
      id: 906,
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.deepEqual(calls.search, [{ id: 906 }]);
  });
});

test('same download id with concurrent changed events opens once', async () => {
  const env = createChromeMock({
    settingsReadDelayMs: 10,
    searchDelaysMs: [0, 40],
  });

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 907,
      url: 'https://example.com/download?id=907',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    searchItems.set(907, {
      id: 907,
      url: 'https://example.com/download?id=907',
      finalUrl: 'https://cdn.example.com/concurrent.hwp',
      filename: 'concurrent.hwp',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });

    await Promise.all([
      listeners.onChanged[0]({
        id: 907,
        filename: { current: '/Users/melee/Downloads/concurrent.hwp' },
      }),
      listeners.onChanged[0]({
        id: 907,
        finalUrl: { current: 'https://cdn.example.com/concurrent.hwp' },
      }),
    ]);
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1, '동시 이벤트도 download id당 탭을 1개만 열어야 함');
  });
});

test('first matching complete event preserves handled state against later changes', async () => {
  const env = createChromeMock();

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 909,
      url: 'https://example.com/download?id=909',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    searchItems.set(909, {
      id: 909,
      url: 'https://example.com/download?id=909',
      filename: 'terminal-first.hwp',
      mime: 'application/octet-stream',
      state: 'complete',
      startTime: new Date().toISOString(),
      endTime: new Date().toISOString(),
    });
    await listeners.onChanged[0]({
      id: 909,
      filename: { current: '/Users/melee/Downloads/terminal-first.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    await listeners.onChanged[0]({
      id: 909,
      finalUrl: { current: 'https://cdn.example.com/terminal-first.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1, 'terminal 상태가 handled marker를 지우면 안 됨');
  });
});

test('concurrent terminal event cannot overwrite an in-flight handled result', async () => {
  const env = createChromeMock({
    settingsReadDelayMs: 20,
    sessionSetDelaysMs: [0, 40, 0],
  });

  await withChromeMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 910,
      url: 'https://example.com/download?id=910',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    searchItems.set(910, {
      id: 910,
      url: 'https://example.com/download?id=910',
      filename: 'concurrent-terminal.hwp',
      mime: 'application/octet-stream',
      state: 'complete',
      startTime: new Date().toISOString(),
      endTime: new Date().toISOString(),
    });
    await Promise.all([
      listeners.onChanged[0]({
        id: 910,
        filename: { current: '/Users/melee/Downloads/concurrent-terminal.hwp' },
      }),
      listeners.onChanged[0]({
        id: 910,
        state: { current: 'complete' },
      }),
    ]);
    await flushAsyncWork();

    await listeners.onChanged[0]({
      id: 910,
      finalUrl: { current: 'https://cdn.example.com/concurrent-terminal.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1, '동시 terminal 기록도 handled marker를 보존해야 함');
  });
});

test('sync read failure never authorizes automatic opening from a local true snapshot', async () => {
  const localSnapshot = {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    updatedAt: 1,
    settings: {
      autoOpen: true,
      showBadges: true,
      hoverPreview: true,
      disableExternalWebFonts: false,
    },
  };
  const env = createChromeMock({
    syncGetError: new Error('temporary sync failure'),
    localSettings: { [LOCAL_BACKUP_KEY]: localSnapshot },
  });

  await withChromeMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 908,
      url: 'https://example.com/fail-closed.hwp',
      filename: 'fail-closed.hwp',
      mime: 'application/x-hwp',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, [], 'sync 상태가 불명확하면 자동 탭을 열지 않아야 함');
  });
});
