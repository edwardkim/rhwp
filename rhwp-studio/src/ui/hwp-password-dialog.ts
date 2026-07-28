import { ModalDialog } from './dialog';

class HwpPasswordDialog extends ModalDialog {
  private input!: HTMLInputElement;
  private resolve!: (value: string | null) => void;
  private inputEnterHandler: ((event: KeyboardEvent) => void) | null = null;

  constructor(private readonly fileName: string, private readonly errorMessage?: string) {
    super('문서 암호', 420);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:16px 20px;line-height:1.6;';

    const message = document.createElement('p');
    message.textContent = `"${this.fileName || '선택한 문서'}"을(를) 열려면 암호를 입력하세요.`;
    body.appendChild(message);

    const label = document.createElement('label');
    label.htmlFor = 'hwp-password-input';
    label.textContent = '문서 암호';
    body.appendChild(label);

    this.input = document.createElement('input');
    this.input.id = 'hwp-password-input';
    this.input.type = 'password';
    // 브라우저의 암호 관리자·자동 완성 대상이 되지 않게 한다. 이 입력은 계정 인증이
    // 아니라 선택한 로컬 문서 한 번을 여는 데만 쓰이며, Studio는 이를 보관하지 않는다.
    this.input.autocomplete = 'off';
    this.input.setAttribute('aria-describedby', 'hwp-password-help');
    this.input.style.cssText = 'display:block;width:100%;box-sizing:border-box;margin-top:6px;height:28px;';
    body.appendChild(this.input);

    const help = document.createElement('p');
    help.id = 'hwp-password-help';
    help.textContent = '입력한 암호는 이 문서를 여는 동안에만 사용하며 저장하지 않습니다.';
    body.appendChild(help);

    if (this.errorMessage) {
      const error = document.createElement('p');
      error.id = 'hwp-password-error';
      error.setAttribute('role', 'alert');
      error.textContent = this.errorMessage;
      body.appendChild(error);
      this.input.setAttribute('aria-describedby', 'hwp-password-help hwp-password-error');
    }
    return body;
  }

  protected onConfirm(): void {
    this.resolve(this.input.value);
  }

  override hide(): void {
    if (this.inputEnterHandler) {
      document.removeEventListener('keydown', this.inputEnterHandler, true);
      this.inputEnterHandler = null;
    }
    if (this.input) this.input.value = '';
    this.resolve(null);
    super.hide();
  }

  showAsync(): Promise<string | null> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (value) => {
        if (!resolved) {
          resolved = true;
          resolve(value);
        }
      };
      super.show();
      this.dialog.setAttribute('role', 'dialog');
      this.dialog.setAttribute('aria-modal', 'true');
      this.dialog.setAttribute('aria-label', '문서 암호 입력');
      // ModalDialog가 document capture 단계에서 편집 영역 밖으로 키 이벤트가 새는 것을
      // 막는다. 같은 capture 대상에 후속 등록해 Enter를 직접 처리하면 입력값을 실제로
      // 받을 수 있고, 편집기 단축키에는 전달되지 않는다.
      this.inputEnterHandler = (event) => {
        if (event.target === this.input && event.key === 'Enter') {
          event.preventDefault();
          this.onConfirm();
          this.hide();
        }
      };
      document.addEventListener('keydown', this.inputEnterHandler, true);
      requestAnimationFrame(() => this.input.focus());
    });
  }
}

export function showHwpPasswordDialog(fileName: string, errorMessage?: string): Promise<string | null> {
  return new HwpPasswordDialog(fileName, errorMessage).showAsync();
}
