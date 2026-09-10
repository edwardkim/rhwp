import { ModalDialog } from './dialog';
import './hyperlink-dialog.css';

export type HyperlinkEdit = { kind: 'save'; uri: string; text: string } | { kind: 'remove' };

export class HyperlinkDialog extends ModalDialog {
  private textInput!: HTMLInputElement;
  private uriInput!: HTMLInputElement;
  private error!: HTMLParagraphElement;

  constructor(
    private initial: { text: string; uri: string; existing: boolean; canInsertText: boolean },
    private apply: (edit: HyperlinkEdit) => void,
  ) { super(initial.existing ? '하이퍼링크 고치기' : '하이퍼링크', 560); }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-hyperlink-body';
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
      row.append(name, field);
      body.append(row);
      return field;
    };
    this.textInput = input('표시할 문자열', 'hyperlink-text');
    this.textInput.value = this.initial.text;
    this.textInput.readOnly = !this.initial.canInsertText;
    if (this.textInput.readOnly) this.textInput.title = '표시할 문자열은 문서에서 직접 편집할 수 있습니다.';
    const targets = document.createElement('fieldset');
    targets.className = 'dialog-hyperlink-target';
    const legend = document.createElement('legend');
    legend.textContent = '연결 대상';
    targets.append(legend);
    const panel = document.createElement('div');
    panel.className = 'dialog-hyperlink-panel';
    const webLabel = document.createElement('label');
    webLabel.htmlFor = 'hyperlink-uri';
    webLabel.textContent = '웹 주소';
    panel.append(webLabel);
    this.uriInput = document.createElement('input');
    this.uriInput.className = 'dialog-input';
    this.uriInput.id = 'hyperlink-uri';
    this.uriInput.type = 'url';
    this.uriInput.setAttribute('aria-label', '웹 주소');
    this.uriInput.placeholder = 'https://example.com';
    this.uriInput.value = this.initial.uri;
    panel.append(this.uriInput);
    targets.append(panel);
    body.append(targets);
    this.error = document.createElement('p');
    this.error.setAttribute('role', 'alert');
    this.error.id = 'hyperlink-error';
    this.uriInput.setAttribute('aria-describedby', this.error.id);
    body.append(this.error);
    return body;
  }

  protected onConfirm(): boolean {
    try {
      this.apply({ kind: 'save', uri: this.uriInput.value.trim(), text: this.textInput.value });
      return true;
    } catch (error) {
      this.error.textContent = error instanceof Error ? error.message : String(error);
      return false;
    }
  }

  override show(): void {
    super.show();
    this.dialog.classList.add('dialog-hyperlink');
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-label', this.initial.existing ? '하이퍼링크 고치기' : '하이퍼링크');
    const confirm = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
    confirm.textContent = this.initial.existing ? '고치기' : '넣기';
    const update = () => { confirm.disabled = !this.uriInput.value.trim() || !this.textInput.value.trim(); };
    this.uriInput.addEventListener('input', update);
    this.textInput.addEventListener('input', update);
    update();
    this.uriInput.focus();
    this.uriInput.select();
  }
}

class ExistingHyperlinkDialog extends ModalDialog {
  private accepted = false;
  constructor(private edit: () => void, private cancel: () => void) { super('하이퍼링크', 390); }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-hyperlink-confirm-body';
    body.textContent = '하이퍼링크가 이미 입력되어 있습니다.\n하이퍼링크를 고칠까요?';
    return body;
  }
  protected onConfirm(): void { this.accepted = true; }
  override show(): void {
    super.show();
    this.dialog.setAttribute('role', 'alertdialog');
    this.dialog.setAttribute('aria-label', '하이퍼링크');
    this.dialog.querySelector('.dialog-btn-primary')!.textContent = '고침';
  }
  override hide(): void {
    super.hide();
    if (this.accepted) this.edit();
    else this.cancel();
  }
}

export function confirmHyperlinkEdit(edit: () => void, cancel: () => void): void {
  new ExistingHyperlinkDialog(edit, cancel).show();
}
