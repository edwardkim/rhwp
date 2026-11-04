import * as wasmExports from '@wasm/rhwp.js';
import type { FontSymbolInspector } from './font-symbol-coverage.ts';

/** 실제 WASM 초기화 뒤 호출한다. 검사 알고리즘을 JS에 복제하지 않는다. */
export const inspectFontSymbols: FontSymbolInspector = (bytes, faceIndex) => {
  const api = wasmExports as unknown as { fontSymbolUnicodeRange: FontSymbolInspector };
  return api.fontSymbolUnicodeRange(bytes, faceIndex);
};
