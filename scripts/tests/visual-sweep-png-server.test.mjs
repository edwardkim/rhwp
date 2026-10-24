import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {createSweepServer, registerCanvasFont, preparePrintReadbackCanvas} from '../export-wasm-png-for-sweep.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');

test('Canvas 공급은 family 이름과 원본 바이트를 보존한다', async t => {
  const supplied = [];
  class Face {
    constructor(family, bytes, descriptors) { Object.assign(this, {family, bytes, descriptors}); }
    async load() { return this; }
  }
  for (const [key, value] of Object.entries({FontFace: Face, document: {fonts: {add: face => supplied.push(face)}}})) {
    const original = Object.getOwnPropertyDescriptor(globalThis, key);
    Object.defineProperty(globalThis, key, {configurable: true, writable: true, value});
    t.after(() => {
      if (original) Object.defineProperty(globalThis, key, original);
      else delete globalThis[key];
    });
  }
  const bytes = new Uint8Array([1, 3, 5, 7]).buffer;
  for (const family of ['RHWP Supply Probe', '공급글꼴검사']) {
    const face = await registerCanvasFont(family, bytes, 400, 'normal');
    assert.equal(face.family, family, 'Canvas 요청 이름에 따옴표나 이스케이프를 추가하지 않는다');
    assert.equal(face.bytes, bytes);
    assert.deepEqual(face.descriptors, {weight: '400', style: 'normal'});
    assert.equal(supplied.at(-1), face);
  }
});

// HTMLCanvasElement은 첫 getContext의 options만 적용하고 같은 context를 돌려준다.
function canvasFixture() {
  let context;
  return {
    getContext(kind, options = {}) {
      assert.equal(kind, '2d');
      context ??= {getContextAttributes: () => ({
        alpha: true, willReadFrequently: options.willReadFrequently === true,
      })};
      return context;
    },
  };
}

test('readback 조건이 이후 WASM의 기본 getContext 호출에도 유지된다', () => {
  const canvas = canvasFixture();
  const attributes = preparePrintReadbackCanvas(canvas);
  assert.deepEqual(canvas.getContext('2d').getContextAttributes(), attributes);
  assert.equal(attributes.willReadFrequently, true);
});

test('이미 만든 기본 context를 반복 readback context로 오인하지 않는다', () => {
  const canvas = canvasFixture();
  canvas.getContext('2d');
  assert.throws(() => preparePrintReadbackCanvas(canvas), /최초 2D context/);
});

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
