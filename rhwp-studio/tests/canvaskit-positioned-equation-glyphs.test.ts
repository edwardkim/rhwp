// 관계형 API 계약: 분할 face의 coverage·생산 좌표·원자적 paint·자원 해제를 검사한다.
// 주석 갱신: 2026-11-07 KST. 합성 글리프/host 목록을 사용하므로 시스템 글꼴과 무관하다.
// 실문서의 시각 점수를 이 API 검사의 측정값으로 사용하지 않는다.
// 실제 Print 대조와 공통 수식 보정 근거: mydocs/pr/pr_8642_review_impl.md.
import test from 'node:test';
import assert from 'node:assert/strict';
import type { Canvas, CanvasKit, Paint, Typeface } from 'canvaskit-wasm';
import type { LayerEquationLayoutBox, PageLayerTree } from '../src/core/types.ts';
import { drawPositionedEquationGlyphs } from '../src/view/canvaskit/positioned-equation-glyphs.ts';
import { setHostFontProvider } from '../src/core/local-fonts.ts';
import { collectHostFontRequests } from '../src/core/host-font-requests.ts';

interface FakeFace { name: string; chars: string }
function harness() {
  const faces: FakeFace[] = [{ name: 'Hangul', chars: '가𝑥' }, { name: 'Latin', chars: 'x=가' }];
  const draws: Array<[string, string, number, number]> = [];
  const translations: number[][] = [], matrices: number[][] = [];
  const deleted: string[] = [];
  let paintCreates = 0, paintDeletes = 0, saves = 0, restores = 0;
  class FakeFont {
    face: FakeFace;
    constructor(face: FakeFace) { this.face = face; }
    setHinting(value: number) { assert.equal(value, 0); }
    setSubpixel(value: boolean) { assert.equal(value, true); }
    getGlyphIDs(text: string, count: number) {
      assert.equal(count, 1);
      assert.equal([...text].length, 1, '보충 평면도 한 scalar로 조회한다');
      return Uint16Array.of(this.face.chars.includes(text) ? 4 : 0);
    }
    delete() { deleted.push(this.face.name); }
  }
  const canvas = {
    save() { saves++; }, restore() { restores++; },
    translate(x: number, y: number) { translations.push([x, y]); },
    concat(matrix: number[]) { matrices.push(matrix); },
    drawText(text: string, x: number, y: number, _paint: Paint, font: FakeFont) {
      draws.push([text, font.face.name, x, y]);
    },
  };
  const context = {
    canvasKit: { Font: FakeFont, FontHinting: { None: 0 } } as unknown as CanvasKit,
    typefaces: faces as unknown as Typeface[], maxTextLength: 4096,
    makeFillPaint: () => {
      paintCreates++;
      return { delete() { paintDeletes++; } } as Paint;
    },
  };
  const run: Extract<LayerEquationLayoutBox['kind'], { type: 'hancomGlyphs' }> = {
    type: 'hancomGlyphs', glyphs: '가x𝑥=', positions: [0, 12, 19, 32],
    shears: [0, .25, 0, 0], baselineShift: -2,
  };
  const draw = () => drawPositionedEquationGlyphs(context, canvas as unknown as Canvas, run, 10, 20, 12, '#000');
  return { context, canvas, run, draw, draws, translations, matrices, deleted,
    counts: () => ({ paintCreates, paintDeletes, saves, restores }) };
}

test('분할 프로그램의 scalar별 coverage와 생산된 위치·기울임을 보존한다', () => {
  const h = harness();
  assert.equal(h.draw(), true);
  assert.deepEqual(h.draws, [['가', 'Hangul', 0, 0], ['x', 'Latin', 0, 0], ['𝑥', 'Hangul', 0, 0], ['=', 'Latin', 0, 0]]);
  assert.deepEqual(h.translations, [[10, 18], [22, 18], [29, 18], [42, 18]]);
  assert.equal(h.matrices[1][1], -.25);
  assert.deepEqual(h.counts(), { paintCreates: 1, paintDeletes: 1, saves: 4, restores: 4 });
  assert.deepEqual(h.deleted, ['Hangul', 'Latin']);
});

test('첫 후보가 가진 glyph는 뒤 후보로 바꾸지 않는다', () => {
  const h = harness();
  Object.assign(h.run, { glyphs: '가', positions: [0], shears: [0] });
  assert.equal(h.draw(), true);
  assert.deepEqual(h.draws, [['가', 'Hangul', 0, 0]]);
  assert.deepEqual(h.deleted, ['Hangul']);
});

test('런 뒤쪽 coverage가 없으면 앞부분도 그리지 않는다', () => {
  const h = harness(); h.run.glyphs = '가x?=';
  assert.equal(h.draw(), false);
  assert.deepEqual(h.draws, []);
  assert.deepEqual(h.deleted, ['Hangul', 'Latin']);
  assert.deepEqual(h.counts(), { paintCreates: 0, paintDeletes: 0, saves: 0, restores: 0 });
});

test('scalar 수·좌표·기울임이 유효하지 않으면 자원을 만들지 않는다', () => {
  for (const modify of [
    (h: ReturnType<typeof harness>) => h.run.positions.pop(),
    (h: ReturnType<typeof harness>) => { h.run.shears![1] = NaN; },
    (h: ReturnType<typeof harness>) => { h.run.positions[0] = Infinity; },
  ]) {
    const h = harness(); modify(h);
    assert.equal(h.draw(), false);
    assert.deepEqual(h.deleted, []);
    assert.equal(h.counts().paintCreates, 0);
  }
});

test('paint 예외에도 canvas 상태와 글꼴·paint를 해제한다', () => {
  const h = harness(); h.canvas.drawText = () => { throw Error('paint failure'); };
  assert.throws(h.draw, /paint failure/);
  assert.deepEqual(h.counts(), { paintCreates: 1, paintDeletes: 1, saves: 1, restores: 1 });
  assert.deepEqual(h.deleted, ['Hangul', 'Latin']);
});

test('host 준비는 family chain과 분할 프로그램을 바이트 읽기 없이 모두 수집한다', async () => {
  let reads = 0;
  const face = (id: string, family: string) => ({
    id, family, fullName: id, postscriptName: id, style: 'Regular', weight: 400, slant: 'normal' as const,
  });
  try {
    await setHostFontProvider({
      getSnapshot: async () => ({ revision: 'equation-split-contract', faces: [
        face('hangul', 'Split Family'), face('latin', 'Split Family'), face('fallback', 'Fallback'),
      ] }),
      readFace: async () => { reads++; return { bytes: new ArrayBuffer(0) }; },
      subscribe: () => () => {},
    });
    const bounds = { x: 0, y: 0, width: 200, height: 100 };
    const tree = { pageWidth: 200, pageHeight: 100, root: { kind: 'leaf', bounds, ops: [{
      type: 'equation', bbox: bounds, layoutBox: { ...bounds, baseline: 10, kind: {
        type: 'hancomGlyphs', fontFamily: '"Split Family", Fallback',
        glyphs: '가x', positions: [0, 12], baselineShift: 0,
      } },
    }] } } as unknown as PageLayerTree;
    assert.deepEqual(collectHostFontRequests(tree).map(r => r.hostReference?.face.id), ['hangul', 'latin', 'fallback']);
    assert.equal(reads, 0);
  } finally { await setHostFontProvider(null); }
});
