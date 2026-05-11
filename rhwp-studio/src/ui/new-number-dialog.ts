import { ModalDialog } from './dialog';
import type { EventBus } from '@/core/event-bus';

export class NewNumberDialog extends ModalDialog {
  private wasm: any;
  private eventBus: EventBus;
  private cursorPos: { sec: number; para: number; offset: number };
  private numInput!: HTMLInputElement;

  constructor(wasm: any, eventBus: EventBus, pos: { sec: number; para: number; offset: number }) {
    super('새 번호로 시작', 300);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.cursorPos = pos;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '16px';

    const row = document.createElement('div');
    row.style.display = 'flex';
    row.style.alignItems = 'center';
    row.style.gap = '8px';

    const label = document.createElement('label');
    label.textContent = '시작 번호:';
    label.style.whiteSpace = 'nowrap';

    this.numInput = document.createElement('input');
    this.numInput.type = 'number';
    this.numInput.min = '1';
    this.numInput.max = '65535';
    this.numInput.value = '1';
    this.numInput.style.width = '80px';
    this.numInput.style.padding = '4px 8px';

    row.appendChild(label);
    row.appendChild(this.numInput);
    body.appendChild(row);

    return body;
  }

  show(): void {
    super.show();
    setTimeout(() => {
      this.numInput.focus();
      this.numInput.select();
    }, 50);
  }

  protected onConfirm(): void | boolean {
    const num = parseInt(this.numInput.value, 10);
    if (isNaN(num) || num < 1 || num > 65535) return false;
    try {
      this.wasm.insertNewNumber(
        this.cursorPos.sec, this.cursorPos.para, this.cursorPos.offset, num,
      );
      this.eventBus.emit('document-changed');
    } catch (e) {
      console.warn('[NewNumberDialog] 삽입 실패:', e);
    }
  }
}
