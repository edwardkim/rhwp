import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));

const JS_NAME = /^\s*#\[wasm_bindgen\(js_name\s*=\s*([A-Za-z0-9_]+)\)\]/;

/**
 * `js_name` export 이름을, 기본 feature 로 나오는 것과 feature 뒤에 있는 것으로 가른다.
 *
 * [#4580] `pkg/rhwp.d.ts` 는 기본 feature 로 만든 산출물이므로 feature 뒤의 export 는 애초에
 * 거기 없다. 그것까지 요구하면 이 게이트가 거짓으로 빨개진다 — 실제로 `subsecond-dev` 의 두
 * export 가 `#[cfg_attr(feature = …, wasm_bindgen(…))]` 라는 항상 참인 형태였다가 평범한
 * `#[wasm_bindgen]` 으로 바뀌자 이 정규식에 걸려 그렇게 됐다. 종전에는 그 형태가 우연히
 * 정규식을 비켜 가고 있었을 뿐이다.
 */
function partitionExportNames(source) {
  const lines = source.split('\n');
  const always = new Set();
  const featureGated = new Set();

  lines.forEach((line, index) => {
    const match = JS_NAME.exec(line);
    if (!match) return;
    // 바로 위에 붙어 있는 특성 줄들만 본다. `#[cfg(… feature = "x" …)]` 가 있으면 이 export 는
    // 그 feature 로 빌드했을 때만 존재한다.
    let gated = false;
    for (let above = index - 1; above >= 0; above -= 1) {
      const text = lines[above].trim();
      if (!text.startsWith('#[')) break;
      if (text.includes('feature = ')) gated = true;
    }
    (gated ? featureGated : always).add(match[1]);
  });

  return { always, featureGated };
}

test('generated WASM declarations contain every explicit js_name export', () => {
  const source = readFileSync(path.join(ROOT, 'src/wasm_api.rs'), 'utf8');
  const declarations = readFileSync(path.join(ROOT, 'pkg/rhwp.d.ts'), 'utf8');
  const { always, featureGated } = partitionExportNames(source);

  // 정규식이 아무것도 못 잡으면 아래 단언이 공짜로 통과한다.
  assert.ok(always.size > 0, 'src/wasm_api.rs 에서 js_name export 를 한 건도 읽지 못했다');

  const missing = [...always]
    .filter((name) => !new RegExp(`\\b${name}\\b`).test(declarations))
    .sort();

  assert.deepEqual(
    missing,
    [],
    `pkg/rhwp.d.ts is stale; rebuild WASM before frontend gates: ${missing.join(', ')}`,
  );

  const leaked = [...featureGated]
    .filter((name) => new RegExp(`\\b${name}\\b`).test(declarations))
    .sort();

  assert.deepEqual(
    leaked,
    [],
    `feature 뒤에 있어야 할 export 가 기본 빌드 산출물에 나왔다 — pkg 를 feature 를 켜고 만들었는가: ${leaked.join(', ')}`,
  );
});
