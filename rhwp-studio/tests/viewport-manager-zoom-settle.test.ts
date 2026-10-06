import test, { type TestContext } from 'node:test';
import assert from 'node:assert/strict';
import { EventBus } from '../src/core/event-bus.ts';
import { ViewportManager } from '../src/view/viewport-manager.ts';
import { MIN_DOCUMENT_ZOOM, MAX_DOCUMENT_ZOOM } from '../src/view/page-arrangement.ts';
import type { ZoomInputSettleHost } from '../src/view/zoom-input-settle.ts';

function fixture(t: TestContext, quietMs = 120) {
  let now = 0;
  let id = 0;
  const frames = new Map<number, FrameRequestCallback>();
  const timers = new Map<number, { at: number; callback: () => void }>();
  const stale: (() => void)[] = [];
  const oldRequest = globalThis.requestAnimationFrame;
  const oldCancel = globalThis.cancelAnimationFrame;
  globalThis.requestAnimationFrame = callback => { frames.set(++id, callback); return id; };
  globalThis.cancelAnimationFrame = id => { frames.delete(id); };
  const host: ZoomInputSettleHost = {
    now: () => now,
    schedule: (callback, delay) => {
      timers.set(++id, { at: now + delay, callback }); stale.push(callback);
      return id as unknown as ReturnType<typeof setTimeout>;
    },
    cancel: id => { timers.delete(id as unknown as number); },
  };
  const bus = new EventBus();
  const vm = new ViewportManager(bus, host, quietMs);
  const events: { name: string; zoom: number; pending: boolean; animating: boolean; args: unknown[] }[] = [];
  for (const name of ['zoom-changed', 'zoom-raster-ready']) bus.on(name, (...args) => {
    events.push({ name, zoom: vm.getZoom(), pending: vm.isZoomRasterPending(), animating: vm.isZoomAnimating(), args });
  });
  t.after(() => {
    vm.cancelPendingZoomRaster();
    globalThis.requestAnimationFrame = oldRequest;
    globalThis.cancelAnimationFrame = oldCancel;
  });
  const wheel = (deltaY: number, ctrlKey = true, metaKey = false) => {
    (vm as unknown as { onWheel(e: unknown): void }).onWheel({
      ctrlKey, metaKey, deltaX: 0, deltaY, deltaMode: 0, clientX: 0, clientY: 0,
      preventDefault() {},
    });
  };
  const advance = (to: number, renderFrame = true) => {
    now = to;
    if (renderFrame) {
      const pending = [...frames]; frames.clear();
      for (const [, callback] of pending) callback(now);
    }
    for (const [key, timer] of timers) {
      if (timer.at <= now) { timers.delete(key); timer.callback(); }
    }
  };
  const ready = () => events.filter(e=>e.name==='zoom-raster-ready');
  return { vm, bus, frames, timers, stale, events, wheel, advance, ready };
}

test('실제 epsilon 경로: 16ms 간격의 작은 입력 20개는 geometry 20회·최종 ready 1회', t => {
  const f = fixture(t);
  for (let i = 0; i < 20; i++) {
    f.advance(i * 16);
    f.wheel(0.05);
    assert.equal(f.vm.isZoomAnimating(), false, '실제 수렴 상태를 true로 위장하지 않는다');
    assert.equal(f.vm.isZoomRasterPending(), true);
    assert.equal(f.frames.size, 0);
    assert.equal(f.timers.size, 1);
    assert.equal(f.ready().length, 0);
  }
  assert.equal(f.events.length, 20);
  assert(f.events.every(e=>e.pending && !e.animating));
  assert(Math.abs(f.vm.getZoom() - Math.exp(-20 * .05 * .00625)) < 1e-12);
  f.advance(423);
  assert.equal(f.ready().length, 0);
  f.advance(424);
  assert.equal(f.ready().length, 1);
  assert.equal(f.events.filter(e=>e.name==='zoom-changed').length, 20, 'ready가 geometry를 재발행하지 않는다');
  assert.equal(f.vm.isZoomRasterPending(), false);
});

test('큰 휠 입력: quiet 뒤에도 animation이 남으면 최종 수렴까지 기다린다', t => {
  const f = fixture(t, 80);
  f.wheel(100);
  f.advance(80, false);
  assert.equal(f.vm.getZoomInputState().inputActive, false);
  assert.equal(f.vm.isZoomAnimating(), true);
  assert.equal(f.ready().length, 0);
  for (let at = 96; at < 400; at += 16) f.advance(at);
  assert.equal(f.ready().length, 1);
  assert.equal(f.ready()[0].animating, false);
  assert.equal(f.frames.size, 0);
  assert.equal(f.timers.size, 0);
});

test('역방향 재입력은 이전 quiet callback을 무효화하고 최종 배율을 보존한다', t => {
  const f = fixture(t);
  f.wheel(8);
  f.advance(16);
  const stale = f.stale[0];
  f.wheel(-8);
  stale();
  for (let at = 32; at <= 144; at += 16) f.advance(at);
  assert.equal(f.ready().length, 1);
  assert.equal(f.vm.getZoom(), 1);
  assert.equal(f.vm.isCurrentZoomRasterReady(f.ready()[0].args[0]), true);
});

for (const action of ['direct', 'smooth', 'cancel', 'plain-wheel'] as const) {
  test(action + ': 명시 조작/무효화는 이전 wheel timer를 남기지 않는다', t => {
    const f = fixture(t);
    f.wheel(0.05);
    const stale = f.stale[0];
    if (action === 'direct') f.vm.setZoom(.8, { x: .2, y: .7 }, 'width');
    if (action === 'smooth') f.vm.smoothZoomTo(.8);
    if (action === 'cancel') f.vm.cancelPendingZoomRaster();
    if (action === 'plain-wheel') f.wheel(20, false);
    stale();
    for (let at = 16; at < 400; at += 16) f.advance(at);
    assert.equal(f.ready().length, action === 'plain-wheel' ? 1 : 0);
    assert.equal(f.vm.isZoomRasterPending(), false);
    assert.equal(f.timers.size, 0);
    if (action === 'direct' || action === 'smooth') assert.equal(f.vm.getZoom(), .8);
    if (action === 'direct') assert.equal(f.vm.getZoomFitMode(), 'width');
  });
}

test('min/max·delta=0 무효 입력은 timer·ready·새 geometry를 만들지 않는다', t => {
  const f = fixture(t);
  for (const [zoom, delta] of [[MIN_DOCUMENT_ZOOM, 120], [MAX_DOCUMENT_ZOOM, -120]]) {
    f.vm.setZoom(zoom);
    const count = f.events.length;
    f.wheel(delta);
    f.wheel(0);
    f.advance(1000);
    assert.equal(f.events.length, count);
    assert.equal(f.timers.size, 0);
    assert.equal(f.frames.size, 0);
  }
});

test('meta wheel도 quiet를 사용하며 zoom-changed 재진입 명령은 이전 ready를 취소한다', t => {
  const f = fixture(t);
  const off = f.bus.on('zoom-changed', () => { off(); f.vm.setZoom(.5); });
  f.wheel(.05, false, true);
  f.advance(1000);
  assert.equal(f.vm.getZoom(), .5);
  assert.equal(f.ready().length, 0);
  assert.equal(f.timers.size, 0);
});

test('detach는 입력 timer와 animation을 함께 무효화한다', t => {
  const f = fixture(t);
  const oldWindow = globalThis.window;
  globalThis.window = { removeEventListener() {} } as unknown as Window & typeof globalThis;
  t.after(() => { globalThis.window = oldWindow; });
  f.wheel(8);
  const stale = f.stale[0];
  f.vm.detach();
  stale();
  f.advance(1000);
  assert.equal(f.ready().length, 0);
  assert.equal(f.frames.size, 0);
  assert.equal(f.timers.size, 0);
});
