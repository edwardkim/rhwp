/**
 * 표 경계선 좌표 병합.
 *
 * 셀 bbox 의 좌/우·상/하 좌표를 소수점 1자리로 반올림해 모으면, **한 물리적 경계**를
 * 공유하는 두 셀의 값이 반올림 경계에 걸쳐 서로 다른 원소로 남는다(관측 예: 셀(0,0) 우측
 * 299.9 / 셀(0,1) 좌측 299.8). 그러면 같은 경계에 괘선이 둘 생기고, 이웃 괘선으로 드래그
 * 범위를 정하는 쪽에서 자기 자신을 이웃으로 잡아 범위가 뒤집힌다.
 *
 * 임계 이내로 붙은 좌표를 하나로 묶고, **묶여 사라진 좌표도 대표 인덱스를 가리키는 맵**을
 * 함께 돌려준다. 맵이 없으면 hit-test 가 자기 좌표로 인덱스를 되찾지 못해 그 셀의 경계가
 * 잡히지 않는다.
 */

/**
 * 같은 경계로 볼 좌표 간 거리 상한(px, 줌 적용 전 페이지 좌표).
 *
 * 최소 셀 크기가 MIN_TABLE_CELL_SIZE_HWP=200(= 2.67px)이므로 1.0px 는 실제로 떨어진 두
 * 경계를 삼키지 않는다. 실제 반올림 오차는 0.1px 수준이라 여유가 충분하다.
 */
export const BORDER_LINE_MERGE_EPS_PX = 1.0;

export type MergedBorderCoords = {
  /** 병합 후 대표 좌표 (오름차순). 각 그룹의 최솟값을 쓴다. */
  positions: number[];
  /** 병합 **전** 반올림 좌표 → 대표 좌표의 인덱스. 그룹의 모든 좌표가 들어 있다. */
  indexByCoord: Map<number, number>;
};

/**
 * 임계 이내로 붙은 좌표들을 하나의 경계선으로 묶는다.
 *
 * 그룹 판정은 **그룹 대표(첫 좌표)와의 거리**로 한다. 직전 좌표와의 거리로 이으면
 * 0.9px 씩 이어진 사슬이 임계를 넘어 커지면서 실제로 떨어진 경계까지 삼킬 수 있다.
 * 대표 기준이면 한 그룹의 폭이 eps 를 넘지 않는다.
 */
export function mergeBorderCoords(
  roundedCoords: Iterable<number>,
  eps: number = BORDER_LINE_MERGE_EPS_PX,
): MergedBorderCoords {
  const sorted = [...new Set(roundedCoords)].sort((a, b) => a - b);
  const positions: number[] = [];
  const indexByCoord = new Map<number, number>();

  for (const coord of sorted) {
    const representative = positions[positions.length - 1];
    if (representative === undefined || coord - representative > eps) {
      positions.push(coord);
    }
    indexByCoord.set(coord, positions.length - 1);
  }

  return { positions, indexByCoord };
}

/** 경계선이 실제로 존재하는 한 구간. */
export interface BorderSpan { start: number; end: number }

/** 칸 상자에서 span 을 뽑는 데 필요한 최소 형태. */
export interface SpanCell { x: number; y: number; w: number; h: number }

/** 맞닿은 칸 변을 잇는 허용 오차(px). 공유 변은 정확히 같으므로 반올림 오차만 허용한다. */
export const BORDER_SPAN_JOIN_EPS_PX = 0.5;

/**
 * 같은 경계에 속한 칸 변들을 이어 붙인다.
 *
 * 맞닿은 칸은 변을 정확히 공유하므로 반올림 오차만 허용해 잇고, 그보다 벌어지면 실제로
 * 끊긴 구간이라 따로 남긴다. 병합 칸이 가로지르는 자리가 그 "끊긴 구간" 이다.
 */
export function coalesceSpans(
  spans: readonly BorderSpan[],
  eps: number = BORDER_SPAN_JOIN_EPS_PX,
): BorderSpan[] {
  if (spans.length <= 1) return spans.map((s) => ({ ...s }));
  const sorted = [...spans].sort((a, b) => a.start - b.start);
  const out: BorderSpan[] = [{ ...sorted[0] }];
  for (const span of sorted.slice(1)) {
    const last = out[out.length - 1];
    if (span.start <= last.end + eps) {
      last.end = Math.max(last.end, span.end);
    } else {
      out.push({ ...span });
    }
  }
  return out;
}

/**
 * [#7191] 각 경계선이 **실제로 존재하는 구간**을 칸 상자에서 모은다.
 *
 * 종전에는 그리는 쪽이 선마다 표 전체 범위 하나만 들고 있었고, 적중 판정은 칸 상자를
 * 훑었다 — 두 범위의 출처가 달랐다. 병합 칸이 있으면 한 열 경계가 일부 행에만 존재하므로
 * 둘이 반드시 어긋난다(3147199 1쪽: 두 칸에만 있는 경계를 표 높이 828px 로 그렸다).
 *
 * 인덱스 조회는 `hitTestBorder` 와 **같은 `indexByCoord` 맵**을 쓴다. 대표 좌표로 맵을
 * 다시 만들면 병합돼 사라진 좌표를 가진 칸의 경계를 놓친다(`mergeBorderCoords` 주석).
 */
export function computeBorderSpans(
  cells: readonly SpanCell[],
  rowIndexByY: ReadonlyMap<number, number>,
  colIndexByX: ReadonlyMap<number, number>,
  round: (value: number) => number,
): { rowSpans: Map<number, BorderSpan[]>; colSpans: Map<number, BorderSpan[]> } {
  const rowSpans = new Map<number, BorderSpan[]>();
  const colSpans = new Map<number, BorderSpan[]>();
  const add = (target: Map<number, BorderSpan[]>, index: number, start: number, end: number) => {
    const list = target.get(index);
    if (list) list.push({ start, end });
    else target.set(index, [{ start, end }]);
  };

  for (const cell of cells) {
    const top = rowIndexByY.get(round(cell.y));
    if (top !== undefined) add(rowSpans, top, cell.x, cell.x + cell.w);
    const bottom = rowIndexByY.get(round(cell.y + cell.h));
    if (bottom !== undefined) add(rowSpans, bottom, cell.x, cell.x + cell.w);

    const left = colIndexByX.get(round(cell.x));
    if (left !== undefined) add(colSpans, left, cell.y, cell.y + cell.h);
    const right = colIndexByX.get(round(cell.x + cell.w));
    if (right !== undefined) add(colSpans, right, cell.y, cell.y + cell.h);
  }

  for (const [index, spans] of rowSpans) rowSpans.set(index, coalesceSpans(spans));
  for (const [index, spans] of colSpans) colSpans.set(index, coalesceSpans(spans));
  return { rowSpans, colSpans };
}
