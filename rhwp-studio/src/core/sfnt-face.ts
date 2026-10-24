import { repairTerminalFontCmap } from './browser-font-cmap.ts';

const TTC_TAG = 0x7474_6366;
const TTC_VERSION_1 = 0x0001_0000;
const TTC_VERSION_2 = 0x0002_0000;
const MAX_TTC_FACE_COUNT = 4096;
const MAX_SFNT_TABLE_COUNT = 4096;
const MAX_NORMALIZED_SFNT_BYTES = 64 * 1024 * 1024;
const HEAD_TAG = 0x6865_6164;
const SFNT_CHECKSUM_MAGIC = 0xb1b0_afba;
// VTT source records are authoring metadata, not executable glyph instructions.
// Empty records (e.g. NanumGothic TSI3) make OTS reject an otherwise usable font.
const VTT_SOURCE_TAGS = new Set([0x5453_4930, 0x5453_4931, 0x5453_4932, 0x5453_4933, 0x5453_4935]);

export function normalizeFontFaceData(fontData: ArrayBuffer, faceIndex: number): ArrayBuffer | null {
  if (!Number.isInteger(faceIndex) || faceIndex < 0 || faceIndex > 0xffff_ffff) return null;

  const bytes = new Uint8Array(fontData);
  if (bytes.byteLength < 4) return faceIndex === 0 ? repairTerminalFontCmap(fontData.slice(0)) : null;
  const view = new DataView(fontData);
  const collection = view.getUint32(0, false) === TTC_TAG;
  let faceOffset = 0;
  let sourceHeaderLength = 0;
  if (collection) {
    if (bytes.byteLength < 12) return null;
    const version = view.getUint32(4, false);
    const faceCount = view.getUint32(8, false);
    if (faceCount > MAX_TTC_FACE_COUNT) return null;
    sourceHeaderLength = 12 + faceCount * 4 + (version === TTC_VERSION_2 ? 12 : 0);
    if (
      (version !== TTC_VERSION_1 && version !== TTC_VERSION_2)
      || faceCount === 0
      || faceIndex >= faceCount
      || sourceHeaderLength > bytes.byteLength
    ) return null;
    faceOffset = view.getUint32(12 + faceIndex * 4, false);
  } else {
    if (faceIndex !== 0) return null;
    // WOFF/WOFF2 and other browser-supported wrappers are not SFNT directories.
    const signature = view.getUint32(0, false);
    if (![0x0001_0000, 0x4f54_544f, 0x7472_7565, 0x7479_7031].includes(signature)) {
      return repairTerminalFontCmap(fontData.slice(0));
    }
    if (bytes.byteLength < 12) return null;
  }
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
  if (!collection) sourceHeaderLength = directoryLength;

  const tables: Array<{
    recordOffset: number;
    sourceOffset: number;
    length: number;
    outputOffset: number;
    repairedData?: Uint8Array;
  }> = [];
  let outputLength = directoryLength;
  for (let tableIndex = 0; tableIndex < tableCount; tableIndex += 1) {
    const recordOffset = faceOffset + 12 + tableIndex * 16;
    const tableOffset = view.getUint32(recordOffset + 8, false);
    const tableLength = view.getUint32(recordOffset + 12, false);
    if (
      (tableLength > 0 && tableOffset < sourceHeaderLength)
      || tableOffset > bytes.byteLength
      || tableLength > bytes.byteLength - tableOffset
    ) return null;
    if (tableLength === 0 && VTT_SOURCE_TAGS.has(view.getUint32(recordOffset, false))) continue;
    outputLength = Math.ceil(outputLength / 4) * 4;
    const outputOffset = outputLength;
    outputLength += tableLength;
    if (!Number.isSafeInteger(outputLength) || outputLength > MAX_NORMALIZED_SFNT_BYTES) return null;
    tables.push({ recordOffset, sourceOffset: tableOffset, length: tableLength, outputOffset });
  }
  if (tables.length === 0) return null;
  const maxp = tables.find((table) => view.getUint32(table.recordOffset, false) === 0x6d617870);
  const glyphCount = maxp && maxp.length >= 6 ? view.getUint16(maxp.sourceOffset + 4, false) : 0;
  const trueTypeGlyphs = [0x0001_0000, 0x7472_7565].includes(sfntVersion)
    && tables.some((table) => view.getUint32(table.recordOffset, false) === 0x676c7966 && table.length > 0);
  for (const table of tables) {
    const tag = view.getUint32(table.recordOffset, false);
    const data = bytes.subarray(table.sourceOffset, table.sourceOffset + table.length);
    if (tag === 0x636d6170) table.repairedData = repairMinimalFormat4Length(data, glyphCount) ?? undefined;
    if (tag === 0x6d617870 && trueTypeGlyphs && data.byteLength >= 32
        && view.getUint32(table.sourceOffset, false) === 0x0001_0000
        && view.getUint16(table.sourceOffset + 14, false) > 2) {
      // OpenType maxp describes allocation for only Z0/Z1. Executable hint programs stay unchanged.
      table.repairedData = data.slice();
      new DataView(table.repairedData.buffer).setUint16(14, 2, false);
    }
  }
  outputLength = Math.ceil(outputLength / 4) * 4;
  // Aligned standalone fonts keep their bytes, including the existing cmap repair.
  if (!collection && tables.length === tableCount && tables.every((table) => table.sourceOffset % 4 === 0 && !table.repairedData)) {
    return repairTerminalFontCmap(fontData.slice(0));
  }
  tables.sort((a, b) => view.getUint32(a.recordOffset, false) - view.getUint32(b.recordOffset, false));
  outputLength = 12 + tables.length * 16;
  for (const table of tables) {
    outputLength = Math.ceil(outputLength / 4) * 4;
    table.outputOffset = outputLength;
    outputLength += table.length;
  }
  outputLength = Math.ceil(outputLength / 4) * 4;

  const selected = new Uint8Array(outputLength);
  const selectedView = new DataView(selected.buffer);
  selected.set(bytes.subarray(faceOffset, faceOffset + 12), 0);
  const selector = Math.floor(Math.log2(tables.length));
  const searchRange = 16 * 2 ** selector;
  selectedView.setUint16(4, tables.length, false);
  selectedView.setUint16(6, searchRange, false);
  selectedView.setUint16(8, selector, false);
  selectedView.setUint16(10, tables.length * 16 - searchRange, false);
  let headOffset: number | null = null;
  for (let tableIndex = 0; tableIndex < tables.length; tableIndex += 1) {
    const table = tables[tableIndex];
    const outputRecordOffset = 12 + tableIndex * 16;
    selected.set(bytes.subarray(table.recordOffset, table.recordOffset + 8), outputRecordOffset);
    selectedView.setUint32(outputRecordOffset + 8, table.outputOffset, false);
    selectedView.setUint32(outputRecordOffset + 12, table.length, false);
    selected.set(table.repairedData ?? bytes.subarray(table.sourceOffset, table.sourceOffset + table.length), table.outputOffset);
    if (view.getUint32(table.recordOffset, false) === HEAD_TAG && table.length >= 12) {
      headOffset = table.outputOffset;
    }
  }

  if (headOffset !== null) selectedView.setUint32(headOffset + 8, 0, false);
  for (let index = 0; index < tables.length; index++) {
    const table = tables[index]; let sum = 0;
    for (let byte = 0; byte < table.length; byte += 4) {
      sum = (sum + selectedView.getUint32(table.outputOffset + byte, false)) >>> 0;
    }
    selectedView.setUint32(12 + index * 16 + 4, sum, false);
  }
  if (headOffset !== null) {
    let checksum = 0;
    for (let offset = 0; offset < selected.byteLength; offset += 4) {
      checksum = (checksum + selectedView.getUint32(offset, false)) >>> 0;
    }
    selectedView.setUint32(headOffset + 8, (SFNT_CHECKSUM_MAGIC - checksum) >>> 0, false);
  }
  return repairTerminalFontCmap(selected.buffer);
}

// A constant-delta format 4 has a fully determined minimum byte count. Repair
// only a one-byte-short declaration when every complete array is already there.
// Range-offset glyph arrays and missing bytes remain the browser validator's responsibility.
function repairMinimalFormat4Length(data: Uint8Array, glyphCount: number): Uint8Array | null {
  if (data.byteLength < 4 || glyphCount <= 0) return null;
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const count = view.getUint16(2, false); const header = 4 + count * 8;
  if (header > data.byteLength) return null;
  const offsets = [...new Set(Array.from({length: count}, (_, index) => view.getUint32(8 + index * 8, false)))].sort((left, right) => left - right);
  let output: Uint8Array | null = null;
  for (let subtableIndex = 0; subtableIndex < offsets.length; subtableIndex++) {
    const offset = offsets[subtableIndex];
    if (offset < header || offset + 14 > data.byteLength || view.getUint16(offset, false) !== 4) continue;
    const doubled = view.getUint16(offset + 6, false); const segments = doubled / 2;
    const minimum = 16 + 8 * segments;
    const limit = Math.min(data.byteLength, offsets[subtableIndex + 1] ?? data.byteLength);
    if (doubled % 2 || segments < 1 || minimum > 0xffff || offset + minimum !== limit
        || view.getUint16(offset + 2, false) + 1 !== minimum
        || view.getUint16(offset + 14 + 2 * segments, false) !== 0) continue;
    let previous = -1; let valid = true;
    for (let index = 0; index < segments; index++) {
      const end = view.getUint16(offset + 14 + 2 * index, false);
      const start = view.getUint16(offset + 16 + 2 * segments + 2 * index, false);
      const delta = view.getUint16(offset + 16 + 4 * segments + 2 * index, false);
      if (start > end || start <= previous
          || view.getUint16(offset + 16 + 6 * segments + 2 * index, false) !== 0
          || (index === segments - 1 && (start !== 0xffff || end !== 0xffff))) { valid = false; break; }
      for (let code = start; code <= end; code++) {
        if (((code + delta) & 0xffff) >= glyphCount) { valid = false; break; }
      }
      if (!valid) break;
      previous = end;
    }
    if (valid) {
      output ??= data.slice();
      new DataView(output.buffer).setUint16(offset + 2, minimum, false);
    }
  }
  return output;
}
