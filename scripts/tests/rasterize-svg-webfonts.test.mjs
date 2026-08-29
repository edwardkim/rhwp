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

test('webfont raster replaces local declarations and retains a terminal Korean fallback', () => {
  const rules = parseWebfontRules(projection);
  const css = buildWebfontCss('/repo', rules);
  const svg = prepareSvgForWebfontRaster(
    '<svg width="600" height="120"><style>@font-face { font-family: "테스트 고딕"; src: local("없는 글꼴"); }</style><text font-family="테스트 고딕">한글</text></svg>',
    css,
  );
  assert.match(svg, /https:\/\/cdn\.example\.test\/demo\.woff2/);
  assert.doesNotMatch(svg, /local\("없는 글꼴"\)/);
  assert.match(svg, /__rhwp_visual_sweep_noto_sans_kr__/);
  assert.deepEqual(svgViewport(svg), { width: 600, height: 120 });
});
