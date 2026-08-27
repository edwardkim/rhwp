import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  initStyleToolbarOverflow,
  shouldReturnStyleToolbarFocus,
  STYLE_TOOLBAR_COMMAND_INLINE_MIN,
  STYLE_TOOLBAR_FULL_ROW_MIN,
  STYLE_TOOLBAR_ONE_ROW_MIN,
  STYLE_TOOLBAR_OVERFLOW_QUERY,
} from '../src/ui/style-toolbar-overflow.ts';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');

test('style toolbar breakpoints share the measured Stage 1 constants', () => {
  assert.equal(STYLE_TOOLBAR_FULL_ROW_MIN, 962);
  assert.equal(STYLE_TOOLBAR_ONE_ROW_MIN, 808);
  assert.equal(STYLE_TOOLBAR_COMMAND_INLINE_MIN, 460);
  assert.equal(STYLE_TOOLBAR_OVERFLOW_QUERY, '(max-width: 459px), (min-width: 808px) and (max-width: 961px)');
});

test('overflow controller keeps the existing paragraph button authority', () => {
  const panelStart = html.indexOf('id="style-overflow-panel"');
  assert.ok(panelStart >= 0);

  for (const id of [
    'btn-align-left',
    'btn-align-center',
    'btn-align-right',
    'btn-align-justify',
    'btn-align-distribute',
    'btn-align-split',
  ]) {
    assert.equal(html.match(new RegExp(`id="${id}"`, 'g'))?.length, 1);
    assert.ok(html.indexOf(`id="${id}"`) > panelStart);
  }
});

test('pointer commands preserve editor focus and keyboard commands return to the trigger', () => {
  assert.equal(shouldReturnStyleToolbarFocus(true, false), true);
  assert.equal(shouldReturnStyleToolbarFocus(true, true), false);
  assert.equal(shouldReturnStyleToolbarFocus(false, false), false);
  assert.equal(shouldReturnStyleToolbarFocus(false, true), false);
});

test('optional chrome initialization is a no-op when the container is absent', () => {
  assert.equal(initStyleToolbarOverflow(null), null);
});
