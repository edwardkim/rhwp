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
} else if (mode === 'group' || mode === 'textbox') {
  const {onClick} = await import(pathToFileURL(join(src, 'engine/input-handler-mouse.ts')));
  const {CursorState} = await import(pathToFileURL(join(src, 'engine/cursor.ts')));
  const events = [], addressReads = [];
  const wasm = {
    pageCount: 383,
    getCursorRect: () => ({pageIndex: 2, x: 350, y: 550, height: 12}),
    hitTest() {
      addressReads.push('텍스트 hit');
      if (mode === 'group') throw Error('묶음의 자식은 외곽 묶음의 셀 0이 아니다');
      return {sectionIndex: 0, paragraphIndex: 0, charOffset: 1, parentParaIndex: 4,
        controlIndex: 0, cellIndex: 0, cellParaIndex: 0, isTextBox: true,
        cellPath: [{controlIndex: 0, cellIndex: 0, cellParaIndex: 0}],
        cursorRect: {pageIndex: 2, x: 350, y: 550, height: 12}};
    },
  };
  const cursor = new CursorState(wasm);
  const rect = {left: 0, top: 0};
  const content = {clientWidth: 740, getBoundingClientRect: () => rect};
  const handler = {
    wasm, cursor, container: {clientWidth: 1000, clientHeight: 1100, getBoundingClientRect: () => rect, querySelector: () => content},
    viewportManager: {getZoom: () => 1},
    virtualScroll: {getPageAtPoint: () => 2, getPageOffset: () => 0, getPageWidth: () => 740, getPageLeftResolved: () => 0},
    findPictureAtClick: () => ({type: mode === 'group' ? 'group' : 'shape', sec: 0, ppi: 4, ci: 0, pageIndex: 2}),
    isShapeBorderClickByRef: () => false,
    exitPictureObjectSelectionIfNeeded() {}, caret: {hide() {}}, fieldMarker: {hide() {}},
    selectionRenderer: {clear() {}}, cellSelectionRenderer: {clear() {}},
    renderPictureObjectSelection() { events.push('그룹 선택 표시'); },
    eventBus: {emit: (...args) => events.push(args)}, textarea: {focus() {}},
    updateCaret() { events.push('텍스트 캐럿 표시'); }, startTextSelectionDrag() {},
  };
  globalThis.document = {addEventListener() {}};
  onClick.call(handler, {clientX: 350, clientY: 550, button: 0, shiftKey: false, preventDefault() {}, target: {closest: () => null}});
  if (mode === 'textbox') {
    assert.equal(cursor.isInPictureObjectSelection(), false, '일반 글상자는 개체로 가로채지 않는다');
    assert.equal(cursor.isInTextBox(), true, '일반 글상자 텍스트 편집은 유지한다');
    assert.equal(cursor.getPosition().charOffset, 1);
    assert.ok(events.includes('텍스트 캐럿 표시'));
  } else {
  assert.equal(cursor.isInPictureObjectSelection(), true, '묶음 내부 클릭은 실제 외곽 묶음을 선택한다');
  assert.equal(cursor.getSelectedPictureRef().type, 'group');
  assert.equal(cursor.getSelectedPictureRef().ppi, 4);
  assert.equal(cursor.getSelectedPictureRef().pageIndex, 2);
  assert.deepEqual(addressReads, [], '잘못된 셀 주소로 텍스트 편집에 진입하지 않는다');
  assert.ok(events.includes('그룹 선택 표시'));
  }
} else throw Error('알 수 없는 검사 모드');
console.log(`${mode}: PASS`);
