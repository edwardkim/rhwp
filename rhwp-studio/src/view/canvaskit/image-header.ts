export interface EncodedImageDimensions {
  width: number;
  height: number;
}

/** CanvasKit decode 전에 bounded raster dimensions를 확인한다. */
export function encodedImageDimensions(bytes: Uint8Array): EncodedImageDimensions | null {
  if (bytes.byteLength < 10) return null;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let width = 0;
  let height = 0;

  if (
    bytes.byteLength >= 24
    && bytes[0] === 0x89
    && bytes[1] === 0x50
    && bytes[2] === 0x4e
    && bytes[3] === 0x47
    && bytes[4] === 0x0d
    && bytes[5] === 0x0a
    && bytes[6] === 0x1a
    && bytes[7] === 0x0a
    && bytes[12] === 0x49
    && bytes[13] === 0x48
    && bytes[14] === 0x44
    && bytes[15] === 0x52
  ) {
    width = view.getUint32(16, false);
    height = view.getUint32(20, false);
  } else if (
    bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46
    && bytes[3] === 0x38 && (bytes[4] === 0x37 || bytes[4] === 0x39) && bytes[5] === 0x61
  ) {
    width = view.getUint16(6, true);
    height = view.getUint16(8, true);
  } else if (bytes.byteLength >= 26 && bytes[0] === 0x42 && bytes[1] === 0x4d) {
    width = view.getInt32(18, true);
    height = Math.abs(view.getInt32(22, true));
  } else if (
    bytes.byteLength >= 30
    && bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46
    && bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  ) {
    if (bytes[12] === 0x56 && bytes[13] === 0x50 && bytes[14] === 0x38 && bytes[15] === 0x58) {
      width = 1 + bytes[24] + (bytes[25] << 8) + (bytes[26] << 16);
      height = 1 + bytes[27] + (bytes[28] << 8) + (bytes[29] << 16);
    } else if (
      bytes[12] === 0x56 && bytes[13] === 0x50 && bytes[14] === 0x38 && bytes[15] === 0x20
      && bytes[23] === 0x9d && bytes[24] === 0x01 && bytes[25] === 0x2a
    ) {
      width = view.getUint16(26, true) & 0x3fff;
      height = view.getUint16(28, true) & 0x3fff;
    } else if (
      bytes[12] === 0x56 && bytes[13] === 0x50 && bytes[14] === 0x38 && bytes[15] === 0x4c
      && bytes[20] === 0x2f
    ) {
      width = 1 + bytes[21] + ((bytes[22] & 0x3f) << 8);
      height = 1 + (bytes[22] >> 6) + (bytes[23] << 2) + ((bytes[24] & 0x0f) << 10);
    } else {
      return null;
    }
  } else if (bytes[0] === 0xff && bytes[1] === 0xd8) {
    let offset = 2;
    while (offset + 3 < bytes.byteLength) {
      if (bytes[offset] !== 0xff) {
        offset += 1;
        continue;
      }
      while (offset < bytes.byteLength && bytes[offset] === 0xff) offset += 1;
      if (offset >= bytes.byteLength) return null;
      const marker = bytes[offset];
      offset += 1;
      if (marker === 0xd9 || marker === 0xda) return null;
      if (marker === 0x01 || (marker >= 0xd0 && marker <= 0xd7)) continue;
      if (offset + 2 > bytes.byteLength) return null;
      const segmentLength = view.getUint16(offset, false);
      if (segmentLength < 2 || offset + segmentLength > bytes.byteLength) return null;
      const isStartOfFrame = (
        marker >= 0xc0 && marker <= 0xcf
        && ![0xc4, 0xc8, 0xcc].includes(marker)
      );
      if (isStartOfFrame) {
        if (segmentLength < 7) return null;
        width = view.getUint16(offset + 5, false);
        height = view.getUint16(offset + 3, false);
        break;
      }
      offset += segmentLength;
    }
  }
  return Number.isInteger(width) && Number.isInteger(height) && width > 0 && height > 0
    ? { width, height }
    : null;
}
