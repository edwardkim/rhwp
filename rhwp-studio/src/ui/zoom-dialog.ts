import { ModalDialog } from './dialog.ts';
import {
  MAX_MULTIPLE_PAGES,
  MIN_MULTIPLE_PAGES,
  normalizePageArrangement,
  type PageArrangement,
} from '../view/page-arrangement.ts';
import {
  normalizePageMovementSettings,
  resolvePageViewSettings,
  type PageMovementSettings,
} from '../view/page-movement.ts';
import {
  MAX_CUSTOM_ZOOM_PERCENT,
  MIN_CUSTOM_ZOOM_PERCENT,
  ZOOM_PRESET_PERCENTAGES,
  clampCustomZoomPercent,
  detectZoomChoice,
  validateCustomZoomPercent,
  type ZoomChoice,
  type ZoomDialogValue,
} from '../view/zoom-dialog-state.ts';

const CUSTOM_ZOOM_ERROR_ID = 'zoom-dialog-custom-error';

export interface ZoomDialogOptions {
  currentZoom: number;
  fitZooms: { fitWidth: number; fitPage: number };
  arrangement: PageArrangement;
  pageMovement: PageMovementSettings;
  onConfirm: (value: ZoomDialogValue) => void;
}

/** 한컴의 비율·쪽 모양 정보 구조를 rhwp 공통 ModalDialog 스타일로 표현한다. */
export class ZoomDialog extends ModalDialog {
  private readonly initialChoice: ZoomChoice;
  private readonly initialArrangement: PageArrangement;
  private readonly initialPageMovement: PageMovementSettings;
  private readonly callback: (value: ZoomDialogValue) => void;
  private customInput!: HTMLInputElement;
  private customError!: HTMLParagraphElement;
  private columnsInput!: HTMLInputElement;
  private rowsInput!: HTMLInputElement;
  private wheelHorizontalInput!: HTMLInputElement;
  private inputEnterHandler: ((event: KeyboardEvent) => void) | null = null;

  constructor(options: ZoomDialogOptions) {
    super('확대/축소', 540);
    this.initialChoice = detectZoomChoice(options.currentZoom, options.fitZooms);
    this.initialArrangement = normalizePageArrangement(options.arrangement);
    this.initialPageMovement = normalizePageMovementSettings(options.pageMovement);
    this.callback = options.onConfirm;
  }

  show(): void {
    super.show();
    this.dialog.classList.add('zoom-dialog');
    const confirm = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary');
    if (confirm) {
      confirm.textContent = '설정';
      this.inputEnterHandler = (event) => {
        if (event.target === this.customInput && event.key === 'Enter') {
          event.preventDefault();
          confirm.click();
        }
      };
      document.addEventListener('keydown', this.inputEnterHandler, true);
    }
  }

  override hide(): void {
    if (this.inputEnterHandler) {
      document.removeEventListener('keydown', this.inputEnterHandler, true);
      this.inputEnterHandler = null;
    }
    super.hide();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'zoom-dialog-body';
    body.append(
      this.createZoomSection(),
      this.createArrangementSection(),
      this.createMovementSection(),
    );
    return body;
  }

  protected onConfirm(): boolean {
    const zoomValue = this.selectedValue('zoom-choice');
    let customPercent: number | undefined;
    if (zoomValue === 'custom' && !this.customInput.disabled) {
      const validation = validateCustomZoomPercent(this.customInput.value);
      if (!validation.valid) {
        this.showCustomZoomError(validation.message);
        return false;
      }
      customPercent = validation.percent;
      this.clearCustomZoomError();
    }
    const arrangementValue = this.selectedValue('page-arrangement');
    const resolved = resolvePageViewSettings(
      this.arrangementFromValue(arrangementValue),
      {
        direction: this.selectedValue('page-movement'),
        wheelHorizontal: this.wheelHorizontalInput.checked,
      },
    );
    this.callback({
      zoomChoice: this.zoomChoiceFromValue(zoomValue, customPercent),
      arrangement: resolved.arrangement,
      pageMovement: resolved.movement,
    });
    return true;
  }

  private createZoomSection(): HTMLFieldSetElement {
    const section = this.section('비율');
    const grid = document.createElement('div');
    grid.className = 'zoom-dialog-grid';
    const presets = document.createElement('div');
    const adaptive = document.createElement('div');
    presets.className = 'zoom-dialog-choice-column';
    adaptive.className = 'zoom-dialog-choice-column';

    for (const percent of ZOOM_PRESET_PERCENTAGES) {
      presets.appendChild(this.radioRow(
        'zoom-choice',
        `preset:${percent}`,
        `${percent}%`,
        this.initialChoice.kind === 'preset' && this.initialChoice.percent === percent,
      ));
    }
    adaptive.append(
      this.radioRow(
        'zoom-choice',
        'fitWidth',
        '폭 맞춤',
        this.initialChoice.kind === 'fitWidth',
      ),
      this.radioRow(
        'zoom-choice',
        'fitPage',
        '쪽 맞춤',
        this.initialChoice.kind === 'fitPage',
      ),
    );

    const customRow = this.radioRow(
      'zoom-choice',
      'custom',
      '사용자 정의',
      this.initialChoice.kind === 'custom',
    );
    this.customInput = this.numberInput(
      this.initialChoice.kind === 'custom' ? this.initialChoice.percent : 100,
      MIN_CUSTOM_ZOOM_PERCENT,
      MAX_CUSTOM_ZOOM_PERCENT,
      '사용자 정의 배율',
    );
    this.customInput.setAttribute('aria-describedby', CUSTOM_ZOOM_ERROR_ID);
    this.customInput.setAttribute('aria-invalid', 'false');
    this.customInput.addEventListener('input', () => {
      if (validateCustomZoomPercent(this.customInput.value).valid) {
        this.clearCustomZoomError();
      }
    });
    const unit = document.createElement('span');
    unit.className = 'dialog-unit';
    unit.textContent = '%';
    customRow.append(this.customInput, unit);
    this.customError = document.createElement('p');
    this.customError.id = CUSTOM_ZOOM_ERROR_ID;
    this.customError.className = 'zoom-dialog-custom-error';
    this.customError.setAttribute('role', 'alert');
    this.customError.hidden = true;
    adaptive.append(customRow, this.customError);
    grid.append(presets, adaptive);
    section.appendChild(grid);
    section.addEventListener('change', () => this.updateDependentInputs());
    queueMicrotask(() => this.updateDependentInputs());
    return section;
  }

  private createMovementSection(): HTMLFieldSetElement {
    const section = this.section('쪽 이동');
    const options = document.createElement('div');
    options.className = 'zoom-dialog-choice-column';
    options.append(
      this.radioRow(
        'page-movement',
        'vertical',
        '세로 방향',
        this.initialPageMovement.direction === 'vertical',
      ),
      this.radioRow(
        'page-movement',
        'horizontal',
        '가로 방향',
        this.initialPageMovement.direction === 'horizontal',
      ),
    );

    const wheelLabel = document.createElement('label');
    wheelLabel.className = 'zoom-dialog-option zoom-dialog-wheel-option';
    this.wheelHorizontalInput = document.createElement('input');
    this.wheelHorizontalInput.type = 'checkbox';
    this.wheelHorizontalInput.checked = this.initialPageMovement.wheelHorizontal;
    this.wheelHorizontalInput.setAttribute(
      'aria-label',
      '마우스 휠을 사용하여 좌우로 스크롤하기',
    );
    const wheelText = document.createElement('span');
    wheelText.textContent = '마우스 휠을 사용하여 좌우로 스크롤하기';
    wheelLabel.append(this.wheelHorizontalInput, wheelText);
    options.appendChild(wheelLabel);
    section.appendChild(options);
    section.addEventListener('change', () => this.updateDependentInputs());
    queueMicrotask(() => this.updateDependentInputs());
    return section;
  }

  private createArrangementSection(): HTMLFieldSetElement {
    const section = this.section('쪽 모양');
    const grid = document.createElement('div');
    grid.className = 'zoom-dialog-grid';
    const standard = document.createElement('div');
    const multiple = document.createElement('div');
    standard.className = 'zoom-dialog-choice-column';
    multiple.className = 'zoom-dialog-choice-column';

    for (const [value, label] of [
      ['auto', '자동'],
      ['single', '한 쪽'],
      ['double', '두 쪽'],
      ['facing', '맞쪽'],
    ] as const) {
      standard.appendChild(this.radioRow(
        'page-arrangement',
        value,
        label,
        this.initialArrangement.kind === value,
      ));
    }

    const multipleRow = this.radioRow(
      'page-arrangement',
      'multiple',
      '여러 쪽',
      this.initialArrangement.kind === 'multiple',
    );
    const columns = this.initialArrangement.kind === 'multiple'
      ? this.initialArrangement.columns
      : MIN_MULTIPLE_PAGES;
    const rows = this.initialArrangement.kind === 'multiple'
      ? this.initialArrangement.rows
      : MIN_MULTIPLE_PAGES;
    this.columnsInput = this.numberInput(
      columns,
      MIN_MULTIPLE_PAGES,
      MAX_MULTIPLE_PAGES,
      '여러 쪽 가로 쪽 수',
    );
    this.rowsInput = this.numberInput(
      rows,
      MIN_MULTIPLE_PAGES,
      MAX_MULTIPLE_PAGES,
      '여러 쪽 세로 쪽 수',
    );
    const multiply = document.createElement('span');
    multiply.className = 'zoom-dialog-multiply';
    multiply.textContent = '×';
    multipleRow.append(this.columnsInput, multiply, this.rowsInput);
    multiple.appendChild(multipleRow);
    grid.append(standard, multiple);
    section.appendChild(grid);
    section.addEventListener('change', () => this.updateDependentInputs());
    queueMicrotask(() => this.updateDependentInputs());
    return section;
  }

  private section(title: string): HTMLFieldSetElement {
    const section = document.createElement('fieldset');
    section.className = 'dialog-section zoom-dialog-section';
    const legend = document.createElement('legend');
    legend.className = 'dialog-section-title';
    legend.textContent = title;
    section.appendChild(legend);
    return section;
  }

  private radioRow(
    name: string,
    value: string,
    labelText: string,
    checked: boolean,
  ): HTMLLabelElement {
    const label = document.createElement('label');
    label.className = 'zoom-dialog-option';
    const radio = document.createElement('input');
    radio.type = 'radio';
    radio.name = name;
    radio.value = value;
    radio.checked = checked;
    radio.setAttribute('aria-label', labelText);
    const text = document.createElement('span');
    text.textContent = labelText;
    label.append(radio, text);
    return label;
  }

  private numberInput(
    value: number,
    min: number,
    max: number,
    ariaLabel: string,
  ): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.className = 'dialog-input zoom-dialog-number';
    input.min = String(min);
    input.max = String(max);
    input.step = '1';
    input.value = String(value);
    input.setAttribute('aria-label', ariaLabel);
    return input;
  }

  private selectedValue(name: string): string {
    return this.dialog.querySelector<HTMLInputElement>(`input[name="${name}"]:checked`)?.value ?? '';
  }

  private zoomChoiceFromValue(value: string, customPercent?: number): ZoomChoice {
    if (value.startsWith('preset:')) {
      return { kind: 'preset', percent: clampCustomZoomPercent(Number(value.slice(7))) };
    }
    if (value === 'fitWidth') return { kind: 'fitWidth' };
    if (value === 'fitPage') return { kind: 'fitPage' };
    return {
      kind: 'custom',
      percent: customPercent ?? clampCustomZoomPercent(Number(this.customInput.value)),
    };
  }

  private showCustomZoomError(message: string): void {
    this.customError.textContent = message;
    this.customError.hidden = false;
    this.customInput.setAttribute('aria-invalid', 'true');
    this.customInput.focus();
    this.customInput.select();
  }

  private clearCustomZoomError(): void {
    if (!this.customError) return;
    this.customError.textContent = '';
    this.customError.hidden = true;
    this.customInput.setAttribute('aria-invalid', 'false');
  }

  private arrangementFromValue(value: string): PageArrangement {
    if (value === 'multiple') {
      return normalizePageArrangement({
        kind: 'multiple',
        columns: Number(this.columnsInput.value),
        rows: Number(this.rowsInput.value),
      });
    }
    return normalizePageArrangement({ kind: value });
  }

  private updateDependentInputs(): void {
    if (
      !this.customInput
      || !this.columnsInput
      || !this.rowsInput
      || !this.wheelHorizontalInput
    ) return;
    const horizontal = this.selectedValue('page-movement') === 'horizontal';
    if (horizontal) {
      const single = this.dialog.querySelector<HTMLInputElement>(
        'input[name="page-arrangement"][value="single"]',
      );
      if (single) single.checked = true;
    }
    this.dialog.querySelectorAll<HTMLInputElement>('input[name="page-arrangement"]')
      .forEach((input) => { input.disabled = horizontal && input.value !== 'single'; });
    const multiple = !horizontal && this.selectedValue('page-arrangement') === 'multiple';
    this.dialog.querySelectorAll<HTMLInputElement>('input[name="zoom-choice"]')
      .forEach((input) => { input.disabled = multiple; });
    this.customInput.disabled = multiple || this.selectedValue('zoom-choice') !== 'custom';
    if (this.customInput.disabled) this.clearCustomZoomError();
    this.columnsInput.disabled = !multiple;
    this.rowsInput.disabled = !multiple;
    this.wheelHorizontalInput.disabled = !horizontal;
  }
}
