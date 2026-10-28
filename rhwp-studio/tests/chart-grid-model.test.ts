import test from 'node:test';
import assert from 'node:assert/strict';

import {
  NEW_CELL_TEXT,
  deleteColumn,
  deleteRow,
  gridFromChartData,
  gridLabels,
  gridSeriesNames,
  gridValues,
  insertColumn,
  insertRow,
  setCell,
  setLabel,
  setSeriesName,
} from '../src/core/chart-grid-model.ts';
import type { ChartDataResult } from '../src/core/chart-data-target.ts';

// [#6053] B1 은 DOM 이 진실이라 행·열 수가 원본에 고정되고 `null` 하나가 "칸 없음"과
// "빈 값" 두 뜻을 겸했다. 여기서는 모델이 진실이므로 그 둘이 갈리고 구조가 움직인다.
// 값 문자열은 한 번도 정규화되지 않아야 한다 — `4.30` 이 `4.3` 이 되면 무편집 왕복이 깨진다.

const CATEGORY: ChartDataResult = {
  ok: true,
  chart: 1,
  axis: 'category',
  plot: 'bar',
  hasUpDownBars: false,
  labelsShared: true,
  labelsMultiLevel: false,
  labels: ['항목 1', '항목 2'],
  series: [
    { name: '계열 1', values: ['4.30', '2.5'] },
    { name: '계열 2', values: ['1', ''] },
  ],
};

/** ragged 원본 — 두 번째 계열에 점이 하나뿐이다. */
const RAGGED: ChartDataResult = {
  ...CATEGORY,
  series: [
    { name: '계열 1', values: ['4.30', '2.5'] },
    { name: null, values: ['1'] },
  ],
};

test('봉투를 펴면 원본 문자열이 그대로 담기고 표기가 보존된다', () => {
  const m = gridFromChartData(CATEGORY);
  assert.equal(m.rowCount, 2);
  assert.equal(m.series.length, 2);
  assert.deepEqual(gridValues(m), [['4.30', '2.5'], ['1', '']]);
  assert.deepEqual(gridLabels(m), ['항목 1', '항목 2']);
  assert.deepEqual(gridSeriesNames(m), ['계열 1', '계열 2']);
});

test('빈 값과 없는 칸이 갈린다 — B1 은 둘 다 null 이었다', () => {
  const empty = gridFromChartData(CATEGORY).series[1].cells[1];
  assert.equal(empty.origin.kind, 'empty', '원본 <c:v/> 는 empty — 제자리 치환 대상이 아니다');

  const absent = gridFromChartData(RAGGED).series[1].cells[1];
  assert.equal(absent.origin.kind, 'absent', 'ragged 원본에 없던 칸은 absent');
});

test('ragged 원본도 직사각형으로 펴진다 — 목표 행렬은 직사각형이어야 한다', () => {
  const m = gridFromChartData(RAGGED);
  assert.equal(m.rowCount, 2);
  for (const s of m.series) assert.equal(s.cells.length, 2);
});

test('입력은 모델로 write-through 되고 원본 모델은 안 바뀐다', () => {
  const m = gridFromChartData(CATEGORY);
  const next = setCell(m, 0, 0, '91.7');
  assert.equal(gridValues(next)[0][0], '91.7');
  assert.equal(gridValues(m)[0][0], '4.30', '원본 모델은 불변이어야 한다');

  assert.equal(gridSeriesNames(setSeriesName(m, 1, '새 이름'))[1], '새 이름');
  assert.equal(gridLabels(setLabel(m, 1, '항목 둘'))[1], '항목 둘');
});

test('행 삽입은 모든 계열과 라벨에 같은 자리를 연다 — 새 칸 기본값은 0', () => {
  const m = insertRow(gridFromChartData(CATEGORY), 1);
  assert.equal(m.rowCount, 3);
  assert.deepEqual(gridValues(m), [
    ['4.30', NEW_CELL_TEXT, '2.5'],
    ['1', NEW_CELL_TEXT, ''],
  ]);
  // 빈 문자열은 코어 is_number 가 거부하므로 새 칸이 빈 값이면 안 된다.
  assert.equal(NEW_CELL_TEXT, '0');
  assert.deepEqual(gridLabels(m), ['항목 1', '', '항목 2']);
});

test('분산형 새 행의 X 는 수치다 — 빈 X 는 코어가 notANumber 로 거부한다', () => {
  const scatter = gridFromChartData({ ...CATEGORY, axis: 'scatter', labels: ['0.7', '1.8'] });
  assert.equal(gridLabels(insertRow(scatter, 2))[2], '0');
});

test('행 삭제는 라벨도 같은 자리를 지우고, 마지막 행은 지우지 않는다', () => {
  const m = deleteRow(gridFromChartData(CATEGORY), 0);
  assert.deepEqual(gridValues(m), [['2.5'], ['']]);
  assert.deepEqual(gridLabels(m), ['항목 2']);

  // lastPointDeleteRefused — 코어가 거부하기 전에 모델이 이미 멈춘다.
  assert.equal(deleteRow(m, 0), m, '행이 하나면 삭제가 무동작이어야 한다');
});

test('계열 삽입은 현재 행 수만큼 새 칸을 만들고 이름을 채운다', () => {
  const m = insertColumn(gridFromChartData(CATEGORY), 1);
  assert.equal(m.series.length, 3);
  assert.deepEqual(gridValues(m)[1], [NEW_CELL_TEXT, NEW_CELL_TEXT]);
  assert.equal(m.series[1].source, null, '신설 계열은 원본 출처가 없다');
  // 템플릿(원본 마지막 계열)에 c:tx 가 있으므로 이름이 필수다 — seriesNameRequired 예방.
  assert.equal(typeof m.series[1].name, 'string');
  assert.notEqual(m.series[1].name, '계열 1');
  assert.notEqual(m.series[1].name, '계열 2');
});

test('템플릿에 c:tx 가 없으면 신설 계열 이름을 주지 않는다 — 주면 거부된다', () => {
  // 원본 마지막 계열의 name 이 null = c:tx 부재. 이름을 주면 seriesNameNotPatchable.
  const m = insertColumn(gridFromChartData(RAGGED), 2);
  assert.equal(m.templateNamed, false);
  assert.equal(m.series[2].name, null);
});

test('계열 삭제는 마지막 하나를 남긴다 — lastSeriesDeleteRefused 예방', () => {
  const one = deleteColumn(gridFromChartData(CATEGORY), 0);
  assert.equal(one.series.length, 1);
  assert.deepEqual(gridValues(one), [['1', '']]);
  assert.equal(deleteColumn(one, 0), one, '계열이 하나면 삭제가 무동작이어야 한다');
});

test('구조 연산은 손대지 않은 칸의 문자열을 그대로 둔다', () => {
  const m = insertColumn(insertRow(gridFromChartData(CATEGORY), 0), 0);
  const values = gridValues(m);
  // 원본 (계열 0, 점 0) 은 이제 (계열 1, 점 1) 자리에 있고 표기가 살아 있어야 한다.
  assert.equal(values[1][1], '4.30');
});

test('라벨 열 사용 가능성 — 비공유·다층은 구조 편집에서 막힌다', () => {
  assert.equal(gridFromChartData(CATEGORY).labelsUsable, true);
  assert.equal(gridFromChartData({ ...CATEGORY, labelsShared: false }).labelsUsable, false);
  assert.equal(gridFromChartData({ ...CATEGORY, labelsMultiLevel: true }).labelsUsable, false);
});
