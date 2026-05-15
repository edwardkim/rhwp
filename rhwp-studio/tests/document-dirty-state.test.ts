import test from 'node:test';
import assert from 'node:assert/strict';

import { EventBus } from '../src/core/event-bus.ts';
import { DocumentDirtyState, type DirtyStateChange } from '../src/core/document-dirty-state.ts';

type Listener = (event: unknown) => unknown;

class FakeWindow {
  private listeners = new Map<string, Set<Listener>>();

  addEventListener(type: string, listener: Listener): void {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set());
    }
    this.listeners.get(type)!.add(listener);
  }

  removeEventListener(type: string, listener: Listener): void {
    this.listeners.get(type)?.delete(listener);
  }

  dispatch(type: string, event: unknown): void {
    for (const listener of this.listeners.get(type) ?? []) {
      listener(event);
    }
  }

  listenerCount(type: string): number {
    return this.listeners.get(type)?.size ?? 0;
  }
}

function createBeforeUnloadEvent() {
  return {
    defaultPrevented: false,
    returnValue: undefined as string | undefined,
    preventDefault() {
      this.defaultPrevented = true;
    },
  };
}

test('DocumentDirtyState는 dirty/clean 전환 시 변경 이벤트를 한 번씩 발행한다', () => {
  const eventBus = new EventBus();
  const changes: DirtyStateChange[] = [];
  eventBus.on('document-dirty-changed', (payload) => changes.push(payload as DirtyStateChange));

  const state = new DocumentDirtyState(eventBus);

  assert.equal(state.isDirty(), false);

  state.markDirty('typing');
  state.markDirty('typing-again');
  state.markClean('save');
  state.markClean('save-again');

  assert.deepEqual(changes, [
    { dirty: true, reason: 'typing' },
    { dirty: false, reason: 'save' },
  ]);
  assert.equal(state.isDirty(), false);
});

test('DocumentDirtyState beforeunload는 dirty 상태에서만 페이지 이탈을 막는다', () => {
  const state = new DocumentDirtyState(new EventBus());
  const fakeWindow = new FakeWindow();

  state.installBeforeUnload(fakeWindow as unknown as Window);

  const cleanEvent = createBeforeUnloadEvent();
  fakeWindow.dispatch('beforeunload', cleanEvent);
  assert.equal(cleanEvent.defaultPrevented, false);
  assert.equal(cleanEvent.returnValue, undefined);

  state.markDirty('typing');
  const dirtyEvent = createBeforeUnloadEvent();
  fakeWindow.dispatch('beforeunload', dirtyEvent);
  assert.equal(dirtyEvent.defaultPrevented, true);
  assert.equal(dirtyEvent.returnValue, '');

  state.markClean('save');
  const savedEvent = createBeforeUnloadEvent();
  fakeWindow.dispatch('beforeunload', savedEvent);
  assert.equal(savedEvent.defaultPrevented, false);
  assert.equal(savedEvent.returnValue, undefined);
});

test('DocumentDirtyState beforeunload 해제 함수는 설치한 핸들러만 제거한다', () => {
  const state = new DocumentDirtyState(new EventBus());
  const fakeWindow = new FakeWindow();

  const uninstall = state.installBeforeUnload(fakeWindow as unknown as Window);
  assert.equal(fakeWindow.listenerCount('beforeunload'), 1);

  uninstall();
  assert.equal(fakeWindow.listenerCount('beforeunload'), 0);

  state.markDirty('typing');
  const event = createBeforeUnloadEvent();
  fakeWindow.dispatch('beforeunload', event);
  assert.equal(event.defaultPrevented, false);
});
