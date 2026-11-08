/**
 * 선택 영역 → 평문.
 *
 * 복사 경로(`copySelection*`)는 WASM 내부 클립보드를 덮어쓴다 — 읽기만 하려는 호출이 사용자의
 * 다음 붙여넣기를 바꾸면 안 되므로, 여기서는 문단·칸 문단 단위의 읽기 API 만 쓴다.
 *
 * 지원 범위는 본문 문단 범위와 **한 표 칸 안**의 칸 문단 범위다. 서로 다른 칸·본문과 칸에 걸친
 * 선택은 평문 하나로 이을 순서가 정해져 있지 않아 `null` 로 돌려준다.
 */
import type { CellPathEntry, DocumentPosition } from '@/core/types';

/** 이 모듈이 문서에서 읽는 것의 전부. 테스트는 이것만 흉내 낸다. */
export interface SelectionTextReader {
  getParagraphLength(sec: number, para: number): number;
  getTextRange(sec: number, para: number, offset: number, count: number): string;
  getCellParagraphLengthByPath(sec: number, parentPara: number, pathJson: string): number;
  getTextInCellByPath(sec: number, parentPara: number, pathJson: string, offset: number, count: number): string;
}

export interface SelectionTextResult {
  text: string;
  truncated: boolean;
}

/** 레거시 평면 셀 좌표까지 경로로 정규화한다. 본문 문단이면 빈 배열. */
export function cellPathOf(pos: DocumentPosition): CellPathEntry[] | null {
  if (pos.cellPath && pos.cellPath.length > 0) {
    return pos.cellPath.map(({ controlIndex, cellIndex, cellParaIndex }) =>
      ({ controlIndex, cellIndex, cellParaIndex }));
  }
  if (pos.parentParaIndex !== undefined && pos.controlIndex !== undefined) {
    return [{
      controlIndex: pos.controlIndex,
      cellIndex: pos.cellIndex ?? 0,
      cellParaIndex: pos.cellParaIndex ?? 0,
    }];
  }
  return pos.parentParaIndex === undefined ? [] : null;
}

/** 마지막 칸 문단 번호만 다른 같은 칸인가. */
function sameCell(a: CellPathEntry[], b: CellPathEntry[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((entry, i) => entry.controlIndex === b[i].controlIndex
    && entry.cellIndex === b[i].cellIndex
    && (i === a.length - 1 || entry.cellParaIndex === b[i].cellParaIndex));
}

/**
 * 정렬된 선택(`start` ≤ `end`)의 평문. 문단 사이는 `\n` 으로 잇는다.
 * `maxChars` 를 넘으면 앞쪽만 담고 `truncated` 를 켠다 — 조용히 자르지 않는다.
 */
export function readSelectionText(
  reader: SelectionTextReader,
  start: DocumentPosition,
  end: DocumentPosition,
  maxChars: number,
): SelectionTextResult | null {
  if (start.isTextBox || end.isTextBox) return null;
  if (start.sectionIndex !== end.sectionIndex) return null;
  const sec = start.sectionIndex;
  const startPath = cellPathOf(start);
  const endPath = cellPathOf(end);
  if (!startPath || !endPath) return null;

  let readPara: (index: number, from: number, to: number | null) => string;
  let firstPara: number;
  let lastPara: number;

  if (startPath.length === 0 && endPath.length === 0) {
    firstPara = start.paragraphIndex;
    lastPara = end.paragraphIndex;
    readPara = (para, from, to) => {
      const length = reader.getParagraphLength(sec, para);
      const stop = to === null ? length : Math.min(to, length);
      return stop > from ? reader.getTextRange(sec, para, from, stop - from) : '';
    };
  } else if (startPath.length > 0 && sameCell(startPath, endPath)
      && start.parentParaIndex === end.parentParaIndex) {
    const parentPara = start.parentParaIndex!;
    firstPara = startPath[startPath.length - 1].cellParaIndex;
    lastPara = endPath[endPath.length - 1].cellParaIndex;
    readPara = (cellPara, from, to) => {
      const path = startPath.map((entry, i) =>
        (i === startPath.length - 1 ? { ...entry, cellParaIndex: cellPara } : entry));
      const pathJson = JSON.stringify(path);
      const length = reader.getCellParagraphLengthByPath(sec, parentPara, pathJson);
      const stop = to === null ? length : Math.min(to, length);
      return stop > from ? reader.getTextInCellByPath(sec, parentPara, pathJson, from, stop - from) : '';
    };
  } else {
    return null;
  }

  if (lastPara < firstPara || (lastPara === firstPara && end.charOffset <= start.charOffset)) {
    return null;
  }

  const parts: string[] = [];
  let total = 0;
  let truncated = false;
  for (let para = firstPara; para <= lastPara; para += 1) {
    const from = para === firstPara ? start.charOffset : 0;
    const to = para === lastPara ? end.charOffset : null;
    let piece = readPara(para, from, to);
    const separator = parts.length > 0 ? 1 : 0;
    if (total + separator + piece.length > maxChars) {
      piece = piece.slice(0, Math.max(0, maxChars - total - separator));
      // 대리 쌍 한가운데서 자르지 않는다.
      if (/[\uD800-\uDBFF]$/.test(piece)) piece = piece.slice(0, -1);
      parts.push(piece);
      truncated = true;
      break;
    }
    parts.push(piece);
    total += separator + piece.length;
  }
  return { text: parts.join('\n'), truncated };
}
