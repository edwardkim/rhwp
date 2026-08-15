import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const helperSource = await readFile(new URL('../e2e/helpers.mjs', import.meta.url), 'utf8');

test('renderer baseline은 문서 웹폰트 준비 뒤에 Canvas를 렌더링한다', () => {
  const start = helperSource.indexOf('export async function loadHwpFile');
  const end = helperSource.indexOf('\nexport ', start + 1);
  const loadHelper = helperSource.slice(start, end === -1 ? undefined : end);
  const fontLoader = loadHelper.indexOf("await import('/src/core/font-loader.ts')");
  const loadFonts = loadHelper.indexOf('await loadWebFonts(docInfo.fontsUsed ?? [])');
  const fontsReady = loadHelper.indexOf('await document.fonts.ready');
  const canvasLoad = loadHelper.indexOf('await window.__canvasView?.loadDocument?.()');

  assert.ok(fontLoader >= 0, 'baseline helper must import the production webfont loader');
  assert.ok(loadFonts > fontLoader, 'baseline helper must load document font families');
  assert.ok(fontsReady > loadFonts, 'baseline helper must wait for FontFace completion');
  assert.ok(canvasLoad > fontsReady, 'Canvas rendering must begin after document fonts are ready');
});
