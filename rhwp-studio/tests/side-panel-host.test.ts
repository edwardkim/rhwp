import test from 'node:test';
import assert from 'node:assert/strict';

import {
  SIDE_PANEL_MAX_WIDTH,
  SIDE_PANEL_MIN_WIDTH,
  SidePanelController,
  clampPanelWidth,
  type SidePanelView,
  type WidthStore,
} from '../src/view/side-panel-host.ts';

function fakeView() {
  const log: string[] = [];
  let closeCb = () => {};
  let resizeCb = (_width: number) => {};
  const view: SidePanelView = {
    createContainer(id) { log.push(`create:${id}`); return { id } as unknown as HTMLElement; },
    removeContainer(id) { log.push(`remove:${id}`); },
    show(id, title, width) { log.push(`show:${id}:${title}:${width}`); },
    hide() { log.push('hide'); },
    onCloseRequest(cb) { closeCb = cb; },
    onResize(cb) { resizeCb = cb; },
  };
  return { view, log, requestClose: () => closeCb(), resize: (w: number) => resizeCb(w) };
}

function memoryWidths(): WidthStore & { values: Map<string, number> } {
  const values = new Map<string, number>();
  return { values, get: (id) => values.get(id) ?? null, set: (id, w) => { values.set(id, w); } };
}

function spec(id: string, events: string[]) {
  return {
    id,
    title: id.toUpperCase(),
    defaultWidth: 400,
    mount: () => { events.push(`mount:${id}`); },
    unmount: () => { events.push(`unmount:${id}`); },
    onVisibilityChange: (open: boolean) => { events.push(`${open ? 'open' : 'close'}:${id}`); },
  };
}

test('패널은 처음 열 때 한 번만 mount 하고 토글로 열고 닫는다', () => {
  const { view, log } = fakeView();
  const events: string[] = [];
  const panels = new SidePanelController(view, memoryWidths());
  panels.register(spec('a', events));

  assert.equal(panels.toggle('a'), true);
  assert.equal(panels.isOpen('a'), true);
  assert.equal(panels.toggle('a'), false);
  assert.equal(panels.toggle('a', true), true);
  assert.equal(panels.toggle('a', true), true, '이미 열린 패널을 다시 열어도 그대로');

  assert.deepEqual(events, ['mount:a', 'open:a', 'close:a', 'open:a']);
  assert.deepEqual(log, ['create:a', 'show:a:A:400', 'hide', 'show:a:A:400']);
});

test('패널은 한 번에 하나만 열린다', () => {
  const { view } = fakeView();
  const events: string[] = [];
  const panels = new SidePanelController(view, memoryWidths());
  panels.register(spec('a', events));
  panels.register(spec('b', events));

  panels.toggle('a', true);
  panels.toggle('b', true);
  assert.equal(panels.isOpen('a'), false);
  assert.equal(panels.isOpen('b'), true);
  assert.deepEqual(events, ['mount:a', 'open:a', 'close:a', 'mount:b', 'open:b']);
});

test('같은 id 재등록과 미등록 패널 토글은 거절한다', () => {
  const { view } = fakeView();
  const panels = new SidePanelController(view, memoryWidths());
  panels.register(spec('a', []));
  assert.throws(() => panels.register(spec('a', [])), /이미 등록된 패널/);
  assert.throws(() => panels.toggle('zzz'), /등록되지 않은 패널/);
});

test('해지는 열린 패널을 닫고 unmount·DOM 제거까지 한 번만 한다', () => {
  const { view, log } = fakeView();
  const events: string[] = [];
  const panels = new SidePanelController(view, memoryWidths());
  const off = panels.register(spec('a', events));
  panels.toggle('a', true);
  off();
  off();
  assert.equal(panels.isOpen('a'), false);
  assert.deepEqual(events, ['mount:a', 'open:a', 'close:a', 'unmount:a']);
  assert.deepEqual(log.slice(-2), ['hide', 'remove:a']);
  assert.throws(() => panels.toggle('a'), /등록되지 않은 패널/);
});

test('mount 전에 해지하면 unmount 를 부르지 않는다', () => {
  const { view, log } = fakeView();
  const events: string[] = [];
  const panels = new SidePanelController(view, memoryWidths());
  panels.register(spec('a', events))();
  assert.deepEqual(events, []);
  assert.deepEqual(log, []);
});

test('닫기 요청과 사용자 폭 조절을 반영하고 폭은 범위 안으로 자른다', () => {
  const { view, log, requestClose, resize } = fakeView();
  const widths = memoryWidths();
  const panels = new SidePanelController(view, widths);
  panels.register(spec('a', []));
  panels.toggle('a', true);

  resize(9999);
  assert.equal(widths.values.get('a'), SIDE_PANEL_MAX_WIDTH);
  requestClose();
  assert.equal(panels.isOpen('a'), false);

  panels.toggle('a', true);
  assert.equal(log.at(-1), `show:a:A:${SIDE_PANEL_MAX_WIDTH}`, '저장한 폭이 기본 폭보다 우선');

  assert.equal(clampPanelWidth(10), SIDE_PANEL_MIN_WIDTH);
  assert.equal(clampPanelWidth(Number.NaN), 360);
});
