import type { CellPathEntry } from '@/core/types';

export interface TableObjectClipboardRef {
  ci: number;
  cellPath?: CellPathEntry[];
}

export interface TableObjectClipboardTarget {
  controlIndex: number;
  ownerCellPathJson: string;
}

/**
 * 표 객체 선택 참조를 native clipboard API가 요구하는 주소로 변환한다.
 *
 * 중첩 표 선택의 전체 cellPath는 선택된 표의 셀까지 포함한다. 따라서 마지막
 * 엔트리의 controlIndex가 선택된 표이고, 앞쪽 prefix가 그 표를 소유한 문단이다.
 * 본문 표는 기존 ref.ci와 빈 경로 계약을 유지한다.
 */
export function tableObjectClipboardTarget(
  ref: TableObjectClipboardRef,
): TableObjectClipboardTarget {
  const cellPath = ref.cellPath ?? [];
  if (cellPath.length > 1) {
    return {
      controlIndex: cellPath[cellPath.length - 1].controlIndex,
      ownerCellPathJson: JSON.stringify(cellPath.slice(0, -1)),
    };
  }
  return { controlIndex: ref.ci, ownerCellPathJson: '' };
}
