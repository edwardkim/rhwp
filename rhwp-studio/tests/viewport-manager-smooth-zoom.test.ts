import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

type AnimationFrameCallback = (timestamp: number) => void;
type ViewportManagerConstructor = new (eventBus: never) => {
  getZoom(): number;
};
let viewportManagerModule: Promise<{ ViewportManager: ViewportManagerConstructor }> | null = null;

function loadViewportManager(): Promise<{ ViewportManager: ViewportManagerConstructor }> {
  if (viewportManagerModule) return viewportManagerModule;
  viewportManagerModule = import('../src/view/viewport-manager.ts') as Promise<{
    ViewportManager: ViewportManagerConstructor;
  }>;
  return viewportManagerModule;
}

class FakeEventBus {
  readonly events: Array<{ event: string; args: unknown[] }> = [];

  emit(event: string, ...args: unknown[]): void {
    this.events.push({ event, args });
  }
}

class FakeAnimationFrames {
  private nextId = 1;
  private callbacks = new Map<number, AnimationFrameCallback>();

  request = (callback: AnimationFrameCallback): number => {
    const id = this.nextId++;
    this.callbacks.set(id, callback);
    return id;
  };

  cancel = (id: number): void => {
    this.callbacks.delete(id);
  };

  get pendingCount(): number {
    return this.callbacks.size;
  }

  flush(timestamp: number): void {
    const callbacks = [...this.callbacks.values()];
    this.callbacks.clear();
    callbacks.forEach((callback) => callback(timestamp));
  }
}

test('rapid zoom requests coalesce into one eased animation', async (t) => {
  const frames = new FakeAnimationFrames();
  const previousRequest = globalThis.requestAnimationFrame;
  const previousCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = frames.request;
  globalThis.cancelAnimationFrame = frames.cancel;
  t.after(() => {
    globalThis.requestAnimationFrame = previousRequest;
    globalThis.cancelAnimationFrame = previousCancel;
  });

  const { ViewportManager } = await loadViewportManager();
  const eventBus = new FakeEventBus();
  const viewport = new ViewportManager(eventBus as never);

  assert.equal(
    typeof (viewport as { smoothZoomBy?: (delta: number) => void }).smoothZoomBy,
    'function',
    'ViewportManager should expose eased zoom requests for user input',
  );
  assert.equal(
    typeof (viewport as { isZoomAnimating?: () => boolean }).isZoomAnimating,
    'function',
    'CanvasView should be able to distinguish preview frames from the settled frame',
  );

  const smoothZoomBy = (
    viewport as unknown as { smoothZoomBy: (delta: number) => void }
  ).smoothZoomBy.bind(viewport);
  smoothZoomBy(-0.1);
  smoothZoomBy(-0.1);

  assert.equal(
    (viewport as unknown as { isZoomAnimating: () => boolean }).isZoomAnimating(),
    true,
  );
  assert.equal(viewport.getZoom(), 1, 'requests should not jump synchronously');
  assert.equal(frames.pendingCount, 1, 'rapid requests should share one frame');

  frames.flush(16);

  assert.ok(viewport.getZoom() < 1, 'the first frame should move toward the target');
  assert.ok(viewport.getZoom() > 0.8, 'the first frame should ease instead of jumping');
  assert.equal(
    eventBus.events.filter(({ event }) => event === 'zoom-changed').length,
    1,
    'one animation frame should emit one layout update',
  );

  let timestamp = 32;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  assert.equal(viewport.getZoom(), 0.8, 'the animation should settle at the combined target');
  assert.equal(frames.pendingCount, 0, 'the settled animation should stop scheduling frames');
  assert.equal(
    (viewport as unknown as { isZoomAnimating: () => boolean }).isZoomAnimating(),
    false,
  );
});

test('a fine trackpad wheel delta produces a fine animated zoom change', async (t) => {
  const frames = new FakeAnimationFrames();
  const previousRequest = globalThis.requestAnimationFrame;
  const previousCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = frames.request;
  globalThis.cancelAnimationFrame = frames.cancel;
  t.after(() => {
    globalThis.requestAnimationFrame = previousRequest;
    globalThis.cancelAnimationFrame = previousCancel;
  });

  const { ViewportManager } = await loadViewportManager();
  const viewport = new ViewportManager(new FakeEventBus() as never);
  let prevented = false;
  const onWheel = (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel.bind(viewport);

  onWheel({
    ctrlKey: true,
    metaKey: false,
    deltaY: 1,
    deltaMode: 0,
    preventDefault: () => {
      prevented = true;
    },
  });

  assert.equal(prevented, true, 'document zoom should replace browser zoom');
  assert.equal(viewport.getZoom(), 1, 'wheel input should not jump synchronously');
  assert.equal(frames.pendingCount, 1, 'wheel input should start one animation');

  let timestamp = 16;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  assert.ok(viewport.getZoom() < 1, 'positive deltaY should zoom out');
  assert.ok(
    viewport.getZoom() > 0.99,
    `a one-pixel trackpad delta should stay fine-grained, got ${viewport.getZoom()}`,
  );
});

test('vertical-dominant wheel input locks horizontal pan in every delta mode', async () => {
  const { ViewportManager } = await loadViewportManager();
  const viewport = new ViewportManager(new FakeEventBus() as never);
  const container = { scrollTop: 100 };
  (
    viewport as unknown as {
      container: typeof container;
      viewportHeight: number;
    }
  ).container = container;
  (viewport as unknown as { viewportHeight: number }).viewportHeight = 600;
  const onWheel = (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        deltaX: number;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel.bind(viewport);

  for (const sample of [
    { deltaY: 20, deltaMode: 0, expected: 120 },
    { deltaY: 2, deltaMode: 1, expected: 132 },
    { deltaY: 0.5, deltaMode: 2, expected: 400 },
  ]) {
    container.scrollTop = 100;
    let prevented = false;
    onWheel({
      ctrlKey: false,
      metaKey: false,
      deltaX: 0.1,
      deltaY: sample.deltaY,
      deltaMode: sample.deltaMode,
      preventDefault: () => {
        prevented = true;
      },
    });
    assert.equal(prevented, true);
    assert.equal(container.scrollTop, sample.expected);
  }
});

test('세로 쪽 이동의 가로 우세 입력은 native 가로 pan을 유지한다', async () => {
  const { ViewportManager } = await loadViewportManager();
  const viewport = new ViewportManager(new FakeEventBus() as never);
  const container = { scrollTop: 100 };
  (viewport as unknown as { container: typeof container }).container = container;
  let prevented = false;
  (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        deltaX: number;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel({
    ctrlKey: false,
    metaKey: false,
    deltaX: 20,
    deltaY: 3,
    deltaMode: 0,
    preventDefault: () => {
      prevented = true;
    },
  });

  assert.equal(prevented, false);
  assert.equal(container.scrollTop, 100);
});

test('가로 쪽 이동은 선택했을 때 가로·세로 우세 입력을 모두 좌우 스크롤로 바꾼다', async () => {
  const { ViewportManager } = await loadViewportManager();
  const viewport = new ViewportManager(new FakeEventBus() as never) as unknown as {
    setPageMovement: (value: { direction: 'horizontal'; wheelHorizontal: boolean }) => void;
    container: { scrollTop: number; scrollLeft: number };
    onWheel: (event: {
      ctrlKey: boolean;
      metaKey: boolean;
      shiftKey: boolean;
      deltaX: number;
      deltaY: number;
      deltaMode: number;
      preventDefault: () => void;
    }) => void;
  };
  viewport.container = { scrollTop: 0, scrollLeft: 100 };
  viewport.setPageMovement({ direction: 'horizontal', wheelHorizontal: true });
  let prevented = false;
  viewport.onWheel({
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    deltaX: 0,
    deltaY: 32,
    deltaMode: 0,
    preventDefault: () => { prevented = true; },
  });
  assert.equal(prevented, true);
  assert.equal(viewport.container.scrollLeft, 132);
  assert.equal(viewport.container.scrollTop, 0);

  prevented = false;
  viewport.onWheel({
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    deltaX: 24,
    deltaY: 3,
    deltaMode: 0,
    preventDefault: () => { prevented = true; },
  });
  assert.equal(prevented, true);
  assert.equal(viewport.container.scrollLeft, 156);
  assert.equal(viewport.container.scrollTop, 0);

  prevented = false;
  viewport.onWheel({
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    deltaX: -10,
    deltaY: -2,
    deltaMode: 0,
    preventDefault: () => { prevented = true; },
  });
  assert.equal(prevented, true);
  assert.equal(viewport.container.scrollLeft, 146);
  assert.equal(viewport.container.scrollTop, 0);

  viewport.setPageMovement({ direction: 'horizontal', wheelHorizontal: false });
  prevented = false;
  viewport.onWheel({
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    deltaX: 0,
    deltaY: 32,
    deltaMode: 0,
    preventDefault: () => { prevented = true; },
  });
  assert.equal(prevented, false);
  assert.equal(viewport.container.scrollLeft, 146);
});

test('an eight-pixel trackpad gesture settles within four frames and moves nearly five percent', async (t) => {
  const frames = new FakeAnimationFrames();
  const previousRequest = globalThis.requestAnimationFrame;
  const previousCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = frames.request;
  globalThis.cancelAnimationFrame = frames.cancel;
  t.after(() => {
    globalThis.requestAnimationFrame = previousRequest;
    globalThis.cancelAnimationFrame = previousCancel;
  });

  const { ViewportManager } = await loadViewportManager();
  const viewport = new ViewportManager(new FakeEventBus() as never);
  const onWheel = (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel.bind(viewport);

  onWheel({
    ctrlKey: true,
    metaKey: false,
    deltaY: 8,
    deltaMode: 0,
    preventDefault: () => {},
  });

  let timestamp = 16;
  let frameCount = 0;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
    frameCount += 1;
  }

  assert.ok(
    Math.abs(viewport.getZoom() - Math.exp(-8 * 0.00625)) < 1e-12,
    `expected the stronger symmetric sensitivity, got ${viewport.getZoom()}`,
  );
  assert.ok(frameCount <= 4, `expected at most four frames, got ${frameCount}`);
});

test('wheel zoom emits the pointer anchor and inverse deltas restore zoom', async (t) => {
  const frames = new FakeAnimationFrames();
  const previousRequest = globalThis.requestAnimationFrame;
  const previousCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = frames.request;
  globalThis.cancelAnimationFrame = frames.cancel;
  t.after(() => {
    globalThis.requestAnimationFrame = previousRequest;
    globalThis.cancelAnimationFrame = previousCancel;
  });

  const { ViewportManager } = await loadViewportManager();
  const eventBus = new FakeEventBus();
  const viewport = new ViewportManager(eventBus as never);
  (viewport as unknown as { container: Pick<HTMLElement, 'getBoundingClientRect'> }).container = {
    getBoundingClientRect: () => ({
      left: 100,
      top: 50,
      right: 900,
      bottom: 650,
      width: 800,
      height: 600,
      x: 100,
      y: 50,
      toJSON: () => ({}),
    }),
  };

  const onWheel = (
    viewport as unknown as {
      onWheel: (event: {
        ctrlKey: boolean;
        metaKey: boolean;
        clientX: number;
        clientY: number;
        deltaY: number;
        deltaMode: number;
        preventDefault: () => void;
      }) => void;
    }
  ).onWheel.bind(viewport);

  onWheel({
    ctrlKey: true,
    metaKey: false,
    clientX: 300,
    clientY: 500,
    deltaY: -8,
    deltaMode: 0,
    preventDefault: () => {},
  });

  let timestamp = 16;
  while (frames.pendingCount > 0 && timestamp < 1000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  const zoomedIn = viewport.getZoom();
  const firstZoomEvent = eventBus.events.find(({ event }) => event === 'zoom-changed');
  assert.deepEqual(firstZoomEvent?.args[1], { x: 0.25, y: 0.75 });

  onWheel({
    ctrlKey: true,
    metaKey: false,
    clientX: 300,
    clientY: 500,
    deltaY: 8,
    deltaMode: 0,
    preventDefault: () => {},
  });

  while (frames.pendingCount > 0 && timestamp < 2000) {
    frames.flush(timestamp);
    timestamp += 16;
  }

  assert.ok(zoomedIn > 1);
  assert.ok(Math.abs(viewport.getZoom() - 1) < 1e-12);
});

test('zoom in and out controls share the command smooth zoom path', () => {
  const mainSource = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  const viewCommandSource = readFileSync(
    new URL('../src/command/commands/view.ts', import.meta.url),
    'utf8',
  );

  assert.match(mainSource, /zoomIn\.addEventListener[\s\S]*?dispatcher\.dispatch\('view:zoom-in'\)/);
  assert.match(mainSource, /zoomOut\.addEventListener[\s\S]*?dispatcher\.dispatch\('view:zoom-out'\)/);
  assert.match(viewCommandSource, /id: 'view:zoom-in'[\s\S]*?smoothZoomBy\(0\.1\)/);
  assert.match(viewCommandSource, /id: 'view:zoom-out'[\s\S]*?smoothZoomBy\(-0\.1\)/);
});

test('CanvasView scales existing pages during zoom and rerenders only after settling', () => {
  const source = readFileSync(new URL('../src/view/canvas-view.ts', import.meta.url), 'utf8');

  assert.match(
    source,
    /eventBus\.on\('viewport-scroll', \(\) => \{[\s\S]*?if \(!this\.viewportManager\.isZoomAnimating\(\)\) this\.updateVisiblePages\('scroll'\);[\s\S]*?\}\)/,
  );
  assert.match(
    source,
    /if \(this\.viewportManager\.isZoomAnimating\(\)\) \{[\s\S]*?this\.cancelPendingPrefetch\(\);[\s\S]*?this\.updateRenderedPageZoomPreview\(\);[\s\S]*?return;/,
  );
  assert.match(source, /dataset\.rhwpRenderedZoom = String\(zoom\)/);
});

test('InputHandler는 zoom과 resize에서 같은 확정 페이지 좌표로 캐럿·선택을 다시 투영한다', () => {
  const source = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');

  assert.match(
    source,
    /eventBus\.on\('zoom-changed', \(\) => this\.updateViewportOverlayPositions\(\)\)/,
  );
  assert.match(
    source,
    /eventBus\.on\('viewport-resize', \(\) => \{[\s\S]*?window\.setTimeout\(\(\) => this\.updateViewportOverlayPositions\(\), 0\);[\s\S]*?\}\)/,
  );
  assert.match(
    source,
    /private updateViewportOverlayPositions\(\): void \{[\s\S]*?this\.caret\.updatePosition\(this\.viewportManager\.getZoom\(\)\)[\s\S]*?this\.updateSelection\(\)[\s\S]*?this\.updateCellSelection\(\)[\s\S]*?this\.renderPictureObjectSelection\(\)[\s\S]*?this\.renderTableObjectSelection\(\)/,
  );
});
