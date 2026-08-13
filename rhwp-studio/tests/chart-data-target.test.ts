import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildChartEdits,
  cellInputIssue,
  chartTargetFromSelection,
  hasAnyEdit,
  labelsEditable,
  matchChartRef,
  type ChartDataResult,
  type ChartRefJson,
} from '../src/core/chart-data-target.ts';

// listCharts() 가 돌려주는 wire 형태 그대로의 목록 —
// 계약은 tests/issue_4694_chart_list.rs(코어)가 고정한다.
const CHARTS: ChartRefJson[] = [
  // 0: 본문 직속 (sec 0, para 4, ctrl 2)
  { index: 0, section: 0, paragraph: 4, control: 2, zipPart: 3, nestedCopy: 5 },
  // 1: 표 셀 안 (루트 para 7 의 ctrl 0 표 → cell 1 → 셀 para 0 → ctrl 0)
  {
    index: 1, section: 0, paragraph: 7, control: 0,
    container: [{ kind: 'tableCell', control: 0, paragraph: 0, cell: 1 }],
    nestedCopy: 6,
  },
  // 2: 머리말 안 (루트 para 0 의 ctrl 1 머리말 → 머리말 para 0 → ctrl 0)
  {
    index: 2, section: 0, paragraph: 0, control: 0,
    container: [{ kind: 'header', control: 1, paragraph: 0 }],
    nestedCopy: 7,
  },
  // 3: 글상자 안 — studio 선택 주소가 실측되지 않은 컨테이너
  {
    index: 3, section: 0, paragraph: 9, control: 0,
    container: [{ kind: 'textbox', control: 3, paragraph: 0 }],
    nestedCopy: 8,
  },
];

test('본문 직속 차트는 (sec, ppi, ci) 3좌표로 맞는다', () => {
  const hit = matchChartRef(CHARTS, { sec: 0, ppi: 4, ci: 2 });
  assert.equal(hit?.index, 0);
});

test('좌표 하나라도 다르면 본문 직속 매칭은 없다', () => {
  assert.equal(matchChartRef(CHARTS, { sec: 0, ppi: 4, ci: 1 }), null);
  assert.equal(matchChartRef(CHARTS, { sec: 1, ppi: 4, ci: 2 }), null);
  assert.equal(matchChartRef(CHARTS, { sec: 0, ppi: 5, ci: 2 }), null);
});

test('표 셀 안 차트는 cellPath 로 맞는다 — 두 키 철자 모두', () => {
  // CellPathEntry 철자 (getSelectedPictureRef 가 주는 형태)
  const viaEntry = matchChartRef(CHARTS, {
    sec: 0, ppi: 7, ci: 0,
    cellPath: [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }],
  });
  assert.equal(viaEntry?.index, 1);
  // CellPathSegment 철자 (wasm by_path API 형태)
  const viaSegment = matchChartRef(CHARTS, {
    sec: 0, ppi: 7, ci: 0,
    cellPath: [{ controlIdx: 0, cellIdx: 1, cellParaIdx: 0 }],
  });
  assert.equal(viaSegment?.index, 1);
});

test('cellPath 의 셀 좌표가 다르면 매칭은 없다', () => {
  assert.equal(
    matchChartRef(CHARTS, {
      sec: 0, ppi: 7, ci: 0,
      cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }],
    }),
    null,
  );
});

test('cellPath 깊이가 container 깊이와 다르면 매칭은 없다', () => {
  assert.equal(
    matchChartRef(CHARTS, {
      sec: 0, ppi: 7, ci: 0,
      cellPath: [
        { controlIndex: 0, cellIndex: 1, cellParaIndex: 0 },
        { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
      ],
    }),
    null,
  );
});

test('중첩 표(2단 cellPath)는 container 두 단계와 전 원소 대조로 맞는다', () => {
  const nested: ChartRefJson[] = [{
    index: 0, section: 0, paragraph: 2, control: 1,
    container: [
      { kind: 'tableCell', control: 0, paragraph: 3, cell: 2 },
      { kind: 'tableCell', control: 1, paragraph: 0, cell: 0 },
    ],
    nestedCopy: 4,
  }];
  const hit = matchChartRef(nested, {
    sec: 0, ppi: 2, ci: 1,
    cellPath: [
      { controlIndex: 0, cellIndex: 2, cellParaIndex: 3 },
      { controlIndex: 1, cellIndex: 0, cellParaIndex: 0 },
    ],
  });
  assert.equal(hit?.index, 0);
});

test('머리말 안 차트는 headerFooter + (내부 ppi, ci) 로 맞는다', () => {
  const hit = matchChartRef(CHARTS, {
    sec: 0, ppi: 0, ci: 0,
    headerFooter: { kind: 'header', outerParaIdx: 0, outerControlIdx: 1 },
  });
  assert.equal(hit?.index, 2);
});

test('kind 가 다른 headerFooter 는 맞지 않는다', () => {
  assert.equal(
    matchChartRef(CHARTS, {
      sec: 0, ppi: 0, ci: 0,
      headerFooter: { kind: 'footer', outerParaIdx: 0, outerControlIdx: 1 },
    }),
    null,
  );
});

test('컨테이너 안 차트는 맨 좌표(3인자)로는 절대 맞지 않는다 — 오매칭 방지', () => {
  // 글상자 차트(3)의 루트 좌표를 그대로 넣어도 본문 직속으로 오인하지 않는다.
  assert.equal(matchChartRef(CHARTS, { sec: 0, ppi: 9, ci: 0 }), null);
  // 표 셀 차트(1)의 루트 좌표도 마찬가지.
  assert.equal(matchChartRef(CHARTS, { sec: 0, ppi: 7, ci: 0 }), null);
});

test('cellPath 가 tableCell 아닌 컨테이너 단계와 겹치면 맞지 않는다', () => {
  assert.equal(
    matchChartRef(CHARTS, {
      sec: 0, ppi: 9, ci: 0,
      cellPath: [{ controlIndex: 3, cellIndex: 0, cellParaIndex: 0 }],
    }),
    null,
  );
});

// ── 선택 ref → 매처 입력 정규화 (#4694 S4) ────────────────

test('본문 직속 ole 선택은 3좌표만 남긴다', () => {
  assert.deepEqual(
    chartTargetFromSelection({ sec: 0, ppi: 4, ci: 2, type: 'ole' }),
    { sec: 0, ppi: 4, ci: 2, cellPath: undefined, headerFooter: undefined },
  );
});

test('명시 cellPath 는 그대로 통과한다', () => {
  const cellPath = [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }];
  const target = chartTargetFromSelection({ sec: 0, ppi: 7, ci: 0, type: 'ole', cellPath });
  assert.deepEqual(target?.cellPath, cellPath);
});

test('셀 문맥 3종이 다 있으면 한 단계 cellPath 를 조립한다 — picture-props 선례', () => {
  const target = chartTargetFromSelection({
    sec: 0, ppi: 7, ci: 0, type: 'ole',
    cellIdx: 1, cellParaIdx: 0, outerTableControlIdx: 0,
  });
  assert.deepEqual(target?.cellPath, [{ controlIdx: 0, cellIdx: 1, cellParaIdx: 0 }]);
});

test('셀 문맥이 불완전하면 cellPath 를 조립하지 않는다', () => {
  const target = chartTargetFromSelection({
    sec: 0, ppi: 7, ci: 0, type: 'ole', cellIdx: 1, cellParaIdx: 0,
  });
  assert.equal(target?.cellPath, undefined);
});

test('headerFooter 는 그대로 통과한다', () => {
  const headerFooter = { kind: 'header' as const, outerParaIdx: 0, outerControlIdx: 1 };
  const target = chartTargetFromSelection({ sec: 0, ppi: 0, ci: 0, type: 'ole', headerFooter });
  assert.deepEqual(target?.headerFooter, headerFooter);
});

test('각주/미주 선택(noteRef)과 비-ole 타입은 대상이 아니다 — 안전 축소', () => {
  assert.equal(
    chartTargetFromSelection({ sec: 0, ppi: 1, ci: 0, type: 'ole', noteRef: { kind: 'footnote' } }),
    null,
  );
  assert.equal(chartTargetFromSelection({ sec: 0, ppi: 4, ci: 2, type: 'image' }), null);
});

// ── 편집 페이로드 로직 (#4694 S3) ──────────────────────────

const CATEGORY_DATA: ChartDataResult = {
  ok: true,
  chart: 1,
  axis: 'category',
  labelsShared: true,
  labelsMultiLevel: false,
  labels: ['항목 1', '항목 2'],
  // "4.30" — 정규화하면 무편집 왕복이 깨지는 표기를 일부러 둔다.
  series: [
    { name: '계열 1', values: ['4.30', '2.5'] },
    { name: null, values: ['1', ''] },
  ],
};

const SCATTER_DATA: ChartDataResult = {
  ...CATEGORY_DATA,
  axis: 'scatter',
  labels: ['0.7', '1.8'],
};

test('셀 입력 검증 — 빈 값과 비수치는 사유가 다르다 (코어 dryRun 이 최종 판정)', () => {
  assert.equal(cellInputIssue('4.3'), null);
  assert.equal(cellInputIssue('-0.5'), null);
  assert.equal(cellInputIssue(''), 'empty');
  assert.equal(cellInputIssue('  '), 'empty');
  assert.equal(cellInputIssue('abc'), 'notANumber');
  assert.equal(cellInputIssue('1e999'), 'notANumber'); // 비유한 — 코어 is_number 와 동형
});

test('라벨 열은 분산형 공유 X축일 때만 편집 가능하다', () => {
  assert.equal(labelsEditable(SCATTER_DATA), true);
  assert.equal(labelsEditable(CATEGORY_DATA), false);
  assert.equal(labelsEditable({ ...SCATTER_DATA, labelsShared: false }), false);
  assert.equal(labelsEditable({ ...SCATTER_DATA, labelsMultiLevel: true }), false);
});

test('페이로드는 미변경 셀 문자열을 원본 그대로 싣고 name 을 싣지 않는다', () => {
  const edits = buildChartEdits(CATEGORY_DATA, [
    ['4.30', '2.5'], // 무변경 — "4.3" 으로 정규화되면 안 된다
    ['9', ''],       // 첫 값만 편집, 빈 값(결측)은 그대로
  ]);
  assert.deepEqual(edits.series, [
    { values: ['4.30', '2.5'] },
    { values: ['9', ''] },
  ]);
  assert.equal('labels' in edits, false, '라벨 미편집이면 labels 키가 없어야 한다');
  for (const s of edits.series) {
    assert.equal('name' in s, false, 'name 은 싣지 않는다 — c:tx 부재 대조 함정');
  }
});

test('라벨 페이로드는 분산형에서 실제로 바뀌었을 때만 실린다', () => {
  const grid: string[][] = [['4.30', '2.5'], ['1', '']];
  const unchanged = buildChartEdits(SCATTER_DATA, grid, ['0.7', '1.8']);
  assert.equal('labels' in unchanged, false, '라벨이 원본과 같으면 싣지 않는다');
  const changed = buildChartEdits(SCATTER_DATA, grid, ['0.9', '1.8']);
  assert.deepEqual(changed.labels, ['0.9', '1.8']);
});

test('hasAnyEdit — 값·라벨 어느 쪽 변경도 감지하고, 무변경이면 false', () => {
  const same: string[][] = [['4.30', '2.5'], ['1', '']];
  assert.equal(hasAnyEdit(CATEGORY_DATA, same), false);
  assert.equal(hasAnyEdit(CATEGORY_DATA, [['4.3', '2.5'], ['1', '']]), true); // 표기 변경도 편집이다
  assert.equal(hasAnyEdit(SCATTER_DATA, same, ['0.7', '1.8']), false);
  assert.equal(hasAnyEdit(SCATTER_DATA, same, ['0.9', '1.8']), true);
});
