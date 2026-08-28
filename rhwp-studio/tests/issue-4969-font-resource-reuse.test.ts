import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { CanvasKit } from 'canvaskit-wasm';

import type { LayerFontResources, LayerResources } from '../src/core/types.ts';
import { CanvasKitGlyphRunFontCache } from '../src/view/canvaskit/glyph-run-fonts.ts';

type FontBytesResolver = (resourceKey: string) => Uint8Array | null;
type ResolverAwareRegister = (
  fontResources: LayerFontResources | undefined,
  resources: LayerResources | undefined,
  resolveFontBytes: FontBytesResolver,
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
