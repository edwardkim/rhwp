/**
 * 다른 이름으로 저장 대화상자
 *
 * 새 문서 저장 시 파일 이름과 선택 암호 설정 요청을 받는다.
 * showSaveAs() 헬퍼로 간단히 사용 가능.
 */
import { ModalDialog } from './dialog';
import { fileNameForFormat, type SaveFormat } from '@/command/save-target';

export interface SaveAsDialogResult {
  fileName: string;
  configurePassword: boolean;
}

class SaveAsDialog extends ModalDialog {
  private defaultName: string;
  private input!: HTMLInputElement;
  private resolve!: (value: SaveAsDialogResult | null) => void;

  constructor(
    defaultName: string,
    private readonly format: SaveFormat,
    private readonly allowPassword: boolean,
  ) {
    super('다른 이름으로 저장', 380);
    this.defaultName = defaultName;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '16px 20px';

    const label = document.createElement('label');
    label.textContent = '파일 이름(N):';
    label.style.display = 'block';
    label.style.marginBottom = '6px';
    label.style.fontSize = '13px';
    body.appendChild(label);

    this.input = document.createElement('input');
    this.input.type = 'text';
    this.input.value = this.defaultName;
    this.input.style.width = '100%';
    this.input.style.boxSizing = 'border-box';
    this.input.style.height = '26px';
    this.input.style.padding = '2px 6px';
    this.input.style.border = '1px solid #b4b4b4';
    this.input.style.fontSize = '13px';
    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        if (this.onConfirm()) this.hide();
      }
    });
    body.appendChild(this.input);

    if (this.allowPassword) {
      const passwordButton = document.createElement('button');
      passwordButton.type = 'button';
      passwordButton.className = 'dialog-btn';
      passwordButton.textContent = '암호 설정...';
      passwordButton.style.marginTop = '12px';
      passwordButton.addEventListener('click', () => {
        const value = this.confirmValue();
        if (value === null) return;
        this.resolve({ fileName: value, configurePassword: true });
        this.hide();
      });
      body.appendChild(passwordButton);
    }

    return body;
  }

  private confirmValue(): string | null {
    const name = this.input.value.trim();
    if (!name) {
      this.input.focus();
      return null;
    }
    return fileNameForFormat(name, this.format);
  }

  protected onConfirm(): boolean {
    const fileName = this.confirmValue();
    if (fileName === null) return false;
    this.resolve({ fileName, configurePassword: false });
    return true;
  }

  override hide(): void {
    this.resolve(null);
    super.hide();
  }

  showAsync(): Promise<SaveAsDialogResult | null> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (v: SaveAsDialogResult | null) => {
        if (!resolved) {
          resolved = true;
          resolve(v);
        }
      };
      super.show();
      requestAnimationFrame(() => {
        this.input.focus();
        this.input.select();
      });
    });
  }
}

/**
 * 파일 이름 입력 대화상자를 표시한다. HWP/HWPX에서 `allowPassword`를 켜면 사용자가
 * `암호 설정...`을 선택해 다음 암호 입력 대화상자로 진행할 수 있다.
 */
export function showSaveAs(
  defaultName: string,
  format: SaveFormat = 'hwp',
  options: { allowPassword?: boolean } = {},
): Promise<SaveAsDialogResult | null> {
  return new SaveAsDialog(defaultName, format, options.allowPassword === true).showAsync();
}
