/**
 * 문단 모양 대화상자 — 탭 설정 탭 / 테두리·배경 탭 빌더
 * ParaShapeDialog에서 분리된 독립 함수로, 클래스에 의존하지 않는다.
 */

import { createFieldset, row, label, numberInput, unit } from './para-shape-helpers';

import { t as i18nText } from '../i18n/index.ts';
// ════════════════════════════════════════════════════════
//  공유 타입
// ════════════════════════════════════════════════════════

export interface TabStop {
  position: number;
  type: number;
  fill: number;
}

export interface TabState {
  currentTabStops: TabStop[];
  deletedTabStops: TabStop[];
  selectedTabIndex: number;
}

export interface BorderSideState {
  type: number;
  width: number;
  color: string;
}

export interface BorderStates {
  left: BorderSideState;
  right: BorderSideState;
  top: BorderSideState;
  bottom: BorderSideState;
}

export interface SideToggles {
  left: boolean;
  right: boolean;
  top: boolean;
  bottom: boolean;
}

// ════════════════════════════════════════════════════════
//  탭 설정 탭
// ════════════════════════════════════════════════════════

export interface TabSettingsResult {
  panel: HTMLDivElement;
  tabTypeRadios: HTMLInputElement[];
  tabFillSelect: HTMLSelectElement;
  tabPositionInput: HTMLInputElement;
  tabListBody: HTMLTableSectionElement;
  deletedTabListBody: HTMLTableSectionElement;
  tabAutoLeftCb: HTMLInputElement;
  tabAutoRightCb: HTMLInputElement;
  defaultTabLabel: HTMLSpanElement;
  renderTabList(): void;
  renderDeletedTabList(): void;
}

function appendHeaderRow(thead: HTMLTableSectionElement, labels: string[]): void {
  const tr = document.createElement('tr');
  labels.forEach(labelText => {
    const th = document.createElement('th');
    th.textContent = labelText;
    tr.appendChild(th);
  });
  thead.appendChild(tr);
}

function appendTableCell(tr: HTMLTableRowElement, text: string): void {
  const td = document.createElement('td');
  td.textContent = text;
  tr.appendChild(td);
}

export function buildTabSettingsTab(state: TabState): TabSettingsResult {
  const TAB_TYPE_NAMES = ['왼쪽', '오른쪽', '가운데', '소수점'];

  const panel = document.createElement('div');
  panel.className = 'dialog-tab-panel';

  // ── 탭 종류 섹션
  const typeSection = document.createElement('fieldset');
  typeSection.className = 'dialog-section';
  const typeTitle = document.createElement('legend');
  typeTitle.className = 'dialog-section-title';
  typeTitle.textContent = i18nText('dialog.paraShapeTabBuilders.typeTitle.text');
  typeSection.appendChild(typeTitle);

  // 라디오 행: 왼쪽/오른쪽/가운데/소수점
  const typeRow = document.createElement('div');
  typeRow.className = 'dialog-row';
  const TAB_TYPES = [
    { value: '0', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label') },
    { value: '1', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x02c2dd') },
    { value: '2', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.xfcad9c') },
    { value: '3', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x8526e8') },
  ];
  const tabTypeRadios = TAB_TYPES.map(({ value, label: lbl }) => {
    const labelEl = document.createElement('label');
    labelEl.className = 'dialog-radio-label';
    const radio = document.createElement('input');
    radio.type = 'radio';
    radio.name = 'ps-tab-type';
    radio.value = value;
    labelEl.appendChild(radio);
    labelEl.appendChild(document.createTextNode(` ${lbl}`));
    typeRow.appendChild(labelEl);
    return radio;
  });
  tabTypeRadios[0].checked = true;
  typeSection.appendChild(typeRow);

  // 채움 모양
  const fillRow = document.createElement('div');
  fillRow.className = 'dialog-row';
  const fillLabel = document.createElement('label');
  fillLabel.className = 'dialog-label';
  fillLabel.textContent = i18nText('dialog.paraShapeTabBuilders.fillLabel.text');
  const tabFillSelect = document.createElement('select');
  tabFillSelect.className = 'dialog-select';
  [
    { value: '0', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x48f5ab') },
    { value: '1', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x503c41') },
    { value: '2', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x386cfb') },
    { value: '3', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.xd6f3d4') },
    { value: '4', label: '-·-·-·-·-·' },
    { value: '5', label: '-··-··-··-··' },
    { value: '6', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x6b0654') },
    { value: '7', label: i18nText('dialog.paraShapeTabBuilders.appendTableCell.label.x2fc737') },
  ].forEach(({ value, label: lbl }) => {
    const opt = document.createElement('option');
    opt.value = value;
    opt.textContent = lbl;
    tabFillSelect.appendChild(opt);
  });
  fillRow.appendChild(fillLabel);
  fillRow.appendChild(tabFillSelect);
  typeSection.appendChild(fillRow);

  // 탭 위치 + 추가 버튼
  const posRow = document.createElement('div');
  posRow.className = 'dialog-row';
  const posLabel = document.createElement('label');
  posLabel.className = 'dialog-label';
  posLabel.textContent = i18nText('dialog.paraShapeTabBuilders.posLabel.text');
  const tabPositionInput = document.createElement('input');
  tabPositionInput.className = 'dialog-input';
  tabPositionInput.type = 'number';
  tabPositionInput.step = '0.1';
  tabPositionInput.value = '0.0';
  tabPositionInput.style.width = '80px';
  const posUnit = document.createElement('span');
  posUnit.className = 'dialog-unit';
  posUnit.textContent = 'pt';
  const addBtn = document.createElement('button');
  addBtn.className = 'dialog-btn';
  addBtn.textContent = i18nText('dialog.paraShapeTabBuilders.addBtn.text');
  addBtn.addEventListener('click', () => addTabStop());
  posRow.appendChild(posLabel);
  posRow.appendChild(tabPositionInput);
  posRow.appendChild(posUnit);
  posRow.appendChild(addBtn);
  typeSection.appendChild(posRow);
  panel.appendChild(typeSection);

  // ── 탭 목록 + 지운 탭 목록 (2열)
  const listArea = document.createElement('div');
  listArea.className = 'ps-tab-list-area';

  // 왼쪽: 탭 목록
  const tabListCol = document.createElement('div');
  tabListCol.className = 'ps-tab-list-col';
  const tabListLabel = document.createElement('div');
  tabListLabel.className = 'dialog-section-title';
  tabListLabel.textContent = i18nText('dialog.paraShapeTabBuilders.tabListLabel.text');
  tabListCol.appendChild(tabListLabel);
  const tabTable = document.createElement('table');
  tabTable.className = 'ps-tab-table';
  const thead = document.createElement('thead');
  appendHeaderRow(thead, ['위치', '종류']);
  tabTable.appendChild(thead);
  const tabListBody = document.createElement('tbody');
  tabTable.appendChild(tabListBody);
  const tabTableWrap = document.createElement('div');
  tabTableWrap.className = 'ps-tab-table-wrap';
  tabTableWrap.appendChild(tabTable);
  tabListCol.appendChild(tabTableWrap);

  // 삭제 버튼들
  const btnCol = document.createElement('div');
  btnCol.className = 'ps-tab-list-btns';
  const delBtn = document.createElement('button');
  delBtn.className = 'dialog-btn ps-tab-del-btn';
  delBtn.textContent = '\u2715';
  delBtn.title = i18nText('dialog.paraShapeTabBuilders.delBtn.tooltip');
  delBtn.addEventListener('click', () => deleteTabStop());
  const delAllBtn = document.createElement('button');
  delAllBtn.className = 'dialog-btn ps-tab-del-btn';
  delAllBtn.textContent = '\u2715\u2715';
  delAllBtn.title = i18nText('dialog.paraShapeTabBuilders.delAllBtn.tooltip');
  delAllBtn.addEventListener('click', () => deleteAllTabStops());
  btnCol.appendChild(delBtn);
  btnCol.appendChild(delAllBtn);

  // 오른쪽: 지운 탭 목록
  const delListCol = document.createElement('div');
  delListCol.className = 'ps-tab-list-col';
  const delListLabel = document.createElement('div');
  delListLabel.className = 'dialog-section-title';
  delListLabel.textContent = i18nText('dialog.paraShapeTabBuilders.delListLabel.text');
  delListCol.appendChild(delListLabel);
  const delTable = document.createElement('table');
  delTable.className = 'ps-tab-table';
  const dThead = document.createElement('thead');
  appendHeaderRow(dThead, ['위치', '종류']);
  delTable.appendChild(dThead);
  const deletedTabListBody = document.createElement('tbody');
  delTable.appendChild(deletedTabListBody);
  const delTableWrap = document.createElement('div');
  delTableWrap.className = 'ps-tab-table-wrap';
  delTableWrap.appendChild(delTable);
  delListCol.appendChild(delTableWrap);

  listArea.appendChild(tabListCol);
  listArea.appendChild(btnCol);
  listArea.appendChild(delListCol);
  panel.appendChild(listArea);

  // ── 자동 탭 섹션
  const autoSection = document.createElement('fieldset');
  autoSection.className = 'dialog-section';
  const autoTitle = document.createElement('legend');
  autoTitle.className = 'dialog-section-title';
  autoTitle.textContent = i18nText('dialog.paraShapeTabBuilders.autoTitle.text');
  autoSection.appendChild(autoTitle);

  const autoRow = document.createElement('div');
  autoRow.className = 'dialog-row';
  const mkCb = (lbl: string): HTMLInputElement => {
    const labelEl = document.createElement('label');
    labelEl.className = 'dialog-checkbox';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    labelEl.appendChild(cb);
    labelEl.appendChild(document.createTextNode(` ${lbl}`));
    autoRow.appendChild(labelEl);
    return cb;
  };
  const tabAutoLeftCb = mkCb(i18nText('dialog.paraShapeTabBuilders.mkCb.label'));
  const tabAutoRightCb = mkCb(i18nText('dialog.paraShapeTabBuilders.mkCb.label.xdf4173'));
  autoSection.appendChild(autoRow);
  panel.appendChild(autoSection);

  // ── 기본 탭 섹션
  const defaultSection = document.createElement('fieldset');
  defaultSection.className = 'dialog-section';
  const defaultTitle = document.createElement('legend');
  defaultTitle.className = 'dialog-section-title';
  defaultTitle.textContent = i18nText('dialog.paraShapeTabBuilders.defaultTitle.text');
  defaultSection.appendChild(defaultTitle);

  const defaultRow = document.createElement('div');
  defaultRow.className = 'dialog-row';
  const defaultLabel = document.createElement('span');
  defaultLabel.textContent = i18nText('dialog.paraShapeTabBuilders.defaultLabel.text');
  const defaultTabLabel = document.createElement('span');
  defaultTabLabel.textContent = '40.0 pt';
  const changeBtn = document.createElement('button');
  changeBtn.className = 'dialog-btn';
  changeBtn.textContent = i18nText('dialog.paraShapeTabBuilders.changeBtn.text');
  changeBtn.disabled = true;
  defaultRow.appendChild(defaultLabel);
  defaultRow.appendChild(defaultTabLabel);
  defaultRow.appendChild(changeBtn);
  defaultSection.appendChild(defaultRow);
  panel.appendChild(defaultSection);

  // ── 탭 헬퍼 함수 (클로저) ──

  function addTabStop(): void {
    const positionPt = parseFloat(tabPositionInput.value) || 0;
    if (positionPt <= 0) return;
    const position = Math.round(positionPt * 100);
    const tabType = parseInt(tabTypeRadios.find(r => r.checked)?.value ?? '0');
    const fill = parseInt(tabFillSelect.value);
    if (state.currentTabStops.some(t => t.position === position)) return;
    state.currentTabStops.push({ position, type: tabType, fill });
    state.currentTabStops.sort((a, b) => a.position - b.position);
    renderTabList();
  }

  function deleteTabStop(): void {
    if (state.selectedTabIndex < 0 || state.selectedTabIndex >= state.currentTabStops.length) return;
    const [removed] = state.currentTabStops.splice(state.selectedTabIndex, 1);
    state.deletedTabStops.push(removed);
    state.selectedTabIndex = -1;
    renderTabList();
    renderDeletedTabList();
  }

  function deleteAllTabStops(): void {
    state.deletedTabStops.push(...state.currentTabStops);
    state.currentTabStops.length = 0;
    state.selectedTabIndex = -1;
    renderTabList();
    renderDeletedTabList();
  }

  function restoreTabStop(idx: number): void {
    if (idx < 0 || idx >= state.deletedTabStops.length) return;
    const [restored] = state.deletedTabStops.splice(idx, 1);
    if (!state.currentTabStops.some(t => t.position === restored.position)) {
      state.currentTabStops.push(restored);
      state.currentTabStops.sort((a, b) => a.position - b.position);
    }
    renderTabList();
    renderDeletedTabList();
  }

  function renderTabList(): void {
    tabListBody.replaceChildren();
    state.currentTabStops.forEach((t, i) => {
      const tr = document.createElement('tr');
      if (i === state.selectedTabIndex) tr.className = 'selected';
      appendTableCell(tr, `${(t.position / 100).toFixed(1)} pt`);
      appendTableCell(tr, TAB_TYPE_NAMES[t.type] ?? '?');
      tr.addEventListener('click', () => {
        state.selectedTabIndex = i;
        renderTabList();
      });
      tabListBody.appendChild(tr);
    });
  }

  function renderDeletedTabList(): void {
    deletedTabListBody.replaceChildren();
    state.deletedTabStops.forEach((t, i) => {
      const tr = document.createElement('tr');
      appendTableCell(tr, `${(t.position / 100).toFixed(1)} pt`);
      appendTableCell(tr, TAB_TYPE_NAMES[t.type] ?? '?');
      tr.addEventListener('dblclick', () => restoreTabStop(i));
      tr.title = i18nText('dialog.paraShapeTabBuilders.tr.tooltip');
      deletedTabListBody.appendChild(tr);
    });
  }

  return {
    panel,
    tabTypeRadios,
    tabFillSelect,
    tabPositionInput,
    tabListBody,
    deletedTabListBody,
    tabAutoLeftCb,
    tabAutoRightCb,
    defaultTabLabel,
    renderTabList,
    renderDeletedTabList,
  };
}

// ════════════════════════════════════════════════════════
//  테두리/배경 탭
// ════════════════════════════════════════════════════════

export interface BorderTabResult {
  panel: HTMLDivElement;
  bdTypeSelect: HTMLSelectElement;
  bdWidthSelect: HTMLSelectElement;
  bdColorInput: HTMLInputElement;
  bdPreviewInner: HTMLDivElement;
  bdConnectCb: HTMLInputElement;
  bdApplyImmCb: HTMLInputElement;
  bgFillSelect: HTMLSelectElement;
  bgFillPicker: HTMLInputElement;
  bgPatColorInput: HTMLInputElement;
  bgPatShapeSelect: HTMLSelectElement;
  bdSpacingInputs: HTMLInputElement[];
  bdAllSpacingInput: HTMLInputElement;
  bdIgnoreMarginCb: HTMLInputElement;
  updateBdPreview(): void;
}

export function buildBorderTab(
  borderStates: BorderStates,
  bdSideToggles: SideToggles,
): BorderTabResult {
  const panel = document.createElement('div');
  panel.className = 'dialog-tab-panel';

  // ── 테두리 섹션
  const borderFs = createFieldset(i18nText('dialog.paraShapeTabBuilders.createFieldset.label'));
  const borderContent = document.createElement('div');
  borderContent.className = 'ps-border-layout';

  // 좌측: 컨트롤들
  const borderLeft = document.createElement('div');
  borderLeft.className = 'ps-border-left';

  // 종류(Y)
  const typeRow = row();
  typeRow.appendChild(label(i18nText('dialog.paraShapeTabBuilders.label.label')));
  const bdTypeSelect = document.createElement('select');
  bdTypeSelect.className = 'dialog-select';
  bdTypeSelect.style.width = '100px';
  for (const [val, lbl] of [
    ['0', '선 없음'], ['1', '실선'], ['2', '파선'], ['3', '점선'],
    ['4', '일점쇄선'], ['5', '이점쇄선'], ['6', '긴 파선'], ['7', '동그라미'],
    ['8', '이중선'], ['9', '가는선+굵은선'], ['10', '굵은선+가는선'],
    ['11', '삼중선'], ['12', '물결'], ['13', '이중 물결'],
    ['14', '두꺼운 3D'], ['15', '두꺼운 3D(반대)'],
    ['16', '3D 단선'], ['17', '3D 단선(반대)'],
  ] as const) {
    const o = document.createElement('option');
    o.value = val; o.textContent = lbl;
    bdTypeSelect.appendChild(o);
  }
  bdTypeSelect.addEventListener('change', () => onBorderControlChange());
  typeRow.appendChild(bdTypeSelect);
  borderLeft.appendChild(typeRow);

  // 굵기(I)
  const widthRow = row();
  widthRow.appendChild(label(i18nText('dialog.paraShapeTabBuilders.label.label.x53f378')));
  const bdWidthSelect = document.createElement('select');
  bdWidthSelect.className = 'dialog-select';
  bdWidthSelect.style.width = '100px';
  const widths = ['0.1 mm', '0.12 mm', '0.15 mm', '0.2 mm', '0.25 mm',
                  '0.3 mm', '0.4 mm', '0.5 mm', '0.6 mm', '0.7 mm',
                  '1 mm', '1.5 mm', '2 mm', '3 mm', '4 mm', '5 mm'];
  widths.forEach((w, i) => {
    const o = document.createElement('option');
    o.value = String(i); o.textContent = w;
    bdWidthSelect.appendChild(o);
  });
  bdWidthSelect.addEventListener('change', () => onBorderControlChange());
  widthRow.appendChild(bdWidthSelect);
  borderLeft.appendChild(widthRow);

  // 색(C)
  const colorRow = row();
  colorRow.appendChild(label(i18nText('dialog.paraShapeTabBuilders.label.label.xb7d4ad')));
  const bdColorInput = document.createElement('input');
  bdColorInput.type = 'color';
  bdColorInput.value = '#000000';
  bdColorInput.className = 'cs-color-btn';
  bdColorInput.style.width = '100px';
  bdColorInput.addEventListener('input', () => onBorderControlChange());
  colorRow.appendChild(bdColorInput);
  borderLeft.appendChild(colorRow);

  // 문단 테두리 연결(M)
  const connectRow = row();
  const bdConnectCb = document.createElement('input');
  bdConnectCb.type = 'checkbox';
  bdConnectCb.id = 'ps-bd-connect';
  const connectLabel = document.createElement('label');
  connectLabel.htmlFor = 'ps-bd-connect';
  connectLabel.textContent = i18nText('dialog.paraShapeTabBuilders.connectLabel.text');
  connectRow.appendChild(bdConnectCb);
  connectRow.appendChild(connectLabel);
  borderLeft.appendChild(connectRow);

  // 선 모양 바로 적용(I)
  const applyRow = row();
  const bdApplyImmCb = document.createElement('input');
  bdApplyImmCb.type = 'checkbox';
  bdApplyImmCb.id = 'ps-bd-apply-imm';
  bdApplyImmCb.checked = true;
  const applyLabel = document.createElement('label');
  applyLabel.htmlFor = 'ps-bd-apply-imm';
  applyLabel.textContent = i18nText('dialog.paraShapeTabBuilders.applyLabel.text');
  applyRow.appendChild(bdApplyImmCb);
  applyRow.appendChild(applyLabel);
  borderLeft.appendChild(applyRow);

  borderContent.appendChild(borderLeft);

  // 우측: 미리보기 + 프리셋
  const borderRight = document.createElement('div');
  borderRight.className = 'ps-border-right';

  const previewBox = document.createElement('div');
  previewBox.className = 'ps-border-preview';
  const bdPreviewInner = document.createElement('div');
  bdPreviewInner.className = 'ps-border-inner';
  previewBox.appendChild(bdPreviewInner);
  borderRight.appendChild(previewBox);

  // 프리셋 버튼 행
  const presetRow = document.createElement('div');
  presetRow.className = 'ps-border-presets';
  const presets: [string, string, () => void][] = [
    ['┄', '테두리 없음', () => applyBorderPreset('none')],
    ['□', '상자형', () => applyBorderPreset('box')],
    ['╬', '격자형', () => applyBorderPreset('box')],
    ['▣', '사용자 정의', () => {}],
    ['全', '모두 적용/해제', () => applyBorderPreset('toggleAll')],
  ];
  for (const [icon, title, handler] of presets) {
    const btn = document.createElement('button');
    btn.className = 'ps-preset-btn'; btn.textContent = icon; btn.title = title;
    btn.addEventListener('click', handler);
    presetRow.appendChild(btn);
  }
  borderRight.appendChild(presetRow);

  borderContent.appendChild(borderRight);
  borderFs.appendChild(borderContent);
  panel.appendChild(borderFs);

  // ── 배경 섹션
  const bgFs = createFieldset(i18nText('dialog.paraShapeTabBuilders.createFieldset.label.x807c30'));

  // 면 색(Q)
  const faceRow = row();
  faceRow.appendChild(label(i18nText('dialog.paraShapeTabBuilders.label.label.x18d3e6')));
  const bgFillSelect = document.createElement('select');
  bgFillSelect.className = 'dialog-select';
  bgFillSelect.style.width = '100px';
  for (const [val, lbl] of [['none', '색 없음'], ['solid', '색 지정']] as const) {
    const o = document.createElement('option');
    o.value = val; o.textContent = lbl;
    bgFillSelect.appendChild(o);
  }
  faceRow.appendChild(bgFillSelect);
  const bgFillPicker = document.createElement('input');
  bgFillPicker.type = 'color';
  bgFillPicker.value = '#ffffff';
  bgFillPicker.className = 'cs-color-btn';
  bgFillPicker.style.marginLeft = '6px';
  faceRow.appendChild(bgFillPicker);
  bgFs.appendChild(faceRow);

  // 무늬 색(P) + 무늬 모양(L)
  const patRow = row();
  patRow.appendChild(label(i18nText('dialog.paraShapeTabBuilders.label.label.x51d4f5')));
  const bgPatColorInput = document.createElement('input');
  bgPatColorInput.type = 'color';
  bgPatColorInput.value = '#000000';
  bgPatColorInput.className = 'cs-color-btn';
  patRow.appendChild(bgPatColorInput);

  const patLabel = label(i18nText('dialog.paraShapeTabBuilders.label.label.xb8c7cc'));
  patLabel.style.marginLeft = '10px';
  patRow.appendChild(patLabel);
  const bgPatShapeSelect = document.createElement('select');
  bgPatShapeSelect.className = 'dialog-select';
  bgPatShapeSelect.style.width = '90px';
  // [Issue #1172] "없음" 의 value 는 -1 (IR patternType: 무늬 없음 = -1).
  // 종전엔 0 이라, patternType=-1 문단을 dialog 에 표시하면 select 가 0 으로
  // 폴백되고, collectMods 가 0!=-1 을 변경으로 오인하여 fillType=solid 를 강제
  // 주입 → 여백만 바꾸거나 확인만 눌러도 의도치 않은 배경/테두리가 생성됐다.
  for (const [val, lbl] of [
    ['-1', '없음'], ['1', '━'], ['2', '┃'],
    ['3', '╲'], ['4', '╱'], ['5', '┼'], ['6', '╳'],
  ] as const) {
    const o = document.createElement('option');
    o.value = val; o.textContent = lbl;
    bgPatShapeSelect.appendChild(o);
  }
  patRow.appendChild(bgPatShapeSelect);
  bgFs.appendChild(patRow);
  panel.appendChild(bgFs);

  // ── 간격 섹션
  const spacingFs = createFieldset(i18nText('dialog.paraShapeTabBuilders.createFieldset.label.x0ad57d'));
  const bdSpacingInputs: HTMLInputElement[] = [];

  const spacingGrid = document.createElement('div');
  spacingGrid.className = 'ps-spacing-grid';

  const makeCell = (labelText: string): [HTMLDivElement, HTMLInputElement] => {
    const cell = document.createElement('div');
    cell.className = 'ps-spacing-cell';
    cell.appendChild(label(labelText));
    const inp = numberInput(0, 999, 0.01); inp.style.width = '55px';
    cell.appendChild(inp);
    cell.appendChild(unit('mm'));
    return [cell, inp];
  };

  // 1행: 왼쪽(E), 위쪽(U)
  const [c0, si0] = makeCell(i18nText('dialog.paraShapeTabBuilders.makeCell.label'));
  const [c1, si2] = makeCell(i18nText('dialog.paraShapeTabBuilders.makeCell.label.x93c89b'));
  bdSpacingInputs.push(si0, si2); // [0]=left, [1]=top
  spacingGrid.appendChild(c0);
  spacingGrid.appendChild(c1);

  // 2행: 오른쪽(B), 아래쪽(V)
  const [c2, si1] = makeCell(i18nText('dialog.paraShapeTabBuilders.makeCell.label.x34c256'));
  const [c3, si3] = makeCell(i18nText('dialog.paraShapeTabBuilders.makeCell.label.x79dae2'));
  bdSpacingInputs.push(si1, si3); // [2]=right, [3]=bottom
  spacingGrid.appendChild(c2);
  spacingGrid.appendChild(c3);

  // 3행: 모두(A), 문단 여백 무시(B)
  const [c4, siAll] = makeCell(i18nText('dialog.paraShapeTabBuilders.makeCell.label.x13a264'));
  const bdAllSpacingInput = siAll;
  bdAllSpacingInput.addEventListener('change', () => {
    const v = bdAllSpacingInput.value;
    bdSpacingInputs.forEach(inp => { inp.value = v; });
  });
  spacingGrid.appendChild(c4);

  const ignoreCell = document.createElement('div');
  ignoreCell.className = 'ps-spacing-cell';
  const bdIgnoreMarginCb = document.createElement('input');
  bdIgnoreMarginCb.type = 'checkbox';
  bdIgnoreMarginCb.id = 'ps-bd-ignore-margin';
  const ignoreLabel = document.createElement('label');
  ignoreLabel.htmlFor = 'ps-bd-ignore-margin';
  ignoreLabel.textContent = i18nText('dialog.paraShapeTabBuilders.ignoreLabel.text');
  ignoreCell.appendChild(bdIgnoreMarginCb);
  ignoreCell.appendChild(ignoreLabel);
  spacingGrid.appendChild(ignoreCell);

  spacingFs.appendChild(spacingGrid);
  panel.appendChild(spacingFs);

  // ── 이벤트 핸들러 (클로저) ──

  function onBorderControlChange(): void {
    if (!bdApplyImmCb?.checked) return;
    const t = parseInt(bdTypeSelect.value);
    const w = parseInt(bdWidthSelect.value);
    const c = bdColorInput.value;
    for (const side of ['left', 'right', 'top', 'bottom'] as const) {
      if (bdSideToggles[side]) {
        borderStates[side] = { type: t, width: w, color: c };
      }
    }
    updateBdPreview();
  }

  function applyBorderPreset(mode: 'none' | 'box' | 'toggleAll'): void {
    const t = parseInt(bdTypeSelect.value);
    const w = parseInt(bdWidthSelect.value);
    const c = bdColorInput.value;
    for (const side of ['left', 'right', 'top', 'bottom'] as const) {
      if (mode === 'none') {
        borderStates[side] = { type: 0, width: 0, color: '#000000' };
        bdSideToggles[side] = false;
      } else if (mode === 'box') {
        borderStates[side] = { type: t || 1, width: w, color: c };
        bdSideToggles[side] = true;
      } else {
        if (bdSideToggles[side]) {
          borderStates[side] = { type: 0, width: 0, color: '#000000' };
          bdSideToggles[side] = false;
        } else {
          borderStates[side] = { type: t || 1, width: w, color: c };
          bdSideToggles[side] = true;
        }
      }
    }
    updateBdPreview();
  }

  function updateBdPreview(): void {
    const cssBorder = (st: BorderSideState) => {
      if (st.type === 0) return '1px dashed #d0d0d0';
      const style = st.type === 8 ? 'double' : st.type === 3 ? 'dotted' : st.type === 2 ? 'dashed' : 'solid';
      const px = Math.max(1, Math.round((st.width + 1) * 0.8));
      return `${px}px ${style} ${st.color}`;
    };
    bdPreviewInner.style.borderLeft = cssBorder(borderStates.left);
    bdPreviewInner.style.borderRight = cssBorder(borderStates.right);
    bdPreviewInner.style.borderTop = cssBorder(borderStates.top);
    bdPreviewInner.style.borderBottom = cssBorder(borderStates.bottom);
  }

  return {
    panel,
    bdTypeSelect, bdWidthSelect, bdColorInput, bdPreviewInner,
    bdConnectCb, bdApplyImmCb,
    bgFillSelect, bgFillPicker, bgPatColorInput, bgPatShapeSelect,
    bdSpacingInputs, bdAllSpacingInput, bdIgnoreMarginCb,
    updateBdPreview,
  };
}
