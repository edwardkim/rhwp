/**
 * 양방향 독립 가드 — 이 패키지는 studio 없이 산다.
 *
 * 두 가지를 잠근다.
 *  1. **소스 어디에서도 `rhwp-studio` 를 import 하지 않는다.** plugin 어댑터는 host 의 모양만 안다.
 *  2. **DOM 없는 Node 에서 문서를 다룬다.** 서버측 서식 채움·배치 변환이 이 성질에 기대고 있다.
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const srcDir = path.join(here, '..', 'src');
const repoRoot = path.resolve(here, '..', '..', '..');
const wasmPath = path.join(repoRoot, 'pkg', 'rhwp_bg.wasm');
const hasWasm = fs.existsSync(wasmPath);

test('패키지 소스는 rhwp-studio 를 참조하지 않는다', () => {
  const offenders = [];
  for (const name of fs.readdirSync(srcDir)) {
    if (!name.endsWith('.mjs')) continue;
    const body = fs.readFileSync(path.join(srcDir, name), 'utf8');
    // 주석의 설명 문구는 허용하고, 실제 모듈 지정자만 본다.
    const specs = [...body.matchAll(/(?:import|from)\s+['"]([^'"]+)['"]/g)].map((m) => m[1]);
    for (const spec of specs) {
      if (spec.includes('rhwp-studio') || spec.startsWith('@/')) offenders.push(`${name}: ${spec}`);
    }
  }
  assert.deepEqual(offenders, [], `studio 참조 금지: ${offenders.join(', ')}`);
});

test('DOM 없이 문서를 열고 읽고 저장한다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  assert.equal(typeof globalThis.document, 'undefined', 'Node 에는 DOM 이 없어야 한다');

  const wasm = await import(path.join(repoRoot, 'pkg', 'rhwp.js'));
  await wasm.default({ module_or_path: fs.readFileSync(wasmPath) });
  const { createHwpCtrl } = await import('../src/index.mjs');

  const bytes = new Uint8Array(
    fs.readFileSync(path.join(repoRoot, 'samples', 'table-001.hwp')),
  );
  const ctrl = createHwpCtrl({ wasm });

  const opened = await new Promise((resolve) => {
    ctrl.Open(bytes, '', '', (ok) => resolve(ok));
  });
  assert.equal(opened, true, 'Node 에서 Open 이 성공해야 한다');
  assert.ok(ctrl.PageCount() >= 1, '쪽수를 읽는다');
  assert.ok(ctrl.getWasmDoc().exportHwp().length > 0, '바이트로 저장한다');
});

test('studio-plugin 진입점은 studio 없이도 import 된다', async () => {
  const mod = await import('../src/studio-plugin.mjs');
  assert.equal(mod.hwpctrlStudioPlugin.id, 'hwpctrl');
  assert.equal(mod.hwpctrlStudioPlugin.apiVersion, 1);
  assert.equal(typeof mod.hwpctrlStudioPlugin.activate, 'function');
});
