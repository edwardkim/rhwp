import test from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

// `engine/command.ts` 는 constructor parameter property 를 쓰므로 Node 의 타입 스트리핑으로는
// 못 읽는다(`page-margin-guides.test.ts` 와 같은 이유·같은 방식으로 vite SSR 로 띄운다).
const studioRoot = fileURLToPath(new URL('..', import.meta.url));
let SetObjectPropsCommand: any;
let getObjectProps: any;
let setObjectProps: any;

test('모듈 로드', async () => {
  const vite = await createServer({
    root: studioRoot,
    appType: 'custom',
    logLevel: 'silent',
    server: { middlewareMode: true },
  });
  try {
    ({ SetObjectPropsCommand } = await vite.ssrLoadModule('/src/engine/command.ts'));
    ({ getObjectProps, setObjectProps } = await vite.ssrLoadModule('/src/engine/object-props.ts'));
  } finally {
    await vite.close();
  }
  assert.equal(typeof SetObjectPropsCommand, 'function');
});

// [#3230] 개체 속성 변경을 스냅샷에서 역연산으로.
//
// 스냅샷 1개는 `Document` 통째 클론이라 문서에 비례한다 — 실측으로 30KB 공문 0.43 MB,
// 10MB 행정편람 10.59 MB. `SnapshotCommand` 는 before/after 로 2개를 쓰므로 회전 한 번이
// 최대 21 MB 다. 실제로 되돌려야 하는 것은 스칼라 속성 하나뿐이다.
//
// 여기서 고정하는 것은 둘이다 — 역연산이 왕복하는가, 그리고 스냅샷 예산을 쓰지 않는가.

interface Call { fn: string; args: unknown[] }

function recordingWasm(): { wasm: any; calls: Call[] } {
  const calls: Call[] = [];
  const rec = (fn: string) => (...args: unknown[]) => {
    calls.push({ fn, args });
    return { ok: true };
  };
  return {
    calls,
    wasm: {
      setShapeProperties: rec('setShapeProperties'),
      setPictureProperties: rec('setPictureProperties'),
      setCellShapePropertiesByPath: rec('setCellShapePropertiesByPath'),
      setCellPicturePropertiesByPath: rec('setCellPicturePropertiesByPath'),
      setHeaderFooterPictureProperties: rec('setHeaderFooterPictureProperties'),
      getShapeProperties: rec('getShapeProperties'),
      getPictureProperties: rec('getPictureProperties'),
      getCellShapePropertiesByPath: rec('getCellShapePropertiesByPath'),
      getCellPicturePropertiesByPath: rec('getCellPicturePropertiesByPath'),
      getHeaderFooterPictureProperties: rec('getHeaderFooterPictureProperties'),
    },
  };
}

const bodyImage = { sec: 0, ppi: 3, ci: 1, type: 'image' as const };

test('[#3230] undo 는 적용 전 값으로, redo 는 적용 값으로 되돌린다', () => {
  const { wasm, calls } = recordingWasm();
  const cmd = new SetObjectPropsCommand(bodyImage, { rotationAngle: 0 }, { rotationAngle: 15 });

  const undoPos = cmd.undo(wasm);
  assert.deepEqual(calls.at(-1), {
    fn: 'setPictureProperties',
    args: [0, 3, 1, { rotationAngle: 0 }],
  });
  assert.deepEqual(undoPos, { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 });

  cmd.execute(wasm);
  assert.deepEqual(calls.at(-1), {
    fn: 'setPictureProperties',
    args: [0, 3, 1, { rotationAngle: 15 }],
  });
});

test('[#3230] 스냅샷 예산을 쓰지 않는다', () => {
  const cmd = new SetObjectPropsCommand(bodyImage, { horzFlip: false }, { horzFlip: true }) as any;
  // 히스토리는 `cmd.snapshotResourceCount?.() ?? 0` 로 예산을 센다(engine/history.ts).
  assert.equal(
    cmd.snapshotResourceCount?.() ?? 0,
    0,
    '역연산 커맨드가 예산을 쓰면 스냅샷에서 옮긴 의미가 없다',
  );
  assert.equal(cmd.discard, undefined, '해제할 WASM 스냅샷 id 가 없어야 한다');
});

test('[#3230] 회전 15° 를 네 번 눌러도 병합하지 않는다', () => {
  const cmd = new SetObjectPropsCommand(bodyImage, { rotationAngle: 0 }, { rotationAngle: 15 });
  assert.equal(
    cmd.mergeWith(),
    null,
    '한컴도 누른 횟수만큼 되돌린다 — 묶으면 되돌리기 단위가 어긋난다',
  );
});

test('[#3230] 적용과 되돌리기는 같은 개체를 만진다 — 셀 안', () => {
  const { wasm, calls } = recordingWasm();
  const cellPath = [{ controlIndex: 2, cellIndex: 5, cellParaIndex: 0 }];
  const ref = { sec: 0, ppi: 7, ci: 0, type: 'shape' as const, cellPath };
  const cmd = new SetObjectPropsCommand(ref, { rotationAngle: 90 }, { rotationAngle: 180 });

  cmd.execute(wasm);
  cmd.undo(wasm);

  assert.deepEqual(calls.map((c) => c.fn), [
    'setCellShapePropertiesByPath',
    'setCellShapePropertiesByPath',
  ], '셀 안 도형은 양방향 모두 by-path setter 여야 한다');
  assert.deepEqual(calls[0].args, [0, 7, cellPath, 0, { rotationAngle: 180 }]);
  assert.deepEqual(calls[1].args, [0, 7, cellPath, 0, { rotationAngle: 90 }]);
});

test('[#3230] 적용과 되돌리기는 같은 개체를 만진다 — 머리말/꼬리말', () => {
  const { wasm, calls } = recordingWasm();
  const ref = {
    sec: 0,
    ppi: 1,
    ci: 0,
    type: 'image' as const,
    headerFooter: { kind: 'header' as const, outerParaIdx: 4, outerControlIdx: 2 },
  };
  const cmd = new SetObjectPropsCommand(ref, { vertFlip: false }, { vertFlip: true });

  cmd.execute(wasm);
  cmd.undo(wasm);

  // [Task #831] HF 그림이 본문 lookup 으로 떨어지면 조용히 아무것도 바뀌지 않는다.
  assert.deepEqual(calls.map((c) => c.fn), [
    'setHeaderFooterPictureProperties',
    'setHeaderFooterPictureProperties',
  ]);
  assert.deepEqual(calls[0].args, [0, 4, 2, 1, 0, { vertFlip: true }]);
  assert.deepEqual(calls[1].args, [0, 4, 2, 1, 0, { vertFlip: false }]);
});

test('[#3230] 조회와 적용이 같은 분기를 쓴다', () => {
  // before 를 읽은 경로와 되돌릴 때 쓰는 경로가 갈리면 되돌리기가 다른 개체를 만진다.
  const cases: Array<[any, string, string]> = [
    [{ sec: 0, ppi: 0, ci: 0, type: 'shape' }, 'getShapeProperties', 'setShapeProperties'],
    [{ sec: 0, ppi: 0, ci: 0, type: 'image' }, 'getPictureProperties', 'setPictureProperties'],
    [
      { sec: 0, ppi: 0, ci: 0, type: 'image', headerFooter: { kind: 'footer', outerParaIdx: 1, outerControlIdx: 0 } },
      'getHeaderFooterPictureProperties',
      'setHeaderFooterPictureProperties',
    ],
    [
      { sec: 0, ppi: 0, ci: 0, type: 'image', cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }] },
      'getCellPicturePropertiesByPath',
      'setCellPicturePropertiesByPath',
    ],
  ];
  for (const [ref, getFn, setFn] of cases) {
    const { wasm, calls } = recordingWasm();
    getObjectProps(wasm, ref);
    setObjectProps(wasm, ref, { rotationAngle: 10 });
    assert.deepEqual(calls.map((c) => c.fn), [getFn, setFn], `${ref.type} 분기 불일치`);
    // locator 인자(속성 bag 앞부분)가 양쪽에서 같아야 한다.
    assert.deepEqual(calls[0].args, calls[1].args.slice(0, -1));
  }
});
