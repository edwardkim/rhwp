import test from 'node:test';
import assert from 'node:assert/strict';

import { DocumentDirtyState } from '../src/core/document-dirty-state.ts';
import { EventBus } from '../src/core/event-bus.ts';
import { AutosaveManager, type AutosaveStoreLike } from '../src/recovery/autosave-manager.ts';
import type { AutosaveDraft } from '../src/recovery/autosave-store.ts';

function tick(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function createStore() {
  const saved: AutosaveDraft[] = [];
  const deleted: string[] = [];
  const store: AutosaveStoreLike = {
    async saveDraft(draft) {
      saved.push({ ...draft, data: new Uint8Array(draft.data) });
    },
    async deleteDraft(id) {
      deleted.push(id);
    },
  };
  return { store, saved, deleted };
}

test('AutosaveManager는 dirty 이벤트 후 현재 문서를 draft로 저장한다', async () => {
  const { store, saved } = createStore();
  const eventBus = new EventBus();
  const manager = new AutosaveManager({
    exportBytes: () => new Uint8Array([1, 2, 3, 4]),
    debounceMs: 0,
    minSaveIntervalMs: 0,
    now: () => 1_000,
    idFactory: () => 'draft-a',
    store,
    logger: { debug() {}, warn() {} },
  });

  manager.connect(eventBus);
  await manager.beginDocument({ fileName: 'a.hwp', sourceFormat: 'hwp' });
  eventBus.emit('document-mutated', 'typing');
  await tick();

  assert.equal(saved.length, 1);
  assert.equal(saved[0].id, 'draft-a');
  assert.equal(saved[0].fileName, 'a.hwp');
  assert.equal(saved[0].sourceFormat, 'hwp');
  assert.equal(saved[0].savedAt, 1_000);
  assert.equal(saved[0].dirtyReason, 'typing');
  assert.deepEqual([...saved[0].data], [1, 2, 3, 4]);
});

test('AutosaveManager는 clean 전환 시 현재 draft를 삭제한다', async () => {
  const { store, saved, deleted } = createStore();
  const eventBus = new EventBus();
  const dirtyState = new DocumentDirtyState(eventBus);
  const manager = new AutosaveManager({
    exportBytes: () => new Uint8Array([5]),
    debounceMs: 0,
    minSaveIntervalMs: 0,
    idFactory: () => 'draft-clean',
    store,
    logger: { debug() {}, warn() {} },
  });

  manager.connect(eventBus);
  await manager.beginDocument({ fileName: 'clean.hwp', sourceFormat: 'hwp' });
  dirtyState.markDirty('typing');
  await tick();
  assert.equal(saved.length, 1);

  dirtyState.markClean('save');
  await tick();
  assert.deepEqual(deleted, ['draft-clean']);
});

test('AutosaveManager는 새 문서 세션 시작 시 이전 draft를 정리하고 새 id를 사용한다', async () => {
  const { store, saved, deleted } = createStore();
  let nextId = 0;
  const manager = new AutosaveManager({
    exportBytes: () => new Uint8Array([9]),
    debounceMs: 0,
    minSaveIntervalMs: 0,
    idFactory: () => `draft-${++nextId}`,
    store,
    logger: { debug() {}, warn() {} },
  });

  await manager.beginDocument({ fileName: 'old.hwp', sourceFormat: 'hwp' });
  await manager.flushNow('typing');
  assert.equal(saved[0].id, 'draft-1');

  await manager.beginDocument(
    { fileName: 'new.hwp', sourceFormat: 'hwp' },
    { discardPreviousDraft: true },
  );
  await manager.flushNow('typing');

  assert.deepEqual(deleted, ['draft-1']);
  assert.equal(saved[1].id, 'draft-2');
  assert.equal(saved[1].fileName, 'new.hwp');
});
