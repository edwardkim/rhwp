// [Task #2759] 행위 러너 — command.ts 의 커맨드 클래스를 실제 로드해 execute/undo 를
// mock WasmBridge 로 검증한다. cursor.ts:85 의 TS 파라미터 프로퍼티 때문에 기본 test
// 러너(strip-only)는 엔진 클래스를 import 하지 못하므로, 이 러너를 부모 테스트가
// --experimental-transform-types 로 spawn 한다(별도 PR #2720 방식). 실패 시 비정상 종료.
import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import assert from 'node:assert/strict';

const srcDir = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'src');

// @/ 별칭 + 확장자 없는 상대 import 를 .ts 로 해석(tsconfig paths 재현).
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) {
      const abs = join(srcDir, specifier.slice(2));
      const withTs = abs.endsWith('.ts') ? abs : abs + '.ts';
      return { url: pathToFileURL(withTs).href, shortCircuit: true };
    }
    if ((specifier.startsWith('./') || specifier.startsWith('../')) && !/\.[cm]?[tj]s$/.test(specifier)) {
      const parent = context.parentURL ? dirname(fileURLToPath(context.parentURL)) : srcDir;
      return { url: pathToFileURL(join(parent, specifier + '.ts')).href, shortCircuit: true };
    }
    return nextResolve(specifier, context);
  },
});

const { MoveLineEndpointCommand, ResizeObjectCommand } =
  await import(pathToFileURL(join(srcDir, 'engine', 'command.ts')).href);

// ── MoveLineEndpointCommand: 끝점 이동 undo/redo ────────────────────────────
{
  let last = null;
  const wasm = { moveLineEndpoint: (...a) => { last = a; } };
  const before = { sx: 100, sy: 200, ex: 300, ey: 400 };
  const after = { sx: 150, sy: 200, ex: 300, ey: 999 };
  const cmd = new MoveLineEndpointCommand(2, 3, 4, before, after);

  cmd.execute(wasm); // redo 경로: after 재적용
  assert.deepEqual(last, [2, 3, 4, 150, 200, 300, 999], 'execute 는 after 끝점을 적용해야 함');

  cmd.undo(wasm); // before 복원
  assert.deepEqual(last, [2, 3, 4, 100, 200, 300, 400], 'undo 는 before 끝점을 복원해야 함');
  assert.equal(cmd.type, 'moveLineEndpoint');
  assert.equal(cmd.mergeWith(), null, '병합 없음(각 드래그는 독립 기록)');
}

// ── ResizeObjectCommand: 회전각 before/after(임의 속성 가방) ─────────────────
{
  const applied = [];
  const wasm = { setShapeProperties: (s, p, c, props) => applied.push(props) };
  const cmd = new ResizeObjectCommand([{
    sec: 0, ppi: 1, ci: 2, type: 'shape',
    before: { rotationAngle: 0 }, after: { rotationAngle: 45 },
  }]);

  cmd.execute(wasm); // 회전 드래그 종료 후 redo
  assert.deepEqual(applied.at(-1), { rotationAngle: 45 }, 'execute 는 after 회전각을 적용');

  cmd.undo(wasm);
  assert.deepEqual(applied.at(-1), { rotationAngle: 0 }, 'undo 는 원래 회전각으로 복원');
}

console.log('DRAG_COMMAND_BEHAVIOUR_OK');
