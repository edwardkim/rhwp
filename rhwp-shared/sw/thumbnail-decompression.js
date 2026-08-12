// Bounded stream reader shared by Chrome, Firefox, and Safari thumbnail consumers.
'use strict';

(() => {
  async function readExactStreamLimited(readable, declaredSize, maxBytes) {
    if (!Number.isSafeInteger(declaredSize)
        || declaredSize <= 0
        || !Number.isSafeInteger(maxBytes)
        || maxBytes <= 0
        || declaredSize > maxBytes) {
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

  globalThis.rhwpBoundedStream = Object.freeze({
    readExactStreamLimited,
  });
})();
