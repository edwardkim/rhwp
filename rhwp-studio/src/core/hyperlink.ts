import type { DocumentPosition } from './types';

export interface HyperlinkTarget {
  section: number;
  para: number;
  cellPath: [number, number, number][];
}
export interface HyperlinkInfo {
  fieldId: number;
  start: number;
  end: number;
  text: string;
  uri: string;
}
export interface HyperlinkContext { text: string; links: HyperlinkInfo[] }

/** 캡션의 특수 셀 번호를 본문/글상자로 재해석하지 않는다. */
export function hyperlinkTarget(pos: DocumentPosition): HyperlinkTarget {
  const path: [number, number, number][] = pos.cellPath?.map(
    p => [p.controlIndex, p.cellIndex, p.cellParaIndex],
  ) ?? [];
  if (pos.parentParaIndex !== undefined && path.length === 0) {
    if (pos.controlIndex === undefined || pos.cellIndex === undefined || pos.cellParaIndex === undefined) {
      throw new Error('셀 위치를 확인할 수 없습니다.');
    }
    path.push([pos.controlIndex, pos.cellIndex, pos.cellParaIndex]);
  }
  if (path.length && pos.parentParaIndex === undefined) throw new Error('중첩 셀 위치가 올바르지 않습니다.');
  if (path.some(p => p[1] === 65534)) throw new Error('캡션의 하이퍼링크 편집은 아직 지원하지 않습니다.');
  const target = { section: pos.sectionIndex, para: pos.parentParaIndex ?? pos.paragraphIndex, cellPath: path };
  if (![target.section, target.para, pos.charOffset, ...path.flat()].every(n => Number.isSafeInteger(n) && n >= 0)) {
    throw new Error('하이퍼링크 위치가 올바르지 않습니다.');
  }
  return target;
}

export function hyperlinkRange(pos: DocumentPosition, selection: { start: DocumentPosition; end: DocumentPosition } | null) {
  const start = selection?.start ?? pos;
  const end = selection?.end ?? pos;
  const target = hyperlinkTarget(start);
  if (JSON.stringify(target) !== JSON.stringify(hyperlinkTarget(end))) {
    throw new Error('한 문단 안의 글자를 선택해 주세요.');
  }
  return { target, pos: structuredClone(start), start: Math.min(start.charOffset, end.charOffset), end: Math.max(start.charOffset, end.charOffset) };
}

/** 경계에서는 다음 글자의 링크를 선택한다. 인접한 링크 두 개를 임의로 합치지 않는다. */
export function selectedHyperlink(links: HyperlinkInfo[], start: number, end: number): HyperlinkInfo | undefined {
  const hits = links.filter(l => start === end
    ? l.start <= start && start < l.end
    : start < l.end && l.start < end);
  if (hits.length > 1 || (hits[0] && start !== end && (start < hits[0].start || end > hits[0].end))) {
    throw new Error('하나의 하이퍼링크 안에서 편집해 주세요.');
  }
  return hits[0];
}

/** 링크 색/밑줄만 변경해 굵기·글꼴 등 기존 서식을 유지한다. snapshot 안에서 호출한다. */
export function applyHyperlinkFormat(wasm: import('./wasm-bridge').WasmBridge, target: HyperlinkTarget,
  start: number, end: number, color: string | null): void {
  const props = JSON.stringify({ textColor: color ?? '#000000', underlineType: color ? 'Bottom' : 'None', underlineColor: color ?? '#000000' });
  if (target.cellPath.length) {
    const path = JSON.stringify(target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) => ({ controlIndex, cellIndex, cellParaIndex })));
    wasm.applyCharFormatInCellByPath(target.section, target.para, path, start, end, props);
  } else wasm.applyCharFormat(target.section, target.para, start, end, props);
}
