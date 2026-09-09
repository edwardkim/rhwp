import { ModalDialog } from './dialog';

export type HyperlinkEdit = { kind: 'save'; uri: string; text: string } | { kind: 'remove' };

export class HyperlinkDialog extends ModalDialog {
  private textInput!: HTMLInputElement;
  private uriInput!: HTMLInputElement;
  private error!: HTMLParagraphElement;

  constructor(
    private initial: { text: string; uri: string; existing: boolean; canInsertText: boolean },
    private apply: (edit: HyperlinkEdit) => void,
  ) { super('하이퍼링크', 440); }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    const input = (label: string, id: string): HTMLInputElement => {
      const row = document.createElement('div');
      row.className = 'dialog-row';
      const name = document.createElement('label');
      name.className = 'dialog-label';
      name.htmlFor = id;
      name.textContent = label;
      const field = document.createElement('input');
      field.className = 'dialog-input';
      field.id = id;
      field.type = 'text';
      field.style.flex = '1';
      field.style.textAlign = 'left';
      row.append(name, field);
      body.append(row);
      return field;
    };
    this.textInput = input('표시할 글자', 'hyperlink-text');
    this.textInput.value = this.initial.text;
    this.textInput.readOnly = !this.initial.canInsertText;
    this.uriInput = input('웹 주소', 'hyperlink-uri');
    this.uriInput.placeholder = 'https://example.com';
    this.uriInput.value = this.initial.uri;
    const hint = document.createElement('p');
    hint.textContent = 'http:// 또는 https://로 시작하는 주소를 입력하세요.';
    body.append(hint);
    this.error = document.createElement('p');
    this.error.setAttribute('role', 'alert');
    this.error.id = 'hyperlink-error';
    this.uriInput.setAttribute('aria-describedby', this.error.id);
    body.append(this.error);
    if (this.initial.existing) {
      const remove = document.createElement('button');
      remove.className = 'dialog-btn';
      remove.textContent = '연결 해제';
      remove.addEventListener('click', () => { if (this.submit({ kind: 'remove' })) this.hide(); });
      body.append(remove);
    }
    return body;
  }

  private submit(edit: HyperlinkEdit): boolean {
    try {
      this.apply(edit);
      return true;
    } catch (error) {
      this.error.textContent = error instanceof Error ? error.message : String(error);
      return false;
    }
  }

  protected onConfirm(): boolean {
    return this.submit({ kind: 'save', uri: this.uriInput.value, text: this.textInput.value });
  }

  override show(): void {
    super.show();
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-label', '하이퍼링크');
    this.uriInput.focus();
    this.uriInput.select();
  }
}
