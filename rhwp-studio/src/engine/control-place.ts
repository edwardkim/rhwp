import type { ControlPlace, DocumentPosition, FieldInfoResult } from '@/core/types';

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

export interface F11Field {
  startOffset: number;
  place: ControlPlace;
  controlIndex: number;
  info: { fieldId?: number; fieldType?: string; guideName?: string };
}

type FieldQueries = {
  getFieldInfoAt(pos: DocumentPosition): FieldInfoResult;
  getFieldInfoForControlAt(place: ControlPlace, controlIndex: number, offset: number): FieldInfoResult;
};

function samePlace(a: ControlPlace, b: ControlPlace): boolean {
  return a.section === b.section && a.para === b.para
    && JSON.stringify(a.cellPath) === JSON.stringify(b.cellPath);
}

/** 누름틀 이외의 필드는 F11이 선택한 원래 자리에서 이전 컨트롤 탐색을 이어 간다. */
export function atOtherSelectedField(pos: DocumentPosition, pin?: F11Field | null): boolean {
  return !!pin && pin.info.fieldType !== 'clickhere' && pos.charOffset === pin.startOffset
    && samePlace(controlPlaceOf(pos), pin.place);
}

/** 입력 뒤 바뀐 범위에서 선택한 누름틀을 조회한다. 다른 문단·셀이나 사라진 필드에는 적용하지 않는다. */
export function selectedFieldAt(wasm: FieldQueries, pos: DocumentPosition, pin?: F11Field | null): FieldInfoResult | null {
  if (!pin || pin.info.fieldType !== 'clickhere') return null;
  const place = controlPlaceOf(pos);
  if (!samePlace(place, pin.place)) return null;
  const info = wasm.getFieldInfoForControlAt(place, pin.controlIndex, pos.charOffset);
  return info.inField && info.fieldId === pin.info.fieldId ? info : null;
}

export function fieldInfoAt(wasm: FieldQueries, pos: DocumentPosition, pin?: F11Field | null): FieldInfoResult {
  return selectedFieldAt(wasm, pos, pin) ?? wasm.getFieldInfoAt(pos);
}
