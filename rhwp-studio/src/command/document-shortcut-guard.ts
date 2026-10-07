/**
 * 글자를 편집하는 요소(input·textarea·contentEditable·role=textbox)와 그 자손은
 * 자기 단축키를 스스로 처리한다. select는 여기 넣지 않는다 — 문서 이동 키(PgUp/PgDn 등)는
 * 툴바 콤보보다 문서를 우선한다.
 */
export function isTextEditingTarget(target: EventTarget | null): boolean {
  let element = target as HTMLElement | null;
  while (element) {
    if (['INPUT', 'TEXTAREA'].includes(element.tagName)
      || element.isContentEditable
      || element.getAttribute?.('role') === 'textbox') return true;
    element = element.parentElement;
  }
  return false;
}

/** ⌘A 판정용: 글자 편집 요소에 더해 select 안에서도 문서 전체 선택을 하지 않는다. */
export function isEditableShortcutTarget(target: EventTarget | null): boolean {
  if (isTextEditingTarget(target)) return true;
  let element = target as HTMLElement | null;
  while (element) {
    if (element.tagName === 'SELECT') return true;
    element = element.parentElement;
  }
  return false;
}

/** Handle document ⌘A only when no modal or editable control owns the key. */
export function handleDocumentSelectAllShortcut(
  event: Pick<KeyboardEvent, 'target' | 'preventDefault'>,
  modalOverlay: Pick<Element, 'isConnected'> | null,
  dispatch: () => void,
  focusEditor: () => void,
): boolean {
  if (modalOverlay?.isConnected || isEditableShortcutTarget(event.target)) return false;
  event.preventDefault();
  dispatch();
  focusEditor();
  return true;
}
