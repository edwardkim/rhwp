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
  width = 0;
  height = 0;
  style = {};
  captures = new Set();
  paint = { labels: [], strokes: 0, fills: 0 };
  constructor(left, top) {
    super();
    this.rect = { left, top };
    this.ctx = {
      save() {}, restore() {}, setTransform() {}, fillRect() {}, beginPath() {},
      moveTo() {}, lineTo() {}, closePath() {}, translate() {}, rotate() {},
      stroke: () => this.paint.strokes++,
      fill: () => this.paint.fills++,
      fillText: text => this.paint.labels.push(text),
    };
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

export function rulerFixture(t) {
  const originals = new Map(['window', 'document', 'getComputedStyle', 'requestAnimationFrame',
    'cancelAnimationFrame'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]));
  const win = Object.assign(new FakeTarget(), { devicePixelRatio: 1 });
  const doc = Object.assign(new FakeTarget(), { documentElement: {} });
  const frames = new Map();
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
    callbacks.forEach(callback => callback(16));
  };
  const h = new FakeCanvas(20, 0);
  const v = new FakeCanvas(0, 20);
  const container = { clientWidth: 355, clientHeight: 700, querySelector: () => ({ offsetLeft: 0 }) };
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
  bus.emit('focused-page-changed', 0);
  bus.emit('cursor-para-changed', { marginLeft: 0, indent: 0 });
  flush();

  t.after(() => {
    ruler.dispose();
    for (const [key, descriptor] of originals) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  });
  return { ruler, h, v, doc, win, container, page, wasm, scroll, viewport, bus, commits, frames, flush };
}
