export interface CellBlockCtrlShiftSContext {
  inCellSelectionMode: boolean;
  blockSumEnabled: boolean;
  saveAsEnabled: boolean;
}

export type ContextualShortcutResolution =
  | { kind: 'dispatch'; commandId: 'table:block-sum' | 'file:save-as' }
  | { kind: 'consume' }
  | null;

export type CellBlockLetterShortcutResolution =
  | { kind: 'dispatch'; commandId: 'table:cell-split' | 'table:cell-merge' }
  | null;

/**
 * Ctrl/Cmd+Shift+S는 일반 문맥에서는 다른 이름으로 저장이지만, F5 셀 블록에서는
 * 한컴 호환 블록 합계다. embed처럼 Save As 소유권이 없는 프로파일에서는 후순위
 * 표 명령으로 우회하지 않고 브라우저 기본 동작만 막는다.
 */
export function resolveCellBlockCtrlShiftS(
  event: Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'>,
  context: CellBlockCtrlShiftSContext,
): ContextualShortcutResolution {
  const ctrlOrMeta = event.ctrlKey || event.metaKey;
  const isS = event.key.toLowerCase() === 's'
    || event.key === 'ㄴ'
    || event.code.toLowerCase() === 'keys';

  if (!ctrlOrMeta || !event.shiftKey || event.altKey || !isS) return null;
  if (!context.inCellSelectionMode) return null;
  if (!context.saveAsEnabled) return { kind: 'consume' };
  if (context.blockSumEnabled) {
    return { kind: 'dispatch', commandId: 'table:block-sum' };
  }
  return { kind: 'dispatch', commandId: 'file:save-as' };
}

/**
 * F5 셀 블록의 수정자 없는 물리 S/M을 입력 언어와 무관하게 셀 명령으로 해석한다.
 * Shift는 대문자 S/M 입력에 쓰일 수 있어 허용하고, Ctrl/Meta/Alt가 있으면 다른 단축키
 * 소유권을 침범하지 않는다. `Process`는 code가 실제 물리 키를 알려줄 때만 처리한다.
 */
export function resolveCellBlockLetterShortcut(
  event: Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'altKey'>,
  context: { inCellSelectionMode: boolean },
): CellBlockLetterShortcutResolution {
  if (!context.inCellSelectionMode || event.ctrlKey || event.metaKey || event.altKey) return null;

  const key = event.key.toLowerCase();
  const code = event.code.toLowerCase();
  if (key === 's' || event.key === 'ㄴ' || code === 'keys') {
    return { kind: 'dispatch', commandId: 'table:cell-split' };
  }
  if (key === 'm' || event.key === 'ㅡ' || code === 'keym') {
    return { kind: 'dispatch', commandId: 'table:cell-merge' };
  }
  return null;
}
