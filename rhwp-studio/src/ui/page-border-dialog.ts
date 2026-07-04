import { ModalDialog } from './dialog';
import type { EventBus } from '@/core/event-bus';
import type { PageBorderFillSettings, BorderLineProps } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';

const HWPUNIT_PER_MM = 7200 / 25.4;

function hwpToMm(value: number): number {
  return Math.round(value / HWPUNIT_PER_MM * 100) / 100;
}

function mmToHwp(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.round(Math.min(25, Math.max(0, value)) * HWPUNIT_PER_MM);
}

type Side = 'Left' | 'Right' | 'Top' | 'Bottom';

const ALL_SIDES: Side[] = ['Left', 'Right', 'Top', 'Bottom'];
const DOC_PAPER_COLOR = 'var(--doc-paper)';
const DOC_PREVIEW_GUIDE_STROKE = '#d0d0d0';

interface TabDef {
  label: string;
  builder: () => HTMLElement;
}

export class PageBorderDialog extends ModalDialog {
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];
  private settings!: PageBorderFillSettings;

  private borderNoneCheck!: HTMLInputElement;
  private lineTypeSelect!: HTMLSelectElement;
  private lineWidthSelect!: HTMLSelectElement;
  private lineColorInput!: HTMLInputElement;
  private immediateCheck!: HTMLInputElement;
  private previewSvg!: SVGSVGElement;
  private basisPaper!: HTMLInputElement;
  private spacingInputs!: Record<Side, HTMLInputElement>;
  private applyAll!: HTMLInputElement;
  private applyExceptFirst!: HTMLInputElement;

  private bgNoneRadio!: HTMLInputElement;
  private bgColorRadio!: HTMLInputElement;
  private bgColorInput!: HTMLInputElement;
  private bgPatternColorInput!: HTMLInputElement;
  private bgPatternSelect!: HTMLSelectElement;
  private fillAreaPaper!: HTMLInputElement;

  private borderEdits: Record<Side, BorderLineProps> = {
    Left: { type: 0, width: 0, color: '#000000' },
    Right: { type: 0, width: 0, color: '#000000' },
    Top: { type: 0, width: 0, color: '#000000' },
    Bottom: { type: 0, width: 0, color: '#000000' },
  };

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private sectionIdx: number,
  ) {
    super('쪽 테두리/배경', 560);
  }

  show(): void {
    this.settings = this.wasm.getPageBorderFill(this.sectionIdx);
    this.borderEdits = {
      Left: { ...this.settings.borderLeft },
      Right: { ...this.settings.borderRight },
      Top: { ...this.settings.borderTop },
      Bottom: { ...this.settings.borderBottom },
    };
    super.show();
    this.populate();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    const tabBar = document.createElement('div');
    tabBar.className = 'dialog-tabs';
    const panelWrap = document.createElement('div');

    const tabs: TabDef[] = [
      { label: '테두리', builder: () => this.buildBorderTab() },
      { label: '배경', builder: () => this.buildBackgroundTab() },
    ];

    tabs.forEach((tab, idx) => {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'dialog-tab';
      button.textContent = tab.label;
      button.addEventListener('click', () => this.switchTab(idx));
      this.tabs.push(button);
      tabBar.appendChild(button);

      const panel = document.createElement('div');
      panel.className = 'dialog-tab-panel';
      panel.appendChild(tab.builder());
      this.panels.push(panel);
      panelWrap.appendChild(panel);
    });

    body.append(tabBar, panelWrap);
    this.switchTab(0);
    return body;
  }

  protected onConfirm(): void {
    const applyPage = this.radioValue('page-border-apply', 'all');
    const next: PageBorderFillSettings = {
      ...this.settings,
      basis: this.basisPaper.checked ? 'paper' : 'page',
      spacingLeft: mmToHwp(parseFloat(this.spacingInputs.Left.value)),
      spacingRight: mmToHwp(parseFloat(this.spacingInputs.Right.value)),
      spacingTop: mmToHwp(parseFloat(this.spacingInputs.Top.value)),
      spacingBottom: mmToHwp(parseFloat(this.spacingInputs.Bottom.value)),
      borderLeft: this.borderNoneCheck.checked ? noneBorder() : { ...this.borderEdits.Left },
      borderRight: this.borderNoneCheck.checked ? noneBorder() : { ...this.borderEdits.Right },
      borderTop: this.borderNoneCheck.checked ? noneBorder() : { ...this.borderEdits.Top },
      borderBottom: this.borderNoneCheck.checked ? noneBorder() : { ...this.borderEdits.Bottom },
      fillType: this.bgColorRadio.checked ? 'solid' : 'none',
      fillColor: this.bgColorInput.value,
      patternColor: this.bgPatternColorInput.value,
      patternType: parseInt(this.bgPatternSelect.value, 10),
      fillArea: this.fillAreaPaper.checked
        ? 'paper'
        : (this.radioValue('page-border-fill-area', 'page') as 'page' | 'border'),
      hideBorder: applyPage === 'exceptFirst',
      hideFill: applyPage === 'exceptFirst',
      applyPage: applyPage === 'exceptFirst' ? 'exceptFirst' : 'all',
    };

    this.wasm.setPageBorderFill(this.sectionIdx, next);
    this.eventBus.emit('document-changed');
  }

  private buildBorderTab(): HTMLElement {
    const root = this.tabContent();
    root.appendChild(this.kindGroup());

    const border = this.group('테두리');
    const borderGrid = document.createElement('div');
    borderGrid.style.cssText = 'display:grid;grid-template-columns:1fr 204px;gap:14px;align-items:start;';
    const controls = document.createElement('div');
    const lineRow = this.row();
    lineRow.append(this.label('종류'), this.buildLineTypeSelect(), this.label('굵기'), this.buildLineWidthSelect());
    const colorRow = this.row();
    colorRow.append(this.label('색'), this.buildColorInput());
    this.immediateCheck = this.checkbox('선 모양 바로 적용');
    this.immediateCheck.checked = true;
    this.borderNoneCheck = this.checkbox('테두리 사용 안 함');
    this.borderNoneCheck.addEventListener('change', () => this.handleBorderNoneChange());
    controls.append(lineRow, colorRow, this.checkboxRow(this.immediateCheck), this.checkboxRow(this.borderNoneCheck));

    const previewWrap = document.createElement('div');
    previewWrap.style.cssText = 'display:grid;grid-template-columns:28px 142px 28px;grid-template-rows:26px 112px 26px;gap:4px;align-items:center;justify-items:center;';
    this.previewSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    this.previewSvg.setAttribute('viewBox', '0 0 142 112');
    this.previewSvg.style.cssText = `width:142px;height:112px;grid-column:2;grid-row:2;background:${DOC_PAPER_COLOR};`;
    previewWrap.append(
      this.sideButton('위쪽', 'Top', '2', '1'),
      this.sideButton('왼쪽', 'Left', '1', '2'),
      this.previewSvg,
      this.sideButton('오른쪽', 'Right', '3', '2'),
      this.sideButton('아래쪽', 'Bottom', '2', '3'),
      this.sideButton('모두', 'All', '3', '3'),
    );
    borderGrid.append(controls, previewWrap);
    border.appendChild(borderGrid);
    root.appendChild(border);

    root.appendChild(this.positionGroup());
    root.appendChild(this.applyGroup());
    root.appendChild(this.dialogConfigRow());
    return root;
  }

  private buildBackgroundTab(): HTMLElement {
    const root = this.tabContent();
    root.appendChild(this.kindGroup());

    const fill = this.group('채우기');
    this.bgNoneRadio = this.radio('page-border-bg', 'none', '색 채우기 없음');
    this.bgColorRadio = this.radio('page-border-bg', 'solid', '색');
    this.bgColorInput = document.createElement('input');
    this.bgColorInput.type = 'color';
    this.bgColorInput.value = '#ffffff';
    this.bgColorInput.style.cssText = 'width:74px;height:24px;';
    this.bgPatternColorInput = document.createElement('input');
    this.bgPatternColorInput.type = 'color';
    this.bgPatternColorInput.value = '#000000';
    this.bgPatternColorInput.style.cssText = 'width:74px;height:24px;';
    this.bgPatternSelect = document.createElement('select');
    this.bgPatternSelect.className = 'dialog-select';
    [
      ['0', '무늬 없음'],
      ['1', '가로선'],
      ['2', '세로선'],
      ['3', '대각선'],
      ['4', '격자'],
    ].forEach(([value, text]) => {
      const option = document.createElement('option');
      option.value = value;
      option.textContent = text;
      this.bgPatternSelect.appendChild(option);
    });
    const colorRow = this.row();
    colorRow.append(this.bgColorRadio, this.bgColorInput, this.label('무늬 색'), this.bgPatternColorInput, this.bgPatternSelect);
    const grad = this.radio('page-border-bg', 'gradient', '그라데이션');
    grad.disabled = true;
    const picture = this.checkbox('그림');
    picture.disabled = true;
    const gradRow = this.row();
    gradRow.append(this.radioRow(grad), this.disabledSelect(['세로', '가로', '오른쪽 대각선']), this.disabledSwatch(), this.disabledSwatch());
    const pictureRow = this.row();
    pictureRow.append(this.checkboxRow(picture), this.disabledSelect(['문서에 포함']), this.disabledSelect(['크기에 맞추어']), this.label('밝기'), this.disabledInput('0'));
    fill.append(this.radioRow(this.bgNoneRadio), colorRow, gradRow, pictureRow);
    root.appendChild(fill);

    const area = this.group('채울 영역');
    this.fillAreaPaper = this.radio('page-border-fill-area', 'paper', '종이');
    area.append(
      this.radioRow(this.fillAreaPaper),
      this.radioRow(this.radio('page-border-fill-area', 'page', '쪽')),
      this.radioRow(this.radio('page-border-fill-area', 'border', '테두리')),
    );
    root.appendChild(area);

    root.appendChild(this.applyGroup());
    root.appendChild(this.dialogConfigRow(true));
    return root;
  }

  private kindGroup(): HTMLElement {
    const group = this.group('테두리/배경 종류');
    const both = this.radio('page-border-kind', 'both', '양쪽');
    const odd = this.radio('page-border-kind', 'odd', '홀수 쪽');
    const even = this.radio('page-border-kind', 'even', '짝수 쪽');
    odd.disabled = true;
    even.disabled = true;
    group.append(this.radioRow(both), this.radioRow(odd), this.radioRow(even));
    return group;
  }

  private positionGroup(): HTMLElement {
    const group = this.group('위치');
    this.basisPaper = this.radio('page-border-basis', 'paper', '종이 기준');
    const basisPage = this.radio('page-border-basis', 'page', '쪽 기준');
    const basisRow = this.row();
    basisRow.append(this.radioRow(this.basisPaper), this.radioRow(basisPage));

    this.spacingInputs = {
      Left: this.numberInput(0),
      Right: this.numberInput(0),
      Top: this.numberInput(0),
      Bottom: this.numberInput(0),
    };
    group.append(
      basisRow,
      this.mmRow('왼쪽', this.spacingInputs.Left, '위쪽', this.spacingInputs.Top),
      this.mmRow('오른쪽', this.spacingInputs.Right, '아래쪽', this.spacingInputs.Bottom),
      this.checkboxRow(this.disabledCheck('머리말 포함')),
      this.checkboxRow(this.disabledCheck('꼬리말 포함')),
    );
    return group;
  }

  private applyGroup(): HTMLElement {
    const group = this.group('적용 쪽');
    this.applyAll = this.radio('page-border-apply', 'all', '모두');
    this.applyExceptFirst = this.radio('page-border-apply', 'exceptFirst', '첫 쪽 제외');
    const firstOnly = this.radio('page-border-apply', 'firstOnly', '첫 쪽만');
    firstOnly.disabled = true;
    group.append(this.radioRow(this.applyAll), this.radioRow(this.applyExceptFirst), this.radioRow(firstOnly));
    return group;
  }

  private populate(): void {
    this.dialog
      .querySelectorAll<HTMLInputElement>('input[name="page-border-kind"][value="both"]')
      .forEach(input => { input.checked = true; });
    this.basisPaper.checked = this.settings.basis === 'paper';
    const basisPage = this.dialog.querySelector<HTMLInputElement>('input[name="page-border-basis"][value="page"]');
    if (basisPage) basisPage.checked = this.settings.basis === 'page';

    this.spacingInputs.Left.value = String(hwpToMm(this.settings.spacingLeft));
    this.spacingInputs.Right.value = String(hwpToMm(this.settings.spacingRight));
    this.spacingInputs.Top.value = String(hwpToMm(this.settings.spacingTop));
    this.spacingInputs.Bottom.value = String(hwpToMm(this.settings.spacingBottom));

    const hasBorder = Object.values(this.borderEdits).some(border => border.type !== 0);
    this.borderNoneCheck.checked = !hasBorder;
    const firstBorder = Object.values(this.borderEdits).find(border => border.type !== 0) ?? this.settings.borderTop;
    this.lineTypeSelect.value = String(firstBorder.type || 1);
    this.lineWidthSelect.value = String(firstBorder.width || 0);
    this.lineColorInput.value = firstBorder.color || '#000000';

    this.dialog
      .querySelectorAll<HTMLInputElement>('input[name="page-border-apply"][value="exceptFirst"]')
      .forEach(input => { input.checked = this.settings.hideBorder || this.settings.hideFill; });
    this.dialog
      .querySelectorAll<HTMLInputElement>('input[name="page-border-apply"][value="all"]')
      .forEach(input => { input.checked = !(this.settings.hideBorder || this.settings.hideFill); });

    this.bgColorRadio.checked = this.settings.fillType === 'solid';
    this.bgNoneRadio.checked = this.settings.fillType !== 'solid';
    this.bgColorInput.value = this.settings.fillColor || '#ffffff';
    this.bgPatternColorInput.value = this.settings.patternColor || '#000000';
    this.bgPatternSelect.value = String(this.settings.patternType || 0);
    const fillArea = this.dialog.querySelector<HTMLInputElement>(
      `input[name="page-border-fill-area"][value="${this.settings.fillArea || 'paper'}"]`,
    );
    if (fillArea) fillArea.checked = true;

    this.updateBorderPreview();
  }

  private switchTab(idx: number): void {
    this.tabs.forEach((tab, i) => tab.classList.toggle('active', i === idx));
    this.panels.forEach((panel, i) => panel.classList.toggle('active', i === idx));
  }

  private tabContent(): HTMLDivElement {
    const root = document.createElement('div');
    root.style.cssText = 'display:grid;gap:10px;padding:12px 14px;';
    return root;
  }

  private group(title: string): HTMLFieldSetElement {
    const fieldset = document.createElement('fieldset');
    fieldset.style.cssText = 'border:1px solid var(--color-border-lighter);padding:10px 12px;margin:0;';
    const legend = document.createElement('legend');
    legend.textContent = title;
    legend.style.cssText = 'font-size:12px;color:var(--color-primary-dark);padding:0 4px;';
    fieldset.appendChild(legend);
    return fieldset;
  }

  private row(): HTMLDivElement {
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin:4px 0;';
    return row;
  }

  private label(text: string): HTMLSpanElement {
    const label = document.createElement('span');
    label.textContent = text;
    label.style.cssText = 'font-size:13px;min-width:42px;color:var(--color-text);';
    return label;
  }

  private checkbox(text: string): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.dataset.label = text;
    return input;
  }

  private checkboxRow(input: HTMLInputElement): HTMLLabelElement {
    const label = document.createElement('label');
    label.style.cssText = 'display:inline-flex;align-items:center;gap:6px;font-size:13px;margin-right:12px;color:var(--color-text);';
    label.append(input, document.createTextNode(input.dataset.label || ''));
    return label;
  }

  private radio(name: string, value: string, text: string): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'radio';
    input.name = name;
    input.value = value;
    input.dataset.label = text;
    return input;
  }

  private radioRow(input: HTMLInputElement): HTMLLabelElement {
    const label = document.createElement('label');
    label.style.cssText = 'display:inline-flex;align-items:center;gap:6px;font-size:13px;margin-right:14px;color:var(--color-text);';
    label.append(input, document.createTextNode(input.dataset.label || ''));
    return label;
  }

  private buildLineTypeSelect(): HTMLSelectElement {
    this.lineTypeSelect = document.createElement('select');
    this.lineTypeSelect.className = 'dialog-select';
    [
      ['0', '없음'],
      ['1', '실선'],
      ['2', '파선'],
      ['3', '점선'],
      ['4', '일점 쇄선'],
      ['5', '이점 쇄선'],
      ['6', '긴 파선'],
      ['8', '이중선'],
      ['12', '물결선'],
    ].forEach(([value, text]) => {
      const option = document.createElement('option');
      option.value = value;
      option.textContent = text;
      this.lineTypeSelect.appendChild(option);
    });
    this.lineTypeSelect.addEventListener('change', () => {
      if (this.immediateCheck.checked) this.applyToActiveSides();
    });
    return this.lineTypeSelect;
  }

  private buildLineWidthSelect(): HTMLSelectElement {
    this.lineWidthSelect = document.createElement('select');
    this.lineWidthSelect.className = 'dialog-select';
    ['0.1mm', '0.12mm', '0.15mm', '0.2mm', '0.25mm', '0.3mm', '0.4mm', '0.5mm', '0.6mm'].forEach((text, idx) => {
      const option = document.createElement('option');
      option.value = String(idx);
      option.textContent = text;
      this.lineWidthSelect.appendChild(option);
    });
    this.lineWidthSelect.addEventListener('change', () => {
      if (this.immediateCheck.checked) this.applyToActiveSides();
    });
    return this.lineWidthSelect;
  }

  private buildColorInput(): HTMLInputElement {
    this.lineColorInput = document.createElement('input');
    this.lineColorInput.type = 'color';
    this.lineColorInput.value = '#000000';
    this.lineColorInput.style.colorScheme = 'inherit';
    this.lineColorInput.addEventListener('change', () => {
      if (this.immediateCheck.checked) this.applyToActiveSides();
    });
    return this.lineColorInput;
  }

  private numberInput(value: number): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.min = '0';
    input.max = '25';
    input.step = '0.1';
    input.value = String(value);
    input.className = 'dialog-input';
    input.style.cssText = 'width:76px;padding:3px 5px;';
    return input;
  }

  private mmRow(labelA: string, inputA: HTMLInputElement, labelB: string, inputB: HTMLInputElement): HTMLDivElement {
    const row = this.row();
    row.append(this.label(labelA), inputA, document.createTextNode('mm'), this.label(labelB), inputB, document.createTextNode('mm'));
    return row;
  }

  private sideButton(text: string, side: Side | 'All', column: string, row: string): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.title = text;
    button.textContent = side === 'All' ? '□' : '▦';
    button.className = 'page-border-side-btn';
    button.style.cssText = [
      `grid-column:${column}`,
      `grid-row:${row}`,
    ].join(';');
    button.addEventListener('click', () => {
      if (side === 'All') {
        this.toggleAllSides();
      } else {
        this.toggleSide(side);
      }
    });
    return button;
  }

  private disabledCheck(text: string): HTMLInputElement {
    const input = this.checkbox(text);
    input.disabled = true;
    return input;
  }

  private disabledSelect(options: string[]): HTMLSelectElement {
    const select = document.createElement('select');
    select.className = 'dialog-select';
    select.disabled = true;
    select.style.minWidth = '92px';
    options.forEach(text => {
      const option = document.createElement('option');
      option.textContent = text;
      select.appendChild(option);
    });
    return select;
  }

  private disabledInput(value: string): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.value = value;
    input.disabled = true;
    input.className = 'dialog-input';
    input.style.cssText = 'width:74px;padding:3px 5px;';
    return input;
  }

  private disabledSwatch(): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'color';
    input.value = '#999999';
    input.disabled = true;
    input.style.cssText = 'width:74px;height:24px;color-scheme:inherit;';
    return input;
  }

  private dialogConfigRow(includeFillArea = false): HTMLDivElement {
    const row = this.row();
    row.style.marginTop = '4px';
    row.append(this.label('적용 범위'), this.disabledSelect(['문서 전체']));
    if (includeFillArea) {
      row.append(this.label('채울 영역'), this.disabledSelect(['종이', '쪽', '테두리']));
    }
    const spacer = document.createElement('span');
    spacer.style.flex = '1';
    row.append(spacer, this.label('대화 상자 설정'), this.disabledSelect(['사용자 지정']), this.smallPlainButton('구성...'));
    return row;
  }

  private smallPlainButton(text: string): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = text;
    button.disabled = true;
    button.className = 'dialog-btn';
    button.style.padding = '3px 8px';
    return button;
  }

  private applyToSides(sides: Side[]): void {
    const next = this.currentBorder();
    sides.forEach(side => {
      this.borderEdits[side] = { ...next };
    });
    this.syncBorderNoneCheck();
    this.updateBorderPreview();
  }

  private applyToActiveSides(): void {
    const activeSides = ALL_SIDES.filter(side => this.isSideActive(side));
    if (activeSides.length === 0) return;
    this.applyToSides(activeSides);
  }

  private handleBorderNoneChange(): void {
    if (this.borderNoneCheck.checked) {
      this.clearAllSides();
    }
    this.updateBorderPreview();
  }

  private toggleSide(side: Side): void {
    if (this.isSideActive(side)) {
      this.borderEdits[side] = noneBorder();
    } else {
      this.borderEdits[side] = { ...this.currentBorder() };
    }
    this.syncBorderNoneCheck();
    this.updateBorderPreview();
  }

  private toggleAllSides(): void {
    const next = ALL_SIDES.every(side => this.isSideActive(side))
      ? noneBorder()
      : this.currentBorder();
    ALL_SIDES.forEach(side => {
      this.borderEdits[side] = { ...next };
    });
    this.syncBorderNoneCheck();
    this.updateBorderPreview();
  }

  private clearAllSides(): void {
    ALL_SIDES.forEach(side => {
      this.borderEdits[side] = noneBorder();
    });
  }

  private isSideActive(side: Side): boolean {
    return this.borderEdits[side].type !== 0;
  }

  private syncBorderNoneCheck(): void {
    this.borderNoneCheck.checked = !ALL_SIDES.some(side => this.isSideActive(side));
  }

  private currentBorder(): BorderLineProps {
    return {
      type: parseInt(this.lineTypeSelect.value, 10),
      width: parseInt(this.lineWidthSelect.value, 10),
      color: this.lineColorInput.value,
    };
  }

  private updateBorderPreview(): void {
    while (this.previewSvg.firstChild) this.previewSvg.removeChild(this.previewSvg.firstChild);
    const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    bg.setAttribute('x', '12');
    bg.setAttribute('y', '8');
    bg.setAttribute('width', '118');
    bg.setAttribute('height', '96');
    bg.style.fill = DOC_PAPER_COLOR;
    bg.style.stroke = DOC_PREVIEW_GUIDE_STROKE;
    this.previewSvg.appendChild(bg);
    if (this.borderNoneCheck.checked) return;

    const lines: [Side, number, number, number, number][] = [
      ['Top', 12, 8, 130, 8],
      ['Bottom', 12, 104, 130, 104],
      ['Left', 12, 8, 12, 104],
      ['Right', 130, 8, 130, 104],
    ];
    for (const [side, x1, y1, x2, y2] of lines) {
      const border = this.borderEdits[side];
      if (border.type === 0) continue;
      const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
      line.setAttribute('x1', String(x1));
      line.setAttribute('y1', String(y1));
      line.setAttribute('x2', String(x2));
      line.setAttribute('y2', String(y2));
      line.setAttribute('stroke', border.color);
      line.setAttribute('stroke-width', String(Math.max(1, border.width + 1)));
      const dash = dashArray(border.type);
      if (dash) line.setAttribute('stroke-dasharray', dash);
      this.previewSvg.appendChild(line);
    }
  }

  private radioValue(name: string, fallback: string): string {
    const selected = this.dialog.querySelector<HTMLInputElement>(`input[name="${name}"]:checked`);
    return selected?.value || fallback;
  }
}

function noneBorder(): BorderLineProps {
  return { type: 0, width: 0, color: '#000000' };
}

function dashArray(type: number): string {
  switch (type) {
    case 2: return '8,4';
    case 3: return '2,4';
    case 4: return '10,4,2,4';
    case 5: return '10,4,2,4,2,4';
    case 6: return '14,4';
    default: return '';
  }
}
