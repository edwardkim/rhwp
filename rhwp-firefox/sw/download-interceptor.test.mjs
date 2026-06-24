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
