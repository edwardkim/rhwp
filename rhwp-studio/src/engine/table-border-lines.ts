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
