import { PDF_PRINT_GUIDANCE, printProgressText } from '@/command/print-pages';
import { ModalDialog } from './dialog';

type PdfPrintDialogState = 'confirm' | 'preparing' | 'complete' | 'error';

/**
 * 브라우저 인쇄 기반 PDF 저장의 남은 단계를 설명하고, 확인 뒤 같은 모달에서
 * 인쇄 문서 준비 진행률을 표시한다.
 */
export class PdfPrintDialog extends ModalDialog {
  private state: PdfPrintDialogState = 'confirm';
  private resolveDecision: (confirmed: boolean) => void = () => {};
  private progressMessage!: HTMLDivElement;
  private progressTrack!: HTMLDivElement;
  private progressBar!: HTMLDivElement;
  private errorMessage!: HTMLDivElement;
  private confirmButton: HTMLButtonElement | null = null;
  private cancelButton: HTMLButtonElement | null = null;
  private closeButton: HTMLButtonElement | null = null;

  constructor(private readonly pageCount: number) {
    super('PDF로 저장', 460);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-pdf-print';
    body.dataset.testid = 'pdf-print-dialog';

    const summary = document.createElement('p');
    summary.className = 'dialog-pdf-summary';
    summary.textContent =
      'rhwp는 브라우저의 인쇄 기능을 사용해 검색 가능한 PDF를 만듭니다.';

    const guidance = document.createElement('div');
    guidance.className = 'dialog-pdf-guidance';

    const guidanceTitle = document.createElement('strong');
    guidanceTitle.textContent = '저장 방법';

    const guidanceText = document.createElement('span');
    guidanceText.textContent =
      `${PDF_PRINT_GUIDANCE} 브라우저에 따라 이 항목은 ‘프린터’로 표시될 수 있습니다.`;

    guidance.append(guidanceTitle, guidanceText);

    const stateNote = document.createElement('p');
    stateNote.className = 'dialog-pdf-note';
    stateNote.textContent =
      '현재 문서의 파일명, 저장 위치와 편집 상태는 변경되지 않습니다.';

    const progress = document.createElement('div');
    progress.className = 'dialog-pdf-progress';
    progress.hidden = true;
    progress.dataset.testid = 'pdf-print-progress';

    this.progressMessage = document.createElement('div');
    this.progressMessage.className = 'dialog-pdf-progress-message';
    this.progressMessage.setAttribute('aria-live', 'polite');

    this.progressTrack = document.createElement('div');
    this.progressTrack.className = 'dialog-pdf-progress-track';
    this.progressTrack.setAttribute('role', 'progressbar');
    this.progressTrack.setAttribute('aria-valuemin', '0');
    this.progressTrack.setAttribute('aria-valuemax', String(this.pageCount));
    this.progressTrack.setAttribute('aria-valuenow', '0');

    this.progressBar = document.createElement('div');
    this.progressBar.className = 'dialog-pdf-progress-bar';
    this.progressTrack.appendChild(this.progressBar);
    progress.append(this.progressMessage, this.progressTrack);

    this.errorMessage = document.createElement('div');
    this.errorMessage.className = 'dialog-pdf-error';
    this.errorMessage.hidden = true;
    this.errorMessage.setAttribute('role', 'alert');

    body.append(summary, guidance, stateNote, progress, this.errorMessage);
    return body;
  }

  protected onConfirm(): boolean {
    if (this.state !== 'confirm') return false;
    this.state = 'preparing';
    this.dialog.classList.add('dialog-pdf-preparing');

    const progress = this.dialog.querySelector('.dialog-pdf-progress') as HTMLElement | null;
    if (progress) progress.hidden = false;
    this.updateProgress(0);

    if (this.confirmButton) {
      this.confirmButton.textContent = '준비 중…';
      this.confirmButton.disabled = true;
    }
    if (this.cancelButton) this.cancelButton.disabled = true;
    if (this.closeButton) this.closeButton.hidden = true;

    this.resolveDecision(true);
    return false;
  }

  override hide(): void {
    if (this.state === 'preparing') return;
    this.resolveDecision(false);
    super.hide();
  }

  showAsync(): Promise<boolean> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolveDecision = (confirmed: boolean) => {
        if (resolved) return;
        resolved = true;
        resolve(confirmed);
      };

      super.show();
      this.dialog.setAttribute('role', 'dialog');
      this.dialog.setAttribute('aria-modal', 'true');
      this.dialog.setAttribute('aria-label', 'PDF로 저장');

      this.confirmButton = this.dialog.querySelector('.dialog-btn-primary');
      this.cancelButton = this.dialog.querySelector(
        '.dialog-footer .dialog-btn:not(.dialog-btn-primary)',
      );
      this.closeButton = this.dialog.querySelector('.dialog-close');

      if (this.confirmButton) this.confirmButton.textContent = '인쇄 창 열기';
    });
  }

  updateProgress(currentPage: number): void {
    const safeCurrent = Math.max(0, Math.min(currentPage, this.pageCount));
    const progressText = printProgressText('pdf', safeCurrent, this.pageCount);
    this.progressMessage.textContent = progressText;
    this.progressTrack.setAttribute('aria-valuenow', String(safeCurrent));
    const percent = this.pageCount > 0 ? (safeCurrent / this.pageCount) * 100 : 0;
    this.progressBar.style.width = `${percent}%`;
  }

  closeBeforePrint(): void {
    this.state = 'complete';
    super.hide();
  }

  showError(message: string): void {
    this.state = 'error';
    this.dialog.classList.remove('dialog-pdf-preparing');
    const progress = this.dialog.querySelector('.dialog-pdf-progress') as HTMLElement | null;
    if (progress) progress.hidden = true;

    this.errorMessage.hidden = false;
    this.errorMessage.textContent = `PDF 준비에 실패했습니다.\n${message}`;

    if (this.confirmButton) this.confirmButton.hidden = true;
    if (this.cancelButton) {
      this.cancelButton.textContent = '닫기';
      this.cancelButton.disabled = false;
      this.cancelButton.focus();
    }
    if (this.closeButton) this.closeButton.hidden = false;
  }
}
