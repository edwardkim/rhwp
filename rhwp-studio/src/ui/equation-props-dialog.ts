import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { EquationProperties, NoteControlRef } from '@/core/types';
import { EquationEditorDialog } from './equation-editor-dialog';
import { enableDialogDrag } from './dialog-drag';

type TabName = '기본' | '여백/캡션' | '수식';

const TAB_NAMES: TabName[] = ['기본', '여백/캡션', '수식'];

function hwpunitToMm(hu: number): number {
  return hu * 25.4 / 7200;
}

function formatMm(hu: number | undefined): string {
  return typeof hu === 'number' && Number.isFinite(hu) ? hwpunitToMm(hu).toFixed(2) : '';
}

function colorRefToHex(colorRef: number): string {
  const r = colorRef & 0xFF;
  const g = (colorRef >> 8) & 0xFF;
  const b = (colorRef >> 16) & 0xFF;
  return '#' + [r, g, b].map(c => c.toString(16).padStart(2, '0')).join('');
}

function hexToColorRef(hex: string): number {
  const clean = hex.replace('#', '');
  const r = parseInt(clean.substring(0, 2), 16);
  const g = parseInt(clean.substring(2, 4), 16);
  const b = parseInt(clean.substring(4, 6), 16);
  return (b << 16) | (g << 8) | r;
}

export class EquationPropertiesDialog {
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private tabGroup!: HTMLDivElement;
  private body!: HTMLDivElement;
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];
  private built = false;

  private sec = 0;
  private para = 0;
  private ci = 0;
  private cellIdx?: number;
  private cellParaIdx?: number;
  private noteRef?: NoteControlRef;
  private props: EquationProperties | null = null;

  private widthInput!: HTMLInputElement;
  private heightInput!: HTMLInputElement;
  private treatAsCharInput!: HTMLInputElement;
  private horzOffsetInput!: HTMLInputElement;
  private vertOffsetInput!: HTMLInputElement;
  private outerMarginLeftInput!: HTMLInputElement;
  private outerMarginRightInput!: HTMLInputElement;
  private outerMarginTopInput!: HTMLInputElement;
  private outerMarginBottomInput!: HTMLInputElement;
  private captionPositionSelect!: HTMLSelectElement;
  private captionWidthInput!: HTMLInputElement;
  private captionSpacingInput!: HTMLInputElement;
  private fontSizeInput!: HTMLInputElement;
  private colorInput!: HTMLInputElement;
  private baselineInput!: HTMLInputElement;
  private fontNameInput!: HTMLInputElement;
  private scriptArea!: HTMLTextAreaElement;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
  ) {}

  open(sec: number, para: number, ci: number, cellIdx?: number, cellParaIdx?: number, noteRef?: NoteControlRef): void {
    this.build();
    this.sec = sec;
    this.para = para;
    this.ci = ci;
    this.cellIdx = cellIdx;
    this.cellParaIdx = cellParaIdx;
    this.noteRef = noteRef;

    try {
      this.props = noteRef
        ? this.wasm.getNoteEquationProperties(noteRef)
        : this.wasm.getEquationProperties(sec, para, ci, cellIdx, cellParaIdx);
    } catch (err) {
      console.warn('[EquationProperties] 수식 속성 가져오기 실패:', err);
      return;
    }

    this.populate();
    document.body.appendChild(this.overlay);
    setTimeout(() => this.dialog.focus(), 20);
  }

  hide(): void {
    this.overlay?.remove();
  }

  private build(): void {
    if (this.built) return;
    this.built = true;

    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap eq-props-dialog';
    this.dialog.tabIndex = -1;

    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '수식 속성';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);

    const mainRow = document.createElement('div');
    mainRow.className = 'cs-main-row';

    const leftCol = document.createElement('div');
    leftCol.className = 'cs-left-col';

    this.tabGroup = document.createElement('div');
    this.tabGroup.className = 'dialog-tabs';
    leftCol.appendChild(this.tabGroup);

    this.body = document.createElement('div');
    this.body.className = 'dialog-body';
    leftCol.appendChild(this.body);

    const rightCol = document.createElement('div');
    rightCol.className = 'cs-right-col';

    const okBtn = document.createElement('button');
    okBtn.className = 'dialog-btn dialog-btn-primary';
    okBtn.textContent = '설정(D)';
    okBtn.addEventListener('click', () => this.handleOk());

    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '취소';
    cancelBtn.addEventListener('click', () => this.hide());

    const editBtn = document.createElement('button');
    editBtn.className = 'dialog-btn';
    editBtn.textContent = '편집(E)';
    editBtn.addEventListener('click', () => this.openEditor());

    rightCol.append(okBtn, cancelBtn, editBtn);
    mainRow.append(leftCol, rightCol);
    this.dialog.appendChild(mainRow);
    this.overlay.appendChild(this.dialog);

    this.rebuildTabs();

    this.overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        this.hide();
      }
    });

    enableDialogDrag(this.dialog, titleBar);
  }

  private rebuildTabs(): void {
    this.tabGroup.replaceChildren();
    this.body.replaceChildren();
    this.tabs = [];
    this.panels = [];

    TAB_NAMES.forEach((name, idx) => {
      const btn = document.createElement('button');
      btn.className = 'dialog-tab';
      btn.textContent = name;
      btn.addEventListener('click', () => this.switchTab(idx));
      this.tabGroup.appendChild(btn);
      this.tabs.push(btn);

      const panel = name === '기본'
        ? this.buildBasicPanel()
        : name === '여백/캡션'
          ? this.buildMarginCaptionPanel()
          : this.buildEquationPanel();
      this.body.appendChild(panel);
      this.panels.push(panel);
    });
    this.switchTab(0);
  }

  private switchTab(idx: number): void {
    this.tabs.forEach((tab, i) => tab.classList.toggle('active', i === idx));
    this.panels.forEach((panel, i) => panel.classList.toggle('active', i === idx));
  }

  private buildBasicPanel(): HTMLDivElement {
    const panel = this.panel();

    const sizeFs = this.fieldset('크기');
    this.widthInput = this.textInput('', true);
    this.heightInput = this.textInput('', true);
    sizeFs.appendChild(this.row('너비', this.select(['고정값'], true), this.widthInput, this.unit('mm')));
    sizeFs.appendChild(this.row('높이', this.select(['고정값'], true), this.heightInput, this.unit('mm'), this.checkbox('크기 고정', true, true)));
    panel.appendChild(sizeFs);

    const posFs = this.fieldset('위치');
    const treatAsChar = this.checkboxWithInput('글자처럼 취급', true, true);
    this.treatAsCharInput = treatAsChar.input;
    posFs.appendChild(this.row('', treatAsChar.label));
    posFs.appendChild(this.row('본문과의 배치', this.wrapButton(), this.wrapButton(), this.wrapButton(), this.wrapButton(), this.label('본문 위치'), this.select(['양쪽'], true)));
    this.horzOffsetInput = this.textInput('0.00', true);
    this.vertOffsetInput = this.textInput('0.00', true);
    posFs.appendChild(this.row('가로', this.select(['문단'], true), this.unit('의'), this.select(['왼쪽'], true), this.label('기준'), this.horzOffsetInput, this.unit('mm')));
    posFs.appendChild(this.row('세로', this.select(['문단'], true), this.unit('의'), this.select(['위'], true), this.label('기준'), this.vertOffsetInput, this.unit('mm')));
    posFs.appendChild(this.row('', this.checkbox('쪽 영역 안으로 제한', true, true)));
    posFs.appendChild(this.row('', this.checkbox('서로 겹침 허용', false, true)));
    posFs.appendChild(this.row('', this.checkbox('개체와 조판 부호를 항상 같은 쪽에 놓기', false, true)));
    panel.appendChild(posFs);

    const bottomGrid = document.createElement('div');
    bottomGrid.className = 'eq-props-two-col';
    const rotateFs = this.fieldset('개체 회전');
    rotateFs.appendChild(this.row('회전각', this.textInput('', true)));
    const skewFs = this.fieldset('기울이기');
    skewFs.appendChild(this.row('가로', this.textInput('', true)));
    skewFs.appendChild(this.row('세로', this.textInput('', true)));
    bottomGrid.append(rotateFs, skewFs);
    panel.appendChild(bottomGrid);

    const etcFs = this.fieldset('기타');
    etcFs.appendChild(this.row('번호 종류', this.select(['수식'], true)));
    etcFs.appendChild(this.row('', this.checkbox('개체 보호하기', false, true)));
    panel.appendChild(etcFs);

    return panel;
  }

  private buildMarginCaptionPanel(): HTMLDivElement {
    const panel = this.panel();

    const marginFs = this.fieldset('바깥 여백');
    this.outerMarginLeftInput = this.textInput('0.00', true);
    this.outerMarginRightInput = this.textInput('0.00', true);
    this.outerMarginTopInput = this.textInput('0.00', true);
    this.outerMarginBottomInput = this.textInput('0.00', true);
    marginFs.appendChild(this.row('왼쪽', this.outerMarginLeftInput, this.unit('mm'), this.label('오른쪽'), this.outerMarginRightInput, this.unit('mm')));
    marginFs.appendChild(this.row('위쪽', this.outerMarginTopInput, this.unit('mm'), this.label('아래쪽'), this.outerMarginBottomInput, this.unit('mm')));
    panel.appendChild(marginFs);

    const captionFs = this.fieldset('캡션');
    this.captionPositionSelect = this.select(['없음', '위', '아래', '왼쪽', '오른쪽'], true);
    this.captionWidthInput = this.textInput('', true);
    this.captionSpacingInput = this.textInput('', true);
    captionFs.appendChild(this.row('위치', this.captionPositionSelect));
    captionFs.appendChild(this.row('폭', this.captionWidthInput, this.unit('mm'), this.label('간격'), this.captionSpacingInput, this.unit('mm')));
    panel.appendChild(captionFs);

    return panel;
  }

  private buildEquationPanel(): HTMLDivElement {
    const panel = this.panel();

    const styleFs = this.fieldset('수식');
    this.fontNameInput = this.textInput('', false);
    styleFs.appendChild(this.row('글꼴', this.fontNameInput));

    this.fontSizeInput = document.createElement('input');
    this.fontSizeInput.type = 'number';
    this.fontSizeInput.className = 'dialog-input';
    this.fontSizeInput.min = '1';
    this.fontSizeInput.max = '127';
    this.fontSizeInput.step = '1';
    styleFs.appendChild(this.row('크기', this.fontSizeInput, this.unit('pt')));

    this.colorInput = document.createElement('input');
    this.colorInput.type = 'color';
    this.colorInput.className = 'eq-props-color';
    styleFs.appendChild(this.row('색', this.colorInput));

    this.baselineInput = document.createElement('input');
    this.baselineInput.type = 'number';
    this.baselineInput.className = 'dialog-input';
    this.baselineInput.step = '1';
    styleFs.appendChild(this.row('기준선', this.baselineInput));
    panel.appendChild(styleFs);

    const scriptFs = this.fieldset('수식 내용');
    this.scriptArea = document.createElement('textarea');
    this.scriptArea.className = 'eq-props-script';
    this.scriptArea.rows = 6;
    this.scriptArea.readOnly = true;
    scriptFs.appendChild(this.scriptArea);
    panel.appendChild(scriptFs);

    return panel;
  }

  private populate(): void {
    if (!this.props) return;
    this.widthInput.value = formatMm(this.props.width);
    this.heightInput.value = formatMm(this.props.height);
    this.treatAsCharInput.checked = this.props.treatAsChar ?? true;
    this.horzOffsetInput.value = formatMm(this.props.horzOffset ?? 0);
    this.vertOffsetInput.value = formatMm(this.props.vertOffset ?? 0);
    this.outerMarginLeftInput.value = formatMm(this.props.outerMarginLeft ?? 0);
    this.outerMarginRightInput.value = formatMm(this.props.outerMarginRight ?? 0);
    this.outerMarginTopInput.value = formatMm(this.props.outerMarginTop ?? 0);
    this.outerMarginBottomInput.value = formatMm(this.props.outerMarginBottom ?? 0);
    this.captionPositionSelect.value = this.captionPositionLabel();
    this.captionWidthInput.value = this.props.hasCaption ? formatMm(this.props.captionWidth ?? 0) : '';
    this.captionSpacingInput.value = this.props.hasCaption ? formatMm(this.props.captionSpacing ?? 0) : '';
    this.fontNameInput.value = this.props.fontName || '';
    this.fontSizeInput.value = String(Math.round(this.props.fontSize / 100));
    this.colorInput.value = colorRefToHex(this.props.color);
    this.baselineInput.value = String(this.props.baseline ?? 0);
    this.scriptArea.value = this.props.script || '';
    this.switchTab(0);
  }

  private handleOk(): void {
    if (!this.props) return;

    const fontSize = (parseInt(this.fontSizeInput.value, 10) || 10) * 100;
    const color = hexToColorRef(this.colorInput.value);
    const baseline = parseInt(this.baselineInput.value, 10) || 0;
    const fontName = this.fontNameInput.value.trim();

    const updated: Record<string, unknown> = {};
    if (fontSize !== this.props.fontSize) updated.fontSize = fontSize;
    if (color !== this.props.color) updated.color = color;
    if (baseline !== this.props.baseline) updated.baseline = baseline;
    if (fontName && fontName !== this.props.fontName) updated.fontName = fontName;

    if (Object.keys(updated).length > 0) {
      try {
        if (this.noteRef) {
          this.wasm.setNoteEquationProperties(this.noteRef, updated);
        } else {
          this.wasm.setEquationProperties(this.sec, this.para, this.ci, this.cellIdx, this.cellParaIdx, updated);
        }
        this.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[EquationProperties] 수식 속성 설정 실패:', err);
      }
    }
    this.hide();
  }

  private openEditor(): void {
    this.hide();
    const editor = new EquationEditorDialog(this.wasm, this.eventBus);
    editor.open(this.sec, this.para, this.ci, this.cellIdx, this.cellParaIdx, this.noteRef);
  }

  private captionPositionLabel(): string {
    if (!this.props?.hasCaption) return '없음';
    switch (this.props.captionDirection) {
      case 'Top':
        return '위';
      case 'Bottom':
        return '아래';
      case 'Left':
        return '왼쪽';
      case 'Right':
        return '오른쪽';
      default:
        return '없음';
    }
  }

  private panel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';
    return panel;
  }

  private fieldset(title: string): HTMLFieldSetElement {
    const fs = document.createElement('fieldset');
    fs.className = 'cs-fieldset';
    const legend = document.createElement('legend');
    legend.textContent = title;
    fs.appendChild(legend);
    return fs;
  }

  private row(labelText: string, ...children: HTMLElement[]): HTMLDivElement {
    const row = document.createElement('div');
    row.className = 'dialog-row eq-props-row';
    if (labelText) row.appendChild(this.label(labelText));
    row.append(...children);
    return row;
  }

  private label(text: string): HTMLSpanElement {
    const label = document.createElement('span');
    label.className = 'dialog-label';
    label.textContent = text;
    return label;
  }

  private unit(text: string): HTMLSpanElement {
    const unit = document.createElement('span');
    unit.className = 'dialog-unit';
    unit.textContent = text;
    return unit;
  }

  private textInput(value: string, disabled: boolean): HTMLInputElement {
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'dialog-input';
    input.value = value;
    input.disabled = disabled;
    return input;
  }

  private select(options: string[], disabled: boolean): HTMLSelectElement {
    const select = document.createElement('select');
    select.className = 'dialog-select';
    select.disabled = disabled;
    for (const optionText of options) {
      const option = document.createElement('option');
      option.textContent = optionText;
      select.appendChild(option);
    }
    return select;
  }

  private checkbox(text: string, checked: boolean, disabled: boolean): HTMLLabelElement {
    return this.checkboxWithInput(text, checked, disabled).label;
  }

  private checkboxWithInput(text: string, checked: boolean, disabled: boolean): { label: HTMLLabelElement; input: HTMLInputElement } {
    const label = document.createElement('label');
    label.className = 'dialog-checkbox';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.checked = checked;
    input.disabled = disabled;
    label.append(input, document.createTextNode(text));
    return { label, input };
  }

  private wrapButton(): HTMLButtonElement {
    const button = document.createElement('button');
    button.className = 'pp-wrap-btn';
    button.type = 'button';
    button.textContent = '▤';
    button.disabled = true;
    return button;
  }

}
