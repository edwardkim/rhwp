// 관계형 계약: 실제 drag 종료 함수가 Native의 도착 소유 경로를 소비한다.
// 합성 API/좌표 대역의 동작 검사이며 Native/WASM·CDP 시각 검증을 대신하지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

async function withHandler(run: (finish: Function) => void, staleOwner = false) {
  let substitutions = 0;
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
    plugins: staleOwner ? [{
      name: 'negative-stale-picture-owner', enforce: 'pre',
      transform(code: string, id: string) {
        if (id.includes('/input-handler-picture.ts')) {
          const changed = code.replace(/cellPath\s*:\s*r\.cellPath/g, 'cellPath: hit.cellPath');
          if (changed !== code) substitutions++;
          return changed;
        }
      },
    }] : [],
  });
  try {
    const { finishPictureMoveDrag } = await vite.ssrLoadModule(
      '/src/engine/input-handler-picture.ts' + (staleOwner ? '?stale-owner-negative' : ''),
    );
    if (staleOwner) assert.ok(substitutions > 0, '음성 대조의 잘못된 소비 경로가 실제 주입됨');
    run(finishPictureMoveDrag);
  } finally { await vite.close(); }
}

function exercise(finish: Function, nested = false) {
  const actual = [{ controlIndex: 0, cellIndex: 119, cellParaIndex: 0 }];
  const stale = [{ ...actual[0], controlIndex: 1 }];
  if (nested) {
    actual.push({ controlIndex: 2, cellIndex: 3, cellParaIndex: 0 });
    stale.push({ controlIndex: 2, cellIndex: 3, cellParaIndex: 0 });
  }
  const source = { type: 'image', secIdx: 0, paraIdx: 4, controlIdx: 0, x: 100, y: 110 };
  const destination = { ...source, cellPath: actual, x: 220, y: 240 };
  const cellWrites: unknown[] = [];
  const snapshots: string[] = [];
  let selected: unknown;
  const props = { horzOffset: 10, vertOffset: 20 };
  const wasm = {
    getPageControlLayout(page: number) { return { controls: [page === 0 ? source : destination] }; },
    hitTest() { return { sectionIndex: 0, parentParaIndex: 4, charOffset: 0, cellPath: stale }; },
    getPictureProperties() { return props; },
    setPictureProperties() {},
    relocateFloatingPicture() { return { ok: true, sec: 0, paraIdx: 4, controlIdx: 0, cellPath: actual }; },
    getCellPicturePropertiesByPath(_sec: number, _para: number, path: unknown) {
      assert.deepEqual(path, actual, '속성 조회도 실제 소유 경로'); return props;
    },
    setCellPicturePropertiesByPath(_sec: number, _para: number, path: unknown) { cellWrites.push(path); },
  };
  const context = {
    wasm, dragRafId: 0, isPictureMoveDragging: true,
    pictureMoveState: {
      ref: { sec: 0, ppi: 4, ci: 0, type: 'image' },
      pageIndex: 0, dropPage: 1, dropX: 200, dropY: 200,
      totalDeltaH: 100, totalDeltaV: 200,
      origHorzOffset: 10, origVertOffset: 20,
    },
    container: { style: { cursor: 'move' }, querySelector() { return { clientWidth: 800 }; } },
    viewportManager: { getZoom() { return 1; } },
    virtualScroll: { getPageLeftResolved() { return 0; }, getPageOffset(page: number) { return page * 1000; } },
    cursor: {
      getPosition() { return { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 }; },
      exitPictureObjectSelection() {},
      enterPictureObjectSelectionDirect(...args: unknown[]) { selected = args[8]; },
    },
    executeOperation(op: any) { snapshots.push(op.operationType); op.operation(wasm); },
    renderPictureObjectSelection() {}, eventBus: { emit() {} },
  };
  finish.call(context);
  assert.deepEqual(selected, actual, '선택은 재번호 뒤 실제 도착 소유 경로');
  assert.deepEqual(cellWrites, [actual], '위치 보정도 같은 소유 경로');
  assert.deepEqual(snapshots, ['relocatePicture']);
  assert.equal(context.pictureMoveState, null);
  assert.equal(context.isPictureMoveDragging, false);
}

test('다른 쪽 그림 이동은 재번호된 칸 경로로 위치와 선택을 함께 갱신한다', async () => {
  await withHandler(finish => { exercise(finish); exercise(finish, true); });
});

test('이전 hit-test 경로를 재사용하는 최소 잘못된 구현을 실제 동작 검사가 검출한다', async () => {
  await withHandler(finish => assert.throws(() => exercise(finish), /선택은 재번호 뒤/), true);
});
