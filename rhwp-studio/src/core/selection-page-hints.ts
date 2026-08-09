import type { SelectionRect } from './types';

export interface SelectionPageHints {
  startPageHint: number;
  endPageHint: number;
}

export interface CellSelectionRectQuery {
  sectionIdx: number;
  parentParaIdx: number;
  controlIdx: number;
  cellIdx: number;
  startCellParaIdx: number;
  startCharOffset: number;
  endCellParaIdx: number;
  endCharOffset: number;
}

export interface CellSelectionRectDocument {
  getSelectionRectsInCell(
    sectionIdx: number,
    parentParaIdx: number,
    controlIdx: number,
    cellIdx: number,
    startCellParaIdx: number,
    startCharOffset: number,
    endCellParaIdx: number,
    endCharOffset: number,
  ): string;
  getSelectionRectsInCellEx?: (optionsJson: string) => string;
}

export interface PathCellSelectionRectQuery {
  sectionIdx: number;
  parentParaIdx: number;
  /** JSON.stringify(cellPath) 결과. WASM options API에는 문자열로 전달한다. */
  path: string;
  startCellParaIdx: number;
  startCharOffset: number;
  endCellParaIdx: number;
  endCharOffset: number;
}

export interface PathCellSelectionRectDocument {
  getSelectionRectsInCellByPath(
    sectionIdx: number,
    parentParaIdx: number,
    path: string,
    startCellParaIdx: number,
    startCharOffset: number,
    endCellParaIdx: number,
    endCharOffset: number,
  ): string;
  getSelectionRectsInCellByPathEx?: (optionsJson: string) => string;
}

function isValidPageHint(value: unknown): value is number {
  return Number.isInteger(value) && Number(value) >= 0;
}

/**
 * #2215 cell selection rect 조회의 호환 dispatch.
 *
 * 두 endpoint page와 Ex API가 모두 있을 때만 hinted path를 사용한다. page hint가
 * 불완전하거나 구버전 WASM이 Ex를 노출하지 않으면 기존 positional API로 복구한다.
 */
export function getSelectionRectsInCellWithPageHints(
  doc: CellSelectionRectDocument,
  query: CellSelectionRectQuery,
  pageHints?: SelectionPageHints,
): SelectionRect[] {
  const canUseHints = isValidPageHint(pageHints?.startPageHint)
    && isValidPageHint(pageHints?.endPageHint)
    && typeof doc.getSelectionRectsInCellEx === 'function';

  const json = canUseHints
    ? doc.getSelectionRectsInCellEx!(JSON.stringify({
      ...query,
      startPageHint: pageHints.startPageHint,
      endPageHint: pageHints.endPageHint,
    }))
    : doc.getSelectionRectsInCell(
      query.sectionIdx,
      query.parentParaIdx,
      query.controlIdx,
      query.cellIdx,
      query.startCellParaIdx,
      query.startCharOffset,
      query.endCellParaIdx,
      query.endCharOffset,
    );

  return JSON.parse(json) as SelectionRect[];
}

/**
 * #4272 중첩 셀 선택 rect 조회의 호환 dispatch.
 *
 * path API 자체가 정확성 계약이므로 구버전 평면 API로는 폴백하지 않는다. page hint가
 * 없거나 Ex가 없을 때만 동일한 path positional API를 사용한다.
 */
export function getSelectionRectsInCellByPathWithPageHints(
  doc: PathCellSelectionRectDocument,
  query: PathCellSelectionRectQuery,
  pageHints?: SelectionPageHints,
): SelectionRect[] {
  const canUseHints = isValidPageHint(pageHints?.startPageHint)
    && isValidPageHint(pageHints?.endPageHint)
    && typeof doc.getSelectionRectsInCellByPathEx === 'function';

  const json = canUseHints
    ? doc.getSelectionRectsInCellByPathEx!(JSON.stringify({
      ...query,
      startPageHint: pageHints.startPageHint,
      endPageHint: pageHints.endPageHint,
    }))
    : doc.getSelectionRectsInCellByPath(
      query.sectionIdx,
      query.parentParaIdx,
      query.path,
      query.startCellParaIdx,
      query.startCharOffset,
      query.endCellParaIdx,
      query.endCharOffset,
    );

  return JSON.parse(json) as SelectionRect[];
}
