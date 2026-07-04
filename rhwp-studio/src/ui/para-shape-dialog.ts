/**
 * 문단 모양 대화상자 (ParaShapeDialog)
 * HWP Alt+T 대화상자에 해당하는 문단 속성 설정 UI
 *
 * 레이아웃: 한컴 문단모양 대화상자를 최대한 재현
 *  ┌─────────────────────────────────────────────┐
 *  │ 문단 모양                              [×]  │
 *  ├────────────────────────────────┬─────────────┤
 *  │ [기본] [확장] [탭 설정] [테두리/배경]│ [설정(D)] │
 *  │                                │  [취  소]   │
 *  │ ─ 정렬 방식 ─────────────────  │             │
 *  │ [양쪽][왼쪽][오른쪽][가운데][배분][나눔] │   │
 *  │                                │             │
 *  │ ─ 여백 ───── ─ 첫 줄 ──────  │             │
 *  │ 왼쪽: [0.0]pt  ○보통           │             │
 *  │ 오른쪽:[0.0]pt  ○들여쓰기 [v]pt │             │
 *  │                 ○내어쓰기       │             │
 *  │ ─ 간격 ────────────────────── │             │
 *  │ 줄 간격: [글자에 따라 v] [160]% │             │
 *  │ 문단 위: [0.0]pt  문단 아래:[0.0]pt│           │
 *  │ ┌─────────────────────────┐   │             │
 *  │ │  [미리보기 영역]          │   │             │
 *  │ └─────────────────────────┘   │             │
 *  └────────────────────────────────┴─────────────┘
 */

import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { ParaProperties } from '@/core/types';
import { createFieldset, row, label, numberInput, unit } from './para-shape-helpers';
import {
  buildTabSettingsTab, buildBorderTab,
  type TabState, type TabSettingsResult, type BorderTabResult,
} from './para-shape-tab-builders';
import { enableDialogDrag } from './dialog-drag';

/** 정렬 아이콘 (SVG 아이콘 — 서식바와 동일) */
const ALIGN_OPTIONS: { value: string; label: string; cssClass: string }[] = [
  { value: 'justify',    label: '양쪽 정렬',   cssClass: 'sb-al-justify' },
  { value: 'left',       label: '왼쪽 정렬',   cssClass: 'sb-al-left' },
  { value: 'right',      label: '오른쪽 정렬',  cssClass: 'sb-al-right' },
  { value: 'center',     label: '가운데 정렬',  cssClass: 'sb-al-center' },
  { value: 'distribute', label: '배분 정렬',   cssClass: 'sb-al-distribute' },
  { value: 'split',      label: '나눔 정렬',   cssClass: 'sb-al-split' },
];

const LINE_SPACING_TYPES: { value: string; label: string }[] = [
  { value: 'Percent',   label: '글자에 따라' },
  { value: 'Fixed',     label: '고정 값' },
  { value: 'SpaceOnly', label: '여백만 지정' },
  { value: 'Minimum',   label: '최소' },
];

// ─── 단위 변환 ─────────────────────────────────
// WASM API (build_para_properties_json) 출력값은 ResolvedParaStyle 기반 px (96dpi).
// 대화상자 표시: px → pt
// 적용(Rust apply): pt → HWPUNIT (raw)

const HWPUNIT_PER_PT = 100;  // 1pt = 100 HWPUNIT

/** px (96dpi, zoom=1) → pt 표시값 */
function pxToPt(px: number): number {
  return px * 72 / 96;
}

/** pt → raw HWPUNIT (2x 저장값) — 여백/들여쓰기 적용용 */
function ptToRaw2x(pt: number): number {
  return Math.round(pt * HWPUNIT_PER_PT * 2);
}

/** pt → raw HWPUNIT (1x) — spacing/lineSpacing 적용용 */
function ptToRaw(pt: number): number {
  return Math.round(pt * HWPUNIT_PER_PT);
}

/** px → raw HWPUNIT (2x) — 비교용 */
function pxToRaw2x(px: number): number {
  return Math.round(px * 150);  // px * 72/96 * 100 * 2
}

/** px → raw HWPUNIT (1x) — 비교용 */
function pxToRaw(px: number): number {
  return Math.round(px * 75);   // px * 72/96 * 100
}

export class ParaShapeDialog {
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private built = false;

  // 탭
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];

  // 기본 탭 컨트롤
  private alignBtns: Record<string, HTMLButtonElement> = {};
  private marginLeftInput!: HTMLInputElement;
  private marginRightInput!: HTMLInputElement;
  private firstLineRadios!: HTMLInputElement[];   // 보통/들여쓰기/내어쓰기
  private indentInput!: HTMLInputElement;
  private lineSpacingTypeSelect!: HTMLSelectElement;
  private lineSpacingInput!: HTMLInputElement;
  private lineSpacingUnitLabel!: HTMLSpanElement;
  private spacingBeforeInput!: HTMLInputElement;
  private spacingAfterInput!: HTMLInputElement;
  private previewEl!: HTMLDivElement;

  // 확장 탭 컨트롤
  private headTypeRadios!: HTMLInputElement[];     // 없음/개요/번호/글머리표
  private paraLevelSelect!: HTMLSelectElement;
  private widowOrphanCb!: HTMLInputElement;
  private keepWithNextCb!: HTMLInputElement;
  private keepLinesCb!: HTMLInputElement;
  private pageBreakBeforeCb!: HTMLInputElement;
  private fontLineHeightCb!: HTMLInputElement;
  private singleLineCb!: HTMLInputElement;
  private autoSpaceKrEnCb!: HTMLInputElement;
  private autoSpaceKrNumCb!: HTMLInputElement;
  private verticalAlignSelect!: HTMLSelectElement;
  private englishBreakSelect!: HTMLSelectElement;
  private koreanBreakSelect!: HTMLSelectElement;

  // 탭 설정 탭 (빌더 결과)
  private tabResult!: TabSettingsResult;
  private tabState: TabState = { currentTabStops: [], deletedTabStops: [], selectedTabIndex: -1 };

  // 테두리/배경 탭 (빌더 결과)
  private borderResult!: BorderTabResult;
  private borderStates = {
    left:   { type: 0, width: 0, color: '#000000' },
    right:  { type: 0, width: 0, color: '#000000' },
    top:    { type: 0, width: 0, color: '#000000' },
    bottom: { type: 0, width: 0, color: '#000000' },
  };
  private bdSideToggles = { left: false, right: false, top: false, bottom: false };

  // 상태
  private props: ParaProperties | null = null;
  private initialProps: ParaProperties | null = null;

  /** 적용 콜백 */
  onApply: ((mods: Partial<ParaProperties>) => void) | null = null;
  /** 대화상자 닫힘 콜백 (적용·취소·ESC 모두) */
  onClose: (() => void) | null = null;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
  ) {}

  show(paraProps: ParaProperties): void {
    this.build();
    this.props = JSON.parse(JSON.stringify(paraProps));
    this.initialProps = JSON.parse(JSON.stringify(paraProps));
    this.populateFromProps();
    this.switchTab(0);
    document.body.appendChild(this.overlay);
  }

  hide(): void {
    this.overlay?.remove();
    if (this.onClose) this.onClose();
  }

  // ════════════════════════════════════════════════════════
  //  빌드
  // ════════════════════════════════════════════════════════

  private build(): void {
    if (this.built) return;
    this.built = true;

    // ── 오버레이
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    // ── 다이얼로그 컨테이너
    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap ps-dialog';

    // 타이틀 바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '문단 모양';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);

    // ── 메인 레이아웃: 좌측(탭+컨텐츠) + 우측(버튼)
    const mainRow = document.createElement('div');
    mainRow.className = 'cs-main-row';

    // 좌측 영역
    const leftCol = document.createElement('div');
    leftCol.className = 'cs-left-col';

    // 탭 그룹
    const tabGroup = document.createElement('div');
    tabGroup.className = 'dialog-tabs';
    const tabNames = ['기본', '확장', '탭 설정', '테두리/배경'];
    tabNames.forEach((name, i) => {
      const btn = document.createElement('button');
      btn.className = 'dialog-tab';
      btn.textContent = name;
      btn.addEventListener('click', () => this.switchTab(i));
      tabGroup.appendChild(btn);
      this.tabs.push(btn);
    });
    leftCol.appendChild(tabGroup);

    // 탭 패널 컨테이너
    const body = document.createElement('div');
    body.className = 'dialog-body';
    this.panels.push(this.buildBasicPanel());
    this.panels.push(this.buildExtendedPanel());
    this.panels.push(this.buildTabSettingsPanel());
    this.panels.push(this.buildBorderPanel());
    this.panels.forEach(p => body.appendChild(p));
    leftCol.appendChild(body);

    // 우측 버튼 영역
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
    rightCol.appendChild(okBtn);
    rightCol.appendChild(cancelBtn);

    mainRow.appendChild(leftCol);
    mainRow.appendChild(rightCol);
    this.dialog.appendChild(mainRow);

    this.overlay.appendChild(this.dialog);

    // Escape
    this.overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') { e.stopPropagation(); this.hide(); }
    });

    enableDialogDrag(this.dialog, titleBar);
  }

  private switchTab(idx: number): void {
    this.tabs.forEach((t, i) => t.classList.toggle('active', i === idx));
    this.panels.forEach((p, i) => p.classList.toggle('active', i === idx));
  }

  // ════════════════════════════════════════════════════════
  //  기본 탭
  // ════════════════════════════════════════════════════════

  private buildBasicPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 정렬 방식
    const alignFs = createFieldset('정렬 방식');
    const alignRow = document.createElement('div');
    alignRow.className = 'ps-align-row';
    ALIGN_OPTIONS.forEach(opt => {
      const btn = document.createElement('button');
      btn.className = 'ps-align-btn';
      btn.title = opt.label;
      const icon = document.createElement('span');
      icon.className = `sb-align ${opt.cssClass}`;
      btn.appendChild(icon);
      btn.addEventListener('click', () => {
        Object.values(this.alignBtns).forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        this.updatePreview();
      });
      this.alignBtns[opt.value] = btn;
      alignRow.appendChild(btn);
    });
    alignFs.appendChild(alignRow);
    panel.appendChild(alignFs);

    // ── 여백 + 첫 줄 (2열 배치)
    const marginFirstRow = document.createElement('div');
    marginFirstRow.className = 'ps-two-col';

    // 여백 (좌측)
    const marginFs = createFieldset('여백');
    const mlRow = row();
    const mlLabel = label('왼쪽(E):');
    mlLabel.style.minWidth = '62px';
    mlLabel.style.textAlign = 'right';
    mlRow.appendChild(mlLabel);
    this.marginLeftInput = numberInput(0, 999, 0.1);
    this.marginLeftInput.style.width = '60px';
    mlRow.appendChild(this.marginLeftInput);
    mlRow.appendChild(unit('pt'));
    marginFs.appendChild(mlRow);

    const mrRow = row();
    const mrLabel = label('오른쪽(O):');
    mrLabel.style.minWidth = '62px';
    mrLabel.style.textAlign = 'right';
    mrRow.appendChild(mrLabel);
    this.marginRightInput = numberInput(0, 999, 0.1);
    this.marginRightInput.style.width = '60px';
    mrRow.appendChild(this.marginRightInput);
    mrRow.appendChild(unit('pt'));
    marginFs.appendChild(mrRow);

    // 첫 줄 (우측)
    const firstLineFs = createFieldset('첫 줄');
    this.firstLineRadios = [];

    const normalRow = row();
    const normalRadio = document.createElement('input');
    normalRadio.type = 'radio';
    normalRadio.name = 'ps-first-line';
    normalRadio.value = 'normal';
    this.firstLineRadios.push(normalRadio);
    normalRow.appendChild(normalRadio);
    normalRow.appendChild(label('보통(N)'));
    firstLineFs.appendChild(normalRow);

    const indentRow = row();
    indentRow.style.whiteSpace = 'nowrap';
    const indentRadio = document.createElement('input');
    indentRadio.type = 'radio';
    indentRadio.name = 'ps-first-line';
    indentRadio.value = 'indent';
    this.firstLineRadios.push(indentRadio);
    indentRow.appendChild(indentRadio);
    const indentLabel = label('들여쓰기(A)');
    indentLabel.style.whiteSpace = 'nowrap';
    indentRow.appendChild(indentLabel);
    this.indentInput = numberInput(0, 999, 0.1);
    this.indentInput.style.width = '50px';
    indentRow.appendChild(this.indentInput);
    indentRow.appendChild(unit('pt'));
    firstLineFs.appendChild(indentRow);

    const hangRow = row();
    const hangRadio = document.createElement('input');
    hangRadio.type = 'radio';
    hangRadio.name = 'ps-first-line';
    hangRadio.value = 'hanging';
    this.firstLineRadios.push(hangRadio);
    hangRow.appendChild(hangRadio);
    const hangLabel = label('내어쓰기(B)');
    hangLabel.style.whiteSpace = 'nowrap';
    hangRow.appendChild(hangLabel);
    firstLineFs.appendChild(hangRow);

    // 라디오 변경 시 indentInput 활성/비활성
    this.firstLineRadios.forEach(r => {
      r.addEventListener('change', () => {
        this.indentInput.disabled = (r.value === 'normal' && r.checked);
        this.updatePreview();
      });
    });

    marginFirstRow.appendChild(marginFs);
    marginFirstRow.appendChild(firstLineFs);
    panel.appendChild(marginFirstRow);

    // ── 간격
    const spacingFs = createFieldset('간격');

    const lsRow = row();
    lsRow.appendChild(label('줄 간격(S):'));
    this.lineSpacingTypeSelect = document.createElement('select');
    this.lineSpacingTypeSelect.className = 'dialog-select';
    this.lineSpacingTypeSelect.style.width = '100px';
    LINE_SPACING_TYPES.forEach(opt => {
      const o = document.createElement('option');
      o.value = opt.value;
      o.textContent = opt.label;
      this.lineSpacingTypeSelect.appendChild(o);
    });
    this.lineSpacingTypeSelect.addEventListener('change', () => this.onLineSpacingTypeChange());
    lsRow.appendChild(this.lineSpacingTypeSelect);
    this.lineSpacingInput = numberInput(0, 9999, 1);
    this.lineSpacingInput.style.width = '55px';
    this.lineSpacingInput.style.marginLeft = '6px';
    lsRow.appendChild(this.lineSpacingInput);
    this.lineSpacingUnitLabel = unit('%');
    lsRow.appendChild(this.lineSpacingUnitLabel);
    spacingFs.appendChild(lsRow);

    const paraSpRow = row();
    paraSpRow.appendChild(label('문단 위(U):'));
    this.spacingBeforeInput = numberInput(0, 999, 0.1);
    this.spacingBeforeInput.style.width = '55px';
    paraSpRow.appendChild(this.spacingBeforeInput);
    paraSpRow.appendChild(unit('pt'));
    const afterLabel = label('문단 아래(V):');
    afterLabel.style.marginLeft = '12px';
    paraSpRow.appendChild(afterLabel);
    this.spacingAfterInput = numberInput(0, 999, 0.1);
    this.spacingAfterInput.style.width = '55px';
    paraSpRow.appendChild(this.spacingAfterInput);
    paraSpRow.appendChild(unit('pt'));
    spacingFs.appendChild(paraSpRow);

    panel.appendChild(spacingFs);

    // ── 미리보기
    this.previewEl = document.createElement('div');
    this.previewEl.className = 'ps-preview';
    this.previewEl.textContent = '미리보기';
    panel.appendChild(this.previewEl);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  확장 탭 (스텁)
  // ════════════════════════════════════════════════════════

  private buildExtendedPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 문단 종류 섹션
    const typeSection = document.createElement('div');
    typeSection.className = 'dialog-section';
    const typeTitle = document.createElement('div');
    typeTitle.className = 'dialog-section-title';
    typeTitle.textContent = '문단 종류';
    typeSection.appendChild(typeTitle);

    this.headTypeRadios = [];
    const headTypes: [string, string][] = [
      ['None', '없음(O)'], ['Outline', '개요 문단(U)'],
      ['Number', '번호 문단(M)'], ['Bullet', '글머리표 문단(B)'],
    ];

    // 수준 드롭다운 (개요/번호 선택 시만 활성)
    this.paraLevelSelect = document.createElement('select');
    this.paraLevelSelect.className = 'dialog-select';
    this.paraLevelSelect.style.width = '80px';
    this.paraLevelSelect.disabled = true;
    for (let i = 0; i < 7; i++) {
      const opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = `${i + 1} 수준`;
      this.paraLevelSelect.appendChild(opt);
    }

    headTypes.forEach(([val, label]) => {
      const row = document.createElement('div');
      row.className = 'dialog-row';
      row.style.padding = '1px 0';
      const lbl = document.createElement('label');
      lbl.className = 'dialog-radio-group';
      const radio = document.createElement('input');
      radio.type = 'radio';
      radio.name = 'ps-head-type';
      radio.value = val;
      radio.addEventListener('change', () => {
        this.paraLevelSelect.disabled = (val === 'None' || val === 'Bullet');
      });
      lbl.appendChild(radio);
      lbl.appendChild(document.createTextNode(` ${label}`));
      row.appendChild(lbl);
      // 개요 문단 옆에 수준 드롭다운 배치
      if (val === 'Outline') {
        const span = document.createElement('span');
        span.style.marginLeft = '12px';
        span.appendChild(document.createTextNode('수준(L): '));
        span.appendChild(this.paraLevelSelect);
        row.appendChild(span);
      }
      typeSection.appendChild(row);
      this.headTypeRadios.push(radio);
    });
    panel.appendChild(typeSection);

    // ── 기타 섹션
    const etcSection = document.createElement('div');
    etcSection.className = 'dialog-section';
    const etcTitle = document.createElement('div');
    etcTitle.className = 'dialog-section-title';
    etcTitle.textContent = '기타';
    etcSection.appendChild(etcTitle);

    const makeCb = (label: string): HTMLInputElement => {
      const row = document.createElement('div');
      row.className = 'dialog-row';
      row.style.padding = '1px 0';
      const lbl = document.createElement('label');
      lbl.className = 'dialog-checkbox';
      const cb = document.createElement('input');
      cb.type = 'checkbox';
      lbl.appendChild(cb);
      lbl.appendChild(document.createTextNode(` ${label}`));
      row.appendChild(lbl);
      etcSection.appendChild(row);
      return cb;
    };

    this.widowOrphanCb = makeCb('외톨이줄 보호(K)');
    this.keepWithNextCb = makeCb('다음 문단과 함께(N)');
    this.keepLinesCb = makeCb('문단 보호(P)');
    this.pageBreakBeforeCb = makeCb('문단 앞에서 항상 쪽 나눔(E)');
    this.fontLineHeightCb = makeCb('글꼴에 어울리는 줄 높이(H)');
    this.singleLineCb = makeCb('한 줄로 입력(W)');
    this.autoSpaceKrEnCb = makeCb('한글과 영어 간격을 자동 조절(G)');
    this.autoSpaceKrNumCb = makeCb('한글과 숫자 간격을 자동 조절(R)');

    // 세로 정렬
    const vaRow = document.createElement('div');
    vaRow.className = 'dialog-row';
    vaRow.style.padding = '2px 0';
    const vaLabel = document.createElement('label');
    vaLabel.className = 'dialog-label';
    vaLabel.textContent = '세로 정렬(S):';
    vaLabel.style.marginRight = '8px';
    this.verticalAlignSelect = document.createElement('select');
    this.verticalAlignSelect.className = 'dialog-select';
    this.verticalAlignSelect.style.width = '100px';
    const vaOptions: [string, string][] = [
      ['0', '글꼴 기준'], ['1', '위쪽'], ['2', '가운데'], ['3', '아래쪽'],
    ];
    vaOptions.forEach(([v, t]) => {
      const opt = document.createElement('option');
      opt.value = v;
      opt.textContent = t;
      this.verticalAlignSelect.appendChild(opt);
    });
    vaRow.appendChild(vaLabel);
    vaRow.appendChild(this.verticalAlignSelect);
    etcSection.appendChild(vaRow);

    // ── 줄바꿈 기준 ──────────────────────
    const breakFs = createFieldset('줄 나눔 기준');

    const krRow = row();
    krRow.appendChild(label('한글(K):'));
    this.koreanBreakSelect = document.createElement('select');
    this.koreanBreakSelect.className = 'dialog-select';
    this.koreanBreakSelect.style.width = '100px';
    for (const [v, t] of [['0', '어절'], ['1', '글자']] as const) {
      const o = document.createElement('option');
      o.value = v; o.textContent = t;
      this.koreanBreakSelect.appendChild(o);
    }
    krRow.appendChild(this.koreanBreakSelect);
    breakFs.appendChild(krRow);

    const enRow = row();
    enRow.appendChild(label('영어(E):'));
    this.englishBreakSelect = document.createElement('select');
    this.englishBreakSelect.className = 'dialog-select';
    this.englishBreakSelect.style.width = '100px';
    for (const [v, t] of [['0', '단어'], ['1', '하이픈'], ['2', '글자']] as const) {
      const o = document.createElement('option');
      o.value = v; o.textContent = t;
      this.englishBreakSelect.appendChild(o);
    }
    enRow.appendChild(this.englishBreakSelect);
    breakFs.appendChild(enRow);

    panel.appendChild(breakFs);

    panel.appendChild(etcSection);
    return panel;
  }

  private buildTabSettingsPanel(): HTMLDivElement {
    this.tabResult = buildTabSettingsTab(this.tabState);
    return this.tabResult.panel;
  }

  private buildBorderPanel(): HTMLDivElement {
    this.borderResult = buildBorderTab(this.borderStates, this.bdSideToggles);
    return this.borderResult.panel;
  }

  // ════════════════════════════════════════════════════════
  //  줄 간격 종류 변경
  // ════════════════════════════════════════════════════════

  private onLineSpacingTypeChange(): void {
    const type = this.lineSpacingTypeSelect.value;
    if (type === 'Percent') {
      this.lineSpacingUnitLabel.textContent = '%';
      this.lineSpacingInput.step = '1';
    } else {
      this.lineSpacingUnitLabel.textContent = 'pt';
      this.lineSpacingInput.step = '0.1';
    }
    this.updatePreview();
  }

  // ════════════════════════════════════════════════════════
  //  속성 채우기 + 미리보기
  // ════════════════════════════════════════════════════════

  private populateFromProps(): void {
    const p = this.props;
    if (!p) return;

    // 정렬
    Object.values(this.alignBtns).forEach(b => b.classList.remove('active'));
    const align = p.alignment || 'justify';
    if (this.alignBtns[align]) this.alignBtns[align].classList.add('active');

    // 여백 (px → pt)
    this.marginLeftInput.value = pxToPt(p.marginLeft ?? 0).toFixed(1);
    this.marginRightInput.value = pxToPt(p.marginRight ?? 0).toFixed(1);

    // 첫 줄 (indent)
    const indent = p.indent ?? 0;
    if (indent > 0) {
      this.firstLineRadios[1].checked = true; // 들여쓰기
      this.indentInput.value = pxToPt(indent).toFixed(1);
      this.indentInput.disabled = false;
    } else if (indent < 0) {
      this.firstLineRadios[2].checked = true; // 내어쓰기
      this.indentInput.value = pxToPt(Math.abs(indent)).toFixed(1);
      this.indentInput.disabled = false;
    } else {
      this.firstLineRadios[0].checked = true; // 보통
      this.indentInput.value = '0.0';
      this.indentInput.disabled = true;
    }

    // 줄 간격
    const lsType = p.lineSpacingType || 'Percent';
    this.lineSpacingTypeSelect.value = lsType;
    if (lsType === 'Percent') {
      this.lineSpacingInput.value = String(p.lineSpacing ?? 160);
      this.lineSpacingUnitLabel.textContent = '%';
      this.lineSpacingInput.step = '1';
    } else {
      this.lineSpacingInput.value = pxToPt(p.lineSpacing ?? 0).toFixed(1);
      this.lineSpacingUnitLabel.textContent = 'pt';
      this.lineSpacingInput.step = '0.1';
    }

    // 문단 간격 (px → pt)
    this.spacingBeforeInput.value = pxToPt(p.spacingBefore ?? 0).toFixed(1);
    this.spacingAfterInput.value = pxToPt(p.spacingAfter ?? 0).toFixed(1);

    // ── 확장 탭
    const ht = p.headType || 'None';
    this.headTypeRadios.forEach(r => { r.checked = r.value === ht; });
    this.paraLevelSelect.value = String(p.paraLevel ?? 0);
    this.paraLevelSelect.disabled = (ht === 'None' || ht === 'Bullet');
    this.widowOrphanCb.checked = p.widowOrphan ?? false;
    this.keepWithNextCb.checked = p.keepWithNext ?? false;
    this.keepLinesCb.checked = p.keepLines ?? false;
    this.pageBreakBeforeCb.checked = p.pageBreakBefore ?? false;
    this.fontLineHeightCb.checked = p.fontLineHeight ?? false;
    this.singleLineCb.checked = p.singleLine ?? false;
    this.autoSpaceKrEnCb.checked = p.autoSpaceKrEn ?? false;
    this.autoSpaceKrNumCb.checked = p.autoSpaceKrNum ?? false;
    this.verticalAlignSelect.value = String(p.verticalAlign ?? 0);
    this.koreanBreakSelect.value = String(p.koreanBreakUnit ?? 0);
    this.englishBreakSelect.value = String(p.englishBreakUnit ?? 0);

    // ── 탭 설정 탭
    this.tabState.currentTabStops = (p.tabStops ?? []).map(t => ({ ...t }));
    this.tabState.deletedTabStops = [];
    this.tabState.selectedTabIndex = -1;
    this.tabResult.tabAutoLeftCb.checked = p.tabAutoLeft ?? false;
    this.tabResult.tabAutoRightCb.checked = p.tabAutoRight ?? false;
    const defSpacing = p.defaultTabSpacing ?? 8000;
    this.tabResult.defaultTabLabel.textContent = `${(defSpacing / 200).toFixed(1)} pt`;
    this.tabResult.renderTabList();
    this.tabResult.renderDeletedTabList();

    // ── 테두리/배경 탭
    const HWPUNIT_PER_MM = 7200 / 25.4; // ≈ 283.46
    const defBorder = { type: 0, width: 0, color: '#000000' };
    for (const side of ['left', 'right', 'top', 'bottom'] as const) {
      const key = `border${side.charAt(0).toUpperCase() + side.slice(1)}` as keyof ParaProperties;
      const val = p[key] as { type: number; width: number; color: string } | undefined;
      this.borderStates[side] = val ? { ...val } : { ...defBorder };
      this.bdSideToggles[side] = val ? val.type !== 0 : false;
    }
    // 배경
    this.borderResult.bgFillSelect.value = p.fillType === 'solid' ? 'solid' : 'none';
    this.borderResult.bgFillPicker.value = p.fillColor || '#ffffff';
    this.borderResult.bgPatColorInput.value = p.patternColor || '#000000';
    this.borderResult.bgPatShapeSelect.value = String(p.patternType ?? 0);
    // 간격 (HWPUNIT → mm)
    const sp = p.borderSpacing ?? [0, 0, 0, 0];
    // bdSpacingInputs: [0]=left, [1]=top, [2]=right, [3]=bottom
    this.borderResult.bdSpacingInputs[0].value = (sp[0] / HWPUNIT_PER_MM).toFixed(2);
    this.borderResult.bdSpacingInputs[1].value = (sp[2] / HWPUNIT_PER_MM).toFixed(2);
    this.borderResult.bdSpacingInputs[2].value = (sp[1] / HWPUNIT_PER_MM).toFixed(2);
    this.borderResult.bdSpacingInputs[3].value = (sp[3] / HWPUNIT_PER_MM).toFixed(2);
    this.borderResult.bdConnectCb.checked = p.borderConnect ?? false;
    this.borderResult.bdIgnoreMarginCb.checked = p.borderIgnoreMargin ?? false;
    this.borderResult.updateBdPreview();

    this.updatePreview();
  }

  private updatePreview(): void {
    // 현재 설정을 반영한 간단한 텍스트 미리보기
    const align = this.getSelectedAlignment();
    const alignMap: Record<string, string> = {
      justify: 'justify', left: 'left', right: 'right',
      center: 'center', distribute: 'justify', split: 'justify',
    };
    const textAlign = alignMap[align] || 'justify';
    const ml = parseFloat(this.marginLeftInput.value) || 0;
    const mr = parseFloat(this.marginRightInput.value) || 0;

    let indent = 0;
    const checkedRadio = this.firstLineRadios.find(r => r.checked);
    if (checkedRadio?.value === 'indent') indent = parseFloat(this.indentInput.value) || 0;
    else if (checkedRadio?.value === 'hanging') indent = -(parseFloat(this.indentInput.value) || 0);

    this.previewEl.replaceChildren();
    const sampleLines = [
      '이것은 문단 미리보기입니다. 이렇게 문단의 정렬과 여백, 들여쓰기가 적용된 모습을 확인할 수 있습니다.',
      '두 번째 줄은 보통 여백만 적용됩니다.',
    ];
    sampleLines.forEach((text, i) => {
      const p = document.createElement('div');
      p.style.textAlign = textAlign;
      p.style.marginLeft = `${ml * 0.8}px`;
      p.style.marginRight = `${mr * 0.8}px`;
      if (i === 0 && indent !== 0) {
        p.style.textIndent = `${indent * 0.8}px`;
      }
      p.style.fontSize = '11px';
      p.style.lineHeight = '1.5';
      p.style.color = '#111111';
      p.textContent = text;
      this.previewEl.appendChild(p);
    });
  }

  // ════════════════════════════════════════════════════════
  //  변경사항 수집
  // ════════════════════════════════════════════════════════

  private collectMods(): Partial<ParaProperties> {
    const mods: Partial<ParaProperties> = {};
    const p = this.initialProps;
    if (!p) return mods;

    // 정렬
    const newAlign = this.getSelectedAlignment();
    if (newAlign !== (p.alignment || 'justify')) {
      mods.alignment = newAlign;
    }

    // 여백 (비교: 원본 px → HWPUNIT 2x 변환 후 비교)
    const newML = ptToRaw2x(parseFloat(this.marginLeftInput.value) || 0);
    if (newML !== pxToRaw2x(p.marginLeft ?? 0)) mods.marginLeft = newML;

    const newMR = ptToRaw2x(parseFloat(this.marginRightInput.value) || 0);
    if (newMR !== pxToRaw2x(p.marginRight ?? 0)) mods.marginRight = newMR;

    // 첫 줄 (indent)
    let newIndent = 0;
    const checkedRadio = this.firstLineRadios.find(r => r.checked);
    if (checkedRadio?.value === 'indent') {
      newIndent = ptToRaw2x(parseFloat(this.indentInput.value) || 0);
    } else if (checkedRadio?.value === 'hanging') {
      newIndent = -ptToRaw2x(parseFloat(this.indentInput.value) || 0);
    }
    if (newIndent !== pxToRaw2x(p.indent ?? 0)) mods.indent = newIndent;

    // 줄 간격
    const newLSType = this.lineSpacingTypeSelect.value;
    if (newLSType !== (p.lineSpacingType || 'Percent')) {
      mods.lineSpacingType = newLSType;
    }

    let newLS: number;
    if (newLSType === 'Percent') {
      newLS = parseInt(this.lineSpacingInput.value) || 160;
    } else {
      newLS = ptToRaw(parseFloat(this.lineSpacingInput.value) || 0);
    }
    // Percent는 raw 비교, 그 외는 px → HWPUNIT 변환 후 비교
    const origLS = (p.lineSpacingType || 'Percent') === 'Percent'
      ? (p.lineSpacing ?? 160)
      : pxToRaw(p.lineSpacing ?? 0);
    if (newLS !== origLS) mods.lineSpacing = newLS;

    // 문단 간격 (비교: 원본 px → HWPUNIT 1x 변환 후 비교)
    const newSB = ptToRaw(parseFloat(this.spacingBeforeInput.value) || 0);
    if (newSB !== pxToRaw(p.spacingBefore ?? 0)) mods.spacingBefore = newSB;

    const newSA = ptToRaw(parseFloat(this.spacingAfterInput.value) || 0);
    if (newSA !== pxToRaw(p.spacingAfter ?? 0)) mods.spacingAfter = newSA;

    // ── 확장 탭
    const newHT = this.headTypeRadios.find(r => r.checked)?.value || 'None';
    if (newHT !== (p.headType || 'None')) mods.headType = newHT;

    const newPL = parseInt(this.paraLevelSelect.value) || 0;
    if (newPL !== (p.paraLevel ?? 0)) mods.paraLevel = newPL;

    const cbFields: [HTMLInputElement, keyof ParaProperties, boolean][] = [
      [this.widowOrphanCb, 'widowOrphan', p.widowOrphan ?? false],
      [this.keepWithNextCb, 'keepWithNext', p.keepWithNext ?? false],
      [this.keepLinesCb, 'keepLines', p.keepLines ?? false],
      [this.pageBreakBeforeCb, 'pageBreakBefore', p.pageBreakBefore ?? false],
      [this.fontLineHeightCb, 'fontLineHeight', p.fontLineHeight ?? false],
      [this.singleLineCb, 'singleLine', p.singleLine ?? false],
      [this.autoSpaceKrEnCb, 'autoSpaceKrEn', p.autoSpaceKrEn ?? false],
      [this.autoSpaceKrNumCb, 'autoSpaceKrNum', p.autoSpaceKrNum ?? false],
    ];
    for (const [cb, key, orig] of cbFields) {
      if (cb.checked !== orig) (mods as Record<string, unknown>)[key] = cb.checked;
    }

    const newVA = parseInt(this.verticalAlignSelect.value) || 0;
    if (newVA !== (p.verticalAlign ?? 0)) mods.verticalAlign = newVA;
    const newKrBreak = parseInt(this.koreanBreakSelect.value) || 0;
    if (newKrBreak !== (p.koreanBreakUnit ?? 0)) mods.koreanBreakUnit = newKrBreak;
    const newEnBreak = parseInt(this.englishBreakSelect.value) || 0;
    if (newEnBreak !== (p.englishBreakUnit ?? 0)) mods.englishBreakUnit = newEnBreak;

    // ── 탭 설정 탭
    const newAutoLeft = this.tabResult.tabAutoLeftCb.checked;
    if (newAutoLeft !== (p.tabAutoLeft ?? false)) mods.tabAutoLeft = newAutoLeft;
    const newAutoRight = this.tabResult.tabAutoRightCb.checked;
    if (newAutoRight !== (p.tabAutoRight ?? false)) mods.tabAutoRight = newAutoRight;

    // 탭 목록 변경 확인 (내용 비교)
    const origStops = p.tabStops ?? [];
    const ts = this.tabState;
    const tabsChanged = ts.currentTabStops.length !== origStops.length
      || ts.currentTabStops.some((t, i) =>
        t.position !== origStops[i].position
        || t.type !== origStops[i].type
        || t.fill !== origStops[i].fill
      );
    if (tabsChanged) {
      mods.tabStops = ts.currentTabStops.map(t => ({ ...t }));
    }

    // ── 테두리/배경 탭
    const HWPUNIT_PER_MM = 7200 / 25.4;
    const defBd = { type: 0, width: 0, color: '#000000' };
    const sideKeys = ['left', 'right', 'top', 'bottom'] as const;
    const propKeys = ['borderLeft', 'borderRight', 'borderTop', 'borderBottom'] as const;
    let borderChanged = false;
    for (let i = 0; i < 4; i++) {
      const cur = this.borderStates[sideKeys[i]];
      const orig = (p[propKeys[i]] as { type: number; width: number; color: string } | undefined) ?? defBd;
      if (cur.type !== orig.type || cur.width !== orig.width || cur.color !== orig.color) {
        borderChanged = true;
        break;
      }
    }
    if (borderChanged) {
      mods.borderLeft = { ...this.borderStates.left };
      mods.borderRight = { ...this.borderStates.right };
      mods.borderTop = { ...this.borderStates.top };
      mods.borderBottom = { ...this.borderStates.bottom };
    }

    // 배경
    const newFillType = this.borderResult.bgFillSelect.value;
    if (newFillType !== (p.fillType ?? 'none')) mods.fillType = newFillType;
    const newFillColor = this.borderResult.bgFillPicker.value;
    if (newFillColor !== (p.fillColor ?? '#ffffff')) mods.fillColor = newFillColor;
    const newPatColor = this.borderResult.bgPatColorInput.value;
    if (newPatColor !== (p.patternColor ?? '#000000')) mods.patternColor = newPatColor;
    // [Issue #1172] patternType: 무늬 없음 = -1 (IR 정합). select '없음' value=-1.
    // 종전 `|| 0` 은 -1(truthy)을 보존하나 폴백 기본값을 0 으로 두어, patternType=-1
    // 문단에서 0!=-1 변경 오인 → fillType=solid 강제 주입(배경 생성) 결함을 냈다.
    const parsedPat = parseInt(this.borderResult.bgPatShapeSelect.value, 10);
    const newPatType = Number.isNaN(parsedPat) ? -1 : parsedPat;
    if (newPatType !== (p.patternType ?? -1)) mods.patternType = newPatType;
    // 배경이 변경되었으면 fillType도 함께 전송
    if (mods.fillColor || mods.patternColor || mods.patternType !== undefined) {
      if (!mods.fillType) mods.fillType = newFillType;
    }

    // 간격 (mm → HWPUNIT)
    const origSp = p.borderSpacing ?? [0, 0, 0, 0];
    const bsi = this.borderResult.bdSpacingInputs;
    // bdSpacingInputs: [0]=left, [1]=top, [2]=right, [3]=bottom
    const newSp = [
      Math.round((parseFloat(bsi[0].value) || 0) * HWPUNIT_PER_MM),
      Math.round((parseFloat(bsi[2].value) || 0) * HWPUNIT_PER_MM),
      Math.round((parseFloat(bsi[1].value) || 0) * HWPUNIT_PER_MM),
      Math.round((parseFloat(bsi[3].value) || 0) * HWPUNIT_PER_MM),
    ];
    if (newSp[0] !== origSp[0] || newSp[1] !== origSp[1] || newSp[2] !== origSp[2] || newSp[3] !== origSp[3]) {
      mods.borderSpacing = newSp;
    }
    const borderConnect = this.borderResult.bdConnectCb.checked;
    if (borderConnect !== (p.borderConnect ?? false)) mods.borderConnect = borderConnect;
    const borderIgnoreMargin = this.borderResult.bdIgnoreMarginCb.checked;
    if (borderIgnoreMargin !== (p.borderIgnoreMargin ?? false)) mods.borderIgnoreMargin = borderIgnoreMargin;

    return mods;
  }

  private getSelectedAlignment(): string {
    for (const [val, btn] of Object.entries(this.alignBtns)) {
      if (btn.classList.contains('active')) return val;
    }
    return 'justify';
  }

  // ════════════════════════════════════════════════════════
  //  설정/취소
  // ════════════════════════════════════════════════════════

  private handleOk(): void {
    const mods = this.collectMods();
    if (Object.keys(mods).length === 0) {
      this.hide();
      return;
    }
    if (this.onApply) this.onApply(mods);
    this.hide();
  }

}
