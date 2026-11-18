// 관계형 계약: 실제 drag 종료 함수가 Native의 도착 소유 경로를 소비한다.
// 합성 API/좌표 대역의 동작 검사이며 Native/WASM·CDP 시각 검증을 대신하지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

async function withHandler(run: (finish: Function) => void, mutant: 'owner' | 'preview' | null = null) {
  let substitutions = 0;
  const cacheBase = fileURLToPath(new URL('../../output/studio-test-cache/', import.meta.url));
  await mkdir(cacheBase, { recursive: true });
  const cacheDir = await mkdtemp(join(cacheBase, 'picture-relocation-owner-'));
  let vite: Awaited<ReturnType<typeof createServer>> | undefined;
  try {
    // 동시 테스트와 개발 서버가 같은 deps 캐시를 정리하지 않도록 분리한다.
    vite = await createServer({
      cacheDir,
      root: fileURLToPath(new URL('..', import.meta.url)),
      appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
      plugins: [{
        name: 'handler-ssr-without-client-prebundle',
        configResolved(config) {
          // SSR 동작 검사에 불필요한 브라우저용 SDK 묶음의 지연 쓰기를 막는다.
          config.optimizeDeps.noDiscovery = true;
          config.optimizeDeps.include = [];
        },
      }, ...(mutant ? [{
        name: 'negative-stale-picture-owner', enforce: 'pre',
        transform(code: string, id: string) {
          if (id.includes('/input-handler-picture.ts')) {
            const changed = mutant === 'owner'
              ? code.replace(/cellPath\s*:\s*r\.cellPath/g, 'cellPath: hit.cellPath')
              : code.replace('x: state.dropX - state.grabOffsetX', 'x: find(ref, state.pageIndex).x');
            if (changed !== code) substitutions++;
            return changed;
          }
        },
      }] : [])],
    });
    const { finishPictureMoveDrag } = await vite.ssrLoadModule(
      '/src/engine/input-handler-picture.ts' + (mutant ? '?negative-' + mutant : ''),
    );
    if (mutant) assert.ok(substitutions > 0, '음성 대조의 잘못된 소비 경로가 실제 주입됨');
    run(finishPictureMoveDrag);
  } finally {
    try { await vite?.close(); }
    finally { await rm(cacheDir, { recursive: true, force: true }); }
  }
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
    relocateFloatingPicture(...args: unknown[]) {
      assert.deepEqual(args.slice(-2), [0, 0], '새 컨테이너의 원점을 기준으로 위치를 구한다');
      props.horzOffset = 0; props.vertOffset = 0;
      return { ok: true, sec: 0, paraIdx: 4, controlIdx: 0, cellPath: actual };
    },
    getCellPicturePropertiesByPath(_sec: number, _para: number, path: unknown) {
      assert.deepEqual(path, actual, '속성 조회도 실제 소유 경로'); return props;
    },
    setCellPicturePropertiesByPath(_sec: number, _para: number, path: unknown, _ctrl: number, props: unknown) { cellWrites.push({ path, props }); },
  };
  const context = {
    wasm, dragRafId: 0, isPictureMoveDragging: true,
    pictureMoveState: {
      ref: { sec: 0, ppi: 4, ci: 0, type: 'image' },
      pageIndex: 0, dropPage: 1, dropX: 200, dropY: 200,
      grabOffsetX: 10, grabOffsetY: 15,
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
  assert.deepEqual(cellWrites, [{ path: actual, props: { horzOffset: -2250, vertOffset: -4125 } }], '위치 보정은 실제 소유 경로와 처음 잡은 점을 소비한다');
  assert.deepEqual(snapshots, ['relocatePicture']);
  assert.equal(context.pictureMoveState, null);
  assert.equal(context.isPictureMoveDragging, false);
}

test('다른 쪽 그림 이동은 재번호된 칸 경로로 위치와 선택을 함께 갱신한다', async () => {
  await withHandler(finish => { exercise(finish); exercise(finish, true); });
});

test('이전 hit-test 경로를 재사용하는 최소 잘못된 구현을 실제 동작 검사가 검출한다', async () => {
  await withHandler(finish => assert.throws(() => exercise(finish), /선택은 재번호 뒤/), 'owner');
});

test('칸 제한을 받은 미리보기 좌표를 도착 위치로 쓰는 잘못된 구현을 검출한다', async () => {
  await withHandler(finish => assert.throws(() => exercise(finish), /위치 보정은 실제 소유 경로/), 'preview');
});
