/** Delete로 링크 글자를 지울 때 필드와 표시 문자열을 한 단위로 제거한다. */
import { hyperlinkTarget } from '@/core/hyperlink';
import type { DocumentPosition } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { OperationDescriptor } from './command';
import { confirmHyperlinkDelete } from '@/ui/hyperlink-delete-dialog';
import { showToast } from '@/ui/toast';

type DeleteHost = {
  wasm: WasmBridge;
  canEditHyperlink?(): boolean;
  isFormMode?(): boolean;
  isActive?(): boolean;
  focusTextarea(): void;
  executeOperation(operation: OperationDescriptor): unknown;
};
const pending = new WeakSet<DeleteHost>();

export function tryConfirmDeleteHyperlink(self: DeleteHost, position: DocumentPosition): boolean {
  if (pending.has(self)) return true;
  const canEdit = () => !!self.canEditHyperlink?.() && !self.isFormMode?.() && self.isActive?.() !== false;
  if (!canEdit()) return false;
  const pos = structuredClone(position);
  let target, context;
  try {
    target = hyperlinkTarget(pos);
    context = self.wasm.getHyperlinkContext(target);
  } catch { return false; } // 캡션 등 신규 링크 편집의 미지원 문맥은 기존 삭제 경로를 유지한다.
  const link = context.links.find(link => link.start <= pos.charOffset && pos.charOffset < link.end);
  if (!link) return false;
  const generation = self.wasm.documentGeneration;
  const before = JSON.stringify(context);
  const finish = () => { pending.delete(self); self.focusTextarea(); };
  pending.add(self);
  confirmHyperlinkDelete(() => {
    try {
      if (!canEdit() || self.wasm.documentGeneration !== generation
        || JSON.stringify(self.wasm.getHyperlinkContext(target)) !== before) {
        throw new Error('편집 상태가 바뀌었습니다. 하이퍼링크를 다시 선택해 주세요.');
      }
      self.executeOperation({
        kind: 'snapshot', operationType: 'deleteHyperlink', selectionBefore: null,
        operation: wasm => {
          // 링크 필드를 먼저 제거해 텍스트 삭제 뒤 빈 필드가 남지 않게 한다.
          // 둘째 mutation이 실패해도 SnapshotCommand가 전체 작업을 복구한다.
          wasm.removeHyperlink(target, link.fieldId);
          if (target.cellPath.length) {
            const path = JSON.stringify(target.cellPath.map(([controlIndex, cellIndex, cellParaIndex]) =>
              ({ controlIndex, cellIndex, cellParaIndex })));
            wasm.deleteTextInCellByPath(target.section, target.para, path, link.start, link.end - link.start);
          } else {
            wasm.deleteText(target.section, target.para, link.start, link.end - link.start);
          }
          return { ...pos, charOffset: link.start };
        },
      });
    } catch (error) {
      showToast({ message: error instanceof Error ? error.message : String(error) });
    } finally { finish(); }
  }, finish);
  return true;
}
