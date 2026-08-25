import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  calculateArrangementFitWidthZoom,
  calculateFitPageZoom,
  calculateFitWidthZoom,
} from '../src/view/zoom-fit.ts';

test('fit page uses the real ten-pixel top and bottom gaps', () => {
  const zoom = calculateFitPageZoom(883, 683, 793.8, 1122.5);
  assert.ok(Math.abs(zoom - (663 / 1122.5)) < 1e-12);
});

test('fit width keeps twenty-pixel side gutters', () => {
  assert.ok(
    Math.abs(calculateFitWidthZoom(883, 793.8) - (843 / 793.8)) < 1e-12,
  );
});

test('fit width uses the visible page row width for fixed arrangements', () => {
  const metrics = { containerWidth: 883, pageWidth: 800, pageGap: 10 };

  assert.equal(
    calculateArrangementFitWidthZoom({
      ...metrics,
      arrangement: { kind: 'auto' },
    }),
    843 / 800,
  );
  assert.equal(
    calculateArrangementFitWidthZoom({
      ...metrics,
      arrangement: { kind: 'double' },
    }),
    833 / 1600,
  );
  assert.equal(
    calculateArrangementFitWidthZoom({
      ...metrics,
      arrangement: { kind: 'facing' },
    }),
    833 / 1600,
  );
  assert.equal(
    calculateArrangementFitWidthZoom({
      ...metrics,
      arrangement: { kind: 'multiple', columns: 3, rows: 2 },
    }),
    823 / 2400,
  );
});

test('status bar and view command share the fit helpers', () => {
  const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  const commands = readFileSync(
    new URL('../src/command/commands/view.ts', import.meta.url),
    'utf8',
  );

  assert.match(main, /calculateFitPageZoom/);
  assert.match(commands, /calculateFitPageZoom/);
  assert.match(main, /calculateArrangementFitWidthZoom/);
  assert.match(commands, /calculateArrangementFitWidthZoom/);
  assert.doesNotMatch(main, /containerHeight - 40/);
  assert.doesNotMatch(commands, /containerH - 40/);
});

test('통합 배율 버튼은 모든 배율에서 고정 폭과 tabular 숫자를 유지한다', () => {
  const css = readFileSync(
    new URL('../src/styles/status-bar.css', import.meta.url),
    'utf8',
  );
  const button = css.match(/\.stb-zoom-display\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const value = css.match(/#sb-zoom-val\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;

  assert.ok(button);
  assert.ok(value);
  assert.match(button, /width:\s*68px/);
  assert.match(button, /min-width:\s*68px/);
  assert.match(button, /box-sizing:\s*border-box/);
  assert.match(button, /justify-content:\s*flex-start/);
  assert.match(button, /gap:\s*2px/);
  assert.match(value, /width:\s*36px/);
  assert.match(value, /font-variant-numeric:\s*tabular-nums/);
  assert.match(value, /text-align:\s*left/);
});

test('통합 배율 버튼은 좌우 확대 아이콘과 같은 18px SVG 돋보기를 사용한다', () => {
  const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
  const css = readFileSync(
    new URL('../src/styles/status-bar.css', import.meta.url),
    'utf8',
  );
  const icon = css.match(/\.stb-zoom-menu-icon\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;

  assert.ok(icon);
  assert.match(
    html,
    /<svg class="stb-zoom-menu-icon" viewBox="0 0 18 18"[^>]*>[\s\S]*?<circle cx="6\.5" cy="6\.5" r="4\.5">[\s\S]*?<path d="M9\.75 9\.75 15\.25 15\.25">[\s\S]*?<\/svg>/,
  );
  assert.doesNotMatch(html, /stb-status-glyph icon-zoom-menu/);
  assert.match(icon, /width:\s*18px/);
  assert.match(icon, /height:\s*18px/);
  assert.match(icon, /flex:\s*0 0 18px/);
  assert.match(icon, /stroke:\s*currentColor/);
  assert.match(icon, /stroke-width:\s*1\.5/);
  assert.match(icon, /stroke-linecap:\s*round/);
  assert.doesNotMatch(css, /\.icon-zoom-menu::(?:before|after)/);
});

test('배율 슬라이더 손잡이와 100% 눈금은 같은 12px 크기이며 눈금이 뒤에 놓인다', () => {
  const css = readFileSync(
    new URL('../src/styles/status-bar.css', import.meta.url),
    'utf8',
  );
  const wrap = css.match(/\.stb-zoom-range-wrap\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const range = css.match(/\.stb-zoom-range\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const mark = css.match(/\.stb-zoom-neutral-mark\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const webkitThumb = css.match(
    /\.stb-zoom-range::-webkit-slider-thumb\s*\{(?<rules>[^}]*)\}/,
  )?.groups?.rules;
  const firefoxThumb = css.match(
    /\.stb-zoom-range::-moz-range-thumb\s*\{(?<rules>[^}]*)\}/,
  )?.groups?.rules;
  const webkitTrack = css.match(
    /\.stb-zoom-range::-webkit-slider-runnable-track\s*\{(?<rules>[^}]*)\}/,
  )?.groups?.rules;
  const firefoxTrack = css.match(
    /\.stb-zoom-range::-moz-range-track\s*\{(?<rules>[^}]*)\}/,
  )?.groups?.rules;

  assert.ok(wrap);
  assert.ok(range);
  assert.ok(mark);
  assert.ok(webkitThumb);
  assert.ok(firefoxThumb);
  assert.ok(webkitTrack);
  assert.ok(firefoxTrack);
  assert.match(wrap, /--stb-zoom-thumb-size:\s*12px/);
  assert.match(wrap, /--stb-zoom-track-size:\s*2px/);
  assert.match(range, /z-index:\s*2/);
  assert.match(range, /appearance:\s*none/);
  assert.match(mark, /z-index:\s*1/);
  assert.match(mark, /height:\s*var\(--stb-zoom-thumb-size\)/);
  assert.match(webkitThumb, /width:\s*var\(--stb-zoom-thumb-size\)/);
  assert.match(webkitThumb, /height:\s*var\(--stb-zoom-thumb-size\)/);
  assert.match(firefoxThumb, /width:\s*var\(--stb-zoom-thumb-size\)/);
  assert.match(firefoxThumb, /height:\s*var\(--stb-zoom-thumb-size\)/);
  assert.match(webkitTrack, /height:\s*var\(--stb-zoom-track-size\)/);
  assert.match(webkitTrack, /background:\s*var\(--ui-border-strong\)/);
  assert.match(firefoxTrack, /height:\s*var\(--stb-zoom-track-size\)/);
  assert.match(firefoxTrack, /background:\s*var\(--ui-border-strong\)/);
  assert.doesNotMatch(css, /\.stb-zoom-range-wrap\.is-neutral[\s\S]*?visibility:\s*hidden/);
});
