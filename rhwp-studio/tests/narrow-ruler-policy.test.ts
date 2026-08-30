import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const responsive = readFileSync(new URL('../src/styles/responsive.css', import.meta.url), 'utf8');
const editor = readFileSync(new URL('../src/styles/editor.css', import.meta.url), 'utf8');

test('narrow desktop keeps rulers; only phone hides them (#6187)', () => {
  assert.match(editor, /#editor-area\s*\{[^}]*display:\s*grid;/s);
  assert.match(
    responsive,
    /@media\s*\(max-width:\s*767px\)\s*\{[\s\S]*?#h-ruler[\s\S]*?display:\s*none/,
  );
  assert.doesNotMatch(
    responsive,
    /@media\s*\(max-width:\s*1023px\)\s*\{[\s\S]*?#h-ruler[\s\S]*?display:\s*none/,
  );
  assert.doesNotMatch(
    responsive,
    /@media\s*\(max-width:\s*1023px\)\s*\{[\s\S]*?#editor-area\s*\{[\s\S]*?display:\s*flex/,
  );
});
