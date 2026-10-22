/**
 * AI 패널 본문.
 *
 * Stage 2 는 준비 상태만 보인다 — 키 없음 / 잠김 / 전송 동의 필요 / 준비됨.
 * 대화 UI 는 Stage 4 에서 「준비됨」 자리에 채운다.
 */
import { t } from '@/i18n/index.ts';
import { showConfirm } from '@/ui/confirm-dialog';
import type { AiContext } from '../ai-context.ts';
import { modelsOf } from '../provider/models.ts';
import { AiSettingsDialog } from './ai-settings-dialog.ts';
import { AiUnlockDialog } from './ai-unlock-dialog.ts';
import './ai-panel.css';

export type AiPanelState = 'none' | 'locked' | 'consent' | 'ready';

export class AiPanel {
  private readonly ctx: AiContext;
  private root: HTMLElement | null = null;
  private notice!: HTMLParagraphElement;
  private action!: HTMLButtonElement;
  private modelLine!: HTMLElement;
  private offChange: (() => void) | null = null;
  private visible = false;
  /** 이번 실행에서 동의 대화상자를 이미 띄웠는가 — 거절한 사람에게 열 때마다 묻지 않는다. */
  private consentAsked = false;
  private state: AiPanelState = 'none';
  private renderSeq = 0;

  constructor(ctx: AiContext) {
    this.ctx = ctx;
  }

  get currentState(): AiPanelState {
    return this.state;
  }

  mount(container: HTMLElement): void {
    const root = document.createElement('div');
    root.className = 'ai-panel';

    this.modelLine = document.createElement('div');
    this.modelLine.className = 'ai-panel-model';
    this.notice = document.createElement('p');
    this.notice.className = 'ai-panel-notice';
    this.action = document.createElement('button');
    this.action.type = 'button';
    this.action.className = 'dialog-btn ai-panel-action';
    this.action.addEventListener('click', () => { void this.runAction(); });

    root.append(this.modelLine, this.notice, this.action);
    container.append(root);
    this.root = root;
    this.offChange = this.ctx.onChange(() => { void this.render(); });
    void this.render();
  }

  setVisible(open: boolean): void {
    this.visible = open;
    if (!this.root) return;
    this.root.dataset.open = open ? 'true' : 'false';
    if (open) void this.render().then(() => this.maybeAskConsent());
  }

  openSettings(): void {
    new AiSettingsDialog(this.ctx).show();
  }

  dispose(): void {
    this.offChange?.();
    this.offChange = null;
    this.root?.remove();
    this.root = null;
  }

  private async render(): Promise<void> {
    if (!this.root) return;
    const seq = ++this.renderSeq;
    const status = await this.ctx.keyStatus();
    // 늦게 끝난 이전 렌더가 새 상태를 덮지 않게 한다.
    if (seq !== this.renderSeq || !this.root) return;

    this.state = status === 'none' ? 'none'
      : status === 'locked' ? 'locked'
        : this.ctx.hasConsent() ? 'ready' : 'consent';
    this.root.dataset.state = this.state;

    const settings = this.ctx.settings;
    const model = modelsOf(settings.provider).find((m) => m.id === settings.model)?.label ?? settings.model;
    this.modelLine.textContent = t('ai.panel.modelLine', { provider: t(`ai.provider.${settings.provider}`), model });

    const [notice, action] = ({
      none: ['ai.panel.setupNotice', 'ai.panel.openSettings'],
      locked: ['ai.panel.lockedNotice', 'ai.panel.unlock'],
      consent: ['ai.panel.consentNotice', 'ai.panel.giveConsent'],
      ready: ['ai.panel.readyNotice', 'ai.panel.openSettings'],
    } as const)[this.state];
    this.notice.textContent = t(notice);
    this.action.textContent = t(action);
  }

  private async runAction(): Promise<void> {
    switch (this.state) {
      case 'none':
      case 'ready':
        this.openSettings();
        return;
      case 'locked':
        new AiUnlockDialog(this.ctx).show();
        return;
      case 'consent':
        await this.askConsent();
    }
  }

  /** 키가 준비됐는데 동의가 없으면, 패널을 처음 열 때 한 번만 묻는다. */
  private async maybeAskConsent(): Promise<void> {
    if (!this.visible || this.consentAsked || this.state !== 'consent') return;
    await this.askConsent();
  }

  private async askConsent(): Promise<void> {
    this.consentAsked = true;
    const provider = this.ctx.settings.provider;
    const agreed = await showConfirm(
      t('ai.consent.title'),
      t('ai.consent.message', { provider: t(`ai.provider.${provider}`) }),
    );
    if (agreed) this.ctx.updateSettings({ consent: { at: new Date().toISOString(), provider } });
  }
}
