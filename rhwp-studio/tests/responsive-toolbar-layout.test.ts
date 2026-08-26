import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const toolbar = readFileSync(new URL('../src/styles/toolbar.css', import.meta.url), 'utf8');
const styleBar = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');

test('icon toolbar wraps complete groups and grows to its row count', () => {
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*flex-wrap:\s*wrap;/s);
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*height:\s*auto;/s);
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*min-height:\s*56px;/s);
  assert.match(toolbar, /\.tb-group\s*\{[^}]*flex-shrink:\s*0;/s);
});

test('constrained layouts hide top-level separators and do not scroll the toolbar', () => {
  assert.match(
    responsive,
    /@media\s*\(max-width:\s*1279px\)[\s\S]*?#icon-toolbar\s*>\s*\.tb-sep\s*\{[^}]*display:\s*none;/,
  );
  assert.match(
    responsive,
    /@media\s*\(max-width:\s*1023px\)[\s\S]*?#icon-toolbar\s*\{[^}]*min-height:\s*40px;/,
  );

  const mobileToolbar = responsive.match(
    /@media\s*\(max-width:\s*767px\)[\s\S]*?#icon-toolbar\s*\{([^}]*)\}/,
  );
  assert.ok(mobileToolbar);
  assert.doesNotMatch(mobileToolbar[1], /overflow-x:\s*auto/);
  assert.doesNotMatch(mobileToolbar[1], /-webkit-overflow-scrolling/);
});

test('style ribbon has only measured one-row and two-row structures', () => {
  assert.match(styleBar, /#style-bar\s*\{[^}]*display:\s*grid;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*height:\s*auto;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*min-height:\s*0;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*overflow:\s*visible;/s);
  assert.match(
    styleBar,
    /@media\s*\(min-width:\s*976px\)\s*\{[\s\S]*?#style-bar\s*\{[^}]*display:\s*flex;[^}]*flex-wrap:\s*nowrap;/,
  );
  assert.match(
    styleBar,
    /\.sb-command-track\s*\{[^}]*width:\s*max-content;[^}]*max-width:\s*100%;[^}]*flex-wrap:\s*nowrap;/s,
  );
  assert.doesNotMatch(styleBar, /#style-bar\s*\{[^}]*overflow-x:\s*auto;/s);
});

test('style ribbon boundaries are content-derived and paragraph overflow never adds a third row', () => {
  assert.match(
    styleBar,
    /@media\s*\(min-width:\s*976px\)/,
  );
  assert.match(
    styleBar,
    /@media\s*\(max-width:\s*459px\)\s*\{[\s\S]*?\.sb-overflow-host\s*\{[^}]*display:\s*flex;/,
  );
  assert.match(
    styleBar,
    /\.sb-overflow-host\.open \.sb-overflow-panel\s*\{[^}]*display:\s*block;/s,
  );
  assert.doesNotMatch(responsive, /@media\s*\(min-width:\s*768px\) and \(max-width:\s*1023px\)/);
  assert.doesNotMatch(responsive, /#style-bar\s*\{[^}]*(?:flex-direction|grid-template-columns)/s);
});
