import { test } from 'node:test';
import { strict as assert } from 'node:assert';

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
        calls.search.push(query);
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
      },
      sync: {
        async get(defaults) {
          return { ...defaults, ...(options.settings || {}) };
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
