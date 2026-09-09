import assert from 'node:assert/strict';
import test from 'node:test';

import {
  buildWebfontCss,
  parseWebfontRules,
  prepareSvgForWebfontRaster,
  selectWebfontRules,
  svgViewport,
} from '../rasterize-svg-webfonts.mjs';

const projection = `export const FONT_RULE_CANVAS2D_WEBFONT_RULES = Object.freeze([
  {
    "ruleId": "rule.demo",
    "sourceFace": "테스트 고딕",
    "supply": {
      "fontFamily": "테스트 고딕",
      "sourceUrl": "https://cdn.example.test/demo.woff2",
      "format": "woff2",
      "unicodeRange": null
    }
  }
]);`;

test('generated Studio projection supplies only font families used by the SVG', () => {
  const rules = parseWebfontRules(projection);
  const selected = selectWebfontRules('<svg><text font-family="테스트 고딕">한글</text></svg>', rules);
  assert.deepEqual(selected.map(rule => rule.ruleId), ['rule.demo']);
  assert.deepEqual(selectWebfontRules('<svg><text>한글</text></svg>', rules), []);
});

test('webfont raster preserves local declarations without shadowing them with webfonts', () => {
  const rules = parseWebfontRules(projection);
  const css = buildWebfontCss('/repo', rules);
  const svg = prepareSvgForWebfontRaster(
    '<svg width="600" height="120"><style>@font-face { font-family: "테스트 고딕"; src: local("없는 글꼴"); }</style><text font-family="테스트 고딕">한글</text></svg>',
    css,
  );
  assert.doesNotMatch(svg, /https:\/\/cdn\.example\.test\/demo\.woff2/);
  assert.match(svg, /@font-face \{ font-family: "테스트 고딕"; src: local\("없는 글꼴"\); \}/);
  assert.match(svg, /__rhwp_visual_sweep_noto_sans_kr__/);
  assert.match(svg, /font-family="테스트 고딕, __rhwp_visual_sweep_noto_sans_kr__"/);
  assert.deepEqual(svgViewport(svg), { width: 600, height: 120 });
});

test('existing font-face families are not selected for supplemental supply', () => {
  const rules = parseWebfontRules(projection);
  const source = '<svg><style>@font-face { font-family: \'테스트 고딕\'; src: local("Batang"); }</style><text font-family="테스트 고딕">한글</text></svg>';
  assert.deepEqual(selectWebfontRules(source, rules), []);
});

test('legacy Korean face safety ordering and single-family descriptors survive preparation', () => {
  const face = '@font-face { font-family: "휴먼명조"; src: local("HCR Batang"), local("Batang"), local("휴먼명조"), local("HumanMyeongJo"); }';
  const source = `<svg width="10" height="10"><style>${face}.body { font-family: "휴먼명조", serif; }</style><text font-family="휴먼명조">한글</text></svg>`;
  const svg = prepareSvgForWebfontRaster(source, '');
  assert.ok(svg.includes(face));
  assert.match(svg, /font-family: "휴먼명조", serif, __rhwp_visual_sweep_noto_sans_kr__;/);
  assert.match(svg, /font-family="휴먼명조, __rhwp_visual_sweep_noto_sans_kr__"/);
});

test('webfont supply remains available when the SVG has no matching declaration', () => {
  const rules = parseWebfontRules(projection);
  const svg = prepareSvgForWebfontRaster('<svg><text font-family="테스트 고딕">한글</text></svg>', buildWebfontCss('/repo', rules));
  assert.match(svg, /https:\/\/cdn\.example\.test\/demo\.woff2/);
  assert.match(svg, /font-family: "테스트 고딕";/);
});

test('supply conflicts are matched against the supplied family case-insensitively', () => {
  const rules = [{ ruleId: 'alias', sourceFace: 'Alias', supply: { fontFamily: 'Safe Face', sourceUrl: 'https://cdn.example.test/safe.woff2', format: 'woff2' } }];
  const source = '<svg><style>@font-face { font-family: \'SAFE FACE\'; src: local("Safe"); }</style><text font-family="Alias">text</text></svg>';
  assert.deepEqual(selectWebfontRules(source, rules), []);
  const svg = prepareSvgForWebfontRaster(source, buildWebfontCss('/repo', rules));
  assert.doesNotMatch(svg, /https:\/\/cdn\.example\.test\/safe\.woff2/);
  assert.match(svg, /src: local\("Safe"\)/);
});
