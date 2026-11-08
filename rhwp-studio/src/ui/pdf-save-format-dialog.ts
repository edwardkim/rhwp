import { ModalDialog } from './dialog';
import { t } from '../i18n/index.ts';

type PdfSaveFormat = 'hwp' | 'hwpx' | null;

class PdfSaveFormatDialog extends ModalDialog {
  private finish!: (value: PdfSaveFormat) => void;

  constructor() {
    super(t('dialog.pdfSaveFormat.title'), 440);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('p');
    body.textContent = t('dialog.pdfSaveFormat.message');
    return body;
  }

  protected onConfirm(): void {
    this.finish('hwpx');
  }

  override hide(): void {
    this.finish(null);
    super.hide();
  }

  showAsync(): Promise<PdfSaveFormat> {
    return new Promise(resolve => {
      let settled = false;
      this.finish = value => { if (!settled) { settled = true; resolve(value); } };
      super.show();
      const primary = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement;
      primary.textContent = t('dialog.pdfSaveFormat.hwpx');
      primary.dataset.saveFormat = 'hwpx';
      const hwp = document.createElement('button');
      hwp.type = 'button';
      hwp.className = 'dialog-btn';
      hwp.dataset.saveFormat = 'hwp';
      hwp.textContent = t('dialog.pdfSaveFormat.hwp');
      hwp.addEventListener('click', () => { this.finish('hwp'); super.hide(); });
      primary.after(hwp);
    });
  }
}

export function showPdfSaveFormatDialog(): Promise<PdfSaveFormat> {
  return new PdfSaveFormatDialog().showAsync();
}
