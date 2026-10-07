import assert from 'node:assert/strict';
import test from 'node:test';
import type { LayerGlyphOutlineOp, LayerPathCommand } from '../src/core/types.ts';
import {
  glyphOutlinePayloadResourceKey,
  glyphOutlinePayloadStatus,
} from '../src/view/glyph-outline-payload-status.ts';

const F32_MAX = 2 ** 128 - 2 ** 104;
const OVERFLOW_VALUES = [1e100, -1e100, F32_MAX * (1 + Number.EPSILON), -F32_MAX * (1 + Number.EPSILON)];
const ALL_PAYLOADS = {
  allowMonochromeFillStroke: true,
  allowColrv0ColorLayers: true,
  allowColrv1Stage1ColorGraph: true,
  allowBitmapGlyph: true,
  allowSvgGlyph: true,
};

function affine() {
  return { a: 1, b: 0, c: 0, d: 1, e: -4, f: -8 };
}

function commands(): LayerPathCommand[] {
  return [
    { type: 'moveTo', x: -2, y: -3 },
    { type: 'lineTo', x: 4, y: 5 },
    { type: 'curveTo', x1: -1, y1: 2, x2: 3, y2: -4, x3: 5, y3: 6 },
    { type: 'arcTo', rx: 2, ry: 3, rotation: -45, largeArc: false, sweep: true, x: -4, y: -5 },
    { type: 'closePath' },
  ];
}

function monochrome() {
  return {
    type: 'glyphOutline',
    payloadKind: 'monochromeFill',
    bbox: { x: -10, y: -10, width: 20, height: 20 },
    placement: { baselineY: -3, runToPage: affine() },
    paths: [{ fillRule: 'nonzero', commands: commands() }],
  } satisfies LayerGlyphOutlineOp;
}

function stroked() {
  return {
    ...monochrome(),
    payloadKind: 'monochromeFillStroke',
    stroke: { width: 2, miterLimit: 4, join: 'miter', cap: 'butt', paintOrder: 'fillThenStroke' },
  } satisfies LayerGlyphOutlineOp;
}

function colrv0() {
  return {
    type: 'glyphOutline',
    payloadKind: 'colorLayers',
    bbox: monochrome().bbox,
    colorLayers: {
      colorFormat: 'colrV0',
      layers: [{
        commands: commands(),
        fill: { rgba: [1, 0, 0, 1] },
        fillRule: 'nonzero',
        sourceRangeUtf8: { start: 0, end: 1 },
        glyphRange: { start: 0, end: 1 },
        transformToRun: affine(),
      }],
    },
  } satisfies LayerGlyphOutlineOp;
}

type LeafKind = 'solidPath' | 'linearGradientPath' | 'radialGradientPath' | 'sweepGradientPath';

function colrv1(kind: LeafKind): LayerGlyphOutlineOp {
  const sourceFontRef = { faceKey: 'portable-geometry-fixture', glyphId: 1, colorFormat: 'colrV1' };
  const ranges = { sourceRangeUtf8: { start: 0, end: 1 }, glyphRange: { start: 0, end: 1 }, sourceFontRef };
  const stops = [{ offset: 0, color: { rgba: [1, 0, 0, 1] } }, { offset: 1, color: { rgba: [0, 0, 1, 1] } }];
  const path = { commands: commands(), fillRule: 'nonzero' };
  const payloads = {
    solidPath: { ...path, fill: { rgba: [1, 0, 0, 1] } },
    linearGradientPath: { ...path, gradient: { x0: -1, y0: -2, x1: 3, y1: 4, stops } },
    radialGradientPath: { ...path, gradient: { cx: -1, cy: -2, radius: 3, stops } },
    sweepGradientPath: { ...path, gradient: { cx: -1, cy: -2, startAngleDegrees: -180, endAngleDegrees: 180, stops } },
  };
  return {
    type: 'glyphOutline',
    payloadKind: 'colorLayers',
    bbox: monochrome().bbox,
    colorLayers: {
      colorFormat: 'colrV1',
      ...ranges,
      paintGraph: {
        rootNodeId: 0,
        nodes: [
          { nodeId: 0, kind: 'transform', transform: { childNodeId: 1, transform: affine() } },
          { nodeId: 1, kind, ...ranges, [kind]: payloads[kind] },
        ],
      },
    },
  };
}

function bitmap() {
  return {
    type: 'glyphOutline',
    payloadKind: 'bitmapGlyph',
    bbox: monochrome().bbox,
    bitmapGlyph: {
      imageRef: 1,
      scalingPolicy: 'sourceExact',
      placement: { x: -4, y: -8, width: 16, height: 24 },
      transformToRun: affine(),
    },
  } satisfies LayerGlyphOutlineOp;
}

function svg() {
  return {
    type: 'glyphOutline',
    payloadKind: 'svgGlyph',
    bbox: monochrome().bbox,
    svgGlyph: {
      svgRef: 1,
      staticSanitized: true,
      viewBox: { x: -4, y: -8, width: 16, height: 24 },
      intrinsicSize: { width: 16, height: 24 },
      transformToRun: affine(),
    },
  } satisfies LayerGlyphOutlineOp;
}

function expectAccepted(op: LayerGlyphOutlineOp) {
  assert.equal(glyphOutlinePayloadStatus(op, ALL_PAYLOADS).supported, true);
  if (['colorLayers', 'bitmapGlyph', 'svgGlyph'].includes(op.payloadKind ?? '')) {
    assert.equal(typeof glyphOutlinePayloadResourceKey(op), 'string');
  }
}

function expectRejected(op: LayerGlyphOutlineOp) {
  assert.equal(glyphOutlinePayloadStatus(op, ALL_PAYLOADS).supported, false);
  if (['colorLayers', 'bitmapGlyph', 'svgGlyph'].includes(op.payloadKind ?? '')) {
    assert.equal(glyphOutlinePayloadResourceKey(op), null);
  }
}

const LEAF_KINDS: LeafKind[] = ['solidPath', 'linearGradientPath', 'radialGradientPath', 'sweepGradientPath'];
const PATH_CASES = [
  { name: 'monochrome fill', create: monochrome, path: (op: LayerGlyphOutlineOp) => op.paths![0].commands! },
  { name: 'monochrome stroke', create: stroked, path: (op: LayerGlyphOutlineOp) => op.paths![0].commands! },
  { name: 'COLRv0', create: colrv0, path: (op: LayerGlyphOutlineOp) => op.colorLayers!.layers![0].commands! },
  ...LEAF_KINDS.map((kind) => ({
    name: `COLRv1 ${kind}`,
    create: () => colrv1(kind),
    path: (op: LayerGlyphOutlineOp) => op.colorLayers!.paintGraph!.nodes![1][kind]!.commands!,
  })),
];

for (const fixture of PATH_CASES) {
  test(`${fixture.name} accepts ordinary paths with negative coordinates`, () => expectAccepted(fixture.create()));
  for (const [index, command] of commands().entries()) {
    for (const field of Object.keys(command).filter((key) => typeof command[key as keyof LayerPathCommand] === 'number')) {
      test(`${fixture.name} checks ${command.type}.${field} at the f32 boundary`, () => {
        for (const value of OVERFLOW_VALUES) {
          const op = fixture.create();
          Object.assign(fixture.path(op)[index], { [field]: value });
          expectRejected(op);
        }
        for (const value of [-F32_MAX, F32_MAX]) {
          const op = fixture.create();
          Object.assign(fixture.path(op)[index], { [field]: value });
          expectAccepted(op);
        }
      });
    }
  }
}

for (const field of ['width', 'miterLimit'] as const) {
  test(`monochrome stroke checks ${field} at the f32 boundary`, () => {
    for (const value of OVERFLOW_VALUES) {
      const op = stroked();
      op.stroke[field] = value;
      expectRejected(op);
    }
    const op = stroked();
    op.stroke[field] = F32_MAX;
    expectAccepted(op);
  });
}

test('run baseline rejects finite f64 overflow and accepts both exact f32 limits', () => {
  for (const value of OVERFLOW_VALUES) {
    const op = monochrome();
    op.placement.baselineY = value;
    expectRejected(op);
  }
  for (const value of [-F32_MAX, F32_MAX]) {
    const op = monochrome();
    op.placement.baselineY = value;
    expectAccepted(op);
  }
});

const AFFINE_CASES = [
  { name: 'run placement', create: monochrome, transform: (op: LayerGlyphOutlineOp) => op.placement!.runToPage! },
  { name: 'COLRv0 layer', create: colrv0, transform: (op: LayerGlyphOutlineOp) => op.colorLayers!.layers![0].transformToRun! },
  { name: 'COLRv1 node', create: () => colrv1('solidPath'), transform: (op: LayerGlyphOutlineOp) => op.colorLayers!.paintGraph!.nodes![0].transform!.transform! },
  { name: 'bitmap glyph', create: bitmap, transform: (op: LayerGlyphOutlineOp) => op.bitmapGlyph!.transformToRun! },
  { name: 'SVG glyph', create: svg, transform: (op: LayerGlyphOutlineOp) => op.svgGlyph!.transformToRun! },
];

for (const fixture of AFFINE_CASES) {
  for (const field of ['a', 'b', 'c', 'd', 'e', 'f'] as const) {
    test(`${fixture.name} checks affine ${field} at the f32 boundary`, () => {
      for (const value of OVERFLOW_VALUES) {
        const op = fixture.create();
        fixture.transform(op)[field] = value;
        expectRejected(op);
      }
      for (const value of [-F32_MAX, F32_MAX]) {
        const op = fixture.create();
        fixture.transform(op)[field] = value;
        expectAccepted(op);
      }
    });
  }
}

for (const [kind, fields] of [
  ['linearGradientPath', ['x0', 'y0', 'x1', 'y1']],
  ['radialGradientPath', ['cx', 'cy', 'radius']],
  ['sweepGradientPath', ['cx', 'cy', 'startAngleDegrees', 'endAngleDegrees']],
] as const) {
  for (const field of fields) {
    test(`COLRv1 ${kind} checks gradient ${field} at the f32 boundary`, () => {
      for (const value of OVERFLOW_VALUES) {
        const op = colrv1(kind);
        Object.assign(op.colorLayers!.paintGraph!.nodes![1][kind]!.gradient!, { [field]: value });
        expectRejected(op);
      }
      // Sweep endpoints must retain a 360-degree span, which cannot be represented near f32::MAX.
      if (field === 'startAngleDegrees' || field === 'endAngleDegrees') return;
      for (const value of field === 'radius' ? [F32_MAX] : [-F32_MAX, F32_MAX]) {
        const op = colrv1(kind);
        Object.assign(op.colorLayers!.paintGraph!.nodes![1][kind]!.gradient!, { [field]: value });
        expectAccepted(op);
      }
    });
  }
}

const BOUNDS_CASES = [
  { name: 'bitmap placement', create: bitmap, bounds: (op: LayerGlyphOutlineOp) => op.bitmapGlyph!.placement! },
  { name: 'SVG viewBox', create: svg, bounds: (op: LayerGlyphOutlineOp) => op.svgGlyph!.viewBox! },
];

for (const fixture of BOUNDS_CASES) {
  test(`${fixture.name} accepts ordinary negative origins`, () => expectAccepted(fixture.create()));
  for (const field of ['x', 'y', 'width', 'height'] as const) {
    test(`${fixture.name} rejects finite f64 overflow in ${field}`, () => {
      for (const value of OVERFLOW_VALUES) {
        const op = fixture.create();
        fixture.bounds(op)[field] = value;
        expectRejected(op);
      }
    });
  }
  for (const [origin, extent] of [['x', 'width'], ['y', 'height']] as const) {
    test(`${fixture.name} rejects overflowing ${origin} + ${extent} with finite f32 components`, () => {
      const op = fixture.create();
      Object.assign(fixture.bounds(op), { [origin]: F32_MAX, [extent]: F32_MAX });
      expectRejected(op);
    });
    test(`${fixture.name} accepts exact f32 edge and extent boundaries on ${origin}`, () => {
      for (const [position, size] of [[F32_MAX / 2, F32_MAX / 2], [-F32_MAX, F32_MAX], [0, F32_MAX]]) {
        const op = fixture.create();
        Object.assign(fixture.bounds(op), { [origin]: position, [extent]: size });
        expectAccepted(op);
      }
    });
  }
}

for (const field of ['width', 'height'] as const) {
  test(`SVG intrinsic ${field} rejects finite f64 overflow and accepts exact f32::MAX`, () => {
    for (const value of OVERFLOW_VALUES) {
      const op = svg();
      op.svgGlyph.intrinsicSize[field] = value;
      expectRejected(op);
    }
    const op = svg();
    op.svgGlyph.intrinsicSize[field] = F32_MAX;
    expectAccepted(op);
  });
}

test('null affine transforms are rejected without throwing', () => {
  const invalidTransform = null as unknown as ReturnType<typeof affine>;
  const placed = monochrome();
  placed.placement.runToPage = invalidTransform;
  const layered = colrv0();
  layered.colorLayers.layers[0].transformToRun = invalidTransform;
  const graph = colrv1('solidPath');
  graph.colorLayers!.paintGraph!.nodes![0].transform!.transform = invalidTransform;
  const bitmapOp = bitmap();
  bitmapOp.bitmapGlyph.transformToRun = invalidTransform;
  const svgOp = svg();
  svgOp.svgGlyph.transformToRun = invalidTransform;
  for (const op of [placed, layered, graph, bitmapOp, svgOp]) expectRejected(op);
});

test('a null COLRv0 layer is rejected without throwing', () => {
  const op = colrv0();
  op.colorLayers.layers[0] = null as unknown as (typeof op.colorLayers.layers)[number];
  expectRejected(op);
});

test('a null SVG intrinsic size is rejected without throwing', () => {
  const op = svg();
  op.svgGlyph.intrinsicSize = null as unknown as typeof op.svgGlyph.intrinsicSize;
  expectRejected(op);
});

test('null bitmap placement and SVG viewBox are rejected without throwing', () => {
  const bitmapOp = bitmap();
  bitmapOp.bitmapGlyph.placement = null as unknown as typeof bitmapOp.bitmapGlyph.placement;
  const svgOp = svg();
  svgOp.svgGlyph.viewBox = null as unknown as typeof svgOp.svgGlyph.viewBox;
  expectRejected(bitmapOp);
  expectRejected(svgOp);
});
