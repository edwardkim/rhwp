import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  excludedCellKey,
  selectCellIndicesInRange,
  paraFormatTargetsForCellBlock,
} from '../src/engine/cell-block-format.ts';

// F5 셀 블록 선택은 cellAnchor/cellFocus 축이라 텍스트 선택(cursor.anchor)을 만들지 않는다.
// 그래서 서식 경로가 getSelectionOrdered() 만 보면 커서가 있는 앵커 셀 하나만 대상이 된다.
// 실측(2x2 표, 전체 셀 선택 후 가운데 정렬): 4칸 중 1칸(앵커)만 변경.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

/** 행 우선으로 배치된 rows x cols 표의 cellIdx -> {row, col} */
function gridCellInfo(cols: number) {
  return (cellIdx: number) => ({ row: Math.floor(cellIdx / cols), col: cellIdx % cols });
}

test('선택 범위에 드는 셀만 고른다', () => {
  // 3x3 표에서 (0,0)~(1,1) 블록
  const picked = selectCellIndicesInRange(
    9, gridCellInfo(3),
    { startRow: 0, startCol: 0, endRow: 1, endCol: 1 },
    new Set(),
  );
  assert.deepEqual(picked, [0, 1, 3, 4]);
});

test('블록 전체 선택은 모든 셀을 고른다', () => {
  const picked = selectCellIndicesInRange(
    4, gridCellInfo(2),
    { startRow: 0, startCol: 0, endRow: 1, endCol: 1 },
    new Set(),
  );
  assert.deepEqual(picked, [0, 1, 2, 3]);
});

test('Ctrl+클릭으로 제외한 셀은 대상에서 빠진다', () => {
  const picked = selectCellIndicesInRange(
    4, gridCellInfo(2),
    { startRow: 0, startCol: 0, endRow: 1, endCol: 1 },
    new Set([excludedCellKey(1, 0)]),
  );
  assert.deepEqual(picked, [0, 1, 3]);
});

test('모든 셀을 Ctrl+클릭으로 제외하면 빈 블록이 된다', () => {
  const picked = selectCellIndicesInRange(
    4, gridCellInfo(2),
    { startRow: 0, startCol: 0, endRow: 1, endCol: 1 },
    new Set([
      excludedCellKey(0, 0), excludedCellKey(0, 1),
      excludedCellKey(1, 0), excludedCellKey(1, 1),
    ]),
  );
  assert.deepEqual(picked, []);
});

test('제외 셀 키 형식은 CursorState 가 조립하는 것과 같은 함수여야 한다', () => {
  // 조립하는 쪽과 조회하는 쪽이 형식을 따로 갖고 있으면 한쪽만 바뀌어도 조회가 조용히
  // 빗나가 제외 셀이 무시된다. 리터럴 재조립을 금지하고 단일 함수를 쓰는지 본다.
  const cursor = source('src/engine/cursor.ts');
  assert.match(cursor, /const key = excludedCellKey\(row, col\)/,
    'CursorState.ctrlToggleCell 이 공유 키 함수를 쓰지 않는다');
  assert.doesNotMatch(cursor, /const key = `\$\{row\},\$\{col\}`/,
    'CursorState 가 제외 셀 키를 리터럴로 재조립한다');
});

test('셀 블록 안 모든 셀의 모든 문단이 문단 서식 대상이 된다', () => {
  const targets = paraFormatTargetsForCellBlock(
    { sec: 0, ppi: 3, ci: 1, cellIndices: [0, 1, 2, 3] },
    () => 1,
  );
  assert.equal(targets.length, 4, '4칸 x 문단 1개 = 대상 4개여야 한다');
  assert.deepEqual(
    targets.map((t) => (t.kind === 'cell' ? t.cellIdx : -1)),
    [0, 1, 2, 3],
  );
  assert.deepEqual(targets[2], {
    kind: 'cell', sec: 0, parentPara: 3, controlIdx: 1, cellIdx: 2, cellParaIdx: 0,
  });
});

test('셀마다 문단 수가 다르면 각 셀의 문단 수만큼 대상이 만들어진다', () => {
  const paraCounts = [1, 3, 2];
  const targets = paraFormatTargetsForCellBlock(
    { sec: 0, ppi: 0, ci: 0, cellIndices: [0, 1, 2] },
    (cellIdx) => paraCounts[cellIdx],
  );
  assert.equal(targets.length, 6);
  assert.deepEqual(
    targets.map((t) => (t.kind === 'cell' ? `${t.cellIdx}:${t.cellParaIdx}` : '')),
    ['0:0', '1:0', '1:1', '1:2', '2:0', '2:1'],
  );
});

test('선택된 셀이 없으면 대상도 없다', () => {
  const targets = paraFormatTargetsForCellBlock(
    { sec: 0, ppi: 0, ci: 0, cellIndices: [] },
    () => 5,
  );
  assert.deepEqual(targets, []);
});

test('문단 서식 대상 산출은 텍스트 선택보다 셀 블록을 먼저 본다', () => {
  // 셀 블록 선택은 getSelectionOrdered() 가 null 이므로, 분기가 뒤에 오면 도달하지 못하고
  // getParaFormatTargetsForRange(pos, pos) 로 떨어져 앵커 셀 하나만 대상이 된다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private getParaFormatTargetsAtCursor()');
  assert.notEqual(start, -1, 'getParaFormatTargetsAtCursor 를 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }', start));

  const blockAt = body.indexOf('this.getSelectedCellBlock()');
  const selAt = body.indexOf('this.cursor.getSelectionOrdered()');
  assert.notEqual(blockAt, -1, '셀 블록 분기가 없다');
  assert.notEqual(selAt, -1, '텍스트 선택 분기가 없다');
  assert.ok(blockAt < selAt, '셀 블록 분기가 텍스트 선택 분기보다 뒤에 있다');
  assert.match(body, /return this\.getParaFormatTargetsForCellBlock\(block\)/,
    '셀 블록 대상 산출을 호출하지 않는다');
});

test('빈 셀 블록 문단 서식은 앵커 셀로 fallback 하지 않는다', () => {
  const ih = source('src/engine/input-handler.ts');
  const blockStart = ih.indexOf('private getSelectedCellBlock()');
  const blockEnd = ih.indexOf('\n  }', blockStart);
  const blockBody = ih.slice(blockStart, blockEnd);
  assert.notEqual(blockStart, -1, 'getSelectedCellBlock 을 찾지 못했다');
  assert.doesNotMatch(blockBody, /if \(cellIndices\.length === 0\) return null;/,
    '모든 셀 제외를 셀 블록 아님으로 바꿔 앵커 셀 fallback을 허용한다');

  const paraStart = ih.indexOf('private getParaFormatTargetsAtCursor()');
  const paraEnd = ih.indexOf('\n  }', paraStart);
  const paraBody = ih.slice(paraStart, paraEnd);
  assert.match(paraBody, /if \(block\) return this\.getParaFormatTargetsForCellBlock\(block\);/,
    '빈 셀 블록을 빈 문단 서식 대상으로 유지하지 않는다');
});

test('글자 서식 대상 판정은 셀 블록 선택도 대상으로 인정한다', () => {
  // hasSelection() 은 anchor/fnAnchor 만 본다. 서식 가드가 그것만 보면 셀 블록 선택에서
  // 통째로 반환돼 굵게/기울임/글꼴 크기가 아무 일도 하지 않는다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private hasFormatTargetSelection()');
  assert.notEqual(start, -1, 'hasFormatTargetSelection 이 없다');
  const body = ih.slice(start, ih.indexOf('\n  }', start));
  assert.match(body, /this\.cursor\.hasSelection\(\)\s*\|\|\s*this\.cursor\.isInCellSelectionMode\(\)/,
    '셀 블록 선택을 서식 대상으로 보지 않는다');
});

test('글자 서식 가드가 모두 셀 블록을 인정하는 판정으로 바뀌어야 한다', () => {
  // 한 곳만 바꾸면 Ctrl+B 는 되는데 글꼴 크기 증감은 안 되는 부분 수정이 된다.
  const ih = source('src/engine/input-handler.ts');
  for (const fn of ['applyToggleFormat', 'adjustFontSize', 'adjustCharRatio', 'adjustCharSpacing']) {
    const start = ih.indexOf(fn + '(');
    assert.notEqual(start, -1, `${fn} 을 찾지 못했다`);
    const body = ih.slice(start, ih.indexOf('\n  }', start));
    assert.match(body, /if \(!this\.hasFormatTargetSelection\(\)\) return;/,
      `${fn} 이 셀 블록 선택을 대상으로 인정하지 않는다`);
  }
});

test('셀 블록 글자 서식은 되돌리기 라우팅(executeOperation)을 거친다', () => {
  // 뮤테이션 표면 원장(mutation-routing-guard)이 세는 사이트이므로 직접 wasm 호출이
  // executeOperation 밖으로 새면 Undo 가 비는 채로 문서만 바뀐다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyCharFormatToCellBlock');
  assert.notEqual(start, -1, 'applyCharFormatToCellBlock 이 없다');
  const body = ih.slice(start, ih.indexOf('\n  }\n', start));
  assert.match(body, /this\.executeOperation\(\{\s*\n?\s*kind: 'snapshot'/,
    '스냅샷 라우팅을 거치지 않는다');
  assert.match(body, /wasm\.applyCharFormatInCell\(/, '셀 글자 서식 적용 호출이 없다');
  assert.match(body, /if \(len <= 0\) continue;/, '빈 문단 건너뛰기가 없다');
});

test('글자 서식 적용 진입점이 셀 블록 분기를 먼저 호출한다', () => {
  // 분기가 없으면 getSelectionOrdered() 가 null 이라 그대로 반환된다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyCharFormat(props');
  assert.notEqual(start, -1, 'applyCharFormat 을 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }', start));
  const blockAt = body.indexOf('this.getSelectedCellBlock()');
  const selAt = body.indexOf('this.cursor.getSelectionOrdered()');
  assert.notEqual(blockAt, -1, '셀 블록 분기가 없다');
  assert.ok(blockAt < selAt, '셀 블록 분기가 텍스트 선택 분기보다 뒤에 있다');
  assert.match(body, /this\.applyCharFormatToCellBlock\(block, props\)/,
    '셀 블록 적용 경로를 호출하지 않는다');
});

test('빈 셀 블록 글자 서식은 history 없이 no-op 이다', () => {
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyCharFormat(props');
  assert.notEqual(start, -1, 'applyCharFormat 을 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }', start));
  assert.match(body, /if \(block\.cellIndices\.length === 0\) return;/,
    '모든 셀 제외 시 글자 서식이 no-op으로 끝나지 않는다');
});

test('[#4151] 토글 방향은 셀 블록 모드에서 블록 앵커 셀의 서식을 읽는다', () => {
  // 커서 위치 조회(getCharPropertiesAtCursor)는 셀 블록 모드에서 블록 밖(호스트 문단 등)을
  // 읽어 방금 적용한 서식이 보이지 않는다 — current[prop] 이 이전 값에 머물러 !current[prop]
  // 이 항상 같은 방향이 되고, 두 번째 클릭이 해제가 아니라 재적용이 된다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyToggleFormat(');
  assert.notEqual(start, -1, 'applyToggleFormat 을 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }\n', start));
  assert.match(body, /this\.getSelectedCellBlock\(\)/,
    '토글 방향 산출에 셀 블록 분기가 없다');
  assert.match(body,
    /\?\s*this\.getCharPropertiesAtCellBlockAnchor\(\w+\)\s*\n?\s*:\s*this\.getCharPropertiesAtCursor\(\)/,
    '셀 블록 모드에서 블록 앵커 셀 대신 커서 위치의 서식으로 토글 방향을 정한다');
});

test('[#4151] 셀 블록 적용 직후 cursor-format-changed 를 앵커 셀 기준으로 방출한다', () => {
  // 텍스트 선택 경로는 적용 후 emitCursorFormatState 로 툴바 눌림 상태를 재조회·방출하지만,
  // 셀 블록 경로는 이 후처리가 없으면 툴바가 이전 상태로 남는다. emitCursorFormatState 는
  // 커서 위치 기준이라 블록에는 오답 — 앵커 셀 서식을 직접 방출해야 한다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyCharFormatToCellBlock');
  assert.notEqual(start, -1, 'applyCharFormatToCellBlock 이 없다');
  const body = ih.slice(start, ih.indexOf('\n  }\n', start));
  const applyAt = body.indexOf('this.executeOperation(');
  const emitAt = body.indexOf("this.eventBus.emit('cursor-format-changed'");
  assert.notEqual(emitAt, -1, '적용 후 cursor-format-changed 방출이 없다');
  assert.ok(applyAt !== -1 && applyAt < emitAt, '상태 방출이 서식 적용보다 앞에 있다');
  assert.match(body,
    /emit\('cursor-format-changed',\s*this\.getCharPropertiesAtCellBlockAnchor\(block\)\)/,
    '방출 값이 블록 앵커 셀 서식이 아니다');
});

test('[#4151] 앵커 셀 서식 기준은 블록 첫 셀의 첫 글자다', () => {
  // 토글 방향(applyToggleFormat)과 툴바 상태 방출이 같은 기준을 공유해야 두 번째 클릭의
  // 해제 방향과 버튼 표시가 일치한다. 기준 조회는 이 함수 한 곳에만 둔다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private getCharPropertiesAtCellBlockAnchor');
  assert.notEqual(start, -1, 'getCharPropertiesAtCellBlockAnchor 가 없다');
  const body = ih.slice(start, ih.indexOf('\n  }\n', start));
  assert.match(body,
    /getCellCharPropertiesAt\(block\.sec, block\.ppi, block\.ci, block\.cellIndices\[0\], 0, 0\)/,
    '앵커 기준이 블록 첫 셀의 첫 글자가 아니다');
});

test('모양 붙여넣기도 같은 셀 산출 함수를 쓴다', () => {
  // 같은 블록을 대상으로 하는 두 경로가 필터를 각자 갖고 있으면 한쪽만 고쳐진다.
  const ih = source('src/engine/input-handler.ts');
  const start = ih.indexOf('private applyCopiedCellPropsToSelection');
  assert.notEqual(start, -1, 'applyCopiedCellPropsToSelection 을 찾지 못했다');
  const body = ih.slice(start, ih.indexOf('\n  }\n', start));
  assert.match(body, /selectCellIndicesInRange\(/,
    '모양 붙여넣기가 셀 산출을 따로 구현하고 있다');
});
