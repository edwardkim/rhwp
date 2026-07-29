import { ModalDialog } from './dialog';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { SectionDef } from '@/core/types';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';

const HWPUNIT_PER_PT = 100; // 1pt = 100 HWPUNIT (HWP 내부 단위)

function hwpunitToPt(hu: number): number {
  return Math.round(hu / HWPUNIT_PER_PT * 10) / 10;
}

function ptToHwpunit(pt: number): number {
  return Math.round(pt * HWPUNIT_PER_PT);
}

/** 레이블 최소 폭 (모든 행 정렬 통일) */
const LABEL_MIN_W = '110px';

/** 콤보+숫자입력 컨트롤 쌍 */
interface NumCombo {
  select: HTMLSelectElement;
  input: HTMLInputElement;
}

export class SectionSettingsDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private sectionIdx: number;
  private sectionDef!: SectionDef;

  // 입력 필드
  private pageNumCombo!: NumCombo;
  private pictureNumCombo!: NumCombo;
  private tableNumCombo!: NumCombo;
  private equationNumCombo!: NumCombo;
  private hideHeaderCheck!: HTMLInputElement;
  private hideMasterPageCheck!: HTMLInputElement;
  private hideBorderCheck!: HTMLInputElement;
  private hideEmptyLineCheck!: HTMLInputElement;
  private columnSpacingInput!: HTMLInputElement;
  private defaultTabSpacingInput!: HTMLInputElement;
  private applyScopeSelect!: HTMLSelectElement;

  constructor(wasm: WasmBridge, eventBus: EventBus, sectionIdx: number, private services?: CommandServices) {
    super('구역 설정', 400);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.sectionIdx = sectionIdx;
  }

  show(): void {
    super.show();
    this.sectionDef = this.wasm.getSectionDef(this.sectionIdx);
    this.populateFields();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    // ── 시작 쪽 번호 ──
    const pageNumSection = this.createSection('시작 쪽 번호');
    this.pageNumCombo = this.createPageNumCombo();
    pageNumSection.appendChild(this.labeledRow('종류(N):', this.pageNumCombo));
    body.appendChild(pageNumSection);

    // ── 개체 시작 번호 ──
    const objNumSection = this.createSection('개체 시작 번호');
    this.pictureNumCombo = this.createObjNumCombo();
    this.tableNumCombo = this.createObjNumCombo();
    this.equationNumCombo = this.createObjNumCombo();
    objNumSection.appendChild(this.labeledRow('그림(P):', this.pictureNumCombo));
    objNumSection.appendChild(this.labeledRow('표(A):', this.tableNumCombo));
    objNumSection.appendChild(this.labeledRow('수식(E):', this.equationNumCombo));
    body.appendChild(objNumSection);

    // ── 기타 ──
    const etcSection = this.createSection('기타');

    this.hideHeaderCheck = document.createElement('input');
    this.hideHeaderCheck.type = 'checkbox';
    etcSection.appendChild(this.checkRow(this.hideHeaderCheck, '첫 쪽에만 머리말/꼬리말 감추기(H)'));

    this.hideMasterPageCheck = document.createElement('input');
    this.hideMasterPageCheck.type = 'checkbox';
    etcSection.appendChild(this.checkRow(this.hideMasterPageCheck, '첫 쪽에만 바탕쪽 감추기(M)'));

    this.hideBorderCheck = document.createElement('input');
    this.hideBorderCheck.type = 'checkbox';
    etcSection.appendChild(this.checkRow(this.hideBorderCheck, '첫 쪽에만 테두리/배경 감추기(E)'));

    this.hideEmptyLineCheck = document.createElement('input');
    this.hideEmptyLineCheck.type = 'checkbox';
    etcSection.appendChild(this.checkRow(this.hideEmptyLineCheck, '빈 줄 감추기(L)'));

    this.columnSpacingInput = this.numberInput();
    etcSection.appendChild(this.labeledRowSimple('단 사이 간격(G):', this.columnSpacingInput, 'pt'));

    this.defaultTabSpacingInput = this.numberInput();
    etcSection.appendChild(this.labeledRowSimple('기본 탭 간격(I):', this.defaultTabSpacingInput, 'pt'));

    body.appendChild(etcSection);

    // ── 적용 범위 ──
    const scopeSection = this.createSection('적용 범위');
    this.applyScopeSelect = document.createElement('select');
    this.applyScopeSelect.className = 'dialog-select';
    this.applyScopeSelect.style.width = '160px';
    for (const [label, value] of [
      ['선택된 문자열', 'selection'],
      ['현재 구역', 'current'],
      ['문서 전체', 'all'],
    ] as const) {
      const opt = document.createElement('option');
      opt.value = value;
      opt.textContent = label;
      this.applyScopeSelect.appendChild(opt);
    }
    this.applyScopeSelect.value = 'current';
    scopeSection.appendChild(this.labeledRowSimple('적용 범위(Y):', this.applyScopeSelect));
    body.appendChild(scopeSection);

    return body;
  }

  protected onConfirm(): void {
    const pageComboVal = this.pageNumCombo.select.value;
    // pageNumType: 0=이어서, 1=홀수, 2=짝수
    // pageNum: 사용자 선택 시 입력값, 그 외 0
    let pageNum = 0;
    let pageNumType = 0;
    if (pageComboVal === 'odd') {
      pageNumType = 1;
    } else if (pageComboVal === 'even') {
      pageNumType = 2;
    } else if (pageComboVal === 'custom') {
      pageNum = Math.max(1, parseInt(this.pageNumCombo.input.value) || 1);
    }

    const newDef: SectionDef = {
      pageNum,
      pageNumType,
      pictureNum: this.getNumComboValue(this.pictureNumCombo),
      tableNum: this.getNumComboValue(this.tableNumCombo),
      equationNum: this.getNumComboValue(this.equationNumCombo),
      // #2938: HTML min='0'은 .value를 자동 clamp하지 않으므로(#2845/#2847과 동일 패턴)
      // 직접 타이핑한 음수가 그대로 WASM으로 넘어가지 않도록 여기서 하한을 강제한다.
      columnSpacing: Math.max(0, ptToHwpunit(parseFloat(this.columnSpacingInput.value) || 0)),
      defaultTabSpacing: Math.max(0, ptToHwpunit(parseFloat(this.defaultTabSpacingInput.value) || 0)),
      hideHeader: this.hideHeaderCheck.checked,
      hideFooter: this.hideHeaderCheck.checked,
      hideMasterPage: this.hideMasterPageCheck.checked,
      hideBorder: this.hideBorderCheck.checked,
      hideFill: this.hideBorderCheck.checked,
      hideEmptyLine: this.hideEmptyLineCheck.checked,
    };

    const scope = this.applyScopeSelect.value;
    // 'current'/'selection' 은 현재 구역과 동일 처리. all=전 구역(전문서 효과).
    const apply = () => scope === 'all'
      ? this.wasm.setSectionDefAll(newDef)
      : this.wasm.setSectionDef(this.sectionIdx, newDef);
    // [구역 설정 이관] snapshot 으로 라우팅(#2077 동형). services 미주입 시 직접 적용 fallback.
    const ih = this.services?.getInputHandler();
    if (ih) {
      ih.executeOperation({
        kind: 'snapshot',
        operationType: 'sectionSettings',
        operation: () => { apply(); return ih.getCursorPosition(); },
      });
    } else if (apply().ok) {
      this.eventBus.emit('document-changed');
    }
  }

  private populateFields(): void {
    const sd = this.sectionDef;
    // 시작 쪽 번호: pageNum > 0 → 사용자, pageNumType 1=홀수, 2=짝수, 0=이어서
    if (sd.pageNum > 0) {
      this.pageNumCombo.select.value = 'custom';
      this.pageNumCombo.input.value = String(sd.pageNum);
      this.pageNumCombo.input.style.display = '';
    } else if (sd.pageNumType === 1) {
      this.pageNumCombo.select.value = 'odd';
      this.pageNumCombo.input.style.display = 'none';
    } else if (sd.pageNumType === 2) {
      this.pageNumCombo.select.value = 'even';
      this.pageNumCombo.input.style.display = 'none';
    } else {
      this.pageNumCombo.select.value = 'continue';
      this.pageNumCombo.input.style.display = 'none';
    }
    this.setNumComboValue(this.pictureNumCombo, sd.pictureNum);
    this.setNumComboValue(this.tableNumCombo, sd.tableNum);
    this.setNumComboValue(this.equationNumCombo, sd.equationNum);
    this.hideHeaderCheck.checked = sd.hideHeader || sd.hideFooter;
    this.hideMasterPageCheck.checked = sd.hideMasterPage;
    this.hideBorderCheck.checked = sd.hideBorder || sd.hideFill;
    this.hideEmptyLineCheck.checked = sd.hideEmptyLine;
    this.columnSpacingInput.value = hwpunitToPt(sd.columnSpacing).toFixed(1);
    this.defaultTabSpacingInput.value = hwpunitToPt(sd.defaultTabSpacing).toFixed(1);
  }

  /** NumCombo에서 값 읽기: 이어서→0, 사용자→입력값 */
  private getNumComboValue(combo: NumCombo): number {
    if (combo.select.value === 'custom') {
      return Math.max(1, parseInt(combo.input.value) || 1);
    }
    return 0; // 이어서
  }

  /** NumCombo에 값 설정: 0→이어서, >0→사용자+숫자 */
  private setNumComboValue(combo: NumCombo, value: number): void {
    if (value === 0) {
      combo.select.value = 'continue';
      combo.input.value = '1';
      combo.input.style.display = 'none';
    } else {
      combo.select.value = 'custom';
      combo.input.value = String(value);
      combo.input.style.display = '';
    }
  }

  // ─── DOM 헬퍼 ─────────────────────────────

  private createSection(title: string): HTMLDivElement {
    const sec = document.createElement('div');
    sec.className = 'dialog-section';
    const t = document.createElement('div');
    t.className = 'dialog-section-title';
    t.textContent = title;
    sec.appendChild(t);
    return sec;
  }

  /** 레이블 + NumCombo를 한 행으로 조합 */
  private labeledRow(labelText: string, combo: NumCombo): HTMLDivElement {
    const row = document.createElement('div');
    row.className = 'dialog-row';

    const lbl = document.createElement('span');
    lbl.className = 'dialog-label';
    lbl.style.minWidth = LABEL_MIN_W;
    lbl.textContent = labelText;
    row.appendChild(lbl);
    row.appendChild(combo.select);
    row.appendChild(combo.input);

    return row;
  }

  /** 레이블 + 단일 컨트롤 + 단위 텍스트를 한 행으로 조합 */
  private labeledRowSimple(labelText: string, control: HTMLElement, unitText?: string): HTMLDivElement {
    const row = document.createElement('div');
    row.className = 'dialog-row';

    const lbl = document.createElement('span');
    lbl.className = 'dialog-label';
    lbl.style.minWidth = LABEL_MIN_W;
    lbl.textContent = labelText;
    row.appendChild(lbl);
    row.appendChild(control);

    if (unitText) {
      const u = document.createElement('span');
      u.className = 'dialog-unit';
      u.textContent = unitText;
      row.appendChild(u);
    }
    return row;
  }

  /** 체크박스 + 텍스트를 한 행으로 조합 */
  private checkRow(checkbox: HTMLInputElement, text: string): HTMLDivElement {
    const row = document.createElement('div');
    row.className = 'dialog-row';

    const lbl = document.createElement('label');
    lbl.className = 'dialog-checkbox';
    lbl.style.cursor = 'pointer';

    checkbox.style.margin = '0';
    lbl.appendChild(checkbox);

    const span = document.createElement('span');
    span.textContent = text;
    lbl.appendChild(span);

    row.appendChild(lbl);
    return row;
  }

  private numberInput(): HTMLInputElement {
    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'dialog-input';
    inp.style.width = '72px';
    inp.step = '0.1';
    inp.min = '0';
    return inp;
  }

  /** 시작 쪽 번호용 콤보: 이어서 / 홀수 / 짝수 / 사용자 + 숫자 입력 */
  private createPageNumCombo(): NumCombo {
    const sel = document.createElement('select');
    sel.className = 'dialog-select';
    sel.style.width = '80px';
    for (const [label, value] of [
      ['이어서', 'continue'],
      ['홀수', 'odd'],
      ['짝수', 'even'],
      ['사용자', 'custom'],
    ] as const) {
      const opt = document.createElement('option');
      opt.value = value;
      opt.textContent = label;
      sel.appendChild(opt);
    }

    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'dialog-input';
    inp.style.width = '60px';
    inp.min = '1';
    inp.step = '1';
    inp.value = '1';
    inp.style.display = 'none';

    sel.addEventListener('change', () => {
      inp.style.display = sel.value === 'custom' ? '' : 'none';
    });

    return { select: sel, input: inp };
  }

  /** 개체 시작 번호용 콤보: 이어서 / 사용자 + 숫자 입력 */
  private createObjNumCombo(): NumCombo {
    const sel = document.createElement('select');
    sel.className = 'dialog-select';
    sel.style.width = '80px';
    for (const [label, value] of [['이어서', 'continue'], ['사용자', 'custom']] as const) {
      const opt = document.createElement('option');
      opt.value = value;
      opt.textContent = label;
      sel.appendChild(opt);
    }

    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'dialog-input';
    inp.style.width = '60px';
    inp.min = '1';
    inp.step = '1';
    inp.value = '1';
    inp.style.display = 'none';

    sel.addEventListener('change', () => {
      inp.style.display = sel.value === 'custom' ? '' : 'none';
    });

    return { select: sel, input: inp };
  }
}
