import type { CommandServices } from '@/command/types';
import type { CellPathEntry } from '@/core/types';

/**
 * 줄/칸 지우기 후 커서 셀 보정 (#1483).
 *
 * 삭제로 셀 수가 줄면 기존 cellIndex가 새 표 범위를 벗어나 updateRect가 "셀 인덱스 초과"로
 * 실패한다. 삭제 후 표 크기(rowCount/colCount) 내로 (row,col)을 clamp하고, 해당 위치의
 * cellIndex를 getTableCellBboxes로 역조회한다 (병합 셀은 rowSpan/colSpan 범위로 매칭).
 * 표가 소멸(rowCount/colCount<=0)하면 null을 반환한다.
 */
export function clampedCellAfterDelete(
  wasm: CommandServices['wasm'],
  sec: number,
  parentPara: number,
  controlIdx: number,
  origRow: number,
  origCol: number,
  rowCount: number,
  colCount: number,
  cellPath?: CellPathEntry[],
): { cellIndex: number; cellParaIndex: number; paragraphIndex: number; cellPath?: CellPathEntry[] } | null {
  if (rowCount <= 0 || colCount <= 0) return null;
  const row = Math.min(origRow, rowCount - 1);
  const col = Math.min(origCol, colCount - 1);
  const bboxes = wasm.getTableCellBboxes(sec, parentPara, controlIdx);
  const hit = bboxes.find(
    (b) =>
      row >= b.row && row < b.row + b.rowSpan &&
      col >= b.col && col < b.col + b.colSpan,
  );
  const cellIndex = hit ? hit.cellIdx : 0;
  return {
    cellIndex,
    cellParaIndex: 0,
    paragraphIndex: 0,
    cellPath: cellPath?.map((entry, index) => index === cellPath.length - 1
      ? { ...entry, cellIndex, cellParaIndex: 0 }
      : { ...entry }),
  };
}
