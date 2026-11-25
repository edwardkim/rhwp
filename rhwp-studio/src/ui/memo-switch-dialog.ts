import { ModalDialog } from './dialog';
import { t } from '../i18n/index.ts';

type MemoSwitchChoice = 'apply' | 'discard' | 'continue';

export class MemoSwitchDialog extends ModalDialog {
  private choice: MemoSwitchChoice = 'continue';
  private resolve?: (choice: MemoSwitchChoice) => void;

  constructor(private destination: 'memo' | 'document') {
    super(t(destination === 'memo' ? 'dialog.memo.switchTitle' : 'dialog.memo.finishTitle'), 420);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('p');
    body.id = 'memo-switch-description';
    body.textContent = t('dialog.memo.switchMessage');
    return body;
  }

  protected onConfirm(): void {
    this.choice = 'apply';
  }

  override hide(): void {
    const resolve = this.resolve;
    if (!resolve) return;
    this.resolve = undefined;
    super.hide();
    resolve(this.choice);
  }

  ask(): Promise<MemoSwitchChoice> {
    return new Promise(resolve => {
      this.resolve = resolve;
      super.show();
      this.dialog.dataset.memoSwitch = '';
      this.dialog.setAttribute('role', 'alertdialog');
      this.dialog.setAttribute('aria-modal', 'true');
      const title = this.destination === 'memo' ? 'dialog.memo.switchTitle' : 'dialog.memo.finishTitle';
      this.dialog.setAttribute('aria-label', t(title));
      this.dialog.setAttribute('aria-describedby', 'memo-switch-description');
      const footer = this.dialog.querySelector('.dialog-footer')!;
      const apply = footer.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
      const keep = footer.querySelector<HTMLButtonElement>('.dialog-btn:not(.dialog-btn-primary)')!;
      apply.textContent = t(this.destination === 'memo' ? 'dialog.memo.switchApply' : 'dialog.memo.finishApply');
      keep.textContent = t('dialog.memo.switchContinue');
      this.dialog.querySelector('.dialog-close')!.setAttribute('aria-label', keep.textContent);
      const discard = document.createElement('button');
      discard.type = 'button';
      discard.className = 'dialog-btn memo-switch-discard';
      discard.textContent = t(this.destination === 'memo' ? 'dialog.memo.switchDiscard' : 'dialog.memo.finishDiscard');
      discard.addEventListener('click', () => {
        this.choice = 'discard';
        this.hide();
      });
      footer.insertBefore(discard, keep);
      keep.focus();
    });
  }

  protected override onTabKey(event: KeyboardEvent): boolean {
    const buttons = [...this.dialog.querySelectorAll('button')];
    const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const next = (current + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length;
    buttons[next].focus();
    return true;
  }
}
