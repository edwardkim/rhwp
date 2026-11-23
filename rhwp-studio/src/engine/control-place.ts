import type { ControlPlace, DocumentPosition } from '@/core/types';

/** 커서 위치가 든 문단의 주소. 셀·글상자 안이면 본문 문단에서 시작하는 경로를 준다. */
export function controlPlaceOf(pos: DocumentPosition): ControlPlace {
  const section = pos.sectionIndex;
  if (pos.cellPath?.length && pos.parentParaIndex !== undefined) {
    return {
      section,
      para: pos.parentParaIndex,
      cellPath: pos.cellPath.map((e) => [e.controlIndex, e.cellIndex, e.cellParaIndex]),
    };
  }
  if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
    return {
      section,
      para: pos.parentParaIndex,
      cellPath: [[pos.controlIndex, pos.cellIndex ?? 0, pos.cellParaIndex ?? pos.paragraphIndex]],
    };
  }
  return { section, para: pos.paragraphIndex, cellPath: [] };
}

/**
 * 문단 주소의 글자 자리를 커서 위치로 바꾼다. 한 겹 셀·글상자는 커서가 쓰는 평면 필드만 채우고,
 * 중첩 표는 경로를 함께 준다. `areas`는 findNearestControlAt이 준 영역 종류다.
 */
export function positionOfPlace(place: ControlPlace, charOffset: number, areas: string[] = []): DocumentPosition {
  const path = place.cellPath;
  if (!path.length) return { sectionIndex: place.section, paragraphIndex: place.para, charOffset };
  const [controlIndex, cellIndex, cellParaIndex] = path[0];
  const pos: DocumentPosition = {
    sectionIndex: place.section,
    paragraphIndex: path[path.length - 1][2],
    charOffset,
    parentParaIndex: place.para,
    controlIndex,
    cellIndex,
    cellParaIndex,
  };
  if (path.length > 1) {
    pos.cellPath = path.map(([c, i, p]) => ({ controlIndex: c, cellIndex: i, cellParaIndex: p }));
  } else if (areas[0] === 'textBox') {
    pos.isTextBox = true;
  }
  return pos;
}

/** F11로 고른 누름틀을 가리는 열쇠. 같은 문단 주소의 같은 글자 자리면 같다. */
export function fieldPinKey(pos: DocumentPosition): string {
  return JSON.stringify([controlPlaceOf(pos), pos.charOffset]);
}
