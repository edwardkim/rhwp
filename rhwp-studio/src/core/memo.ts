import type { HyperlinkTarget } from './hyperlink';

/** `getMemoList()` 경로 항목. 글상자는 칸 번호 없이 0번 칸으로 다룬다. */
export type MemoPathEntry =
  | { type: 'cell'; controlIndex: number; cellIndex: number; paraIndex: number }
  | { type: 'textbox'; controlIndex: number; paraIndex: number };

/** `getMemoList()` 항목의 기준 글자 위치. 위치를 알 수 없는 메모는 null 이다. */
export interface MemoAnchor {
  sectionIndex: number;
  paraIndex: number | null;
  startCharIdx: number | null;
  endCharIdx: number | null;
  path: MemoPathEntry[] | null;
}

/**
 * `target` 문단의 `[start, end)` 글자 삭제를 글자 재삽입으로 되돌릴 수 없는 메모가 있는지 판정한다.
 *
 * - 기준 글자가 모두 지워지면 코어가 메모를 같은 편집에서 지운다.
 * - 메모 경계에 다시 넣은 글자는 메모 범위에 붙는다. 기준 바로 앞·뒤 글자를 지운 뒤 재삽입하면
 *   범위가 넓어지고, 그 자리의 길이 0 메모는 재삽입한 글자 앞에 남는다.
 *
 * 그래서 메모 범위가 삭제 범위와 겹치거나 맞닿으면(양 끝 포함) 참이다.
 */
export function touchesMemoAnchor(
  anchors: readonly MemoAnchor[], target: HyperlinkTarget, start: number, end: number,
): boolean {
  return anchors.some(memo => memo.sectionIndex === target.section
    && memo.paraIndex === target.para
    && memo.startCharIdx !== null && memo.endCharIdx !== null
    && memo.startCharIdx <= end && start <= memo.endCharIdx
    && sameCellPath(memo.path ?? [], target.cellPath));
}

function sameCellPath(path: readonly MemoPathEntry[], cellPath: readonly [number, number, number][]): boolean {
  return path.length === cellPath.length && path.every((entry, i) => {
    const [controlIndex, cellIndex, paraIndex] = cellPath[i];
    return entry.controlIndex === controlIndex
      && (entry.type === 'cell' ? entry.cellIndex : 0) === cellIndex
      && entry.paraIndex === paraIndex;
  });
}
