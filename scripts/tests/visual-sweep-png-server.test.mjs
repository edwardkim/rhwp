import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {createSweepServer} from '../export-wasm-png-for-sweep.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');

async function withServer(t, fonts = new Map()) {
  const server = createSweepServer(new Map([
    ['/source', ['application/octet-stream', Buffer.from('document bytes')]],
  ]), fonts);
  await new Promise((accept, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', accept);
  });
  t.after(async () => {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  });
  return `http://127.0.0.1:${server.address().port}`;
}

function fixture(t) {
  const folder = mkdtempSync(join(tmpdir(), 'rhwp-private-font-'));
  t.after(() => rmSync(folder, {recursive: true, force: true}));
  return join(folder, 'private-installation-path.ttf');
}

test('missing font returns a generic error without exposing the server file path', async t => {
  const log = t.mock.method(console, 'error', () => {});
  const path = fixture(t);
  const base = await withServer(t, new Map([['/font/0', {path, sha256: sha(Buffer.from('font'))}]]));
  const response = await fetch(`${base}/font/0`);
  assert.equal(response.status, 500);
  const body = await response.text();
  assert.equal(body, 'Font resource unavailable.');
  assert.ok(!body.includes(path));
  assert.ok(!body.includes('ENOENT'));
  assert.equal(log.mock.calls.length, 1);
  assert.equal(log.mock.calls[0].arguments[1].path, path);
});

test('a changed font fails closed without sending the internal diagnostic', async t => {
  const log = t.mock.method(console, 'error', () => {});
  const path = fixture(t);
  writeFileSync(path, 'changed font bytes');
  const base = await withServer(t, new Map([['/font/0', {path, sha256: sha(Buffer.from('original'))}]]));
  const response = await fetch(`${base}/font/0`);
  assert.equal(response.status, 500);
  assert.equal(await response.text(), 'Font resource unavailable.');
  assert.equal(log.mock.calls.length, 1);
  assert.match(log.mock.calls[0].arguments[1].message, /catalogue/);
});

test('an unchanged allowed font and the source retain their exact bytes', async t => {
  const path = fixture(t), bytes = Buffer.from('validated font bytes');
  writeFileSync(path, bytes);
  const base = await withServer(t, new Map([['/font/0', {path, sha256: sha(bytes)}]]));
  const font = await fetch(`${base}/font/0`);
  assert.equal(font.status, 200);
  assert.equal(font.headers.get('content-type'), 'application/octet-stream');
  assert.deepEqual(Buffer.from(await font.arrayBuffer()), bytes);
  const source = await fetch(`${base}/source`);
  assert.equal(source.status, 200);
  assert.equal(await source.text(), 'document bytes');
});

test('unknown and traversal URLs cannot read files outside the supply list', async t => {
  const base = await withServer(t);
  for (const path of ['/font/unknown', '/font/../../private.ttf', '/font/%2e%2e%2fprivate.ttf']) {
    const response = await fetch(`${base}${path}`);
    assert.equal(response.status, 404);
    assert.equal(await response.text(), '');
  }
});
