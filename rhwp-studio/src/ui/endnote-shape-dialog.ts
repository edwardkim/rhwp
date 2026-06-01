import { ModalDialog } from './dialog';
import type { EventBus } from '@/core/event-bus';
import type { EndnoteShapeSettings } from '@/core/types';
import { HWPUNIT_PER_MM } from '@/core/hwp-constants';
import type { WasmBridge } from '@/core/wasm-bridge';

function hwpToMm(value: number): number {
  return Math.round(value / HWPUNIT_PER_MM * 10) / 10;
}

function mmToHwp(value: number, max = 300): number {
  if (!Number.isFinite(value)) return 0;
  return Math.round(Math.min(max, Math.max(0, value)) * HWPUNIT_PER_MM);
}

export class EndnoteShapeDialog extends ModalDialog {
  private settings!: EndnoteShapeSettings;
  private numberFormatSelect!: HTMLSelectElement;
  private prefixInput!: HTMLInputElement;
  private suffixInput!: HTMLInputElement;
  private separatorCheck!: HTMLInputElement;
  private lineTypeSelect!: HTMLSelectElement;
  private lineWidthSelect!: HTMLSelectElement;
  private lineColorInput!: HTMLInputElement;
  private separatorLengthInput!: HTMLInputElement;
  private marginTopInput!: HTMLInputElement;
  private noteSpacingInput!: HTMLInputElement;
  private marginBottomInput!: HTMLInputElement;
  private numberingContinue!: HTMLInputElement;
  private numberingRestart!: HTMLInputElement;
  private placementDocument!: HTMLInputElement;
  private placementSection!: HTMLInputElement;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private sectionIdx: number,
  ) {
    super('주석 모양', 520);
  }

  show(): void {
    this.settings = this.wasm.getEndnoteShape(this.sectionIdx);
    super.show();
    this.populate();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:12px 14px;display:flex;flex-direction:column;gap:10px;';

    const tabs = document.createElement('div');
    tabs.className = 'dialog-tabs';
    const tab = document.createElement('button');
    tab.type = 'button';
    tab.className = 'dialog-tab active';
    tab.textContent = '미주 모양';
    tabs.appendChild(tab);

    body.append(
      tabs,
      this.numberGroup(),
      this.spacingGroup(),
      this.numberingGroup(),
      this.contentNumberGroup(),
      this.placementGroup(),
    );
    return body;
  }

  protected onConfirm(): void {
    const next: EndnoteShapeSettings = {
      ...this.settings,
      numberFormat: this.numberFormatSelect.value,
      prefixChar: this.prefixInput.value.slice(0, 1),
      suffixChar: this.suffixInput.value.slice(0, 1),
      separatorEnabled: this.separatorCheck.checked,
      separatorLineType: this.separatorCheck.checked ? parseInt(this.lineTypeSelect.value, 10) : 0,
      separatorLineWidth: this.separatorCheck.checked ? parseInt(this.lineWidthSelect.value, 10) : 0,
      separatorColor: this.lineColorInput.value,
      separatorLength: this.separatorCheck.checked ? mmToHwp(parseFloat(this.separatorLengthInput.value)) : 0,
      separatorMarginTop: mmToHwp(parseFloat(this.marginTopInput.value)),
      noteSpacing: mmToHwp(parseFloat(this.noteSpacingInput.value)),
      separatorMarginBottom: mmToHwp(parseFloat(this.marginBottomInput.value)),
      numbering: this.numberingRestart.checked ? 'restartSection' : 'continue',
      placement: this.placementSection.checked ? 'sectionEnd' : 'documentEnd',
    };

    this.wasm.applyEndnoteShape(this.sectionIdx, next);
    this.eventBus.emit('document-changed');
  }

  private populate(): void {
    this.numberFormatSelect.value = this.settings.numberFormat || 'digit';
    this.prefixInput.value = this.settings.prefixChar || '';
    this.suffixInput.value = this.settings.suffixChar || ')';
    this.separatorCheck.checked = this.settings.separatorEnabled !== false;
    this.lineTypeSelect.value = String(this.settings.separatorLineType ?? 1);
    this.lineWidthSelect.value = String(this.settings.separatorLineWidth ?? 1);
    this.lineColorInput.value = this.settings.separatorColor || '#000000';
    this.separatorLengthInput.value = String(hwpToMm(this.settings.separatorLength || mmToHwp(50)));
    this.marginTopInput.value = String(hwpToMm(this.settings.separatorMarginTop || 0));
    this.noteSpacingInput.value = String(hwpToMm(this.settings.noteSpacing || 0));
    this.marginBottomInput.value = String(hwpToMm(this.settings.separatorMarginBottom || 0));
    this.numberingContinue.checked = this.settings.numbering !== 'restartSection';
    this.numberingRestart.checked = this.settings.numbering === 'restartSection';
    this.placementDocument.checked = this.settings.placement !== 'sectionEnd';
    this.placementSection.checked = this.settings.placement === 'sectionEnd';
    this.updateSeparatorEnabled();
  }

  private numberGroup(): HTMLElement {
    const group = this.group('번호 서식');
    this.numberFormatSelect = document.createElement('select');
    this.numberFormatSelect.className = 'dialog-select';
    for (const [value, label] of [
      ['digit', '1,2,3'],
      ['circledDigit', '①,②,③'],
      ['upperRoman', 'I,II,III'],
      ['lowerRoman', 'i,ii,iii'],
      ['upperAlpha', 'A,B,C'],
      ['lowerAlpha', 'a,b,c'],
      ['hangulSyllable', '가,나,다'],
      ['hangulJamo', 'ㄱ,ㄴ,ㄷ'],
      ['hangulDigit', '일,이,삼'],
      ['hanjaDigit', '一,二,三'],
    ]) {
      const option = document.createElement('option');
      option.value = value;
      option.textContent = label;
      this.numberFormatSelect.appendChild(option);
    }

    this.prefixInput = this.charInput();
    this.suffixInput = this.charInput();
    this.separatorCheck = document.createElement('input');
    this.separatorCheck.type = 'checkbox';
    this.separatorCheck.addEventListener('change', () => this.updateSeparatorEnabled());
    this.lineTypeSelect = this.select([
      ['1', '실선'],
      ['2', '긴 점선'],
      ['3', '점선'],
      ['4', '이중선'],
    ]);
    this.lineWidthSelect = this.select([
      ['1', '0.1 mm'],
      ['2', '0.2 mm'],
      ['3', '0.3 mm'],
      ['4', '0.5 mm'],
      ['5', '0.7 mm'],
    ]);
    this.separatorLengthInput = this.numberInput(50, 0, 300, 0.5);
    this.lineColorInput = document.createElement('input');
    this.lineColorInput.type = 'color';
    this.lineColorInput.style.cssText = 'width:54px;height:26px;padding:1px;';

    group.append(
      this.row(this.label('번호 모양'), this.numberFormatSelect),
      this.row(this.label('앞 장식 문자'), this.prefixInput, this.label('뒤 장식 문자'), this.suffixInput),
      this.checkboxRow(this.separatorCheck, '구분선 넣기'),
      this.row(this.label('종류'), this.lineTypeSelect, this.label('길이'), this.withUnit(this.separatorLengthInput, 'mm')),
      this.row(this.label('굵기'), this.lineWidthSelect, this.label('색'), this.lineColorInput),
    );
    return group;
  }

  private spacingGroup(): HTMLElement {
    const group = this.group('여백');
    this.marginTopInput = this.numberInput(0, 0, 100, 0.5);
    this.noteSpacingInput = this.numberInput(7, 0, 100, 0.5);
    this.marginBottomInput = this.numberInput(2, 0, 100, 0.5);
    group.append(
      this.row(this.label('구분선 위'), this.withUnit(this.marginTopInput, 'mm')),
      this.row(this.label('미주 사이'), this.withUnit(this.noteSpacingInput, 'mm')),
      this.row(this.label('구분선 아래'), this.withUnit(this.marginBottomInput, 'mm')),
    );
    return group;
  }

  private numberingGroup(): HTMLElement {
    const group = this.group('번호 매기기');
    this.numberingContinue = this.radio('endnote-numbering', 'continue');
    this.numberingRestart = this.radio('endnote-numbering', 'restartSection');
    group.append(
      this.radioRow(this.numberingContinue, '앞 구역에 이어서'),
      this.radioRow(this.numberingRestart, '현재 구역부터 새로 시작'),
    );
    return group;
  }

  private contentNumberGroup(): HTMLElement {
    const group = this.group('미주 내용 번호 속성');
    const normal = this.radio('endnote-content-number', 'normal');
    const small = this.radio('endnote-content-number', 'small');
    normal.checked = true;
    small.disabled = true;
    group.append(this.radioRow(normal, '보통'), this.radioRow(small, '작게'));
    return group;
  }

  private placementGroup(): HTMLElement {
    const group = this.group('미주 위치');
    this.placementDocument = this.radio('endnote-placement', 'documentEnd');
    this.placementSection = this.radio('endnote-placement', 'sectionEnd');
    group.append(
      this.radioRow(this.placementDocument, '문서의 끝'),
      this.radioRow(this.placementSection, '구역의 끝'),
    );
    return group;
  }

  private updateSeparatorEnabled(): void {
    const enabled = this.separatorCheck.checked;
    for (const el of [
      this.lineTypeSelect,
      this.lineWidthSelect,
      this.separatorLengthInput,
      this.lineColorInput,
    ]) {
      el.disabled = !enabled;
    }
  }

  private group(title: string): HTMLFieldSetElement {
    const fieldset = document.createElement('fieldset');
    fieldset.style.cssText = 'border:1px solid #d8dce5;padding:9px 10px 10px;margin:0;';
    const legend = document.createElement('legend');
    legend.textContent = title;
    legend.style.cssText = 'font-size:12px;color:#315fc0;padding:0 4px;';
    fieldset.appendChild(legend);
    return fieldset;
  }

  private row(...children: HTMLElement[]): HTMLElement {
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;align-items:center;gap:8px;margin:5px 0;font-size:13px;flex-wrap:wrap;';
    row.append(...children);
    return row;
  }

  private label(text: string): HTMLSpanElement {
    const label = document.createElement('span');
    label.textContent = text;
    label.style.cssText = 'min-width:78px;color:#222;';
    return label;
  }

  private charInput(): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'text';
    input.maxLength = 1;
    input.className = 'dialog-input';
    input.style.width = '44px';
    return input;
  }

  private numberInput(value: number, min: number, max: number, step: number): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.min = String(min);
    input.max = String(max);
    input.step = String(step);
    input.value = String(value);
    input.className = 'dialog-input';
    input.style.width = '72px';
    return input;
  }

  private select(options: [string, string][]): HTMLSelectElement {
    const select = document.createElement('select');
    select.className = 'dialog-select';
    for (const [value, label] of options) {
      const option = document.createElement('option');
      option.value = value;
      option.textContent = label;
      select.appendChild(option);
    }
    return select;
  }

  private withUnit(input: HTMLElement, unit: string): HTMLElement {
    const wrap = document.createElement('span');
    wrap.style.cssText = 'display:inline-flex;align-items:center;gap:4px;';
    const unitEl = document.createElement('span');
    unitEl.textContent = unit;
    unitEl.style.color = '#555';
    wrap.append(input, unitEl);
    return wrap;
  }

  private checkboxRow(input: HTMLInputElement, labelText: string): HTMLElement {
    const label = document.createElement('label');
    label.style.cssText = 'display:flex;align-items:center;gap:6px;margin:5px 0;font-size:13px;';
    label.append(input, document.createTextNode(labelText));
    return label;
  }

  private radio(name: string, value: string): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'radio';
    input.name = name;
    input.value = value;
    return input;
  }

  private radioRow(input: HTMLInputElement, labelText: string): HTMLElement {
    const label = document.createElement('label');
    label.style.cssText = 'display:inline-flex;align-items:center;gap:6px;margin:4px 14px 4px 0;font-size:13px;';
    label.append(input, document.createTextNode(labelText));
    return label;
  }
}
