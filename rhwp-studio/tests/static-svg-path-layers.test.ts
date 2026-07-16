import assert from 'node:assert/strict';
import test from 'node:test';

import { parseStaticSvgPathLayers } from '../src/view/static-svg-path-layers.ts';

test('parses the font-native static SVG path subset', () => {
  const layers = parseStaticSvgPathLayers(
    '<svg viewBox="0 0 16 16"><metadata>fixture</metadata><path d="M2 2H14V14H2Z" fill="#00a0c8"/></svg>',
  );
  assert.equal(layers.length, 1);
  assert.equal(layers[0].pathData, 'M2 2H14V14H2Z');
  assert.equal(layers[0].fill, '#00a0c8');
});

test('inherits currentColor from the glyph paint style', () => {
  const [layer] = parseStaticSvgPathLayers(
    '<svg><path d="M0 0H1V1Z" fill="currentColor"/></svg>',
    '#123456',
  );
  assert.equal(layer?.fill, '#123456');
});

test('rejects active, external, escaped, and malformed SVG payloads', () => {
  const invalid = [
    '<svg><script>alert(1)</script><path d="M0 0H1V1Z"/></svg>',
    '<svg><path d="M0 0H1V1Z" fill="url(https://example.invalid/x.svg#p)"/></svg>',
    '<svg><path d="M0 0H1V1Z" fill="u\\72l(https://example.invalid/x.svg#p)"/></svg>',
    '<svg><path d="M0 0H1V1Z"/></svgx>',
  ];
  for (const fragment of invalid) {
    assert.deepEqual(parseStaticSvgPathLayers(fragment), [], fragment);
  }
});
