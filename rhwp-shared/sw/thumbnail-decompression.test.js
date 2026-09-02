import { strict as assert } from 'node:assert';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';

await import('./thumbnail-decompression.js');

const { readExactStreamLimited } = globalThis.rhwpBoundedStream;
const THUMBNAIL_OUTPUT_LIMIT_BYTES = 10 * 1024 * 1024;

function streamOf(...chunks) {
  return new ReadableStream({
    start(controller) {
      for (const chunk of chunks) controller.enqueue(chunk);
      controller.close();
    },
  });
}

test('bounded stream reader accepts output matching its explicit allowance', async () => {
  const output = await readExactStreamLimited(
    streamOf(Uint8Array.of(1, 2), Uint8Array.of(3, 4)),
    4,
    4,
  );

  assert.deepEqual(output, Uint8Array.of(1, 2, 3, 4));
});

test('bounded stream reader rejects a declared size above its caller allowance', async () => {
  let readerRequests = 0;
  const readable = {
    getReader() {
      readerRequests += 1;
      throw new Error('must not start reading');
    },
  };

  assert.equal(
    await readExactStreamLimited(readable, 5, 4),
    null,
  );
  assert.equal(
    readerRequests,
    0,
    'oversized metadata must be rejected before decompression starts',
  );
});

test('bounded stream reader rejects streamed output beyond declared size', async () => {
  const output = await readExactStreamLimited(
    streamOf(Uint8Array.of(1, 2), Uint8Array.of(3)),
    2,
    4,
  );

  assert.equal(output, null);
});

test('bounded stream reader rejects output that disagrees with ZIP metadata', async () => {
  const output = await readExactStreamLimited(streamOf(Uint8Array.of(1, 2, 3)), 2, 4);
  assert.equal(output, null);
});

function pushU16(bytes, value) {
  bytes.push(value & 0xff, (value >>> 8) & 0xff);
}

function pushU32(bytes, value) {
  bytes.push(
    value & 0xff,
    (value >>> 8) & 0xff,
    (value >>> 16) & 0xff,
    (value >>> 24) & 0xff,
  );
}

function storedPreviewZip(payload, declaredSize = payload.byteLength) {
  const name = new TextEncoder().encode('Preview/PrvImage.png');
  const local = [];
  pushU32(local, 0x04034b50);
  pushU16(local, 20);
  pushU16(local, 0);
  pushU16(local, 0);
  pushU16(local, 0);
  pushU16(local, 0);
  pushU32(local, 0);
  pushU32(local, payload.byteLength);
  pushU32(local, declaredSize);
  pushU16(local, name.byteLength);
  pushU16(local, 0);
  local.push(...name, ...payload);

  const centralOffset = local.length;
  const central = [];
  pushU32(central, 0x02014b50);
  pushU16(central, 20);
  pushU16(central, 20);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU32(central, 0);
  pushU32(central, payload.byteLength);
  pushU32(central, declaredSize);
  pushU16(central, name.byteLength);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU16(central, 0);
  pushU32(central, 0);
  pushU32(central, 0);
  central.push(...name);

  const end = [];
  pushU32(end, 0x06054b50);
  pushU16(end, 0);
  pushU16(end, 0);
  pushU16(end, 1);
  pushU16(end, 1);
  pushU32(end, central.length);
  pushU32(end, centralOffset);
  pushU16(end, 0);
  return Uint8Array.from([...local, ...central, ...end]);
}

function pngPreview() {
  const png = new Uint8Array(24);
  png.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  png.set([0, 0, 0, 1], 16);
  png.set([0, 0, 0, 1], 20);
  return png;
}

test('Chrome public thumbnail consumer preserves a valid preview and rejects oversized metadata', { concurrency: false }, async () => {
  const { extractThumbnailFromUrl } = await import('../../rhwp-chrome/sw/thumbnail-extractor.js');
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (url) => new Response(
    String(url).endsWith('oversized.hwpx')
      ? storedPreviewZip(new Uint8Array(), THUMBNAIL_OUTPUT_LIMIT_BYTES + 1)
      : storedPreviewZip(pngPreview()),
  );

  try {
    const valid = await extractThumbnailFromUrl('https://example.test/valid.hwpx');
    assert.equal(valid?.mime, 'image/png');
    assert.equal(valid?.width, 1);
    assert.equal(valid?.height, 1);
    assert.match(valid?.dataUri || '', /^data:image\/png;base64,/);

    assert.equal(
      await extractThumbnailFromUrl('https://example.test/oversized.hwpx'),
      null,
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('thumbnail consumers choose their output policy at the public boundary', async () => {
  const sourceUrls = [
    new URL('../../rhwp-chrome/sw/thumbnail-extractor.js', import.meta.url),
    new URL('../../rhwp-firefox/sw/thumbnail-extractor.js', import.meta.url),
    new URL('../../rhwp-safari/src/background.js', import.meta.url),
  ];
  for (const sourceUrl of sourceUrls) {
    const source = await readFile(sourceUrl, 'utf8');
    assert.match(source, /THUMBNAIL_OUTPUT_LIMIT_BYTES = 10 \* 1024 \* 1024/);
    assert.match(source, /readExactStreamLimited/);
    assert.match(source, /compSize !== uncompSize|compSz !== uncSz/);
  }

  const helperSource = await readFile(new URL('./thumbnail-decompression.js', import.meta.url), 'utf8');
  assert.doesNotMatch(helperSource, /THUMBNAIL_OUTPUT_LIMIT_BYTES|MAX_THUMBNAIL_BYTES/);
  assert.match(helperSource, /readExactStreamLimited\(readable, declaredSize, maxBytes\)/);

  const rustSource = await readFile(new URL('../../src/parser/mod.rs', import.meta.url), 'utf8');
  assert.match(
    rustSource,
    /pub const MAX_THUMBNAIL_BYTES: usize = 10 \* 1024 \* 1024;/,
  );
  assert.match(
    rustSource,
    /extract_preview\(&mut cfb, MAX_THUMBNAIL_BYTES\)/,
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
