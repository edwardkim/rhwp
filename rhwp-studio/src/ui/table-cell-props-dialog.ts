import { ModalDialog } from './dialog';
import { appendSvgMarkup } from './dom-utils';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { CellProperties, TableProperties } from '@/core/types';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';

const HWPUNIT_PER_MM = 7200 / 25.4;

function hwpunitToMm(hu: number): number {
  return Math.round(hu * 25.4 / 7200 * 10) / 10;
}

function mmToHwpunit(mm: number): number {
  return Math.round(mm * HWPUNIT_PER_MM);
}

/** HWP16 (i16) → mm */
function hwp16ToMm(hu: number): number {
  return Math.round(hu * 25.4 / 7200 * 10) / 10;
}

function mmToHwp16(mm: number): number {
  return Math.round(mm * HWPUNIT_PER_MM);
}

const DOC_PAPER_COLOR = 'var(--doc-paper)';
const PREVIEW_GUIDE_STROKE = 'var(--ui-border-light)';
const LINE_SAMPLE_STROKE = 'currentColor';

/** 탭 정의 */
interface TabDef {
  id: string;
  label: string;
  builder: () => HTMLElement;
}

/**
 * 표/셀 속성 다이얼로그 (HWP 표준 6탭)
 */
export class TableCellPropsDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private tableCtx: { sec: number; ppi: number; ci: number };
  private cellIdx: number;
  /** 'table' = 표 선택 (6탭), 'cell' = 셀 선택 (4탭: 테두리·배경 제외) */
  private mode: 'table' | 'cell';

  // ─── 탭 UI ───
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];

  // ─── 셀 탭 필드 ───
  private cellWidthInput!: HTMLInputElement;
  private cellHeightInput!: HTMLInputElement;
  private cellPaddingInputs!: Record<string, HTMLInputElement>;
  private cellPaddingCheck!: HTMLInputElement;
  private cellVAlignBtns!: HTMLButtonElement[];
  private cellTextDirBtns!: HTMLButtonElement[];
  private cellHeaderCheck!: HTMLInputElement;
  private cellSingleLineCheck!: HTMLInputElement;
  private cellProtectCheck!: HTMLInputElement;
  private cellFieldNameInput!: HTMLInputElement;
  private cellEditableCheck!: HTMLInputElement;
  private cellApplySizeCheck!: HTMLInputElement;

  // ─── 표 탭 필드 ───
  private tablePageBreakSelect!: HTMLSelectElement;
  private tableRepeatHeaderCheck!: HTMLInputElement;
  private tablePaddingInputs!: Record<string, HTMLInputElement>;
  private tableAutoBorderCheck!: HTMLInputElement;
  private tableAutoBorderFields!: HTMLDivElement;

  // ─── 테두리 탭 필드 ───
  private borderCellSpacingInput!: HTMLInputElement;
  private borderLineTypeGrid!: HTMLDivElement;
  private borderSelectedLineType = 1;
  private borderWidthSelect!: HTMLSelectElement;
  private borderColorInput!: HTMLInputElement;
  private borderPreviewSvg!: SVGSVGElement;
  private borderApplyImmediateCheck!: HTMLInputElement;
  /** 4방향 테두리 편집 상태 */
  private borderEdits!: { type: number; width: number; color: string }[];
  /** 적용 대상: 'cell' 또는 'table' */
  private borderTarget!: string;
  /** 자동 경계선 설정 필드 */
  private borderAutoBorderCheck!: HTMLInputElement;
  private borderAutoBorderFields!: HTMLDivElement;

  // ─── 배경 탭 필드 ───
  private bgNoneRadio!: HTMLInputElement;
  private bgColorRadio!: HTMLInputElement;
  private bgColorPicker!: HTMLInputElement;
  private bgPatternColorPicker!: HTMLInputElement;
  private bgPatternTypeSelect!: HTMLSelectElement;
  private bgPreviewBox!: HTMLDivElement;
  /** 배경 적용 대상: 'cell' 또는 'table' */
  private bgTarget!: string;

  // ─── 기본 탭 필드 ───
  private basicWidthInput!: HTMLInputElement;
  private basicHeightInput!: HTMLInputElement;
  private treatAsCharCheck!: HTMLInputElement;
  private wrapBtns: HTMLButtonElement[] = [];
  private wrapValues = ['Square', 'TopAndBottom', 'BehindText', 'InFrontOfText'];
  private horzRelSelect!: HTMLSelectElement;
  private horzAlignSelect!: HTMLSelectElement;
  private horzOffsetInput!: HTMLInputElement;
  private vertRelSelect!: HTMLSelectElement;
  private vertAlignSelect!: HTMLSelectElement;
  private vertOffsetInput!: HTMLInputElement;
  private posGroup!: HTMLDivElement;
  private restrictInPageCheck!: HTMLInputElement;
  private allowOverlapCheck!: HTMLInputElement;
  private keepWithAnchorCheck!: HTMLInputElement;

  // ─── 여백/캡션 탭 필드 ───
  private marginOuterInputs!: Record<string, HTMLInputElement>;
  private captionDirSelect!: HTMLSelectElement;
  private captionSpacingInput!: HTMLInputElement;
  private captionWidthInput!: HTMLInputElement;
  private captionExpandCheck!: HTMLInputElement;
  private captionSection!: HTMLDivElement;
  private captionPosBtns!: HTMLButtonElement[];
  private captionFieldsWrap!: HTMLDivElement;

  // 현재 속성값 캐시
  private cellProps!: CellProperties;
  private tableProps!: TableProperties;
  /** undo 기록 라우팅용 (없으면 wasm 직접 호출 fallback). */
  private services: CommandServices | undefined;

  constructor(
    wasm: WasmBridge,
    eventBus: EventBus,
    tableCtx: { sec: number; ppi: number; ci: number },
    cellIdx: number,
    mode: 'table' | 'cell' = 'cell',
    services?: CommandServices,
  ) {
    super('표/셀 속성', 480);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.tableCtx = tableCtx;
    this.cellIdx = cellIdx;
    this.mode = mode;
    this.services = services;
  }

  show(): void {
    super.show();
    this.dialog.classList.add('tcp-dialog');
    // 속성 조회
    const { sec, ppi, ci } = this.tableCtx;
    this.cellProps = this.wasm.getCellProperties(sec, ppi, ci, this.cellIdx);
    this.tableProps = this.wasm.getTableProperties(sec, ppi, ci);
    this.populateFields();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'tcp-dialog-body';

    // 탭 정의: mode에 따라 테두리/배경 탭 포함 여부 결정
    const tabDefs: TabDef[] = [
      { id: 'basic', label: '기본', builder: () => this.buildBasicTab() },
      { id: 'margin', label: '여백/캡션', builder: () => this.buildMarginTab() },
      // 표 선택 시에만 테두리·배경 탭 표시 (셀 선택 시 별도 "셀 테두리/배경" 대화상자 사용)
      ...(this.mode === 'table' ? [
        { id: 'border', label: '테두리', builder: () => this.buildBorderTab() },
        { id: 'background', label: '배경', builder: () => this.buildBackgroundTab() },
      ] as TabDef[] : []),
      { id: 'table', label: '표', builder: () => this.buildTableTab() },
      { id: 'cell', label: '셀', builder: () => this.buildCellTab() },
    ];

    // 탭 헤더
    const tabBar = document.createElement('div');
    tabBar.className = 'dialog-tabs';

    const panelContainer = document.createElement('div');
    panelContainer.className = 'tcp-panel-container';

    for (let i = 0; i < tabDefs.length; i++) {
      const def = tabDefs[i];

      // 탭 버튼
      const btn = document.createElement('button');
      btn.className = 'dialog-tab';
      btn.textContent = def.label;
      btn.type = 'button';
      btn.addEventListener('click', () => this.switchTab(i));
      this.tabs.push(btn);
      tabBar.appendChild(btn);

      // 탭 패널
      const panel = document.createElement('div');
      panel.className = 'dialog-tab-panel';
      panel.appendChild(def.builder());
      this.panels.push(panel);
      panelContainer.appendChild(panel);
    }

    body.appendChild(tabBar);
    body.appendChild(panelContainer);

    // 기본 활성 탭: 표 선택 시 '기본' 탭, 셀 선택 시 '셀' 탭(마지막)
    this.switchTab(this.mode === 'table' ? 0 : tabDefs.length - 1);

    return body;
  }

  private switchTab(idx: number): void {
    for (let i = 0; i < this.tabs.length; i++) {
      this.tabs[i].classList.toggle('active', i === idx);
      this.panels[i].classList.toggle('active', i === idx);
    }
  }

  // ─── 셀 탭 ───────────────────────────────────────

  private buildCellTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    // 셀 크기
    const sizeSection = this.createSection('셀 크기');
    const sizeCheck = this.row();
    this.cellApplySizeCheck = this.checkbox('셀 크기 적용');
    sizeCheck.appendChild(this.cellApplySizeCheck.parentElement!);
    sizeSection.appendChild(sizeCheck);

    const sizeRow = this.row();
    sizeRow.appendChild(this.label('너비'));
    this.cellWidthInput = this.numberInput();
    sizeRow.appendChild(this.cellWidthInput);
    sizeRow.appendChild(this.unit('mm'));
    sizeRow.appendChild(this.label('높이'));
    this.cellHeightInput = this.numberInput();
    sizeRow.appendChild(this.cellHeightInput);
    sizeRow.appendChild(this.unit('mm'));
    sizeSection.appendChild(sizeRow);
    this.cellApplySizeCheck.addEventListener('change', () => this.updateCellSizeState());
    frag.appendChild(sizeSection);

    // 안 여백
    const padSection = this.createSection('안 여백');
    const padCheck = this.row();
    this.cellPaddingCheck = this.checkbox('안 여백 지정');
    padCheck.appendChild(this.cellPaddingCheck.parentElement!);
    padSection.appendChild(padCheck);

    const padRow = document.createElement('div');
    padRow.className = 'tcp-margin-row';
    const padGrid = document.createElement('div');
    padGrid.className = 'dialog-margin-grid';
    this.cellPaddingInputs = {};
    for (const [key, text] of [['left', '왼쪽'], ['right', '오른쪽'], ['top', '위쪽'], ['bottom', '아래쪽']] as const) {
      padGrid.appendChild(this.label(text));
      this.cellPaddingInputs[key] = this.numberInput();
      padGrid.appendChild(this.cellPaddingInputs[key]);
      padGrid.appendChild(this.unit('mm'));
    }
    padRow.appendChild(padGrid);
    padRow.appendChild(this.buildAllSpinner(this.cellPaddingInputs));
    padSection.appendChild(padRow);
    this.cellPaddingCheck.addEventListener('change', () => this.updateCellPaddingState());
    frag.appendChild(padSection);

    // 속성
    const attrSection = this.createSection('속성');

    // 세로 정렬
    const valignRow = this.row();
    valignRow.appendChild(this.label('세로 정렬'));
    const valignGroup = document.createElement('div');
    valignGroup.className = 'dialog-btn-group';
    this.cellVAlignBtns = ['위쪽', '가운데', '아래쪽'].map((text, i) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = text;
      btn.addEventListener('click', () => this.setButtonGroupActive(this.cellVAlignBtns, i));
      valignGroup.appendChild(btn);
      return btn;
    });
    valignRow.appendChild(valignGroup);
    attrSection.appendChild(valignRow);

    // 세로쓰기
    const tdirRow = this.row();
    tdirRow.appendChild(this.label('세로쓰기'));
    const tdirGroup = document.createElement('div');
    tdirGroup.className = 'dialog-btn-group';
    this.cellTextDirBtns = ['가로쓰기', '세로쓰기'].map((text, i) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = text;
      btn.addEventListener('click', () => {
        this.setButtonGroupActive(this.cellTextDirBtns, i);
        vertSubRow.classList.toggle('tcp-disabled', i === 0);
      });
      tdirGroup.appendChild(btn);
      return btn;
    });
    tdirRow.appendChild(tdirGroup);
    attrSection.appendChild(tdirRow);

    // 세로쓰기 하위: 문 눕힘/문 세움
    const vertSubRow = this.row();
    vertSubRow.className = 'dialog-row tcp-disabled';
    vertSubRow.appendChild(this.label(''));
    const vertSubGroup = document.createElement('div');
    vertSubGroup.className = 'dialog-btn-group';
    const vertSubLabels = ['문 눕힘(Q)', '문 세움(U)'];
    vertSubLabels.forEach((text, i) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = text;
      btn.addEventListener('click', () => {
        vertSubGroup.querySelectorAll('button').forEach((b, j) =>
          b.classList.toggle('active', j === i));
      });
      vertSubGroup.appendChild(btn);
    });
    // 기본: 문 눕힘 활성
    (vertSubGroup.firstChild as HTMLButtonElement)?.classList.add('active');
    vertSubRow.appendChild(vertSubGroup);
    attrSection.appendChild(vertSubRow);

    // 체크박스 옵션들
    const optRow1 = this.row();
    this.cellSingleLineCheck = this.checkbox('한 줄로 입력(S)');
    optRow1.appendChild(this.cellSingleLineCheck.parentElement!);
    this.cellProtectCheck = this.checkbox('셀 보호');
    optRow1.appendChild(this.cellProtectCheck.parentElement!);
    attrSection.appendChild(optRow1);

    const optRow2 = this.row();
    this.cellHeaderCheck = this.checkbox('제목 셀');
    optRow2.appendChild(this.cellHeaderCheck.parentElement!);
    attrSection.appendChild(optRow2);

    frag.appendChild(attrSection);

    // 필드
    const fieldSection = this.createSection('필드');
    const fieldRow = this.row();
    fieldRow.appendChild(this.label('필드 이름'));
    this.cellFieldNameInput = document.createElement('input');
    this.cellFieldNameInput.type = 'text';
    this.cellFieldNameInput.className = 'dialog-text-input';
    fieldRow.appendChild(this.cellFieldNameInput);
    fieldSection.appendChild(fieldRow);

    const fieldRow2 = this.row();
    this.cellEditableCheck = this.checkbox('양식 모드에서 편집 가능');
    fieldRow2.appendChild(this.cellEditableCheck.parentElement!);
    fieldSection.appendChild(fieldRow2);

    frag.appendChild(fieldSection);

    return frag;
  }

  // ─── 표 탭 ───────────────────────────────────────

  private buildTableTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    // 여러 쪽 지원
    const pageSection = this.createSection('여러 쪽 지원');

    const pbRow = this.row();
    pbRow.appendChild(this.label('쪽 경계에서(Q)'));
    this.tablePageBreakSelect = this.selectOptions([
      ['2', '나눔'], ['1', '셀 단위로 나눔'], ['0', '나누지 않음'],
    ]);
    pbRow.appendChild(this.tablePageBreakSelect);
    pageSection.appendChild(pbRow);

    const rhRow = this.row();
    this.tableRepeatHeaderCheck = this.checkbox('제목 줄 자동 반복');
    rhRow.appendChild(this.tableRepeatHeaderCheck.parentElement!);
    pageSection.appendChild(rhRow);

    // 자동으로 나뉜 표의 경계선 설정
    const abRow = this.row();
    this.tableAutoBorderCheck = this.checkbox('자동으로 나뉜 표의 경계선 설정(J)');
    abRow.appendChild(this.tableAutoBorderCheck.parentElement!);
    pageSection.appendChild(abRow);

    this.tableAutoBorderFields = document.createElement('div');
    this.tableAutoBorderFields.className = 'tcp-disabled';
    const abLineRow = this.row();
    abLineRow.appendChild(this.label('종류(N)'));
    const abLineType = this.selectOptions([
      ['0', '없음'], ['1', '실선'], ['2', '파선'], ['3', '점선'],
      ['4', '일점쇄선'], ['5', '이점쇄선'], ['6', '긴 파선'], ['7', '이중 실선'],
    ]);
    abLineType.disabled = true;
    abLineRow.appendChild(abLineType);
    this.tableAutoBorderFields.appendChild(abLineRow);
    const abWidthRow = this.row();
    abWidthRow.appendChild(this.label('굵기(H)'));
    const abWidth = this.selectOptions([
      ['0', '0.1mm'], ['1', '0.12mm'], ['2', '0.15mm'], ['3', '0.2mm'],
      ['4', '0.25mm'], ['5', '0.3mm'], ['6', '0.4mm'],
    ]);
    abWidth.disabled = true;
    abWidthRow.appendChild(abWidth);
    this.tableAutoBorderFields.appendChild(abWidthRow);
    const abColorRow = this.row();
    abColorRow.appendChild(this.label('색(S)'));
    const abColor = document.createElement('input');
    abColor.type = 'color';
    abColor.value = '#000000';
    abColor.disabled = true;
    abColor.style.width = '40px';
    abColor.style.height = '22px';
    abColorRow.appendChild(abColor);
    this.tableAutoBorderFields.appendChild(abColorRow);
    pageSection.appendChild(this.tableAutoBorderFields);

    this.tableAutoBorderCheck.addEventListener('change', () => {
      const enabled = this.tableAutoBorderCheck.checked;
      this.tableAutoBorderFields.classList.toggle('tcp-disabled', !enabled);
      abLineType.disabled = !enabled;
      abWidth.disabled = !enabled;
      abColor.disabled = !enabled;
    });

    frag.appendChild(pageSection);

    // 모든 셀 안 여백
    const padSection = this.createSection('모든 셀의 안 여백');
    const padRow = document.createElement('div');
    padRow.className = 'tcp-margin-row';
    const padGrid = document.createElement('div');
    padGrid.className = 'dialog-margin-grid';
    this.tablePaddingInputs = {};
    for (const [key, text] of [['left', '왼쪽'], ['right', '오른쪽'], ['top', '위쪽'], ['bottom', '아래쪽']] as const) {
      padGrid.appendChild(this.label(text));
      this.tablePaddingInputs[key] = this.numberInput();
      padGrid.appendChild(this.tablePaddingInputs[key]);
      padGrid.appendChild(this.unit('mm'));
    }
    padRow.appendChild(padGrid);
    padRow.appendChild(this.buildAllSpinner(this.tablePaddingInputs));
    padSection.appendChild(padRow);
    frag.appendChild(padSection);

    return frag;
  }

  // ─── 기본 탭 ──────────────────────────────

  private buildBasicTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    // ── 크기 ──
    const sizeSection = this.createSection('크기');
    const sizeRow = this.row();
    sizeRow.appendChild(this.label('너비'));
    this.basicWidthInput = this.numberInput();
    sizeRow.appendChild(this.basicWidthInput);
    sizeRow.appendChild(this.unit('mm'));
    sizeRow.appendChild(this.label('높이'));
    this.basicHeightInput = this.numberInput();
    sizeRow.appendChild(this.basicHeightInput);
    sizeRow.appendChild(this.unit('mm'));
    sizeSection.appendChild(sizeRow);
    const sizeNote = document.createElement('div');
    sizeNote.className = 'tcp-note';
    sizeNote.textContent = '※ 표 크기는 읽기 전용입니다 (셀 크기의 합)';
    sizeSection.appendChild(sizeNote);
    frag.appendChild(sizeSection);

    // ── 위치 ──
    const posSection = this.createSection('위치');

    // 글자처럼 취급 체크박스
    const tacRow = this.row();
    this.treatAsCharCheck = this.checkbox('글자처럼 취급');
    tacRow.appendChild(this.treatAsCharCheck.parentElement!);
    posSection.appendChild(tacRow);
    this.treatAsCharCheck.addEventListener('change', () => this.updatePositionVisibility());

    // ── 본문과의 배치 그룹 (글자처럼 취급 해제 시 활성) ──
    this.posGroup = document.createElement('div');
    this.posGroup.className = 'dialog-pos-group';

    // 본문과의 배치 (버튼 4개)
    const wrapRow = this.row();
    wrapRow.appendChild(this.label('본문과의 배치'));
    const wrapGroup = document.createElement('div');
    wrapGroup.className = 'dialog-btn-group';
    this.wrapBtns = [];
    const wrapLabels = ['어울림', '자리 차지', '글 뒤로', '글 앞으로'];
    wrapLabels.forEach((text, i) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'dialog-btn';
      btn.textContent = text;
      btn.addEventListener('click', () => this.selectWrap(i));
      wrapGroup.appendChild(btn);
      this.wrapBtns.push(btn);
    });
    wrapRow.appendChild(wrapGroup);
    this.posGroup.appendChild(wrapRow);

    // 가로 위치
    const hRow = this.row();
    hRow.appendChild(this.label('가로'));
    this.horzRelSelect = this.selectOptions([
      ['Paper', '종이'], ['Page', '쪽'], ['Column', '단'], ['Para', '문단'],
    ]);
    hRow.appendChild(this.horzRelSelect);
    hRow.appendChild(this.unit('의'));
    this.horzAlignSelect = this.selectOptions([
      ['Left', '왼쪽'], ['Center', '가운데'], ['Right', '오른쪽'],
      ['Inside', '안쪽'], ['Outside', '바깥쪽'],
    ]);
    hRow.appendChild(this.horzAlignSelect);
    hRow.appendChild(this.unit('기준'));
    this.horzOffsetInput = this.numberInput();
    hRow.appendChild(this.horzOffsetInput);
    hRow.appendChild(this.unit('mm'));
    this.posGroup.appendChild(hRow);

    // 세로 위치
    const vRow = this.row();
    vRow.appendChild(this.label('세로'));
    this.vertRelSelect = this.selectOptions([
      ['Paper', '종이'], ['Page', '쪽'], ['Para', '문단'],
    ]);
    vRow.appendChild(this.vertRelSelect);
    vRow.appendChild(this.unit('의'));
    this.vertAlignSelect = this.selectOptions([
      ['Top', '위'], ['Center', '가운데'], ['Bottom', '아래'],
      ['Inside', '안쪽'], ['Outside', '바깥쪽'],
    ]);
    vRow.appendChild(this.vertAlignSelect);
    vRow.appendChild(this.unit('기준'));
    this.vertOffsetInput = this.numberInput();
    vRow.appendChild(this.vertOffsetInput);
    vRow.appendChild(this.unit('mm'));
    this.posGroup.appendChild(vRow);

    // 체크박스 옵션들
    const optRow = this.row();
    this.restrictInPageCheck = this.checkbox('쪽 영역 안으로 제한');
    optRow.appendChild(this.restrictInPageCheck.parentElement!);
    this.allowOverlapCheck = this.checkbox('서로 겹침 허용');
    optRow.appendChild(this.allowOverlapCheck.parentElement!);
    this.posGroup.appendChild(optRow);

    const anchorRow = this.row();
    this.keepWithAnchorCheck = this.checkbox('개체와 조판부호를 항상 같은 쪽에 놓기');
    anchorRow.appendChild(this.keepWithAnchorCheck.parentElement!);
    this.posGroup.appendChild(anchorRow);

    posSection.appendChild(this.posGroup);
    frag.appendChild(posSection);

    // ── 개체 회전 ──
    const rotSection = this.createSection('개체 회전');
    const rotRow = this.row();
    rotRow.appendChild(this.label('회전각'));
    const rotInput = this.numberInput();
    rotInput.disabled = true;
    rotInput.value = '0';
    rotRow.appendChild(rotInput);
    rotRow.appendChild(this.unit('°'));
    rotSection.appendChild(rotRow);
    frag.appendChild(rotSection);

    // ── 기울이기 ──
    const skewSection = this.createSection('기울이기');
    const skewRow = this.row();
    skewRow.appendChild(this.label('가로'));
    const skewH = this.numberInput();
    skewH.disabled = true;
    skewH.value = '0';
    skewRow.appendChild(skewH);
    skewRow.appendChild(this.unit('°'));
    skewRow.appendChild(this.label('세로'));
    const skewV = this.numberInput();
    skewV.disabled = true;
    skewV.value = '0';
    skewRow.appendChild(skewV);
    skewRow.appendChild(this.unit('°'));
    skewSection.appendChild(skewRow);
    frag.appendChild(skewSection);

    // ── 기타 ──
    const etcSection = this.createSection('기타');
    const etcRow = this.row();
    etcRow.appendChild(this.label('번호 종류'));
    const numSelect = this.selectOptions([['Table', '표']]);
    numSelect.disabled = true;
    etcRow.appendChild(numSelect);
    etcSection.appendChild(etcRow);
    frag.appendChild(etcSection);

    return frag;
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

  private updatePositionVisibility(): void {
    const disabled = this.treatAsCharCheck.checked;
    this.posGroup.classList.toggle('disabled', disabled);
  }

  private updateCellPaddingState(): void {
    const enabled = this.cellPaddingCheck.checked;
    for (const input of Object.values(this.cellPaddingInputs)) {
      input.disabled = !enabled;
    }
  }

  private updateCellSizeState(): void {
    const enabled = this.cellApplySizeCheck.checked;
    this.cellWidthInput.disabled = !enabled;
    this.cellHeightInput.disabled = !enabled;
  }

  private selectWrap(idx: number): void {
    this.wrapBtns.forEach((b, i) => b.classList.toggle('active', i === idx));
  }

  private getSelectedWrap(): string {
    const idx = this.wrapBtns.findIndex(b => b.classList.contains('active'));
    return idx >= 0 ? this.wrapValues[idx] : 'Square';
  }

  // ─── 여백/캡션 탭 ──────────────────────────

  private buildMarginTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    // 바깥 여백 (활성)
    const outerSection = this.createSection('바깥 여백');
    const outerRow = document.createElement('div');
    outerRow.className = 'tcp-margin-row';
    const outerGrid = document.createElement('div');
    outerGrid.className = 'dialog-margin-grid';
    this.marginOuterInputs = {};
    for (const [key, text] of [['left', '왼쪽'], ['right', '오른쪽'], ['top', '위쪽'], ['bottom', '아래쪽']] as const) {
      outerGrid.appendChild(this.label(text));
      this.marginOuterInputs[key] = this.numberInput();
      outerGrid.appendChild(this.marginOuterInputs[key]);
      outerGrid.appendChild(this.unit('mm'));
    }
    outerRow.appendChild(outerGrid);
    outerRow.appendChild(this.buildAllSpinner(this.marginOuterInputs));
    outerSection.appendChild(outerRow);
    frag.appendChild(outerSection);

    // 캡션 넣기
    this.captionSection = this.createSection('캡션');

    // 캡션 하위 필드 래퍼 (가운데 선택 시 비활성)
    this.captionFieldsWrap = document.createElement('div');

    // 캡션 위치 3×3 아이콘 그리드 (가운데 = 캡션 없음)
    const captionGrid = document.createElement('div');
    captionGrid.className = 'tcp-caption-grid';
    this.captionPosBtns = [];
    // 3×3: 행(위/가운데/아래) × 열(왼쪽/가운데/오른쪽)
    // dir: 0=왼쪽, 1=오른쪽, 2=위, 3=아래, -1=없음(가운데)
    const capPositions = [
      { dir: 0, sub: 0, svg: this.captionSvg('left-top') },   // 왼쪽 위
      { dir: 2, sub: 0, svg: this.captionSvg('top') },        // 위
      { dir: 1, sub: 0, svg: this.captionSvg('right-top') },  // 오른쪽 위
      { dir: 0, sub: 1, svg: this.captionSvg('left-mid') },   // 왼쪽 가운데
      { dir: -1, sub: 0, svg: this.captionSvg('none') },      // 가운데 = 캡션 없음
      { dir: 1, sub: 1, svg: this.captionSvg('right-mid') },  // 오른쪽 가운데
      { dir: 0, sub: 2, svg: this.captionSvg('left-bot') },   // 왼쪽 아래
      { dir: 3, sub: 0, svg: this.captionSvg('bottom') },     // 아래
      { dir: 1, sub: 2, svg: this.captionSvg('right-bot') },  // 오른쪽 아래
    ];
    capPositions.forEach((pos) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'tcp-caption-item';
      appendSvgMarkup(btn, pos.svg);
      btn.dataset.dir = String(pos.dir);
      btn.dataset.sub = String(pos.sub);
      btn.addEventListener('click', () => {
        this.captionPosBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        const isNone = pos.dir === -1;
        this.captionFieldsWrap.classList.toggle('tcp-disabled', isNone);
        if (!isNone) {
          this.captionDirSelect.value = String(pos.dir);
        }
        this.updateCaptionWidthState();
      });
      captionGrid.appendChild(btn);
      this.captionPosBtns.push(btn);
    });
    this.captionSection.appendChild(captionGrid);

    // 숨겨진 방향 select (내부 값 관리용)
    this.captionDirSelect = document.createElement('select');
    this.captionDirSelect.className = 'dialog-select';
    this.captionDirSelect.style.display = 'none';
    const capDirs = [
      [0, '왼쪽'], [1, '오른쪽'], [2, '위쪽'], [3, '아래쪽'],
    ] as const;
    for (const [val, text] of capDirs) {
      const opt = document.createElement('option');
      opt.value = String(val);
      opt.textContent = text;
      this.captionDirSelect.appendChild(opt);
    }
    this.captionFieldsWrap.appendChild(this.captionDirSelect);

    const capGapRow = this.row();
    capGapRow.appendChild(this.label('간격'));
    this.captionSpacingInput = this.numberInput();
    capGapRow.appendChild(this.captionSpacingInput);
    capGapRow.appendChild(this.unit('mm'));
    this.captionFieldsWrap.appendChild(capGapRow);

    const capSizeRow = this.row();
    capSizeRow.appendChild(this.label('캡션 크기(S)'));
    this.captionWidthInput = this.numberInput();
    capSizeRow.appendChild(this.captionWidthInput);
    capSizeRow.appendChild(this.unit('mm'));
    this.captionFieldsWrap.appendChild(capSizeRow);

    const capExpandRow = this.row();
    this.captionExpandCheck = this.checkbox('여백 부분까지 너비 확대(W)');
    capExpandRow.appendChild(this.captionExpandCheck.parentElement!);
    this.captionFieldsWrap.appendChild(capExpandRow);

    this.captionSection.appendChild(this.captionFieldsWrap);
    frag.appendChild(this.captionSection);

    return frag;
  }

  /** 위/아래 선택 시 캡션 크기 비활성, 좌/우 선택 시 활성 */
  private updateCaptionWidthState(): void {
    const activeBtn = this.captionPosBtns.find(b => b.classList.contains('active'));
    const dir = activeBtn ? parseInt(activeBtn.dataset.dir!, 10) : -1;
    // dir 2=위, 3=아래 → 캡션 크기 비활성 (위/아래는 표 너비와 동일)
    const isTopBottom = dir === 2 || dir === 3;
    this.captionWidthInput.disabled = isTopBottom;
    if (isTopBottom) {
      this.captionWidthInput.style.opacity = '0.5';
    } else {
      this.captionWidthInput.style.opacity = '';
    }
  }

  /** 캡션 위치별 간단한 SVG 아이콘 */
  private captionSvg(pos: string): string {
    const table = '<rect x="12" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>';
    const capH = '<rect fill="#ffd966" stroke="#c09000" stroke-width="0.5" rx="1"';
    const capV = '<rect fill="#ffd966" stroke="#c09000" stroke-width="0.5" rx="1"';
    const w = 54, h = 40;
    let inner = '';
    switch (pos) {
      case 'top':    inner = `${capH} x="12" y="2" width="30" height="5"/><rect x="12" y="10" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>`; break;
      case 'bottom': inner = `${table}${capH} x="12" y="32" width="30" height="5"/>`; break;
      case 'left-top':  inner = `<rect x="18" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="4" y="8" width="12" height="6"/>`; break;
      case 'left-mid':  inner = `<rect x="18" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="4" y="14" width="12" height="6"/>`; break;
      case 'left-bot':  inner = `<rect x="18" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="4" y="24" width="12" height="6"/>`; break;
      case 'right-top': inner = `<rect x="4" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="38" y="8" width="12" height="6"/>`; break;
      case 'right-mid': inner = `<rect x="4" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="38" y="14" width="12" height="6"/>`; break;
      case 'right-bot': inner = `<rect x="4" y="8" width="30" height="22" rx="1" fill="#d0d8e8" stroke="#6182d6" stroke-width="0.5"/>${capV} x="38" y="24" width="12" height="6"/>`; break;
      case 'none': inner = `${table}<line x1="10" y1="6" x2="44" y2="34" stroke="#c00" stroke-width="1.5"/>`; break;
      default: inner = table;
    }
    return `<svg viewBox="0 0 ${w} ${h}" xmlns="http://www.w3.org/2000/svg">${inner}</svg>`;
  }

  // ─── 테두리 탭 ──────────────────────────────────

  private buildBorderTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    this.borderTarget = 'table';

    // ── 선 종류 시각적 격자 ──
    const lineSection = this.createSection('선 종류(Y)');
    this.borderLineTypeGrid = document.createElement('div');
    this.borderLineTypeGrid.className = 'tcp-line-type-grid';
    const lineTypeDefs = [
      { type: 0, label: '없음' },
      { type: 1, dash: '' },        // 실선
      { type: 2, dash: '6,3' },     // 파선
      { type: 3, dash: '2,2' },     // 점선
      { type: 4, dash: '8,3,2,3' }, // 일점쇄선
      { type: 5, dash: '8,3,2,3,2,3' }, // 이점쇄선
      { type: 6, dash: '12,3' },    // 긴 파선
      { type: 8, label: '이중' },   // 이중 실선 (HWP Double=8)
    ];
    lineTypeDefs.forEach(def => {
      const item = document.createElement('div');
      item.className = 'tcp-line-type-item';
      if (def.type === 1) item.classList.add('active');
      if (def.type === 0) {
        const span = document.createElement('span');
        span.className = 'tcp-line-type-none';
        span.textContent = '없음';
        item.appendChild(span);
      } else if (def.type === 8) {
        // 이중 실선 SVG
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('viewBox', '0 0 48 10');
        const l1 = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        l1.setAttribute('x1', '0'); l1.setAttribute('y1', '3');
        l1.setAttribute('x2', '48'); l1.setAttribute('y2', '3');
        l1.setAttribute('stroke', LINE_SAMPLE_STROKE); l1.setAttribute('stroke-width', '1');
        const l2 = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        l2.setAttribute('x1', '0'); l2.setAttribute('y1', '7');
        l2.setAttribute('x2', '48'); l2.setAttribute('y2', '7');
        l2.setAttribute('stroke', LINE_SAMPLE_STROKE); l2.setAttribute('stroke-width', '1');
        svg.appendChild(l1); svg.appendChild(l2);
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

    // ── 굵기 + 색 ──
    const attrSection = this.createSection('선 속성');
    const widthRow = this.row();
    widthRow.appendChild(this.label('굵기'));
    this.borderWidthSelect = document.createElement('select');
    this.borderWidthSelect.className = 'dialog-select';
    const widths = ['0.1mm', '0.12mm', '0.15mm', '0.2mm', '0.25mm', '0.3mm', '0.4mm'];
    widths.forEach((text, i) => {
      const opt = document.createElement('option');
      opt.value = String(i); opt.textContent = text;
      this.borderWidthSelect.appendChild(opt);
    });
    widthRow.appendChild(this.borderWidthSelect);
    attrSection.appendChild(widthRow);

    const colorRow = this.row();
    colorRow.appendChild(this.label('색'));
    this.borderColorInput = document.createElement('input');
    this.borderColorInput.type = 'color';
    this.borderColorInput.value = '#000000';
    this.borderColorInput.style.width = '40px';
    this.borderColorInput.style.height = '22px';
    colorRow.appendChild(this.borderColorInput);
    attrSection.appendChild(colorRow);
    frag.appendChild(attrSection);

    // ── 미리보기 + 방향 버튼 (그리드 배치) ──
    const previewSection = this.createSection('미리 보기');
    const previewWrap = document.createElement('div');
    previewWrap.className = 'tcp-border-preview-wrap';

    // 방향 버튼: 모두(좌상), 위(상중), 왼(좌중), SVG(중중), 오른(우중), 아래(하중)
    const mkDirBtn = (text: string, cls: string, dirIdx: number) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = `tcp-dir-btn ${cls}`;
      btn.textContent = text;
      btn.addEventListener('click', () => this.applyBorderToDirection(dirIdx));
      return btn;
    };
    previewWrap.appendChild(mkDirBtn('O', 'tcp-dir-all', 4));   // 모두
    previewWrap.appendChild(mkDirBtn('▲', 'tcp-dir-top', 2));   // 위
    previewWrap.appendChild(document.createElement('span'));      // 우상 빈칸
    previewWrap.appendChild(mkDirBtn('◀', 'tcp-dir-left', 0));  // 왼
    // SVG 미리보기
    this.borderPreviewSvg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    this.borderPreviewSvg.classList.add('tcp-border-preview-svg');
    this.borderPreviewSvg.setAttribute('viewBox', '0 0 120 100');
    previewWrap.appendChild(this.borderPreviewSvg);
    previewWrap.appendChild(mkDirBtn('▶', 'tcp-dir-right', 1)); // 오른
    previewWrap.appendChild(document.createElement('span'));      // 좌하 빈칸
    previewWrap.appendChild(mkDirBtn('▼', 'tcp-dir-bottom', 3));// 아래
    previewSection.appendChild(previewWrap);

    // 선 모양 바로 적용
    const immediateRow = this.row();
    this.borderApplyImmediateCheck = this.checkbox('선 모양 바로 적용(I)');
    immediateRow.appendChild(this.borderApplyImmediateCheck.parentElement!);
    previewSection.appendChild(immediateRow);

    frag.appendChild(previewSection);

    // ── 셀 간격 ──
    const spacingSection = this.createSection('셀 간격');
    const spacingRow = this.row();
    spacingRow.appendChild(this.label('셀 간격'));
    this.borderCellSpacingInput = this.numberInput();
    spacingRow.appendChild(this.borderCellSpacingInput);
    spacingRow.appendChild(this.unit('mm'));
    spacingSection.appendChild(spacingRow);

    const noteDiv = document.createElement('div');
    noteDiv.className = 'tcp-note';
    noteDiv.textContent = '※ 표 테두리는 [셀 간격]에 값을 입력해야 나타납니다';
    spacingSection.appendChild(noteDiv);
    frag.appendChild(spacingSection);

    // ── 자동 나뉜 표 경계선 설정 ──
    const abSection = this.createSection('자동 경계선');
    const abRow = this.row();
    this.borderAutoBorderCheck = this.checkbox('자동으로 나뉜 표의 경계선 설정(J)');
    abRow.appendChild(this.borderAutoBorderCheck.parentElement!);
    abSection.appendChild(abRow);

    this.borderAutoBorderFields = document.createElement('div');
    this.borderAutoBorderFields.className = 'tcp-disabled';
    const abLineRow = this.row();
    abLineRow.appendChild(this.label('종류'));
    const abLineType = this.selectOptions([
      ['0', '없음'], ['1', '실선'], ['2', '파선'], ['3', '점선'],
      ['4', '일점쇄선'], ['5', '이점쇄선'], ['6', '긴 파선'], ['7', '이중 실선'],
    ]);
    abLineType.disabled = true;
    abLineRow.appendChild(abLineType);
    this.borderAutoBorderFields.appendChild(abLineRow);
    const abWidthRow = this.row();
    abWidthRow.appendChild(this.label('굵기'));
    const abWidth = this.selectOptions([
      ['0', '0.1mm'], ['1', '0.12mm'], ['2', '0.15mm'], ['3', '0.2mm'],
      ['4', '0.25mm'], ['5', '0.3mm'], ['6', '0.4mm'],
    ]);
    abWidth.disabled = true;
    abWidthRow.appendChild(abWidth);
    this.borderAutoBorderFields.appendChild(abWidthRow);
    const abColorRow = this.row();
    abColorRow.appendChild(this.label('색'));
    const abColor = document.createElement('input');
    abColor.type = 'color'; abColor.value = '#000000';
    abColor.disabled = true;
    abColor.style.width = '40px'; abColor.style.height = '22px';
    abColorRow.appendChild(abColor);
    this.borderAutoBorderFields.appendChild(abColorRow);
    abSection.appendChild(this.borderAutoBorderFields);

    this.borderAutoBorderCheck.addEventListener('change', () => {
      const en = this.borderAutoBorderCheck.checked;
      this.borderAutoBorderFields.classList.toggle('tcp-disabled', !en);
      abLineType.disabled = !en; abWidth.disabled = !en; abColor.disabled = !en;
    });
    frag.appendChild(abSection);

    // 초기 편집 상태
    this.borderEdits = [
      { type: 1, width: 0, color: '#000000' },
      { type: 1, width: 0, color: '#000000' },
      { type: 1, width: 0, color: '#000000' },
      { type: 1, width: 0, color: '#000000' },
    ];

    return frag;
  }

  /** 현재 선택된 선 종류/굵기/색을 지정 방향에 적용 */
  private applyBorderToDirection(dirIdx: number): void {
    const lineType = this.borderSelectedLineType;
    const width = parseInt(this.borderWidthSelect.value, 10);
    const color = this.borderColorInput.value;
    const val = { type: lineType, width, color };
    if (dirIdx === 4) { // 모두
      this.borderEdits = [val, val, val, val];
    } else {
      this.borderEdits[dirIdx] = val;
    }
    this.updateBorderPreview();
  }

  /** SVG 기반 테두리 미리보기 갱신 (십자선 포함) */
  private updateBorderPreview(): void {
    const svg = this.borderPreviewSvg;
    if (!svg) return;
    // clear
    while (svg.firstChild) svg.removeChild(svg.firstChild);

    const ns = 'http://www.w3.org/2000/svg';
    // 배경
    const bg = document.createElementNS(ns, 'rect');
    bg.setAttribute('x', '0'); bg.setAttribute('y', '0');
    bg.setAttribute('width', '120'); bg.setAttribute('height', '100');
    bg.style.setProperty('fill', DOC_PAPER_COLOR);
    svg.appendChild(bg);

    // 십자선 (셀 구분선) — 연한 회색 점선
    const cross1 = document.createElementNS(ns, 'line');
    cross1.setAttribute('x1', '60'); cross1.setAttribute('y1', '5');
    cross1.setAttribute('x2', '60'); cross1.setAttribute('y2', '95');
    cross1.style.setProperty('stroke', PREVIEW_GUIDE_STROKE); cross1.setAttribute('stroke-width', '0.5');
    cross1.setAttribute('stroke-dasharray', '3,2');
    svg.appendChild(cross1);
    const cross2 = document.createElementNS(ns, 'line');
    cross2.setAttribute('x1', '5'); cross2.setAttribute('y1', '50');
    cross2.setAttribute('x2', '115'); cross2.setAttribute('y2', '50');
    cross2.style.setProperty('stroke', PREVIEW_GUIDE_STROKE); cross2.setAttribute('stroke-width', '0.5');
    cross2.setAttribute('stroke-dasharray', '3,2');
    svg.appendChild(cross2);

    // 4방향 테두리 선
    const drawBorder = (x1: number, y1: number, x2: number, y2: number, b: { type: number; width: number; color: string }) => {
      if (b.type === 0) return;
      const w = Math.max(0.5, (b.width + 1) * 0.7);
      const dashMap: Record<number, string> = {
        2: '6,3', 3: '2,2', 4: '8,3,2,3', 5: '8,3,2,3,2,3', 6: '12,3',
      };
      if (b.type === 7) {
        // 이중선
        const offset = w * 0.8;
        for (const off of [-offset, offset]) {
          const line = document.createElementNS(ns, 'line');
          const isVert = x1 === x2;
          line.setAttribute('x1', String(x1 + (isVert ? off : 0)));
          line.setAttribute('y1', String(y1 + (isVert ? 0 : off)));
          line.setAttribute('x2', String(x2 + (isVert ? off : 0)));
          line.setAttribute('y2', String(y2 + (isVert ? 0 : off)));
          line.setAttribute('stroke', b.color); line.setAttribute('stroke-width', String(w * 0.5));
          svg.appendChild(line);
        }
      } else {
        const line = document.createElementNS(ns, 'line');
        line.setAttribute('x1', String(x1)); line.setAttribute('y1', String(y1));
        line.setAttribute('x2', String(x2)); line.setAttribute('y2', String(y2));
        line.setAttribute('stroke', b.color); line.setAttribute('stroke-width', String(w));
        if (dashMap[b.type]) line.setAttribute('stroke-dasharray', dashMap[b.type]);
        svg.appendChild(line);
      }
    };

    // left, right, top, bottom
    drawBorder(2, 2, 2, 98, this.borderEdits[0]);     // 왼쪽
    drawBorder(118, 2, 118, 98, this.borderEdits[1]);  // 오른쪽
    drawBorder(2, 2, 118, 2, this.borderEdits[2]);     // 위
    drawBorder(2, 98, 118, 98, this.borderEdits[3]);   // 아래
  }

  /** 적용 대상(셀/표) 전환 시 해당 속성으로 테두리 편집 상태 갱신 */
  private populateBorderFromTarget(): void {
    const props = this.borderTarget === 'table' ? this.tableProps : this.cellProps;
    const dirs = ['borderLeft', 'borderRight', 'borderTop', 'borderBottom'] as const;
    for (let i = 0; i < 4; i++) {
      const b = props[dirs[i]];
      if (b) {
        this.borderEdits[i] = { type: b.type, width: b.width, color: b.color };
      }
    }
    this.updateBorderPreview();
  }

  // ─── 배경 탭 ──────────────────────────────────

  private buildBackgroundTab(): HTMLElement {
    const frag = document.createElement('div');
    frag.className = 'tcp-tab-content';

    this.bgTarget = 'table';

    // 색 채우기
    const fillSection = this.createSection('채우기');

    const noneRow = this.row();
    this.bgNoneRadio = document.createElement('input');
    this.bgNoneRadio.type = 'radio';
    this.bgNoneRadio.name = 'bgFill';
    this.bgNoneRadio.checked = true;
    this.bgNoneRadio.addEventListener('change', () => this.updateBgPreview());
    noneRow.appendChild(this.bgNoneRadio);
    noneRow.appendChild(document.createTextNode(' 채우기 없음'));
    fillSection.appendChild(noneRow);

    const colorRow = this.row();
    this.bgColorRadio = document.createElement('input');
    this.bgColorRadio.type = 'radio';
    this.bgColorRadio.name = 'bgFill';
    this.bgColorRadio.addEventListener('change', () => this.updateBgPreview());
    colorRow.appendChild(this.bgColorRadio);
    colorRow.appendChild(document.createTextNode(' 색(Q)'));
    fillSection.appendChild(colorRow);

    // 면색 + 무늬색 + 무늬모양
    const colorFields = document.createElement('div');
    colorFields.style.marginLeft = '20px';

    const faceRow = this.row();
    faceRow.appendChild(this.label('면색(C)'));
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
    patColorRow.appendChild(this.label('무늬색(K)'));
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
    patTypeRow.appendChild(this.label('무늬모양(L)'));
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

    // 그러데이션 (읽기 전용)
    const gradSection = this.createSection('그러데이션');
    gradSection.classList.add('disabled');
    const gradRow = this.row();
    gradRow.appendChild(this.label('유형'));
    const gradSelect = document.createElement('select');
    gradSelect.className = 'dialog-select';
    gradSelect.disabled = true;
    for (const text of ['선형', '방사형', '원뿔형', '사각형']) {
      const opt = document.createElement('option');
      opt.textContent = text;
      gradSelect.appendChild(opt);
    }
    gradRow.appendChild(gradSelect);
    gradSection.appendChild(gradRow);
    frag.appendChild(gradSection);

    // 그림 (읽기 전용)
    const imgSection = this.createSection('그림');
    imgSection.classList.add('disabled');
    const imgRow = this.row();
    imgRow.appendChild(this.label('그림 파일'));
    const imgBtn = document.createElement('button');
    imgBtn.type = 'button';
    imgBtn.className = 'dialog-btn';
    imgBtn.textContent = '열기...';
    imgBtn.disabled = true;
    imgRow.appendChild(imgBtn);
    imgSection.appendChild(imgRow);
    frag.appendChild(imgSection);

    return frag;
  }

  /** 배경 대상(셀/표) 전환 시 해당 속성으로 배경 상태 갱신 */
  private populateBgFromTarget(): void {
    const props = this.bgTarget === 'table' ? this.tableProps : this.cellProps;
    if (props.fillType === 'solid' && props.fillColor) {
      this.bgColorRadio.checked = true;
      this.bgColorPicker.value = props.fillColor;
      if (props.patternColor) this.bgPatternColorPicker.value = props.patternColor;
      if (props.patternType != null) this.bgPatternTypeSelect.value = String(props.patternType);
    } else {
      this.bgNoneRadio.checked = true;
    }
    this.updateBgPreview();
  }

  /** 배경 미리보기 갱신 (무늬 패턴 포함) */
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
    // CSS repeating-linear-gradient 패턴
    const patternMap: Record<number, string> = {
      1: `repeating-linear-gradient(0deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`,  // 가로줄
      2: `repeating-linear-gradient(90deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`, // 세로줄
      3: `repeating-linear-gradient(135deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`,// 역슬래시
      4: `repeating-linear-gradient(45deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`, // 슬래시
      5: `repeating-linear-gradient(0deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px),repeating-linear-gradient(90deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 4px)`, // 십자
      6: `repeating-linear-gradient(45deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px),repeating-linear-gradient(135deg,${patColor} 0px,${patColor} 1px,transparent 1px,transparent 5px)`, // X자
    };
    this.bgPreviewBox.style.background = `${patternMap[patType] || ''},${faceColor}`;
  }

  // ─── 필드 채우기 ─────────────────────────────────

  private populateFields(): void {
    const cp = this.cellProps;
    const tp = this.tableProps;

    // 셀 탭
    this.cellWidthInput.value = hwpunitToMm(cp.width).toFixed(1);
    this.cellHeightInput.value = hwpunitToMm(cp.height).toFixed(1);
    this.updateCellSizeState();
    this.cellPaddingInputs['left'].value = hwp16ToMm(cp.paddingLeft).toFixed(1);
    this.cellPaddingInputs['right'].value = hwp16ToMm(cp.paddingRight).toFixed(1);
    this.cellPaddingInputs['top'].value = hwp16ToMm(cp.paddingTop).toFixed(1);
    this.cellPaddingInputs['bottom'].value = hwp16ToMm(cp.paddingBottom).toFixed(1);
    this.cellPaddingCheck.checked = cp.applyInnerMargin ?? false;
    this.updateCellPaddingState();
    this.setButtonGroupActive(this.cellVAlignBtns, cp.verticalAlign);
    this.setButtonGroupActive(this.cellTextDirBtns, cp.textDirection);
    this.cellHeaderCheck.checked = cp.isHeader;
    this.cellProtectCheck.checked = cp.cellProtect ?? false;
    this.cellFieldNameInput.value = cp.fieldName ?? '';
    this.cellEditableCheck.checked = cp.editableInForm ?? false;

    // 표 탭
    this.tablePageBreakSelect.value = String(tp.pageBreak ?? 0);
    this.tableRepeatHeaderCheck.checked = tp.repeatHeader;
    this.tablePaddingInputs['left'].value = hwp16ToMm(tp.paddingLeft).toFixed(1);
    this.tablePaddingInputs['right'].value = hwp16ToMm(tp.paddingRight).toFixed(1);
    this.tablePaddingInputs['top'].value = hwp16ToMm(tp.paddingTop).toFixed(1);
    this.tablePaddingInputs['bottom'].value = hwp16ToMm(tp.paddingBottom).toFixed(1);

    // 기본 탭
    if (tp.tableWidth != null) {
      this.basicWidthInput.value = hwpunitToMm(tp.tableWidth).toFixed(1);
      this.basicWidthInput.readOnly = true;
      this.basicWidthInput.style.removeProperty('background');
    }
    if (tp.tableHeight != null) {
      this.basicHeightInput.value = hwpunitToMm(tp.tableHeight).toFixed(1);
      this.basicHeightInput.readOnly = true;
      this.basicHeightInput.style.removeProperty('background');
    }
    this.treatAsCharCheck.checked = tp.treatAsChar ?? true;
    this.selectWrap(this.wrapValues.indexOf(tp.textWrap ?? 'Square'));
    this.horzRelSelect.value = tp.horzRelTo ?? 'Paper';
    this.horzAlignSelect.value = tp.horzAlign ?? 'Left';
    this.horzOffsetInput.value = hwpunitToMm(tp.horzOffset ?? 0).toFixed(1);
    this.vertRelSelect.value = tp.vertRelTo ?? 'Paper';
    this.vertAlignSelect.value = tp.vertAlign ?? 'Top';
    this.vertOffsetInput.value = hwpunitToMm(tp.vertOffset ?? 0).toFixed(1);
    this.restrictInPageCheck.checked = tp.restrictInPage ?? true;
    this.allowOverlapCheck.checked = tp.allowOverlap ?? false;
    this.keepWithAnchorCheck.checked = tp.keepWithAnchor ?? false;
    this.updatePositionVisibility();

    // 여백/캡션 탭
    this.marginOuterInputs['left'].value = hwp16ToMm(tp.outerLeft ?? 0).toFixed(1);
    this.marginOuterInputs['right'].value = hwp16ToMm(tp.outerRight ?? 0).toFixed(1);
    this.marginOuterInputs['top'].value = hwp16ToMm(tp.outerTop ?? 0).toFixed(1);
    this.marginOuterInputs['bottom'].value = hwp16ToMm(tp.outerBottom ?? 0).toFixed(1);

    // 기본값 설정
    this.captionSpacingInput.value = '3.0';
    this.captionWidthInput.value = '30.0';

    if (tp.hasCaption) {
      const dir = tp.captionDirection ?? 3;
      const va = tp.captionVertAlign ?? 0;
      this.captionDirSelect.value = String(dir);
      this.captionSpacingInput.value = hwp16ToMm(tp.captionSpacing ?? 0).toFixed(1);
      this.captionWidthInput.value = hwpunitToMm(tp.captionWidth ?? 0).toFixed(1);
      // 캡션 위치 아이콘 활성화 (dir + sub 매칭)
      const activeBtn = this.captionPosBtns.find(b =>
        b.dataset.dir === String(dir) && b.dataset.sub === String(va));
      if (activeBtn) activeBtn.classList.add('active');
      this.captionFieldsWrap.classList.remove('tcp-disabled');
    } else {
      // 가운데(캡션 없음) 버튼 활성화
      const noneBtn = this.captionPosBtns.find(b => b.dataset.dir === '-1');
      if (noneBtn) noneBtn.classList.add('active');
      this.captionFieldsWrap.classList.add('tcp-disabled');
    }
    this.updateCaptionWidthState();

    // 테두리 탭 (table 모드에서만 존재)
    if (this.borderCellSpacingInput) {
      this.borderCellSpacingInput.value = hwp16ToMm(tp.cellSpacing).toFixed(1);
      this.populateBorderFromTarget();
    }

    // 배경 탭 (table 모드에서만 존재)
    if (this.bgNoneRadio) {
      this.populateBgFromTarget();
    }
  }

  protected onConfirm(): void {
    const { sec, ppi, ci } = this.tableCtx;

    // 셀 속성 수정
    const newCellProps: Record<string, unknown> = {};
    if (this.cellApplySizeCheck.checked) {
      newCellProps.width = mmToHwpunit(parseFloat(this.cellWidthInput.value) || 0);
      newCellProps.height = mmToHwpunit(parseFloat(this.cellHeightInput.value) || 0);
    }
    newCellProps.applyInnerMargin = this.cellPaddingCheck.checked;
    if (this.cellPaddingCheck.checked) {
      newCellProps.paddingLeft = mmToHwp16(parseFloat(this.cellPaddingInputs['left'].value) || 0);
      newCellProps.paddingRight = mmToHwp16(parseFloat(this.cellPaddingInputs['right'].value) || 0);
      newCellProps.paddingTop = mmToHwp16(parseFloat(this.cellPaddingInputs['top'].value) || 0);
      newCellProps.paddingBottom = mmToHwp16(parseFloat(this.cellPaddingInputs['bottom'].value) || 0);
    }
    const activeVAlign = this.cellVAlignBtns.findIndex(b => b.classList.contains('active'));
    if (activeVAlign >= 0) newCellProps.verticalAlign = activeVAlign;
    const activeTextDir = this.cellTextDirBtns.findIndex(b => b.classList.contains('active'));
    if (activeTextDir >= 0) newCellProps.textDirection = activeTextDir;
    newCellProps.isHeader = this.cellHeaderCheck.checked;
    newCellProps.cellProtect = this.cellProtectCheck.checked;
    newCellProps.fieldName = this.cellFieldNameInput.value;
    newCellProps.editableInForm = this.cellEditableCheck.checked;

    // 셀 테두리/배경 (cell 모드에서는 테두리/배경 탭이 없으므로 스킵)
    if (this.mode === 'table' && this.borderTarget === 'cell' && this.borderEdits) {
      newCellProps.borderLeft = this.borderEdits[0];
      newCellProps.borderRight = this.borderEdits[1];
      newCellProps.borderTop = this.borderEdits[2];
      newCellProps.borderBottom = this.borderEdits[3];
    }
    if (this.mode === 'table' && this.bgTarget === 'cell' && this.bgColorRadio) {
      if (this.bgColorRadio.checked) {
        newCellProps.fillType = 'solid';
        newCellProps.fillColor = this.bgColorPicker.value;
        newCellProps.patternColor = this.bgPatternColorPicker.value;
        newCellProps.patternType = parseInt(this.bgPatternTypeSelect.value, 10);
      } else {
        newCellProps.fillType = 'none';
      }
    }

    // 표 속성 수정
    const pbValue = parseInt(this.tablePageBreakSelect.value, 10);
    const newTableProps: Record<string, unknown> = {
      treatAsChar: this.treatAsCharCheck.checked,
      textWrap: this.getSelectedWrap(),
      vertRelTo: this.vertRelSelect.value,
      vertAlign: this.vertAlignSelect.value,
      vertOffset: mmToHwpunit(parseFloat(this.vertOffsetInput.value) || 0),
      horzRelTo: this.horzRelSelect.value,
      horzAlign: this.horzAlignSelect.value,
      horzOffset: mmToHwpunit(parseFloat(this.horzOffsetInput.value) || 0),
      restrictInPage: this.restrictInPageCheck.checked,
      allowOverlap: this.allowOverlapCheck.checked,
      keepWithAnchor: this.keepWithAnchorCheck.checked,
      pageBreak: pbValue,
      repeatHeader: this.tableRepeatHeaderCheck.checked,
      paddingLeft: mmToHwp16(parseFloat(this.tablePaddingInputs['left'].value) || 0),
      paddingRight: mmToHwp16(parseFloat(this.tablePaddingInputs['right'].value) || 0),
      paddingTop: mmToHwp16(parseFloat(this.tablePaddingInputs['top'].value) || 0),
      paddingBottom: mmToHwp16(parseFloat(this.tablePaddingInputs['bottom'].value) || 0),
      cellSpacing: this.borderCellSpacingInput ? mmToHwp16(parseFloat(this.borderCellSpacingInput.value) || 0) : this.tableProps.cellSpacing,
      // 바깥 여백
      outerLeft: mmToHwp16(parseFloat(this.marginOuterInputs['left'].value) || 0),
      outerRight: mmToHwp16(parseFloat(this.marginOuterInputs['right'].value) || 0),
      outerTop: mmToHwp16(parseFloat(this.marginOuterInputs['top'].value) || 0),
      outerBottom: mmToHwp16(parseFloat(this.marginOuterInputs['bottom'].value) || 0),
    };

    // 캡션 속성 (가운데 = 캡션 없음)
    const activeCapBtn = this.captionPosBtns.find(b => b.classList.contains('active'));
    const capDir = activeCapBtn ? parseInt(activeCapBtn.dataset.dir!, 10) : -1;
    newTableProps.hasCaption = capDir !== -1;
    if (capDir !== -1) {
      newTableProps.captionDirection = parseInt(this.captionDirSelect.value, 10);
      const activeCapBtn = this.captionPosBtns.find(b => b.classList.contains('active'));
      newTableProps.captionVertAlign = activeCapBtn ? parseInt(activeCapBtn.dataset.sub!, 10) : 0;
      newTableProps.captionSpacing = mmToHwp16(parseFloat(this.captionSpacingInput.value) || 0);
      newTableProps.captionWidth = mmToHwpunit(parseFloat(this.captionWidthInput.value) || 0);
    }

    // 표 테두리/배경 (table 모드에서만 테두리/배경 탭 존재)
    if (this.mode === 'table' && this.borderTarget === 'table' && this.borderEdits) {
      newTableProps.borderLeft = this.borderEdits[0];
      newTableProps.borderRight = this.borderEdits[1];
      newTableProps.borderTop = this.borderEdits[2];
      newTableProps.borderBottom = this.borderEdits[3];
    }
    if (this.mode === 'table' && this.bgTarget === 'table' && this.bgColorRadio) {
      if (this.bgColorRadio.checked) {
        newTableProps.fillType = 'solid';
        newTableProps.fillColor = this.bgColorPicker.value;
        newTableProps.patternColor = this.bgPatternColorPicker.value;
        newTableProps.patternType = parseInt(this.bgPatternTypeSelect.value, 10);
      } else {
        newTableProps.fillType = 'none';
      }
    }

    const applyProps = () => {
      this.wasm.setCellProperties(sec, ppi, ci, this.cellIdx, newCellProps as Partial<CellProperties>);
      this.wasm.setTableProperties(sec, ppi, ci, newTableProps as Partial<TableProperties>);
    };
    // 표/셀 속성 변경도 undo 대상이다 — 편집 라우터를 통과시켜 스냅샷으로
    // 기록한다 (#1320 계약, picture-props-dialog(#2027)와 동일 패턴).
    // services 미주입 환경에서만 직접 적용 fallback.
    const ih = this.services?.getInputHandler();
    if (ih) {
      ih.executeOperation({
        kind: 'snapshot',
        operationType: 'objectProps',
        operation: () => {
          applyProps();
          return ih.getCursorPosition();
        },
      });
    } else {
      applyProps();
      this.eventBus.emit('document-changed');
    }
  }

  // ─── "모두(A)" 일괄 여백 스피너 ─────────────────────

  /** 4방향 여백 입력을 일괄 조정하는 "모두(A)" 스피너 생성 */
  private buildAllSpinner(inputs: Record<string, HTMLInputElement>): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'tcp-all-spinner';
    const lbl = this.label('모두(A)');
    wrap.appendChild(lbl);

    const setAll = (delta: number) => {
      for (const inp of Object.values(inputs)) {
        const cur = parseFloat(inp.value) || 0;
        inp.value = Math.max(0, cur + delta).toFixed(1);
      }
    };

    const upBtn = document.createElement('button');
    upBtn.type = 'button';
    upBtn.className = 'tcp-all-spinner-btn';
    upBtn.textContent = '▲';
    upBtn.addEventListener('click', () => setAll(0.5));

    const downBtn = document.createElement('button');
    downBtn.type = 'button';
    downBtn.className = 'tcp-all-spinner-btn';
    downBtn.textContent = '▼';
    downBtn.addEventListener('click', () => setAll(-0.5));

    wrap.appendChild(upBtn);
    wrap.appendChild(downBtn);

    return wrap;
  }

  // ─── DOM 헬퍼 ────────────────────────────────────

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

  private numberInput(): HTMLInputElement {
    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'dialog-input';
    inp.step = '0.1';
    inp.min = '0';
    return inp;
  }

  private unit(text: string): HTMLSpanElement {
    const u = document.createElement('span');
    u.className = 'dialog-unit';
    u.textContent = text;
    return u;
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

  private setButtonGroupActive(btns: HTMLButtonElement[], idx: number): void {
    btns.forEach((b, i) => b.classList.toggle('active', i === idx));
  }
}
