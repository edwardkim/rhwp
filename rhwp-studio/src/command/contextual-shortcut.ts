export interface CellBlockCtrlShiftSContext {
  inCellSelectionMode: boolean;
  blockSumEnabled: boolean;
  saveAsEnabled: boolean;
}

export type ContextualShortcutResolution =
  | { kind: 'dispatch'; commandId: 'table:block-sum' | 'file:save-as' }
  | { kind: 'consume' }
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
