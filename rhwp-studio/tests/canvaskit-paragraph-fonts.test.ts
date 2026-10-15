import test from 'node:test';
import assert from 'node:assert/strict';
import type { CanvasKit, Typeface } from 'canvaskit-wasm';
import { CanvasKitParagraphFonts } from '../src/view/canvaskit/paragraph-fonts.ts';

function setup(maxEntries = 3, maxBytes = 12) {
  const providers: Array<{ deleted: boolean; registrations: Array<{ alias: string; bytes: number[] }>; delete(): void }> = [];
  const kit = { ParagraphBuilder: { MakeFromFontProvider() {} }, TypefaceFontProvider: { Make() {
    const provider = { deleted: false, registrations: [] as Array<{ alias: string; bytes: number[] }>,
      registerFont(bytes: Uint8Array, alias: string) { this.registrations.push({ alias, bytes: [...bytes] }); },
      delete() { this.deleted = true; } };
    providers.push(provider); return provider;
  } } } as unknown as CanvasKit;
  return { cache: new CanvasKitParagraphFonts(kit, maxEntries, maxBytes), providers };
}

test('prepared Paragraph fonts copy bytes, cap ownership, and dispose a stale provider', () => {
  const { cache, providers } = setup(1, 4), face = {} as Typeface, other = {} as Typeface;
  const bytes = Uint8Array.of(1, 2, 3, 4);
  assert.equal(cache.remember(face, bytes), true);
  assert.equal(cache.remember(other, bytes), false);
  bytes.fill(0);
  assert.ok(cache.source([face]));
  assert.deepEqual(providers[0].registrations[0].bytes, [1, 2, 3, 4]);
  cache.retain(new Set([other]));
  assert.equal(providers[0].deleted, true);
  assert.equal(cache.source([face]), null);
  assert.deepEqual(cache.diagnostics(), { entries: 0, bytes: 0, builds: 1, rejections: 1 });
  assert.equal(cache.remember(other, Uint8Array.of(1, 2, 3, 4, 5)), false);
  cache.clear();
});

test('prepared bundled styles share a Paragraph family and stale members invalidate it', () => {
  const { cache, providers } = setup(), regular = {} as Typeface, bold = {} as Typeface, fallback = {} as Typeface;
  for (const [face, byte] of [[regular, 1], [bold, 2], [fallback, 3]] as const) {
    assert.equal(cache.remember(face, Uint8Array.of(byte)), true);
  }
  const source = cache.source([[regular, bold], fallback])!;
  assert.equal(source.families.length, 2);
  const styles = providers[0].registrations.filter(entry => entry.alias === source.families[0]);
  assert.deepEqual(styles.map(entry => entry.bytes), [[1], [2]]);
  assert.equal(cache.source([[bold, regular], fallback])!.provider, source.provider);
  cache.retain(new Set([regular, fallback]));
  assert.equal(providers[0].deleted, true);
  assert.equal(cache.source([[regular, bold], fallback])!.families.length, 2);
  assert.equal(providers[1].registrations.length, 2, 'removed physical face is not registered again');
  cache.clear();
  assert.equal(providers[1].deleted, true);
});

test('additional Paragraph family registrations obey entry and byte caps', () => {
  const { cache } = setup(), a = {} as Typeface, b = {} as Typeface, c = {} as Typeface;
  for (const face of [a, b, c]) assert.equal(cache.remember(face, Uint8Array.of(1, 2, 3, 4)), true);
  assert.ok(cache.source([[a, b]]));
  assert.equal(cache.source([[a, c]]), null, 'overlapping alias groups cannot retain unlimited native copies');
  assert.ok(cache.source([a, c]), 'single prepared faces remain available after group rejection');
  cache.clear();
});
