/**
 * 다른 이름으로 저장 대화상자
 *
 * 새 문서 저장 시 파일 이름과 선택 암호 설정 요청을 받는다.
 * 이미 보호된 문서는 기본 확인이 암호를 계승하고, 평문은 명시적 선택만 허용한다 (#5991).
 * showSaveAs() 헬퍼로 간단히 사용 가능.
 */
import { ModalDialog } from './dialog';
import { fileNameForFormat, type SaveFormat } from '@/command/save-target';

export interface SaveAsDialogResult {
  fileName: string;
  configurePassword: boolean;
}

export interface SaveAsDialogOptions {
  allowPassword?: boolean;
  inheritPassword?: boolean;
}

class SaveAsDialog extends ModalDialog {
  private defaultName: string;
  private input!: HTMLInputElement;
  private resolve!: (value: SaveAsDialogResult | null) => void;

  constructor(
    defaultName: string,
    private readonly format: SaveFormat,
    private readonly allowPassword: boolean,
    private readonly inheritPassword: boolean,
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

    if (this.inheritPassword) {
      const note = document.createElement('p');
      note.style.margin = '12px 0 0';
      note.style.fontSize = '12px';
      note.style.lineHeight = '1.5';
      note.textContent =
        '이 문서는 암호로 보호되어 있습니다. 확인하면 암호를 다시 입력합니다. 평문 사본은 암호 없이 저장을 선택하세요.';
      body.appendChild(note);
    }

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

      if (this.inheritPassword) {
        const plaintextButton = document.createElement('button');
        plaintextButton.type = 'button';
        plaintextButton.className = 'dialog-btn';
        plaintextButton.textContent = '암호 없이 저장';
        plaintextButton.style.marginTop = '8px';
        plaintextButton.addEventListener('click', () => {
          const value = this.confirmValue();
          if (value === null) return;
          this.resolve({ fileName: value, configurePassword: false });
          this.hide();
        });
        body.appendChild(plaintextButton);
      }
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
    // 보호된 문서의 기본 확인은 암호를 계승한다. 평문은 `암호 없이 저장`만.
    this.resolve({ fileName, configurePassword: this.inheritPassword });
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
 * `inheritPassword`가 켜지면 기본 확인이 암호 입력으로 이어지고, 평문은
 * `암호 없이 저장`을 눌러야 한다 (#5991).
 */
export function showSaveAs(
  defaultName: string,
  format: SaveFormat = 'hwp',
  options: SaveAsDialogOptions = {},
): Promise<SaveAsDialogResult | null> {
  return new SaveAsDialog(
    defaultName,
    format,
    options.allowPassword === true,
    options.inheritPassword === true,
  ).showAsync();
}
