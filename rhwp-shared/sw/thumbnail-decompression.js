// Browser extension thumbnail decompression policy shared by Chrome, Firefox, and Safari.
'use strict';

(() => {
  const MAX_THUMBNAIL_BYTES = 10 * 1024 * 1024;

  async function readThumbnailStreamLimited(readable, declaredSize) {
    if (!Number.isSafeInteger(declaredSize)
        || declaredSize <= 0
        || declaredSize > MAX_THUMBNAIL_BYTES) {
      return null;
    }

    const reader = readable.getReader();
    const chunks = [];
    let total = 0;

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = value instanceof Uint8Array ? value : new Uint8Array(value);
        if (chunk.byteLength > declaredSize - total) {
          await reader.cancel();
          return null;
        }
        chunks.push(chunk);
        total += chunk.byteLength;
      }
    } catch {
      return null;
    }

    if (total !== declaredSize) return null;

    const output = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) {
      output.set(chunk, offset);
      offset += chunk.byteLength;
    }
    return output;
  }

  globalThis.rhwpThumbnailDecompression = Object.freeze({
    MAX_THUMBNAIL_BYTES,
    readThumbnailStreamLimited,
  });
})();
