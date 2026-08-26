import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const toolbar = readFileSync(new URL('../src/styles/toolbar.css', import.meta.url), 'utf8');
const styleBar = readFileSync(new URL('../src/styles/style-bar.css', import.meta.url), 'utf8');
const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');
const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');

test('icon toolbar keeps one desktop-density row around a single command track', () => {
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*flex-wrap:\s*nowrap;/s);
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*height:\s*56px;/s);
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*min-height:\s*56px;/s);
  assert.match(toolbar, /#icon-toolbar\s*\{[^}]*overflow:\s*hidden;/s);
  assert.match(toolbar, /\.tb-scroll-track\s*\{[^}]*flex-wrap:\s*nowrap;/s);
  assert.match(toolbar, /\.tb-scroll-track\s*\{[^}]*width:\s*max-content;/s);
  assert.match(toolbar, /\.tb-group\s*\{[^}]*flex-shrink:\s*0;/s);
  assert.match(toolbar, /\.tb-btn\s*\{[^}]*min-width:\s*44px;/s);
});

test('icon toolbar reuses one DOM authority inside a native horizontal viewport', () => {
  assert.match(html, /id="icon-toolbar-prev"[^>]*aria-controls="icon-toolbar-viewport"[^>]*hidden disabled/);
  assert.match(html, /id="icon-toolbar-viewport"[^>]*class="tb-scroll-viewport"[^>]*tabindex="0"/);
  assert.match(html, /class="tb-scroll-track"/);
  assert.match(html, /id="icon-toolbar-next"[^>]*aria-controls="icon-toolbar-viewport"[^>]*hidden disabled/);
  assert.match(toolbar, /\.tb-scroll-viewport\s*\{[^}]*overflow-x:\s*auto;/s);
  assert.match(toolbar, /\.tb-scroll-viewport\s*\{[^}]*touch-action:\s*pan-x;/s);
  assert.match(toolbar, /\.tb-scroll-nav\[hidden\]\s*\{[^}]*display:\s*none;/s);
  assert.doesNotMatch(responsive, /#icon-toolbar\s*>\s*\.tb-sep/);
  assert.doesNotMatch(responsive, /\.tb-btn\s+\.tb-label\s*\{[^}]*display:\s*none;/s);
  assert.doesNotMatch(responsive, /\.tb-btn\s*\{[^}]*min-width:\s*36px;/s);
});

test('style ribbon has only measured one-row and two-row structures', () => {
  assert.match(styleBar, /#style-bar\s*\{[^}]*display:\s*grid;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*height:\s*auto;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*min-height:\s*0;/s);
  assert.match(styleBar, /#style-bar\s*\{[^}]*overflow:\s*visible;/s);
  assert.match(
    styleBar,
    /@media\s*\(min-width:\s*992px\)\s*\{[\s\S]*?#style-bar\s*\{[^}]*display:\s*flex;[^}]*flex-wrap:\s*nowrap;/,
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
    /@media\s*\(min-width:\s*992px\)/,
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

test('desktop style ribbon uses the menu text visual anchor without spending compact width', () => {
  assert.match(
    styleBar,
    /@media\s*\(min-width:\s*992px\)[\s\S]*?#style-bar\s*\{[^}]*padding:\s*var\(--style-bar-full-padding-top, 3px\) 8px 3px 22px;/,
  );
  assert.match(styleBar, /#style-bar\s*\{[^}]*padding:\s*4px 6px 5px;/s);
});
