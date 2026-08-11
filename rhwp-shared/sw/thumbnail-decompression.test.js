import { strict as assert } from 'node:assert';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';

await import('./thumbnail-decompression.js');

const {
  MAX_THUMBNAIL_BYTES,
  readThumbnailStreamLimited,
} = globalThis.rhwpThumbnailDecompression;

function streamOf(...chunks) {
  return new ReadableStream({
    start(controller) {
      for (const chunk of chunks) controller.enqueue(chunk);
      controller.close();
    },
  });
}

test('thumbnail decompression accepts output matching its declared size', async () => {
  const output = await readThumbnailStreamLimited(
    streamOf(Uint8Array.of(1, 2), Uint8Array.of(3, 4)),
    4,
  );

  assert.deepEqual(output, Uint8Array.of(1, 2, 3, 4));
});

test('thumbnail decompression rejects a declared size above the shared limit', async () => {
  let readerRequests = 0;
  const readable = {
    getReader() {
      readerRequests += 1;
      throw new Error('must not start reading');
    },
  };

  assert.equal(
    await readThumbnailStreamLimited(readable, MAX_THUMBNAIL_BYTES + 1),
    null,
  );
  assert.equal(
    readerRequests,
    0,
    'oversized metadata must be rejected before decompression starts',
  );
});

test('thumbnail decompression rejects streamed output beyond the shared limit', async () => {
  const first = new Uint8Array(MAX_THUMBNAIL_BYTES);
  const output = await readThumbnailStreamLimited(
    streamOf(first, Uint8Array.of(1)),
    MAX_THUMBNAIL_BYTES,
  );

  assert.equal(output, null);
});

test('thumbnail decompression rejects output that disagrees with ZIP metadata', async () => {
  const output = await readThumbnailStreamLimited(streamOf(Uint8Array.of(1, 2, 3)), 2);
  assert.equal(output, null);
});

test('all thumbnail consumers use the shared output policy', async () => {
  const sourceUrls = [
    new URL('../../rhwp-chrome/sw/thumbnail-extractor.js', import.meta.url),
    new URL('../../rhwp-firefox/sw/thumbnail-extractor.js', import.meta.url),
    new URL('../../rhwp-safari/src/background.js', import.meta.url),
  ];
  for (const sourceUrl of sourceUrls) {
    const source = await readFile(sourceUrl, 'utf8');
    assert.match(source, /MAX_THUMBNAIL_BYTES/);
    assert.match(source, /readThumbnailStreamLimited/);
    assert.match(source, /compSize !== uncompSize|compSz !== uncSz/);
  }

  const rustSource = await readFile(new URL('../../src/parser/mod.rs', import.meta.url), 'utf8');
  assert.match(
    rustSource,
    /pub const MAX_THUMBNAIL_BYTES: usize = 10 \* 1024 \* 1024;/,
  );
  assert.match(
    rustSource,
    /read_preview_image_limited\(MAX_THUMBNAIL_BYTES\)/,
  );

  const safariManifest = JSON.parse(
    await readFile(new URL('../../rhwp-safari/src/manifest.json', import.meta.url), 'utf8'),
  );
  assert.deepEqual(
    safariManifest.background.scripts.slice(-2),
    ['thumbnail-decompression.js', 'background.js'],
  );

  const safariBuild = await readFile(
    new URL('../../rhwp-safari/build.sh', import.meta.url),
    'utf8',
  );
  assert.match(safariBuild, /thumbnail-decompression\.js/);
});
