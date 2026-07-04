import { ModalDialog } from './dialog';
import type { EventBus } from '@/core/event-bus';
import type { EndnoteShapeSettings } from '@/core/types';
import { HWPUNIT_PER_MM } from '@/core/hwp-constants';
import type { WasmBridge } from '@/core/wasm-bridge';

type LineTypeChoice = {
  value: string;
  label: string;
  css: 'solid' | 'dashed' | 'dotted' | 'double';
  dash?: string;
};

type LineWidthChoice = {
  value: string;
  label: string;
  px: number;
};

const LINE_TYPE_CHOICES: LineTypeChoice[] = [
  { value: '1', label: '실선', css: 'solid' },
  { value: '2', label: '파선', css: 'dashed', dash: '9 4' },
  { value: '3', label: '점선', css: 'dotted', dash: '1 4' },
  { value: '4', label: '쇄선', css: 'dashed', dash: '9 4 2 4' },
  { value: '5', label: '이점 쇄선', css: 'dashed', dash: '9 4 2 4 2 4' },
  { value: '6', label: '긴 파선', css: 'dashed', dash: '14 5' },
  { value: '8', label: '이중선', css: 'double' },
  { value: '9', label: '얇고 굵은 이중선', css: 'double' },
  { value: '10', label: '굵고 얇은 이중선', css: 'double' },
  { value: '11', label: '삼중선', css: 'double' },
];

const LINE_WIDTH_CHOICES: LineWidthChoice[] = [
  { value: '0', label: '0.1mm', px: 1 },
  { value: '1', label: '0.12mm', px: 1.2 },
  { value: '2', label: '0.15mm', px: 1.4 },
  { value: '3', label: '0.2mm', px: 1.6 },
  { value: '4', label: '0.25mm', px: 1.8 },
  { value: '5', label: '0.3mm', px: 2 },
  { value: '6', label: '0.4mm', px: 2.4 },
  { value: '7', label: '0.5mm', px: 2.8 },
  { value: '8', label: '0.6mm', px: 3.2 },
  { value: '9', label: '0.7mm', px: 3.6 },
  { value: '10', label: '1mm', px: 4.2 },
  { value: '11', label: '1.5mm', px: 5 },
  { value: '12', label: '2mm', px: 6 },
  { value: '13', label: '3mm', px: 7.5 },
  { value: '14', label: '4mm', px: 9 },
  { value: '15', label: '5mm', px: 10.5 },
];

const COLOR_SWATCHES = [
  '#000000', '#ffffff', '#ff0000', '#ff7f00', '#ffff00', '#00b050', '#00b0f0', '#0000ff',
  '#7030a0', '#808080', '#c0c0c0', '#c00000', '#ffc000', '#92d050', '#00b050', '#00b0f0',
  '#002060', '#7030a0', '#f4cccc', '#fce5cd', '#fff2cc', '#d9ead3', '#d9eaf7', '#d9d2e9',
  '#eadcf8', '#e6e6e6', '#f9cb9c', '#ffe599', '#b6d7a8', '#76a5af', '#6fa8dc', '#8e7cc3',
];

let dialogInstanceSeq = 0;

function normalizeColor(value: string | undefined | null): string {
  const text = (value || '').trim();
  if (/^#[0-9a-fA-F]{6}$/.test(text)) {
    return text.toLowerCase();
  }
  return '#000000';
}

function normalizeEndnotePlacement(value: string | undefined | null): 'documentEnd' | 'sectionEnd' {
  switch ((value || '').trim()) {
    case 'sectionEnd':
    case 'belowText':
    case 'END_OF_SECTION':
    case 'BELOW_TEXT':
      return 'sectionEnd';
    case 'documentEnd':
    case 'eachColumn':
    case 'rightColumn':
    case 'END_OF_DOCUMENT':
    case 'EACH_COLUMN':
    case 'RIGHT_COLUMN':
    default:
      return 'documentEnd';
  }
}

function normalizeEndnoteNumbering(value: string | undefined | null): 'continue' | 'restartSection' {
  switch ((value || '').trim()) {
    case 'restartSection':
    case 'restartPage':
    case 'ON_SECTION':
    case 'ON_PAGE':
    case 'RESTART_SECTION':
    case 'RESTART_PAGE':
      return 'restartSection';
    case 'continue':
    case 'CONTINUOUS':
    default:
      return 'continue';
  }
}

function hwpToMm(value: number): number {
  return Math.round(value / HWPUNIT_PER_MM * 10) / 10;
}

function mmToHwp(value: number, max = 300): number {
  if (!Number.isFinite(value)) return 0;
  return Math.round(Math.min(max, Math.max(0, value)) * HWPUNIT_PER_MM);
}

export class EndnoteShapeDialog extends ModalDialog {
  private readonly radioNameSuffix = ++dialogInstanceSeq;
  private settings!: EndnoteShapeSettings;
  private numberFormatSelect!: HTMLSelectElement;
  private prefixInput!: HTMLInputElement;
  private suffixInput!: HTMLInputElement;
  private separatorCheck!: HTMLInputElement;
  private lineTypeSelect!: HTMLSelectElement;
  private lineTypeButton!: HTMLButtonElement;
  private lineTypeMenu!: HTMLDivElement;
  private lineWidthSelect!: HTMLSelectElement;
  private lineWidthButton!: HTMLButtonElement;
  private lineWidthMenu!: HTMLDivElement;
  private lineColorInput!: HTMLInputElement;
  private lineColorButton!: HTMLButtonElement;
  private lineColorMenu!: HTMLDivElement;
  private separatorLengthModeSelect!: HTMLSelectElement;
  private separatorLengthInput!: HTMLInputElement;
  private marginTopInput!: HTMLInputElement;
  private noteSpacingInput!: HTMLInputElement;
  private marginBottomInput!: HTMLInputElement;
  private numberingContinue!: HTMLInputElement;
  private numberingRestart!: HTMLInputElement;
  private placementDocument!: HTMLInputElement;
  private placementSection!: HTMLInputElement;
  private popupDismissPointerHandler: ((event: PointerEvent) => void) | null = null;
  private popupDismissFocusHandler: ((event: FocusEvent) => void) | null = null;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private sectionIdx: number,
  ) {
    super('미주', 620);
  }

  show(): void {
    this.settings = this.wasm.getEndnoteShape(this.sectionIdx);
    super.show();
    this.attachPopupDismissHandlers();
    this.populate();
  }

  hide(): void {
    this.detachPopupDismissHandlers();
    this.closePopupMenus();
    super.hide();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.cssText = 'padding:12px 14px;display:flex;flex-direction:column;gap:10px;min-width:560px;';

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
    this.setLineTypeValue(this.settings.separatorLineType ?? 1);
    this.setLineWidthValue(this.settings.separatorLineWidth ?? 1);
    this.setSeparatorColor(this.settings.separatorColor);
    this.updateLineTypePreview();
    this.updateLineWidthPreview();
    this.separatorLengthModeSelect.value = 'custom';
    this.separatorLengthInput.value = String(hwpToMm(this.settings.separatorLength || mmToHwp(50)));
    this.marginTopInput.value = String(hwpToMm(this.settings.separatorMarginTop || 0));
    this.noteSpacingInput.value = String(hwpToMm(this.settings.noteSpacing || 0));
    this.marginBottomInput.value = String(hwpToMm(this.settings.separatorMarginBottom || 0));
    this.setNumberingRadio(normalizeEndnoteNumbering(this.settings.numbering));
    this.setPlacementRadio(normalizeEndnotePlacement(this.settings.placement));
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
    this.buildLineTypeControl();
    this.buildLineWidthControl();
    this.separatorLengthModeSelect = this.select([['custom', '사용자']]);
    this.separatorLengthModeSelect.style.width = '86px';
    this.separatorLengthInput = this.numberInput(50, 0, 300, 0.5);
    this.buildColorControl();

    group.append(
      this.row(this.label('번호 모양'), this.numberFormatSelect),
      this.row(this.label('앞 장식 문자'), this.prefixInput, this.label('뒤 장식 문자'), this.suffixInput),
      this.checkboxRow(this.separatorCheck, '구분선 넣기'),
      this.pairRow(
        '종류',
        this.linePreviewSelect(this.lineTypeSelect, this.lineTypeButton, this.lineTypeMenu),
        '길이',
        this.inlineControls(
          this.separatorLengthModeSelect,
          this.withUnit(this.separatorLengthInput, 'mm'),
        ),
      ),
      this.pairRow('굵기', this.widthPreviewSelect(), '색', this.colorPicker()),
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
    this.numberingContinue = this.radio(`endnote-numbering-${this.radioNameSuffix}`, 'continue');
    this.numberingRestart = this.radio(`endnote-numbering-${this.radioNameSuffix}`, 'restartSection');
    this.numberingContinue.checked = true;
    this.numberingContinue.defaultChecked = true;
    group.append(
      this.radioRow(this.numberingContinue, '앞 구역에 이어서'),
      this.radioRow(this.numberingRestart, '현재 구역부터 새로 시작'),
    );
    return group;
  }

  private contentNumberGroup(): HTMLElement {
    const group = this.group('미주 내용 번호 속성');
    const normal = this.radio(`endnote-content-number-${this.radioNameSuffix}`, 'normal');
    const small = this.radio(`endnote-content-number-${this.radioNameSuffix}`, 'small');
    normal.checked = true;
    normal.defaultChecked = true;
    small.disabled = true;
    group.append(this.radioRow(normal, '보통'), this.radioRow(small, '작게'));
    return group;
  }

  private placementGroup(): HTMLElement {
    const group = this.group('미주 위치');
    this.placementDocument = this.radio(`endnote-placement-${this.radioNameSuffix}`, 'documentEnd');
    this.placementSection = this.radio(`endnote-placement-${this.radioNameSuffix}`, 'sectionEnd');
    this.placementDocument.checked = true;
    this.placementDocument.defaultChecked = true;
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
      this.lineTypeButton,
      this.lineWidthSelect,
      this.lineWidthButton,
      this.separatorLengthModeSelect,
      this.separatorLengthInput,
      this.lineColorInput,
      this.lineColorButton,
    ]) {
      el.disabled = !enabled;
    }
  }

  private setNumberingRadio(numbering: 'continue' | 'restartSection'): void {
    this.numberingContinue.checked = numbering === 'continue';
    this.numberingRestart.checked = numbering === 'restartSection';
    if (!this.numberingContinue.checked && !this.numberingRestart.checked) {
      this.numberingContinue.checked = true;
    }
  }

  private setPlacementRadio(placement: 'documentEnd' | 'sectionEnd'): void {
    this.placementDocument.checked = placement === 'documentEnd';
    this.placementSection.checked = placement === 'sectionEnd';
    if (!this.placementDocument.checked && !this.placementSection.checked) {
      this.placementDocument.checked = true;
    }
  }

  private group(title: string): HTMLFieldSetElement {
    const fieldset = document.createElement('fieldset');
    fieldset.style.cssText = 'border:1px solid var(--color-border-lighter);padding:9px 10px 10px;margin:0;';
    const legend = document.createElement('legend');
    legend.textContent = title;
    legend.style.cssText = 'font-size:12px;color:var(--color-primary-dark);padding:0 4px;';
    fieldset.appendChild(legend);
    return fieldset;
  }

  private row(...children: HTMLElement[]): HTMLElement {
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;align-items:center;gap:8px;margin:5px 0;font-size:13px;flex-wrap:wrap;min-height:28px;';
    row.append(...children);
    return row;
  }

  private pairRow(
    leftLabel: string,
    leftControl: HTMLElement,
    rightLabel: string,
    rightControl: HTMLElement,
  ): HTMLElement {
    const row = document.createElement('div');
    row.style.cssText = [
      'display:grid;grid-template-columns:48px 118px 48px minmax(0,1fr);',
      'align-items:center;column-gap:10px;row-gap:6px;margin:5px 0;',
      'font-size:13px;min-height:28px;',
    ].join('');
    row.append(
      this.pairLabel(leftLabel),
      leftControl,
      this.pairLabel(rightLabel),
      rightControl,
    );
    return row;
  }

  private pairLabel(text: string): HTMLSpanElement {
    const label = document.createElement('span');
    label.textContent = text;
    label.style.cssText = 'color:var(--color-text);text-align:right;white-space:nowrap;';
    return label;
  }

  private label(text: string): HTMLSpanElement {
    const label = document.createElement('span');
    label.textContent = text;
    label.style.cssText = 'min-width:78px;color:var(--color-text);';
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

  private hiddenSelect(options: [string, string][]): HTMLSelectElement {
    const select = this.select(options);
    select.style.display = 'none';
    return select;
  }

  private linePreviewSelect(
    select: HTMLSelectElement,
    button: HTMLButtonElement,
    menu: HTMLDivElement,
  ): HTMLElement {
    const wrap = this.popupWrap();
    wrap.append(select, button, menu);
    return wrap;
  }

  private widthPreviewSelect(): HTMLElement {
    const wrap = this.popupWrap();
    wrap.append(this.lineWidthSelect, this.lineWidthButton, this.lineWidthMenu);
    return wrap;
  }

  private colorPicker(): HTMLElement {
    const wrap = this.popupWrap();
    wrap.append(this.lineColorInput, this.lineColorButton, this.lineColorMenu);
    return wrap;
  }

  private popupWrap(): HTMLElement {
    const wrap = document.createElement('span');
    wrap.dataset.endnotePopupRoot = '1';
    wrap.style.cssText = 'position:relative;display:inline-flex;align-items:center;';
    return wrap;
  }

  private inlineControls(...children: HTMLElement[]): HTMLElement {
    const wrap = document.createElement('span');
    wrap.style.cssText = 'display:inline-flex;align-items:center;gap:8px;min-width:0;';
    wrap.append(...children);
    return wrap;
  }

  private buildLineTypeControl(): void {
    this.lineTypeSelect = this.hiddenSelect(LINE_TYPE_CHOICES.map(choice => [choice.value, choice.label]));
    this.lineTypeButton = this.previewButton();
    this.lineTypeMenu = this.popupMenu(178);
    for (const choice of LINE_TYPE_CHOICES) {
      const option = this.menuOption(() => {
        this.setLineTypeValue(choice.value);
        this.updateLineTypePreview();
        this.closePopupMenus();
      });
      option.title = choice.label;
      option.append(this.linePreview(choice, 2, 'var(--color-text)'));
      this.lineTypeMenu.appendChild(option);
    }
    this.lineTypeButton.addEventListener('click', (event) => {
      event.stopPropagation();
      this.toggleMenu(this.lineTypeMenu);
    });
    this.updateLineTypePreview();
  }

  private buildLineWidthControl(): void {
    this.lineWidthSelect = this.hiddenSelect(LINE_WIDTH_CHOICES.map(choice => [choice.value, choice.label]));
    this.lineWidthButton = this.previewButton();
    this.lineWidthMenu = this.popupMenu(136);
    for (const choice of LINE_WIDTH_CHOICES) {
      const option = this.menuOption(() => {
        this.setLineWidthValue(choice.value);
        this.updateLineWidthPreview();
        this.closePopupMenus();
      });
      const label = document.createElement('span');
      label.textContent = choice.label;
      label.style.cssText = 'width:50px;color:var(--color-text);';
      option.append(label, this.widthPreview(choice));
      this.lineWidthMenu.appendChild(option);
    }
    this.lineWidthButton.addEventListener('click', (event) => {
      event.stopPropagation();
      this.toggleMenu(this.lineWidthMenu);
    });
    this.updateLineWidthPreview();
  }

  private buildColorControl(): void {
    this.lineColorInput = document.createElement('input');
    this.lineColorInput.type = 'color';
    this.lineColorInput.value = '#000000';
    this.lineColorInput.style.cssText = [
      'position:absolute;left:0;top:0;width:1px;height:1px;opacity:0;',
      'pointer-events:none;border:0;padding:0;',
    ].join('');
    this.lineColorInput.addEventListener('input', () => this.setSeparatorColor(this.lineColorInput.value));
    this.lineColorInput.addEventListener('change', () => this.setSeparatorColor(this.lineColorInput.value));

    this.lineColorButton = this.previewButton();
    this.lineColorButton.style.width = '74px';
    this.lineColorMenu = this.popupMenu(220);
    this.lineColorMenu.style.display = 'none';
    this.lineColorMenu.dataset.display = 'grid';
    this.lineColorMenu.style.gridTemplateColumns = 'repeat(8, 20px)';
    this.lineColorMenu.style.gap = '4px';
    this.lineColorMenu.style.padding = '8px';

    for (const color of COLOR_SWATCHES) {
      const swatch = document.createElement('button');
      swatch.type = 'button';
      swatch.title = color;
      swatch.dataset.color = color;
      swatch.style.cssText = [
        'width:20px;height:20px;border:1px solid var(--color-border);background:var(--color-surface);padding:0;',
        'display:flex;align-items:center;justify-content:center;cursor:pointer;',
      ].join('');
      const chip = document.createElement('span');
      chip.style.cssText = `display:block;width:14px;height:14px;background:${color};border:1px solid var(--color-border-dark);`;
      swatch.appendChild(chip);
      swatch.addEventListener('click', (event) => {
        event.stopPropagation();
        this.setSeparatorColor(color);
        this.closePopupMenus();
      });
      this.lineColorMenu.appendChild(swatch);
    }

    const custom = document.createElement('button');
    custom.type = 'button';
    custom.textContent = '다른 색...';
    custom.style.cssText = [
      'grid-column:1 / -1;height:24px;border:1px solid var(--color-border);background:var(--color-surface);',
      'font-size:12px;color:var(--color-text);cursor:pointer;margin-top:2px;color-scheme:inherit;',
    ].join('');
    custom.addEventListener('click', (event) => {
      event.stopPropagation();
      this.closePopupMenus();
      this.lineColorInput.click();
    });
    this.lineColorMenu.appendChild(custom);

    this.lineColorButton.addEventListener('click', (event) => {
      event.stopPropagation();
      this.toggleMenu(this.lineColorMenu);
    });
    this.updateColorPreview();
  }

  private previewButton(): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.style.cssText = [
      'width:104px;height:26px;border:1px solid var(--color-border);background:var(--color-surface);',
      'display:flex;align-items:center;justify-content:center;padding:0 20px 0 8px;',
      'position:relative;cursor:pointer;color:var(--color-text);color-scheme:inherit;',
    ].join('');
    const arrow = document.createElement('span');
    arrow.textContent = '▾';
    arrow.dataset.dropdownArrow = '1';
    arrow.style.cssText = 'position:absolute;right:6px;top:4px;color:var(--color-text-secondary);font-size:11px;';
    button.appendChild(arrow);
    return button;
  }

  private popupMenu(width: number): HTMLDivElement {
    const menu = document.createElement('div');
    menu.style.cssText = [
      'display:none;position:absolute;left:0;top:27px;z-index:1200;',
      `width:${width}px;background:var(--color-surface);border:1px solid var(--color-border);box-shadow:var(--shadow-dropdown);`,
      'padding:4px;max-height:260px;overflow:auto;color:var(--color-text);',
    ].join('');
    menu.addEventListener('click', event => event.stopPropagation());
    return menu;
  }

  private menuOption(onClick: () => void): HTMLButtonElement {
    const option = document.createElement('button');
    option.type = 'button';
    option.style.cssText = [
      'width:100%;height:24px;border:0;background:var(--color-surface);display:flex;align-items:center;',
      'gap:8px;padding:2px 8px;cursor:pointer;color:var(--color-text);color-scheme:inherit;',
    ].join('');
    option.addEventListener('mouseenter', () => { option.style.background = 'var(--color-accent-bg)'; });
    option.addEventListener('mouseleave', () => { option.style.background = 'var(--color-surface)'; });
    option.addEventListener('click', event => {
      event.stopPropagation();
      onClick();
    });
    return option;
  }

  private linePreview(choice: LineTypeChoice, widthPx: number, color: string): HTMLElement {
    const preview = document.createElement('span');
    preview.style.cssText = 'display:flex;align-items:center;width:92px;height:16px;';
    if (choice.css === 'double') {
      const inner = document.createElement('span');
      inner.style.cssText = [
        'display:block;width:82px;height:7px;border-top:2px solid currentColor;',
        'border-bottom:2px solid currentColor;color:inherit;',
      ].join('');
      preview.style.color = color;
      preview.appendChild(inner);
      return preview;
    }
    const line = document.createElement('span');
    const style = choice.css === 'dashed' || choice.css === 'dotted' ? choice.css : 'solid';
    line.style.cssText = [
      'display:block;width:82px;height:0;border-top-style:', style, ';',
      `border-top-width:${widthPx}px;border-top-color:${color};`,
    ].join('');
    preview.appendChild(line);
    return preview;
  }

  private widthPreview(choice: LineWidthChoice): HTMLElement {
    const preview = document.createElement('span');
    preview.style.cssText = 'display:flex;align-items:center;width:72px;height:16px;';
    const line = document.createElement('span');
    line.style.cssText = [
      'display:block;width:62px;height:0;border-top-style:solid;',
      `border-top-width:${choice.px}px;border-top-color:var(--color-text);`,
    ].join('');
    preview.appendChild(line);
    return preview;
  }

  private updateLineTypePreview(): void {
    const choice = LINE_TYPE_CHOICES.find(item => item.value === this.lineTypeSelect.value)
      ?? LINE_TYPE_CHOICES[0];
    this.setLineTypeValue(choice.value);
    this.replaceButtonPreview(this.lineTypeButton, this.linePreview(choice, 2, 'var(--color-text)'));
  }

  private updateLineWidthPreview(): void {
    const choice = LINE_WIDTH_CHOICES.find(item => item.value === this.lineWidthSelect.value)
      ?? LINE_WIDTH_CHOICES[0];
    this.setLineWidthValue(choice.value);
    this.replaceButtonPreview(this.lineWidthButton, this.widthPreview(choice));
  }

  private setLineTypeValue(value: string | number | undefined | null): void {
    const text = String(value ?? '');
    const choice = LINE_TYPE_CHOICES.find(item => item.value === text) ?? LINE_TYPE_CHOICES[0];
    this.lineTypeSelect.value = choice.value;
  }

  private setLineWidthValue(value: string | number | undefined | null): void {
    const text = String(value ?? '');
    const choice = LINE_WIDTH_CHOICES.find(item => item.value === text) ?? LINE_WIDTH_CHOICES[0];
    this.lineWidthSelect.value = choice.value;
  }

  private setSeparatorColor(color: string | undefined | null): void {
    this.lineColorInput.value = normalizeColor(color);
    this.updateColorPreview();
  }

  private updateColorPreview(): void {
    const color = normalizeColor(this.lineColorInput.value);
    const chip = document.createElement('span');
    chip.style.cssText = [
      `display:block;width:44px;height:16px;background:${color};`,
      'border:1px solid var(--color-border-dark);',
    ].join('');
    this.replaceButtonPreview(this.lineColorButton, chip);
    this.lineColorMenu?.querySelectorAll<HTMLButtonElement>('button[data-color]').forEach((button) => {
      const active = normalizeColor(button.dataset.color) === color;
      button.style.outline = active ? '2px solid var(--color-primary)' : 'none';
      button.style.outlineOffset = active ? '1px' : '0';
    });
  }

  private replaceButtonPreview(button: HTMLButtonElement, preview: HTMLElement): void {
    const arrow = button.querySelector('[data-dropdown-arrow="1"]');
    button.querySelectorAll('[data-preview="1"]').forEach(el => el.remove());
    preview.dataset.preview = '1';
    if (arrow) {
      button.insertBefore(preview, arrow);
    } else {
      button.appendChild(preview);
    }
  }

  private toggleMenu(menu: HTMLDivElement): void {
    const nextDisplay = menu.style.display === 'none' ? (menu.dataset.display || 'block') : 'none';
    this.closePopupMenus();
    menu.style.display = nextDisplay;
  }

  private attachPopupDismissHandlers(): void {
    if (this.popupDismissPointerHandler || this.popupDismissFocusHandler) return;
    this.popupDismissPointerHandler = (event: PointerEvent) => {
      if (this.isPopupTarget(event.target)) return;
      this.closePopupMenus();
    };
    this.popupDismissFocusHandler = (event: FocusEvent) => {
      if (this.isPopupTarget(event.target)) return;
      this.closePopupMenus();
    };
    document.addEventListener('pointerdown', this.popupDismissPointerHandler, true);
    document.addEventListener('focusin', this.popupDismissFocusHandler, true);
  }

  private detachPopupDismissHandlers(): void {
    if (this.popupDismissPointerHandler) {
      document.removeEventListener('pointerdown', this.popupDismissPointerHandler, true);
      this.popupDismissPointerHandler = null;
    }
    if (this.popupDismissFocusHandler) {
      document.removeEventListener('focusin', this.popupDismissFocusHandler, true);
      this.popupDismissFocusHandler = null;
    }
  }

  private isPopupTarget(target: EventTarget | null): boolean {
    return target instanceof HTMLElement
      && Boolean(target.closest('[data-endnote-popup-root="1"]'));
  }

  private closePopupMenus(): void {
    for (const menu of [this.lineTypeMenu, this.lineWidthMenu, this.lineColorMenu]) {
      if (menu) menu.style.display = 'none';
    }
  }

  private withUnit(input: HTMLElement, unit: string): HTMLElement {
    const wrap = document.createElement('span');
    wrap.style.cssText = 'display:inline-flex;align-items:center;gap:4px;';
    const unitEl = document.createElement('span');
    unitEl.textContent = unit;
    unitEl.style.color = 'var(--color-text-secondary)';
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
