/**
 * 누름틀 고치기 대화상자
 *
 * 누름틀(ClickHere) 필드의 안내문, 메모, 필드 이름, 양식 모드 편집 가능 여부를 편집한다.
 */
import { ModalDialog } from './dialog';

export interface ClickHereProps {
  guide: string;
  memo: string;
  name: string;
  editable: boolean;
}

export class FieldEditDialog extends ModalDialog {
  private guideInput!: HTMLInputElement;
  private memoInput!: HTMLTextAreaElement;
  private nameInput!: HTMLInputElement;
  private editableCheckbox!: HTMLInputElement;

  /** 적용 콜백 */
  onApply: ((props: ClickHereProps) => void) | null = null;
  /** 닫힘 후 문서 입력 포커스 복구용 콜백 */
  onClose: (() => void) | null = null;

  private initialProps: ClickHereProps = { guide: '', memo: '', name: '', editable: true };

  constructor() {
    super('필드 입력 고치기', 420, false);
  }

  /** 대화상자를 열고 초기값을 설정한다 */
  showWith(props: ClickHereProps): void {
    this.initialProps = props;
    this.show();

    // 초기값 반영
    this.guideInput.value = props.guide;
    this.memoInput.value = props.memo;
    this.nameInput.value = props.name;
    this.editableCheckbox.checked = props.editable;

    // 안내문 입력에 포커스
    this.guideInput.focus();
    this.guideInput.select();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'field-edit-body';

    // ── 탭 헤더 (누름틀 탭만) ──
    const tabBar = document.createElement('div');
    tabBar.className = 'dialog-tabs';
    const tab = document.createElement('button');
    tab.className = 'dialog-tab active';
    tab.textContent = '누름틀';
    tab.type = 'button';
    tabBar.appendChild(tab);
    body.appendChild(tabBar);

    const panel = document.createElement('div');
    panel.className = 'field-edit-panel';

    // ── 입력할 내용의 안내문(P) ──
    const guideLabel = document.createElement('label');
    guideLabel.className = 'field-edit-label';
    guideLabel.textContent = '입력할 내용의 안내문(P):';
    panel.appendChild(guideLabel);

    this.guideInput = document.createElement('input');
    this.guideInput.type = 'text';
    this.guideInput.className = 'field-edit-input';
    panel.appendChild(this.guideInput);

    // ── 메모 내용(M) ──
    const memoLabel = document.createElement('label');
    memoLabel.className = 'field-edit-label';
    memoLabel.textContent = '메모 내용(M):';
    panel.appendChild(memoLabel);

    this.memoInput = document.createElement('textarea');
    this.memoInput.className = 'field-edit-textarea';
    this.memoInput.rows = 4;
    panel.appendChild(this.memoInput);

    // ── 필드 이름(N) ──
    const nameLabel = document.createElement('label');
    nameLabel.className = 'field-edit-label';
    nameLabel.textContent = '필드 이름(N):';
    panel.appendChild(nameLabel);

    this.nameInput = document.createElement('input');
    this.nameInput.type = 'text';
    this.nameInput.className = 'field-edit-input';
    panel.appendChild(this.nameInput);

    // ── 양식 모드에서 편집 가능(F) ──
    const editableRow = document.createElement('label');
    editableRow.className = 'field-edit-checkbox-row';
    this.editableCheckbox = document.createElement('input');
    this.editableCheckbox.type = 'checkbox';
    editableRow.appendChild(this.editableCheckbox);
    const editableText = document.createTextNode(' 양식 모드에서 편집 가능(F)');
    editableRow.appendChild(editableText);
    panel.appendChild(editableRow);

    body.appendChild(panel);
    return body;
  }

  protected onConfirm(): void {
    if (this.onApply) {
      this.onApply({
        guide: this.guideInput.value,
        memo: this.memoInput.value,
        name: this.nameInput.value,
        editable: this.editableCheckbox.checked,
      });
    }
  }

  override show(): void {
    super.show();

    // footer 버튼 텍스트를 "고치기(D)" / "취소"로 변경
    const footer = this.dialog.querySelector('.dialog-footer');
    if (footer) {
      const buttons = footer.querySelectorAll('button');
      if (buttons[0]) buttons[0].textContent = '고치기(D)';
      if (buttons[1]) buttons[1].textContent = '취소';
    }
  }

  override hide(): void {
    super.hide();
    this.onClose?.();
  }
}
