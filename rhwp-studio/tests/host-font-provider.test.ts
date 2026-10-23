import test from 'node:test';
import assert from 'node:assert/strict';
import { HostFontSource, type HostFontProvider, type HostFontSnapshot } from '../src/core/host-font-provider.ts';
import { setHostFontProvider, prepareHostFontCatalog, resolveRendererLocalFont, resolveLocalFont,
  loadRendererLocalFont, getLocalFontState } from '../src/core/local-fonts.ts';
import { fontFamilyChainForDisplay } from '../src/core/font-substitution.ts';
import { collectHostFontRequests, collectDocumentCanvasKitFontUsage, resolveDocumentCanvasKitFontUsage, resolveDocumentCanvasKitFontFamilies } from '../src/core/host-font-requests.ts';
import type { LayerEquationLayoutBox, PageLayerTree } from '../src/core/types.ts';

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
    assert.equal(resolveRendererLocalFont('Host Only'), null);
    const bold = resolveRendererLocalFont('호스트전용', { weight: 700, slant: 'normal' })!;
    assert.equal(bold.hostReference?.face.id, 'bold');
    assert.equal(resolveRendererLocalFont('HostOnly-Regular', { weight: 700, slant: 'normal' })?.hostReference?.face.id, 'regular');
    assert.deepEqual(new Uint8Array((await loadRendererLocalFont(bold))!.bytes), Uint8Array.of(7));
    assert.deepEqual(getLocalFontState(), before);
    assert.equal(fontFamilyChainForDisplay('Host Only'), cssBefore);
    f.replace({ ...f.snapshot(), revision: 'new-bytes' });
    await prepareHostFontCatalog();
    assert.equal(await loadRendererLocalFont(bold), null);
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
    await loadRendererLocalFont(records[0]);
    assert.equal(f.reads(), 1);
  } finally { await setHostFontProvider(null); }
});

test('#8109 document substitute candidates reach preparation without changing other source requirements', async () => {
  const f = fixture();
  const bounds = { x: 0, y: 0, width: 200, height: 100 };
  const tree: PageLayerTree = { pageWidth: 200, pageHeight: 100,
    root: { kind: 'leaf', bounds, ops: [
      { type: 'textRun', text: 'fallback', bbox: bounds, style: { fontFamily: 'Missing Source, "Host Only"', bold: true } },
      { type: 'textRun', text: 'original', bbox: bounds, style: { fontFamily: 'Missing Source' } },
      { type: 'textRun', text: 'primary', bbox: bounds, style: { fontFamily: '"Host Only", Missing Source' } },
      { type: 'footnoteMarker', text: '1)', bbox: bounds, fontFamily: 'Missing Marker, "Host Only"' },
    ] } };
  try {
    await setHostFontProvider(f.provider);
    assert.deepEqual(collectHostFontRequests(tree).map(record => record.hostReference?.face.id), ['bold', 'regular']);
    assert.deepEqual(resolveDocumentCanvasKitFontFamilies(['Missing Source', 'Unrelated', 'Host Only'], [tree]),
      ['Host Only', 'Missing Source', 'Unrelated']);
    assert.equal(tree.root.kind === 'leaf' && tree.root.ops[0].type === 'textRun'
      ? tree.root.ops[0].style?.fontFamily : null, 'Missing Source, "Host Only"', 'source declaration stays intact');
    assert.equal(f.reads(), 0, 'metadata selection does not read font programs');
    const usage = collectDocumentCanvasKitFontUsage([tree, tree]);
    assert.equal(usage.length, 4, 'repeated runs/pages share family/style declarations');
    await setHostFontProvider(null);
    assert.deepEqual(resolveDocumentCanvasKitFontUsage(['Missing Source', 'Unrelated', 'Host Only'], usage),
      ['Missing Source', 'Unrelated', 'Host Only'], 'a detached provider cannot survive in a cached face choice');
    await setHostFontProvider(f.provider);
    assert.deepEqual(resolveDocumentCanvasKitFontUsage(['Missing Source', 'Unrelated', 'Host Only'], usage),
      ['Host Only', 'Missing Source', 'Unrelated'], 'the same declarations resolve a newly available provider');

  } finally { await setHostFontProvider(null); }
});

test('Hancom equation glyphs request their actual regular host face without reading bytes', async () => {
  const f = fixture();
  const equationFace = { id: 'equation', family: 'HyhwpEQ', fullName: 'HyhwpEQ',
    postscriptName: 'HyhwpEQ', style: 'Regular', weight: 400, slant: 'normal' as const };
  f.replace({ ...f.snapshot(), faces: [...f.snapshot().faces, equationFace] });
  const bounds = { x: 0, y: 0, width: 200, height: 100 };
  const glyph: LayerEquationLayoutBox = { ...bounds, baseline: 10,
    kind: { type: 'hancomGlyphs', glyphs: '1', positions: [0], baselineShift: 0 } };
  const root: LayerEquationLayoutBox = { ...bounds, baseline: 10,
    kind: { type: 'row', children: [glyph] } };
  const plain: LayerEquationLayoutBox = { ...bounds, baseline: 10,
    kind: { type: 'number', text: '1' } };
  const tree: PageLayerTree = { pageWidth: 200, pageHeight: 100,
    root: { kind: 'leaf', bounds, ops: [
      { type: 'equation', bbox: bounds, layoutBox: root },
      { type: 'equation', bbox: bounds, layoutBox: glyph },
    ] } };
  try {
    await setHostFontProvider(f.provider);
    const requests = collectHostFontRequests(tree);
    assert.equal(requests.length, 1, 'the same equation face is prepared once');
    assert.equal(requests[0].hostReference?.face.id, 'equation');
    assert.equal(f.reads(), 0, 'request collection is metadata only');
    const name: LayerEquationLayoutBox = { ...bounds, baseline: 10,
      kind: { type: 'hancomGlyphs', glyphs: 'lim', positions: [0, 5, 10], baselineShift: 0 } };
    const limit: LayerEquationLayoutBox = { ...bounds, baseline: 10,
      kind: { type: 'limit', isUpper: false, name } };
    tree.root = { kind: 'leaf', bounds, ops: [
      { type: 'equation', bbox: bounds, layoutBox: limit },
    ] };
    assert.equal(collectHostFontRequests(tree)[0]?.hostReference?.face.id, 'equation',
      '첨자가 없는 극한도 이름이 소비하는 실제 face를 요청한다');
    assert.equal(f.reads(), 0, '이름의 요청 수집도 글꼴 바이트를 읽지 않는다');
    tree.root = { kind: 'leaf', bounds, ops: [
      { type: 'equation', bbox: bounds, layoutBox: plain },
      { type: 'equation', bbox: bounds, svgContent: '<svg/>' },
    ] };
    assert.equal(collectHostFontRequests(tree).length, 0, 'other equation paint paths do not consume HyhwpEQ');
  } finally { await setHostFontProvider(null); }
});

test('서로 다른 한컴 수식 face를 같은 페이지에서 각각 준비하고 기본 face도 보존한다', async () => {
  const f = fixture();
  const faces = ['HyhwpEQ', 'HancomEQN'].map((family, index) => ({
    id: `equation-${index}`, family, fullName: family, postscriptName: family,
    style: 'Regular', weight: 400, slant: 'normal' as const,
  }));
  f.replace({ ...f.snapshot(), faces: [...f.snapshot().faces, ...faces] });
  const bounds = { x: 0, y: 0, width: 200, height: 100 };
  const glyph = (fontFamily?: string): LayerEquationLayoutBox => ({
    ...bounds, baseline: 10,
    kind: { type: 'hancomGlyphs', fontFamily, glyphs: '\ue007', positions: [0], baselineShift: 0 },
  });
  const tree: PageLayerTree = { pageWidth: 200, pageHeight: 100,
    root: { kind: 'leaf', bounds, ops: [{ type: 'equation', bbox: bounds,
      layoutBox: { ...bounds, baseline: 10,
        kind: { type: 'fraction', numer: glyph('HancomEQN'), denom: glyph() } },
    }] } };
  try {
    await setHostFontProvider(f.provider);
    const requests = collectHostFontRequests(tree);
    assert.deepEqual(requests.map(r => r.hostReference?.face.id).sort(), ['equation-0', 'equation-1']);
    assert.equal(f.reads(), 0, '실제 글꼴 바이트를 읽기 전에 두 face를 식별한다');
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

test('Regular fullName equal to family does not intercept a family Bold request', async () => {
  const f = fixture();
  const faces = f.snapshot().faces.map((face, index) => index === 0 ? { ...face, fullName: face.family } : face);
  f.provider.getSnapshot = async () => ({ revision: 'same-full-name', faces });
  try {
    await setHostFontProvider(f.provider);
    assert.equal(resolveRendererLocalFont('Host Only', { weight: 700, slant: 'normal' })?.hostReference?.face.id, 'bold');
  } finally { await setHostFontProvider(null); }
});

test('italic runs can select an unambiguous oblique face when no italic face exists', async () => {
  const f = fixture();
  const regular = f.snapshot().faces[0];
  const oblique = { ...regular, id: 'oblique', fullName: 'Host Only Oblique',
    postscriptName: 'HostOnly-Oblique', style: 'Oblique', slant: 'oblique' as const };
  f.provider.getSnapshot = async () => ({ revision: 'oblique', faces: [regular, oblique] });
  try {
    await setHostFontProvider(f.provider);
    assert.equal(resolveRendererLocalFont('Host Only', { weight: 400, slant: 'italic' })?.hostReference?.face.id, 'oblique');
    assert.equal(resolveRendererLocalFont('Host Only', { weight: 400, slant: 'normal' })?.hostReference?.face.id, 'regular');
    const italic = { ...oblique, id: 'italic', fullName: 'Host Only Italic',
      postscriptName: 'HostOnly-Italic', style: 'Italic', slant: 'italic' as const };
    f.provider.getSnapshot = async () => ({ revision: 'both', faces: [regular, oblique, italic] });
    f.change(); await prepareHostFontCatalog();
    assert.equal(resolveRendererLocalFont('Host Only', { weight: 400, slant: 'italic' })?.hostReference?.face.id, 'italic');
    f.provider.getSnapshot = async () => ({ revision: 'ambiguous', faces: [oblique, { ...oblique, id: 'duplicate' }] });
    f.change(); await prepareHostFontCatalog();
    assert.equal(resolveRendererLocalFont('Host Only', { weight: 400, slant: 'italic' }), null);
  } finally { await setHostFontProvider(null); }
});
