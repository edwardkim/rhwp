import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

// `engine/command.ts` 는 constructor parameter property 를 쓰므로 Node 의 타입 스트리핑으로는
// 못 읽는다(#3230 과 같은 이유·같은 방식으로 vite SSR 로 띄운다).
const studioRoot = fileURLToPath(new URL('..', import.meta.url));
const rootDir = studioRoot;
let SetCellBorderFillCommand: any;

test('모듈 로드', async () => {
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    ({ SetCellBorderFillCommand } = await vite.ssrLoadModule('/src/engine/command.ts'));
  } finally {
    await vite.close();
  }
  assert.equal(typeof SetCellBorderFillCommand, 'function');
});

// [#5959] 셀 테두리/배경을 스냅샷에서 속성쌍 역연산으로.
//
// 변경 실체는 대상+이웃 집단의 border_fill_id 재배정과 스타일 테이블 append 다 —
// Rust 가 self-describing 기록(changes/borderFillLenBefore/docInfoDirtyBefore)을
// 응답으로 주고, undo 는 ① id 직접 대입 ② push 분 꼬리 절단 ③ 구역 raw 복원의
// 3단으로 원상 복구한다(저장 바이트 수렴 실증:
// tests/cases/issue_5959_cell_borderfill_inverse_convergence.rs).
//
// 여기서 고정하는 것은 셋이다 — probe 선차단 배선, 저널 생명주기 순서,
// 스냅샷 예산을 쓰지 않는다는 것.

interface Call { fn: string; args: unknown[] }

function recordingWasm(responses: {
  setCellProperties?: any;
  setCellZoneProperties?: any;
}): { wasm: any; calls: Call[] } {
  const calls: Call[] = [];
  const rec = (fn: string) => (...args: unknown[]) => {
    calls.push({ fn, args });
    return { ok: true };
  };
  return {
    calls,
    wasm: {
      hasCellBorderFillInverse: (() => {
        let called = false;
        return () => { calls.push({ fn: 'hasCellBorderFillInverse', args: [] }); called = true; return true; };
      })(),
      captureSectionRaw: (() => {
        let id = 700;
        return () => { calls.push({ fn: 'captureSectionRaw', args: [] }); return ++id; };
      })(),
      restoreSectionRaw: rec('restoreSectionRaw'),
      discardSectionRaw: rec('discardSectionRaw'),
      runInBatch: (fn: () => unknown) => fn(),
      setCellProperties: (...args: unknown[]) => {
        calls.push({ fn: 'setCellProperties', args });
        return responses.setCellProperties;
      },
      setCellZoneProperties: (...args: unknown[]) => {
        calls.push({ fn: 'setCellZoneProperties', args });
        return responses.setCellZoneProperties;
      },
      applyCellBorderFillIds: rec('applyCellBorderFillIds'),
      removeBorderFillTails: rec('removeBorderFillTails'),
    },
  };
}

test('execute 는 probe → 캡처 순서를 지키고 변경 기록을 모은다', () => {
  const { wasm, calls } = recordingWasm({
    setCellProperties: {
      ok: true,
      changes: [{ cellIdx: 3, beforeId: 2, afterId: 9 }],
      borderFillLenBefore: 9,
      docInfoDirtyBefore: false,
    },
  });
  const cmd = new SetCellBorderFillCommand(
    0, 12, 5, { kind: 'cells', cellIdxes: [1] }, { fillType: 'solid' }, { sectionIndex: 0 },
  );
  cmd.execute(wasm);

  const names = calls.map((c) => c.fn);
  const probeAt = names.indexOf('hasCellBorderFillInverse');
  const captureAt = names.indexOf('captureSectionRaw');
  assert.ok(probeAt !== -1 && captureAt !== -1, 'probe 와 캡처가 있어야 한다');
  assert.ok(probeAt < captureAt, 'probe 가 캡처보다 먼저여야 한다(구버전 wasm 원천 거절)');
  assert.match(names.join(','), /setCellProperties/, '적용이 기록돼야 한다');
  for (const fn of ['saveSnapshot', 'restoreSnapshot']) {
    assert.ok(!names.includes(fn), `${fn} 을 쓰면 스냅샷 회귀다`);
  }
  assert.equal(cmd.snapshotResourceCount(), 0);
  assert.equal(cmd.isNoOp(), false);
});

test('undo 는 id 대입 → 꼬리 절단 → raw 복원 순서다', () => {
  const { wasm, calls } = recordingWasm({
    setCellProperties: {
      ok: true,
      changes: [
        { cellIdx: 1, beforeId: 2, afterId: 10 },
        { cellIdx: 0, beforeId: 3, afterId: 10 },
      ],
      borderFillLenBefore: 9,
      docInfoDirtyBefore: false,
    },
  });
  const cmd = new SetCellBorderFillCommand(
    0, 12, 5, { kind: 'cells', cellIdxes: [1] }, { fillType: 'solid' }, { sectionIndex: 0 },
  );
  cmd.execute(wasm);
  calls.length = 0;

  cmd.undo(wasm);
  const names = calls.map((c) => c.fn);
  const applyAt = names.indexOf('applyCellBorderFillIds');
  const gcAt = names.indexOf('removeBorderFillTails');
  const restoreAt = names.indexOf('restoreSectionRaw');
  assert.ok(applyAt !== -1 && gcAt !== -1 && restoreAt !== -1, '3 단계가 모두 있어야 한다');
  assert.ok(applyAt < gcAt && gcAt < restoreAt, '대입 → 절단 → 복원 순서가 어긋나면 저널 전제가 깨진다');

  const applyCall = calls[applyAt];
  const payload = applyCall.args[3];
  // 이웃 재배정으로 같은 셀이 여러 번 기록돼도 undo 는 원본(beforeId) 하나면 충분하다.
  assert.deepEqual(
    [...payload.cells].sort((a: any, b: any) => a.cellIdx - b.cellIdx),
    [{ cellIdx: 0, id: 3 }, { cellIdx: 1, id: 2 }],
  );
  const gcCall = calls[gcAt];
  assert.equal(gcCall.args[0], 9, 'fromLen');
  assert.equal(gcCall.args[1], false, 'dirtyWas');
});

test('변경이 없으면 no-op 으로 저널을 남기지 않는다', () => {
  const { wasm, calls } = recordingWasm({
    setCellProperties: { ok: true, changes: [], borderFillLenBefore: 9, docInfoDirtyBefore: false },
  });
  const cmd = new SetCellBorderFillCommand(
    0, 12, 5, { kind: 'cells', cellIdxes: [1] }, { fillType: 'solid' }, { sectionIndex: 0 },
  );
  cmd.execute(wasm);
  assert.equal(cmd.isNoOp(), true);
  const names = calls.map((c) => c.fn);
  assert.ok(names.includes('discardSectionRaw'), '캡처를 즉시 해제해야 한다');
  assert.ok(!names.includes('restoreSectionRaw'), 'no-op 에 undo 경로가 남으면 안 된다');
  assert.throws(() => cmd.undo(wasm), /변경 기록이 없다/);
});

test('zone(asOne) 적용도 같은 생명주기를 쓴다', () => {
  const { wasm, calls } = recordingWasm({
    setCellZoneProperties: {
      ok: true, borderFillId: 11, zoneBeforeId: null,
      borderFillLenBefore: 10, docInfoDirtyBefore: true,
    },
  });
  const range = { startRow: 0, startCol: 0, endRow: 1, endCol: 1 };
  const cmd = new SetCellBorderFillCommand(
    0, 12, 5, { kind: 'zone', range }, { fillType: 'none' }, { sectionIndex: 0 },
  );
  cmd.execute(wasm);
  cmd.undo(wasm);

  const zoneUndo = calls.find((c) => c.fn === 'applyCellBorderFillIds');
  assert.ok(zoneUndo, 'zone undo 가 기록돼야 한다');
  assert.deepEqual(zoneUndo.args[3].zones, [{ ...range, id: null }], '신설 zone 은 제거로 되돌린다');
  const gcCall = calls.find((c) => c.fn === 'removeBorderFillTails');
  assert.equal(gcCall.args[0], 10, 'fromLen');
  assert.equal(gcCall.args[1], true, 'dirtyWas');
});

test('소스 핀 — 다이얼로그는 커맨드 경유, 레지스트리는 신규 네이티브를 변이로 분류', () => {
  const dialogSrc = readFileSync(join(rootDir, 'src/ui/cell-border-bg-dialog.ts'), 'utf8');
  assert.match(dialogSrc, /applyCommandThroughRouter/, '다이얼로그가 커맨드 경유여야 한다');
  assert.match(dialogSrc, /new SetCellBorderFillCommand/);
  assert.ok(!dialogSrc.includes("kind: 'snapshot'"), '스냅샷 기록 잔존은 회귀다');

  const registrySrc = readFileSync(join(rootDir, 'src/core/mutation-method-registry.ts'), 'utf8');
  assert.match(registrySrc, /'applyCellBorderFillIds'/);
  assert.match(registrySrc, /'removeBorderFillTails'/);

  const bridgeSrc = readFileSync(join(rootDir, 'src/core/wasm-bridge.ts'), 'utf8');
  assert.match(bridgeSrc, /hasCellBorderFillInverse/, 'probe 가 브리지에 있어야 한다');
});

test('execute 실패가 아무 뮤테이션 전이면 롤백 호출 없이 캡처만 해제한다', () => {
  const calls: Call[] = [];
  const wasm: any = {
    hasCellBorderFillInverse: () => true,
    captureSectionRaw: () => { calls.push({ fn: 'captureSectionRaw', args: [] }); return 801; },
    discardSectionRaw: (...args: unknown[]) => { calls.push({ fn: 'discardSectionRaw', args }); },
    runInBatch: (fn: () => void) => fn(),
    setCellProperties: () => { calls.push({ fn: 'setCellProperties', args: [] }); throw new Error('셀 인덱스 초과'); },
  };
  const cmd = new SetCellBorderFillCommand(
    0, 12, 5, { kind: 'cells', cellIdxes: [5] }, { fillType: 'solid' }, { sectionIndex: 0 },
  );
  assert.throws(() => cmd.execute(wasm), /셀 인덱스 초과/);
  const names = calls.map((c) => c.fn);
  assert.ok(names.includes('discardSectionRaw'), '캡처 해제는 해야 한다');
  assert.ok(!names.includes('applyCellBorderFillIds'), '뮤테이션 전 실패엔 빈 롤백 대입을 낭비하지 않는다');
  assert.ok(!names.includes('removeBorderFillTails'), '빈 절단은 table.dirty 만 세운다');
  assert.throws(() => cmd.undo(wasm), /변경 기록이 없다/);
});

test('source paint만 지워도 no-op이 아니며 ID 0과 배치 역순 복원 기록을 전달한다', () => {
  const { wasm, calls } = recordingWasm({ setCellProperties: {
    ok: true, changes: [], borderPaintUndoId: 0,
    borderFillLenBefore: 9, docInfoDirtyBefore: false,
  } });
  const cmd = new SetCellBorderFillCommand(0, 0, 1,
    { kind: 'cells', cellIdxes: [0] }, { borderRight: { type: 0 } }, { sectionIndex: 0 });
  cmd.execute(wasm);
  assert.equal(cmd.isNoOp(), false, 'ID 0 is a real source-paint change');
  cmd.undo(wasm);
  const payload = calls.find(c => c.fn === 'applyCellBorderFillIds')!.args[3] as any;
  assert.deepEqual(payload.borderPaintUndoIds, [0]);
  assert.equal(cmd.snapshotResourceCount(), 0);
});

test('다중 셀 실패의 rollback도 이미 제거한 source paint를 복원한다', () => {
  const { wasm, calls } = recordingWasm({ setCellProperties: {
    ok: true, changes: [], borderPaintUndoId: 31,
    borderFillLenBefore: 9, docInfoDirtyBefore: false,
  } });
  const first = wasm.setCellProperties;
  let n = 0;
  wasm.setCellProperties = (...args: unknown[]) => {
    if (++n === 2) throw new Error('unsupported shared edge');
    return first(...args);
  };
  const cmd = new SetCellBorderFillCommand(0, 0, 1,
    { kind: 'cells', cellIdxes: [0, 1] }, {}, { sectionIndex: 0 });
  assert.throws(() => cmd.execute(wasm), /unsupported shared edge/);
  const payload = calls.find(c => c.fn === 'applyCellBorderFillIds')!.args[3] as any;
  assert.deepEqual(payload.borderPaintUndoIds, [31]);
  assert.ok(calls.some(c => c.fn === 'restoreSectionRaw'));
});

test('history에서 제거한 command는 source paint 저널을 해제한다', () => {
  const { wasm } = recordingWasm({ setCellProperties: {
    ok: true, changes: [], borderPaintUndoId: 42,
    borderFillLenBefore: 9, docInfoDirtyBefore: false,
  } });
  const ids: number[] = [];
  wasm.discardBorderPaintUndo = (id: number) => ids.push(id);
  const cmd = new SetCellBorderFillCommand(0, 0, 1,
    { kind: 'cells', cellIdxes: [0] }, {}, { sectionIndex: 0 });
  cmd.execute(wasm);
  cmd.discard(wasm);
  cmd.discard(wasm);
  assert.deepEqual(ids, [42]);
});

test('범위 적용은 첫 셀의 미변경 preview 변과 BorderFill ID를 다른 셀에 복사하지 않는다', () => {
  const { wasm, calls } = recordingWasm({ setCellProperties: {
    ok: true, changes: [], borderFillLenBefore: 9, docInfoDirtyBefore: false,
  } });
  const edge = { type: 1, width: 5, color: '#d00000' };
  const cmd = new SetCellBorderFillCommand(0, 0, 1,
    { kind: 'cells', cellIdxes: [0, 1] },
    { borderFillId: 77, borderLeft: edge, borderRight: edge, borderTop: edge,
      borderBottom: edge, borderPaintSides: [false, true, false, false] }, { sectionIndex: 0 });
  cmd.execute(wasm);
  const patches = calls.filter(c => c.fn === 'setCellProperties').map(c => c.args[4]);
  assert.equal(patches.length, 2);
  for (const patch of patches) assert.deepEqual(patch, {
    borderRight: edge, borderPaintSides: [false, true, false, false],
  });
});
