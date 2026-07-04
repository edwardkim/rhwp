import { ModalDialog } from './dialog';
import type {
  GridLayer,
  GridOffsetMm,
  GridOrigin,
  GridPattern,
  GridSnapMode,
  GridViewSettings,
} from '../view/grid-settings';
import { convertGridOffsetForOrigin, normalizeGridSettings } from '../view/grid-settings';

/**
 * 격자 설정 대화상자 — 보기 격자와 표/개체 이동 간격을 설정한다.
 */
export class GridSettingsDialog extends ModalDialog {
  private visibleInput!: HTMLInputElement;
  private horizontalInput!: HTMLInputElement;
  private verticalInput!: HTMLInputElement;
  private offsetXInput!: HTMLInputElement;
  private offsetYInput!: HTMLInputElement;
  private moveStepInput!: HTMLInputElement;
  private callback: (settings: GridViewSettings, moveStepMm: number) => void;
  private currentSettings: GridViewSettings;
  private originBases: Record<GridOrigin, GridOffsetMm>;
  private currentMoveStepMm: number;
  private lastOrigin: GridOrigin;

  constructor(
    currentSettings: GridViewSettings,
    originBases: Record<GridOrigin, GridOffsetMm>,
    currentMoveStepMm: number,
    onConfirm: (settings: GridViewSettings, moveStepMm: number) => void,
  ) {
    super('격자 설정', 430);
    this.currentSettings = currentSettings;
    this.originBases = originBases;
    this.currentMoveStepMm = currentMoveStepMm;
    this.lastOrigin = currentSettings.origin;
    this.callback = onConfirm;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.padding = '14px 16px';
    body.style.display = 'flex';
    body.style.flexDirection = 'column';
    body.style.gap = '12px';

    this.visibleInput = document.createElement('input');
    this.visibleInput.type = 'checkbox';
    this.visibleInput.checked = this.currentSettings.visible;
    body.appendChild(this.checkboxRow(this.visibleInput, '격자 보기'));

    body.appendChild(this.radioGroup<GridPattern>('격자 모양', 'grid-pattern', [
      ['dots', '점'],
      ['horizontal', '가로선'],
      ['vertical', '세로선'],
      ['both', '가로/세로선'],
    ], this.currentSettings.pattern));

    body.appendChild(this.radioGroup<GridLayer>('격자 위치', 'grid-layer', [
      ['behindText', '글 뒤'],
      ['inFrontOfText', '글 앞'],
    ], this.currentSettings.layer));

    body.appendChild(this.radioGroup<GridSnapMode>('격자 방식', 'grid-snap-mode', [
      ['free', '상관 없이'],
      ['magnetic', '자석 효과'],
      ['gridOnly', '격자에만 붙이기'],
    ], this.currentSettings.snapMode));

    const spacing = this.group('격자 간격');
    this.horizontalInput = this.numberInput(this.currentSettings.horizontalMm);
    this.horizontalInput.name = 'grid-horz';
    this.verticalInput = this.numberInput(this.currentSettings.verticalMm);
    this.verticalInput.name = 'grid-vert';
    spacing.append(
      this.numberRow('가로', this.horizontalInput),
      this.numberRow('세로', this.verticalInput),
    );
    body.appendChild(spacing);

    body.appendChild(this.originGroup());

    const move = this.group('표/개체 이동');
    this.moveStepInput = this.numberInput(this.currentMoveStepMm);
    this.moveStepInput.name = 'move-step';
    move.appendChild(this.numberRow('이동 간격', this.moveStepInput));
    body.appendChild(move);

    return body;
  }

  protected onConfirm(): void {
    const settings = normalizeGridSettings({
      visible: this.visibleInput.checked,
      pattern: this.radioValue<GridPattern>('grid-pattern', this.currentSettings.pattern),
      layer: this.radioValue<GridLayer>('grid-layer', this.currentSettings.layer),
      snapMode: this.radioValue<GridSnapMode>('grid-snap-mode', this.currentSettings.snapMode),
      horizontalMm: parseFloat(this.horizontalInput.value),
      verticalMm: parseFloat(this.verticalInput.value),
      origin: this.radioValue<GridOrigin>('grid-origin', this.currentSettings.origin),
      offsetXmm: parseFloat(this.offsetXInput.value),
      offsetYmm: parseFloat(this.offsetYInput.value),
    });
    const moveStepMm = this.clampMoveStep(parseFloat(this.moveStepInput.value));
    this.callback(settings, moveStepMm);
  }

  private checkboxRow(input: HTMLInputElement, labelText: string): HTMLElement {
    const label = document.createElement('label');
    label.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:13px;color:var(--color-text);';
    label.append(input, document.createTextNode(labelText));
    return label;
  }

  private group(title: string): HTMLElement {
    const fieldset = document.createElement('fieldset');
    fieldset.style.cssText = 'border:1px solid var(--color-border-lighter);padding:10px 12px 12px;margin:0;';
    const legend = document.createElement('legend');
    legend.textContent = title;
    legend.style.cssText = 'font-size:12px;color:var(--color-primary-dark);padding:0 4px;';
    fieldset.appendChild(legend);
    return fieldset;
  }

  private radioGroup<T extends string>(
    title: string,
    name: string,
    options: [T, string][],
    current: T,
  ): HTMLElement {
    const fieldset = this.group(title);
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;flex-wrap:wrap;gap:10px 14px;color:var(--color-text);';
    for (const [value, labelText] of options) {
      const label = document.createElement('label');
      label.style.cssText = 'display:flex;align-items:center;gap:5px;font-size:13px;color:var(--color-text);';
      const input = document.createElement('input');
      input.type = 'radio';
      input.name = name;
      input.value = value;
      input.checked = value === current;
      label.append(input, document.createTextNode(labelText));
      row.appendChild(label);
    }
    fieldset.appendChild(row);
    return fieldset;
  }

  private originGroup(): HTMLElement {
    const fieldset = this.group('격자 기준 위치');
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;flex-wrap:wrap;gap:10px 14px;margin-bottom:8px;color:var(--color-text);';

    for (const [value, labelText] of [
      ['page', '쪽'],
      ['paper', '종이'],
    ] as [GridOrigin, string][]) {
      const label = document.createElement('label');
      label.style.cssText = 'display:flex;align-items:center;gap:5px;font-size:13px;color:var(--color-text);';
      const input = document.createElement('input');
      input.type = 'radio';
      input.name = 'grid-origin';
      input.value = value;
      input.checked = value === this.currentSettings.origin;
      input.addEventListener('change', () => this.onOriginChanged(value));
      label.append(input, document.createTextNode(labelText));
      row.appendChild(label);
    }

    this.offsetXInput = this.numberInput(this.currentSettings.offsetXmm, -500, 500);
    this.offsetXInput.name = 'grid-offset-x';
    this.offsetYInput = this.numberInput(this.currentSettings.offsetYmm, -500, 500);
    this.offsetYInput.name = 'grid-offset-y';
    fieldset.append(
      row,
      this.numberRow('가로', this.offsetXInput),
      this.numberRow('세로', this.offsetYInput),
    );
    return fieldset;
  }

  private numberInput(value: number, min = 0.5, max = 50): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.min = String(min);
    input.max = String(max);
    input.step = '0.5';
    input.value = String(value);
    input.className = 'dialog-input';
    input.style.cssText = 'width:78px;padding:3px 5px;';
    return input;
  }

  private numberRow(labelText: string, input: HTMLInputElement): HTMLElement {
    const row = document.createElement('label');
    row.style.cssText = 'display:inline-flex;align-items:center;gap:6px;margin-right:12px;font-size:13px;color:var(--color-text);';
    const label = document.createElement('span');
    label.textContent = labelText;
    label.style.minWidth = '56px';
    const unit = document.createElement('span');
    unit.textContent = 'mm';
    unit.style.color = 'var(--color-text-secondary)';
    row.append(label, input, unit);
    return row;
  }

  private radioValue<T extends string>(name: string, fallback: T): T {
    const selected = this.dialog.querySelector<HTMLInputElement>(`input[name="${name}"]:checked`);
    return (selected?.value as T | undefined) ?? fallback;
  }

  private onOriginChanged(nextOrigin: GridOrigin): void {
    if (nextOrigin === this.lastOrigin) return;
    const nextOffset = convertGridOffsetForOrigin(
      {
        x: parseFloat(this.offsetXInput.value),
        y: parseFloat(this.offsetYInput.value),
      },
      this.lastOrigin,
      nextOrigin,
      this.originBases,
    );
    this.offsetXInput.value = String(nextOffset.x);
    this.offsetYInput.value = String(nextOffset.y);
    this.lastOrigin = nextOrigin;
  }

  private clampMoveStep(value: number): number {
    if (!Number.isFinite(value)) return this.currentMoveStepMm;
    return Math.min(50, Math.max(0.5, value));
  }
}
