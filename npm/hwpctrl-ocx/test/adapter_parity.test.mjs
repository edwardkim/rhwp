/**
 * 모드 동등성 — standalone 과 plugin 이 **같은 문서를 만든다**.
 *
 * 이 검사가 있는 이유: 결합 모드를 만들면서 `index.mjs` 를 건드리면 단독 동작이 조용히 갈릴 수
 * 있다. 같은 호출 순서를 두 모드로 돌려 산출 바이트를 비교하면, 어댑터가 조작의 **의미**를
 * 바꿨는지 한 번에 드러난다.
 *
 * plugin 모드의 호스트는 여기서 최소 구현으로 세운다 — studio 를 끌어오지 않는다는 계약을
 * 이 파일 자체가 증명한다(패키지는 studio 없이 테스트된다).
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { createHwpCtrl } from '../src/index.mjs';
import { hwpctrlStudioPlugin } from '../src/studio-plugin.mjs';
import { isMutating, READ_ONLY_METHODS } from '../src/adapter.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '..', '..', '..');
const wasmPath = path.join(repoRoot, 'pkg', 'rhwp_bg.wasm');
const hasWasm = fs.existsSync(wasmPath);

async function loadWasm() {
  const mod = await import(path.join(repoRoot, 'pkg', 'rhwp.js'));
  await mod.default({ module_or_path: fs.readFileSync(wasmPath) });
  return mod;
}

const sampleBytes = () =>
  new Uint8Array(fs.readFileSync(path.join(repoRoot, 'samples', 'table-001.hwp')));

/**
 * 최소 호스트 — studio 의 계약만 흉내 낸다.
 *
 * 트랜잭션은 "감싸고 세는" 것으로 충분하다. 여기서 검증하려는 것은 undo 기구가 아니라
 * **어댑터가 문서에 하는 일이 두 모드에서 같은가** 이다.
 */
function fakeHost(wasm, doc) {
  const stats = { transactions: 0, reads: 0, deferred: 0, labels: [] };
  let current = doc;
  const swapListeners = [];
  const host = {
    stats,
    borrowDocument: () => ({ handle: current, generation: 1 }),
    currentGeneration: () => 1,
    read(fn) { stats.reads += 1; return fn(current); },
    transaction(label, fn) {
      stats.transactions += 1;
      stats.labels.push(label);
      return fn({
        doc: () => current,
        deferPagination: () => { stats.deferred += 1; },
      });
    },
    async loadDocument(bytes) { current = new wasm.HwpDocument(bytes); },
    createBlankDocument() {
      current = wasm.HwpDocument.createEmpty();
      try { current.createBlankDocument(); } catch { /* 최소 문서라도 남긴다 */ }
    },
    automation: { execute: () => ({ ok: true }) },
    events: { on: () => () => {} },
    onDocumentSwap(cb) { swapListeners.push(cb); return () => {}; },
  };
  return host;
}

/** 두 모드에 똑같이 먹일 호출 순서. 읽기와 쓰기를 섞는다. */
const SCENARIO = [
  { m: 'MovePos', a: [2, 0, 0] },
  { m: 'GetPos', a: [] },
  { m: 'PageCount', a: [] },
  { m: 'MovePos', a: [3, 0, 0] },
];

test('두 모드가 같은 문서 바이트를 만든다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const wasm = await loadWasm();

  // standalone — 이 층이 문서를 소유한다
  const soloDoc = new wasm.HwpDocument(sampleBytes());
  soloDoc.convertToEditable?.();
  const solo = createHwpCtrl({ wasm, doc: soloDoc });
  const soloResults = SCENARIO.map(({ m, a }) => solo[m](...a));
  const soloBytes = solo.getWasmDoc().exportHwp();

  // plugin — studio 가 소유하고 빌려준다
  const hostedDoc = new wasm.HwpDocument(sampleBytes());
  hostedDoc.convertToEditable?.();
  const host = fakeHost(wasm, hostedDoc);
  const surface = hwpctrlStudioPlugin.activate(host);
  const pluginResults = SCENARIO.map(({ m, a }) => surface.invoke(m, a));
  const pluginBytes = surface.exportBytes();

  assert.deepEqual(pluginResults, soloResults, '같은 호출은 같은 값을 돌려준다');
  assert.equal(pluginBytes.length, soloBytes.length, '산출 바이트 길이가 같다');
  assert.deepEqual(Buffer.from(pluginBytes), Buffer.from(soloBytes), '산출 바이트가 같다');
});

test('배치는 트랜잭션 한 번으로 묶인다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const wasm = await loadWasm();
  const doc = new wasm.HwpDocument(sampleBytes());
  doc.convertToEditable?.();
  const host = fakeHost(wasm, doc);
  const surface = hwpctrlStudioPlugin.activate(host);

  const before = host.stats.transactions;
  surface.batch([
    { m: 'MovePos', a: [2, 0, 0] },
    { m: 'MovePos', a: [3, 0, 0] },
    { m: 'MovePos', a: [4, 0, 0] },
  ]);
  assert.equal(host.stats.transactions - before, 1, '3개 호출이 트랜잭션 1건');
  assert.ok(host.stats.deferred >= 1, '조판을 미룬다');
});

test('읽기만 모인 배치는 트랜잭션을 열지 않는다', { skip: !hasWasm && 'pkg WASM 없음' }, async () => {
  const wasm = await loadWasm();
  const doc = new wasm.HwpDocument(sampleBytes());
  doc.convertToEditable?.();
  const host = fakeHost(wasm, doc);
  const surface = hwpctrlStudioPlugin.activate(host);

  const before = host.stats.transactions;
  surface.batch([{ m: 'GetPos', a: [] }, { m: 'PageCount', a: [] }]);
  assert.equal(host.stats.transactions - before, 0, '읽기 배치는 히스토리를 건드리지 않는다');
});

test('모르는 이름은 바꾼다고 본다 (보수적 기본값)', () => {
  assert.equal(isMutating('전혀모르는이름'), true);
  assert.equal(isMutating('GetPos'), false);
  assert.ok(READ_ONLY_METHODS.has('PageCount'));
});

test('Open 은 호스트에 위임한다 — 문서를 따로 만들지 않는다', {
  skip: !hasWasm && 'pkg WASM 없음',
}, async () => {
  const wasm = await loadWasm();
  const doc = new wasm.HwpDocument(sampleBytes());
  doc.convertToEditable?.();
  const host = fakeHost(wasm, doc);
  let delegated = false;
  const origLoad = host.loadDocument.bind(host);
  host.loadDocument = async (bytes) => { delegated = true; await origLoad(bytes); };

  const surface = hwpctrlStudioPlugin.activate(host);
  await new Promise((resolve, reject) => {
    surface.invoke('Open', [sampleBytes(), '', '', (ok) => (ok ? resolve() : reject(new Error('Open 실패')))]);
  });
  assert.ok(delegated, 'host.loadDocument 를 태워야 한다');
});
