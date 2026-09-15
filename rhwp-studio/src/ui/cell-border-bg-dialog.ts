import { ModalDialog } from './dialog';
import { applyCommandThroughRouter } from './dialog-apply';
import { SetCellBorderFillCommand } from '@/engine/command';
import type { CellBorderFillTarget } from '@/engine/command';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { CellBbox, CellProperties } from '@/core/types';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';

import { t as i18nText } from '../i18n/index.ts';
import { t } from '@/i18n/index.ts';
const HWPUNIT_PER_MM = 7200 / 25.4;

function hwp16ToMm(hu: number): number {
  return Math.round(hu * 25.4 / 7200 * 10) / 10;
}

function mmToHwp16(mm: number): number {
  return Math.round(mm * HWPUNIT_PER_MM);
}

const DOC_PAPER_COLOR = 'var(--doc-paper)';
const PREVIEW_GUIDE_STROKE = 'var(--ui-border-light)';
const LINE_SAMPLE_STROKE = 'currentColor';
const DIAGONAL_LINE_TYPE_OPTIONS: string[][] = [
  ['0', t('option.diagonalLineTypeOptions.0')],
  ['1', t('option.diagonalLineTypeOptions.1')],
  ['2', t('option.diagonalLineTypeOptions.2')],
  ['3', t('option.diagonalLineTypeOptions.3')],
  ['4', t('option.diagonalLineTypeOptions.4')],
  ['5', t('option.diagonalLineTypeOptions.5')],
  ['6', t('option.diagonalLineTypeOptions.6')],
  ['7', t('option.diagonalLineTypeOptions.7')],
  ['8', t('option.diagonalLineTypeOptions.8')],
  ['9', t('option.diagonalLineTypeOptions.9')],
  ['10', t('option.diagonalLineTypeOptions.10')],
  ['11', t('option.diagonalLineTypeOptions.11')],
  ['12', t('option.diagonalLineTypeOptions.12')],
  ['13', t('option.diagonalLineTypeOptions.13')],
  ['14', t('option.diagonalLineTypeOptions.14')],
  ['15', t('option.diagonalLineTypeOptions.15')],
  ['16', t('option.diagonalLineTypeOptions.16')],
  ['17', t('option.diagonalLineTypeOptions.17')],
];
const DIAGONAL_WIDTH_OPTIONS: string[][] = [
  ['0', '0.1mm'],
  ['1', '0.12mm'],
  ['2', '0.15mm'],
  ['3', '0.2mm'],
  ['4', '0.25mm'],
  ['5', '0.3mm'],
  ['6', '0.4mm'],
  ['7', '0.5mm'],
  ['8', '0.6mm'],
  ['9', '0.7mm'],
  ['10', '1.0mm'],
  ['11', '1.5mm'],
  ['12', '2.0mm'],
  ['13', '3.0mm'],
  ['14', '4.0mm'],
  ['15', '5.0mm'],
];
const DIAGONAL_WIDTH_MM = [0.1, 0.12, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];

function diagonalPreviewWidthPx(widthIndex: number): number {
  const mm = DIAGONAL_WIDTH_MM[widthIndex] ?? DIAGONAL_WIDTH_MM[0];
  return Math.min(8, Math.max(0.8, mm * 2.6));
}

/** 탭 정의 */
interface TabDef {
  id: string;
  label: string;
  builder: () => HTMLElement;
}

type CellRange = { startRow: number; startCol: number; endRow: number; endCol: number };

/**
 * 셀 테두리/배경 대화상자 (3탭: 테두리/배경/대각선)
 *
 * 셀 선택 모드에서 우클릭 컨텍스트 메뉴를 통해 접근.
 * applyMode: 'each' = 각 셀마다 적용, 'asOne' = 하나의 셀처럼 적용
 */
export class CellBorderBgDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private tableCtx: { sec: number; ppi: number; ci: number };
  private cellIdx: number;
  private applyMode: 'each' | 'asOne';
  private selectionRange: CellRange | null;
  /** undo 기록 라우팅용 (없으면 wasm 직접 호출 fallback). */
  private services: CommandServices | undefined;

  // 탭 UI
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];

  // 테두리 탭 필드
  private borderLineTypeGrid!: HTMLDivElement;
  private borderSelectedLineType = 1;
  private borderWidthSelect!: HTMLSelectElement;
  private borderColorInput!: HTMLInputElement;
  private borderPreviewSvg!: SVGSVGElement;
  private borderEdits: { type: number; width: number; color: string }[] = [
    { type: 1, width: 0, color: '#000000' },
    { type: 1, width: 0, color: '#000000' },
    { type: 1, width: 0, color: '#000000' },
    { type: 1, width: 0, color: '#000000' },
  ];
  private borderApplyImmediateCheck!: HTMLInputElement;
  private borderScopeRadios!: HTMLInputElement[];

  // 배경 탭 필드
  private bgNoneRadio!: HTMLInputElement;
  private bgColorRadio!: HTMLInputElement;
  private bgColorPicker!: HTMLInputElement;
  private bgPatternColorPicker!: HTMLInputElement;
  private bgPatternTypeSelect!: HTMLSelectElement;
  private bgPreviewBox!: HTMLDivElement;
  private bgScopeRadios!: HTMLInputElement[];

  // 대각선 탭 필드
  private diagLineTypeSelect!: HTMLSelectElement;
  private diagWidthSelect!: HTMLSelectElement;
  private diagColorInput!: HTMLInputElement;
  private diagScopeRadios!: HTMLInputElement[];
  private diagPreviewSvg!: SVGSVGElement;
  private diagButtons: HTMLButtonElement[] = [];
  private diagSlashBits = 0;
  private diagBackSlashBits = 0;
  private diagCenterLine = 'NONE';
  private activeTabId = 'border';

  // 셀 속성 캐시
  private cellProps!: CellProperties;

  constructor(
    wasm: WasmBridge,
    eventBus: EventBus,
    tableCtx: { sec: number; ppi: number; ci: number },
    cellIdx: number,
    applyMode: 'each' | 'asOne' = 'each',
    selectionRange: CellRange | null = null,
    services?: CommandServices,
  ) {
    super(i18nText('dialog.cellBorderBg.diagonalPreviewWidthPx.title'), 460);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.tableCtx = tableCtx;
    this.cellIdx = cellIdx;
    this.applyMode = applyMode;
    this.selectionRange = selectionRange;
    this.services = services;
  }

  show(): void {
    super.show();
    this.dialog.classList.add('tcp-border-bg-dialog');
    const { sec, ppi, ci } = this.tableCtx;
    this.cellProps = this.applyMode === 'each'
      ? this.wasm.getCellOwnProperties(sec, ppi, ci, this.cellIdx)
      : this.wasm.getCellProperties(sec, ppi, ci, this.cellIdx);
    this.populateFields();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'tcp-dialog-body';

    const tabDefs: TabDef[] = [
      { id: 'border', label: i18nText('dialog.cellBorderBg.createBody.label'), builder: () => this.buildBorderTab() },
      { id: 'background', label: i18nText('dialog.cellBorderBg.createBody.label.x807c30'), builder: () => this.buildBackgroundTab() },
      { id: 'diagonal', label: i18nText('dialog.cellBorderBg.createBody.label.x89bace'), builder: () => this.buildDiagonalTab() },
    ];

    // 탭 헤더
    const tabBar = document.createElement('div');
    tabBar.className = 'dialog-tabs';
    const panelContainer = document.createElement('div');
    panelContainer.className = 'tcp-panel-container';

    for (let i = 0; i < tabDefs.length; i++) {
      const def = tabDefs[i];
      const btn = document.createElement('button');
      btn.className = 'dialog-tab';
      btn.textContent = def.label;
      btn.type = 'button';
      btn.addEventListener('click', () => this.switchTab(i));
      this.tabs.push(btn);
      tabBar.appendChild(btn);

      const panel = document.createElement('div');
      panel.className = 'dialog-tab-panel';
      panel.appendChild(def.builder());
      this.panels.push(panel);
      panelContainer.appendChild(panel);
    }

    body.appendChild(tabBar);
    body.appendChild(panelContainer);
    this.switchTab(0);

    return body;
  }

  private switchTab(idx: number): void {
    this.activeTabId = ['border', 'background', 'diagonal'][idx] ?? 'border';
    for (let i = 0; i < this.tabs.length; i++) {
      this.tabs[i].classList.toggle('active', i === idx);
      this.panels[i].classList.toggle('active', i === idx);
    }
  }

  // ─── 테두리 탭 ────────────────────────────────

  private buildBorderTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    // 선 종류 시각적 격자
    const lineSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label'));
    this.borderLineTypeGrid = document.createElement('div');
    this.borderLineTypeGrid.className = 'tcp-line-type-grid';
    const lineTypeDefs = [
      { type: 0, label: i18nText('dialog.cellBorderBg.buildBorderTab.label') },
      { type: 1, dash: '' },
      { type: 2, dash: '6,3' },
      { type: 3, dash: '2,2' },
      { type: 4, dash: '8,3,2,3' },
      { type: 5, dash: '8,3,2,3,2,3' },
      { type: 6, dash: '12,3' },
      { type: 8, label: i18nText('dialog.cellBorderBg.buildBorderTab.label.x87ab78') },
    ];
    lineTypeDefs.forEach(def => {
      const item = document.createElement('div');
      item.className = 'tcp-line-type-item';
      if (def.type === 1) item.classList.add('active');
      if (def.type === 0) {
        const span = document.createElement('span');
        span.className = 'tcp-line-type-none';
        span.textContent = i18nText('dialog.cellBorderBg.span.text');
        item.appendChild(span);
      } else if (def.type === 8) {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('viewBox', '0 0 48 10');
        for (const y of [3, 7]) {
          const l = document.createElementNS('http://www.w3.org/2000/svg', 'line');
          l.setAttribute('x1', '0'); l.setAttribute('y1', String(y));
          l.setAttribute('x2', '48'); l.setAttribute('y2', String(y));
          l.setAttribute('stroke', LINE_SAMPLE_STROKE); l.setAttribute('stroke-width', '1');
          svg.appendChild(l);
        }
        item.appendChild(svg);
      } else {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('viewBox', '0 0 48 10');
        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', '0'); line.setAttribute('y1', '5');
        line.setAttribute('x2', '48'); line.setAttribute('y2', '5');
        line.setAttribute('stroke', LINE_SAMPLE_STROKE); line.setAttribute('stroke-width', '1.5');
        if (def.dash) line.setAttribute('stroke-dasharray', def.dash);
        svg.appendChild(line);
        item.appendChild(svg);
      }
      item.addEventListener('click', () => {
        this.borderLineTypeGrid.querySelectorAll('.tcp-line-type-item').forEach(el =>
          el.classList.remove('active'));
        item.classList.add('active');
        this.borderSelectedLineType = def.type;
      });
      this.borderLineTypeGrid.appendChild(item);
    });
    lineSection.appendChild(this.borderLineTypeGrid);
    frag.appendChild(lineSection);

    // 굵기 + 색
    const attrSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.x6ac29f'));
    const widthRow = this.row();
    widthRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label')));
    this.borderWidthSelect = document.createElement('select');
    this.borderWidthSelect.className = 'dialog-select';
    ['0.1mm', '0.12mm', '0.15mm', '0.2mm', '0.25mm', '0.3mm', '0.4mm'].forEach((text, i) => {
      const opt = document.createElement('option');
      opt.value = String(i); opt.textContent = text;
      this.borderWidthSelect.appendChild(opt);
    });
    widthRow.appendChild(this.borderWidthSelect);
    attrSection.appendChild(widthRow);

    const colorRow = this.row();
    colorRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.xa002ac')));
    this.borderColorInput = document.createElement('input');
    this.borderColorInput.type = 'color';
    this.borderColorInput.value = '#000000';
    this.borderColorInput.style.width = '40px';
    this.borderColorInput.style.height = '22px';
    colorRow.appendChild(this.borderColorInput);
    attrSection.appendChild(colorRow);
    frag.appendChild(attrSection);

    // 프리셋 버튼 + 미리보기
    const previewSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.x8eb63e'));

    // 프리셋: 모두/바깥쪽/안쪽
    const presetRow = this.row();
    const presetGroup = document.createElement('div');
    presetGroup.className = 'dialog-btn-group';
    const presets = [
      { label: i18nText('dialog.cellBorderBg.buildBorderTab.label.xf7473b'), dirs: [0, 1, 2, 3] },
      { label: i18nText('dialog.cellBorderBg.buildBorderTab.label.x634f07'), dirs: [0, 1, 2, 3] },
      { label: i18nText('dialog.cellBorderBg.buildBorderTab.label.x8eb06a'), dirs: [] as number[] },
    ];
    presets.forEach(p => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = p.label;
      btn.addEventListener('click', () => {
        for (const d of p.dirs) this.applyBorderDir(d);
        this.updateBorderPreview();
      });
      presetGroup.appendChild(btn);
    });
    presetRow.appendChild(presetGroup);
    previewSection.appendChild(presetRow);

    // SVG 미리보기 + 방향 버튼
    const previewWrap = document.createElement('div');
    previewWrap.className = 'tcp-border-preview-wrap';

    const mkDirBtn = (text: string, cls: string, dirIdx: number) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = `tcp-dir-btn ${cls}`;
      btn.textContent = text;
      btn.addEventListener('click', () => { this.applyBorderDir(dirIdx); this.updateBorderPreview(); });
      return btn;
    };
    previewWrap.appendChild(mkDirBtn('O', 'tcp-dir-all', 4));
    previewWrap.appendChild(mkDirBtn('▲', 'tcp-dir-top', 2));
    previewWrap.appendChild(document.createElement('span'));
    previewWrap.appendChild(mkDirBtn('◀', 'tcp-dir-left', 0));
    this.borderPreviewSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    this.borderPreviewSvg.classList.add('tcp-border-preview-svg');
    this.borderPreviewSvg.setAttribute('viewBox', '0 0 120 100');
    previewWrap.appendChild(this.borderPreviewSvg);
    previewWrap.appendChild(mkDirBtn('▶', 'tcp-dir-right', 1));
    previewWrap.appendChild(document.createElement('span'));
    previewWrap.appendChild(mkDirBtn('▼', 'tcp-dir-bottom', 3));
    previewSection.appendChild(previewWrap);

    // 선 모양 바로 적용
    const immediateRow = this.row();
    this.borderApplyImmediateCheck = this.checkbox(i18nText('dialog.cellBorderBg.checkbox.label'));
    immediateRow.appendChild(this.borderApplyImmediateCheck.parentElement!);
    previewSection.appendChild(immediateRow);

    frag.appendChild(previewSection);

    // 적용 범위
    frag.appendChild(this.buildScopeSection('border'));

    return frag;
  }

  private applyBorderDir(dirIdx: number): void {
    const val = {
      type: this.borderSelectedLineType,
      width: parseInt(this.borderWidthSelect.value, 10),
      color: this.borderColorInput.value,
    };
    if (dirIdx === 4) {
      this.borderEdits = [val, val, val, val];
    } else {
      this.borderEdits[dirIdx] = val;
    }
  }

  private updateBorderPreview(): void {
    const svg = this.borderPreviewSvg;
    if (!svg) return;
    while (svg.firstChild) svg.removeChild(svg.firstChild);

    const ns = 'http://www.w3.org/2000/svg';
    const bg = document.createElementNS(ns, 'rect');
    bg.setAttribute('x', '0'); bg.setAttribute('y', '0');
    bg.setAttribute('width', '120'); bg.setAttribute('height', '100');
    bg.style.setProperty('fill', DOC_PAPER_COLOR);
    svg.appendChild(bg);

    // 십자선
    for (const [x1, y1, x2, y2] of [['60', '5', '60', '95'], ['5', '50', '115', '50']]) {
      const line = document.createElementNS(ns, 'line');
      line.setAttribute('x1', x1); line.setAttribute('y1', y1);
      line.setAttribute('x2', x2); line.setAttribute('y2', y2);
      line.style.setProperty('stroke', PREVIEW_GUIDE_STROKE); line.setAttribute('stroke-width', '0.5');
      line.setAttribute('stroke-dasharray', '3,2');
      svg.appendChild(line);
    }

    const dashMap: Record<number, string> = {
      2: '6,3', 3: '2,2', 4: '8,3,2,3', 5: '8,3,2,3,2,3', 6: '12,3',
    };
    const drawLine = (x1: number, y1: number, x2: number, y2: number, b: { type: number; width: number; color: string }) => {
      if (b.type === 0) return;
      const w = Math.max(0.5, (b.width + 1) * 0.7);
      if (b.type === 7) {
        const offset = w * 0.8;
        const isVert = x1 === x2;
        for (const off of [-offset, offset]) {
          const l = document.createElementNS(ns, 'line');
          l.setAttribute('x1', String(x1 + (isVert ? off : 0)));
          l.setAttribute('y1', String(y1 + (isVert ? 0 : off)));
          l.setAttribute('x2', String(x2 + (isVert ? off : 0)));
          l.setAttribute('y2', String(y2 + (isVert ? 0 : off)));
          l.setAttribute('stroke', b.color); l.setAttribute('stroke-width', String(w * 0.5));
          svg.appendChild(l);
        }
      } else {
        const l = document.createElementNS(ns, 'line');
        l.setAttribute('x1', String(x1)); l.setAttribute('y1', String(y1));
        l.setAttribute('x2', String(x2)); l.setAttribute('y2', String(y2));
        l.setAttribute('stroke', b.color); l.setAttribute('stroke-width', String(w));
        if (dashMap[b.type]) l.setAttribute('stroke-dasharray', dashMap[b.type]);
        svg.appendChild(l);
      }
    };

    drawLine(2, 2, 2, 98, this.borderEdits[0]);
    drawLine(118, 2, 118, 98, this.borderEdits[1]);
    drawLine(2, 2, 118, 2, this.borderEdits[2]);
    drawLine(2, 98, 118, 98, this.borderEdits[3]);
  }

  // ─── 배경 탭 ────────────────────────────────

  private buildBackgroundTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    const fillSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.x2ce970'));

    const noneRow = this.row();
    this.bgNoneRadio = document.createElement('input');
    this.bgNoneRadio.type = 'radio';
    this.bgNoneRadio.name = 'cellBgFill';
    this.bgNoneRadio.checked = true;
    this.bgNoneRadio.addEventListener('change', () => this.updateBgPreview());
    noneRow.appendChild(this.bgNoneRadio);
    noneRow.appendChild(document.createTextNode(' 채우기 없음'));
    fillSection.appendChild(noneRow);

    const colorRow = this.row();
    this.bgColorRadio = document.createElement('input');
    this.bgColorRadio.type = 'radio';
    this.bgColorRadio.name = 'cellBgFill';
    this.bgColorRadio.addEventListener('change', () => this.updateBgPreview());
    colorRow.appendChild(this.bgColorRadio);
    colorRow.appendChild(document.createTextNode(' 색(Q)'));
    fillSection.appendChild(colorRow);

    // 면색 + 무늬색 + 무늬모양
    const colorFields = document.createElement('div');
    colorFields.style.marginLeft = '20px';

    const faceRow = this.row();
    faceRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.x596a8a')));
    this.bgColorPicker = document.createElement('input');
    this.bgColorPicker.type = 'color';
    this.bgColorPicker.value = '#ffffff';
    this.bgColorPicker.style.width = '40px';
    this.bgColorPicker.style.height = '22px';
    this.bgColorPicker.addEventListener('input', () => {
      this.bgColorRadio.checked = true;
      this.updateBgPreview();
    });
    faceRow.appendChild(this.bgColorPicker);
    colorFields.appendChild(faceRow);

    const patColorRow = this.row();
    patColorRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.x8c4c9a')));
    this.bgPatternColorPicker = document.createElement('input');
    this.bgPatternColorPicker.type = 'color';
    this.bgPatternColorPicker.value = '#000000';
    this.bgPatternColorPicker.style.width = '40px';
    this.bgPatternColorPicker.style.height = '22px';
    this.bgPatternColorPicker.addEventListener('input', () => {
      this.bgColorRadio.checked = true;
      this.updateBgPreview();
    });
    patColorRow.appendChild(this.bgPatternColorPicker);
    colorFields.appendChild(patColorRow);

    const patTypeRow = this.row();
    patTypeRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.x0829e0')));
    this.bgPatternTypeSelect = this.selectOptions([
      ['0', '없음'], ['1', '가로줄'], ['2', '세로줄'], ['3', '역슬래시'],
      ['4', '슬래시'], ['5', '십자'], ['6', 'X자'],
    ]);
    this.bgPatternTypeSelect.addEventListener('change', () => {
      this.bgColorRadio.checked = true;
      this.updateBgPreview();
    });
    patTypeRow.appendChild(this.bgPatternTypeSelect);
    colorFields.appendChild(patTypeRow);

    fillSection.appendChild(colorFields);

    // 미리보기
    this.bgPreviewBox = document.createElement('div');
    this.bgPreviewBox.className = 'tcp-bg-preview';
    fillSection.appendChild(this.bgPreviewBox);

    frag.appendChild(fillSection);

    // 적용 범위
    frag.appendChild(this.buildScopeSection('bg'));

    return frag;
  }

  private updateBgPreview(): void {
    if (!this.bgColorRadio.checked) {
      this.bgPreviewBox.style.background = DOC_PAPER_COLOR;
      return;
    }
    const faceColor = this.bgColorPicker.value;
    const patType = parseInt(this.bgPatternTypeSelect.value, 10);
    if (patType === 0) {
      this.bgPreviewBox.style.background = faceColor;
      return;
    }
    const patColor = this.bgPatternColorPicker.value;
    const patternMap: Record<number, string> = {
      1: `repeating-linear-gradient(0deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`,
      2: `repeating-linear-gradient(90deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`,
      3: `repeating-linear-gradient(135deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`,
      4: `repeating-linear-gradient(45deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`,
      5: `repeating-linear-gradient(0deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px),repeating-linear-gradient(90deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`,
      6: `repeating-linear-gradient(45deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px),repeating-linear-gradient(135deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`,
    };
    this.bgPreviewBox.style.background = `${patternMap[patType] || ''},${faceColor}`;
  }

  // ─── 대각선 탭 ────────────────────────────────

  private buildDiagonalTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';
    const layout = document.createElement('div');
    layout.className = 'tcp-diag-layout';
    const controls = document.createElement('div');
    controls.className = 'tcp-diag-controls';

    // 선 속성
    const lineSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.x6ac29f'));
    const typeRow = this.row();
    typeRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.x2ec5ae')));
    this.diagLineTypeSelect = this.selectOptions(DIAGONAL_LINE_TYPE_OPTIONS);
    this.diagLineTypeSelect.addEventListener('change', () => this.updateDiagonalPreview());
    typeRow.appendChild(this.diagLineTypeSelect);
    lineSection.appendChild(typeRow);

    const widthRow = this.row();
    widthRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label')));
    this.diagWidthSelect = this.selectOptions(DIAGONAL_WIDTH_OPTIONS);
    this.diagWidthSelect.addEventListener('change', () => this.updateDiagonalPreview());
    widthRow.appendChild(this.diagWidthSelect);
    lineSection.appendChild(widthRow);

    const colorRow = this.row();
    colorRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.xa002ac')));
    this.diagColorInput = document.createElement('input');
    this.diagColorInput.type = 'color';
    this.diagColorInput.value = '#000000';
    this.diagColorInput.style.width = '40px';
    this.diagColorInput.style.height = '22px';
    this.diagColorInput.addEventListener('input', () => this.updateDiagonalPreview());
    colorRow.appendChild(this.diagColorInput);
    lineSection.appendChild(colorRow);
    controls.appendChild(lineSection);

    // 대각선 방향 아이콘
    const dirSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.xb76080'));

    const bsRow = this.row();
    bsRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.xa2ebd4')));
    bsRow.appendChild(this.createDiagonalButtonGroup('backSlash', [
      ['CENTER', 0b010, '단순 역대각선'],
      ['CENTER_BELOW', 0b011, '아래쪽 분기 역대각선'],
      ['ALL', 0b111, '전체 분기 역대각선'],
    ]));
    dirSection.appendChild(bsRow);

    const fsRow = this.row();
    fsRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.xde6e59')));
    fsRow.appendChild(this.createDiagonalButtonGroup('slash', [
      ['CENTER', 0b010, '단순 대각선'],
      ['CENTER_BELOW', 0b011, '아래쪽 분기 대각선'],
      ['ALL', 0b111, '전체 분기 대각선'],
    ]));
    dirSection.appendChild(fsRow);

    const csRow = this.row();
    csRow.appendChild(this.label(i18nText('dialog.cellBorderBg.label.label.xdadce3')));
    csRow.appendChild(this.createCenterLineButtonGroup());
    dirSection.appendChild(csRow);

    controls.appendChild(dirSection);

    // 적용 범위
    controls.appendChild(this.buildScopeSection('diag'));

    const previewSection = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.x8eb63e'));
    this.diagPreviewSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    this.diagPreviewSvg.classList.add('tcp-diag-preview-svg');
    this.diagPreviewSvg.setAttribute('viewBox', '0 0 160 120');
    previewSection.appendChild(this.diagPreviewSvg);

    layout.appendChild(controls);
    layout.appendChild(previewSection);
    frag.appendChild(layout);

    return frag;
  }

  private createDiagonalButtonGroup(
    kind: 'slash' | 'backSlash',
    defs: [string, number, string][],
  ): HTMLDivElement {
    const group = document.createElement('div');
    group.className = 'tcp-diag-button-grid';
    const clearTitle = kind === 'slash' ? '대각선 해제' : '역대각선 해제';
    const clearBtn = this.createIconButton(clearTitle, this.createEmptyDiagonalIcon());
    clearBtn.addEventListener('click', () => {
      if (kind === 'slash') {
        this.diagSlashBits = 0;
      } else {
        this.diagBackSlashBits = 0;
      }
      this.updateDiagonalButtons();
      this.updateDiagonalPreview();
    });
    clearBtn.dataset.kind = kind;
    clearBtn.dataset.bits = '0';
    group.appendChild(clearBtn);
    this.diagButtons.push(clearBtn);

    for (const [shape, bits, title] of defs) {
      const btn = this.createIconButton(title, this.createDiagonalIcon(kind, shape));
      btn.addEventListener('click', () => {
        if (kind === 'slash') {
          this.diagSlashBits = this.diagSlashBits === bits ? 0 : bits;
        } else {
          this.diagBackSlashBits = this.diagBackSlashBits === bits ? 0 : bits;
        }
        if (this.hasDiagonalSelection()) this.diagCenterLine = 'NONE';
        this.updateDiagonalButtons();
        this.updateDiagonalPreview();
      });
      btn.dataset.kind = kind;
      btn.dataset.bits = String(bits);
      group.appendChild(btn);
      this.diagButtons.push(btn);
    }
    return group;
  }

  private createCenterLineButtonGroup(): HTMLDivElement {
    const group = document.createElement('div');
    group.className = 'tcp-diag-button-grid';
    const clearBtn = this.createIconButton(i18nText('dialog.cellBorderBg.createIconButton.label'), this.createEmptyDiagonalIcon());
    clearBtn.addEventListener('click', () => {
      this.diagCenterLine = 'NONE';
      this.updateDiagonalButtons();
      this.updateDiagonalPreview();
    });
    clearBtn.dataset.kind = 'centerLine';
    clearBtn.dataset.value = 'NONE';
    group.appendChild(clearBtn);
    this.diagButtons.push(clearBtn);

    const defs: [string, string][] = [
      ['VERTICAL', '가로 중심선'],
      ['HORIZONTAL', '세로 중심선'],
      ['CROSS', '가로세로 중심선'],
    ];
    for (const [value, title] of defs) {
      const btn = this.createIconButton(title, this.createCenterLineIcon(value));
      btn.addEventListener('click', () => {
        this.diagCenterLine = this.diagCenterLine === value ? 'NONE' : value;
        if (this.hasCenterLineSelection()) {
          this.diagSlashBits = 0;
          this.diagBackSlashBits = 0;
        }
        this.updateDiagonalButtons();
        this.updateDiagonalPreview();
      });
      btn.dataset.kind = 'centerLine';
      btn.dataset.value = value;
      group.appendChild(btn);
      this.diagButtons.push(btn);
    }
    return group;
  }

  private createIconButton(title: string, svg: SVGSVGElement): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'tcp-diag-btn';
    btn.title = title;
    btn.appendChild(svg);
    return btn;
  }

  private createEmptyDiagonalIcon(): SVGSVGElement {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', '0 0 36 28');
    const rect = document.createElementNS(ns, 'rect');
    rect.setAttribute('x', '3'); rect.setAttribute('y', '3');
    rect.setAttribute('width', '30'); rect.setAttribute('height', '22');
    rect.setAttribute('fill', 'none');
    rect.setAttribute('stroke', 'currentColor');
    rect.setAttribute('stroke-width', '1');
    svg.appendChild(rect);
    return svg;
  }

  private createDiagonalIcon(kind: 'slash' | 'backSlash', shape: string): SVGSVGElement {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', '0 0 36 28');
    const rect = document.createElementNS(ns, 'rect');
    rect.setAttribute('x', '3'); rect.setAttribute('y', '3');
    rect.setAttribute('width', '30'); rect.setAttribute('height', '22');
    rect.setAttribute('fill', 'none'); rect.setAttribute('stroke', 'currentColor');
    rect.setAttribute('stroke-width', '1');
    svg.appendChild(rect);
    const segments = this.diagonalSegments(kind, shape, 3, 3, 30, 22);
    for (const [x1, y1, x2, y2] of segments) {
      const line = document.createElementNS(ns, 'line');
      line.setAttribute('x1', String(x1)); line.setAttribute('y1', String(y1));
      line.setAttribute('x2', String(x2)); line.setAttribute('y2', String(y2));
      line.setAttribute('stroke', 'currentColor'); line.setAttribute('stroke-width', '1.6');
      svg.appendChild(line);
    }
    return svg;
  }

  private createCenterLineIcon(value: string): SVGSVGElement {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', '0 0 36 28');
    const rect = document.createElementNS(ns, 'rect');
    rect.setAttribute('x', '3'); rect.setAttribute('y', '3');
    rect.setAttribute('width', '30'); rect.setAttribute('height', '22');
    rect.setAttribute('fill', 'none'); rect.setAttribute('stroke', 'currentColor');
    rect.setAttribute('stroke-width', '1');
    svg.appendChild(rect);
    const lines: [number, number, number, number][] = [];
    if (value === 'VERTICAL' || value === 'CROSS') lines.push([3, 14, 33, 14]);
    if (value === 'HORIZONTAL' || value === 'CROSS') lines.push([18, 3, 18, 25]);
    for (const [x1, y1, x2, y2] of lines) {
      const line = document.createElementNS(ns, 'line');
      line.setAttribute('x1', String(x1)); line.setAttribute('y1', String(y1));
      line.setAttribute('x2', String(x2)); line.setAttribute('y2', String(y2));
      line.setAttribute('stroke', 'currentColor'); line.setAttribute('stroke-width', '1.6');
      svg.appendChild(line);
    }
    return svg;
  }

  private diagonalSegments(
    kind: 'slash' | 'backSlash',
    shape: string,
    x: number,
    y: number,
    w: number,
    h: number,
  ): [number, number, number, number][] {
    const x1 = x;
    const y1 = y;
    const x2 = x + w;
    const y2 = y + h;
    const cx = x + w / 2;
    const cy = y + h / 2;
    if (kind === 'slash') {
      if (shape === 'CENTER_BELOW') return [[x1, y1, x2, cy], [x1, y1, cx, y2]];
      if (shape === 'ALL') return [[x1, y2, x2, y1], [x1, y2, x2, cy], [x1, y2, cx, y1]];
      return [[x1, y2, x2, y1]];
    }
    if (shape === 'CENTER_BELOW') return [[x2, y1, x1, cy], [x2, y1, cx, y2]];
    if (shape === 'ALL') return [[x1, y1, x2, y2], [x1, y1, cx, y2], [x1, y1, x2, cy]];
    return [[x1, y1, x2, y2]];
  }

  private hasDiagonalSelection(): boolean {
    return this.diagSlashBits !== 0 || this.diagBackSlashBits !== 0;
  }

  private hasCenterLineSelection(): boolean {
    return this.diagCenterLine !== 'NONE';
  }

  private normalizeDiagonalExclusive(): void {
    if (this.applyMode === 'asOne') {
      this.diagCenterLine = 'NONE';
    } else if (this.hasCenterLineSelection()) {
      this.diagSlashBits = 0;
      this.diagBackSlashBits = 0;
    } else if (this.hasDiagonalSelection()) {
      this.diagCenterLine = 'NONE';
    }
  }

  private updateDiagonalButtons(): void {
    const centerLineUnavailable = this.applyMode === 'asOne';
    const centerLineSelected = this.hasCenterLineSelection();
    const diagonalSelected = this.hasDiagonalSelection();
    for (const btn of this.diagButtons) {
      const kind = btn.dataset.kind;
      if (kind === 'slash') {
        const active = Number(btn.dataset.bits) === this.diagSlashBits;
        btn.classList.toggle('active', active);
        btn.disabled = centerLineSelected;
        btn.setAttribute('aria-pressed', active ? 'true' : 'false');
      } else if (kind === 'backSlash') {
        const active = Number(btn.dataset.bits) === this.diagBackSlashBits;
        btn.classList.toggle('active', active);
        btn.disabled = centerLineSelected;
        btn.setAttribute('aria-pressed', active ? 'true' : 'false');
      } else if (kind === 'centerLine') {
        const active = !centerLineUnavailable && btn.dataset.value === this.diagCenterLine;
        btn.classList.toggle('active', active);
        btn.disabled = centerLineUnavailable || diagonalSelected;
        btn.setAttribute('aria-pressed', active ? 'true' : 'false');
      }
    }
  }

  private updateDiagonalPreview(): void {
    const svg = this.diagPreviewSvg;
    if (!svg) return;
    while (svg.firstChild) svg.removeChild(svg.firstChild);
    const ns = 'http://www.w3.org/2000/svg';
    const rect = document.createElementNS(ns, 'rect');
    rect.setAttribute('x', '20'); rect.setAttribute('y', '16');
    rect.setAttribute('width', '120'); rect.setAttribute('height', '88');
    rect.style.setProperty('fill', DOC_PAPER_COLOR);
    rect.setAttribute('stroke', 'var(--color-border)');
    svg.appendChild(rect);

    const lineType = parseInt(this.diagLineTypeSelect?.value ?? '0', 10);
    if (lineType === 0) return;
    const color = this.diagColorInput?.value ?? '#000000';
    const widthIndex = parseInt(this.diagWidthSelect?.value ?? '0', 10);
    const width = diagonalPreviewWidthPx(widthIndex);
    const dashMap: Record<number, string> = {
      2: '7,4', 3: '2,3', 4: '8,3,2,3', 5: '8,3,2,3,2,3', 6: '12,4',
    };
    const draw = (x1: number, y1: number, x2: number, y2: number, strokeWidth = width, offset = 0) => {
      const dx = x2 - x1;
      const dy = y2 - y1;
      const len = Math.hypot(dx, dy) || 1;
      const ox = (-dy / len) * offset;
      const oy = (dx / len) * offset;
      const line = document.createElementNS(ns, 'line');
      line.setAttribute('x1', String(x1 + ox)); line.setAttribute('y1', String(y1 + oy));
      line.setAttribute('x2', String(x2 + ox)); line.setAttribute('y2', String(y2 + oy));
      line.setAttribute('stroke', color); line.setAttribute('stroke-width', String(strokeWidth));
      if (dashMap[lineType]) line.setAttribute('stroke-dasharray', dashMap[lineType]);
      svg.appendChild(line);
    };

    const drawStyled = (x1: number, y1: number, x2: number, y2: number) => {
      const thin = Math.max(0.8, width * 0.28);
      const thick = Math.max(1.4, width * 0.72);
      const gap = Math.max(2.2, width * 0.5);
      if (lineType === 8) {
        draw(x1, y1, x2, y2, thin, -gap / 2);
        draw(x1, y1, x2, y2, thin, gap / 2);
      } else if (lineType === 9) {
        draw(x1, y1, x2, y2, thin, -gap / 2);
        draw(x1, y1, x2, y2, thick, gap / 2);
      } else if (lineType === 10) {
        draw(x1, y1, x2, y2, thick, -gap / 2);
        draw(x1, y1, x2, y2, thin, gap / 2);
      } else if (lineType === 11) {
        draw(x1, y1, x2, y2, thin, -gap);
        draw(x1, y1, x2, y2, thick, 0);
        draw(x1, y1, x2, y2, thin, gap);
      } else {
        draw(x1, y1, x2, y2);
      }
    };

    if (this.diagSlashBits !== 0) drawStyled(20, 104, 140, 16);
    if (this.diagBackSlashBits !== 0) drawStyled(20, 16, 140, 104);
    if (this.diagCenterLine === 'VERTICAL' || this.diagCenterLine === 'CROSS') drawStyled(20, 60, 140, 60);
    if (this.diagCenterLine === 'HORIZONTAL' || this.diagCenterLine === 'CROSS') drawStyled(80, 16, 80, 104);
  }

  // ─── 공통: 적용 범위 섹션 ────────────────────

  private buildScopeSection(prefix: string): HTMLDivElement {
    const section = this.createSection(i18nText('dialog.cellBorderBg.createSection.label.xb1824b'));
    const radioGroup = document.createElement('div');
    radioGroup.className = 'dialog-radio-group';
    const radios: HTMLInputElement[] = [];

    for (const [val, text] of [['selected', '선택된 셀(S)'], ['all', '모든 셀(E)']] as const) {
      const lbl = document.createElement('label');
      const inp = document.createElement('input');
      inp.type = 'radio';
      inp.name = `${prefix}Scope`;
      inp.value = val;
      if (val === 'selected') inp.checked = true;
      lbl.appendChild(inp);
      lbl.appendChild(document.createTextNode(text));
      radioGroup.appendChild(lbl);
      radios.push(inp);
    }
    section.appendChild(radioGroup);

    if (prefix === 'border') this.borderScopeRadios = radios;
    else if (prefix === 'bg') this.bgScopeRadios = radios;
    else this.diagScopeRadios = radios;

    return section;
  }

  // ─── 필드 채우기 ────────────────────────────

  private populateFields(): void {
    const cp = this.cellProps;

    // 테두리
    const dirs = ['borderLeft', 'borderRight', 'borderTop', 'borderBottom'] as const;
    for (let i = 0; i < 4; i++) {
      const b = cp[dirs[i]];
      if (b) {
        this.borderEdits[i] = { type: b.type, width: b.width, color: b.color };
      }
    }
    this.updateBorderPreview();

    // 배경
    if (cp.fillType === 'solid' && cp.fillColor) {
      this.bgColorRadio.checked = true;
      this.bgColorPicker.value = cp.fillColor;
      if (cp.patternColor) this.bgPatternColorPicker.value = cp.patternColor;
      if (cp.patternType != null) this.bgPatternTypeSelect.value = String(cp.patternType);
    } else {
      this.bgNoneRadio.checked = true;
    }
    this.updateBgPreview();

    this.diagLineTypeSelect.value = String(cp.diagonalLine ?? 0);
    this.diagWidthSelect.value = String(cp.diagonalWidth ?? 0);
    this.diagColorInput.value = cp.diagonalColor ?? '#000000';
    this.diagSlashBits = cp.diagonalSlash ?? 0;
    this.diagBackSlashBits = cp.diagonalBackSlash ?? 0;
    this.diagCenterLine = cp.centerLine ?? 'NONE';
    this.normalizeDiagonalExclusive();
    this.updateDiagonalButtons();
    this.updateDiagonalPreview();
  }

  private selectedCellIndicesForRange(range: CellRange): number[] {
    const { sec, ppi, ci } = this.tableCtx;
    const indices = new Set<number>();
    const overlaps = (bbox: CellBbox): boolean => {
      const endRow = bbox.row + Math.max(1, bbox.rowSpan) - 1;
      const endCol = bbox.col + Math.max(1, bbox.colSpan) - 1;
      return bbox.row <= range.endRow &&
        endRow >= range.startRow &&
        bbox.col <= range.endCol &&
        endCol >= range.startCol;
    };

    for (const bbox of this.wasm.getTableCellBboxes(sec, ppi, ci)) {
      if (overlaps(bbox)) indices.add(bbox.cellIdx);
    }
    return [...indices];
  }

  protected onConfirm(): void {
    const { sec, ppi, ci } = this.tableCtx;
    this.normalizeDiagonalExclusive();

    const newProps: Record<string, unknown> = {};
    newProps.borderFillId = this.cellProps.borderFillId ?? 0;

    // 테두리
    newProps.borderLeft = this.borderEdits[0];
    newProps.borderRight = this.borderEdits[1];
    newProps.borderTop = this.borderEdits[2];
    newProps.borderBottom = this.borderEdits[3];

    // 배경
    if (this.bgColorRadio.checked) {
      newProps.fillType = 'solid';
      newProps.fillColor = this.bgColorPicker.value;
      newProps.patternColor = this.bgPatternColorPicker.value;
      newProps.patternType = parseInt(this.bgPatternTypeSelect.value, 10);
    } else {
      newProps.fillType = 'none';
    }

    // 대각선/중심선
    newProps.diagonalLine = parseInt(this.diagLineTypeSelect.value, 10);
    newProps.diagonalSlash = this.diagSlashBits;
    newProps.diagonalBackSlash = this.diagBackSlashBits;
    newProps.diagonalWidth = parseInt(this.diagWidthSelect.value, 10);
    newProps.diagonalColor = this.diagColorInput.value;
    newProps.centerLine = this.diagCenterLine;

    // 적용 범위 결정: 마지막으로 선택한 탭의 scope를 따른다.
    const scopeRadios = this.activeTabId === 'background'
      ? this.bgScopeRadios
      : this.activeTabId === 'diagonal'
        ? this.diagScopeRadios
        : this.borderScopeRadios;
    const scope = scopeRadios?.find(r => r.checked)?.value ?? 'selected';
    // [#5959] 적용 대상을 데이터로 고정한다 — 커맨드 execute 가 다시 읽지 않는다.
    const buildTargets = (): CellBorderFillTarget => {
      if (this.applyMode === 'asOne') {
        if (scope === 'all') {
          const dims = this.wasm.getTableDimensions(sec, ppi, ci);
          return {
            kind: 'zone',
            range: {
              startRow: 0,
              startCol: 0,
              endRow: Math.max(0, dims.rowCount - 1),
              endCol: Math.max(0, dims.colCount - 1),
            },
          };
        }
        if (this.selectionRange) return { kind: 'zone', range: this.selectionRange };
        return { kind: 'cells', cellIdxes: [this.cellIdx] };
      }
      if (scope === 'all') {
        const dims = this.wasm.getTableDimensions(sec, ppi, ci);
        const all = Array.from({ length: dims.cellCount }, (_, i) => i);
        return { kind: 'cells', cellIdxes: all };
      }
      if (this.selectionRange) {
        const cellIndices = this.selectedCellIndicesForRange(this.selectionRange);
        return { kind: 'cells', cellIdxes: cellIndices.length > 0 ? cellIndices : [this.cellIdx] };
      }
      return { kind: 'cells', cellIdxes: [this.cellIdx] };
    };
    // 셀 테두리/배경 일괄 적용도 undo 대상이다 — 속성쌍 역연산 커맨드로 기록해
    // 스냅샷 슬롯을 쓰지 않는다(#5959). services 미주입 환경(구 호출부)에서만
    // 직접 적용 fallback(기록 없음 — 종전 계약 유지).
    // services 미주입 환경의 직접 적용 — 기록 없음(종전 계약 유지).
    const applyDirect = () => {
      const t = buildTargets();
      if (t.kind === 'zone') {
        this.wasm.setCellZoneProperties(sec, ppi, ci, t.range, newProps as Partial<CellProperties>);
        return;
      }
      // 셀 수만큼 setCellProperties 를 호출하므로 재페이지네이션을 묶는다(#4118).
      this.wasm.runInBatch(() => {
        for (const cellIdx of t.cellIdxes) {
          this.wasm.setCellProperties(sec, ppi, ci, cellIdx, newProps as Partial<CellProperties>);
        }
      });
    };
    const ih = this.services?.getInputHandler();
    if (ih) {
      applyCommandThroughRouter({
        services: this.services,
        label: 'CellBorderBgDialog',
        command: () => ({
          kind: 'command',
          command: new SetCellBorderFillCommand(
            sec, ppi, ci, buildTargets(), newProps, ih.getCursorPosition(),
          ),
        }),
        fallback: applyDirect,
      });
    } else {
      applyDirect();
      this.eventBus.emit('document-changed');
    }
  }

  // ─── DOM 헬퍼 ────────────────────────────────

  private createSection(title: string): HTMLDivElement {
    const sec = document.createElement('div');
    sec.className = 'dialog-section';
    const t = document.createElement('div');
    t.className = 'dialog-section-title';
    t.textContent = title;
    sec.appendChild(t);
    return sec;
  }

  private row(): HTMLDivElement {
    const r = document.createElement('div');
    r.className = 'dialog-row';
    return r;
  }

  private label(text: string): HTMLSpanElement {
    const l = document.createElement('span');
    l.className = 'dialog-label';
    l.textContent = text;
    return l;
  }

  private checkbox(text: string): HTMLInputElement {
    const lbl = document.createElement('label');
    lbl.className = 'dialog-checkbox';
    const inp = document.createElement('input');
    inp.type = 'checkbox';
    lbl.appendChild(inp);
    lbl.appendChild(document.createTextNode(text));
    return inp;
  }

  private selectOptions(items: string[][]): HTMLSelectElement {
    const sel = document.createElement('select');
    sel.className = 'dialog-select';
    for (const [value, text] of items) {
      const opt = document.createElement('option');
      opt.value = value;
      opt.textContent = text;
      sel.appendChild(opt);
    }
    return sel;
  }
}
