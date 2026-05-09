import { ModalDialog } from './dialog';

export class NumberingRestartDialog extends ModalDialog {
  private input!: HTMLInputElement;
  private callback: (startNum: number) => void;
  private defaultValue: number;

  constructor(defaultStartNum: number, callback: (startNum: number) => void) {
    super('새 번호로 시작', 300);
    this.callback = callback;
    this.defaultValue = defaultStartNum;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;align-items:center;gap:8px;margin-bottom:10px;';
    const lbl = document.createElement('label');
    lbl.textContent = '시작 번호';
    lbl.style.cssText = 'min-width:70px;font-size:13px;';
    this.input = document.createElement('input');
    this.input.type = 'number';
    this.input.min = '1';
    this.input.value = String(this.defaultValue);
    this.input.style.cssText = 'width:80px;padding:4px 6px;font-size:13px;';
    row.appendChild(lbl);
    row.appendChild(this.input);
    body.appendChild(row);
    return body;
  }

  show(): void {
    super.show();
    this.input.select();
  }

  protected onConfirm(): boolean {
    const num = parseInt(this.input.value, 10);
    if (isNaN(num) || num < 1) {
      this.input.style.outline = '2px solid red';
      this.input.focus();
      return false;
    }
    this.callback(num);
    return true;
  }
}
