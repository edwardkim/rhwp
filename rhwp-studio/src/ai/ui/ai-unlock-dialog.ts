/**
 * 암호로 잠근 API 키를 여는 대화상자. 성공하면 키가 이번 실행 동안 메모리에 올라온다.
 */
import { ModalDialog } from '@/ui/dialog';
import { t } from '@/i18n/index.ts';
import type { AiContext } from '../ai-context.ts';
import { KeyStoreError } from '../settings/key-store.ts';

export class AiUnlockDialog extends ModalDialog {
  private readonly ctx: AiContext;
  private input!: HTMLInputElement;
  private statusLine!: HTMLElement;
  private busy = false;

  constructor(ctx: AiContext) {
    super(t('ai.unlock.title'), 360);
    this.ctx = ctx;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'ai-settings-body';
    const message = document.createElement('p');
    message.className = 'ai-settings-note';
    message.textContent = t('ai.unlock.message');
    this.input = document.createElement('input');
    this.input.type = 'password';
    this.input.className = 'dialog-input';
    this.input.autocomplete = 'current-password';
    this.input.setAttribute('aria-label', t('ai.settings.passphrase'));
    this.statusLine = document.createElement('div');
    this.statusLine.className = 'ai-settings-status';
    this.statusLine.setAttribute('role', 'status');
    this.statusLine.setAttribute('aria-live', 'polite');
    // 대화상자 틀은 입력란의 keydown 전파를 막는다. form 의 암묵 제출(Enter)은 기본 동작이라
    // 전파 차단과 무관하게 일어나므로 그것으로 Enter 확인을 받는다.
    const form = document.createElement('form');
    form.addEventListener('submit', (event) => {
      event.preventDefault();
      this.onConfirm();
    });
    form.append(this.input);
    body.append(message, form, this.statusLine);
    return body;
  }

  override show(): void {
    super.show();
    this.input.focus();
  }

  protected onConfirm(): boolean {
    if (!this.busy) void this.unlock();
    return false;
  }

  private async unlock(): Promise<void> {
    this.busy = true;
    try {
      await this.ctx.keyStore.unlock(this.ctx.settings.provider, this.input.value);
      this.ctx.emit();
      this.hide();
    } catch (error) {
      this.statusLine.dataset.kind = 'error';
      this.statusLine.textContent = error instanceof KeyStoreError && error.code === 'EMPTY_PASSPHRASE'
        ? t('ai.settings.emptyPassphrase')
        : t('ai.unlock.wrong');
      this.input.select();
    } finally {
      this.busy = false;
    }
  }
}
