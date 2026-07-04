import { enableDialogDrag } from './dialog-drag';

/**
 * 모달 다이얼로그 베이스 클래스 (WebGian dialog_wrap 패턴)
 *
 * DOM은 show() 호출 시 생성된다 (ES2022 class field 초기화 순서 이슈 방지).
 */
export abstract class ModalDialog {
  afterClose?: () => void;

  protected overlay!: HTMLDivElement;
  protected dialog!: HTMLDivElement;
  private title: string;
  private width: number;
  private closeOnOverlayClick: boolean;
  private built = false;
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(title: string, width: number, closeOnOverlayClick = false) {
    this.title = title;
    this.width = width;
    this.closeOnOverlayClick = closeOnOverlayClick;
  }

  private build(): void {
    if (this.built) return;
    this.built = true;

    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap';
    this.dialog.style.width = `${this.width}px`;

    // 타이틀 바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = this.title;

    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7'; // ×
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);

    this.dialog.appendChild(titleBar);

    // 본문
    const body = this.createBody();
    body.classList.add('dialog-body');
    this.dialog.appendChild(body);

    // 하단 버튼
    const footer = document.createElement('div');
    footer.className = 'dialog-footer';

    const confirmBtn = document.createElement('button');
    confirmBtn.className = 'dialog-btn dialog-btn-primary';
    confirmBtn.textContent = '확인';
    confirmBtn.addEventListener('click', () => {
      const shouldClose = this.onConfirm();
      if (shouldClose !== false) this.hide();
    });

    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '취소';
    cancelBtn.addEventListener('click', () => this.hide());

    footer.appendChild(confirmBtn);
    footer.appendChild(cancelBtn);
    this.dialog.appendChild(footer);

    this.overlay.appendChild(this.dialog);

    // 모달은 기본적으로 명시적 버튼/Escape로만 닫는다.
    this.overlay.addEventListener('click', (e) => {
      if (e.target === this.overlay && this.closeOnOverlayClick) this.hide();
    });

    enableDialogDrag(this.dialog, titleBar);
  }

  show(): void {
    this.build();
    document.body.appendChild(this.overlay);

    // document capture 단계에서 키 이벤트를 가로채 편집 영역 도달 차단
    // input/textarea 내부의 일반 타이핑은 허용한다.
    this.captureHandler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isEditable = target instanceof HTMLInputElement
        || target instanceof HTMLTextAreaElement;

      if (e.key === 'Escape') {
        e.stopPropagation();
        e.preventDefault();
        this.hide();
        return;
      }
      if (e.key === 'Enter' && !isEditable) {
        e.stopPropagation();
        e.preventDefault();
        const btn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
        btn?.click();
        return;
      }
      // 편집 가능한 요소 내부 → 키 입력 허용, 외부 전파만 차단
      e.stopPropagation();
      if (!isEditable) {
        e.preventDefault();
      }
    };
    document.addEventListener('keydown', this.captureHandler, true);

    // 확인 버튼에 포커스 → 시각적 피드백 + 접근성
    const btn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
    btn?.focus();
  }

  hide(): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this.overlay?.remove();
    this.afterClose?.();
  }

  /** 서브클래스에서 본문 DOM을 생성 */
  protected abstract createBody(): HTMLElement;

  /** 서브클래스에서 확인 버튼 동작 구현. false 반환 시 대화상자 유지 */
  protected abstract onConfirm(): void | boolean;
}
