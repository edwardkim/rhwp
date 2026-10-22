/**
 * AI 패널 본문. Stage 1 은 자리만 잡는다 — 설정 안내 문구 하나.
 * 대화 UI 는 Stage 4 에서 이 클래스에 채운다.
 */
import { t } from '@/i18n/index.ts';
import './ai-panel.css';

export class AiPanel {
  private root: HTMLElement | null = null;

  mount(container: HTMLElement): void {
    const root = document.createElement('div');
    root.className = 'ai-panel';
    const notice = document.createElement('p');
    notice.className = 'ai-panel-notice';
    notice.textContent = t('ai.panel.setupNotice');
    root.append(notice);
    container.append(root);
    this.root = root;
  }

  setVisible(open: boolean): void {
    if (!this.root) return;
    this.root.dataset.open = open ? 'true' : 'false';
  }

  dispose(): void {
    this.root?.remove();
    this.root = null;
  }
}
