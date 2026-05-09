import { ModalDialog } from './dialog';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';

const HWPUNIT_PER_MM = 283.46; // 1mm ≈ 283.46 HWPUNIT (7200/25.4)

function hwpunitToMm(hu: number): number {
  return Math.round(hu / HWPUNIT_PER_MM * 10) / 10;
}

function mmToHwpunit(mm: number): number {
  return Math.round(mm * HWPUNIT_PER_MM);
}

const LABEL_MIN_W = '80px';

export class ColumnSettingsDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private sectionIdx: number;

  private countInput!: HTMLInputElement;
  private typeSelect!: HTMLSelectElement;
  private sameWidthCheck!: HTMLInputElement;
  private spacingInput!: HTMLInputElement;

  constructor(wasm: WasmBridge, eventBus: EventBus, sectionIdx: number) {
    super('다단 설정', 360);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.sectionIdx = sectionIdx;
  }

  show(): void {
    super.show();
    this.populateFields();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    const addRow = (label: string): HTMLElement => {
      const row = document.createElement('div');
      row.style.cssText = 'display:flex;align-items:center;margin-bottom:10px;gap:8px;';
      const lbl = document.createElement('label');
      lbl.textContent = label;
      lbl.style.cssText = `min-width:${LABEL_MIN_W};font-size:13px;`;
      row.appendChild(lbl);
      body.appendChild(row);
      return row;
    };

    // 단 수
    const countRow = addRow('단 수');
    this.countInput = document.createElement('input');
    this.countInput.type = 'number';
    this.countInput.min = '1';
    this.countInput.max = '8';
    this.countInput.style.cssText = 'width:60px;padding:4px 6px;font-size:13px;';
    countRow.appendChild(this.countInput);

    // 단 종류
    const typeRow = addRow('종류');
    this.typeSelect = document.createElement('select');
    this.typeSelect.style.cssText = 'width:120px;padding:4px;font-size:13px;';
    for (const [val, text] of [['0', '일반'], ['1', '배분'], ['2', '평행']]) {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = text;
      this.typeSelect.appendChild(opt);
    }
    typeRow.appendChild(this.typeSelect);

    // 너비 동일
    const sameRow = addRow('너비 동일');
    this.sameWidthCheck = document.createElement('input');
    this.sameWidthCheck.type = 'checkbox';
    sameRow.appendChild(this.sameWidthCheck);

    // 단 간격
    const spacingRow = addRow('간격 (mm)');
    this.spacingInput = document.createElement('input');
    this.spacingInput.type = 'number';
    this.spacingInput.min = '0';
    this.spacingInput.step = '0.5';
    this.spacingInput.style.cssText = 'width:80px;padding:4px 6px;font-size:13px;';
    spacingRow.appendChild(this.spacingInput);

    return body;
  }

  private populateFields(): void {
    try {
      const def = this.wasm.getColumnDef(this.sectionIdx);
      this.countInput.value = String(Math.max(def.columnCount, 1));
      this.typeSelect.value = String(def.columnType);
      this.sameWidthCheck.checked = def.sameWidth;
      this.spacingInput.value = String(hwpunitToMm(def.spacing));
    } catch {
      this.countInput.value = '1';
      this.typeSelect.value = '0';
      this.sameWidthCheck.checked = true;
      this.spacingInput.value = '8';
    }
  }

  protected onOk(): void {
    const count = parseInt(this.countInput.value, 10) || 1;
    const type = parseInt(this.typeSelect.value, 10) || 0;
    const sameWidth = this.sameWidthCheck.checked ? 1 : 0;
    const spacingHu = mmToHwpunit(parseFloat(this.spacingInput.value) || 0);
    try {
      this.wasm.setColumnDef(this.sectionIdx, count, type, sameWidth, spacingHu);
      this.eventBus.emit('document-changed');
    } catch (err) {
      console.warn('[ColumnSettingsDialog] 다단 설정 실패:', err);
    }
    this.close();
  }
}
