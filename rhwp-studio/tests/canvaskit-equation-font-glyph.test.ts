import test from 'node:test';
import assert from 'node:assert/strict';
import type { Canvas, CanvasKit, Paint, Typeface } from 'canvaskit-wasm';
import { drawDeclaredEquationFontGlyph } from '../src/view/canvaskit/equation-font-glyph.ts';

function harness(available: Record<string, boolean>) {
  const lookups: string[] = [];
  const created: string[] = [];
  const deleted: string[] = [];
  const draws: Array<{ text: string; family: string; x: number; baseline: number }> = [];
  const fallbacks: Array<[string, string]> = [];
  const faces = new Map(Object.entries(available).map(([family, hasGlyph]) => [family, { family, hasGlyph }]));
  class FakeFont {
    face: { family: string; hasGlyph: boolean };
    constructor(face: { family: string; hasGlyph: boolean }) {
      this.face = face;
      created.push(face.family);
    }
    getGlyphIDs(text: string, count: number) {
      assert.equal(count, Array.from(text).length, 'UTF-16 길이가 아니라 scalar 수를 전달해야 함');
      return new Uint16Array(Array.from(text).map(() => this.face.hasGlyph ? 1 : 0));
    }
    getGlyphWidths(ids: Uint16Array) { return Array.from(ids).map(() => 1); }
    delete() { deleted.push(this.face.family); }
  }
  const canvas = {
    drawText(text: string, x: number, baseline: number, _paint: Paint, font: FakeFont) {
      draws.push({ text, x, baseline, family: font.face.family });
    },
  } as unknown as Canvas;
  const context = {
    canvasKit: { Font: FakeFont } as unknown as CanvasKit,
    maxTextLength: 4096,
    lookupTypeface(family: string) {
      lookups.push(family);
      return faces.get(family) as unknown as Typeface ?? null;
    },
    makeFillPaint: () => ({ delete() {} }) as Paint,
    onFallback(requested: string, resolved: string) { fallbacks.push([requested, resolved]); },
  };
  const draw = (family: string, text = '∙') => {
    const owner = { x: 7, baseline: 11, reservedWidth: 19, fontSize: 12 };
    const ok = drawDeclaredEquationFontGlyph(context, canvas, text, family,
      owner.x, owner.baseline, owner.fontSize, '#000', owner.reservedWidth);
    if (draws.length) {
      assert.equal(draws[0].x, owner.x, '논리 예약 폭으로 글리프를 가운데로 옮기면 안 됨');
      assert.equal(draws[0].baseline, owner.baseline, '생산된 기준선을 다시 추정하면 안 됨');
    }
    return ok;
  };
  return { draw, draws, lookups, created, deleted, fallbacks };
}

test('선언한 첫 face에 glyph가 있으면 뒤 후보로 바꾸지 않는다', () => {
  const h = harness({ HYhwpEQ: true, 'Latin Modern Math': true });
  assert.equal(h.draw('HYhwpEQ, Latin Modern Math'), true);
  assert.equal(h.draws[0].family, 'HYhwpEQ');
  assert.deepEqual(h.created, ['HYhwpEQ']);
  assert.deepEqual(h.fallbacks, []);
  assert.deepEqual(h.deleted, ['HYhwpEQ']);
});

test('glyph가 없는 첫 face 뒤에서 선언된 실제 glyph 후보를 선택하고 기록한다', () => {
  const h = harness({ HYhwpEQ: false, 'Latin Modern Math': true });
  assert.equal(h.draw('HYhwpEQ, "Latin Modern Math"'), true);
  assert.equal(h.draws[0].family, 'Latin Modern Math');
  assert.deepEqual(h.fallbacks, [['HYhwpEQ', 'Latin Modern Math']]);
  assert.deepEqual(h.deleted, ['HYhwpEQ', 'Latin Modern Math']);
});

test('목록에 없는 기본 face는 준비돼 있어도 요청 후보로 끼워 넣지 않는다', () => {
  const h = harness({ HYhwpEQ: false, 'Latin Modern Math': true });
  assert.equal(h.draw('HYhwpEQ'), false);
  assert.deepEqual(h.lookups, ['HYhwpEQ']);
  assert.deepEqual(h.draws, []);
  assert.deepEqual(h.fallbacks, []);
});

test('준비된 후보 모두 glyph가 없으면 성공이나 대체 표시를 보고하지 않는다', () => {
  const h = harness({ HYhwpEQ: false, 'Latin Modern Math': false });
  assert.equal(h.draw('HYhwpEQ, Latin Modern Math'), false);
  assert.deepEqual(h.draws, []);
  assert.deepEqual(h.fallbacks, []);
  assert.deepEqual(h.deleted, ['HYhwpEQ', 'Latin Modern Math']);
});

test('보충 평면 문자의 glyph 수와 선택한 face를 보존한다', () => {
  const h = harness({ 'Latin Modern Math': true });
  assert.equal(h.draw("'Latin Modern Math'", '𝑥'), true);
  assert.equal(h.draws[0].text, '𝑥');
  assert.deepEqual(h.fallbacks, []);
});
