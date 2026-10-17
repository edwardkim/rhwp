/**
 * 셀 블록 지우기 확인 대화상자 (한컴 "지우기" 정합).
 *
 * 셀 블록이 표의 전체 줄 또는 전체 칸을 덮을 때 edit:delete(⌘E/⌘⌫/⌘Delete)는
 * "선택된 셀들을 지웁니다. 내용만 지우고 셀 모양은 남겨 둘까요?" 를 묻는다:
 *   [지우기]  → 셀 자체를 구조적으로 삭제 (표 모양이 바뀐다)
 *   [남김]    → 셀을 남기고 내용만 지운다
 *   [취소]·Esc·× → 아무 것도 하지 않는다 (블록 유지)
 */
import { ModalDialog } from './dialog';
import { t } from '../i18n/index.ts';

export type CellBlockDeleteAnswer = 'delete' | 'keep' | 'cancel';

class CellBlockDeleteDialog extends ModalDialog {
  private result: CellBlockDeleteAnswer = 'cancel';
  private resolve!: (value: CellBlockDeleteAnswer) => void;

  constructor() {
    super(t('dialog.cellBlockDelete.title'), 390);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.id = 'cell-block-delete-description';
    body.style.padding = '16px 20px';
    body.style.lineHeight = '1.6';
    body.style.whiteSpace = 'pre-line';
    body.textContent = t('dialog.cellBlockDelete.body.text');
    return body;
  }

  /** [지우기] — 기본 확인 버튼이 구조 삭제를 선택한다. */
  protected onConfirm(): void {
    this.result = 'delete';
  }

  private settle(result: CellBlockDeleteAnswer): void {
    if (this.resolve) this.resolve(result);
  }

  override show(): void {
    super.show();
    this.dialog.setAttribute('role', 'alertdialog');
    this.dialog.setAttribute('aria-label', t('dialog.cellBlockDelete.title'));
    this.dialog.setAttribute('aria-describedby', 'cell-block-delete-description');

    const footer = this.dialog.querySelector('.dialog-footer');
    const confirmBtn = footer?.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
    const cancelBtn = footer?.querySelector('.dialog-btn:not(.dialog-btn-primary)') as HTMLButtonElement | null;
    if (!footer || !confirmBtn || !cancelBtn) return;

    confirmBtn.textContent = t('dialog.cellBlockDelete.deleteBtn.text');

    const keepBtn = document.createElement('button');
    keepBtn.className = 'dialog-btn';
    keepBtn.textContent = t('dialog.cellBlockDelete.keepBtn.text');
    keepBtn.addEventListener('click', () => {
      this.result = 'keep';
      this.hide();
    });
    footer.insertBefore(keepBtn, cancelBtn);
  }

  override hide(): void {
    const result = this.result;
    super.hide();
    this.settle(result);
  }

  protected override onTabKey(e: KeyboardEvent): boolean {
    const buttons = [
      ...this.dialog.querySelectorAll<HTMLButtonElement>('.dialog-footer button'),
      this.dialog.querySelector<HTMLButtonElement>('.dialog-close'),
    ].filter((button): button is HTMLButtonElement => button !== null);
    const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const next = (current + (e.shiftKey ? -1 : 1) + buttons.length) % buttons.length;
    buttons[next]?.focus();
    return true;
  }

  showAsync(): Promise<CellBlockDeleteAnswer> {
    return new Promise((resolve) => {
      let resolved = false;
      this.resolve = (v: CellBlockDeleteAnswer) => {
        if (!resolved) {
          resolved = true;
          resolve(v);
        }
      };
      this.show();
    });
  }
}

/** 셀 블록 지우기 방식을 묻는 모달. Esc/×/취소 는 'cancel' 을 돌려준다. */
export function askCellBlockDelete(): Promise<CellBlockDeleteAnswer> {
  return new CellBlockDeleteDialog().showAsync();
}
