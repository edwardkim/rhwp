import assert from 'node:assert/strict';
import test from 'node:test';
import { isCompletePng } from './png-integrity.mjs';

// Tiny PNGs constructed from the PNG chunk/scanline specification with an
// independent zlib encoder and CRC32, not from the validator under test.
const pngs = {
  validAdam7: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAFNeavDAAAACklEQVR4nGOoBwAAgQCA05RTSgAAAABJRU5ErkJggg==',
  validPalette: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEUAAACnej3aAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==',
  shortRows: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAACCAAAAAC86un7AAAACklEQVR4nGOoBwAAgQCA05RTSgAAAABJRU5ErkJggg==',
  badFilter: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGOtBwAAiwCF/FENtAAAAABJRU5ErkJggg==',
  missingPalette: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==',
  largeImage: 'iVBORw0KGgoAAAANSUhEUgABhqAAAYagCAYAAACoUgvIAAAACklEQVR4nGNgAAAAAgABSK+kcQAAAABJRU5ErkJggg==',
  badDeflate: 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAAC0lEQVRub3QgZGVmbGF0ZWWCmHUAAAAASUVORK5CYII=',
};

test('PNG validation accepts complete palette and Adam7 images', () => {
  for (const name of ['validAdam7', 'validPalette']) assert.equal(isCompletePng(Buffer.from(pngs[name], 'base64')), true, name);
});

test('PNG validation rejects invalid scanlines, deflate, palette and excessive decode size', () => {
  for (const name of ['shortRows', 'badFilter', 'missingPalette', 'largeImage', 'badDeflate']) {
    assert.equal(isCompletePng(Buffer.from(pngs[name], 'base64')), false, name);
  }
});

test('PNG validation detects a corrupt CRC independently of compressed data', () => {
  const bytes = Buffer.from(pngs.validAdam7, 'base64');
  bytes[29] ^= 1; // IHDR CRC, leaving dimensions and image data intact.
  assert.equal(isCompletePng(bytes), false);
});
