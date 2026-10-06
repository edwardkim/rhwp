import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

async function withJournal(run: (Journal: any) => void) {
  const vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  try {
    const { PictureResizeJournal } = await vite.ssrLoadModule('/src/engine/picture-resize-journal.ts');
    run(PictureResizeJournal);
  } finally { await vite.close(); }
}

function fake() {
  let current = { height: 7296, currentHeight: 7295, raw: 'original-146-bytes' };
  let next = 0;
  const store = new Map<number, typeof current>();
  const refs: unknown[] = [];
  return { store, refs, get current() { return current; },
    capturePictureTransform(ref: unknown) { refs.push(ref); const id = next++; store.set(id, { ...current }); return id; },
    swapPictureTransform(id: number) {
      const saved = store.get(id); assert.ok(saved);
      store.set(id, current); current = saved;
    },
    discardPictureTransform(id: number) { store.delete(id); },
    resize() { current = { height: 7696, currentHeight: 7696, raw: '' }; },
    setPictureProperties() { assert.fail('그림 Undo를 스칼라 setter로 우회하면 안 된다'); },
  };
}

test('실제 resize 뒤 Undo/Redo는 원본 변환을 교환하고 discard로 해제한다', async () => {
  await withJournal(Journal => {
    const wasm = fake();
    const ref = { sec: 0, ppi: 236, ci: 0, type: 'image' };
    const original = { ...wasm.current };
    const journal = Journal.capture(wasm, [ref]);
    wasm.resize();
    const changed = { ...wasm.current };
    const command = journal.command([{ ...ref, before: { height: 7296 }, after: { height: 7696 } }]);
    for (let i = 0; i < 3; i++) {
      command.undo(wasm); assert.deepEqual(wasm.current, original);
      command.execute(wasm); assert.deepEqual(wasm.current, changed);
    }
    assert.equal(command.snapshotResourceCount?.() ?? 0, 0);
    command.discard(wasm); assert.equal(wasm.store.size, 0);
  });
});

test('취소는 변경을 되돌리고 셀·머리말 경로는 캡처에 그대로 전달한다', async () => {
  await withJournal(Journal => {
    for (const location of [
      { cellPath: [{ controlIndex: 2, cellIndex: 0, cellParaIndex: 1 }] },
      { headerFooter: { kind: 'header', outerParaIdx: 3, outerControlIdx: 1 } },
    ]) {
      const wasm = fake();
      const ref = { sec: 0, ppi: 4, ci: 0, type: 'image', ...location };
      const original = { ...wasm.current };
      const journal = Journal.capture(wasm, [ref]);
      assert.deepEqual(wasm.refs[0], ref);
      wasm.resize(); journal.cancel(wasm);
      assert.deepEqual(wasm.current, original);
      assert.equal(wasm.store.size, 0);
    }
  });
});

test('다중 선택 캡처 실패는 앞서 캡처한 핸들을 해제한다', async () => {
  await withJournal(Journal => {
    const wasm = fake();
    const capture = wasm.capturePictureTransform.bind(wasm);
    wasm.capturePictureTransform = ref => {
      if (wasm.store.size) throw new Error('capture failed');
      return capture(ref);
    };
    assert.throws(() => Journal.capture(wasm, [{ type: 'image' }, { type: 'image' }]));
    assert.equal(wasm.store.size, 0);
  });
});
