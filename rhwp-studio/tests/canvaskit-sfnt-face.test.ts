import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { canvasKitFontFaceData, canvasKitSfntGlyphCount } from '../src/view/canvaskit/sfnt-face.ts';

function fixtureBuffer(name: string): ArrayBuffer {
  const bytes = readFileSync(new URL(`../../tests/fixtures/fonts/${name}`, import.meta.url));
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
}

test('CanvasKit SFNT normalization selects exact TTC faces', () => {
  const collection = fixtureBuffer('RHWPExactFaceSmoke.ttc');
  const face0 = canvasKitFontFaceData(collection, 0);
  const face1 = canvasKitFontFaceData(collection, 1);

  assert.ok(face0);
  assert.ok(face1);
  assert.notEqual(new DataView(face0).getUint32(0, false), 0x7474_6366);
  assert.notEqual(new DataView(face1).getUint32(0, false), 0x7474_6366);
  assert.notDeepEqual(
    new Uint8Array(face0, 0, 64),
    new Uint8Array(face1, 0, 64),
    'each selected face must expose its own SFNT directory',
  );
  assert.equal(canvasKitFontFaceData(collection, 2), null);
});

test('CanvasKit SFNT normalization keeps standalone fonts on face zero', () => {
  const standalone = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  const selected = canvasKitFontFaceData(standalone, 0);

  assert.ok(selected);
  assert.deepEqual(new Uint8Array(selected), new Uint8Array(standalone));
  assert.equal(canvasKitFontFaceData(standalone, 1), null);
});

test('standalone SFNT tables are aligned without changing glyph or metric data', () => {
  const original = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  const source = new DataView(original);
  const count = source.getUint16(4, false);
  const directoryEnd = 12 + count * 16;
  // Some installed legacy fonts have valid table payloads at unaligned offsets.
  // OTS rejects these before FontFace can expose any glyphs.
  const shifted = new Uint8Array(original.byteLength + 1);
  shifted.set(new Uint8Array(original, 0, directoryEnd));
  shifted.set(new Uint8Array(original, directoryEnd), directoryEnd + 1);
  const input = new DataView(shifted.buffer);
  for (let i = 0; i < count; i++) {
    const record = 12 + i * 16;
    input.setUint32(record + 8, source.getUint32(record + 8, false) + 1, false);
  }
  const normalized = canvasKitFontFaceData(shifted.buffer, 0);
  assert.ok(normalized);
  const output = new DataView(normalized);
  for (let i = 0; i < count; i++) {
    const record = 12 + i * 16;
    const offset = output.getUint32(record + 8, false);
    const length = output.getUint32(record + 12, false);
    const originalOffset = source.getUint32(record + 8, false);
    assert.equal(offset % 4, 0, 'browser SFNT directory alignment');
    const before = new Uint8Array(original.slice(originalOffset, originalOffset + length));
    const after = new Uint8Array(normalized.slice(offset, offset + length));
    if (source.getUint32(record, false) === 0x68656164) {
      before.fill(0, 8, 12);
      after.fill(0, 8, 12);
    }
    assert.deepEqual(after, before, 'table payload, apart from head checksumAdjustment');
  }
  assert.equal(canvasKitFontFaceData(shifted.buffer, 1), null);
});

test('CanvasKit SFNT normalization rejects malformed collection directories', () => {
  const collection = fixtureBuffer('RHWPExactFaceSmoke.ttc');
  const malformed = collection.slice(0);
  const view = new DataView(malformed);
  const secondFaceOffset = view.getUint32(16, false);
  view.setUint32(secondFaceOffset + 20, 0, false);

  assert.equal(canvasKitFontFaceData(malformed, 1), null);
  assert.equal(canvasKitFontFaceData(new ArrayBuffer(8), 1), null);
  assert.equal(canvasKitFontFaceData(collection, -1), null);
  assert.equal(canvasKitFontFaceData(collection, 1.5), null);
});

test('empty VTT source tables do not block browser font loading or change runtime tables', () => {
  const original = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  const source = new DataView(original);
  const count = source.getUint16(4, false);
  const directoryEnd = 12 + count * 16;
  const input = new Uint8Array(original.byteLength + 16);
  input.set(new Uint8Array(original, 0, directoryEnd));
  input.set(new Uint8Array(original, directoryEnd), directoryEnd + 16);
  const view = new DataView(input.buffer);
  view.setUint16(4, count + 1, false);
  for (let i = 0; i < count; i++) {
    const record = 12 + i * 16;
    view.setUint32(record + 8, source.getUint32(record + 8, false) + 16, false);
  }
  // NanumGothic's empty TSI3 is VTT authoring metadata. OTS rejects its
  // directory entry before it can use the unchanged cmap/glyf/hmtx program.
  view.setUint32(directoryEnd, 0x54534933, false);
  view.setUint32(directoryEnd + 8, input.byteLength, false);
  const normalized = canvasKitFontFaceData(input.buffer, 0);
  assert.ok(normalized);
  const output = new DataView(normalized);
  assert.equal(output.getUint16(4, false), count, 'omit only the empty source record');
  for (let i = 0; i < count; i++) {
    const record = 12 + i * 16;
    const tag = source.getUint32(record, false);
    const found = Array.from({length: count}, (_, n) => 12 + n * 16)
      .find((offset) => output.getUint32(offset, false) === tag);
    assert.notEqual(found, undefined);
    const start = source.getUint32(record + 8, false);
    const length = source.getUint32(record + 12, false);
    const actual = output.getUint32(found! + 8, false);
    const before = new Uint8Array(original.slice(start, start + length));
    const after = new Uint8Array(normalized.slice(actual, actual + length));
    if (tag === 0x68656164) { before.fill(0, 8, 12); after.fill(0, 8, 12); }
    assert.deepEqual(after, before, 'runtime tables and nonempty source tables keep their bytes');
  }
});

test('CanvasKit SFNT normalization protects TTC v2 DSIG metadata', () => {
  const collection = fixtureBuffer('RHWPExactFaceSmoke.ttc');
  const malformed = collection.slice(0);
  const view = new DataView(malformed);
  const faceCount = view.getUint32(8, false);
  const secondFaceOffset = view.getUint32(16, false);
  const v2HeaderLength = 12 + faceCount * 4 + 12;

  view.setUint32(4, 0x0002_0000, false);
  view.setUint32(secondFaceOffset + 20, v2HeaderLength - 4, false);

  assert.equal(canvasKitFontFaceData(malformed, 1), null);
});

test('exact SFNT maxp proof distinguishes the selected TTC face and rejects malformed or compressed containers', () => {
  const standalone = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  assert.equal(canvasKitSfntGlyphCount(standalone), 7);
  const collection = fixtureBuffer('RHWPExactFaceSmoke.ttc');
  for (const index of [0, 1]) assert.ok(canvasKitSfntGlyphCount(canvasKitFontFaceData(collection, index)!)! > 0);
  assert.equal(canvasKitSfntGlyphCount(collection), null, 'a TTC directory is not a selected face');
  const malformed = standalone.slice(0), view = new DataView(malformed);
  for (let record = 12; record < 12 + view.getUint16(4, false) * 16; record += 16) {
    if (view.getUint32(record, false) === 0x6d61_7870) view.setUint32(record + 8, malformed.byteLength - 2, false);
  }
  assert.equal(canvasKitSfntGlyphCount(malformed), null);
  // `public/fonts` is a symlink to `assets/fonts`; read the target so a checkout without
  // symlink support (Windows `core.symlinks=false`) still finds the bundled font.
  const woff = readFileSync(new URL('../../assets/fonts/D2Coding-Regular.woff2', import.meta.url));
  assert.equal(canvasKitSfntGlyphCount(woff.buffer.slice(woff.byteOffset, woff.byteOffset + woff.byteLength)), null);
});

function tableRange(data: ArrayBuffer, tag: number): {offset: number; length: number} {
  const view = new DataView(data);
  for (let i = 0; i < view.getUint16(4, false); i++) {
    const at = 12 + i * 16;
    if (view.getUint32(at, false) === tag) return {
      offset: view.getUint32(at + 8, false), length: view.getUint32(at + 12, false),
    };
  }
  throw new Error(`missing table ${tag}`);
}

test('legacy profile headers preserve glyphs, hints, metrics and complete cmap ranges', () => {
  const original = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  const malformed = original.slice(0);
  const view = new DataView(malformed);
  const cmap = tableRange(malformed, 0x636d6170);
  const subtable = cmap.offset + view.getUint32(cmap.offset + 8, false);
  assert.equal(view.getUint16(subtable + 2, false), 32);
  view.setUint16(subtable + 2, 31, false); // all arrays are present; declaration is one byte short
  const maxp = tableRange(malformed, 0x6d617870);
  view.setUint16(maxp.offset + 14, 4, false); // TrueType has only Z0 and Z1
  const unchanged = malformed.slice(0);
  const normalized = canvasKitFontFaceData(malformed, 0);
  assert.ok(normalized);
  const output = new DataView(normalized);
  const outMaxp = tableRange(normalized, 0x6d617870);
  const outCmap = tableRange(normalized, 0x636d6170);
  const outSubtable = outCmap.offset + output.getUint32(outCmap.offset + 8, false);
  assert.equal(output.getUint16(outMaxp.offset + 14, false), 2);
  assert.equal(output.getUint16(outSubtable + 2, false), 32);
  assert.deepEqual(new Uint8Array(malformed), new Uint8Array(unchanged), 'caller bytes stay immutable');
  for (let i = 0; i < view.getUint16(4, false); i++) {
    const at = 12 + i * 16; const tag = view.getUint32(at, false);
    const before = tableRange(original, tag); const after = tableRange(normalized, tag);
    const a = new Uint8Array(original.slice(before.offset, before.offset + before.length));
    const b = new Uint8Array(normalized.slice(after.offset, after.offset + after.length));
    if (tag === 0x68656164) { a.fill(0, 8, 12); b.fill(0, 8, 12); }
    assert.deepEqual(b, a, 'restored metadata and all executable payload match the valid source');
  }
});

test('an incomplete range-offset cmap is not guessed from surrounding bytes', () => {
  const malformed = fixtureBuffer('RHWPBitmapSvgGlyphSmoke.ttf');
  const view = new DataView(malformed); const cmap = tableRange(malformed, 0x636d6170);
  const sub = cmap.offset + view.getUint32(cmap.offset + 8, false);
  view.setUint16(sub + 2, 31, false); view.setUint16(sub + 30, 2, false);
  const normalized = canvasKitFontFaceData(malformed, 0);
  assert.ok(normalized);
  assert.deepEqual(new Uint8Array(normalized), new Uint8Array(malformed), 'unsupported mappings still reach the browser validator unchanged');
});
