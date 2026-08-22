import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';
import { attachPageBudget, rejectProxyConnect } from './extension-smoke.test.mjs';

function createTarget(type, url = '') {
  return {
    type: () => type,
    url: () => url,
  };
}

test('unexpected page rejects an in-flight surface without waiting for it', { timeout: 1_000 }, async () => {
  const browser = new EventEmitter();
  const ownedTarget = createTarget('page', 'about:blank');
  const ownedPage = { target: () => ownedTarget };
  const diagnostics = { unexpectedPageTargets: [] };
  const budget = attachPageBudget(browser, ownedPage, diagnostics);

  try {
    const inFlightSurface = new Promise(() => {});
    const guardedSurface = budget.guard(inFlightSurface);
    browser.emit('targetcreated', createTarget('page', 'chrome-extension://unexpected/viewer.html'));

    await assert.rejects(
      guardedSurface,
      /예기치 않은 page target이 생성되었습니다/,
    );
    assert.deepEqual(
      diagnostics.unexpectedPageTargets,
      ['chrome-extension://unexpected/viewer.html'],
    );
  } finally {
    budget.detach();
  }
});

test('owned page and non-page targets stay within the page budget', async () => {
  const browser = new EventEmitter();
  const ownedTarget = createTarget('page', 'about:blank');
  const ownedPage = { target: () => ownedTarget };
  const diagnostics = { unexpectedPageTargets: [] };
  const budget = attachPageBudget(browser, ownedPage, diagnostics);

  try {
    browser.emit('targetcreated', ownedTarget);
    browser.emit('targetcreated', createTarget('service_worker', 'chrome-extension://worker/background.js'));
    assert.equal(await budget.guard(Promise.resolve('completed')), 'completed');
    assert.deepEqual(diagnostics.unexpectedPageTargets, []);
  } finally {
    budget.detach();
  }
});

function proxyDiagnostics() {
  return {
    blockedProxyRequests: [],
    errors: [],
    proxyClientAborts: [],
  };
}

test('blocked proxy CONNECT installs its socket error listener before responding', () => {
  const diagnostics = proxyDiagnostics();
  const socket = new EventEmitter();
  socket.end = (response) => {
    assert.equal(socket.listenerCount('error'), 1);
    assert.match(response, /^HTTP\/1\.1 502 Bad Gateway/);
  };

  rejectProxyConnect('example.com:443', socket, diagnostics);
  socket.emit('error', Object.assign(new Error('client closed early'), { code: 'ECONNRESET' }));

  assert.deepEqual(diagnostics.blockedProxyRequests, ['CONNECT example.com:443']);
  assert.deepEqual(diagnostics.proxyClientAborts, ['example.com:443: ECONNRESET']);
  assert.deepEqual(diagnostics.errors, []);
});

test('blocked proxy CONNECT preserves unexpected socket errors as diagnostics', () => {
  const diagnostics = proxyDiagnostics();
  const socket = new EventEmitter();
  socket.end = () => {};

  rejectProxyConnect('example.com:443', socket, diagnostics);
  socket.emit('error', Object.assign(new Error('permission denied'), { code: 'EACCES' }));

  assert.deepEqual(diagnostics.proxyClientAborts, []);
  assert.deepEqual(
    diagnostics.errors,
    ['[fixture-proxy] socket error for example.com:443: EACCES'],
  );
});
