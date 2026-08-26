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
  /**
   * [#6037] 둘러싼 plot 요소의 종류 — `axis` 와 같이 **첫 계열 기준**이다.
   * 표면이 "이 편집이 화면에 나타나는가"를 사전에 판단할 근거다(원형은 첫 계열만 그린다).
   */
  plot?:
    | 'bar'
    | 'line'
    | 'area'
    | 'pie'
    | 'ofPie'
    | 'doughnut'
    | 'radar'
    | 'scatter'
    | 'bubble'
    | 'stock'
    | 'other';
  /** [#6037] `c:upDownBars` 캔들 장치 — 있으면 양끝 계열을 바꾸는 구조 편집이 거부된다. */
  hasUpDownBars?: boolean;
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
  /**
   * 라벨. `structure` 가 없으면 편집할 때만 싣는다(코어는 분산형에서만 기록).
   * `structure: true` 면 **목표 라벨**이라 행 수가 바뀌면 필수다.
   */
  labels?: string[];
  /**
   * `structure` 가 없으면 `name` 을 싣지 않는다 — B1 은 계열명을 바꾸지 않고, 대조 함정
   * (`c:tx` 부재)만 만든다. `structure: true` 면 `name` 은 목표 계열명이다.
   */
  series: { name?: string; values: string[] }[];
  dryRun?: boolean;
  /**
   * [#5652] 구조 편집 의도. 켜면 행렬이 **목표 상태**로 해석돼 개수·이름·라벨이 달라도 되고,
   * 치수 차이는 위치 기반 꼬리 증감으로 적용된다.
   *
   * 의도 없이 켜지 않는다 — 켜는 순간 개수 불일치가 "의도"로 읽혀, 그리드 조립 버그가
   * 거부 대신 **조용한 계열 절단**이 된다. B1 의 네 거부가 그 사고를 막는 마지막 그물이다.
   */
  structure?: boolean;
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
 * [#6053] 라벨 열의 **구조** 편집 가능성 — 코어 라벨 규칙(`labels_shared` ∧ 다층 아님)과 동형.
 *
 * `labelsEditable`(B1, 분산형 전용)과 다른 술어다. 코어는 카테고리 라벨도 `structure: true`
 * 에서는 기록하므로(`plan_edits` 의 `apply_labels = scatter || structure`), 축 조건을 벗긴다.
 * B1 판정은 의미가 그대로 유효하므로 위 함수를 건드리지 않고 따로 세운다.
 */
export function labelsStructurallyEditable(data: ChartDataResult): boolean {
  return (
    data.labelsShared === true &&
    data.labelsMultiLevel !== true &&
    (data.labels?.length ?? 0) > 0
  );
}

/**
 * [#6053] 계열명·카테고리 라벨의 선제 텍스트 검증 — 코어 `is_safe_text` 와 동형.
 *
 * 수치가 아니므로 `cellInputIssue` 를 쓸 수 없다. 코어는 이스케이프하지 않고 거부하므로
 * (`unsafeText`) 같은 문자 집합을 미리 잡는다.
 */
export function unsafeTextIssue(text: string): 'unsafeText' | null {
  return /[<>&]|\p{Cc}/u.test(text) ? 'unsafeText' : null;
}

/**
 * [#6053] 결과 계열 수가 주식형 **렌더러의 역할 규약**을 벗어나는가.
 *
 * 주식형은 계열의 뜻이 XML 순서로 정해진다 — 3계열=고·저·종, 4계열=시·고·저·종
 * (`crates/rhwp-ooxml-chart/src/renderer.rs` `render_stock`). 그 밖의 계열 수에는 역할
 * 매핑이 없어 렌더러가 `render_line` 으로 폴백한다. 문서는 멀쩡하지만 캔들도 고저선도
 * 사라지고 **평범한 꺾은선으로 그려진다.**
 *
 * `candleAnchorBroken`(#6037)이 막는 것과 다른 축이다. 그쪽은 양끝이 바뀌어 캔들 몸통이
 * 엉뚱한 짝으로 잡히는 것을 보고, 이쪽은 계열 수 자체가 규약 밖으로 나가는 것을 본다 —
 * OHLC 중간 삽입은 양끝이 그대로라 그 가드를 통과하지만 5계열이 되어 선형으로 떨어진다.
 */
export function stockRoleCountBroken(data: ChartDataResult, nextSeriesCount: number): boolean {
  return data.plot === 'stock' && (nextSeriesCount < 3 || nextSeriesCount > 4);
}

/** `buildChartEdits` 의 구조 편집 확장 — 주지 않으면 B1 페이로드 그대로다. */
export interface ChartEditsOptions {
  /** 목표 행렬로 해석시킨다. `needsStructure` 가 판정한 값을 그대로 넘긴다. */
  structure?: boolean;
  /** 목표 계열명. `null`/`undefined` 인 자리는 `name` 을 싣지 않는다(`c:tx` 부재 자리). */
  names?: (string | null)[];
}

/**
 * [#6053] 이 페이로드가 `structure: true` 를 필요로 하는가.
 *
 * 규칙을 새로 발명하지 않는다 — 코어 `validate_values` 의 네 거부
 * (`seriesCountMismatch`·`valueCountMismatch`·`seriesNameMismatch`·`categoryMismatch`)가
 * 하나라도 설 페이로드면 true, 그 넷의 **부정**으로만 정의한다. 그래야 무편집·값편집은
 * B1 과 글자 단위로 같은 페이로드가 나가고(`structure` 키조차 없다), B1 의 네 거부가
 * 그리드 조립 버그를 잡는 그물로 계속 선다.
 */
export function needsStructure(
  data: ChartDataResult,
  values: string[][],
  labels?: string[],
  names?: (string | null)[],
): boolean {
  const series = data.series ?? [];
  // ① seriesCountMismatch
  if (values.length !== series.length) return true;
  // ② valueCountMismatch
  for (let i = 0; i < values.length; i++) {
    if (values[i].length !== series[i].values.length) return true;
  }
  // ③ seriesNameMismatch — 코어는 `c:tx` 부재(null)와 빈 이름을 같은 무편집 값으로 본다.
  if (names) {
    for (let i = 0; i < values.length; i++) {
      const want = names[i];
      if (want === null || want === undefined) continue;
      if (want !== (series[i].name ?? '')) return true;
    }
  }
  if (labels) {
    const have = data.labels ?? [];
    // 분산형은 개수만 어긋나도 거부(valueCountMismatch), 카테고리는 텍스트 차이도 거부.
    if (labels.length !== have.length) return true;
    if (data.axis !== 'scatter' && !sameStrings(labels, have)) return true;
  }
  return false;
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
 *
 * [#6053] `opts.structure` 면 두 규칙이 뒤집힌다:
 * - `labels` 를 **원본과 같아도 항상** 싣는다. 구조 편집에서 `labels` 는 목표 상태라,
 *   행 수가 바뀌는데 빠지면 `labelsRequired`/`scatterXYMismatch` 로 반드시 거부된다.
 * - `name` 을 싣는다. `null` 자리는 여전히 싣지 않는다 — `c:tx` 가 없는 계열에 빈 아닌
 *   이름을 주면 `seriesNameNotPatchable`, 빈 이름을 주면 기존 이름을 지우게 된다.
 *
 * `opts` 를 주지 않으면 B1 페이로드와 글자 단위로 같다(기존 호출부 무변경).
 */
export function buildChartEdits(
  data: ChartDataResult,
  values: string[][],
  labels?: string[],
  opts?: ChartEditsOptions,
): ChartEditsInput {
  const structure = opts?.structure === true;
  const names = opts?.names;
  const edits: ChartEditsInput = {
    series: values.map((v, i) => {
      const name = structure ? names?.[i] : undefined;
      return name === null || name === undefined
        ? { values: [...v] }
        : { name, values: [...v] };
    }),
  };
  if (labels && (structure || !sameStrings(labels, data.labels ?? []))) {
    edits.labels = [...labels];
  }
  if (structure) edits.structure = true;
  return edits;
}

/**
 * 그리드/라벨 어느 쪽이든 원본과 다른가 — 무변경이면 쓰기·undo 기록 없이 닫기 위한 판정.
 *
 * [#6053] `names` 를 주면 계열명 변경도 편집으로 센다. 코어와 같이 `c:tx` 부재(null)와
 * 빈 이름을 같은 값으로 보고, `null` 자리(싣지 않을 이름)는 대조하지 않는다.
 */
export function hasAnyEdit(
  data: ChartDataResult,
  values: string[][],
  labels?: string[],
  names?: (string | null)[],
): boolean {
  const series = data.series ?? [];
  if (values.length !== series.length) return true;
  for (let i = 0; i < values.length; i++) {
    if (!sameStrings(values[i], series[i].values)) return true;
  }
  if (labels && !sameStrings(labels, data.labels ?? [])) return true;
  if (names) {
    for (let i = 0; i < values.length; i++) {
      const want = names[i];
      if (want === null || want === undefined) continue;
      if (want !== (series[i].name ?? '')) return true;
    }
  }
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
