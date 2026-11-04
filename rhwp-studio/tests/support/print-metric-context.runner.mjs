// 제품의 실제 bridge·mouse 함수를 실행한다. DOM·WASM 입출력만 대역이다.
import assert from 'node:assert/strict';
import {registerHooks} from 'node:module';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {dirname, join} from 'node:path';

const src = fileURLToPath(new URL('../../src/', import.meta.url));
registerHooks({resolve(specifier, context, next) {
  if (specifier.endsWith('.json')) return next(specifier, context);
  if (specifier === '@wasm/rhwp.js') return {url: 'data:text/javascript,export default function(){};export class HwpDocument{};export function version(){}', shortCircuit: true};
  if (specifier.startsWith('@/')) return next(pathToFileURL(join(src, specifier.slice(2) + '.ts')).href, context);
  if (/^\.{1,2}\//.test(specifier) && !/\.[cm]?[tj]s$/.test(specifier)) {
    return next(pathToFileURL(join(dirname(fileURLToPath(context.parentURL)), specifier + '.ts')).href, context);
  }
  return next(specifier, context);
}});

const mode = process.argv[2];
if (mode === 'print') {
  const {WasmBridge} = await import(pathToFileURL(join(src, 'core/wasm-bridge.ts')));
  const bridge = new WasmBridge();
  let active = true, fail = false;
  const painted = [];
  bridge.doc = {
    collectCanvasMetricRequests: () => '{"revision":1,"requests":[]}',
    registerCanvasMetricReplies: () => false,
    getCanvasPageLayerTree: () => '{}',
    canvasMetricsActive: () => active,
    selectCanvasMetrics: value => { active = value; },
    renderPageToCanvasFilteredWithProfile(_page, _canvas, _scale, _kind, profile) {
      painted.push({profile, active});
      if (fail) throw Error('실제 paint 실패 대역');
    },
  };
  for (const profile of ['screen', 'print', 'screen']) {
    bridge.renderPageToCanvasFiltered(0, {}, 1, 'all', profile);
    assert.equal(active, true, '화면 보충 메트릭을 복원한다');
  }
  assert.deepEqual(painted, [{profile: 'screen', active: true}, {profile: 'print', active: false}, {profile: 'screen', active: true}]);
  fail = true;
  assert.throws(() => bridge.renderPageToCanvasFiltered(0, {}, 1, 'all', 'print'), /paint 실패/);
  assert.equal(active, true, '인쇄 실패 뒤에도 화면 상태를 복원한다');
} else throw Error('알 수 없는 검사 모드');
console.log(`${mode}: PASS`);
