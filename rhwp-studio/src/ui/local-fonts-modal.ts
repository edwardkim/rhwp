/**
 * 문서 글꼴 상태 기반 로컬 글꼴 감지 안내 모달.
 *
 * Local Font Access API는 사용자 PC의 설치 글꼴 목록을 읽으므로,
 * 문서 로드 중 자동 조회하지 않고 사용자 선택 후 호출한다.
 */

import type { DocumentFontStatusItem, DocumentFontStatusReport } from '@/core/document-font-status';
import { enableDialogDrag } from './dialog-drag';

export type LocalFontsChoice = 'detect' | 'web-substitute' | 'cancel';

export interface LocalFontsModalOptions {
  disableExternalWebFonts?: boolean;
}

const STATUS_LABEL: Record<DocumentFontStatusItem['status'], string> = {
  available: '사용 가능',
  'needs-local-check': '로컬 확인 필요',
  'web-substitute': '대체 글꼴 사용',
  missing: '누락',
};

export class LocalFontsModal {
  private overlay: HTMLDivElement | null = null;
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;
  private resolver: ((choice: LocalFontsChoice) => void) | null = null;

  constructor(
    private readonly report: DocumentFontStatusReport,
    private readonly options: LocalFontsModalOptions = {},
  ) {}

  async showAsync(): Promise<LocalFontsChoice> {
    return new Promise((resolve) => {
      this.resolver = resolve;
      this.build();
      document.body.appendChild(this.overlay!);
      this.bindKeyboard();

      const primaryBtn = this.overlay!.querySelector(
        '.dialog-btn-primary',
      ) as HTMLButtonElement | null;
      primaryBtn?.focus();
    });
  }

  private build(): void {
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    const dialog = document.createElement('div');
    dialog.className = 'dialog-wrap';
    dialog.style.width = '520px';

    const title = document.createElement('div');
    title.className = 'dialog-title';
    title.textContent = '로컬 글꼴 감지';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.resolve('cancel'));
    title.appendChild(closeBtn);
    dialog.appendChild(title);
    enableDialogDrag(dialog, title);

    const body = document.createElement('div');
    body.className = 'dialog-body';
    body.style.padding = '16px 20px';
    body.style.lineHeight = '1.6';

    const desc = document.createElement('p');
    desc.style.margin = '0 0 12px 0';
    desc.textContent = this.report.detectionMethod === 'font-presence-probe'
      ? '현재 문서에 rhwp 기본 지원 글꼴이 아닌 글꼴이 있습니다. 원본에 가깝게 표시하기 위해, 이 문서에 필요한 글꼴이 이 기기에 설치되어 있는지 확인합니다.'
      : '현재 문서에 rhwp 기본 지원 글꼴이 아닌 글꼴이 있습니다. 원본에 가깝게 표시하려면 로컬 글꼴 감지를 허용해 주세요.';
    body.appendChild(desc);

    const privacy = document.createElement('p');
    privacy.style.margin = '0 0 12px 0';
    privacy.style.fontSize = '13px';
    privacy.style.color = 'var(--color-text-secondary)';
    privacy.textContent = this.report.detectionMethod === 'font-presence-probe'
      ? '이 브라우저에서는 설치된 모든 글꼴 목록을 가져오지 않고, 현재 문서에 필요한 글꼴만 확인합니다. 확인 결과는 이 브라우저/확장 로컬 저장소에만 보관되며 서버로 전송하지 않습니다. 감지를 건너뛰면 대체 글꼴로 계속 표시합니다.'
      : '감지 결과는 이 브라우저/확장 로컬 저장소에만 보관되며 서버로 전송하지 않습니다. 감지를 건너뛰면 대체 글꼴로 계속 표시합니다.';
    body.appendChild(privacy);

    if (this.options.disableExternalWebFonts) {
      const offlineNotice = document.createElement('div');
      offlineNotice.style.margin = '0 0 12px 0';
      offlineNotice.style.padding = '8px 10px';
      offlineNotice.style.border = '1px solid var(--color-border)';
      offlineNotice.style.borderRadius = '4px';
      offlineNotice.style.fontSize = '13px';
      offlineNotice.style.color = 'var(--color-text-secondary)';
      offlineNotice.textContent = '외부 웹폰트 사용 안 함: 켜짐. 대체 글꼴은 외부 CDN 폰트를 요청하지 않고 번들/시스템 글꼴 기준으로 표시됩니다.';
      body.appendChild(offlineNotice);
    }

    const summary = document.createElement('ul');
    summary.style.margin = '0 0 12px 16px';
    summary.style.padding = '0';
    summary.style.fontSize = '13px';
    summary.style.color = 'var(--color-text-secondary)';
    const rows: Array<[string, number]> = [
      ['사용 가능', this.report.summary.available],
      ['로컬 확인 필요', this.report.summary.needsLocalCheck],
      ['대체 글꼴 사용', this.report.summary.webSubstitute],
      ['누락', this.report.summary.missing],
    ];
    for (const [label, count] of rows) {
      if (count === 0) continue;
      const li = document.createElement('li');
      li.textContent = `${label}: ${count}개`;
      summary.appendChild(li);
    }
    body.appendChild(summary);

    const details = document.createElement('details');
    details.style.marginTop = '8px';
    const summaryEl = document.createElement('summary');
    summaryEl.textContent = '문서 글꼴 상태 보기';
    summaryEl.style.cursor = 'pointer';
    summaryEl.style.fontSize = '13px';
    summaryEl.style.color = 'var(--ui-link)';
    details.appendChild(summaryEl);

    const detailList = document.createElement('div');
    detailList.style.maxHeight = '180px';
    detailList.style.overflow = 'auto';
    detailList.style.marginTop = '8px';
    detailList.style.padding = '8px';
    detailList.style.background = 'var(--color-surface-raised)';
    detailList.style.borderRadius = '4px';
    detailList.style.fontSize = '12px';
    detailList.style.color = 'var(--color-text)';

    const maxShow = 50;
    for (const item of this.report.fonts.slice(0, maxShow)) {
      const line = document.createElement('div');
      const substitute = item.substituteFont ? ` → ${item.substituteFont}` : '';
      line.textContent = `${item.fontName}: ${STATUS_LABEL[item.status]}${substitute}`;
      detailList.appendChild(line);
    }
    if (this.report.fonts.length > maxShow) {
      const more = document.createElement('div');
      more.style.color = 'var(--color-text-hint)';
      more.style.marginTop = '4px';
      more.textContent = `... 외 ${this.report.fonts.length - maxShow}개`;
      detailList.appendChild(more);
    }
    details.appendChild(detailList);
    body.appendChild(details);

    dialog.appendChild(body);

    const footer = document.createElement('div');
    footer.className = 'dialog-footer';

    const detectBtn = document.createElement('button');
    detectBtn.className = 'dialog-btn dialog-btn-primary';
    detectBtn.textContent = '로컬 글꼴 감지 (권장)';
    detectBtn.addEventListener('click', () => this.resolve('detect'));

    const webBtn = document.createElement('button');
    webBtn.className = 'dialog-btn';
    webBtn.textContent = '대체 글꼴로 보기';
    webBtn.addEventListener('click', () => this.resolve('web-substitute'));

    footer.appendChild(detectBtn);
    footer.appendChild(webBtn);
    dialog.appendChild(footer);

    this.overlay.appendChild(dialog);
  }

  private bindKeyboard(): void {
    this.captureHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        e.preventDefault();
        this.resolve('cancel');
        return;
      }
      if (e.key === 'Enter') {
        e.stopPropagation();
        e.preventDefault();
        this.resolve('detect');
        return;
      }
      e.stopPropagation();
    };
    document.addEventListener('keydown', this.captureHandler, true);
  }

  private resolve(choice: LocalFontsChoice): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this.overlay?.remove();
    this.overlay = null;
    if (this.resolver) {
      this.resolver(choice);
      this.resolver = null;
    }
  }
}

export async function showLocalFontsModalIfNeeded(
  report: DocumentFontStatusReport,
  options: LocalFontsModalOptions = {},
): Promise<LocalFontsChoice> {
  if (!report.shouldPromptLocalAccess) return 'web-substitute';
  return new LocalFontsModal(report, options).showAsync();
}
