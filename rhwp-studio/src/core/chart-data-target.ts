/**
 * [#4694] 차트 데이터 편집 대상 해석 — listCharts() 열거와 studio 선택 ref 의 대조.
 *
 * 정본 주소는 문서 순번(by_index)이다. 3인자 좌표는 본문 직속 차트만 표현할 수 있어,
 * studio 는 항상 열거 → 대조 → `get/setChartDataByIndex` 단일 경로를 쓴다.
 * 대조 실패는 null — 오매칭으로 다른 차트를 고치는 것이 최악이므로, 확실할 때만 맞춘다.
 *
 * wire 계약(필드명·구조)은 코어 tests/issue_4694_chart_list.rs 가 고정한다.
 */

import type { CellPathLike } from './types';

export interface ChartContainerRefJson {
  kind: 'textbox' | 'header' | 'footer' | 'footnote' | 'endnote' | 'tableCell';
  control: number;
  paragraph: number;
  cell?: number;
}

export interface ChartRefJson {
  /** 문서 순서 0-based 순번 — `getChartDataByIndex` 의 주소. */
  index: number;
  section: number;
  /** 본문(루트) 문단 인덱스 — 컨테이너 안이면 그 컨테이너를 품은 본문 문단. */
  paragraph: number;
  /** 차트가 놓인 문단(컨테이너 안이면 내부 문단) 안의 컨트롤 인덱스. */
  control: number;
  /** 컨테이너 경로 — 본문 직속이면 키 자체가 없다. */
  container?: ChartContainerRefJson[];
  zipPart?: number;
  nestedCopy?: number;
}

/** 검증 거부 항목 — 코어 `invalid[]` 의 원소. */
export interface ChartInvalidEntry {
  reason: string;
  message: string;
  [key: string]: unknown;
}

/** getChartData/getChartDataByIndex 응답. 실패는 `{ ok: false, invalid: [...] }`. */
export interface ChartDataResult {
  ok: boolean;
  /** 1-based 표시 번호 — CLI `--chart N` 과 같은 값. */
  chart?: number;
  axis?: 'scatter' | 'category';
  source?: 'zipPart' | 'nestedCopy';
  representations?: { zipPart: boolean; nestedCopy: boolean };
  labelsShared?: boolean;
  labelsMultiLevel?: boolean;
  labels?: string[];
  series?: { name: string | null; values: string[] }[];
  invalid?: ChartInvalidEntry[];
}

/**
 * setChartData* 입력 — 코어 `ChartEdits` 와 동형.
 * 값은 **문자열**이어야 한다(숫자로 보내면 `4.3`→`4.30` 되쓰기로 무편집 왕복이 깨진다).
 */
export interface ChartEditsInput {
  /** 분산형 X 라벨 — 편집할 때만 싣는다. category 라벨은 B1 범위 밖. */
  labels?: string[];
  /** name 은 싣지 않는다 — B1 은 계열명을 바꾸지 않고, 대조 함정(c:tx 부재)만 만든다. */
  series: { name?: string; values: string[] }[];
  dryRun?: boolean;
}

/** setChartData/setChartDataByIndex 응답. */
export interface SetChartDataResult {
  ok: boolean;
  chart?: number;
  changedCount?: number;
  changed?: unknown[];
  /** 실제로 쓴 표현 — `["zipPart","nestedCopy"]` 또는 HWP5 는 `["nestedCopy"]`. */
  wrote?: string[];
  dryRun?: boolean;
  invalid?: ChartInvalidEntry[];
}

/** 선택 ref 중 대조에 쓰는 부분 — `getSelectedPictureRef()` 의 부분집합. */
export interface ChartTargetRef {
  sec: number;
  /** headerFooter 동반 시 내부 문단, cellPath 동반 시 본문(루트) 문단. */
  ppi: number;
  /** 차트가 놓인 문단 안의 컨트롤 인덱스(컨테이너 안이면 내부 컨트롤). */
  ci: number;
  cellPath?: CellPathLike;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
}

/** `getSelectedPictureRef()` 반환값 중 이 모듈이 읽는 부분. */
export interface SelectedOleRefLike {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellIdx?: number;
  cellParaIdx?: number;
  outerTableControlIdx?: number;
  cellPath?: CellPathLike;
  noteRef?: unknown;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
}

/**
 * 선택 ref 를 매처 입력으로 정규화한다. 표현할 수 없는 선택(비-ole, 각주/미주)은
 * null — 메뉴 미노출로 이어지는 안전 축소다.
 *
 * 셀 문맥 3종(cellIdx/cellParaIdx/outerTableControlIdx)에서의 한 단계 cellPath 조립은
 * `insert:picture-props` 선례(command/commands/insert.ts)와 같은 규칙이다.
 */
export function chartTargetFromSelection(ref: SelectedOleRefLike): ChartTargetRef | null {
  if (ref.type !== 'ole' || ref.noteRef) return null;
  const cellPath: CellPathLike | undefined =
    ref.cellPath ??
    (ref.cellIdx !== undefined &&
    ref.cellParaIdx !== undefined &&
    ref.outerTableControlIdx !== undefined
      ? [{ controlIdx: ref.outerTableControlIdx, cellIdx: ref.cellIdx, cellParaIdx: ref.cellParaIdx }]
      : undefined);
  return { sec: ref.sec, ppi: ref.ppi, ci: ref.ci, cellPath, headerFooter: ref.headerFooter };
}

// ── 편집 페이로드 로직 — DOM 없는 순수 함수 (다이얼로그가 사용) ──────────

/**
 * 셀 입력의 선제 검증. 최종 판정은 코어 dryRun(`is_number`)이다 — 여기서는 UX 용으로
 * 명백한 위반만 미리 잡는다(공백 표기 등 미세 차이는 dryRun 이 거른다).
 */
export function cellInputIssue(text: string): 'empty' | 'notANumber' | null {
  if (text.trim() === '') return 'empty';
  return Number.isFinite(Number(text)) ? null : 'notANumber';
}

/** 라벨 열 편집 가능성 — 코어가 분산형 공유 X축에서만 라벨을 기록한다는 계약과 동형. */
export function labelsEditable(data: ChartDataResult): boolean {
  return data.axis === 'scatter' && data.labelsShared === true && data.labelsMultiLevel !== true;
}

/**
 * 그리드 상태를 코어 `ChartEdits` 페이로드로 조립한다.
 *
 * - 값은 **원본 문자열 그대로**(정규화 금지) — 코어가 문자열 diff 로 미변경 셀을
 *   무기록 처리하므로 표기(`4.30`)가 보존된다.
 * - `name` 은 싣지 않는다 — B1 은 계열명을 바꾸지 않고, `c:tx` 부재/빈 문자열 대조
 *   함정만 만든다.
 * - `labels` 는 원본과 다를 때만 싣는다(코어는 분산형에서만 기록).
 *
 * `values` 는 계열-major — `values[seriesIdx][pointIdx]`.
 */
export function buildChartEdits(
  data: ChartDataResult,
  values: string[][],
  labels?: string[],
): ChartEditsInput {
  const edits: ChartEditsInput = {
    series: values.map((v) => ({ values: [...v] })),
  };
  if (labels && !sameStrings(labels, data.labels ?? [])) {
    edits.labels = [...labels];
  }
  return edits;
}

/** 그리드/라벨 어느 쪽이든 원본과 다른가 — 무변경이면 쓰기·undo 기록 없이 닫기 위한 판정. */
export function hasAnyEdit(data: ChartDataResult, values: string[][], labels?: string[]): boolean {
  const series = data.series ?? [];
  if (values.length !== series.length) return true;
  for (let i = 0; i < values.length; i++) {
    if (!sameStrings(values[i], series[i].values)) return true;
  }
  if (labels && !sameStrings(labels, data.labels ?? [])) return true;
  return false;
}

function sameStrings(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}

interface NormalizedCellSeg {
  controlIdx: number;
  cellIdx: number;
  cellParaIdx: number;
}

/** CellPathEntry(controlIndex…)/CellPathSegment(controlIdx…) 두 철자를 한 형태로. */
function normalizeCellPath(path: CellPathLike): NormalizedCellSeg[] | null {
  const out: NormalizedCellSeg[] = [];
  for (const seg of path) {
    const s = seg as unknown as Record<string, unknown>;
    const controlIdx = s.controlIdx ?? s.controlIndex;
    const cellIdx = s.cellIdx ?? s.cellIndex;
    const cellParaIdx = s.cellParaIdx ?? s.cellParaIndex;
    if (
      typeof controlIdx !== 'number' ||
      typeof cellIdx !== 'number' ||
      typeof cellParaIdx !== 'number'
    ) {
      return null;
    }
    out.push({ controlIdx, cellIdx, cellParaIdx });
  }
  return out;
}

/**
 * 선택 ref 에 맞는 열거 항목을 돌려준다. 못 찾으면(또는 선택 형태를 표현할 수 없으면) null.
 *
 * 대조 규칙 (#4694 계획서 §4-1):
 * - 본문 직속: container 없음 ∧ (section, paragraph, control) === (sec, ppi, ci)
 * - 표 셀: container 전 단계가 tableCell 이고 cellPath 와 전 원소 일치 ∧ 루트 문단 === ppi
 *   ∧ 내부 컨트롤 === ci
 * - 머리말/꼬리말: container 한 단계 {kind, control: outerControlIdx, paragraph: ppi(내부)}
 *   ∧ 루트 문단 === outerParaIdx ∧ 내부 컨트롤 === ci
 */
export function matchChartRef(charts: ChartRefJson[], ref: ChartTargetRef): ChartRefJson | null {
  // [#4694 R1] 맨 3좌표 선택은 머리말/꼬리말 안 ole(레이아웃이 컨테이너 문맥을 아직
  // 싣지 않는 유일한 경로)와 구분되지 않을 수 있다. 컨테이너 차트의 루트 좌표
  // (paragraph=본문 앵커, control=내부 인덱스)가 같은 3좌표와 겹치면 어느 쪽을
  // 클릭했는지 알 수 없다 — 오매칭(다른 차트 편집)이 최악이므로 거부한다.
  if (!ref.headerFooter && (!ref.cellPath || ref.cellPath.length === 0)) {
    const shadowed = charts.some(
      (chart) =>
        (chart.container?.length ?? 0) > 0 &&
        chart.section === ref.sec &&
        chart.paragraph === ref.ppi &&
        chart.control === ref.ci,
    );
    if (shadowed) return null;
  }
  const hits = charts.filter((chart) => matchesOne(chart, ref));
  // 주소는 문서 안에서 유일해야 한다 — 둘 이상 맞으면 계약 밖 상태이므로 안전하게 실패.
  return hits.length === 1 ? hits[0] : null;
}

function matchesOne(chart: ChartRefJson, ref: ChartTargetRef): boolean {
  if (chart.section !== ref.sec) return false;
  const container = chart.container ?? [];

  if (ref.headerFooter) {
    const hf = ref.headerFooter;
    return (
      container.length === 1 &&
      container[0].kind === hf.kind &&
      container[0].control === hf.outerControlIdx &&
      container[0].paragraph === ref.ppi &&
      chart.paragraph === hf.outerParaIdx &&
      chart.control === ref.ci
    );
  }

  if (ref.cellPath && ref.cellPath.length > 0) {
    const path = normalizeCellPath(ref.cellPath);
    if (!path || container.length !== path.length) return false;
    for (let i = 0; i < path.length; i++) {
      const level = container[i];
      const seg = path[i];
      // 표 셀 정합 — (control, cell, paragraph) 전 좌표 일치.
      const cellMatch =
        level.kind === 'tableCell' &&
        level.control === seg.controlIdx &&
        level.cell === seg.cellIdx &&
        level.paragraph === seg.cellParaIdx;
      // 글상자 sentinel(#1171 계약) — 코어가 cellIndex 0 으로 방출한다. 같은 컨트롤이
      // 표이면서 글상자일 수는 없으므로 이 분기가 모호성을 만들지 않는다.
      const textboxMatch =
        level.kind === 'textbox' &&
        seg.cellIdx === 0 &&
        level.control === seg.controlIdx &&
        level.paragraph === seg.cellParaIdx;
      if (!cellMatch && !textboxMatch) return false;
    }
    return chart.paragraph === ref.ppi && chart.control === ref.ci;
  }

  // 본문 직속 — 컨테이너 안 차트를 맨 좌표로 오인하지 않는다.
  return container.length === 0 && chart.paragraph === ref.ppi && chart.control === ref.ci;
}
