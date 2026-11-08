import test from 'node:test';
import assert from 'node:assert/strict';
import { localFontFaceStyle } from '../src/view/canvaskit/local-font-style.ts';
import { replayableEncodedImageHeader, encodedImageHasStableFrame } from '../src/view/canvaskit/image-header.ts';
import { decodeBrowserImage } from '../src/view/canvaskit/browser-image-decoder.ts';

test('SFNT metadata outranks misleading bold and italic face names', () => {
  assert.deepEqual(localFontFaceStyle({ fullName: 'Bold Italic', postscriptName: '', style: '', weightClass: 300, italic: false } as never),
    { weight: 300, italic: false });
});

test('SVG image admission bounds intrinsic dimensions and rejects active or externally loaded content', () => {
  const header = (value: string) => replayableEncodedImageHeader(new TextEncoder().encode(value));
  assert.deepEqual(header('<svg width="1in" height="12pt"><rect width="1" height="1"/></svg>'),
    { format: 'svg', width: 96, height: 16 });
  assert.deepEqual(header('<svg viewBox="0 0 12 24"><path fill="url(\'#gradient\')" d="M0 0"/></svg>'),
    { format: 'svg', width: 12, height: 24 });
  for (const fragment of ['<script/>', '<animate/>', '<foreignObject/>', '<image href="#local"/>',
    '<path onclick="x"/>', '<path fill="url(https://example.test/image)"/>', '<use href="other.svg#a"/>',
    '<path style="fill:u\\72l(https://example.test/image)"/>', '<style>@im\\70ort "external.css"</style>',
    '<path fill="url&#40;https://example.test/image&#41;"/>', '<g xml:base="https://example.test"/>',
    '<style>@font-face{src:url(#font)}</style>', '<path style="fill:u/**/rl(https://example.test)"/>',
    '<?processing-instruction?>', '<text>&amp;</text>']) {
    assert.equal(header(`<svg width="10" height="10">${fragment}</svg>`), null, fragment);
  }
  assert.equal(header('<!DOCTYPE svg><svg width="10" height="10"/>'), null);
  assert.equal(header('<svg width="9000" height="10"/>'), null);
  assert.equal(header('<svg width="10" width="20" height="10"/>'), null);
  assert.equal(header('<svg width="10" height="10">' + ' '.repeat(4 * 1024 * 1024) + '</svg>'), null);
  assert.deepEqual(header('<?xml version="1.0"?><svg width="10" height="10"><!-- <g> --> <path d="M0 0"/></svg>'),
    { format: 'svg', width: 10, height: 10 });
});

test('SVG image admission validates comment bodies and retains markup boundaries', () => {
  const header = (value: string) => replayableEncodedImageHeader(new TextEncoder().encode(value));
  const dimensions = { format: 'svg', width: 10, height: 10 };
  assert.deepEqual(header('<!-- before --><svg width="10" height="10"><!----><g/><!-- <path/> -->\n</svg><!-- after -->'),
    dimensions);
  assert.deepEqual(header('<?xml version="1.0"?><!-- 설명 😀 --><svg width="10" height="10"/>'),
    dimensions);
  assert.deepEqual(header('<svg width="10" height="10"><text>한<!-- note -->글</text></svg>'),
    dimensions);
  assert.deepEqual(header('<svg width="10" height="10"><!-- note --><style>path {fill:red}</style></svg>'),
    dimensions);
  for (const comment of ['<!-- unfinished', '<!-- alpha--beta -->', '<!-- trailing--->', '<!-- \u0000 -->',
    '<!-- outer <!-- inner --> -->']) {
    assert.equal(header(`<svg width="10" height="10">${comment}</svg>`), null, comment);
  }
  // XML comments cannot interrupt a tag, attribute name, or attribute value.
  for (const source of ['<sv<!-- gap -->g width="10" height="10"/>',
    '<svg wid<!-- gap -->th="10" height="10"/>',
    '<svg width="1<!-- gap -->0" height="10"/>',
    '<svg width="10"<!-- gap --> height="10"/>']) {
    assert.equal(header(source), null, source);
  }
  // XML comments disappear from CSS text content. Admission cannot substitute
  // whitespace there while the browser receives the unmodified input bytes.
  for (const style of ['<style>path {fi<!-- note -->ll:red}</style>',
    '<s:style xmlns:s="http://www.w3.org/2000/svg">path {fill:red}<!-- note --></s:style>',
    '<style><style/><!-- note --></style>']) {
    assert.equal(header(`<svg width="10" height="10">${style}</svg>`), null, style);
  }
});

test('browser image decoder validates dimensions, revokes URLs, and ignores cancellation races', async () => {
  const header = { format: 'svg', width: 2, height: 3 } as const;
  let deleted = 0;
  let revoked = 0;
  let nativeCalls = 0;
  const element = { src: '', onload: null as null | (() => void), onerror: null as null | (() => void) };
  const kit = { MakeImageFromCanvasImageSource: () => { nativeCalls++; return {
    width: () => 2, height: () => 3, delete: () => { deleted++; },
  }; } };
  const env = { createImage: () => element, createObjectURL: () => 'blob:fixture', revokeObjectURL: () => { revoked++; } };
  const load = decodeBrowserImage(kit as never, Uint8Array.of(1), header, env as never);
  element.onload!();
  assert.notEqual(await load.promise, null);
  assert.equal(revoked, 1);
  assert.equal(element.onload, null);
  const cancelled = decodeBrowserImage(kit as never, Uint8Array.of(1), header, env as never);
  const staleOnload = element.onload!;
  cancelled.cancel();
  staleOnload();
  assert.equal(await cancelled.promise, null);
  assert.equal(revoked, 2);
  assert.equal(deleted, 1);
  assert.equal(nativeCalls, 2);
  const mismatch = decodeBrowserImage(kit as never, Uint8Array.of(1), { ...header, width: 10 }, env as never);
  element.onload!();
  assert.equal(await mismatch.promise, null);
  assert.equal(deleted, 2);
});

test('browser PNG fallback rejects APNG rather than sampling an arbitrary animation frame', () => {
  const png = Uint8Array.from(Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLttAAAAABJRU5ErkJggg==', 'base64'));
  const header = replayableEncodedImageHeader(png)!;
  assert.equal(encodedImageHasStableFrame(png, header), true);
  const animated = Uint8Array.from([...png.slice(0, 33), 0, 0, 0, 0, 0x61, 0x63, 0x54, 0x4c, 0, 0, 0, 0, ...png.slice(33)]);
  assert.equal(encodedImageHasStableFrame(animated, header), false);
});
