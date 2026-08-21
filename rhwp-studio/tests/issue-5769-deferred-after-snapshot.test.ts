import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#5769] after 스냅샷 지연 저장 — undo 스택 엔트리를 2슬롯에서 1슬롯으로 줄인다.
//
// 히스토리 불변식상 undo 스택 top 의 after 상태는 "지금 문서" 와 같다. 그래서 after 는
// 실행 시점이 아니라 undo 시점에 찍어도 값이 같고, 그 사이에는 들고 있을 이유가 없다.
// 예산 98 기준 최악 undo 깊이가 49 → 98 로 배가된다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

/** 스냅샷 id 발급·해제를 세는 최소 wasm 대역. */
function fakeWasm() {
  let next = 1;
  const live = new Set<number>();
  const log: string[] = [];
  return {
    live,
    log,
    saveSnapshot() {
      const id = next++;
      live.add(id);
      log.push(`save:${id}`);
      return id;
    },
    restoreSnapshot(id: number) {
      assert.ok(live.has(id), `해제된 스냅샷 ${id} 복원 시도 — orphan`);
      log.push(`restore:${id}`);
    },
    discardSnapshot(id: number) {
      assert.ok(live.has(id), `이미 해제된 스냅샷 ${id} 재해제`);
      live.delete(id);
      log.push(`discard:${id}`);
    },
  };
}

const pos = (o: number) => ({ sectionIndex: 0, paragraphIndex: 0, charOffset: o });

async function load() {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  const mod = await vite.ssrLoadModule('/src/engine/command.ts');
  return { vite, SnapshotCommand: mod.SnapshotCommand };
}

test('[#5769] 실행 직후 엔트리는 1슬롯, undo 하면 2슬롯, redo 하면 다시 1슬롯', async () => {
  const { vite, SnapshotCommand } = await load();
  try {
    const w = fakeWasm();
    const cmd: any = new SnapshotCommand('paste', pos(0), pos(5), () => pos(5));

    cmd.execute(w);
    assert.equal(cmd.snapshotResourceCount(), 1, '실행 직후는 before 하나뿐');
    assert.equal(w.live.size, 1);

    cmd.undo(w);
    assert.equal(cmd.snapshotResourceCount(), 2, 'undo 가 after 를 잡는다(redo 대비)');

    cmd.execute(w);
    assert.equal(cmd.snapshotResourceCount(), 1, 'redo 뒤 after 는 반환한다');
    assert.equal(w.live.size, 1, '살아있는 스냅샷도 하나로 돌아온다');

    // 왕복을 반복해도 슬롯이 늘지 않는다(누수 없음).
    for (let i = 0; i < 5; i++) { cmd.undo(w); cmd.execute(w); }
    assert.equal(cmd.snapshotResourceCount(), 1, '왕복 반복에도 1슬롯 유지');
    assert.equal(w.live.size, 1, '왕복 반복에도 누수 없음');
  } finally { await vite.close(); }
});

test('[#5769] after 는 before 복원 **전에** 찍는다', async () => {
  const { vite, SnapshotCommand } = await load();
  try {
    const w = fakeWasm();
    const cmd: any = new SnapshotCommand('paste', pos(0), pos(5), () => pos(5));
    cmd.execute(w);
    const beforeId = 1;
    w.log.length = 0;
    cmd.undo(w);
    const idxSave = w.log.findIndex((e) => e.startsWith('save:'));
    const idxRestore = w.log.indexOf(`restore:${beforeId}`);
    assert.ok(idxSave !== -1 && idxRestore !== -1 && idxSave < idxRestore,
      'before 를 먼저 복원하면 after 로 before 상태가 찍힌다');
  } finally { await vite.close(); }
});

test('[#5769] after 저장이 실패해도 undo 는 수행되고 redo 만 포기한다', async () => {
  const { vite, SnapshotCommand } = await load();
  try {
    const w = fakeWasm();
    const cmd: any = new SnapshotCommand('paste', pos(0), pos(5), () => pos(5));
    cmd.execute(w);

    const realSave = w.saveSnapshot.bind(w);
    w.saveSnapshot = () => { throw new Error('OOM'); };
    const at = cmd.undo(w);
    assert.deepEqual(at, pos(0), '저장 실패와 무관하게 되돌리기는 수행된다');
    assert.equal(cmd.snapshotResourceCount(), 1, 'after 는 없다');

    w.saveSnapshot = realSave;
    assert.throws(() => cmd.execute(w), /redo 불가/,
      'redo 는 성공한 척하지 말고 던져야 한다 — 히스토리가 엔트리를 드롭한다');
  } finally { await vite.close(); }
});

test('[#5769] 무변경 연산은 종전대로 아무 스냅샷도 남기지 않는다', async () => {
  const { vite, SnapshotCommand } = await load();
  try {
    const w = fakeWasm();
    const cmd: any = new SnapshotCommand('noop', pos(0), pos(0), () => null);
    cmd.execute(w);
    assert.equal(cmd.isNoOp(), true, '#2370 무변경 신호 유지');
    assert.equal(w.live.size, 0, '무변경이면 before 도 즉시 해제');
    assert.equal(cmd.snapshotResourceCount(), 0);
  } finally { await vite.close(); }
});
