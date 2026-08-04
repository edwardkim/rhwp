import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

// [#2756] 중첩 표 안쪽 셀 선택 영역 정렬의 "축 정합" 행위 가드.
//
// hit-test 는 flat 필드(controlIndex/cellIndex/cellParaIndex)를 cellPath[0], 즉 **최외곽**
// 엔트리에서 채운다(cursor_rect.rs 의 `outer = &ctx.path[0]`). 따라서 중첩 셀에서
// pos.cellParaIndex 는 바깥 셀의 문단 인덱스이고, 안쪽 셀 값은 cellPath[last].cellParaIndex 다
// (command.ts 의 cellParaIndexOf 주석 / tests/undo-nested-cell-merge-offset.test.ts).
//
// CursorState.comparePositions 가 flat cellIndex/cellParaIndex 를 맨몸으로 비교하면
//   A. 안쪽 셀의 서로 다른 문단(바깥 문단은 동일) → 문단 비교가 통과돼 charOffset 으로 낙하,
//      선택 양끝이 뒤바뀐다. 소비자 DeleteSelectionCommand 는 cellParaIndexOf(안쪽 축)로
//      start/end 를 읽으므로 startPara > endPara → savedTexts 가 비고, Rust
//      delete_range_in_cell_by_path 는 선택 범위의 **여집합**을 지운다. undo 는 무효.
//   B. 안쪽 셀이 서로 다름(바깥 셀 식별자는 동일) → flat cellIndex 가 같아 정렬이 붕괴.
// 두 방향 모두 어긋난다.
//
// 검증 방식: 소스 복제 없이 실제 cursor.ts 를 로드해 comparePositions/getSelectionOrdered 를
// 직접 호출한다. CursorState 는 constructor(private wasm) parameter property 때문에 Node 의
// strip-only 로더로는 직접 import 가 불가하므로(`TypeScript parameter property is not supported
// in strip-only mode`), process.execPath 자식 프로세스를 --experimental-transform-types 로
// 띄우고 module.registerHooks 로 `@/` 별칭과 확장자 없는 상대 import 를 매핑한다.
// node_modules/.bin 실행 파일에 의존하지 않아 플랫폼 무관하게 동작한다.

const studioRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const workDir = mkdtempSync(path.join(tmpdir(), 'rhwp-sel-order-'));

const driverPath = path.join(workDir, 'driver.mjs');
writeFileSync(driverPath, `
import { registerHooks } from 'node:module';
import { pathToFileURL } from 'node:url';

const srcRoot = ${JSON.stringify(pathToFileURL(path.join(studioRoot, 'src') + path.sep).href)};

registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier.startsWith('@/')) return nextResolve(srcRoot + specifier.slice(2) + '.ts', context);
    // 스튜디오 소스의 확장자 없는 상대 import (번들러 관례) 보정.
    if (/^\\.{1,2}\\//.test(specifier) && !/\\.[a-z]+$/.test(specifier)) {
      return nextResolve(specifier + '.ts', context);
    }
    return nextResolve(specifier, context);
  },
});

const { CursorState } = await import(srcRoot + 'engine/cursor.ts');

const R = { pageIndex: 0, x: 0, y: 0, height: 10 };

// 안쪽 셀(cellIndex 2)의 문단 para, 오프셋 off. 바깥 표 cellIndex 0 의 문단 outerPara 에 중첩.
// flat 필드는 hit-test 규약대로 **바깥** 엔트리(cellPath[0]) 값으로 채운다.
function innerPos(outerPara, innerCell, innerPara, off) {
  return {
    sectionIndex: 0, paragraphIndex: innerPara, charOffset: off,
    parentParaIndex: 3, controlIndex: 0, cellIndex: 0, cellParaIndex: outerPara,
    cellPath: [
      { controlIndex: 0, cellIndex: 0, cellParaIndex: outerPara },
      { controlIndex: 0, cellIndex: innerCell, cellParaIndex: innerPara },
    ],
    cursorRect: R,
  };
}

// depth 1 (비중첩) 셀 위치 — flat 과 cellPath[0] 이 곧 최내곽으로 일치.
function flatCellPos(cellPara, off) {
  return {
    sectionIndex: 0, paragraphIndex: cellPara, charOffset: off,
    parentParaIndex: 3, controlIndex: 0, cellIndex: 2, cellParaIndex: cellPara,
    cellPath: [{ controlIndex: 0, cellIndex: 2, cellParaIndex: cellPara }],
    cursorRect: R,
  };
}

function bodyPos(para, off) {
  return { sectionIndex: 0, paragraphIndex: para, charOffset: off, cursorRect: R };
}

const cmp = (a, b) => CursorState.comparePositions(a, b);
const sign = (n) => (n < 0 ? -1 : n > 0 ? 1 : 0);

// getSelectionOrdered 실경로 — moveToHit(cursorRect 有 → updateRect 미호출) + setAnchor.
function orderedInner(anchor, focus) {
  const cs = new CursorState({});
  cs.moveToHit(anchor);
  cs.setAnchor();
  cs.moveToHit(focus);
  const sel = cs.getSelectionOrdered();
  const innerPara = (p) => p.cellPath[p.cellPath.length - 1].cellParaIndex;
  const innerCell = (p) => p.cellPath[p.cellPath.length - 1].cellIndex;
  return {
    startInnerPara: innerPara(sel.start), endInnerPara: innerPara(sel.end),
    startInnerCell: innerCell(sel.start), endInnerCell: innerCell(sel.end),
    startOff: sel.start.charOffset, endOff: sel.end.charOffset,
  };
}

const result = {};

// A. 같은 안쪽 셀, 안쪽 문단 0 끝(off 5) → 문단 1 시작(off 0). 바깥 문단은 둘 다 0.
//    올바르면 cmp<0 (anchor<focus), 뒤바뀌면 cmp>0.
const aAnchor = innerPos(0, 2, 0, 5);
const aFocus  = innerPos(0, 2, 1, 0);
result.sameCellCmp = sign(cmp(aAnchor, aFocus));
result.sameCellOrdered = orderedInner(aAnchor, aFocus);

// B. 서로 다른 안쪽 셀(2 vs 4), 바깥 셀 식별자는 동일. 올바르면 cellIndex 2<4 로 cmp<0.
const bAnchor = innerPos(0, 2, 1, 3);
const bFocus  = innerPos(0, 4, 0, 0);
result.diffInnerCellCmp = sign(cmp(bAnchor, bFocus));

// C. depth 1 회귀 대조군 — 문단 0 off5 vs 문단 1 off0. 현행 동작 불변(cmp<0).
result.depth1Cmp = sign(cmp(flatCellPos(0, 5), flatCellPos(1, 0)));

// D. 본문 회귀 대조군 — 문단 2 off0 vs 문단 1 off9. cmp>0 (2>1).
result.bodyCmp = sign(cmp(bodyPos(2, 0), bodyPos(1, 9)));

// E. 본문↔셀 혼합 회귀 대조군 — 본문 문단 3 vs 같은 문단의 셀(parentPara 3). 셀이 뒤 → cmp<0.
result.bodyVsCellCmp = sign(cmp(bodyPos(3, 0), innerPos(0, 2, 0, 0)));

// F. 반사성/대칭성 — cmp(a,b) === -cmp(b,a), cmp(a,a)===0.
result.antisymmetry = sign(cmp(aAnchor, aFocus)) === -sign(cmp(aFocus, aAnchor));
result.reflexivity = sign(cmp(aAnchor, { ...aAnchor })) === 0;

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
  `comparePositions 행위 드라이버 실행 실패:\n${run.stdout}\n${run.stderr}`,
);

const captured = /###([\s\S]*)###/.exec(run.stdout);
assert.ok(captured, `드라이버 출력에서 결과 JSON 을 찾지 못함:\n${run.stdout}\n${run.stderr}`);
const observed = JSON.parse(captured[1]) as {
  sameCellCmp: number;
  sameCellOrdered: {
    startInnerPara: number; endInnerPara: number;
    startInnerCell: number; endInnerCell: number;
    startOff: number; endOff: number;
  };
  diffInnerCellCmp: number;
  depth1Cmp: number;
  bodyCmp: number;
  bodyVsCellCmp: number;
  antisymmetry: boolean;
  reflexivity: boolean;
};

test('중첩 안쪽 셀의 문단 0 끝 → 문단 1 시작 선택이 뒤바뀌지 않는다 (데이터 손실 방지)', () => {
  assert.equal(
    observed.sameCellCmp,
    -1,
    'flat cellParaIndex(바깥=0) 로 비교하면 두 문단이 같아 보여 charOffset(5>0)으로 낙하, 양끝이 뒤바뀐다',
  );
  // 정렬 결과의 안쪽 축이 start <= end 여야 한다 (DeleteSelectionCommand 가 읽는 축).
  assert.ok(
    observed.sameCellOrdered.startInnerPara <= observed.sameCellOrdered.endInnerPara,
    `정렬된 start 의 안쪽 문단(${observed.sameCellOrdered.startInnerPara})이 end(${observed.sameCellOrdered.endInnerPara})보다 뒤면 ` +
    'startPara>endPara 로 savedTexts 가 비고 삭제가 여집합을 지운다',
  );
  assert.deepEqual(
    { p: observed.sameCellOrdered.startInnerPara, off: observed.sameCellOrdered.startOff },
    { p: 0, off: 5 },
    'start 는 문단 0 끝(anchor)이어야 한다',
  );
  assert.deepEqual(
    { p: observed.sameCellOrdered.endInnerPara, off: observed.sameCellOrdered.endOff },
    { p: 1, off: 0 },
    'end 는 문단 1 시작(focus)이어야 한다',
  );
});

test('서로 다른 안쪽 셀은 경로의 안쪽 cellIndex 로 정렬한다', () => {
  assert.equal(
    observed.diffInnerCellCmp,
    -1,
    'flat cellIndex 는 둘 다 바깥 셀(0)이라 같다 — 안쪽 cellPath[last].cellIndex(2<4)로 정렬해야 한다',
  );
});

test('depth 1 (비중첩) 셀 정렬은 불변이다 (회귀 대조군)', () => {
  assert.equal(observed.depth1Cmp, -1, 'depth 1 은 cellPath[0]=최내곽=flat 이라 종전대로 문단 0<1');
});

test('본문 정렬은 불변이다 (회귀 대조군)', () => {
  assert.equal(observed.bodyCmp, 1, '본문 문단 2>1');
});

test('본문↔셀 혼합 정렬은 불변이다 (회귀 대조군)', () => {
  assert.equal(observed.bodyVsCellCmp, -1, '같은 문단에서 셀은 본문보다 뒤로 간주');
});

test('comparePositions 는 반대칭·반사적이다', () => {
  assert.ok(observed.antisymmetry, 'cmp(a,b) === -cmp(b,a)');
  assert.ok(observed.reflexivity, 'cmp(a,a) === 0');
});
