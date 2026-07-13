import test from 'node:test';
import assert from 'node:assert/strict';

import {
  EMBED_CAPABILITIES,
  isConnectMessage,
  isRequestEnvelope,
  isUsableParentOrigin,
} from '../src/embed/protocol.ts';
import { routeEmbedRequest, type EmbedRpcHandlers } from '../src/embed/rpc-router.ts';
import { installEmbedRuntime } from '../src/embed/runtime.ts';

test('embed protocol은 capability를 포함한 v1 connect와 session-bound request만 허용한다', () => {
  assert.equal(isConnectMessage({
    type: 'rhwp-connect', version: 1, sessionId: 's-1', capabilities: EMBED_CAPABILITIES,
  }), true);
  assert.equal(isConnectMessage({ type: 'rhwp-connect', version: 1, sessionId: 's-1' }), false);
  assert.equal(isConnectMessage({ type: 'rhwp-connect', version: 2, sessionId: 's-1' }), false);
  assert.equal(isConnectMessage({ type: 'rhwp-connect', version: 1, sessionId: '' }), false);

  assert.equal(isRequestEnvelope({
    type: 'rhwp-request', version: 1, sessionId: 's-1', id: 1, method: 'ready', params: {},
  }, 's-1'), true);
  assert.equal(isRequestEnvelope({
    type: 'rhwp-request', version: 1, sessionId: 'other', id: 1, method: 'ready', params: {},
  }, 's-1'), false);
  assert.equal(isUsableParentOrigin('https://host.example'), true);
  assert.equal(isUsableParentOrigin('null'), false);
});

test('embed router는 binary load와 unknown method를 공개 동작으로 처리한다', async () => {
  let loaded: Uint8Array | undefined;
  const handlers: EmbedRpcHandlers = {
    ready: async () => true,
    loadFile: async (data) => {
      loaded = data;
      return { pageCount: 2 };
    },
    pageCount: async () => 2,
    getRendererDiagnostics: async (page) => ({ page }),
    getPageSvg: async () => '<svg/>',
    exportHwp: async () => new Uint8Array([1]),
    exportHwpx: async () => new Uint8Array([2]),
    exportHwpVerify: async () => ({ recovered: true }),
  };

  assert.deepEqual(
    await routeEmbedRequest('loadFile', { data: new Uint8Array([3, 4]), fileName: 'a.hwp' }, handlers),
    { pageCount: 2 },
  );
  assert.deepEqual([...(loaded ?? [])], [3, 4]);
  assert.deepEqual(
    await routeEmbedRequest('getRendererDiagnostics', { page: 3 }, handlers),
    { page: 3 },
  );
  await assert.rejects(
    () => routeEmbedRequest('getRendererDiagnostics', { page: -1 }, handlers),
    /page must be a non-negative integer/,
  );
  await assert.rejects(() => routeEmbedRequest('missing', {}, handlers), /Unknown method: missing/);
  await assert.rejects(
    () => routeEmbedRequest('loadFile', { data: [3, 4], fileName: 'legacy.hwp' }, handlers),
    /binary data/,
  );
});

test('embed runtime은 parent의 exact origin에서 v1 port session을 설치한다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const handlers: EmbedRpcHandlers = {
    ready: async () => true,
    loadFile: async () => ({ pageCount: 1 }),
    pageCount: async () => 7,
    getRendererDiagnostics: async (page) => ({ page }),
    getPageSvg: async () => '<svg/>',
    exportHwp: async () => new Uint8Array([1]),
    exportHwpx: async () => new Uint8Array([2]),
    exportHwpVerify: async () => ({ recovered: true }),
  };
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers,
  });
  const channel = new MessageChannel();
  const messages: unknown[] = [];
  const received = new Promise<void>((resolve) => {
    channel.port1.onmessage = ({ data }) => {
      messages.push(data);
      if (data.type === 'rhwp-connected') {
        channel.port1.postMessage({
          type: 'rhwp-request', version: 1, sessionId: 'session-a',
          id: 4, method: 'pageCount', params: {},
        });
      } else {
        resolve();
      }
    };
  });
  channel.port1.start();

  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'session-a',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow,
    origin: 'https://host.example',
    ports: [channel.port2],
  } as unknown as MessageEvent);
  await received;

  assert.deepEqual(messages, [
    {
      type: 'rhwp-connected', version: 1, sessionId: 'session-a',
      capabilities: ['transferable-array-buffer'],
    },
    { type: 'rhwp-response', version: 1, sessionId: 'session-a', id: 4, result: 7 },
  ]);
  cleanup();
  channel.port1.close();
});

test('embed runtime은 bound session의 malformed request에만 구조화된 오류를 반환한다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers: {} as EmbedRpcHandlers,
  });
  const channel = new MessageChannel();
  const messages: unknown[] = [];
  let resolveConnected: () => void = () => {};
  let resolveInvalid: (message: unknown) => void = () => {};
  const connected = new Promise<void>((resolve) => { resolveConnected = resolve; });
  const invalidResponse = new Promise<unknown>((resolve) => { resolveInvalid = resolve; });
  channel.port1.onmessage = ({ data }) => {
    messages.push(data);
    if (data.type === 'rhwp-connected') resolveConnected();
    if (data.type === 'rhwp-response') resolveInvalid(data);
  };
  channel.port1.start();

  try {
    messageListener({
      data: {
        type: 'rhwp-connect', version: 1, sessionId: 'session-a',
        capabilities: ['transferable-array-buffer'],
      },
      source: parentWindow, origin: 'https://host.example', ports: [channel.port2],
    } as unknown as MessageEvent);
    await connected;
    channel.port1.postMessage({
      type: 'rhwp-request', version: 1, sessionId: 'session-a', id: 7, method: '',
    });
    channel.port1.postMessage({
      type: 'rhwp-request', version: 1, sessionId: 'other', id: 8, method: '',
    });
    channel.port1.postMessage({
      type: 'rhwp-request', version: 1, sessionId: 'session-a',
      id: Number.MAX_SAFE_INTEGER + 1, method: '',
    });

    assert.deepEqual(await Promise.race([
      invalidResponse,
      new Promise((_, reject) => setTimeout(() => reject(new Error('INVALID_REQUEST timeout')), 50)),
    ]), {
      type: 'rhwp-response', version: 1, sessionId: 'session-a', id: 7,
      error: { code: 'INVALID_REQUEST', message: 'Invalid embed request.' },
    });
    await new Promise((resolve) => setTimeout(resolve, 20));
    assert.equal(messages.length, 2);
  } finally {
    cleanup();
    channel.port1.close();
  }
});

test('embed runtime은 bound session의 unsupported request version을 명시적으로 거부한다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers: {} as EmbedRpcHandlers,
  });
  const channel = new MessageChannel();
  let resolveConnected: () => void = () => {};
  let resolveMismatch: (message: unknown) => void = () => {};
  const connected = new Promise<void>((resolve) => { resolveConnected = resolve; });
  const mismatchResponse = new Promise<unknown>((resolve) => { resolveMismatch = resolve; });
  channel.port1.onmessage = ({ data }) => {
    if (data.type === 'rhwp-connected') resolveConnected();
    if (data.type === 'rhwp-response') resolveMismatch(data);
  };
  channel.port1.start();

  try {
    messageListener({
      data: {
        type: 'rhwp-connect', version: 1, sessionId: 'session-a',
        capabilities: ['transferable-array-buffer'],
      },
      source: parentWindow, origin: 'https://host.example', ports: [channel.port2],
    } as unknown as MessageEvent);
    await connected;
    channel.port1.postMessage({
      type: 'rhwp-request', version: 2, sessionId: 'session-a', id: 9, method: 'pageCount',
    });

    assert.deepEqual(await Promise.race([
      mismatchResponse,
      new Promise((_, reject) => setTimeout(() => reject(new Error('UNSUPPORTED_VERSION timeout')), 50)),
    ]), {
      type: 'rhwp-response', version: 1, sessionId: 'session-a', id: 9,
      error: {
        code: 'UNSUPPORTED_VERSION',
        message: 'Unsupported embed protocol version: 2',
        supportedVersions: [1],
      },
    });
  } finally {
    cleanup();
    channel.port1.close();
  }
});

test('embed runtime은 bound session의 missing/non-numeric version을 malformed로 거부한다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers: {} as EmbedRpcHandlers,
  });
  const channel = new MessageChannel();
  let resolveConnected: () => void = () => {};
  const malformedMessages: unknown[] = [];
  let resolveMalformed: (messages: unknown[]) => void = () => {};
  const connected = new Promise<void>((resolve) => { resolveConnected = resolve; });
  const malformedResponses = new Promise<unknown[]>((resolve) => { resolveMalformed = resolve; });
  channel.port1.onmessage = ({ data }) => {
    if (data.type === 'rhwp-connected') resolveConnected();
    if (data.type === 'rhwp-response') {
      malformedMessages.push(data);
      if (malformedMessages.length === 2) resolveMalformed(malformedMessages);
    }
  };
  channel.port1.start();

  try {
    messageListener({
      data: {
        type: 'rhwp-connect', version: 1, sessionId: 'session-a',
        capabilities: ['transferable-array-buffer'],
      },
      source: parentWindow, origin: 'https://host.example', ports: [channel.port2],
    } as unknown as MessageEvent);
    await connected;
    channel.port1.postMessage({
      type: 'rhwp-request', sessionId: 'session-a', id: 10, method: 'pageCount',
    });
    channel.port1.postMessage({
      type: 'rhwp-request', version: '2', sessionId: 'session-a', id: 11, method: 'pageCount',
    });

    assert.deepEqual(await Promise.race([
      malformedResponses,
      new Promise((_, reject) => setTimeout(() => reject(new Error('INVALID_REQUEST timeout')), 50)),
    ]), [
      {
        type: 'rhwp-response', version: 1, sessionId: 'session-a', id: 10,
        error: { code: 'INVALID_REQUEST', message: 'Invalid embed request.' },
      },
      {
        type: 'rhwp-response', version: 1, sessionId: 'session-a', id: 11,
        error: { code: 'INVALID_REQUEST', message: 'Invalid embed request.' },
      },
    ]);
  } finally {
    cleanup();
    channel.port1.close();
  }
});

test('embed runtime은 첫 v1 origin/session/port만 사용하고 이후 legacy dispatch를 막는다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  let pageCountCalls = 0;
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const handlers = {
    pageCount: async () => { pageCountCalls += 1; return 1; },
  } as EmbedRpcHandlers;
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers,
  });
  const first = new MessageChannel();
  first.port1.start();
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'first',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow, origin: 'https://host.example', ports: [first.port2],
  } as unknown as MessageEvent);
  const second = new MessageChannel();
  second.port1.start();
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'second',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow, origin: 'https://other.example', ports: [second.port2],
  } as unknown as MessageEvent);
  messageListener({
    data: { type: 'rhwp-request', id: 7, method: 'pageCount', params: {} },
    source: parentWindow, origin: 'https://host.example', ports: [],
  } as unknown as MessageEvent);
  await new Promise((resolve) => setTimeout(resolve, 10));

  assert.equal(pageCountCalls, 0);
  cleanup();
  first.port1.close();
  second.port1.close();
});

test('embed runtime은 지원하지 않는 version에 구조화된 협상 오류를 반환한다', async () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const handlers = {} as EmbedRpcHandlers;
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers,
  });
  const channel = new MessageChannel();
  const response = new Promise<unknown>((resolve) => {
    channel.port1.onmessage = ({ data }) => resolve(data);
    channel.port1.start();
  });

  messageListener({
    data: {
      type: 'rhwp-connect', version: 2, sessionId: 'session-v2',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow,
    origin: 'https://host.example',
    ports: [channel.port2],
  } as unknown as MessageEvent);

  assert.deepEqual(await response, {
    type: 'rhwp-connect-error',
    version: 1,
    sessionId: 'session-v2',
    error: {
      code: 'UNSUPPORTED_VERSION',
      message: '지원하지 않는 embed protocol version: 2',
      supportedVersions: [1],
    },
  });
  cleanup();
  channel.port1.close();
});

test('embed runtime은 거부되거나 정리된 모든 transferred port의 소유권을 해제한다', () => {
  let messageListener: (event: MessageEvent) => void = () => {};
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const port = () => ({
    onmessage: null,
    closed: false,
    start() {},
    postMessage() {},
    close() { this.closed = true; },
  });
  const rejected = port();
  const malformed = port();
  const bound = port();
  const foreign = port();
  const foreignSource = port();
  const nonConnect = port();
  const surplus = port();
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers: {} as EmbedRpcHandlers,
  });

  messageListener({
    data: { type: 'rhwp-connect', version: 2, sessionId: 'bad', capabilities: [] },
    source: parentWindow, origin: 'https://host.example', ports: [rejected],
  } as unknown as MessageEvent);
  messageListener({
    data: { type: 'rhwp-connect', version: 1, sessionId: '', capabilities: [] },
    source: parentWindow, origin: 'https://host.example', ports: [malformed],
  } as unknown as MessageEvent);
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'bound',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow, origin: 'https://host.example', ports: [bound],
  } as unknown as MessageEvent);
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'foreign',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow, origin: 'https://other.example', ports: [foreign],
  } as unknown as MessageEvent);
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'forged',
      capabilities: ['transferable-array-buffer'],
    },
    source: {}, origin: 'https://host.example', ports: [foreignSource],
  } as unknown as MessageEvent);
  messageListener({
    data: { type: 'rhwp-request', id: 9, method: 'ready' },
    source: parentWindow, origin: 'https://host.example', ports: [nonConnect],
  } as unknown as MessageEvent);
  messageListener({
    data: {
      type: 'rhwp-connect', version: 1, sessionId: 'surplus',
      capabilities: ['transferable-array-buffer'],
    },
    source: parentWindow, origin: 'https://host.example', ports: [port(), surplus],
  } as unknown as MessageEvent);

  assert.equal(rejected.closed, true);
  assert.equal(malformed.closed, true);
  assert.equal(foreign.closed, true);
  assert.equal(foreignSource.closed, true);
  assert.equal(nonConnect.closed, true);
  assert.equal(surplus.closed, true);
  assert.notEqual(bound.onmessage, null);
  cleanup();
  assert.equal(bound.closed, true);
  assert.equal(bound.onmessage, null);
});
