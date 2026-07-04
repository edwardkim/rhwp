import { ModalDialog } from './dialog';

export type TableInsertRowColumnMode = 'row-above' | 'row-below' | 'col-left' | 'col-right';
export type TableDeleteRowColumnMode = 'row' | 'col';

export interface TableInsertRowColumnResult {
  mode: TableInsertRowColumnMode;
  count: number;
}

export interface TableDeleteRowColumnResult {
  mode: TableDeleteRowColumnMode;
}

const MAX_INSERT_COUNT = 63;

function clampInsertCount(value: string): number {
  const n = Number.parseInt(value, 10);
  if (!Number.isFinite(n)) return 1;
  return Math.max(1, Math.min(MAX_INSERT_COUNT, n));
}

function createRadio(
  name: string,
  value: string,
  label: string,
  checked: boolean,
  onChange: (value: string) => void,
): HTMLLabelElement {
  const wrapper = document.createElement('label');
  wrapper.style.display = 'flex';
  wrapper.style.alignItems = 'center';
  wrapper.style.gap = '6px';
  wrapper.style.minHeight = '24px';

  const radio = document.createElement('input');
  radio.type = 'radio';
  radio.name = name;
  radio.value = value;
  radio.checked = checked;
  radio.addEventListener('change', () => {
    if (radio.checked) onChange(value);
  });

  wrapper.appendChild(radio);
  wrapper.appendChild(document.createTextNode(label));
  return wrapper;
}

export class TableInsertRowColumnDialog extends ModalDialog {
  onApply: ((result: TableInsertRowColumnResult) => void) | null = null;

  private mode: TableInsertRowColumnMode;
  private countInput!: HTMLInputElement;
  private readonly radioName = `table-insert-row-column-${Math.random().toString(36).slice(2)}`;

  constructor(defaultMode: TableInsertRowColumnMode = 'row-above') {
    super('줄/칸 추가하기', 360);
    this.mode = defaultMode;
  }

  show(): void {
    super.show();
    const confirmBtn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
    if (confirmBtn) confirmBtn.textContent = '추가';
    this.countInput.focus();
    this.countInput.select();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    const addSection = document.createElement('div');
    addSection.className = 'dialog-section';

    const title = document.createElement('div');
    title.className = 'dialog-section-title';
    title.textContent = '추가';
    addSection.appendChild(title);

    const radioGroup = document.createElement('div');
    radioGroup.style.display = 'grid';
    radioGroup.style.gridTemplateColumns = '1fr 1fr';
    radioGroup.style.columnGap = '12px';
    radioGroup.style.rowGap = '2px';
    const setMode = (value: string) => {
      this.mode = value as TableInsertRowColumnMode;
    };
    radioGroup.appendChild(createRadio(this.radioName, 'row-above', '위쪽에 줄 추가하기', this.mode === 'row-above', setMode));
    radioGroup.appendChild(createRadio(this.radioName, 'row-below', '아래쪽에 줄 추가하기', this.mode === 'row-below', setMode));
    radioGroup.appendChild(createRadio(this.radioName, 'col-left', '왼쪽에 칸 추가하기', this.mode === 'col-left', setMode));
    radioGroup.appendChild(createRadio(this.radioName, 'col-right', '오른쪽에 칸 추가하기', this.mode === 'col-right', setMode));
    addSection.appendChild(radioGroup);

    const countRow = document.createElement('div');
    countRow.className = 'dialog-row';
    countRow.style.marginTop = '10px';

    const countLabel = document.createElement('label');
    countLabel.className = 'dialog-label';
    countLabel.textContent = '줄/칸 수:';
    countLabel.style.width = '78px';

    this.countInput = document.createElement('input');
    this.countInput.type = 'number';
    this.countInput.className = 'dialog-input';
    this.countInput.min = '1';
    this.countInput.max = String(MAX_INSERT_COUNT);
    this.countInput.value = '1';
    this.countInput.style.width = '64px';
    this.countInput.addEventListener('input', () => {
      this.countInput.value = String(clampInsertCount(this.countInput.value));
    });
    this.countInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.onConfirm();
        this.hide();
      }
    });

    const hint = document.createElement('span');
    hint.className = 'dialog-unit';
    hint.textContent = `최대 ${MAX_INSERT_COUNT}`;

    countRow.appendChild(countLabel);
    countRow.appendChild(this.countInput);
    countRow.appendChild(hint);
    addSection.appendChild(countRow);

    body.appendChild(addSection);
    return body;
  }

  protected onConfirm(): void {
    this.onApply?.({
      mode: this.mode,
      count: clampInsertCount(this.countInput.value),
    });
  }
}

export class TableDeleteRowColumnDialog extends ModalDialog {
  onApply: ((result: TableDeleteRowColumnResult) => void) | null = null;

  private mode: TableDeleteRowColumnMode;
  private readonly radioName = `table-delete-row-column-${Math.random().toString(36).slice(2)}`;

  constructor(defaultMode: TableDeleteRowColumnMode = 'row') {
    super('줄/칸 지우기', 320);
    this.mode = defaultMode;
  }

  show(): void {
    super.show();
    const confirmBtn = this.dialog.querySelector('.dialog-btn-primary') as HTMLButtonElement | null;
    if (confirmBtn) confirmBtn.textContent = '지우기';
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    const delSection = document.createElement('div');
    delSection.className = 'dialog-section';

    const title = document.createElement('div');
    title.className = 'dialog-section-title';
    title.textContent = '지우기';
    delSection.appendChild(title);

    const radioGroup = document.createElement('div');
    radioGroup.style.display = 'grid';
    radioGroup.style.gridTemplateColumns = '1fr 1fr';
    radioGroup.style.columnGap = '12px';
    const setMode = (value: string) => {
      this.mode = value as TableDeleteRowColumnMode;
    };
    radioGroup.appendChild(createRadio(this.radioName, 'row', '줄', this.mode === 'row', setMode));
    radioGroup.appendChild(createRadio(this.radioName, 'col', '칸', this.mode === 'col', setMode));
    delSection.appendChild(radioGroup);

    body.appendChild(delSection);
    return body;
  }

  protected onConfirm(): void {
    this.onApply?.({ mode: this.mode });
  }
}
