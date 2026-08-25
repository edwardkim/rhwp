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
  assert.match(value, /font-variant-numeric:\s*tabular-nums/);
});

test('통합 배율 버튼의 돋보기는 좌우 확대 아이콘과 같은 18px 상자를 사용한다', () => {
  const css = readFileSync(
    new URL('../src/styles/status-bar.css', import.meta.url),
    'utf8',
  );
  const icon = css.match(/\.icon-zoom-menu\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const ring = css.match(/\.icon-zoom-menu::before\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;
  const handle = css.match(/\.icon-zoom-menu::after\s*\{(?<rules>[^}]*)\}/)?.groups?.rules;

  assert.ok(icon);
  assert.ok(ring);
  assert.ok(handle);
  assert.match(icon, /width:\s*18px/);
  assert.match(icon, /height:\s*18px/);
  assert.match(icon, /border:\s*0/);
  assert.match(icon, /margin:\s*0/);
  assert.match(icon, /flex-shrink:\s*0/);
  assert.match(ring, /width:\s*12px/);
  assert.match(ring, /height:\s*12px/);
  assert.match(handle, /width:\s*7px/);
});
