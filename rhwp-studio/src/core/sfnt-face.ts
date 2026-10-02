const TTC_TAG = 0x7474_6366;
const TTC_VERSION_1 = 0x0001_0000;
const TTC_VERSION_2 = 0x0002_0000;
const MAX_TTC_FACE_COUNT = 4096;
const MAX_SFNT_TABLE_COUNT = 4096;
const MAX_NORMALIZED_SFNT_BYTES = 64 * 1024 * 1024;
const HEAD_TAG = 0x6865_6164;
const SFNT_CHECKSUM_MAGIC = 0xb1b0_afba;

export function normalizeFontFaceData(fontData: ArrayBuffer, faceIndex: number): ArrayBuffer | null {
  if (!Number.isInteger(faceIndex) || faceIndex < 0 || faceIndex > 0xffff_ffff) return null;

  const bytes = new Uint8Array(fontData);
  if (bytes.byteLength < 4) return faceIndex === 0 ? fontData.slice(0) : null;
  const view = new DataView(fontData);
  if (view.getUint32(0, false) !== TTC_TAG) return faceIndex === 0 ? fontData.slice(0) : null;
  if (bytes.byteLength < 12) return null;

  const version = view.getUint32(4, false);
  const faceCount = view.getUint32(8, false);
  if (faceCount > MAX_TTC_FACE_COUNT) return null;
  const collectionHeaderLength = 12 + faceCount * 4 + (version === TTC_VERSION_2 ? 12 : 0);
  if (
    (version !== TTC_VERSION_1 && version !== TTC_VERSION_2)
    || faceCount === 0
    || faceIndex >= faceCount
    || collectionHeaderLength > bytes.byteLength
  ) return null;

  const faceOffset = view.getUint32(12 + faceIndex * 4, false);
  if (faceOffset > bytes.byteLength - 12) return null;
  const sfntVersion = view.getUint32(faceOffset, false);
  const tableCount = view.getUint16(faceOffset + 4, false);
  const directoryLength = 12 + tableCount * 16;
  if (
    (
      sfntVersion !== 0x0001_0000
      && sfntVersion !== 0x4f54_544f
      && sfntVersion !== 0x7472_7565
      && sfntVersion !== 0x7479_7031
    )
    || tableCount === 0
    || tableCount > MAX_SFNT_TABLE_COUNT
    || directoryLength > bytes.byteLength - faceOffset
  ) return null;

  const tables: Array<{
    recordOffset: number;
    sourceOffset: number;
    length: number;
    outputOffset: number;
  }> = [];
  let outputLength = directoryLength;
  for (let tableIndex = 0; tableIndex < tableCount; tableIndex += 1) {
    const recordOffset = faceOffset + 12 + tableIndex * 16;
    const tableOffset = view.getUint32(recordOffset + 8, false);
    const tableLength = view.getUint32(recordOffset + 12, false);
    if (
      (tableLength > 0 && tableOffset < collectionHeaderLength)
      || tableOffset > bytes.byteLength
      || tableLength > bytes.byteLength - tableOffset
    ) return null;
    outputLength = Math.ceil(outputLength / 4) * 4;
    const outputOffset = outputLength;
    outputLength += tableLength;
    if (!Number.isSafeInteger(outputLength) || outputLength > MAX_NORMALIZED_SFNT_BYTES) return null;
    tables.push({ recordOffset, sourceOffset: tableOffset, length: tableLength, outputOffset });
  }
  outputLength = Math.ceil(outputLength / 4) * 4;

  const selected = new Uint8Array(outputLength);
  const selectedView = new DataView(selected.buffer);
  selected.set(bytes.subarray(faceOffset, faceOffset + 12), 0);
  let headOffset: number | null = null;
  for (let tableIndex = 0; tableIndex < tables.length; tableIndex += 1) {
    const table = tables[tableIndex];
    const outputRecordOffset = 12 + tableIndex * 16;
    selected.set(bytes.subarray(table.recordOffset, table.recordOffset + 8), outputRecordOffset);
    selectedView.setUint32(outputRecordOffset + 8, table.outputOffset, false);
    selectedView.setUint32(outputRecordOffset + 12, table.length, false);
    selected.set(bytes.subarray(table.sourceOffset, table.sourceOffset + table.length), table.outputOffset);
    if (view.getUint32(table.recordOffset, false) === HEAD_TAG && table.length >= 12) {
      headOffset = table.outputOffset;
    }
  }

  if (headOffset !== null) {
    selectedView.setUint32(headOffset + 8, 0, false);
    let checksum = 0;
    for (let offset = 0; offset < selected.byteLength; offset += 4) {
      checksum = (checksum + selectedView.getUint32(offset, false)) >>> 0;
    }
    selectedView.setUint32(headOffset + 8, (SFNT_CHECKSUM_MAGIC - checksum) >>> 0, false);
  }
  return selected.buffer;
}
