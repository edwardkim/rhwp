/**
 * 개체 드래그 종료 시 "히스토리에 기록할 before/after" 결정 로직 (#2759) — 순수 함수 모듈.
 *
 * 회전 핸들 드래그(input-handler-picture.ts:finishPictureRotateDrag)와 직선 끝점
 * 드래그(input-handler-mouse.ts:finishLineEndpointDrag)가 사용한다. 두 종료 핸들러는
 * 드래그 중 이미 문서에 반영된 변경을 executeOperation({kind:'record'})로 기록해야
 * undo/redo 계약(#2027/#2037/#2053/#2077 재발 계급)을 지킨다.
 *
 * picture-resize.ts / navigation-keymap.ts 와 동일하게 외부 import 없이 유지해
 * node:test 단위 테스트(tests/object-drag-record.test.ts) 대상으로 둔다. "변화가 없으면
 * null" 계약으로 무의미한 Undo 항목·redo 스택 무효화를 막는 책임이 이 모듈에 있다.
 */

/** 회전각 변경 기록. before/after 는 setObjectProperties 에 그대로 전달되는 속성 가방. */
export interface RotationRecord {
  before: { rotationAngle: number };
  after: { rotationAngle: number };
}

/**
 * 드래그 시작 각도(origAngle)와 종료 시 문서에 실제 반영된 각도(finalAngle)를 비교해
 * 기록 대상 before/after 를 돌려준다. 두 각이 같거나(=드래그 없음) 입력이 비정상이면 null.
 * 각도는 호출부에서 이미 정규화·라운딩된 값이 들어온다(그대로 보존).
 */
export function computeRotationRecord(
  origAngle: number,
  finalAngle: number,
): RotationRecord | null {
  if (!Number.isFinite(origAngle) || !Number.isFinite(finalAngle)) return null;
  if (origAngle === finalAngle) return null;
  return {
    before: { rotationAngle: origAngle },
    after: { rotationAngle: finalAngle },
  };
}

/** 직선 끝점 좌표(글로벌 HWPUNIT). moveLineEndpoint(sec,ppi,ci, sx,sy,ex,ey) 인자와 1:1. */
export interface LineEndpoints {
  sx: number;
  sy: number;
  ex: number;
  ey: number;
}

/** 직선 끝점 드래그 기록. before/after 는 MoveLineEndpointCommand 가 그대로 사용. */
export interface LineEndpointRecord {
  before: LineEndpoints;
  after: LineEndpoints;
}

function endpointsEqual(a: LineEndpoints, b: LineEndpoints): boolean {
  return a.sx === b.sx && a.sy === b.sy && a.ex === b.ex && a.ey === b.ey;
}

function endpointsFinite(p: LineEndpoints): boolean {
  return Number.isFinite(p.sx) && Number.isFinite(p.sy) &&
    Number.isFinite(p.ex) && Number.isFinite(p.ey);
}

/**
 * 드래그 시작 시 캡처한 끝점(before)과 종료 시 문서에서 읽은 끝점(after)을 비교해 기록
 * 대상을 돌려준다. 좌표가 동일하거나(=클릭만 하고 안 움직임) 비정상이면 null.
 */
export function computeLineEndpointRecord(
  before: LineEndpoints,
  after: LineEndpoints,
): LineEndpointRecord | null {
  if (!endpointsFinite(before) || !endpointsFinite(after)) return null;
  if (endpointsEqual(before, after)) return null;
  return { before: { ...before }, after: { ...after } };
}
