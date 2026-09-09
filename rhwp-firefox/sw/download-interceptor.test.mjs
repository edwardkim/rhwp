import { test } from 'node:test';
import { strict as assert } from 'node:assert';

let importSerial = 0;

function createBrowserMock(options = {}) {
  const listeners = {
    onCreated: [],
    onChanged: [],
  };
  const calls = {
    search: [],
    sessionGet: [],
    sessionRemove: [],
    sessionSet: [],
    tabsCreate: [],
  };
  const searchItems = new Map();
  const sessionItems = new Map(Object.entries(options.session || {}));

  function getSessionValues(query) {
    if (query == null) return Object.fromEntries(sessionItems);
    if (typeof query === 'string') {
      return { [query]: sessionItems.get(query) };
    }
    if (Array.isArray(query)) {
      return Object.fromEntries(query.map((key) => [key, sessionItems.get(key)]));
    }
    if (typeof query === 'object') {
      return Object.fromEntries(
        Object.entries(query).map(([key, fallback]) => [
          key,
          sessionItems.has(key) ? sessionItems.get(key) : fallback,
        ]),
      );
    }
    return {};
  }

  const storage = {
    sync: {
      async get(defaults) {
        return { ...defaults, ...(options.settings || {}) };
      },
    },
  };

  if (options.session !== false) {
    storage.session = {
      async get(query) {
        calls.sessionGet.push(query);
        return getSessionValues(query);
      },
      async set(items) {
        calls.sessionSet.push(items);
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
    };
  }

  const browser = {
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
      async search(query) {
        calls.search.push(query);
        const item = searchItems.get(query.id);
        return item ? [item] : [];
      },
    },
    runtime: {
      getURL(path) {
        return `moz-extension://rhwp/${path}`;
      },
    },
    storage,
    tabs: {
      async create(options) {
        calls.tabsCreate.push(options);
        return { id: calls.tabsCreate.length };
      },
    },
  };

  return { browser, listeners, calls, searchItems, sessionItems };
}

async function importFreshInterceptor() {
  importSerial += 1;
  return import(`./download-interceptor.js?test=${Date.now()}-${importSerial}`);
}

async function withBrowserMock(env, run) {
  const originalBrowser = globalThis.browser;
  globalThis.browser = env.browser;
  try {
    const module = await importFreshInterceptor();
    module.setupDownloadInterceptor();
    await run(env);
  } finally {
    if (originalBrowser === undefined) {
      delete globalThis.browser;
    } else {
      globalThis.browser = originalBrowser;
    }
  }
}

async function flushAsyncWork() {
  for (let i = 0; i < 5; i += 1) {
    await Promise.resolve();
    await new Promise((resolve) => setImmediate(resolve));
  }
}

function lastListener(list) {
  return list[list.length - 1];
}

test('Firefox interceptor registers download observers', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners }) => {
    assert.equal(listeners.onCreated.length, 1);
    assert.equal(listeners.onChanged.length, 1);
  });
});

test('HWP download opens viewer once', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 201,
      url: 'https://example.com/sample.hwp',
      filename: 'sample.hwp',
      mime: 'application/x-hwp',
      fileSize: 1024,
    });
    await flushAsyncWork();

    listeners.onChanged[0]({
      id: 201,
      filename: { current: '/Users/melee/Downloads/sample.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.match(calls.tabsCreate[0].url, /^moz-extension:\/\/rhwp\/viewer\.html\?/);
    assert.match(calls.tabsCreate[0].url, /filename=sample\.hwp/);
    assert.deepEqual(calls.search, []);
  });
});

test('autoOpen=false does not open viewer', async () => {
  const env = createBrowserMock({ settings: { autoOpen: false } });

  await withBrowserMock(env, async ({ listeners, calls }) => {
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
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
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
    listeners.onChanged[0]({
      id: 401,
      filename: { current: '/Users/melee/Downloads/sample.hwp' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 401 }]);
    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('Firefox ignores XLSX filename even when source URL ends with hwp (#6534)', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 402,
      url: 'https://public.example.go.kr/download/report.hwp',
      filename: '/Users/melee/Downloads/public-report.xlsx',
      mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('Firefox defers HWP URL until XLSX filename finalization (#6534)', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    listeners.onCreated[0]({
      id: 403,
      url: 'https://public.example.go.kr/download/report.hwp',
      filename: 'download',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, [], 'URL 단독 근거는 onCreated에서 보류해야 함');

    searchItems.set(403, {
      id: 403,
      url: 'https://public.example.go.kr/download/report.hwp',
      filename: '/Users/melee/Downloads/final-report.xlsx',
      mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      startTime: new Date().toISOString(),
    });
    listeners.onChanged[0]({
      id: 403,
      filename: { current: '/Users/melee/Downloads/final-report.xlsx' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 403 }]);
    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('Firefox opens extensionless HWP MIME only after terminal recheck (#198/#6534)', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    const item = {
      id: 404,
      url: 'https://public.example.go.kr/download?id=404',
      filename: 'download',
      mime: 'application/x-hwp',
      startTime: new Date().toISOString(),
    };
    listeners.onCreated[0](item);
    await flushAsyncWork();

    assert.deepEqual(calls.tabsCreate, [], 'MIME 단독 근거는 onCreated에서 보류해야 함');

    searchItems.set(404, { ...item, state: 'complete', endTime: new Date().toISOString() });
    listeners.onChanged[0]({
      id: 404,
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 404 }]);
    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('past download onChanged only does not open the viewer', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    searchItems.set(900, {
      id: 900,
      url: 'https://example.com/old.hwp',
      filename: 'old.hwp',
      mime: 'application/x-hwp',
    });
    listeners.onChanged[0]({
      id: 900,
      filename: { current: '/Users/melee/Downloads/old.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, []);
    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('past completed download delivered through onCreated does not open the viewer', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls }) => {
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

    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('past download returned from onChanged search does not open the viewer', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
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
    listeners.onChanged[0]({
      id: 903,
      filename: { current: '/Users/melee/Downloads/old-search.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();

    assert.deepEqual(calls.search, [{ id: 903 }]);
    assert.deepEqual(calls.tabsCreate, []);
  });
});

test('download tracked before event page restart opens on onChanged recheck', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls }) => {
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

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    searchItems.set(904, {
      id: 904,
      url: 'https://example.com/download?id=904',
      filename: 'restart-fresh.hwp',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });
    lastListener(listeners.onChanged)({
      id: 904,
      filename: { current: '/Users/melee/Downloads/restart-fresh.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.match(calls.tabsCreate[0].url, /filename=restart-fresh\.hwp/);
  });
});

test('handled state in storage prevents duplicate open after restart', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls }) => {
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

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    searchItems.set(905, {
      id: 905,
      url: 'https://example.com/already-opened.hwp',
      filename: 'already-opened.hwp',
      mime: 'application/x-hwp',
      startTime: new Date().toISOString(),
    });
    lastListener(listeners.onChanged)({
      id: 905,
      filename: { current: '/Users/melee/Downloads/already-opened.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('same download id with terminal changed event opens once', async () => {
  const env = createBrowserMock();

  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
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
      finalUrl: 'https://cdn.example.com/fresh-terminal.hwp',
      filename: 'fresh-terminal.hwp',
      mime: 'application/octet-stream',
      startTime: new Date().toISOString(),
    });

    listeners.onChanged[0]({
      id: 906,
      filename: { current: '/Users/melee/Downloads/fresh-terminal.hwp' },
      state: { current: 'complete' },
    });
    await flushAsyncWork();
    listeners.onChanged[0]({
      id: 906,
      finalUrl: { current: 'https://cdn.example.com/fresh-terminal.hwp' },
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.deepEqual(calls.search, [{ id: 906 }]);
  });
});

test('memory fallback still handles a direct HWP download', async () => {
  const env = createBrowserMock({ session: false });

  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0]({
      id: 907,
      url: 'https://example.com/fallback.hwp',
      filename: 'fallback.hwp',
      mime: 'application/x-hwp',
    });
    await flushAsyncWork();

    assert.equal(calls.tabsCreate.length, 1);
    assert.deepEqual(calls.sessionGet, []);
    assert.deepEqual(calls.sessionSet, []);
  });
});

function hwpItem(id, overrides = {}) {
  return {
    id, url: 'https://example.com/sample.hwp', filename: 'sample.hwp',
    startTime: new Date().toISOString(), ...overrides,
  };
}

for (const session of [{}, false]) {
  test(`Firefox serializes overlapping created/changed/terminal events (session=${session !== false}) (#6964)`, async () => {
    const env = createBrowserMock({ session });
    const gate = Promise.withResolvers();
    env.browser.storage.sync.get = async defaults => { await gate.promise; return defaults; };
    await withBrowserMock(env, async ({ listeners, calls, searchItems, sessionItems }) => {
      const item = hwpItem(6964);
      searchItems.set(item.id, item);
      listeners.onCreated[0](item);
      await flushAsyncWork(); // paused after tracking, before marking handled
      listeners.onChanged[0]({ id: item.id, filename: { current: item.filename } });
      listeners.onChanged[0]({ id: item.id, state: { current: 'complete' } });
      listeners.onCreated[0](item);
      await flushAsyncWork();
      gate.resolve();
      await flushAsyncWork();
      assert.equal(calls.tabsCreate.length, 1);
      if (session !== false) {
        const state = sessionItems.get('rhwpDownloadState:6964');
        assert.ok(state.handledAt);
        assert.ok(state.terminalAt);
      }
      listeners.onChanged[0]({ id: item.id, state: { current: 'complete' } });
      await flushAsyncWork();
      assert.equal(calls.tabsCreate.length, 1, 'late terminal must retain handled state');
    });
  });
}

test('Firefox preserves a filename event arriving before created storage completes (#6964)', async () => {
  const env = createBrowserMock();
  const gate = Promise.withResolvers();
  const set = env.browser.storage.session.set;
  let first = true;
  env.browser.storage.session.set = async items => {
    if (first) { first = false; await gate.promise; }
    await set(items);
  };
  await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
    const item = hwpItem(6965, { filename: 'download', url: 'https://example.com/download' });
    searchItems.set(item.id, { ...item, filename: 'final.hwp' });
    listeners.onCreated[0](item);
    listeners.onChanged[0]({ id: item.id, filename: { current: 'final.hwp' } });
    await flushAsyncWork();
    gate.resolve();
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('Firefox does not block other IDs while settings are pending (#6964)', async () => {
  const env = createBrowserMock();
  const gate = Promise.withResolvers();
  let first = true;
  env.browser.storage.sync.get = async defaults => {
    if (first) { first = false; await gate.promise; }
    return defaults;
  };
  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0](hwpItem(6966));
    await flushAsyncWork();
    listeners.onCreated[0](hwpItem(6967));
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 1);
    gate.resolve();
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 2);
  });
});

test('Firefox continues the same ID after a storage read fails (#6964)', async t => {
  const env = createBrowserMock();
  const get = env.browser.storage.session.get;
  let first = true;
  env.browser.storage.session.get = async key => {
    if (first) { first = false; throw new Error('session temporarily unavailable'); }
    return get(key);
  };
  const errors = t.mock.method(console, 'error', () => {});
  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0](hwpItem(6968));
    listeners.onCreated[0](hwpItem(6968));
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 1);
    assert.equal(errors.mock.callCount(), 1);
  });
});

for (const [url, expectedTabs] of [
  ['blob:moz-extension://rhwp/saved-output', 0],
  ['blob:https://example.com/external-output', 1],
  ['blob:moz-extension://another-extension/external-output', 1],
  ['blob:moz-extension://rhwp-lookalike/external-output', 1],
  ['https://example.com/sample.hwp', 1],
]) {
  test(`firefox preserves external downloads and skips only its own Blob: ${url} (#6964)`, async () => {
    const env = createBrowserMock();
    await withBrowserMock(env, async ({ listeners, calls, searchItems }) => {
      const item = { id: 6970, url, filename: '/Downloads/saved.hwp', startTime: new Date().toISOString() };
      searchItems.set(item.id, item);
      listeners.onCreated[0](item);
      await flushAsyncWork();
      listeners.onChanged[0]({ id: item.id, filename: { current: item.filename } });
      listeners.onChanged[0]({ id: item.id, state: { current: 'complete' } });
      await flushAsyncWork();
      assert.equal(calls.tabsCreate.length, expectedTabs);
      assert.equal(item.url, url);
      assert.equal(item.filename, '/Downloads/saved.hwp');
      if (calls.cancel) assert.deepEqual(calls.cancel, []);
      if (calls.erase) assert.deepEqual(calls.erase, []);
    });
  });
}

test('Firefox uses receipt time for freshness even when an earlier event stalls (#6964)', async t => {
  const env = createBrowserMock();
  const gate = Promise.withResolvers();
  const get = env.browser.storage.session.get;
  let first = true;
  env.browser.storage.session.get = async key => {
    if (first) { first = false; await gate.promise; }
    return get(key);
  };
  const receivedAt = Date.now();
  let now = receivedAt;
  t.mock.method(Date, 'now', () => now);
  await withBrowserMock(env, async ({ listeners, calls }) => {
    const item = hwpItem(6969, { startTime: new Date(receivedAt).toISOString() });
    listeners.onCreated[0]({ ...item, filename: 'download', url: 'https://example.com/download' });
    listeners.onCreated[0](item);
    await flushAsyncWork();
    now += 6000; // longer than the 5-second fresh-download window
    gate.resolve();
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 1);
  });
});

test('Firefox starts the first storage request during event delivery (#6964)', async () => {
  const env = createBrowserMock();
  await withBrowserMock(env, async ({ listeners, calls }) => {
    listeners.onCreated[0](hwpItem(6971));
    assert.deepEqual(calls.sessionGet, ['rhwpDownloadState:6971'],
      'the first browser API call must keep the original event-delivery timing');
    listeners.onChanged[0]({ id: 6971, state: { current: 'complete' } });
    assert.equal(calls.sessionGet.length, 1, 'only overlapping work waits');
    await flushAsyncWork();
    assert.equal(calls.tabsCreate.length, 1);
  });
});
