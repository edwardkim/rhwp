/**
 * 저장되지 않은 변경사항 확인 대화상자.
 *
 * 브라우저 창/탭 닫기는 beforeunload 기본 확인창만 사용할 수 있으므로,
 * 이 대화상자는 앱 내부 문서 교체 동작에서만 사용한다.
 */
import { ModalDialog } from './dialog';

export type UnsavedChangesChoice = 'save' | 'discard' | 'cancel';

interface UnsavedChangesDialogOptions {
  fileName: string;
  canSave: boolean;
}

class UnsavedChangesDialog extends ModalDialog {
  private resolve!: (value: UnsavedChangesChoice) => void;

  constructor(private readonly options: UnsavedChangesDialogOptions) {
    super('저장 확인', 420);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '16px 20px';
    body.style.lineHeight = '1.6';
    body.style.whiteSpace = 'pre-line';

    const fileName = this.options.fileName || '현재 문서';
    body.textContent = this.options.canSave
      ? `"${fileName}" 문서에 저장하지 않은 변경사항이 있습니다.\n계속하기 전에 저장하시겠습니까?`
      : `"${fileName}" 문서에 저장하지 않은 변경사항이 있습니다.\n이 문서는 현재 직접 저장할 수 없습니다. 변경사항을 버리고 계속할 수 있습니다.`;

    return body;
  }

  protected onConfirm(): void {
    this.resolve('save');
  }

  override hide(): void {
    this.resolve('cancel');
    super.hide();
  }

  showAsync(): Promise<UnsavedChangesChoice> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (value: UnsavedChangesChoice) => {
        if (!resolved) {
          resolved = true;
          resolve(value);
        }
      };

      super.show();

      const footer = this.dialog.querySelector('.dialog-footer');
      const saveBtn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
      const cancelBtn = footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)') as HTMLButtonElement | null;

      if (saveBtn) {
        saveBtn.textContent = '저장';
        saveBtn.disabled = !this.options.canSave;
        saveBtn.title = this.options.canSave ? '' : 'HWPX 문서는 현재 직접 저장할 수 없습니다.';
      }
      if (cancelBtn) {
        cancelBtn.textContent = '취소';
      }

      const discardBtn = document.createElement('button');
      discardBtn.type = 'button';
      discardBtn.className = 'dialog-btn';
      discardBtn.textContent = '저장 안 함';
      discardBtn.addEventListener('click', () => {
        this.resolve('discard');
        super.hide();
      });
      footer?.insertBefore(discardBtn, cancelBtn ?? null);
    });
  }
}

export function showUnsavedChangesDialog(options: UnsavedChangesDialogOptions): Promise<UnsavedChangesChoice> {
  return new UnsavedChangesDialog(options).showAsync();
}
