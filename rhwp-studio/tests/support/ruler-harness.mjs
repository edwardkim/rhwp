import { registerHooks } from 'node:module';

// 실제 클래스와 좌표 계산을 로드한다. WASM/화면 객체만 fake로 주입한다.
const src = new URL('../../src/', import.meta.url);
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) {
      return { url: new URL(`${specifier.slice(2)}.ts`, src).href, shortCircuit: true };
    }
    if (specifier.startsWith('.') && !/\.[cm]?[tj]s$/.test(specifier)) {
      return { url: new URL(`${specifier}.ts`, context.parentURL).href, shortCircuit: true };
    }
    return nextResolve(specifier, context);
  },
});
const { Ruler } = await import('../../src/view/ruler.ts');
const { EventBus } = await import('../../src/core/event-bus.ts');

class FakeTarget extends EventTarget {
  listeners = new Map();
  addEventListener(type, listener, options) {
    super.addEventListener(type, listener, options);
    if (!this.listeners.has(type)) this.listeners.set(type, new Set());
    this.listeners.get(type).add(listener);
  }
  removeEventListener(type, listener, options) {
    super.removeEventListener(type, listener, options);
    this.listeners.get(type)?.delete(listener);
  }
  get listenerCount() {
    return [...this.listeners.values()].reduce((count, listeners) => count + listeners.size, 0);
  }
}

class FakeCanvas extends FakeTarget {
  bitmapSize = { width: 0, height: 0 };
  bitmapBlank = true;
  captures = new Set();
  paint = { labels: [], strokes: 0, fills: 0 };
  constructor(left, top, record) {
    super();
    this.rect = { left, top };
    this.record = record;
    this.style = new Proxy({}, {
      set: (style, property, value) => {
        style[property] = value;
        if (property === 'width' || property === 'height') record('css', { property, value });
        return true;
      },
    });
    let backgroundPending = false;
    this.ctx = {
      save() {}, restore() {}, beginPath() {},
      setTransform: (...value) => {
        backgroundPending = true;
        record('transform', { value });
      },
      fillRect: (...rect) => {
        // Ruler는 transform 뒤 전체 배경부터 그린다. 최신 paint의 숫자·핀만 보존한다.
        if (backgroundPending) this.paint = { labels: [], strokes: 0, fills: 0 };
        backgroundPending = false;
        this.bitmapBlank = false;
        record('paint', { rect, color: this.ctx.fillStyle });
      },
      moveTo() {}, lineTo() {}, closePath() {}, translate() {}, rotate() {},
      stroke: () => this.paint.strokes++,
      fill: () => this.paint.fills++,
      fillText: text => this.paint.labels.push(text),
    };
  }
  get width() { return this.bitmapSize.width; }
  set width(value) { this.resetBitmap('width', value); }
  get height() { return this.bitmapSize.height; }
  set height(value) { this.resetBitmap('height', value); }
  resetBitmap(dimension, value) {
    // HTML canvas는 같은 값 대입도 bitmap을 지운다. 실제 setter 호출 순서를 기록한다.
    this.bitmapSize[dimension] = value;
    this.bitmapBlank = true;
    this.paint = { labels: [], strokes: 0, fills: 0 };
    this.record('reset', { dimension, value });
  }
  getContext() { return this.ctx; }
  getBoundingClientRect() { return this.rect; }
  setPointerCapture(id) { this.captures.add(id); }
  hasPointerCapture(id) { return this.captures.has(id); }
  releasePointerCapture(id) {
    this.captures.delete(id);
    pointer(this, 'lostpointercapture', { pointerId: id });
  }
}

export function pointer(target, type, overrides = {}) {
  const event = Object.assign(new Event(type, { cancelable: true }), {
    pointerType: 'mouse', pointerId: 1, isPrimary: true,
    button: 0, buttons: type === 'pointerup' ? 0 : 1,
    clientX: 60, clientY: 18, ...overrides,
  });
  target.dispatchEvent(event);
  return event;
}

export function rulerFixture(t, { initialize = true, context = true, width = 355, height = 700, dpr = 1 } = {}) {
  const originals = new Map(['window', 'document', 'getComputedStyle', 'requestAnimationFrame',
    'cancelAnimationFrame'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const win = Object.assign(new FakeTarget(), { devicePixelRatio: dpr });
  const doc = Object.assign(new FakeTarget(), { documentElement: {} });
  const frames = new Map();
  const operations = [];
  let frame = null;
  let frameNumber = 0;
  const record = axis => (kind, data) => operations.push({ axis, kind, frame, ...data });
  let nextFrameId = 1;
  Object.assign(globalThis, {
    window: win, document: doc,
    getComputedStyle: () => ({ getPropertyValue: () => '' }),
    requestAnimationFrame: callback => {
      const id = nextFrameId++;
      frames.set(id, callback);
      return id;
    },
    cancelAnimationFrame: id => frames.delete(id),
  });
  const flush = () => {
    const callbacks = [...frames.values()];
    frames.clear();
    frame = ++frameNumber;
    try { callbacks.forEach(callback => callback(frameNumber * 16)); }
    finally { frame = null; }
    return callbacks.length;
  };
  const h = new FakeCanvas(20, 0, record('h'));
  const v = new FakeCanvas(0, 20, record('v'));
  const scrollContent = { offsetLeft: 0 };
  const container = { clientWidth: width, clientHeight: height, querySelector: () => scrollContent };
  const page = {
    width: 300, height: 600, bodyLeft: 40, bodyRight: 260,
    marginTop: 40, marginBottom: 40, marginHeader: 0, marginFooter: 0,
  };
  const wasm = { pageCount: 1, getPageInfo: () => page };
  const scroll = {
    pageCount: 1, getPageLeftResolved: () => 0, getTotalWidth: () => 300,
    getPageOffset: () => 0, isGridMode: () => false,
  };
  const viewport = { getZoom: () => 1, getScrollX: () => 0, getScrollY: () => 0 };
  const bus = new EventBus();
  const ruler = new Ruler(h, v, container, bus, wasm, scroll, viewport);
  const commits = [];
  ruler.onCommitPin = commit => commits.push(commit);
  if (context) {
    bus.emit('focused-page-changed', 0);
    bus.emit('cursor-para-changed', { marginLeft: 0, indent: 0 });
  }
  if (initialize) flush();

  t.after(() => {
    ruler.dispose();
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  });
  return { ruler, h, v, doc, win, container, scrollContent, page, wasm, scroll, viewport,
    bus, commits, frames, operations, flush };
}
