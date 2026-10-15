import assert from 'node:assert/strict';
import test from 'node:test';
import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import { LayerResourceStore } from '../src/core/layer-resource-store.ts';
import { readLayerTreeTransport } from '../src/core/layer-tree-transport.ts';
import { normalizeTextV2Tree } from '../src/core/text-v2-normalizer.ts';
import { selectLayerTextVariantsForLeaf } from '../src/view/canvaskit/text-variant-selection.ts';

const bbox = { x: 0, y: 0, width: 20, height: 20 };
const key = (kind: string, bytes: Uint8Array) => `${kind}:blake3:${bytes.length}:${bytesToHex(blake3(bytes))}`;
const v1 = (): any => ({ pageWidth: 20, pageHeight: 20, profile: 'screen', schemaVersion: 1,
  root: { kind: 'leaf', bounds: bbox, ops: [] } });
const envelope = (id = 'text-0'): any => ({ id, type: 'text', bbox, paintOrderSlotId: id,
  selectionPolicy: 'exclusiveVariantSet', defaultVariantId: 'textRun', fallbackPolicy: 'required', variants: [
    { variantId: 'textRun', kind: 'textRun', requiredFeatures: [], parts: [{ partIndex: 0, partCount: 1,
      payload: { type: 'textRun', bbox, text: 'AB' } }] },
    { variantId: 'outline', kind: 'glyphOutline', requiredFeatures: ['text.glyphOutline'], parts: [
      { partIndex: 1, partCount: 2, localPaintOrder: 2, payload: { type: 'glyphOutline', bbox, payloadKind: 'monochromePath' } },
      { partIndex: 0, partCount: 2, localPaintOrder: 1, payload: { type: 'glyphOutline', bbox, payloadKind: 'monochromePath' } },
    ] },
  ] });
const v2 = (op = envelope()): any => ({ ...v1(), schemaVersion: 2, schema: { major: 2, minor: 0 },
  requiredFeatures: ['text.variants', 'text.paintOrderSlot'], textV2: { profile: 'compatibility', fallbackPolicy: 'required' },
  root: { kind: 'leaf', bounds: bbox, ops: [op] } });

test('object resource hints hydrate by content key across page indices and never lend mutable cache bytes', () => {
  const store = new LayerResourceStore();
  const a = Uint8Array.of(1, 2), b = Uint8Array.of(3, 4);
  const ka = key('img', a), kb = key('img', b);
  const first = store.hydrate({ images: [a, b], imageKeys: [ka, kb] })!;
  a[0] = 99;
  (first.images![1] as Uint8Array)[0] = 88;
  const second = store.hydrate({ images: new Array(2), imageKeys: [kb, ka] })!;
  assert.deepEqual(second.images, [Uint8Array.of(3, 4), Uint8Array.of(1, 2)]);
  const svg = '<svg/>'; const ks = key('svg', new TextEncoder().encode(svg));
  store.hydrate({ svgFragments: [svg], svgKeys: [ks] });
  assert.deepEqual(store.hydrate({ svgFragments: [undefined], svgKeys: [ks] })!.svgFragments, [svg]);
  assert.deepEqual(store.knownSvgKeys(), [ks]);
  store.clear(); assert.deepEqual(store.diagnostics(), { entries: 0, bytes: 0 });
  assert.throws(() => store.hydrate({ images: [undefined], imageKeys: [ka] }), /missing img/);
});

test('resource hints fail closed for missing, mismatched, malformed, or oversized tables and stay count bounded', () => {
  const store = new LayerResourceStore();
  assert.throws(() => store.hydrate({ images: [Uint8Array.of(1)], imageKeys: ['img:wrong'] }), /content-addressed/);
  assert.throws(() => store.hydrate({ images: [[300]] }), /invalid image/);
  assert.throws(() => store.hydrate({ images: new Array(4097) }), /bounds/);
  for (let index = 0; index < 257; index++) {
    const bytes = Uint8Array.of(index & 255, index >> 8);
    store.hydrate({ images: [bytes], imageKeys: [key('img', bytes)] });
  }
  assert.equal(store.diagnostics().entries, 256);
  assert.equal(store.knownImageKeys().includes(key('img', Uint8Array.of(0, 0))), false);
});

test('default transport keeps the current JSON omit-font optimization and optional Value uses owned hints', () => {
  const store = new LayerResourceStore();
  const bytes = Uint8Array.of(1, 2); const imageKey = key('img', bytes);
  const calls: unknown[][] = [];
  const doc = {
    getSourceFontBytes() {}, getSourceImageBytes() {},
    getPageLayerTreeWithProfile(...args: unknown[]) { assert.equal(this, doc); calls.push(['json', ...args]); return JSON.stringify(v1()); },
    getPageLayerTreeValueWithProfileAndResourceKeys(...args: unknown[]) {
      assert.equal(this, doc); calls.push(['value', ...args]);
      return { ...v1(), resources: { images: (args[2] as string[]).includes(imageKey) ? [undefined] : [bytes], imageKeys: [imageKey] } };
    },
  };
  readLayerTreeTransport(doc, 3, 'screen', {}, store);
  assert.deepEqual(calls[0], ['json', 3, 'screen', false, true]);
  const result: any = readLayerTreeTransport(doc, 3, 'screen', { transport: 'value' }, store);
  assert.deepEqual(calls[1], ['value', 3, 'screen', [], [], true, true]);
  result.resources.images[0][0] = 99;
  const again: any = readLayerTreeTransport(doc, 4, 'screen', { transport: 'value' }, store);
  assert.deepEqual(calls[2], ['value', 4, 'screen', [imageKey], [], true, true]);
  assert.deepEqual(again.resources.images[0], Uint8Array.of(1, 2));
  assert.throws(() => readLayerTreeTransport({}, 0, 'screen', { transport: 'value' }, store), /does not support/);
  assert.throws(() => readLayerTreeTransport({ getPageLayerTree: () => '{}' }, 0, 'print', {}, store), /does not support/);
});

test('v2 normalization keeps each exclusive set at its paint slot and sorts only its local parts', () => {
  const tree = v2(); const before = { type: 'rectangle', bbox }, after = { type: 'line', bbox };
  tree.root.ops.unshift(before); tree.root.ops.push(after);
  const normalized: any = normalizeTextV2Tree(tree);
  assert.equal(normalized.schemaVersion, 1); assert.deepEqual(normalized.normalizedFromSchema, { major: 2, minor: 0 });
  assert.equal(tree.root.ops[1].type, 'text', 'normalization must not mutate the original wire envelope');
  assert.deepEqual(normalized.root.ops.map((op: any) => op.type), ['rectangle', 'textRun', 'glyphOutline', 'glyphOutline', 'line']);
  assert.deepEqual(normalized.root.ops.slice(2, 4).map((op: any) => op.variant.partIndex), [0, 1]);
  const selected = selectLayerTextVariantsForLeaf(normalized.root.ops, () => true);
  assert.equal(selected.size, 2); assert.equal([...selected].every(op => op.type === 'glyphOutline'), true);
  const fallback = selectLayerTextVariantsForLeaf(normalized.root.ops, () => false);
  assert.equal(fallback.size, 1); assert.equal([...fallback][0].type, 'textRun');
});

test('v2 normalization rejects duplicate/incomplete parts, cycles, unknown required features, and ambiguous slots', () => {
  for (const mutate of [
    (tree: any) => { tree.root.ops[0].variants[1].parts[0].partIndex = 0; },
    (tree: any) => { tree.root.ops[0].variants[1].parts.pop(); },
    (tree: any) => { tree.root.ops[0].variants[1].parts[0].payload.type = 'textRun'; },
    (tree: any) => { tree.root.ops[0].variants[1].parts[0].localPaintOrder = 1; },
    (tree: any) => { tree.root.ops.push(structuredClone(tree.root.ops[0])); },
    (tree: any) => { tree.requiredFeatures.push('text.futureRequired'); },
    (tree: any) => { tree.root.ops[0].fallbackPolicy = 'none'; },
    (tree: any) => { tree.root.ops[0].defaultVariantId = 'absent'; },
    (tree: any) => { tree.root.ops[0].bbox = { ...bbox, x: NaN }; },
  ]) {
    const tree = v2(); mutate(tree); assert.throws(() => normalizeTextV2Tree(tree));
  }
  const cyclic = v2(); cyclic.root = { kind: 'clipRect' }; cyclic.root.child = cyclic.root;
  assert.throws(() => normalizeTextV2Tree(cyclic), /cyclic/);
  const unknownOptional = v2(); unknownOptional.root.ops[0].variants[1].requiredFeatures = ['text.futureOptional'];
  assert.equal((normalizeTextV2Tree(unknownOptional).root as any).ops.length, 1);
});

test('strict v2 remains fallback-free and uses explicitly requested JSON or Value bindings without downgrade', () => {
  const tree = v2();
  tree.requiredFeatures.push('text.strictVisualFallbackFree', 'text.glyphOutline');
  tree.textV2 = { profile: 'strictVisual', strictVisualFallbackFree: true, fallbackPolicy: 'none' };
  tree.root.ops[0].fallbackPolicy = 'none'; tree.root.ops[0].defaultVariantId = 'outline';
  tree.root.ops[0].variants.shift();
  const doc = { getPageLayerTreeV2StrictGlyphOutlineWithProfile: () => JSON.stringify(tree),
    getPageLayerTreeValueV2StrictGlyphOutlineWithProfileAndResourceKeys: () => structuredClone(tree) };
  for (const transport of ['json', 'value'] as const) {
    const normalized: any = readLayerTreeTransport(doc, 0, 'screen', { transport, textProfile: 'strictGlyphOutline' }, new LayerResourceStore());
    assert.equal(normalized.root.ops.every((op: any) => op.type === 'glyphOutline' && !op.variant.isDefaultFallback), true);
    assert.equal(selectLayerTextVariantsForLeaf(normalized.root.ops, () => false).size, 0);
  }
  const malformed = structuredClone(tree); malformed.root.ops[0].variants.push(envelope().variants[0]);
  assert.throws(() => normalizeTextV2Tree(malformed), /cannot contain/);
  assert.throws(() => readLayerTreeTransport({ getPageLayerTreeWithProfile: () => JSON.stringify(v1()) }, 0, 'screen',
    { textProfile: 'strictGlyphOutline' }, new LayerResourceStore()), /does not support/);
});
