import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const blocked = [
  '::', '::1', '::ffff:127.0.0.1', '::ffff:7f00:1',
  '0:0:0:0:0:ffff:a00:1', '::ffff:10.0.0.1',
  '::ffff:172.16.0.1', '::ffff:192.168.1.1', '::ffff:169.254.169.254',
  '::ffff:100.64.0.1', '::ffff:224.0.0.1', 'fe90::1', 'febf::1', 'fc00::1', 'fd00::1', 'ff02::1',
];
const allowed = ['2001:4860:4860::8888', '::ffff:8.8.8.8', '::ffff:172.32.0.1'];
const url = host => `http://[${host}]/document.hwp`;

for (const extension of ['rhwp-chrome', 'rhwp-firefox']) {
  const policy = await import(new URL(`../../${extension}/sw/fetch-security.js`, import.meta.url));
  for (const host of blocked) {
    assert.throws(() => policy.validateDocumentFetchUrl(url(host)),
      error => error.reason === 'private-host-blocked', `${extension}: ${host}`);
  }
  for (const host of allowed) assert.equal(policy.validateDocumentFetchUrl(url(host)).href, new URL(url(host)).href);
  const originalFetch = globalThis.fetch;
  let calls = 0;
  try {
    globalThis.fetch = async () => {
      calls++;
      return new Response(null, {status: 302, headers: {Location: url('::ffff:127.0.0.1')}});
    };
    await assert.rejects(policy.fetchDocumentWithPolicy(url('::ffff:10.0.0.1')),
      error => error.reason === 'private-host-blocked');
    assert.equal(calls, 0, 'initial private URL cannot reach fetch');
    await assert.rejects(policy.fetchDocumentWithPolicy('https://example.com/document.hwp'),
      error => error.reason === 'private-host-blocked');
    assert.equal(calls, 1, 'redirect private URL cannot reach a second fetch');
  } finally { globalThis.fetch = originalFetch; }
}

// Exercise the actual Safari background fetch-file handler, without network.
let messageHandler;
let safariFetches = 0;
const listener = {addListener() {}};
const safari = vm.createContext({URL, Map, console: {log() {}},
  rhwpBoundedStream: {readExactStreamLimited() {throw Error('unexpected stream');}},
  fetch() {safariFetches++; throw Error('private fetch must not execute');},
  browser: {
    runtime: {getURL: () => 'safari-web-extension://private-test/',
      onMessage: {addListener(handler) {messageHandler = handler;}}, onInstalled: listener},
    contextMenus: {onClicked: listener}, action: {onClicked: listener},
    storage: {local: {async get(defaults) {return defaults;}}},
  },
});
vm.runInContext(readFileSync(new URL('../../rhwp-safari/src/background.js', import.meta.url), 'utf8'), safari);
for (const host of blocked) {
  assert.equal(vm.runInContext(`isPrivateHost(${JSON.stringify(new URL(url(host)).hostname)})`, safari), true, `Safari: ${host}`);
  const result = await new Promise(resolve => messageHandler({type: 'fetch-file', url: url(host)},
    {url: 'safari-web-extension://private-test/viewer.html'}, resolve));
  assert.equal(result.error, '내부 네트워크 접근 차단');
}
for (const host of allowed) {
  assert.equal(vm.runInContext(`isPrivateHost(${JSON.stringify(new URL(url(host)).hostname)})`, safari), false);
}
assert.equal(safariFetches, 0);

const shared = vm.createContext({URL});
vm.runInContext(readFileSync(new URL('./url-validator.js', import.meta.url), 'utf8'), shared);
for (const host of blocked) assert.equal(vm.runInContext(`isPrivateHost(${JSON.stringify(new URL(url(host)).hostname)})`, shared), true, `shared: ${host}`);
for (const host of allowed) assert.equal(vm.runInContext(`isPrivateHost(${JSON.stringify(new URL(url(host)).hostname)})`, shared), false);
console.log('IPv6 literal, redirect and Safari message-handler boundary checks passed');
