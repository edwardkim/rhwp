import test from 'node:test';
import assert from 'node:assert/strict';

import {
  getSelectionRectsInCellByPathWithPageHints,
  getSelectionRectsInCellWithPageHints,
  type CellSelectionRectDocument,
  type CellSelectionRectQuery,
  type PathCellSelectionRectDocument,
  type PathCellSelectionRectQuery,
  type SelectionPageHints,
} from '../src/core/selection-page-hints.ts';

const query: CellSelectionRectQuery = {
  sectionIdx: 0,
  parentParaIdx: 0,
  controlIdx: 2,
  cellIdx: 2,
  startCellParaIdx: 1250,
  startCharOffset: 0,
  endCellParaIdx: 1275,
  endCharOffset: 1,
};

function createDocument() {
  const calls: Array<{ kind: 'positional'; args: number[] } | { kind: 'ex'; options: unknown }> = [];
  const doc: CellSelectionRectDocument = {
    getSelectionRectsInCell(...args) {
      calls.push({ kind: 'positional', args });
      return '[{"pageIndex":54,"x":1,"y":2,"width":3,"height":4}]';
    },
    getSelectionRectsInCellEx(optionsJson) {
      calls.push({ kind: 'ex', options: JSON.parse(optionsJson) });
      return '[{"pageIndex":55,"x":5,"y":6,"width":7,"height":8}]';
    },
  };
  return { doc, calls };
}

test('두 endpoint page hint가 있으면 Ex options로 조회한다', () => {
  const { doc, calls } = createDocument();
  const rects = getSelectionRectsInCellWithPageHints(doc, query, {
    startPageHint: 54,
    endPageHint: 55,
  });

  assert.deepEqual(rects, [{ pageIndex: 55, x: 5, y: 6, width: 7, height: 8 }]);
  assert.deepEqual(calls, [{
    kind: 'ex',
    options: {
      ...query,
      startPageHint: 54,
      endPageHint: 55,
    },
  }]);
});

test('page hint가 없거나 불완전하면 positional API를 유지한다', () => {
  for (const hints of [
    undefined,
    { startPageHint: 54 } as SelectionPageHints,
    { startPageHint: -1, endPageHint: 55 },
  ]) {
    const { doc, calls } = createDocument();
    const rects = getSelectionRectsInCellWithPageHints(doc, query, hints);

    assert.deepEqual(rects, [{ pageIndex: 54, x: 1, y: 2, width: 3, height: 4 }]);
    assert.deepEqual(calls, [{
      kind: 'positional',
      args: [0, 0, 2, 2, 1250, 0, 1275, 1],
    }]);
  }
});

test('구버전 WASM에 Ex가 없으면 hints가 있어도 positional로 복구한다', () => {
  const calls: number[][] = [];
  const doc: CellSelectionRectDocument = {
    getSelectionRectsInCell(...args) {
      calls.push(args);
      return '[]';
    },
  };

  assert.deepEqual(
    getSelectionRectsInCellWithPageHints(doc, query, {
      startPageHint: 54,
      endPageHint: 55,
    }),
    [],
  );
  assert.deepEqual(calls, [[0, 0, 2, 2, 1250, 0, 1275, 1]]);
});

const pathQuery: PathCellSelectionRectQuery = {
  sectionIdx: 0,
  parentParaIdx: 7,
  path: JSON.stringify([
    { controlIndex: 1, cellIndex: 0, cellParaIndex: 0 },
    { controlIndex: 2, cellIndex: 0, cellParaIndex: 12 },
    { controlIndex: 0, cellIndex: 50, cellParaIndex: 0 },
  ]),
  startCellParaIdx: 0,
  startCharOffset: 0,
  endCellParaIdx: 0,
  endCharOffset: 5,
};

test('중첩 셀은 전체 path와 endpoint page hint를 Ex options로 전달한다', () => {
  const calls: unknown[] = [];
  const doc: PathCellSelectionRectDocument = {
    getSelectionRectsInCellByPath() {
      throw new Error('hinted path에서 positional API를 호출하면 안 됨');
    },
    getSelectionRectsInCellByPathEx(optionsJson) {
      calls.push(JSON.parse(optionsJson));
      return '[{"pageIndex":4,"x":10,"y":20,"width":30,"height":12}]';
    },
  };

  const rects = getSelectionRectsInCellByPathWithPageHints(doc, pathQuery, {
    startPageHint: 4,
    endPageHint: 4,
  });

  assert.equal(rects.length, 1);
  assert.deepEqual(calls, [{ ...pathQuery, startPageHint: 4, endPageHint: 4 }]);
});

test('중첩 셀 page hint가 없으면 같은 path positional API를 유지한다', () => {
  const calls: unknown[][] = [];
  const doc: PathCellSelectionRectDocument = {
    getSelectionRectsInCellByPath(...args) {
      calls.push(args);
      return '[]';
    },
  };

  assert.deepEqual(getSelectionRectsInCellByPathWithPageHints(doc, pathQuery), []);
  assert.deepEqual(calls, [[0, 7, pathQuery.path, 0, 0, 0, 5]]);
});
