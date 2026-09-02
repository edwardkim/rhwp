import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildChartEdits,
  cellInputIssue,
  chartTargetFromSelection,
  hasAnyEdit,
  labelsEditable,
  labelsStructurallyEditable,
  matchChartRef,
  needsStructure,
  unsafeTextIssue,
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

test('글상자 안 차트는 sentinel cellPath(cellIdx 0, #1171 계약)로 textbox 와 맞는다', () => {
  // 코어 레이아웃이 글상자 ole 에 [{controlIndex: 글상자ci, cellIndex: 0,
  // cellParaIndex: 내부문단}] sentinel 을 방출한다(#4694 R1 정공법).
  const hit = matchChartRef(CHARTS, {
    sec: 0, ppi: 9, ci: 0,
    cellPath: [{ controlIndex: 3, cellIndex: 0, cellParaIndex: 0 }],
  });
  assert.equal(hit?.index, 3);
});

test('sentinel 아닌 cellIdx 는 textbox 컨테이너와 맞지 않는다', () => {
  assert.equal(
    matchChartRef(CHARTS, {
      sec: 0, ppi: 9, ci: 0,
      cellPath: [{ controlIndex: 3, cellIndex: 1, cellParaIndex: 0 }],
    }),
    null,
  );
});

test('맨 좌표 대조는 컨테이너 차트의 루트 좌표와 겹치면 모호로 보고 거부한다', () => {
  // 머리말 ole 는 레이아웃이 컨테이너 문맥을 아직 싣지 않아 맨 3좌표로 선택된다 —
  // 그 좌표가 본문 직속 차트와 겹치면 어느 쪽을 클릭했는지 구분할 수 없다.
  // 오매칭(다른 차트 편집)이 최악이므로 거부한다.
  const shadowed: ChartRefJson[] = [
    { index: 0, section: 0, paragraph: 0, control: 0, nestedCopy: 1 },
    {
      index: 1, section: 0, paragraph: 0, control: 0,
      container: [{ kind: 'header', control: 1, paragraph: 0 }],
      nestedCopy: 2,
    },
  ];
  assert.equal(matchChartRef(shadowed, { sec: 0, ppi: 0, ci: 0 }), null);
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

// ── 구조 편집 (#6053 S1) ──────────────────────────────────

const SAME_GRID: string[][] = [['4.30', '2.5'], ['1', '']];

test('계열명·라벨 텍스트 검증 — 코어 is_safe_text 와 같은 문자 집합', () => {
  assert.equal(unsafeTextIssue('계열 1'), null);
  assert.equal(unsafeTextIssue(''), null, '빈 이름은 안전한 텍스트다 — 별개로 코어가 판정한다');
  assert.equal(unsafeTextIssue('a < b'), 'unsafeText');
  assert.equal(unsafeTextIssue('a > b'), 'unsafeText');
  assert.equal(unsafeTextIssue('AT&T'), 'unsafeText');
  assert.equal(unsafeTextIssue('줄\n바꿈'), 'unsafeText', '제어문자도 거부 대상이다');
});

test('구조용 라벨 판정은 카테고리도 연다 — B1 판정(labelsEditable)은 그대로 둔다', () => {
  assert.equal(labelsStructurallyEditable(CATEGORY_DATA), true);
  assert.equal(labelsEditable(CATEGORY_DATA), false, 'B1 술어는 한 글자도 안 바뀐다');
  assert.equal(labelsStructurallyEditable(SCATTER_DATA), true);
  assert.equal(labelsStructurallyEditable({ ...CATEGORY_DATA, labelsShared: false }), false);
  assert.equal(labelsStructurallyEditable({ ...CATEGORY_DATA, labelsMultiLevel: true }), false);
  assert.equal(labelsStructurallyEditable({ ...CATEGORY_DATA, labels: [] }), false);
});

test('needsStructure — 코어 네 거부가 설 때만 true, 값 편집은 B1 그대로', () => {
  // 무편집·값편집은 B1 페이로드로 나가야 한다 — 그래야 네 거부가 그물로 계속 선다.
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID), false);
  assert.equal(needsStructure(CATEGORY_DATA, [['9', '2.5'], ['1', '']]), false);
  // ① seriesCountMismatch  ② valueCountMismatch
  assert.equal(needsStructure(CATEGORY_DATA, [['4.30', '2.5']]), true);
  assert.equal(needsStructure(CATEGORY_DATA, [['4.30', '2.5', '7'], ['1', '', '8']]), true);
});

test('needsStructure ③ — 계열명은 c:tx 부재와 빈 이름을 같게 보고, null 자리는 안 본다', () => {
  // series[1].name === null. 코어도 unwrap_or_default() 로 빈 이름과 같게 본다.
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, undefined, ['계열 1', '']), false);
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, undefined, ['계열 1', null]), false);
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, undefined, ['다른 이름', '']), true);
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, undefined, ['계열 1', '이름 생김']), true);
});

test('needsStructure ④ — 카테고리 라벨은 텍스트 차이도, 분산형은 개수만 구조다', () => {
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, ['항목 1', '항목 2']), false);
  assert.equal(needsStructure(CATEGORY_DATA, SAME_GRID, ['항목 1', '바뀜']), true);
  // 분산형 X 텍스트 편집은 B1 범위다 — 개수가 같으면 구조가 아니다.
  assert.equal(needsStructure(SCATTER_DATA, SAME_GRID, ['0.9', '1.8']), false);
  assert.equal(needsStructure(SCATTER_DATA, SAME_GRID, ['0.7', '1.8', '2.9']), true);
});

test('opts 없는 buildChartEdits 는 B1 페이로드와 같다 — structure 키조차 없다', () => {
  const edits = buildChartEdits(CATEGORY_DATA, SAME_GRID);
  assert.equal('structure' in edits, false, '무편집 왕복이 B1 과 글자 단위로 같아야 한다');
});

test('structure 페이로드는 라벨을 원본과 같아도 항상 싣는다 — 빠지면 labelsRequired 로 거부된다', () => {
  const grid: string[][] = [['4.30', '2.5', '0'], ['1', '', '0']];
  const edits = buildChartEdits(CATEGORY_DATA, grid, ['항목 1', '항목 2', ''], {
    structure: true,
    names: ['계열 1', null],
  });
  assert.equal(edits.structure, true);
  assert.deepEqual(edits.labels, ['항목 1', '항목 2', ''], '원본과 같아도 목표 상태로 실린다');
});

test('structure 페이로드의 name — null 자리는 싣지 않는다(c:tx 부재에 이름을 주면 거부)', () => {
  const edits = buildChartEdits(CATEGORY_DATA, SAME_GRID, undefined, {
    structure: true,
    names: ['새 이름', null],
  });
  assert.deepEqual(edits.series[0], { name: '새 이름', values: ['4.30', '2.5'] });
  assert.equal('name' in edits.series[1], false, 'c:tx 없는 계열에는 name 을 싣지 않는다');
});

test('hasAnyEdit 는 계열명 변경도 편집으로 센다', () => {
  assert.equal(hasAnyEdit(CATEGORY_DATA, SAME_GRID, undefined, ['계열 1', '']), false);
  assert.equal(hasAnyEdit(CATEGORY_DATA, SAME_GRID, undefined, ['계열 하나', '']), true);
});
