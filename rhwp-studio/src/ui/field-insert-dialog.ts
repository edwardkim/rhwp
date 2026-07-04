/**
 * 누름틀 삽입 대화상자
 *
 * 현재 커서 위치에 ClickHere 필드를 만들기 위한 안내문, 메모, 필드 이름,
 * 양식 모드 편집 가능 여부를 입력한다.
 */
import { ModalDialog } from './dialog';
import type { ClickHereProps } from './field-edit-dialog';

export class FieldInsertDialog extends ModalDialog {
  private guideInput!: HTMLInputElement;
  private memoInput!: HTMLTextAreaElement;
  private nameInput!: HTMLInputElement;
  private editableCheckbox!: HTMLInputElement;

  onApply: ((props: ClickHereProps) => void) | null = null;

  constructor() {
    super('필드 입력', 420, false);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'field-edit-body';

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

    const guideLabel = document.createElement('label');
    guideLabel.className = 'field-edit-label';
    guideLabel.textContent = '입력할 내용의 안내문(P):';
    panel.appendChild(guideLabel);

    this.guideInput = document.createElement('input');
    this.guideInput.type = 'text';
    this.guideInput.className = 'field-edit-input';
    this.guideInput.value = '입력하세요';
    panel.appendChild(this.guideInput);

    const memoLabel = document.createElement('label');
    memoLabel.className = 'field-edit-label';
    memoLabel.textContent = '메모 내용(M):';
    panel.appendChild(memoLabel);

    this.memoInput = document.createElement('textarea');
    this.memoInput.className = 'field-edit-textarea';
    this.memoInput.rows = 4;
    panel.appendChild(this.memoInput);

    const nameLabel = document.createElement('label');
    nameLabel.className = 'field-edit-label';
    nameLabel.textContent = '필드 이름(N):';
    panel.appendChild(nameLabel);

    this.nameInput = document.createElement('input');
    this.nameInput.type = 'text';
    this.nameInput.className = 'field-edit-input';
    panel.appendChild(this.nameInput);

    const editableRow = document.createElement('label');
    editableRow.className = 'field-edit-checkbox-row';
    this.editableCheckbox = document.createElement('input');
    this.editableCheckbox.type = 'checkbox';
    this.editableCheckbox.checked = true;
    editableRow.appendChild(this.editableCheckbox);
    editableRow.appendChild(document.createTextNode(' 양식 모드에서 편집 가능(F)'));
    panel.appendChild(editableRow);

    body.appendChild(panel);
    return body;
  }

  protected onConfirm(): void {
    this.onApply?.({
      guide: this.guideInput.value,
      memo: this.memoInput.value,
      name: this.nameInput.value,
      editable: this.editableCheckbox.checked,
    });
  }

  override show(): void {
    super.show();

    const footer = this.dialog.querySelector('.dialog-footer');
    if (footer) {
      const buttons = footer.querySelectorAll('button');
      if (buttons[0]) buttons[0].textContent = '넣기(D)';
      if (buttons[1]) buttons[1].textContent = '취소';
    }

    this.guideInput.focus();
    this.guideInput.select();
  }
}
