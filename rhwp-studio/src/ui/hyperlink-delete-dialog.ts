import { ModalDialog } from './dialog';

import { t } from '../i18n/index.ts';
class DeleteHyperlinkDialog extends ModalDialog {
  private accepted = false;
  constructor(private remove: () => void, private cancel: () => void) { super(t('dialog.hyperlinkDelete.title'), 390); }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-hyperlink-confirm-body';
    body.textContent = t('dialog.hyperlinkDelete.body.text');
    return body;
  }
  protected onConfirm(): void { this.accepted = true; }
  override show(): void {
    super.show();
    this.dialog.setAttribute('role', 'alertdialog');
    this.dialog.setAttribute('aria-label', t('dialog.hyperlinkDelete.show.label'));
    this.dialog.querySelector('.dialog-btn-primary')!.textContent = t('dialog.hyperlinkDelete.show.text');
  }
  override hide(): void {
    super.hide();
    if (this.accepted) this.remove();
    else this.cancel();
  }
}

export function confirmHyperlinkDelete(remove: () => void, cancel: () => void): void {
  new DeleteHyperlinkDialog(remove, cancel).show();
}
