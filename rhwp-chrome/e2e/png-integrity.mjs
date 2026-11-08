import { inflateSync } from 'node:zlib';

const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
const maxDecodedBytes = 128 * 1024 * 1024;
const crcTable = Uint32Array.from({ length: 256 }, (_, n) => {
  for (let bit = 0; bit < 8; bit++) n = (n & 1) ? 0xedb88320 ^ (n >>> 1) : n >>> 1;
  return n >>> 0;
});
function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = crcTable[(crc ^ byte) & 255] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

// Validate PNG framing/checksums and the complete scanline stream. This checks
// file integrity, not the screenshot's visual content. Bound decompression of
// untrusted evidence; exceeding the limit is incomplete evidence, never success.
export function isCompletePng(bytes) {
  if (!bytes.subarray(0, 8).equals(signature)) return false;
  let header, paletteEntries = 0, hasTransparency = false, idatClosed = false, ended = false;
  const idat = [];
  let offset = 8;
  while (offset < bytes.length) {
    if (bytes.length - offset < 12) return false;
    const length = bytes.readUInt32BE(offset);
    if (length > bytes.length - offset - 12) return false;
    const type = bytes.toString('latin1', offset + 4, offset + 8);
    if (!/^[A-Za-z]{2}[A-Z][A-Za-z]$/.test(type)) return false;
    const end = offset + 8 + length;
    if (crc32(bytes.subarray(offset + 4, end)) !== bytes.readUInt32BE(end)) return false;
    const data = bytes.subarray(offset + 8, end);
    offset = end + 4;
    if (!header && type !== 'IHDR') return false;
    if (idat.length && type !== 'IDAT') idatClosed = true;
    if (type === 'IHDR') {
      if (header || length !== 13) return false;
      const width = data.readUInt32BE(0), height = data.readUInt32BE(4);
      const depth = data[8], color = data[9];
      const depths = { 0: [1, 2, 4, 8, 16], 2: [8, 16], 3: [1, 2, 4, 8], 4: [8, 16], 6: [8, 16] };
      if (!width || !height || width > 0x7fffffff || height > 0x7fffffff ||
          !depths[color]?.includes(depth) || data[10] !== 0 || data[11] !== 0 || data[12] > 1) return false;
      header = { width, height, depth, color, interlace: data[12] };
    } else if (type === 'PLTE') {
      if (paletteEntries || idat.length || [0, 4].includes(header.color) || !length || length % 3 || length > 768) return false;
      paletteEntries = length / 3;
      if (header.color === 3 && paletteEntries > 2 ** header.depth) return false;
    } else if (type === 'tRNS') {
      if (hasTransparency || idat.length || [4, 6].includes(header.color)) return false;
      if ((header.color === 0 && length !== 2) || (header.color === 2 && length !== 6) ||
          (header.color === 3 && (!paletteEntries || !length || length > paletteEntries))) return false;
      hasTransparency = true;
    } else if (type === 'IDAT') {
      if (idatClosed || (header.color === 3 && !paletteEntries)) return false;
      idat.push(data);
    } else if (type === 'IEND') {
      if (length || !idat.length || offset !== bytes.length) return false;
      ended = true;
      break;
    } else if (type[0] === type[0].toUpperCase()) {
      return false; // Unknown critical chunks cannot be safely interpreted.
    }
  }
  if (!ended) return false;
  const channels = { 0: 1, 2: 3, 3: 1, 4: 2, 6: 4 }[header.color];
  const passes = header.interlace ? [
    [0, 0, 8, 8], [4, 0, 8, 8], [0, 4, 4, 8], [2, 0, 4, 4],
    [0, 2, 2, 4], [1, 0, 2, 2], [0, 1, 1, 2],
  ] : [[0, 0, 1, 1]];
  const rows = [];
  let expected = 0;
  for (const [x, y, dx, dy] of passes) {
    const width = Math.max(0, Math.ceil((header.width - x) / dx));
    const height = Math.max(0, Math.ceil((header.height - y) / dy));
    if (!width || !height) continue;
    const stride = 1 + Math.ceil(width * channels * header.depth / 8);
    expected += stride * height;
    if (!Number.isSafeInteger(expected) || expected > maxDecodedBytes) return false;
    rows.push({ stride, height });
  }
  try {
    const compressed = Buffer.concat(idat);
    const { buffer, engine } = inflateSync(compressed, { maxOutputLength: expected, info: true });
    if (buffer.length !== expected || engine.bytesWritten !== compressed.length) return false;
    let position = 0;
    for (const { stride, height } of rows) {
      for (let row = 0; row < height; row++, position += stride) {
        if (buffer[position] > 4) return false;
      }
    }
    return true;
  } catch {
    return false;
  }
}
