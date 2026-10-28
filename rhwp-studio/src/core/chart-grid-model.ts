/**
 * [#6053] 차트 데이터 그리드의 편집 모델 — 불변 모델이 진실, DOM 은 투영.
 *
 * B1(#4694)은 `valueInputs: (HTMLInputElement|null)[][]` 가 곧 상태였다. 그래서 행·열 수가
 * 원본에 고정됐고, `null` 하나가 "원본에 칸이 없음"과 "원본이 빈 값"을 겸했고, 수집이
 * `data.series` 를 축으로 돌아 구조가 못 바뀌었다. 셋 다 **DOM 이 진실**이라는 한 병의 증상이다.
 *
 * 여기서는 모델이 진실이다 — 재렌더가 무손실이고, 겸용하던 두 뜻이 `absent` 와 `empty` 로 갈린다.
 *
 * 미변경 문자열은 원본 그대로 담고 손대지 않는다(`trim()`·`Number()` 왕복이 이 경로에 한 번도
 * 없다) — `4.30` 표기가 보존돼야 무편집 왕복의 바이트 동일이 유지된다.
 *
 * 엔진 계약(`structure: true`)은 목표 행렬을 **위치 기반**으로 적용한다 — 겹치는 칸은 제자리
 * 치환, 남는 행·열은 꼬리 증감, 계열 신설은 마지막 계열 복제
 * (`src/document_core/commands/object_ops/chart.rs` `plan_edits`). 이 모듈은 "목표 행렬"만
 * 만들고 적용 규칙을 재현하지 않는다. 거부 판정은 코어 dryRun 이 단독으로 한다.
 */

import { labelsStructurallyEditable } from './chart-data-target.ts';
import type { ChartDataResult } from './chart-data-target.ts';

/** 칸이 어디서 왔는가 — B1 이 `null` 하나로 겸하던 네 가지 뜻. */
export type CellOrigin =
  /** 원본 값 — 제자리 치환 가능. */
  | { kind: 'value'; series: number; point: number }
  /** 원본이 `<c:v/>` 빈 값 — 코어가 `valueNotPatchable` 로 거부한다. */
  | { kind: 'empty'; series: number; point: number }
  /** ragged 원본에서 그 계열에 없던 칸. */
  | { kind: 'absent'; series: number }
  /** 이번 편집으로 생긴 칸. */
  | { kind: 'new' };

export interface GridCell {
  text: string;
  origin: CellOrigin;
}

export interface GridSeries {
  /** 원본 계열 인덱스 — 이번 편집으로 생긴 계열은 null. */
  source: number | null;
  /** 목표 계열명. `null` 은 "이름 없음"(= `c:tx` 부재)이며 페이로드에 `name` 을 싣지 않는다. */
  name: string | null;
  cells: GridCell[];
}

export interface GridLabel {
  text: string;
  origin: 'existing' | 'new';
}

export interface GridModel {
  readonly axis: 'scatter' | 'category';
  /** 라벨 열을 구조적으로 편집할 수 있는가 — 모델 조립 시 한 번 판정한다. */
  readonly labelsUsable: boolean;
  /**
   * 원본 마지막 계열에 `c:tx` 이름이 있었는가. 신설 계열의 이름 유무가 여기 매인다 —
   * 있으면 이름이 **필수**(`seriesNameRequired`), 없으면 이름을 주면 **거부**
   * (`seriesNameNotPatchable`). 코어 §5 신설 계열 규칙과 동형.
   */
  readonly templateNamed: boolean;
  rowCount: number;
  series: GridSeries[];
  labels: GridLabel[];
}

/** 새 칸의 기본값 — 빈 문자열은 코어 `is_number` 가 거부하므로 `'0'` 이다. */
export const NEW_CELL_TEXT = '0';

function newCell(): GridCell {
  return { text: NEW_CELL_TEXT, origin: { kind: 'new' } };
}

function newLabel(axis: 'scatter' | 'category'): GridLabel {
  // 분산형 X 는 수치여야 한다. 카테고리는 빈 문자열도 안전한 텍스트다(사용자가 채운다).
  return { text: axis === 'scatter' ? NEW_CELL_TEXT : '', origin: 'new' };
}

/** 읽기 봉투를 편집 모델로 편다. ragged 원본은 `absent` 칸으로 직사각형이 된다. */
export function gridFromChartData(data: ChartDataResult): GridModel {
  const series = data.series ?? [];
  const labels = data.labels ?? [];
  const axis: 'scatter' | 'category' = data.axis === 'scatter' ? 'scatter' : 'category';
  const rowCount = Math.max(labels.length, ...series.map((s) => s.values.length), 0);

  return {
    axis,
    labelsUsable: labelsStructurallyEditable(data),
    templateNamed: series.length > 0 && series[series.length - 1].name !== null,
    rowCount,
    series: series.map((s, si) => ({
      source: si,
      name: s.name,
      cells: Array.from({ length: rowCount }, (_, r): GridCell => {
        const original = s.values[r];
        if (original === undefined) return { text: '', origin: { kind: 'absent', series: si } };
        if (original === '') return { text: '', origin: { kind: 'empty', series: si, point: r } };
        return { text: original, origin: { kind: 'value', series: si, point: r } };
      }),
    })),
    labels: Array.from({ length: rowCount }, (_, r): GridLabel => ({
      text: labels[r] ?? '',
      origin: r < labels.length ? 'existing' : 'new',
    })),
  };
}

function cloneSeries(s: GridSeries): GridSeries {
  return { source: s.source, name: s.name, cells: [...s.cells] };
}

function withSeries(model: GridModel, series: GridSeries[], labels: GridLabel[]): GridModel {
  return { ...model, series, labels, rowCount: labels.length };
}

// ── 셀 write-through — 입력이 모델로 바로 들어가 재렌더가 무손실이다 ──────────

export function setCell(model: GridModel, series: number, row: number, text: string): GridModel {
  const next = model.series.map(cloneSeries);
  const cells = next[series]?.cells;
  if (!cells || !cells[row]) return model;
  cells[row] = { ...cells[row], text };
  return { ...model, series: next };
}

export function setSeriesName(model: GridModel, series: number, name: string): GridModel {
  const next = model.series.map(cloneSeries);
  if (!next[series]) return model;
  next[series].name = name;
  return { ...model, series: next };
}

export function setLabel(model: GridModel, row: number, text: string): GridModel {
  const labels = [...model.labels];
  if (!labels[row]) return model;
  labels[row] = { ...labels[row], text };
  return { ...model, labels };
}

// ── 구조 연산 — 전부 순수, 새 모델을 돌려준다 ────────────────────────────────

/** `at` 위치에 행을 끼운다. `at === rowCount` 면 꼬리에 붙인다. */
export function insertRow(model: GridModel, at: number): GridModel {
  const where = clamp(at, 0, model.rowCount);
  const series = model.series.map((s) => {
    const cells = [...s.cells];
    cells.splice(where, 0, newCell());
    return { source: s.source, name: s.name, cells };
  });
  const labels = [...model.labels];
  labels.splice(where, 0, newLabel(model.axis));
  return withSeries(model, series, labels);
}

export function deleteRow(model: GridModel, at: number): GridModel {
  if (model.rowCount <= 1) return model;
  const where = clamp(at, 0, model.rowCount - 1);
  const series = model.series.map((s) => {
    const cells = [...s.cells];
    cells.splice(where, 1);
    return { source: s.source, name: s.name, cells };
  });
  const labels = [...model.labels];
  labels.splice(where, 1);
  return withSeries(model, series, labels);
}

/**
 * `at` 위치에 계열을 끼운다.
 *
 * 신설 계열의 이름은 `templateNamed` 가 가른다 — 원본 마지막 계열에 `c:tx` 가 있으면 이름이
 * 필수라 기본 이름을 채워 넣고(그래야 `seriesNameRequired` 가 애초에 서지 않는다), 없으면
 * `null` 로 두어 페이로드에 `name` 을 싣지 않는다.
 */
export function insertColumn(model: GridModel, at: number): GridModel {
  const where = clamp(at, 0, model.series.length);
  const series = model.series.map(cloneSeries);
  series.splice(where, 0, {
    source: null,
    name: model.templateNamed ? defaultSeriesName(model, where) : null,
    cells: Array.from({ length: model.rowCount }, newCell),
  });
  return { ...model, series };
}

export function deleteColumn(model: GridModel, at: number): GridModel {
  if (model.series.length <= 1) return model;
  const where = clamp(at, 0, model.series.length - 1);
  const series = model.series.map(cloneSeries);
  series.splice(where, 1);
  return { ...model, series };
}

/** 이미 쓰이는 이름과 겹치지 않는 `계열 N`. */
function defaultSeriesName(model: GridModel, at: number): string {
  const taken = new Set(model.series.map((s) => s.name).filter((n): n is string => n !== null));
  let n = at + 1;
  while (taken.has(`계열 ${n}`)) n += 1;
  return `계열 ${n}`;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

// ── 목표 행렬 추출 — 코어 페이로드의 입력 ────────────────────────────────────

/** `values[seriesIdx][pointIdx]` — 코어 `ChartEdits.series[].values` 와 같은 계열-major. */
export function gridValues(model: GridModel): string[][] {
  return model.series.map((s) => s.cells.map((c) => c.text));
}

export function gridLabels(model: GridModel): string[] {
  return model.labels.map((l) => l.text);
}

/** 목표 계열명. `null` 인 자리는 페이로드에 `name` 을 싣지 않는다는 뜻이다. */
export function gridSeriesNames(model: GridModel): (string | null)[] {
  return model.series.map((s) => s.name);
}
