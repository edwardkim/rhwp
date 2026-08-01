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

class HwpSavePasswordDialog extends ModalDialog {
  private passwordInput!: HTMLInputElement;
  private confirmationInput!: HTMLInputElement;
  private error!: HTMLParagraphElement;
  private resolve!: (value: string | null) => void;
  private inputEnterHandler: ((event: KeyboardEvent) => void) | null = null;

  constructor(private readonly fileName: string) {
    super('문서 암호 설정', 420);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:16px 20px;line-height:1.6;';

    const message = document.createElement('p');
    message.textContent = `"${this.fileName || '저장할 문서'}"에 적용할 새 암호를 입력하세요.`;
    body.appendChild(message);

    this.passwordInput = this.createPasswordInput('hwp-save-password-input', '새 암호');
    body.appendChild(this.passwordInput.closest('label')!);
    this.confirmationInput = this.createPasswordInput('hwp-save-password-confirmation', '암호 확인');
    body.appendChild(this.confirmationInput.closest('label')!);

    this.error = document.createElement('p');
    this.error.setAttribute('role', 'alert');
    this.error.hidden = true;
    body.appendChild(this.error);

    const help = document.createElement('p');
    help.textContent = '암호는 한글 또는 영문 5자 이상으로 입력해야 합니다. 암호를 잊으면 문서를 열 수 없습니다.';
    body.appendChild(help);
    return body;
  }

  private createPasswordInput(id: string, labelText: string): HTMLInputElement {
    const label = document.createElement('label');
    label.htmlFor = id;
    label.textContent = labelText;
    label.style.cssText = 'display:block;margin-top:10px;';
    const input = document.createElement('input');
    input.id = id;
    input.type = 'password';
    // 문서 암호는 계정 자격 증명이 아니며 Studio가 보관하지 않으므로 암호 관리자 대상에서 제외한다.
    input.autocomplete = 'off';
    input.style.cssText = 'display:block;width:100%;box-sizing:border-box;margin-top:6px;height:28px;';
    label.appendChild(input);
    return input;
  }

  protected onConfirm(): boolean {
    const password = this.passwordInput.value;
    if (password.length < 5) {
      this.showError('암호는 5자 이상으로 입력하세요.');
      this.passwordInput.focus();
      return false;
    }
    if (password !== this.confirmationInput.value) {
      this.showError('새 암호와 암호 확인이 일치하지 않습니다.');
      this.confirmationInput.focus();
      return false;
    }
    this.resolve(password);
    return true;
  }

  private showError(message: string): void {
    this.error.textContent = message;
    this.error.hidden = false;
  }

  override hide(): void {
    if (this.inputEnterHandler) {
      document.removeEventListener('keydown', this.inputEnterHandler, true);
      this.inputEnterHandler = null;
    }
    if (this.passwordInput) this.passwordInput.value = '';
    if (this.confirmationInput) this.confirmationInput.value = '';
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
      this.dialog.setAttribute('aria-label', '문서 저장 암호 설정');
      this.inputEnterHandler = (event) => {
        if ((event.target === this.passwordInput || event.target === this.confirmationInput) && event.key === 'Enter') {
          event.preventDefault();
          if (this.onConfirm()) this.hide();
        }
      };
      document.addEventListener('keydown', this.inputEnterHandler, true);
      requestAnimationFrame(() => this.passwordInput.focus());
    });
  }
}

/** 새 암호와 확인 입력을 받아 HWP/HWPX 저장에 한 번만 사용한다. */
export function showHwpSavePasswordDialog(fileName: string): Promise<string | null> {
  return new HwpSavePasswordDialog(fileName).showAsync();
}
