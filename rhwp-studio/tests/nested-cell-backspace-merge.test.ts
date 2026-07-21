import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

// [#2717] 중첩 표 안쪽 셀 문단 시작 Backspace 의 "축 정합" 행위 가드.
//
// hit-test 는 flat 필드(controlIndex/cellIndex/cellParaIndex)를 cellPath[0], 즉 **최외곽**
// 엔트리에서 채운다(cursor_rect.rs 의 `outer = &ctx.path[0]`). 따라서 중첩 셀에서
// pos.cellParaIndex 는 바깥 셀의 문단 인덱스이고, 안쪽 셀 값은 cellPath[last].cellParaIndex 다
// (command.ts 의 cellParaIndexOf 주석 / tests/undo-nested-cell-merge-offset.test.ts).
//
// handleBackspace 가 "셀 문단 시작에서 이전 문단과 병합" 여부를 flat 으로 판정하면
//   A. 안쪽 셀 2번째 문단 시작(바깥은 0) → 병합이 통째로 누락돼 Backspace 무반응
//   B. 안쪽 셀 첫 문단(바깥은 ≥1)      → 병합 대상이 없는데 실행돼 cellParaIndex:-1 경로 호출
// 두 방향 모두 어긋난다. 바로 아래 형제 handleDelete(:307 useCellPath)는 이미 안쪽 축이다.
//
// 검증 방식: 소스 복제 없이 실제 input-handler-text.ts 를 로드해 executeOperation/wasm 호출을
// 캡처한다. command.ts 의 parameter property 때문에 Node 의 strip-only 로더로는 직접 import 가
// 불가하므로(`TypeScript parameter property is not supported in strip-only mode`),
// process.execPath 자식 프로세스를 --experimental-transform-types 로 띄우고
// module.registerHooks 로 `@/` 별칭만 매핑한다. node_modules/.bin 실행 파일에 의존하지 않아
// 플랫폼 무관하게 동작한다.

const studioRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const workDir = mkdtempSync(path.join(tmpdir(), 'rhwp-nested-backspace-'));

const stubPath = path.join(workDir, 'confirm-dialog-stub.mjs');
writeFileSync(stubPath, 'export function showConfirm() { return Promise.resolve(false); }\n');

const driverPath = path.join(workDir, 'driver.mjs');
writeFileSync(driverPath, `
import { registerHooks } from 'node:module';
import { pathToFileURL } from 'node:url';

const srcRoot = ${JSON.stringify(pathToFileURL(path.join(studioRoot, 'src') + path.sep).href)};
const stubUrl = ${JSON.stringify(pathToFileURL(stubPath).href)};

registerHooks({
  resolve(specifier, context, nextResolve) {
    // 별칭 중 유일한 값(런타임) import 인 confirm-dialog 는 스텁으로 — 모달은 이 시나리오 밖.
    if (specifier === '@/ui/confirm-dialog') return { url: stubUrl, shortCircuit: true };
    if (specifier.startsWith('@/')) return nextResolve(srcRoot + specifier.slice(2) + '.ts', context);
    // 스튜디오 소스의 확장자 없는 상대 import (번들러 관례) 보정.
    if (/^\\.{1,2}\\//.test(specifier) && !/\\.[a-z]+$/.test(specifier)) {
      return nextResolve(specifier + '.ts', context);
    }
    return nextResolve(specifier, context);
  },
});

const mod = await import(srcRoot + 'engine/input-handler-text.ts');

/** executeOperation 과 wasm 호출을 모두 기록하는 최소 InputHandler 대역. */
function makeHost() {
  const ops = [];
  const wasmCalls = [];
  const base = {
    // 필드(누름틀) 경계 검사는 이 시나리오와 무관 — 호출부가 try/catch 로 감싼다.
    getFieldInfoAt() { throw new Error('no field'); },
    getCellParagraphLengthByPath: () => 7,
    getCellParagraphCountByPath: () => 3,
    getCellParagraphLength: () => 7,
    getCellParagraphCount: () => 3,
    mergeParagraphInCellByPath: () => undefined,
    splitParagraphInCellByPath: () => undefined,
    mergeParagraphInCell: () => undefined,
    splitParagraphInCell: () => undefined,
  };
  const wasm = new Proxy(base, {
    get(target, prop) {
      const value = target[prop];
      if (typeof value !== 'function') return value;
      return (...args) => { wasmCalls.push([prop, ...args]); return value(...args); };
    },
  });
  return {
    ops,
    wasmCalls,
    host: {
      cursor: { isInHeaderFooter: () => false },
      wasm,
      executeOperation(op) { ops.push(op); },
    },
  };
}

function nestedPos(outerPara, innerPara) {
  // 바깥 표(cellIndex 0)의 outerPara 번째 문단에 중첩 표가 있고,
  // 캐럿은 안쪽 표(cellIndex 2)의 innerPara 번째 문단 맨 앞.
  return {
    sectionIndex: 0, paragraphIndex: innerPara, charOffset: 0,
    parentParaIndex: 3, controlIndex: 0, cellIndex: 0, cellParaIndex: outerPara,
    cellPath: [
      { controlIndex: 0, cellIndex: 0, cellParaIndex: outerPara },
      { controlIndex: 0, cellIndex: 2, cellParaIndex: innerPara },
    ],
  };
}

function flatPos(cellPara) {
  return {
    sectionIndex: 0, paragraphIndex: cellPara, charOffset: 0,
    parentParaIndex: 3, controlIndex: 0, cellIndex: 2, cellParaIndex: cellPara,
    cellPath: [{ controlIndex: 0, cellIndex: 2, cellParaIndex: cellPara }],
  };
}

function backspace(pos) {
  const h = makeHost();
  mod.handleBackspace.call(h.host, pos, true);
  return h;
}

const result = {};

// A. 중첩 안쪽 셀 2번째 문단 시작 — 병합돼야 한다.
result.nestedInnerSecondPara = backspace(nestedPos(0, 1)).ops.map((o) => o.command?.type ?? o.operationType);

// A-대조. 같은 위치의 handleDelete 는 안쪽 축(...ByPath)으로 조회한다.
{
  const h = makeHost();
  mod.handleDelete.call(h.host, nestedPos(0, 1), true);
  result.deleteSiblingWasmCalls = h.wasmCalls.map((c) => c[0]);
}

// B. 중첩 안쪽 셀 첫 문단 시작(바깥은 2) — 병합 대상이 없으므로 무동작이어야 한다.
{
  const h = backspace(nestedPos(2, 0));
  result.nestedInnerFirstPara = h.ops.map((o) => o.command?.type ?? o.operationType);
  result.nestedInnerFirstParaPaths = [];
  for (const op of h.ops) {
    const exec = makeHost();
    try { op.command.execute(exec.host.wasm); } catch { /* 경로만 본다 */ }
    for (const call of exec.wasmCalls) result.nestedInnerFirstParaPaths.push(call[3]);
  }
}

// C. 비중첩(depth 1) 회귀 대조군 — 현행 동작이 그대로여야 한다.
result.flatSecondPara = backspace(flatPos(1)).ops.map((o) => o.command?.type ?? o.operationType);
result.flatFirstPara = backspace(flatPos(0)).ops.map((o) => o.command?.type ?? o.operationType);

process.stdout.write('###' + JSON.stringify(result) + '###');
`);

const run = spawnSync(
  process.execPath,
  ['--experimental-transform-types', '--no-warnings', driverPath],
  { cwd: studioRoot, encoding: 'utf8' },
);

rmSync(workDir, { recursive: true, force: true });

assert.equal(
  run.status,
  0,
  `handleBackspace 행위 드라이버 실행 실패:\n${run.stdout}\n${run.stderr}`,
);

const captured = /###([\s\S]*)###/.exec(run.stdout);
assert.ok(captured, `드라이버 출력에서 결과 JSON 을 찾지 못함:\n${run.stdout}\n${run.stderr}`);
const observed = JSON.parse(captured[1]) as {
  nestedInnerSecondPara: string[];
  deleteSiblingWasmCalls: string[];
  nestedInnerFirstPara: string[];
  nestedInnerFirstParaPaths: string[];
  flatSecondPara: string[];
  flatFirstPara: string[];
};

test('중첩 셀 안쪽 2번째 문단 시작 Backspace 는 이전 문단과 병합한다', () => {
  assert.deepEqual(
    observed.nestedInnerSecondPara,
    ['mergeParagraphInCell'],
    'flat cellParaIndex(바깥 셀=0)로 판정하면 병합이 통째로 누락돼 Backspace 가 무반응이 된다',
  );
});

test('같은 위치의 handleDelete 는 이미 안쪽 축(...ByPath)으로 조회한다(대조군)', () => {
  assert.ok(
    observed.deleteSiblingWasmCalls.includes('getCellParagraphLengthByPath'),
    `handleDelete 는 cellPath 축 조회를 써야 한다: ${observed.deleteSiblingWasmCalls.join(', ')}`,
  );
});

test('중첩 셀 안쪽 첫 문단 시작 Backspace 는 셀 안에서 병합하지 않는다', () => {
  assert.deepEqual(
    observed.nestedInnerFirstPara,
    [],
    'flat(바깥 셀=2)로 판정하면 병합 대상이 없는데도 병합이 실행된다',
  );
  const negative = observed.nestedInnerFirstParaPaths.filter((p) => typeof p === 'string' && p.includes('-1'));
  assert.deepEqual(
    negative,
    [],
    `cellParaIndex 가 음수인 cellPath 로 WASM 을 호출하면 안 된다(json_usize 파싱 실패): ${negative.join(' / ')}`,
  );
});

test('비중첩(depth 1) 셀 동작은 불변이다', () => {
  assert.deepEqual(observed.flatSecondPara, ['mergeParagraphInCell'], 'depth 1 2번째 문단은 종전대로 병합');
  assert.deepEqual(observed.flatFirstPara, [], 'depth 1 첫 문단은 종전대로 무동작');
});
