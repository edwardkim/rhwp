import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';

let vite: ViteDevServer;
let prototype: Record<string, any>;
before(async () => {
  vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
});
after(async () => { await vite?.close(); });

// Source-position contracts, not a real document's absolute pixel baseline.
// The original no-lineseg fixture's six TABs are absent from independent Print.
// Deliberately nonzero control glyph ids detect accidental ink even with a font
// that provides a visible TAB glyph. Source text and following origins must stay.
const samples: Array<{ name: string; text: string; positions?: number[]; expected: unknown[] }> = [
  { name: 'tabs occupy source advances without glyph ink', text: '\t\t', positions: [0, 40, 80], expected: [] },
  { name: 'ink after a tab keeps its source origin', text: 'A\tB', positions: [0, 10, 40, 50], expected: [{ id: 65, x: 0 }, { id: 66, x: 40 }] },
  { name: 'zero-advance tab does not shift the following glyph', text: 'A\tB', positions: [0, 10, 10, 20], expected: [{ id: 65, x: 0 }, { id: 66, x: 10 }] },
  { name: 'ordinary positioned glyphs retain their ink', text: 'AB', positions: [0, 10, 20], expected: [{ id: 65, x: 0 }, { id: 66, x: 10 }] },
  { name: 'legacy nominal advance survives hidden tab ink', text: 'A\tB', expected: [{ id: 65, x: 0 }, { id: 66, x: 20 }] },
  { name: 'legacy tab-only run emits no glyph ink', text: '\t\t', expected: [] },
  { name: 'ordinary legacy text keeps its existing replay path', text: 'AB', expected: [{ text: 'AB', x: 0 }] },
];
for (const sample of samples) test(sample.name, () => {
  const calls: unknown[] = [];
  const unsupported = new Set<string>();
  const paint = { setAntiAlias() {}, delete() {} };
  const font = { getGlyphIDs(text: string) { return Uint16Array.from([...text].map(ch => ch.codePointAt(0)!)); },
    getGlyphWidths(ids: Uint16Array) { return [...ids].map(() => 10); }, delete() {} };
  const renderer = Object.create(prototype);
  Object.assign(renderer, { unsupportedOps: unsupported,
    currentReplayFeatureCounts: { verticalTextRuns: 0, verticalPresentationPunctuation: 0 },
    requirePreparedFontFamilies: false, findPreparedTypeface: () => ({ typeface: {}, fontFamily: 'ControlContract' }),
    makeFont: () => font, makeFillPaint: () => paint, affineToCanvasKitMatrix: () => null,
    drawCachedGlyphs(_canvas: unknown, ids: Uint16Array, positions: Float32Array) {
      for (let i = 0; i < ids.length; i += 1) calls.push({ id: ids[i], x: positions[i * 2] });
    }, drawCachedText(_canvas: unknown, text: string, x: number) { calls.push({ text, x }); },
    recordFontSubstitution() {} });
  const op = { type: 'textRun', text: sample.text, positions: sample.positions,
    bbox: { x: 0, y: 0, width: sample.positions?.at(-1) ?? sample.text.length * 10, height: 12 },
    baseline: 10, style: { fontFamily: 'ControlContract', fontSize: 12 } };
  const beforeSource = JSON.stringify(op);
  renderer.renderTextRun({ save() {}, restore() {} }, op);
  assert.deepEqual(calls, sample.expected);
  assert.deepEqual([...unsupported], []);
  assert.equal(JSON.stringify(op), beforeSource, '원문·전진 경계는 잉크 생략으로 바뀌지 않는다');
});


// 독립 HCR/HY신명조 6크기 Print 대조군의 밑줄 좌표를 사용한다.
// 실제 문서의 임시 배치를 고정하는 검사가 아니라, 주어진 paint descriptor를
// CanvasKit이 font size에 맞게 소비하는 합성 계약이다. 600DPI 종이 CTM만 역산해
// 논리 좌표에서 비교하며 허용치는 장치 반 점(0.08 CSSpx)과 기록 정밀도다.
const independentUnderlineSamples = [
  {
    "name": "hwp HxHxHx9",
    "bbox": {
      "x": 113.387,
      "y": 132.267,
      "width": 52.533,
      "height": 12
    },
    "baseline": 10.2,
    "fontSize": 12,
    "independentPaperY": 144.32267252604166
  },
  {
    "name": "hwp HxHxHx12",
    "bbox": {
      "x": 113.387,
      "y": 151.467,
      "width": 78.88,
      "height": 16
    },
    "baseline": 13.6,
    "fontSize": 16,
    "independentPaperY": 167.656005859375
  },
  {
    "name": "hwp HxHxHx15",
    "bbox": {
      "x": 113.387,
      "y": 177.067,
      "width": 98.453,
      "height": 20
    },
    "baseline": 17,
    "fontSize": 20,
    "independentPaperY": 197.22403971354166
  },
  {
    "name": "hwp HxHxHx18",
    "bbox": {
      "x": 113.387,
      "y": 209.067,
      "width": 118.293,
      "height": 24
    },
    "baseline": 20.4,
    "fontSize": 24,
    "independentPaperY": 233.34529622395834
  },
  {
    "name": "hwp HxHxHx24",
    "bbox": {
      "x": 113.387,
      "y": 247.467,
      "width": 157.6,
      "height": 32
    },
    "baseline": 27.2,
    "fontSize": 32,
    "independentPaperY": 279.8533528645833
  },
  {
    "name": "hwp HxHxHx36",
    "bbox": {
      "x": 113.387,
      "y": 298.667,
      "width": 236.64,
      "height": 48
    },
    "baseline": 40.8,
    "fontSize": 48,
    "independentPaperY": 347.2999674479167
  },
  {
    "name": "hys HxHxHx9",
    "bbox": {
      "x": 113.387,
      "y": 132.267,
      "width": 58.56,
      "height": 12
    },
    "baseline": 10.2,
    "fontSize": 12,
    "independentPaperY": 144.32267252604166
  },
  {
    "name": "hys HxHxHx12",
    "bbox": {
      "x": 113.387,
      "y": 151.467,
      "width": 88.053,
      "height": 16
    },
    "baseline": 13.6,
    "fontSize": 16,
    "independentPaperY": 167.656005859375
  },
  {
    "name": "hys HxHxHx15",
    "bbox": {
      "x": 113.387,
      "y": 177.067,
      "width": 109.92,
      "height": 20
    },
    "baseline": 17,
    "fontSize": 20,
    "independentPaperY": 197.22403971354166
  },
  {
    "name": "hys HxHxHx18",
    "bbox": {
      "x": 113.387,
      "y": 209.067,
      "width": 131.893,
      "height": 24
    },
    "baseline": 20.4,
    "fontSize": 24,
    "independentPaperY": 233.34529622395834
  },
  {
    "name": "hys HxHxHx24",
    "bbox": {
      "x": 113.387,
      "y": 247.467,
      "width": 176,
      "height": 32
    },
    "baseline": 27.2,
    "fontSize": 32,
    "independentPaperY": 279.8533528645833
  },
  {
    "name": "hys HxHxHx36",
    "bbox": {
      "x": 113.387,
      "y": 298.667,
      "width": 264.053,
      "height": 48
    },
    "baseline": 40.8,
    "fontSize": 48,
    "independentPaperY": 347.2999674479167
  }
] as const;
for (const sample of independentUnderlineSamples) test(`bottom underline consumes Print size: ${sample.name}`, () => {
  const lines: Array<{ x1: number; y1: number; x2: number; y2: number }> = [];
  const renderer = Object.create(prototype);
  Object.assign(renderer, {
    unsupportedOps: new Set<string>(),
    drawTextVisualStroke(_canvas: unknown, x1: number, y1: number, x2: number, y2: number) {
      lines.push({ x1, y1, x2, y2 });
    },
  });
  const op = { type: 'textDecoration', bbox: { ...sample.bbox }, decoration: {
    kind: 'underline', baseline: sample.baseline, rotation: 0, isVertical: false,
    fontSize: sample.fontSize, ratio: 1, color: '#000000', shape: 0,
    underline: 'bottom', emphasisDot: 0, positions: [0, sample.bbox.width], positionsComplete: true,
  } };
  const before = JSON.stringify(op);
  renderer.renderTextDecoration({}, op);
  assert.equal(lines.length, 1, '밑줄은 같은 원문 run의 한 구간을 그린다');
  const printerScaleY = (841 / 7016) * (600 / 72);
  const expectedLogicalY = sample.independentPaperY / printerScaleY;
  assert.ok(Math.abs(lines[0].y1 - expectedLogicalY) <= 0.081,
    `${sample.name}: ${lines[0].y1} vs independent Print ${expectedLogicalY}`);
  assert.equal(lines[0].y1, lines[0].y2);
  assert.equal(lines[0].x1, sample.bbox.x);
  assert.equal(lines[0].x2, sample.bbox.x + sample.bbox.width);
  assert.deepEqual([...renderer.unsupportedOps], []);
  assert.equal(JSON.stringify(op), before, '원문 소속·전진·glyph descriptor를 변경하지 않는다');
});
