import test from 'node:test';
import assert from 'node:assert/strict';
import { HostCanvasFontSession, scopedHostCanvasFont, withHostCanvasFonts } from '../src/core/host-canvas-fonts.ts';
import { setHostFontProvider, resolveRendererLocalFont } from '../src/core/local-fonts.ts';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
async function fixture() {
  const oldDocument = globalThis.document;
  const oldFontFace = globalThis.FontFace;
  const fonts = new Set<FontFace>();
  let loaded = () => Promise.resolve();
  class FakeFontFace {
    family: string;
    constructor(family: string, _bytes: ArrayBuffer, _descriptors: FontFaceDescriptors) { this.family = family; }
    async load() { await loaded(); return this; }
  }
  Object.assign(globalThis, { document: { fonts }, FontFace: FakeFontFace });
  const session = new HostCanvasFontSession();
  let changed = () => {};
  const provider = {
    async getSnapshot() { return { revision: 'r1', faces: [{ id: 'face', family: 'Host Test',
      fullName: 'Host Test Italic', postscriptName: 'HostTest-Italic', style: 'Italic', weight: 400, slant: 'italic' as const }] }; },
    async readFace() { return { bytes: Uint8Array.of(1, 2, 3).buffer }; },
    subscribe(listener: () => void) { changed = listener; return () => {}; },
  };
  await setHostFontProvider(provider);
  const records = () => [resolveRendererLocalFont('Host Test', { weight: 400, slant: 'italic' })!];
  return { session, provider, records, fonts, changed: () => changed(), setLoad: (fn: typeof loaded) => { loaded = fn; },
    async dispose() { session.reset(); await setHostFontProvider(null); Object.assign(globalThis, { document: oldDocument, FontFace: oldFontFace }); } };
}

test('host aliases exist only in synchronous measurement/paint scope and are released on detach', async () => {
  const f = await fixture();
  try {
    await f.session.prepare(f.records());
    const original = 'italic 20px "Host Test", sans-serif';
    assert.equal(scopedHostCanvasFont(original), null);
    const aliased = withHostCanvasFonts(f.session, () => scopedHostCanvasFont(original));
    assert.match(aliased!, /__rhwp_host_face_/);
    assert.equal(withHostCanvasFonts(f.session, () => scopedHostCanvasFont(aliased!)), aliased, 'restored context descriptors keep their selected face');
    assert.equal(withHostCanvasFonts(f.session, () => withHostCanvasFonts(null, () => scopedHostCanvasFont(original))), null);
    assert.throws(() => withHostCanvasFonts(f.session, () => { throw new Error('paint failure'); }));
    assert.equal(scopedHostCanvasFont(original), null);
    assert.equal(f.fonts.size, 1);
    await setHostFontProvider(null);
    assert.equal(f.fonts.size, 0);
    assert.deepEqual(f.session.diagnostics(), { loaded: 0, pending: 0, failed: 0 });
  } finally { await f.dispose(); }
});

test('late FontFace.load cannot register after a document reset, even if bytes were already read', async () => {
  const f = await fixture();
  try {
    const waiting = deferred<void>(); let started = false;
    f.setLoad(() => { started = true; return waiting.promise; });
    const preparing = f.session.prepare(f.records());
    while (!started) await Promise.resolve();
    assert.equal(f.fonts.size, 0);
    f.session.reset(); waiting.resolve(); await preparing;
    assert.equal(f.fonts.size, 0);
  } finally { await f.dispose(); }
});

test('failed loads are bounded within a generation and retry after replacement', async () => {
  const f = await fixture();
  try {
    let attempts = 0;
    f.setLoad(() => { attempts++; return Promise.reject(new Error('invalid font')); });
    await f.session.prepare(f.records()); await f.session.prepare(f.records());
    assert.equal(attempts, 1); assert.equal(f.session.diagnostics().failed, 1);
    f.setLoad(async () => {});
    await setHostFontProvider(f.provider);
    await f.session.prepare(f.records());
    assert.equal(f.fonts.size, 1); assert.equal(f.session.diagnostics().failed, 0);
    const previous = [...f.fonts][0];
    f.changed();
    assert(!f.fonts.has(previous), 'notification immediately releases old faces');
  } finally { await f.dispose(); }
});
