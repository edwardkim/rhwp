/**
 * AI 설정 대화상자 (도구 > AI 설정).
 *
 * 키 입력란은 비워 두면 기존 키를 유지한다. 저장된 키 원문은 화면에 다시 채우지 않고
 * 가린 표시(`maskKey`)만 보인다.
 */
import { ModalDialog } from '@/ui/dialog';
import { t } from '@/i18n/index.ts';
import type { AiContext } from '../ai-context.ts';
import { AI_EFFORTS, AI_PROVIDERS, modelsOf, type AiEffort, type AiProviderId } from '../provider/models.ts';
import type { AiFailureReason } from '../provider/types.ts';
import { KeyStoreError, maskKey } from '../settings/key-store.ts';

export function failureMessage(reason: AiFailureReason, retryAfterSeconds?: number): string {
  if (reason === 'rate' && retryAfterSeconds !== undefined) {
    return t('ai.error.rateRetry', { s: Math.ceil(retryAfterSeconds) });
  }
  return t(`ai.error.${reason}`);
}

function keyStoreMessage(error: unknown): string {
  if (error instanceof KeyStoreError) {
    switch (error.code) {
      case 'INVALID_KEY': return t('ai.settings.invalidKey');
      case 'EMPTY_PASSPHRASE': return t('ai.settings.emptyPassphrase');
      case 'WRONG_PASSPHRASE': return t('ai.unlock.wrong');
    }
  }
  return t('ai.error.unknown');
}

function row(label: string, control: HTMLElement): HTMLElement {
  const wrap = document.createElement('div');
  wrap.className = 'dialog-row ai-settings-row';
  const lab = document.createElement('label');
  lab.className = 'dialog-label';
  lab.textContent = label;
  if (control.id) lab.htmlFor = control.id;
  wrap.append(lab, control);
  return wrap;
}

function select(id: string, options: Array<{ value: string; label: string }>, value: string): HTMLSelectElement {
  const el = document.createElement('select');
  el.id = id;
  el.className = 'dialog-select';
  for (const option of options) {
    const opt = document.createElement('option');
    opt.value = option.value;
    opt.textContent = option.label;
    el.append(opt);
  }
  el.value = value;
  return el;
}

export class AiSettingsDialog extends ModalDialog {
  private readonly ctx: AiContext;
  private providerSelect!: HTMLSelectElement;
  private modelSelect!: HTMLSelectElement;
  private effortSelect!: HTMLSelectElement;
  private keyInput!: HTMLInputElement;
  private keyState!: HTMLElement;
  private rememberCheck!: HTMLInputElement;
  private passphraseRow!: HTMLElement;
  private passphraseInput!: HTMLInputElement;
  private statusLine!: HTMLElement;
  private testButton!: HTMLButtonElement;
  private testAbort: AbortController | null = null;
  private saving = false;

  constructor(ctx: AiContext) {
    super(t('ai.settings.title'), 480);
    this.ctx = ctx;
  }

  protected createBody(): HTMLElement {
    const settings = this.ctx.settings;
    const body = document.createElement('div');
    body.className = 'ai-settings-body';

    this.providerSelect = select('ai-settings-provider',
      AI_PROVIDERS.map((id) => ({ value: id, label: t(`ai.provider.${id}`) })), settings.provider);
    this.providerSelect.addEventListener('change', () => this.fillModels());
    this.modelSelect = select('ai-settings-model', [], settings.model);
    this.fillModels(settings.model);
    this.effortSelect = select('ai-settings-effort',
      AI_EFFORTS.map((id) => ({ value: id, label: t(`ai.effort.${id}`) })), settings.effort);

    this.keyInput = document.createElement('input');
    this.keyInput.id = 'ai-settings-key';
    this.keyInput.type = 'password';
    this.keyInput.className = 'dialog-input';
    this.keyInput.autocomplete = 'off';
    this.keyInput.spellcheck = false;
    this.keyInput.placeholder = t('ai.settings.keyPlaceholder');
    this.keyState = document.createElement('div');
    this.keyState.className = 'ai-settings-key-state';

    this.rememberCheck = document.createElement('input');
    this.rememberCheck.type = 'checkbox';
    this.rememberCheck.id = 'ai-settings-remember';
    this.rememberCheck.checked = settings.rememberKey && this.ctx.keyStore.canPersist;
    this.rememberCheck.disabled = !this.ctx.keyStore.canPersist;
    const rememberLabel = document.createElement('label');
    rememberLabel.className = 'ai-settings-check';
    rememberLabel.append(this.rememberCheck, document.createTextNode(
      this.ctx.keyStore.canPersist ? t('ai.settings.remember') : t('ai.settings.rememberUnavailable')));
    this.rememberCheck.addEventListener('change', () => this.syncPassphrase());

    this.passphraseInput = document.createElement('input');
    this.passphraseInput.id = 'ai-settings-passphrase';
    this.passphraseInput.type = 'password';
    this.passphraseInput.className = 'dialog-input';
    this.passphraseInput.autocomplete = 'new-password';
    this.passphraseInput.placeholder = t('ai.settings.passphrasePlaceholder');
    this.passphraseRow = row(t('ai.settings.passphrase'), this.passphraseInput);

    const warning = document.createElement('p');
    warning.className = 'ai-settings-warning';
    warning.textContent = t('ai.settings.warning');

    const actions = document.createElement('div');
    actions.className = 'ai-settings-actions';
    this.testButton = document.createElement('button');
    this.testButton.type = 'button';
    this.testButton.className = 'dialog-btn';
    this.testButton.textContent = t('ai.settings.test');
    this.testButton.addEventListener('click', () => { void this.runTest(); });
    const clearButton = document.createElement('button');
    clearButton.type = 'button';
    clearButton.className = 'dialog-btn';
    clearButton.textContent = t('ai.settings.clearKey');
    clearButton.addEventListener('click', () => { void this.clearKey(); });
    actions.append(this.testButton, clearButton);

    this.statusLine = document.createElement('div');
    this.statusLine.className = 'ai-settings-status';
    this.statusLine.setAttribute('role', 'status');
    this.statusLine.setAttribute('aria-live', 'polite');

    const keyBlock = document.createElement('div');
    keyBlock.className = 'ai-settings-key';
    keyBlock.append(this.keyInput, this.keyState);

    body.append(
      row(t('ai.settings.provider'), this.providerSelect),
      row(t('ai.settings.model'), this.modelSelect),
      row(t('ai.settings.effort'), this.effortSelect),
      row(t('ai.settings.key'), keyBlock),
      rememberLabel,
      this.passphraseRow,
      warning,
      actions,
      this.statusLine,
    );
    this.syncPassphrase();
    void this.refreshKeyState();
    return body;
  }

  override show(): void {
    super.show();
    this.keyInput.focus();
  }

  override hide(): void {
    this.testAbort?.abort();
    this.testAbort = null;
    super.hide();
  }

  protected onConfirm(): boolean {
    if (!this.saving) void this.save();
    return false;
  }

  private get provider(): AiProviderId {
    return this.providerSelect.value as AiProviderId;
  }

  private fillModels(preferred?: string): void {
    const models = modelsOf(this.provider);
    const keep = preferred ?? this.modelSelect.value;
    this.modelSelect.replaceChildren(...models.map((model) => {
      const opt = document.createElement('option');
      opt.value = model.id;
      opt.textContent = model.label;
      return opt;
    }));
    this.modelSelect.value = models.some((m) => m.id === keep) ? keep : models[0].id;
  }

  private syncPassphrase(): void {
    this.passphraseRow.hidden = !this.rememberCheck.checked;
  }

  private async refreshKeyState(): Promise<void> {
    const status = await this.ctx.keyStore.status(this.provider);
    const key = this.ctx.keyStore.getKey(this.provider);
    this.keyState.textContent = status === 'ready' && key
      ? t('ai.settings.keyReady', { key: maskKey(key) })
      : status === 'locked' ? t('ai.settings.keyLocked') : t('ai.settings.keyNone');
  }

  private setStatus(text: string, kind: 'info' | 'ok' | 'error'): void {
    this.statusLine.textContent = text;
    this.statusLine.dataset.kind = kind;
  }

  private async runTest(): Promise<void> {
    const typed = this.keyInput.value.trim();
    const key = typed || this.ctx.keyStore.getKey(this.provider);
    if (!key) {
      this.setStatus(t('ai.settings.testNeedsKey'), 'error');
      return;
    }
    this.testAbort?.abort();
    const abort = new AbortController();
    this.testAbort = abort;
    this.testButton.disabled = true;
    this.setStatus(t('ai.settings.testing'), 'info');
    try {
      // SDK 는 처음 필요할 때 불러온다 — 패널만 열어 보는 사람에게 내려보내지 않는다.
      const { testClaudeConnection } = await import('../provider/claude.ts');
      const result = await testClaudeConnection(key, this.modelSelect.value, abort.signal);
      if (abort.signal.aborted) return;
      if (result.ok) this.setStatus(t('ai.settings.testOk', { model: result.modelName }), 'ok');
      else this.setStatus(failureMessage(result.reason, result.retryAfterSeconds), 'error');
    } finally {
      if (this.testAbort === abort) this.testAbort = null;
      this.testButton.disabled = false;
    }
  }

  private async clearKey(): Promise<void> {
    await this.ctx.keyStore.clear(this.provider);
    this.keyInput.value = '';
    this.passphraseInput.value = '';
    await this.refreshKeyState();
    this.ctx.emit();
    this.setStatus(t('ai.settings.keyCleared'), 'info');
  }

  private async save(): Promise<void> {
    this.saving = true;
    try {
      const provider = this.provider;
      const remember = this.rememberCheck.checked;
      const passphrase = remember && this.passphraseInput.value.length > 0
        ? this.passphraseInput.value
        : undefined;
      const typed = this.keyInput.value;
      const store = this.ctx.keyStore;
      let persisted = true;

      if (typed.trim().length > 0) {
        persisted = (await store.setKey(provider, typed, { remember, passphrase })).persisted || !remember;
      } else if (store.getKey(provider)) {
        // 키는 그대로, 보관 방식만 바뀌었을 수 있다.
        if (remember !== this.ctx.settings.rememberKey || passphrase !== undefined) {
          persisted = (await store.setRemember(provider, { remember, passphrase })).persisted || !remember;
        }
      } else if (!remember) {
        // 잠긴 키를 더는 기억하지 않기로 했다 — 저장소에서 지운다.
        await store.clear(provider);
      }

      this.ctx.updateSettings({
        provider,
        model: this.modelSelect.value,
        effort: this.effortSelect.value as AiEffort,
        rememberKey: remember && persisted,
      });
      this.ctx.emit();
      if (!persisted) {
        this.setStatus(t('ai.settings.notPersisted'), 'error');
        return;
      }
      this.hide();
    } catch (error) {
      this.setStatus(keyStoreMessage(error), 'error');
    } finally {
      this.saving = false;
    }
  }
}
