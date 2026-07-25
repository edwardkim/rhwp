import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const styles = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

const buttonMarkup = (id: string): string => {
  const match = html.match(new RegExp(`<button[^>]*id="${id}"[\\s\\S]*?<\\/button>`));
  assert.ok(match, `missing #${id}`);
  return match[0];
};

test('style toolbar uses ordered field and command groups', () => {
  const fields = html.indexOf('class="sb-field-grid"');
  const characters = html.indexOf('class="sb-command-band sb-character-band"');
  const paragraphs = html.indexOf('class="sb-command-band sb-paragraph-band"');

  assert.ok(fields >= 0);
  assert.ok(fields < characters);
  assert.ok(characters < paragraphs);
  assert.match(html, /class="sb-command-group sb-character-group"/);
  assert.match(html, /class="sb-command-group sb-color-group"/);
  assert.match(html, /class="sb-command-group sb-align-group"/);

  const fieldGrid = html.slice(fields, characters);
  for (const id of ['style-name', 'font-lang', 'font-name', 'font-size', 'linespacing-select']) {
    assert.match(fieldGrid, new RegExp(`id="${id}"`));
  }
});

test('style toolbar shows its default before a document is loaded', () => {
  assert.match(
    html,
    /<select id="style-name"[^>]*>\s*<option value="0">바탕글<\/option>\s*<\/select>/,
  );
});

test('desktop formatting surface exposes visible ribbon hierarchy', () => {
  assert.match(html, /class="sb-ribbon-group sb-field-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-character-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-color-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-paragraph-ribbon-group"/);

  for (const label of ['글꼴 및 간격', '글자 모양', '색', '문단']) {
    assert.match(html, new RegExp(`<span class="sb-ribbon-label">${label}<\\/span>`));
  }

  assert.match(styles, /#style-bar\s*\{[^}]*min-height:\s*68px;[^}]*align-items:\s*stretch;/s);
  assert.match(styles, /\.sb-ribbon-group\s*\{[^}]*flex-direction:\s*column;/s);
  assert.match(styles, /\.sb-ribbon-label\s*\{[^}]*display:\s*block;/s);
  assert.match(
    styles,
    /\.sb-field-ribbon-group \.sb-field\s*\{[^}]*flex-direction:\s*column;/s,
  );
});

test('only real menus retain dropdown affordances', () => {
  const strike = buttonMarkup('btn-strike');
  assert.doesNotMatch(strike, /sb-has-arrow|sb-dd/);
  assert.match(strike, /sb-strike/);

  for (const id of ['btn-charfx', 'btn-text-color', 'btn-highlight']) {
    const button = buttonMarkup(id);
    assert.match(button, /sb-has-arrow/);
    assert.match(button, /sb-dd/);
  }
  assert.match(buttonMarkup('btn-charfx'), /sb-effect-icon/);
  assert.match(buttonMarkup('btn-text-color'), /sb-color-visual/);
  assert.match(buttonMarkup('btn-highlight'), /sb-highlight-visual/);
});

test('mobile ribbon is compact without hiding command glyphs', () => {
  assert.match(responsive, /#style-bar\s*\{[^}]*flex-direction:\s*column;/s);
  assert.match(
    responsive,
    /\.sb-field-grid\s*\{[^}]*grid-template-columns:\s*68px 54px minmax\(96px,\s*1fr\) 72px 72px;/s,
  );
  assert.match(responsive, /#style-bar \.sb-btn\s*\{[^}]*width:\s*29px;[^}]*height:\s*29px;/s);
  assert.match(responsive, /#style-bar \.sb-has-arrow\s*\{[^}]*width:\s*38px;/s);
  assert.doesNotMatch(responsive, /\.sb-ga\s*\{\s*display:\s*none;/);
});

test('mobile font size field uses one cohesive control shell', () => {
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size-group\s*\{[^}]*border:\s*1px solid var\(--ui-border-light\);[^}]*border-radius:\s*var\(--radius-sm\);[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size\s*\{[^}]*height:\s*100%;[^}]*border:\s*0;[^}]*border-radius:\s*0;/s,
  );
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size-unit\s*\{[^}]*height:\s*100%;[^}]*border:\s*0;[^}]*border-left:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size-arrows\s*\{[^}]*height:\s*100%;[^}]*border-left:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size-arrows \.sb-arrow\s*\{[^}]*height:\s*50%;[^}]*border:\s*0;[^}]*border-radius:\s*0;/s,
  );
  assert.match(
    responsive,
    /\.sb-field-grid \.sb-size-arrows \.sb-arrow \+ \.sb-arrow\s*\{[^}]*border-top:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    responsive,
    /#style-bar #btn-size-up,\s*#style-bar #btn-size-down\s*\{[^}]*border-radius:\s*0;/s,
  );
});

test('font size unit shares the input surface instead of the spinner surface', () => {
  assert.match(
    styles,
    /\.sb-size-unit\s*\{[^}]*background:\s*var\(--color-surface\);/s,
  );
  assert.doesNotMatch(
    styles,
    /\.sb-size-unit\s*\{[^}]*background:\s*var\(--ui-surface-muted\);/s,
  );
});

test('alignment icons use the shared theme-aware mask contract', () => {
  assert.match(styles, /\.sb-align\s*\{[^}]*background-color:\s*currentColor;[^}]*mask/s);
  for (const name of ['left', 'center', 'right', 'justify', 'distribute', 'split']) {
    assert.match(styles, new RegExp(`\\.sb-al-${name}\\s*\\{[^}]*--sb-align-icon:`));
  }
  assert.doesNotMatch(
    styles,
    /\.sb-al-(?:left|center|right|justify|distribute|split)\s*\{[^}]*background-image:/s,
  );
});
