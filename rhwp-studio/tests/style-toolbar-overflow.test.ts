import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const styles = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');
const source = readFileSync(new URL('../src/ui/style-toolbar-overflow.ts', import.meta.url), 'utf8');
const toolbar = readFileSync(new URL('../src/ui/toolbar.ts', import.meta.url), 'utf8');

test('style toolbar breakpoints share the measured Stage 1 constants', () => {
  assert.match(source, /STYLE_TOOLBAR_FULL_ROW_MIN = 992/);
  assert.match(source, /STYLE_TOOLBAR_COMMAND_INLINE_MIN = 460/);
  assert.match(source, /STYLE_TOOLBAR_COMMAND_INLINE_MIN - 1/);
  assert.match(styles, /@media \(min-width: 992px\)/);
  assert.match(styles, /@media \(max-width: 459px\)/);
});

test('overflow controller keeps the existing paragraph button authority', () => {
  const panelStart = html.indexOf('id="style-overflow-panel"');
  assert.ok(panelStart >= 0);
  const panelEnd = html.indexOf('</div>', html.indexOf('class="sb-overflow-host"'));
  assert.ok(panelEnd > panelStart);

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

test('overflow controller owns keyboard, outside-click, focus, and active-state cleanup', () => {
  assert.match(source, /event\.key !== 'ArrowDown'/);
  assert.match(source, /event\.key !== 'Escape'/);
  assert.match(source, /document\.addEventListener\('pointerdown'/);
  assert.match(source, /requestAnimationFrame\(\(\) => this\.paragraphButtons/);
  assert.match(source, /requestAnimationFrame\(\(\) => this\.trigger\.focus\(\)\)/);
  assert.match(source, /new MutationObserver/);
  assert.match(source, /button\.classList\.contains\('active'\)/);
  assert.match(source, /this\.triggerIcon\.classList\.remove\(\.\.\.ALIGNMENT_ICON_CLASSES\)/);
  assert.doesNotMatch(source, /this\.trigger\.classList\.toggle\('active'/);
  assert.match(source, /문단 정렬 더보기, 현재 \$\{currentAlignment\}/);
  assert.match(source, /dispose\(\): void/);
});

test('cursor paragraph alignment selects the source button mirrored by the overflow trigger', () => {
  assert.match(toolbar, /private alignButtons: Array<\{ button: HTMLButtonElement; alignment: string \}>/);
  assert.match(toolbar, /this\.alignButtons\.push\(\{ button: btn, alignment \}\)/);
  assert.match(
    toolbar,
    /for \(const \{ button, alignment \} of this\.alignButtons\) \{\s*this\.setActive\(button, props\.alignment === alignment\);/,
  );
});
