import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
const styles = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');

const buttonMarkup = (id: string): string => {
  const match = html.match(new RegExp(`<button[^>]*id="${id}"[\\s\\S]*?<\\/button>`));
  assert.ok(match, `missing #${id}`);
  return match[0];
};

test('style toolbar uses ordered field and command groups', () => {
  const fields = html.indexOf('class="sb-field-grid"');
  const commandTrack = html.indexOf('class="sb-command-track"');
  const characters = html.indexOf('class="sb-command-band sb-character-band"');
  const paragraphs = html.indexOf('class="sb-command-band sb-paragraph-band"');

  assert.ok(fields >= 0);
  assert.ok(fields < commandTrack);
  assert.ok(commandTrack < characters);
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
    /<select id="style-name"[^>]*>\s*<option value="0"[^>]*>바탕글<\/option>\s*<\/select>/,
  );
});

test('formatting surface preserves ribbon hierarchy while captions stay visually compact', () => {
  assert.match(html, /class="sb-ribbon-group sb-field-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-character-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-color-ribbon-group"/);
  assert.match(html, /class="sb-ribbon-group sb-paragraph-ribbon-group"/);

  for (const label of ['글꼴 및 간격', '글자 모양', '색', '문단']) {
    assert.match(html, new RegExp(`<span class="sb-ribbon-label"[^>]*>${label}<\\/span>`));
  }

  assert.match(styles, /#style-bar\s*\{[^}]*display:\s*grid;[^}]*min-height:\s*0;/s);
  assert.match(styles, /\.sb-ribbon-group\s*\{[^}]*flex-direction:\s*column;/s);
  assert.match(styles, /\.sb-ribbon-label\s*\{[^}]*display:\s*none;/s);
  assert.match(
    styles,
    /\.sb-field-ribbon-group \.sb-field\s*\{[^}]*flex-direction:\s*column;/s,
  );
});

test('paragraph commands keep one DOM authority across inline and overflow layouts', () => {
  assert.match(html, /class="sb-overflow-host"/);
  assert.match(
    buttonMarkup('btn-style-overflow'),
    /aria-controls="style-overflow-panel"[^>]*aria-expanded="false"/,
  );
  assert.match(buttonMarkup('btn-style-overflow'), /id="style-overflow-current-icon"/);
  assert.match(buttonMarkup('btn-style-overflow'), /class="sb-align sb-al-left sb-overflow-current-icon"/);
  assert.match(buttonMarkup('btn-style-overflow'), /class="sb-dd"/);
  assert.doesNotMatch(buttonMarkup('btn-style-overflow'), /⋯/);
  assert.match(
    html,
    /id="style-overflow-panel"[^>]*role="group"[^>]*aria-label="문단 정렬"/,
  );
  for (const id of [
    'btn-align-left',
    'btn-align-center',
    'btn-align-right',
    'btn-align-justify',
    'btn-align-distribute',
    'btn-align-split',
  ]) {
    assert.equal(html.match(new RegExp(`id="${id}"`, 'g'))?.length, 1, `${id} must be unique`);
  }
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

test('two-row ribbon keeps the fixed 136px font field and 460px command contract', () => {
  assert.match(
    styles,
    /\.sb-field-grid\s*\{[^}]*minmax\(68px,\s*88px\)[^}]*minmax\(54px,\s*64px\)[^}]*136px[^}]*minmax\(72px,\s*86px\)[^}]*minmax\(72px,\s*86px\);/s,
  );
  assert.match(styles, /\.sb-font\s*\{[^}]*width:\s*136px;[^}]*text-overflow:\s*ellipsis;/s);
  assert.match(styles, /\.sb-command-track\s*\{[^}]*width:\s*max-content;[^}]*flex-wrap:\s*nowrap;/s);
  assert.match(styles, /\.sb-btn\s*\{[^}]*width:\s*29px;[^}]*height:\s*29px;/s);
  assert.match(styles, /\.sb-has-arrow\s*\{[^}]*width:\s*38px;/s);
  assert.match(styles, /@media\s*\(max-width:\s*459px\)/);
  assert.match(
    styles,
    /@media\s*\(max-width:\s*459px\)\s*\{\s*\.sb-field-grid\s*\{[^}]*minmax\(54px,\s*1fr\)[^}]*minmax\(40px,\s*0\.75fr\)[^}]*136px[^}]*minmax\(54px,\s*0\.95fr\)[^}]*minmax\(54px,\s*0\.95fr\);/s,
  );
  assert.doesNotMatch(styles, /\.sb-ga\s*\{\s*display:\s*none;/);
});

test('mobile font size field uses one cohesive control shell', () => {
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size-group\s*\{[^}]*border:\s*1px solid var\(--ui-border-light\);[^}]*border-radius:\s*var\(--radius-sm\);[^}]*overflow:\s*hidden;/s,
  );
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size\s*\{[^}]*height:\s*100%;[^}]*border:\s*0;[^}]*border-radius:\s*0;/s,
  );
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size-unit\s*\{[^}]*height:\s*100%;[^}]*border:\s*0;[^}]*border-left:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size-arrows\s*\{[^}]*height:\s*100%;[^}]*border-left:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size-arrows \.sb-arrow\s*\{[^}]*height:\s*50%;[^}]*border:\s*0;[^}]*border-radius:\s*0;/s,
  );
  assert.match(
    styles,
    /\.sb-field-grid \.sb-size-arrows \.sb-arrow \+ \.sb-arrow\s*\{[^}]*border-top:\s*1px solid var\(--ui-border-light\);/s,
  );
  assert.match(
    styles,
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
