/**
 * 드래그&드롭 로컬 파일 열기 확인 대화상자 (#1439).
 *
 * 보안: 드롭 한 번으로 로컬 파일이 읽히던 동작을 기본에서 제외하고, 사용자가
 * 명시적으로 [열기]를 눌러야 로딩하도록 게이트한다. 미동의(취소/×/Escape/밖 클릭)는
 * `false` 로 resolve 되어 로딩하지 않는다 — 보안 기본값 안전.
 *
 * 크롬 확장 모드에서도 동작: 확장은 `/rhwp/` 를 standalone 탭으로 열므로 popup
 * 제약 없이 이 DOM 모달이 정상 렌더된다. `chrome` API 의존 없음.
 */
import { ModalDialog } from './dialog';

class DropConfirmDialog extends ModalDialog {
  private resolve!: (value: boolean) => void;

  constructor(private readonly fileName: string) {
    super('로컬 파일 열기 확인', 420);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:16px 20px;line-height:1.6;white-space:pre-line;';
    const name = this.fileName || '선택한 파일';
    body.textContent =
      `드래그한 로컬 파일을 엽니다.\n\n"${name}"\n\n` +
      `이 동작은 로컬 파일의 내용을 읽습니다. 계속하시겠습니까?`;
    return body;
  }

  protected onConfirm(): void {
    this.resolve(true); // [열기]
  }

  override hide(): void {
    this.resolve(false); // 취소/×/Escape/밖 클릭 — 미동의
    super.hide();
  }

  showAsync(): Promise<boolean> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (value: boolean) => {
        if (!resolved) {
          resolved = true;
          resolve(value);
        }
      };

      super.show();

      const footer = this.dialog.querySelector('.dialog-footer');
      const okBtn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
      const cancelBtn = footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)') as HTMLButtonElement | null;
      if (okBtn) okBtn.textContent = '열기';
      if (cancelBtn) cancelBtn.textContent = '취소';
    });
  }
}

/**
 * 드롭한 로컬 파일을 열지 사용자에게 확인한다.
 * @returns [열기]=true, 미동의=false
 */
export function showDropConfirmDialog(fileName: string): Promise<boolean> {
  return new DropConfirmDialog(fileName).showAsync();
}
