import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const wasmApi = readFileSync(new URL('../../src/wasm_api.rs', import.meta.url), 'utf8');
const bridge = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');

const methods = [
  'getSelectionRectsInHeaderFooter',
  'replaceRangeInHeaderFooter',
  'copySelectionInHeaderFooter',
  'getCharPropertiesInHeaderFooter',
  'applyCharFormatInHeaderFooter',
  'getHeaderFooterPreviewPage',
  'hitTestInHeaderFooterTarget',
  'renderHeaderFooterEditPreviewToCanvas',
];

for (const method of methods) {
  test(`#4121 ${method}가 WASM과 Studio bridge에 함께 노출된다`, () => {
    assert.match(wasmApi, new RegExp(`js_name\\s*=\\s*${method}`));
    assert.match(bridge, new RegExp(`\\n\\s*${method}\\(`));
    assert.match(bridge, new RegExp(`\\.${method}\\(`));
  });
}

test('#4121 HF hit-test가 클릭 페이지의 resolved target을 전달한다', () => {
  assert.match(
    bridge,
    /hitTestInHeaderFooter[\s\S]*?sectionIndex\?: number; applyTo\?: number;/,
  );
  assert.match(wasmApi, /hitTestInHeaderFooter/);
});
