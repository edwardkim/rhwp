import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { HostCanvasFontSession, scopedHostCanvasFont, withHostCanvasFonts } from '../src/core/host-canvas-fonts.ts';
import { setHostFontProvider, resolveRendererLocalFont, resolveRendererLocalFontCandidates } from '../src/core/local-fonts.ts';
import type { FontSymbolInspector } from '../src/core/font-symbol-coverage.ts';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
async function fixture(inspect?: FontSymbolInspector, bytes = Uint8Array.of(1, 2, 3).buffer) {
  const oldDocument = globalThis.document;
  const oldFontFace = globalThis.FontFace;
  const fonts = new Set<FontFace>();
  const registrations: Array<{ bytes: ArrayBuffer; descriptors: FontFaceDescriptors }> = [];
  let loaded = () => Promise.resolve();
  class FakeFontFace {
    family: string;
    constructor(family: string, bytes: ArrayBuffer, descriptors: FontFaceDescriptors) {
      this.family = family; registrations.push({ bytes, descriptors });
    }
    async load() { await loaded(); return this; }
  }
  Object.assign(globalThis, { document: { fonts }, FontFace: FakeFontFace });
  const session = new HostCanvasFontSession(inspect);
  let changed = () => {};
  const provider = {
    async getSnapshot() { return { revision: 'r1', faces: [{ id: 'face', family: 'Host Test',
      fullName: 'Host Test Italic', postscriptName: 'HostTest-Italic', style: 'Italic', weight: 400, slant: 'italic' as const }] }; },
    async readFace() { return { bytes }; },
    subscribe(listener: () => void) { changed = listener; return () => {}; },
  };
  await setHostFontProvider(provider);
  const records = () => [resolveRendererLocalFont('Host Test', { weight: 400, slant: 'italic' })!];
  return { session, provider, records, fonts, registrations, changed: () => changed(), setLoad: (fn: typeof loaded) => { loaded = fn; },
    async dispose() { session.reset(); await setHostFontProvider(null); Object.assign(globalThis, { document: oldDocument, FontFace: oldFontFace }); } };
}

test('분할 OTF는 준비된 순서대로 Canvas family 사다리에 포함한다', async () => {
  const f = await fixture();
  const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHftOutline.otf', import.meta.url));
  const bytes = font.buffer.slice(font.byteOffset, font.byteOffset + font.byteLength) as ArrayBuffer;
  const faces = ['Latin', 'Hangul'].map(id => ({id, family: 'Split', fullName: `Split ${id}`,
    postscriptName: `Split-${id}`, style: 'Regular', weight: 400, slant: 'normal' as const}));
  try {
    await setHostFontProvider({ ...f.provider, async getSnapshot() { return {revision: 'split', faces}; },
      async readFace() { return {bytes}; } });
    const records = resolveRendererLocalFontCandidates('Split', {weight: 400, slant: 'normal'});
    assert.deepEqual(records.map(record => record.postscriptName), ['Split-Latin', 'Split-Hangul']);
    const outlines: string[] = [], coverage: string[] = [], clears: string[] = [];
    await f.session.prepare(records, {registerHftOutlineFont(family) { outlines.push(family); },
      registerTextFontGlyphCoverage(family) { coverage.push(family); },
      clearTextFontFamily(family) { clears.push(family); }});
    assert.equal(clears.filter(family => family === 'Split').length, 1,
      '분할 family의 이전 세대 coverage를 한 번 비운다');
    assert.equal(outlines.filter(family => family.toLowerCase() === 'split').length, 1,
      '뒤의 분할 face가 첫 face의 윤곽 공급을 덮어쓰지 않는다');
    assert.equal(coverage.filter(family => family === 'Split').length, 0,
      '마지막 조각의 cmap을 family 전체의 coverage로 기록하지 않는다');
    const css = withHostCanvasFonts(f.session, () => scopedHostCanvasFont('20px "Split", serif'));
    assert.match(css!, /__rhwp_host_face_\d+_1.*__rhwp_host_face_\d+_2/);
  } finally { await f.dispose(); }
});

test('같은 family의 OTF와 TTF를 모두 준비하고 OTF를 먼저 표시한다', async () => {
  const f = await fixture();
  const data = (name: string) => {
    const file = readFileSync(new URL(`../../tests/fixtures/fonts/${name}`, import.meta.url));
    return file.buffer.slice(file.byteOffset, file.byteOffset + file.byteLength) as ArrayBuffer;
  };
  const faces = ['TTF', 'CFF'].map(id => ({id, family: 'Mixed', fullName: `Mixed ${id}`,
    postscriptName: `Mixed-${id}`, style: 'Regular', weight: 400, slant: 'normal' as const}));
  try {
    await setHostFontProvider({ ...f.provider, async getSnapshot() { return {revision: 'mixed', faces}; },
      async readFace(id) { return {bytes: data(id === 'TTF' ? 'RHWPHostFixture-Regular.ttf' : 'RHWPHftOutline.otf')}; } });
    const records = resolveRendererLocalFontCandidates('Mixed', {weight: 400, slant: 'normal'});
    await f.session.prepare(records);
    const css = withHostCanvasFonts(f.session, () => scopedHostCanvasFont('20px "Mixed", serif'));
    assert.match(css!, /__rhwp_host_face_\d+_2.*__rhwp_host_face_\d+_1/);
    await setHostFontProvider({ ...f.provider, async getSnapshot() { return {revision: 'only-ttf', faces}; },
      async readFace() { return {bytes: data('RHWPHostFixture-Regular.ttf')}; } });
    await f.session.prepare(resolveRendererLocalFontCandidates('Mixed', {weight: 400, slant: 'normal'}));
    assert.equal(withHostCanvasFonts(f.session, () => scopedHostCanvasFont('20px "Mixed", serif')), null,
      '동일 스타일의 모호한 TTF만 있을 때는 기존 선택 정책을 유지한다');
  } finally { await f.dispose(); }
});

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

test('host FontFace와 주입한 공통 기호 검사기는 같은 실제 프로그램을 소비한다', async () => {
  const original = Uint8Array.from(readFileSync(new URL('../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf', import.meta.url))).buffer;
  const observed: Array<{ bytes: Uint8Array; faceIndex: number }> = [];
  const f = await fixture((bytes, faceIndex) => {
    observed.push({ bytes, faceIndex }); return 'U+0-26AA,U+26AC-10FFFF';
  }, original);
  try {
    await f.session.prepare(f.records());
    assert.equal(f.fonts.size, 1);
    assert.equal(observed.length, 1);
    assert.equal(observed[0].faceIndex, 0, '정규화한 단일 face의 실제 index');
    assert.equal(observed[0].bytes.buffer, f.registrations[0].bytes, 'FontFace와 검사기의 동일한 프로그램 바이트');
    assert.deepEqual(f.registrations[0].descriptors, {
      weight: '400', style: 'italic', unicodeRange: 'U+0-26AA,U+26AC-10FFFF',
    });
  } finally { await f.dispose(); }
});

test('#8109 a prepared document substitute remains available behind an absent primary', async () => {
  const f = await fixture();
  try {
    await f.session.prepare(f.records());
    const direct = withHostCanvasFonts(f.session, () => scopedHostCanvasFont('italic 20px "Host Test"'))!;
    const fallback = withHostCanvasFonts(f.session, () => scopedHostCanvasFont('italic 20px "Missing Source", "Host Test"'))!;
    assert.match(fallback, /__rhwp_host_face_/);
    assert.equal(fallback.split(',')[0], direct.split(',')[0], 'both paths consume the same prepared physical face');
    assert.match(fallback, /"Missing Source", "Host Test"/, 'the original candidate chain is preserved');
  } finally { await f.dispose(); }
});

test('일반 글리프 대체도 준비한 프로그램의 별칭을 선언 순서로 소비한다', async () => {
  const f = await fixture();
  try {
    const snapshot = await f.provider.getSnapshot();
    const second = { ...snapshot.faces[0], id: 'fallback', family: 'Fallback Test',
      fullName: 'Fallback Test Italic', postscriptName: 'FallbackTest-Italic' };
    await setHostFontProvider({ ...f.provider,
      getSnapshot: async () => ({ revision: 'two-faces', faces: [...snapshot.faces, second] }) });
    await f.session.prepare([...f.records(), resolveRendererLocalFont('Fallback Test', { weight: 400, slant: 'italic' })!]);
    const resolve = (family: string) => withHostCanvasFonts(f.session,
      () => scopedHostCanvasFont(`italic 20px ${family}`))!.replace(/^italic 20px /, '');
    const firstAlias = resolve('"Host Test"').split(',')[0];
    const secondAlias = resolve('"Fallback Test"').split(',')[0];
    assert.notEqual(firstAlias, secondAlias);
    assert.ok(resolve('"Host Test", "Fallback Test"').startsWith(`${firstAlias}, ${secondAlias}`));
    assert.ok(resolve('"Fallback Test", "Host Test"').startsWith(`${secondAlias}, ${firstAlias}`));
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

test('host 글꼴의 누락 글리프를 위해 준비한 뒤쪽 후보도 같은 순서로 전달한다', async () => {
  const f = await fixture();
  try {
    const snapshot = await f.provider.getSnapshot();
    await setHostFontProvider({ ...f.provider, async getSnapshot() {
      return { revision: 'fallback-chain', faces: [...snapshot.faces, {
        id: 'fallback', family: 'Host Fallback', fullName: 'Host Fallback Italic',
        postscriptName: 'HostFallback-Italic', style: 'Italic', weight: 400, slant: 'italic' as const,
      }] };
    } });
    const style = { weight: 400, slant: 'italic' as const };
    await f.session.prepare(['Host Test', 'Host Fallback'].map(name => resolveRendererLocalFont(name, style)!));
    const resolve = (families: string) => withHostCanvasFonts(f.session,
      () => scopedHostCanvasFont(`italic 20px ${families}`))!;
    const primary = resolve('"Host Test"').split(',')[0];
    const fallback = resolve('"Host Fallback"').split(',')[0].replace('italic 20px ', '');
    const chain = resolve('"Host Test", "Host Fallback", serif');
    assert(chain.startsWith(primary), '주 글꼴의 선택과 스타일을 유지한다');
    assert(chain.includes(fallback), '첫 글꼴에 없는 글리프를 뒤쪽의 실제 공급 face가 표시할 수 있어야 한다');
    assert(chain.indexOf(fallback) < chain.indexOf('"Host Test"'), '준비한 물리 face는 기존 표시 후보보다 먼저 전달한다');
    assert.match(chain, /"Host Test", "Host Fallback", serif$/, '원래 후보 체인도 보존한다');
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
