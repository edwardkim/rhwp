import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
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

test('status bar and view command share the fit helpers', () => {
  const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  const commands = readFileSync(
    new URL('../src/command/commands/view.ts', import.meta.url),
    'utf8',
  );

  assert.match(main, /calculateFitPageZoom/);
  assert.match(commands, /calculateFitPageZoom/);
  assert.doesNotMatch(main, /containerHeight - 40/);
  assert.doesNotMatch(commands, /containerH - 40/);
});
