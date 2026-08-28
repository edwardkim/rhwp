import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { CanvasKit } from 'canvaskit-wasm';

import type { LayerFontResources, LayerPaintOp, LayerResources } from '../src/core/types.ts';
import { CanvasKitGlyphRunFontCache } from '../src/view/canvaskit/glyph-run-fonts.ts';
import { selectLayerTextVariantsForLeaf } from '../src/view/canvaskit/text-variant-selection.ts';

type FontBytesResolver = (resourceKey: string) => Uint8Array | null;
type ResolverAwareRegister = (
  fontResources: LayerFontResources | undefined,
  resources: LayerResources | undefined,
  resolveFontBytes: FontBytesResolver,
  documentGeneration?: number,
) => void;

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bytes = new Uint8Array(fs.readFileSync(
  path.join(studioRoot, '../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf'),
));

function fontFixture() {
  const digest = bytesToHex(blake3(bytes));
  const blobKey = `font:blake3:${bytes.byteLength}:${digest}`;
  const fontResources: LayerFontResources = {
    blobs: [{
      id: blobKey,
      source: 'embedded',
      portability: 'portableBlob',
      digest: { algorithm: 'blake3', value: digest },
      dataRef: { kind: 'fontBlob', id: blobKey },
    }],
    faces: [],
  };
  return { blobKey, fontResources };
}

test('issue #4969 Q2-D5-R0 records page-linear inline input with one retained blob', () => {
  const { blobKey, fontResources } = fontFixture();
  const inlineResources: LayerResources = {
    fontBlobs: [bytes],
    fontBlobKeys: [blobKey],
  };
  const observations = [];

  for (const pageCount of [1, 2, 8]) {
    const cache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
    let presentedBytes = 0;
    try {
      for (let page = 0; page < pageCount; page += 1) {
        presentedBytes += bytes.byteLength;
        cache.registerResources(fontResources, inlineResources);
      }
      observations.push({
        pages: pageCount,
        presentedBytes,
        retainedBlobs: cache.diagnostics().blobs,
        retainedBytes: cache.diagnostics().bytes,
      });
    } finally {
      cache.clear();
    }
  }

  assert.deepEqual(observations, [
    { pages: 1, presentedBytes: bytes.byteLength, retainedBlobs: 1, retainedBytes: bytes.byteLength },
    { pages: 2, presentedBytes: bytes.byteLength * 2, retainedBlobs: 1, retainedBytes: bytes.byteLength },
    { pages: 8, presentedBytes: bytes.byteLength * 8, retainedBlobs: 1, retainedBytes: bytes.byteLength },
  ]);
});

test('issue #4969 Q2-D5-R0 reuses one verified font fetch across 1/2/8 pages', () => {
  const { blobKey, fontResources } = fontFixture();
  const omittedResources: LayerResources = {
    fontBlobs: [],
    fontBlobKeys: [blobKey],
  };
  const observations = [];

  for (const pageCount of [1, 2, 8]) {
    const cache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
    const registerWithResolver = cache.registerResources as unknown as ResolverAwareRegister;
    let fetches = 0;
    const resolveFontBytes: FontBytesResolver = (resourceKey) => {
      fetches += 1;
      return resourceKey === blobKey ? bytes : null;
    };
    try {
      for (let page = 0; page < pageCount; page += 1) {
        registerWithResolver.call(cache, fontResources, omittedResources, resolveFontBytes);
      }
      observations.push({
        pages: pageCount,
        fetches,
        verifiedBlobs: cache.diagnostics().blobs,
        transferredBytes: fetches * bytes.byteLength,
      });
    } finally {
      cache.clear();
    }
  }

  assert.deepEqual(observations, [
    { pages: 1, fetches: 1, verifiedBlobs: 1, transferredBytes: bytes.byteLength },
    { pages: 2, fetches: 1, verifiedBlobs: 1, transferredBytes: bytes.byteLength },
    { pages: 8, fetches: 1, verifiedBlobs: 1, transferredBytes: bytes.byteLength },
  ]);
});

test('issue #4969 Q2-D5-R2 rejects missing, wrong, oversized, and stale font fetches', () => {
  const { blobKey, fontResources } = fontFixture();
  const omittedResources: LayerResources = { fontBlobs: [], fontBlobKeys: [blobKey] };

  for (const resolveFontBytes of [
    () => null,
    () => new Uint8Array([1, 2, 3, 4]),
  ] satisfies FontBytesResolver[]) {
    const cache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
    try {
      cache.registerResources(fontResources, omittedResources, resolveFontBytes, 7);
      assert.deepEqual(cache.diagnostics(), { blobs: 0, typefaces: 0, fonts: 0, bytes: 0 });
    } finally {
      cache.clear();
    }
  }

  const oversizedKey = `font:blake3:${32 * 1024 * 1024 + 1}:${'0'.repeat(64)}`;
  const oversizedResources: LayerFontResources = {
    blobs: [{
      ...fontResources.blobs[0],
      id: oversizedKey,
      digest: { algorithm: 'blake3', value: '0'.repeat(64) },
      dataRef: { kind: 'fontBlob', id: oversizedKey },
    }],
    faces: [],
  };
  const oversizedCache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
  let oversizedFetches = 0;
  try {
    oversizedCache.registerResources(
      oversizedResources,
      { fontBlobs: [], fontBlobKeys: [oversizedKey] },
      () => {
        oversizedFetches += 1;
        return bytes;
      },
      7,
    );
    assert.equal(oversizedFetches, 0, 'oversized declaration must fail before transport');
    assert.equal(oversizedCache.diagnostics().blobs, 0);
  } finally {
    oversizedCache.clear();
  }

  const secondBytes = new Uint8Array([9, 8, 7, 6]);
  const secondDigest = bytesToHex(blake3(secondBytes));
  const secondKey = `font:blake3:${secondBytes.byteLength}:${secondDigest}`;
  const atomicCache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
  try {
    atomicCache.registerResources(
      {
        blobs: [
          fontResources.blobs[0],
          {
            ...fontResources.blobs[0],
            id: secondKey,
            digest: { algorithm: 'blake3', value: secondDigest },
            dataRef: { kind: 'fontBlob', id: secondKey },
          },
        ],
        faces: [],
      },
      { fontBlobs: [], fontBlobKeys: [blobKey, secondKey] },
      key => (key === blobKey ? bytes : null),
      7,
    );
    assert.equal(atomicCache.diagnostics().blobs, 0, 'failed by-key batch must not partially commit');
  } finally {
    atomicCache.clear();
  }

  const generationCache = new CanvasKitGlyphRunFontCache({} as CanvasKit);
  try {
    generationCache.registerResources(fontResources, omittedResources, () => bytes, 7);
    assert.equal(generationCache.diagnostics().blobs, 1);
    generationCache.registerResources(fontResources, omittedResources, () => null, 8);
    assert.equal(generationCache.diagnostics().blobs, 0, 'new generation clears verified bytes');
    generationCache.registerResources(fontResources, omittedResources, () => bytes, 7);
    assert.equal(generationCache.diagnostics().blobs, 0, 'older generation cannot repopulate cache');
  } finally {
    generationCache.clear();
  }
});

test('issue #4969 Q2-D5-R2 failed font verification leaves the TextRun fallback selected', () => {
  const fallback = {
    type: 'textRun',
    variant: {
      equivalenceGroup: 'q2-d5-r2',
      variantId: 'fallback',
      variantKind: 'textRun',
      partIndex: 0,
      partCount: 1,
      isDefaultFallback: true,
    },
  } as unknown as LayerPaintOp;
  const glyphRun = {
    type: 'glyphRun',
    variant: {
      equivalenceGroup: 'q2-d5-r2',
      variantId: 'portable',
      variantKind: 'glyphRun',
      partIndex: 0,
      partCount: 1,
      isDefaultFallback: false,
    },
  } as unknown as LayerPaintOp;

  const selected = selectLayerTextVariantsForLeaf(
    [fallback, glyphRun],
    () => false,
    () => false,
  );
  assert.deepEqual([...selected], [fallback]);
});
