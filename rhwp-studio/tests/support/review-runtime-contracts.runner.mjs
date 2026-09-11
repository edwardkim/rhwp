// 실제 제품 모듈을 로드한다. WASM 저장소만 결정적 대역이며 제품 history/command는 복사하지 않는다.
import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const src = join(dirname(fileURLToPath(import.meta.url)), '../../src');
const [mode, mutation] = process.argv.slice(2);
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === '@wasm/rhwp.js') return {
      url: 'data:text/javascript,export default function init(){return {}};export class HwpDocument{};export function version(){return "test"}',
      shortCircuit: true,
    };
    if (specifier.startsWith('@/')) return {
      url: pathToFileURL(join(src, specifier.slice(2) + '.ts')).href, shortCircuit: true,
    };
    if (/^\.{1,2}\//.test(specifier) && !/\.[cm]?[tj]s$/.test(specifier)) return {
      url: pathToFileURL(join(dirname(fileURLToPath(context.parentURL)), specifier + '.ts')).href,
      shortCircuit: true,
    };
    return next(specifier, context);
  },
  load(url, context, next) {
    const result = next(url, context);
    let source = result.source;
    if (source == null) return result;
    source = typeof source === 'string' ? source : Buffer.from(source).toString();
    if (mutation === 'fixed-budget' && url.endsWith('/engine/history.ts')) {
      const needle = '(wasm.snapshotCapacity() ?? FALLBACK_MAX_SNAPSHOTS)';
      assert.ok(source.includes(needle), '음성 대조 주입 위치 누락');
      return { ...result, source: source.replace(needle, '100') };
    }
    if (mutation === 'drop-engine-chain' && url.endsWith('/core/wasm-bridge.ts')) {
      const needle = 'for (const name of incoming) push(name);';
      assert.ok(source.includes(needle), '음성 대조 주입 위치 누락');
      return { ...result, source: source.replace(needle, 'push(primary);') };
    }
    return result;
  },
});

const { WasmBridge } = await import(pathToFileURL(join(src, 'core/wasm-bridge.ts')));
if (mode.startsWith('history-')) {
  const { CommandHistory } = await import(pathToFileURL(join(src, 'engine/history.ts')));
  const { SnapshotCommand } = await import(pathToFileURL(join(src, 'engine/command.ts')));
  const cap = mode === 'history-50' ? 50 : 100;
  const probe = new WasmBridge();
  assert.equal(probe.snapshotCapacity(), null, '문서 미로드 fallback');
  probe.doc = mode === 'history-fallback' ? {} : { limit: cap, snapshotCapacity() { return this.limit; } };
  assert.equal(probe.snapshotCapacity(), mode === 'history-fallback' ? null : cap);
  const budget = cap - 2;
  const pos = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
  function fixture() {
    const store = new Map();
    const discarded = [];
    let nextId = 0, value = 0, implicitEvictions = 0;
    const wasm = {
      snapshotCapacity: () => probe.snapshotCapacity(),
      saveSnapshot() {
        const id = ++nextId;
        store.set(id, value);
        while (store.size > cap) { store.delete(store.keys().next().value); implicitEvictions++; }
        return id;
      },
      restoreSnapshot(id) { assert.ok(store.has(id), `소실된 snapshot ${id}`); value = store.get(id); },
      discardSnapshot(id) { discarded.push(id); store.delete(id); },
    };
    const history = new CommandHistory();
    const execute = (v, fail = false) => history.execute(new SnapshotCommand('budget-contract', pos, pos, () => {
      value = v;
      if (fail) throw new Error('operation failed');
      return pos;
    }), wasm);
    const check = () => {
      assert.equal(implicitEvictions, 0, 'CORE_EVICTION: 제품 예산이 코어 축출을 예방해야 한다');
      assert.ok(store.size <= budget, `BUDGET_EXCEEDED: ${store.size} > ${budget}`);
    };
    return { history, wasm, store, discarded, execute, check, value: () => value };
  }
  const f = fixture();
  const total = cap + 20;
  for (let i = 1; i <= total; i++) { f.execute(i); f.check(); }
  assert.equal(f.store.size, budget);
  assert.equal(f.discarded.length, total - budget);
  assert.equal(Math.min(...f.store.values()), total - budget, '가장 오래된 참조부터 축출');
  f.history.undo(f.wasm); f.check(); assert.equal(f.value(), total - 1);
  f.history.redo(f.wasm); f.check(); assert.equal(f.value(), total, '최신 상태 복원');
  const size = f.store.size;
  assert.throws(() => f.execute(999, true), /operation failed/);
  assert.equal(f.value(), total, '실패한 편집 rollback');
  assert.equal(f.store.size, size, '실패한 편집 snapshot 해제');
  f.history.undo(f.wasm); f.check();
  f.execute(777); f.check();
  assert.equal(f.history.redo(f.wasm), null, '새 편집은 redo 폐기');
  assert.equal(f.value(), 777);

  const deep = fixture();
  for (let i = 1; i <= budget; i++) deep.execute(i);
  let previous = budget, undos = 0;
  while (deep.history.undo(deep.wasm) !== null) {
    deep.check();
    assert.equal(deep.value(), --previous, '연속 undo 값 보존');
    assert.ok(++undos <= budget, 'undo 종료');
  }
  // 첫 undo는 after ID를 추가한다. redo가 하나뿐이면 이를 보존하고 가장 오래된 undo 하나를 축출한다.
  assert.equal(undos, budget - 1, '첫 undo의 추가 슬롯 이후 남은 과거 이력 보존');
  let redos = 0;
  while (deep.history.redo(deep.wasm) !== null) {
    deep.check();
    assert.equal(deep.value(), ++previous, '가까운 미래부터 연속 redo');
    assert.ok(++redos <= budget, 'redo 종료');
  }
  assert.equal(redos, Math.floor(budget / 2), 'undo 축출은 가장 먼 미래부터');
} else if (mode === 'font') {
  class CanvasContext {
    _font = '';
    get font() { return this._font; }
    set font(value) { this._font = value; }
  }
  globalThis.CanvasRenderingContext2D = CanvasContext;
  const bridge = new WasmBridge();
  await bridge.initialize();
  const { parseCssFontFamilyList, fontFamilyCandidatesForDisplay } =
    await import(pathToFileURL(join(src, 'core/font-substitution.ts')));
  const context = new CanvasContext();
  context.font = 'bold 18.667px "한양중고딕", "HY중고딕", "HYGothic", "HYGothic-Medium", \'Malgun Gothic\', sans-serif';
  assert.ok(context.font.startsWith('bold 18.667px '));
  const names = parseCssFontFamilyList(context.font.split('px ')[1]);
  assert.ok(names.includes('HYGothic'), 'ENGINE_ALIAS_LOST: 실제 font setter에서 설치 별칭 보존');
  assert.ok(names.indexOf('HYGothic') < names.indexOf('Malgun Gothic'));
  context.font = '16px "휴먼명조", "Engine Only Face", serif';
  const selected = parseCssFontFamilyList(context.font.split('px ')[1]);
  assert.equal(selected[0], fontFamilyCandidatesForDisplay('휴먼명조', 0, 0)[0], 'Studio 문서 치환 우선');
  assert.ok(selected.includes('Engine Only Face'));
  context.font = '12px "A, B", "Alias", "alias", sans-serif';
  const escaped = parseCssFontFamilyList(context.font.split('px ')[1]);
  assert.ok(escaped.includes('A, B'));
  assert.equal(escaped.filter(n => n.toLowerCase() === 'alias').length, 1);
} else {
  throw new Error(`알 수 없는 검사 ${mode}`);
}
console.log(`RUNTIME_CONTRACT_OK ${mode}`);
