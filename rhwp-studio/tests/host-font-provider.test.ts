import test from 'node:test';
import assert from 'node:assert/strict';
import { HostFontSource, type HostFontProvider, type HostFontSnapshot } from '../src/core/host-font-provider.ts';
import { setHostFontProvider, prepareHostFontCatalog, resolveCanvasKitLocalFont, resolveLocalFont,
  loadCanvasKitLocalFont, getLocalFontState } from '../src/core/local-fonts.ts';
import { fontFamilyChainForDisplay } from '../src/core/font-substitution.ts';
import { collectHostFontRequests } from '../src/core/host-font-requests.ts';
import type { PageLayerTree } from '../src/core/types.ts';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
function fixture() {
  let change = () => {};
  let reads = 0;
  let off = 0;
  let snapshot: HostFontSnapshot = { revision: 'r1', faces: [
    { id: 'regular', family: 'Host Only', fullName: 'Host Only Regular', postscriptName: 'HostOnly-Regular',
      style: 'Regular', weight: 400, slant: 'normal', aliases: ['호스트전용'] },
    { id: 'bold', family: 'Host Only', fullName: 'Host Only Bold', postscriptName: 'HostOnly-Bold',
      style: 'Bold', weight: 700, slant: 'normal', aliases: ['호스트전용'] },
  ] };
  const provider: HostFontProvider = {
    getSnapshot: async () => snapshot,
    readFace: async id => { reads++; return { bytes: Uint8Array.of(id === 'bold' ? 7 : 4).buffer }; },
    subscribe: listener => { change = listener; return () => { off++; }; },
  };
  return { provider, change: () => change(), reads: () => reads, off: () => off,
    replace: (value: HostFontSnapshot) => { snapshot = value; change(); }, snapshot: () => snapshot };
}

test('catalog is metadata-only; concurrent face reads share an in-flight request', async () => {
  const f = fixture(); const source = new HostFontSource();
  await source.setProvider(f.provider);
  assert.equal(f.reads(), 0);
  const face = source.references()[0];
  const [a, b] = await Promise.all([source.read(face), source.read(face)]);
  assert.deepEqual(new Uint8Array(a!.bytes), Uint8Array.of(4));
  assert.equal(a, b);
  assert.equal(f.reads(), 1);
  await source.setProvider(null);
  assert.equal(f.off(), 1);
});

test('change immediately invalidates bytes, even if the provider ignores abort', async () => {
  const f = fixture(); const source = new HostFontSource();
  const data = deferred<{ bytes: ArrayBuffer }>();
  let signal: AbortSignal | undefined;
  f.provider.readFace = async (_id, _revision, nextSignal) => { signal = nextSignal; return data.promise; };
  await source.setProvider(f.provider);
  const old = source.references()[0];
  const pending = source.read(old);
  await Promise.resolve();
  f.replace({ ...f.snapshot(), revision: 'r2' });
  assert.equal(signal?.aborted, true);
  data.resolve({ bytes: Uint8Array.of(99).buffer });
  assert.equal(await pending, null);
  await source.ready();
  assert.notEqual(source.references()[0].key, old.key);
  assert.equal(await source.read(old), null);
});

test('a second change during enumeration supersedes the pending snapshot', async () => {
  const f = fixture(); const source = new HostFontSource();
  const old = deferred<HostFontSnapshot>();
  f.provider.getSnapshot = () => old.promise;
  const installing = source.setProvider(f.provider);
  await Promise.resolve();
  f.provider.getSnapshot = async () => ({ ...f.snapshot(), revision: 'latest' });
  f.change();
  await source.ready();
  old.resolve(f.snapshot());
  await installing;
  assert.equal(source.references()[0].revision, 'latest');
});

test('replacement distinguishes providers with identical IDs and revisions and ignores detached listeners', async () => {
  const f = fixture(); const source = new HostFontSource();
  await source.setProvider(f.provider);
  const old = source.references()[0];
  const staleNotify = f.change;
  await source.setProvider(fixture().provider);
  staleNotify();
  assert.equal(source.references().length, 2);
  assert.notEqual(source.references()[0].key, old.key);
  assert.equal(await source.read(old), null);
});

test('malformed catalog fails closed and recovers on the next notification', async () => {
  const f = fixture(); const source = new HostFontSource();
  f.provider.getSnapshot = async () => ({ revision: 'bad', faces: [f.snapshot().faces[0], f.snapshot().faces[0]] });
  await source.setProvider(f.provider);
  assert.equal(source.references().length, 0);
  assert.match(source.lastError!, /Invalid host font face/);
  f.provider.getSnapshot = async () => f.snapshot();
  f.change(); await source.ready();
  assert.equal(source.references().length, 2);
  assert.equal(source.lastError, null);
});

test('host faces are selected by exact identity or family and style, without advertising CSS availability', async () => {
  const f = fixture();
  const before = getLocalFontState();
  const cssBefore = fontFamilyChainForDisplay('Host Only');
  try {
    await setHostFontProvider(f.provider);
    assert.equal(resolveLocalFont('Host Only'), null);
    assert.equal(resolveCanvasKitLocalFont('Host Only'), null);
    const bold = resolveCanvasKitLocalFont('호스트전용', { weight: 700, slant: 'normal' })!;
    assert.equal(bold.hostReference?.face.id, 'bold');
    assert.equal(resolveCanvasKitLocalFont('HostOnly-Regular', { weight: 700, slant: 'normal' })?.hostReference?.face.id, 'regular');
    assert.deepEqual(new Uint8Array((await loadCanvasKitLocalFont(bold))!.bytes), Uint8Array.of(7));
    assert.deepEqual(getLocalFontState(), before);
    assert.equal(fontFamilyChainForDisplay('Host Only'), cssBefore);
    f.replace({ ...f.snapshot(), revision: 'new-bytes' });
    await prepareHostFontCatalog();
    assert.equal(await loadCanvasKitLocalFont(bold), null);
  } finally { await setHostFontProvider(null); }
});

test('paint requests preserve weight and only read the selected faces', async () => {
  const f = fixture();
  const bounds = { x: 0, y: 0, width: 200, height: 100 };
  const tree: PageLayerTree = { pageWidth: 200, pageHeight: 100,
    root: { kind: 'leaf', bounds, ops: [
      { type: 'textRun', text: '가나', bbox: bounds, style: { fontFamily: 'Host Only', bold: true } },
      { type: 'textRun', text: '다라', bbox: bounds, style: { fontFamily: 'Host Only', bold: true } },
    ] } };
  try {
    await setHostFontProvider(f.provider);
    const records = collectHostFontRequests(tree);
    assert.equal(records.length, 1);
    assert.equal(records[0].hostReference?.face.id, 'bold');
    assert.equal(f.reads(), 0);
    await loadCanvasKitLocalFont(records[0]);
    assert.equal(f.reads(), 1);
  } finally { await setHostFontProvider(null); }
});

test('detaching during enumeration discards the late catalog and stops notifications', async () => {
  const f = fixture(); const source = new HostFontSource();
  const catalog = deferred<HostFontSnapshot>();
  f.provider.getSnapshot = () => catalog.promise;
  const installing = source.setProvider(f.provider);
  await Promise.resolve();
  await source.setProvider(null);
  const generation = source.generation;
  catalog.resolve(f.snapshot()); await installing;
  f.change();
  assert.equal(source.generation, generation);
  assert.equal(source.active, false);
  assert.deepEqual(source.references(), []);
});

test('failed byte reads recover on a new revision and buffers are isolated from the provider', async () => {
  const f = fixture(); const source = new HostFontSource();
  f.provider.readFace = async () => { throw new Error('permission revoked'); };
  await source.setProvider(f.provider);
  assert.equal(await source.read(source.references()[0]), null);
  const original = Uint8Array.of(4, 5).buffer;
  f.provider.readFace = async () => ({ bytes: original });
  f.replace({ ...f.snapshot(), revision: 'recovered' }); await source.ready();
  const data = await source.read(source.references()[0]);
  new Uint8Array(data!.bytes)[0] = 9;
  assert.equal(new Uint8Array(original)[0], 4);
});
