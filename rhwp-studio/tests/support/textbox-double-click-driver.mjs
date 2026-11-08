import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../src/', import.meta.url));
const src = pathToFileURL(root).href;
registerHooks({
  resolve(specifier, context, next) {
    if (specifier.startsWith('@/')) return next(src + specifier.slice(2) + '.ts', context);
    if (/^\.{1,2}\//.test(specifier) && !/\.[a-z]+$/.test(specifier)) {
      return next(specifier + '.ts', context);
    }
    return next(specifier, context);
  },
});
const { onDblClick } = await import(src + 'engine/input-handler-mouse.ts');
const { CursorState } = await import(src + 'engine/cursor.ts');
const results = [];
for (const entry of [
  { name: 'boolean-page15', page: 14, target: 14, oldPage: 0, offset: 4 },
  { name: 'rotated-page4', page: 3, target: 3, oldPage: 3, offset: 2, rotation: -45 },
  { name: 'repeated-master', page: 15, target: 0, oldPage: 15, offset: 2 },
  { name: 'different-owner', page: 3, target: 3, oldPage: 3, offset: 2, mismatch: true },
]) {
  const rect = {
    pageIndex: entry.page, x: 432.9, y: 863.5, height: 12,
    ...(entry.rotation ? { rotation: entry.rotation } : {}),
  };
  const hit = {
    sectionIndex: entry.page, parentParaIndex: 0, paragraphIndex: 0,
    controlIndex: 275, cellIndex: 0, cellParaIndex: 0,
    charOffset: entry.offset, isTextBox: true,
    cellPath: [{ controlIndex: 275, cellIndex: 0, cellParaIndex: 0 }], cursorRect: rect,
  };
  const wasm = {
    getCursorRect() { return { pageIndex: entry.oldPage, x: 1, y: 1, height: 12 }; },
    getCursorRectInCell() { return rect; },
    getCursorRectByPathNear() { return { ...rect, pageIndex: entry.target }; },
  };
  const cursor = new CursorState(wasm);
  cursor.moveTo({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
  cursor.enterPictureObjectSelectionDirect(entry.page, 0, entry.mismatch ? 274 : 275, 'shape');
  let updates = 0;
  const handler = {
    active: true, cursor, wasm,
    container: { querySelector() { return null; } },
    textarea: { focus() {} }, eventBus: { emit() {} },
    pagePointFromClientPoint() { return { pageIdx: entry.page, pageX: 430, pageY: 860 }; },
    hitTestFromEvent() { return hit; }, pictureObjectRenderer: { clear() {} },
    updateCaret() { updates += 1; },
    // Previous mouse handler used this real entry shape, replacing pointer offset.
    enterTextboxEditing(sectionIndex, parentParaIndex, controlIndex) {
      cursor.moveTo({
        sectionIndex, paragraphIndex: 0, charOffset: 0, parentParaIndex,
        controlIndex, cellIndex: 0, cellParaIndex: 0, isTextBox: true,
      });
      updates += 1;
    },
  };
  onDblClick.call(handler, {
    target: { closest() { return null; } }, preventDefault() {}, clientX: 10, clientY: 10,
  });
  results.push({ name: entry.name, position: cursor.getPosition(), rect: cursor.getRect(), updates });
}
const { finishPictureResizeDrag, updatePictureResizeDrag } = await import(src + 'engine/input-handler-picture.ts');
for (const moved of [false, true]) {
  const writes = [], operations = [], events = [];
  const state = {
    ref: { sec: 3, ppi: 0, ci: 80, type: 'shape' }, dir: 'e',
    bbox: { x: 161.5, y: 499.7, w: 21.7, h: 10.7 },
    origWidth: 1633, origHeight: 800, origHorzOffset: 12000, origVertOffset: 37000,
    rotationAngle: -45, startClientX: 505, startClientY: 644,
  };
  const handler = {
    pictureResizeState: state,
    wasm: { getShapeProperties() { return {}; }, setShapeProperties(...args) { writes.push(args); } },
    viewportManager: { getZoom() { return 1; } },
    pictureObjectRenderer: { renderDragPreview() {} },
    cleanupPictureResizeDrag() { this.pictureResizeState = null; },
    renderPictureObjectSelection() {},
    executeOperation(operation) { operations.push(operation.kind); },
    eventBus: { emit(event) { events.push(event); } },
  };
  const event = { clientX: 505 + (moved ? 10 : 0), clientY: 644 };
  updatePictureResizeDrag.call(handler, event);
  finishPictureResizeDrag.call(handler, event);
  results.push({ name: moved ? 'real-resize' : 'handle-click', writes, operations, events });
}
console.log(JSON.stringify(results));
