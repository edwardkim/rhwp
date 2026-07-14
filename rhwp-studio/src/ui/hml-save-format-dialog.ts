import { ModalDialog } from './dialog';
import type { HmlOpenMetadata } from '../core/wasm-bridge';
import { resolveHmlSaveCapability } from '../core/hml-save-capability';
import { buildHmlSaveFormatMessage } from './hml-save-format-message';

export type HmlSaveFormatChoice = 'hml' | 'hwp' | 'hwpx' | null;

class HmlSaveFormatDialog extends ModalDialog {
  private resolve!: (value: HmlSaveFormatChoice) => void;
  private readonly hmlEnabled: boolean;
  private readonly message: string;

  constructor(
    metadata: HmlOpenMetadata | null,
    exporterAvailable: boolean,
  ) {
    super('HML 문서 저장', 440);
    const capability = resolveHmlSaveCapability(metadata, exporterAvailable);
    this.hmlEnabled = capability.hmlEnabled;
    this.message = buildHmlSaveFormatMessage(metadata, exporterAvailable);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '16px 20px';
    body.style.lineHeight = '1.6';
    body.style.whiteSpace = 'pre-line';

    body.textContent = this.message;
    return body;
  }

  protected onConfirm(): void {
    this.resolve(this.hmlEnabled ? 'hml' : 'hwp');
  }

  override hide(): void {
    this.resolve(null);
    super.hide();
  }

  showAsync(): Promise<HmlSaveFormatChoice> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (value) => {
        if (resolved) return;
        resolved = true;
        resolve(value);
      };

      super.show();
      const footer = this.dialog.querySelector('.dialog-footer');
      const primaryButton = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
      const cancelButton = footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)') as HTMLButtonElement | null;

      primaryButton?.classList.add('hml-save-format-primary');

      const addFormatButton = (label: string, choice: HmlSaveFormatChoice, disabled = false): void => {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'dialog-btn';
        button.textContent = label;
        if (disabled) {
          button.disabled = true;
        } else {
          button.addEventListener('click', () => {
            this.resolve(choice);
            super.hide();
          });
        }
        footer?.insertBefore(button, cancelButton ?? null);
      };

      if (this.hmlEnabled) {
        if (primaryButton) primaryButton.textContent = 'HML로 저장';
        addFormatButton('HWP로 저장', 'hwp');
      } else {
        if (primaryButton) primaryButton.textContent = 'HWP로 저장';
        addFormatButton('HML로 저장 (저장 불가)', null, true);
      }
      addFormatButton('HWPX로 저장', 'hwpx');
    });
  }
}

export function showHmlSaveFormatDialog(
  metadata: HmlOpenMetadata | null = null,
  exporterAvailable = false,
): Promise<HmlSaveFormatChoice> {
  return new HmlSaveFormatDialog(metadata, exporterAvailable).showAsync();
}
