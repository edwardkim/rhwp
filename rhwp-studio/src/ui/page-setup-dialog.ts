import { ModalDialog } from './dialog';
import { appendSvgMarkup } from './dom-utils';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { PageDef } from '@/core/types';
import type { EventBus } from '@/core/event-bus';
import type { CommandServices } from '@/command/types';

const HWPUNIT_PER_MM = 7200 / 25.4; // ≈283.46
const PAPER_PRESET_TOLERANCE_HU = 3;

function hwpunitToMm(hu: number): number {
  return Math.round(hu * 25.4 / 7200 * 10) / 10; // 소수 1자리
}

function mmToHwpunit(mm: number): number {
  return Math.round(mm * HWPUNIT_PER_MM);
}

function samePaperSize(a: number, b: number): boolean {
  return Math.abs(a - b) <= PAPER_PRESET_TOLERANCE_HU;
}

function matchesPaperPreset(
  width: number, height: number,
  presetWidth: number, presetHeight: number,
): boolean {
  return (
    samePaperSize(width, presetWidth) && samePaperSize(height, presetHeight)
  ) || (
    samePaperSize(width, presetHeight) && samePaperSize(height, presetWidth)
  );
}

/** 용지 프리셋 (이름 → [폭, 길이] HWPUNIT) */
const PAPER_PRESETS: [string, number, number][] = [
  ['A4',     59528,  84188],
  ['A3',     84188, 119055],
  ['B4',     72850, 103040],
  ['B5',     51502,  72850],
  ['Letter', 62208,  80496],
  ['Legal',  62208, 102816],
];

/** 용지 방향 SVG 아이콘 (세로/가로) */
const ORIENT_ICONS: Record<string, string> = {
  portrait: `<svg xmlns="http://www.w3.org/2000/svg" class="orient-icon orient-icon-portrait" width="28" height="36" viewBox="0 0 28 36" aria-hidden="true" focusable="false"><rect x="1" y="1" width="26" height="34" rx="2" fill="#fff" stroke="#748bc9" stroke-width="1.5"/><line x1="6" y1="8" x2="22" y2="8" stroke="#aab" stroke-width="1.2"/><line x1="6" y1="12" x2="22" y2="12" stroke="#aab" stroke-width="1.2"/><line x1="6" y1="16" x2="18" y2="16" stroke="#aab" stroke-width="1.2"/></svg>`,
  landscape: `<svg xmlns="http://www.w3.org/2000/svg" class="orient-icon orient-icon-landscape" width="40" height="28" viewBox="0 0 40 28" aria-hidden="true" focusable="false"><rect x="1" y="1" width="38" height="26" rx="2" fill="#fff" stroke="#748bc9" stroke-width="1.5"/><line x1="7" y1="7" x2="33" y2="7" stroke="#aab" stroke-width="1.2"/><line x1="7" y1="11" x2="33" y2="11" stroke="#aab" stroke-width="1.2"/><line x1="7" y1="15" x2="29" y2="15" stroke="#aab" stroke-width="1.2"/><line x1="7" y1="19" x2="23" y2="19" stroke="#c5cce0" stroke-width="1.1"/></svg>`,
};

/** 제본 SVG 아이콘 (한쪽/맞쪽/위로) */
const BINDING_ICONS: Record<string, string> = {
  single: `<svg width="24" height="32" viewBox="0 0 24 32"><rect x="1" y="1" width="22" height="30" rx="1.5" fill="#fff" stroke="#748bc9" stroke-width="1.2"/><line x1="5" y1="7" x2="19" y2="7" stroke="#bbc" stroke-width="1"/><line x1="5" y1="10" x2="19" y2="10" stroke="#bbc" stroke-width="1"/><line x1="5" y1="13" x2="15" y2="13" stroke="#bbc" stroke-width="1"/></svg>`,
  duplex: `<svg width="32" height="32" viewBox="0 0 32 32"><rect x="1" y="1" width="13" height="30" rx="1.5" fill="#fff" stroke="#748bc9" stroke-width="1.2"/><rect x="18" y="1" width="13" height="30" rx="1.5" fill="#fff" stroke="#748bc9" stroke-width="1.2"/><line x1="14" y1="4" x2="14" y2="28" stroke="#335095" stroke-width="1.5" stroke-dasharray="2,2"/></svg>`,
  top: `<svg width="24" height="32" viewBox="0 0 24 32"><rect x="1" y="1" width="22" height="13" rx="1.5" fill="#fff" stroke="#748bc9" stroke-width="1.2"/><rect x="1" y="18" width="22" height="13" rx="1.5" fill="#fff" stroke="#748bc9" stroke-width="1.2"/><line x1="4" y1="15" x2="20" y2="15" stroke="#335095" stroke-width="1.5" stroke-dasharray="2,2"/></svg>`,
};

export class PageSetupDialog extends ModalDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;
  private sectionIdx: number;
  private pageDef!: PageDef;

  // 입력 필드 참조
  private paperSelect!: HTMLSelectElement;
  private widthInput!: HTMLInputElement;
  private heightInput!: HTMLInputElement;
  private landscapeRadios!: HTMLInputElement[];
  private bindingRadios!: HTMLInputElement[];
  private marginInputs!: Record<string, HTMLInputElement>;
  private scopeSelect!: HTMLSelectElement;

  constructor(wasm: WasmBridge, eventBus: EventBus, sectionIdx: number, private services?: CommandServices) {
    super('편집 용지', 440);
    this.wasm = wasm;
    this.eventBus = eventBus;
    this.sectionIdx = sectionIdx;
  }

  show(): void {
    super.show(); // build() → createBody() 호출
    this.pageDef = this.wasm.getPageDef(this.sectionIdx);
    this.populateFields();
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');

    // ── 용지 종류 ──
    const paperSection = this.createSection('용지 종류');
    const paperRow = this.row();

    this.paperSelect = document.createElement('select');
    this.paperSelect.className = 'dialog-select';
    this.paperSelect.style.width = '100px';
    for (const [name] of PAPER_PRESETS) {
      const opt = document.createElement('option');
      opt.value = name;
      opt.textContent = name;
      this.paperSelect.appendChild(opt);
    }
    const customOpt = document.createElement('option');
    customOpt.value = 'custom';
    customOpt.textContent = '사용자 정의';
    this.paperSelect.appendChild(customOpt);

    this.paperSelect.addEventListener('change', () => this.onPaperChange());
    paperRow.appendChild(this.paperSelect);

    const dimRow = this.row();
    dimRow.appendChild(this.label('폭'));
    this.widthInput = this.numberInput();
    dimRow.appendChild(this.widthInput);
    dimRow.appendChild(this.unit('mm'));
    dimRow.appendChild(this.label('길이'));
    this.heightInput = this.numberInput();
    dimRow.appendChild(this.heightInput);
    dimRow.appendChild(this.unit('mm'));

    paperSection.appendChild(paperRow);
    paperSection.appendChild(dimRow);
    body.appendChild(paperSection);

    // ── 용지 방향 + 제본 (가로 배치) ──
    const sectionsRow = document.createElement('div');
    sectionsRow.className = 'page-setup-sections';

    // 용지 방향
    const orientSection = this.createSection('용지 방향');
    const orientRow = document.createElement('div');
    orientRow.className = 'dialog-icon-radio-group';
    this.landscapeRadios = [
      this.iconRadio('orient', '세로', 'false', ORIENT_ICONS.portrait, orientRow),
      this.iconRadio('orient', '가로', 'true', ORIENT_ICONS.landscape, orientRow),
    ];
    for (const r of this.landscapeRadios) {
      r.addEventListener('change', () => this.onOrientChange());
    }
    orientSection.appendChild(orientRow);

    // 제본
    const bindSection = this.createSection('제본');
    const bindRow = document.createElement('div');
    bindRow.className = 'dialog-icon-radio-group';
    this.bindingRadios = [
      this.iconRadio('binding', '한쪽', '0', BINDING_ICONS.single, bindRow),
      this.iconRadio('binding', '맞쪽', '1', BINDING_ICONS.duplex, bindRow),
      this.iconRadio('binding', '위로', '2', BINDING_ICONS.top, bindRow),
    ];
    bindSection.appendChild(bindRow);

    sectionsRow.appendChild(orientSection);
    sectionsRow.appendChild(bindSection);
    body.appendChild(sectionsRow);

    // ── 용지 여백 ──
    const marginSection = this.createSection('용지 여백');
    const marginGrid = document.createElement('div');
    marginGrid.className = 'margin-grid';

    this.marginInputs = {} as Record<string, HTMLInputElement>;
    const fields: [string, string][] = [
      ['marginTop', '위쪽'],
      ['marginBottom', '아래쪽'],
      ['marginLeft', '왼쪽'],
      ['marginRight', '오른쪽'],
      ['marginHeader', '머리말'],
      ['marginFooter', '꼬리말'],
      ['marginGutter', '제본'],
    ];

    // 2열 배치: grid 직접 자식으로 배치 (label, input, unit × 2)
    const pairs: [number, number | null][] = [[0, 1], [2, 3], [4, 5], [6, null]];
    for (const [a, b] of pairs) {
      marginGrid.appendChild(this.label(fields[a][1]));
      this.marginInputs[fields[a][0]] = this.numberInput();
      marginGrid.appendChild(this.marginInputs[fields[a][0]]);
      marginGrid.appendChild(this.unit('mm'));
      if (b !== null) {
        marginGrid.appendChild(this.label(fields[b][1]));
        this.marginInputs[fields[b][0]] = this.numberInput();
        marginGrid.appendChild(this.marginInputs[fields[b][0]]);
        marginGrid.appendChild(this.unit('mm'));
      }
    }
    marginSection.appendChild(marginGrid);
    body.appendChild(marginSection);

    // ── 적용 범위 ──
    const scopeRow = this.row();
    scopeRow.appendChild(this.label('적용 범위'));
    this.scopeSelect = document.createElement('select');
    this.scopeSelect.className = 'dialog-select';
    this.scopeSelect.style.width = '120px';
    for (const [val, text] of [['all', '문서 전체'], ['new-section', '새 구역으로']] as const) {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = text;
      this.scopeSelect.appendChild(opt);
    }
    scopeRow.appendChild(this.scopeSelect);
    scopeRow.style.marginTop = '4px';
    body.appendChild(scopeRow);

    return body;
  }

  protected onConfirm(): void {
    // mm → HWPUNIT 변환
    const landscape = this.landscapeRadios[1].checked;
    let w = mmToHwpunit(parseFloat(this.widthInput.value) || 0);
    let h = mmToHwpunit(parseFloat(this.heightInput.value) || 0);

    // landscape는 PageDef에서 원본(세로) 크기로 저장
    if (landscape) { [w, h] = [h, w]; }

    const newDef: PageDef = {
      width: w,
      height: h,
      marginLeft: mmToHwpunit(parseFloat(this.marginInputs['marginLeft'].value) || 0),
      marginRight: mmToHwpunit(parseFloat(this.marginInputs['marginRight'].value) || 0),
      marginTop: mmToHwpunit(parseFloat(this.marginInputs['marginTop'].value) || 0),
      marginBottom: mmToHwpunit(parseFloat(this.marginInputs['marginBottom'].value) || 0),
      marginHeader: mmToHwpunit(parseFloat(this.marginInputs['marginHeader'].value) || 0),
      marginFooter: mmToHwpunit(parseFloat(this.marginInputs['marginFooter'].value) || 0),
      marginGutter: mmToHwpunit(parseFloat(this.marginInputs['marginGutter'].value) || 0),
      landscape,
      binding: parseInt(this.bindingRadios.find(r => r.checked)?.value ?? '0'),
    };

    // [편집 용지 이관] 쪽 정의 변경을 snapshot 으로 라우팅해 undo 가능(#2077 수식 속성 동형).
    // services 미주입 환경(구 호출부)에서만 직접 적용 fallback.
    const apply = () => this.wasm.setPageDef(this.sectionIdx, newDef);
    const ih = this.services?.getInputHandler();
    if (ih) {
      ih.executeOperation({
        kind: 'snapshot',
        operationType: 'pageSetup',
        operation: () => { apply(); return ih.getCursorPosition(); },
      });
    } else if (apply().ok) {
      this.eventBus.emit('document-changed');
    }
  }

  private populateFields(): void {
    const pd = this.pageDef;
    // 용지 종류 매칭
    let w = pd.width;
    let h = pd.height;
    // landscape면 실제 표시 크기로 교환
    if (pd.landscape) { [w, h] = [h, w]; }

    const matched = PAPER_PRESETS.find(([, pw, ph]) =>
      matchesPaperPreset(pd.width, pd.height, pw, ph)
    );
    this.paperSelect.value = matched ? matched[0] : 'custom';
    this.widthInput.value = hwpunitToMm(w).toFixed(1);
    this.heightInput.value = hwpunitToMm(h).toFixed(1);
    this.widthInput.disabled = !!matched;
    this.heightInput.disabled = !!matched;

    // 방향
    this.landscapeRadios[pd.landscape ? 1 : 0].checked = true;
    this.updateIconRadioState('orient');

    // 제본
    const bindIdx = Math.min(pd.binding, 2);
    this.bindingRadios[bindIdx].checked = true;
    this.updateIconRadioState('binding');

    // 여백
    this.marginInputs['marginTop'].value = hwpunitToMm(pd.marginTop).toFixed(1);
    this.marginInputs['marginBottom'].value = hwpunitToMm(pd.marginBottom).toFixed(1);
    this.marginInputs['marginLeft'].value = hwpunitToMm(pd.marginLeft).toFixed(1);
    this.marginInputs['marginRight'].value = hwpunitToMm(pd.marginRight).toFixed(1);
    this.marginInputs['marginHeader'].value = hwpunitToMm(pd.marginHeader).toFixed(1);
    this.marginInputs['marginFooter'].value = hwpunitToMm(pd.marginFooter).toFixed(1);
    this.marginInputs['marginGutter'].value = hwpunitToMm(pd.marginGutter).toFixed(1);
  }

  private onPaperChange(): void {
    const val = this.paperSelect.value;
    const preset = PAPER_PRESETS.find(([name]) => name === val);
    if (preset) {
      const landscape = this.landscapeRadios[1].checked;
      const [w, h] = landscape ? [preset[2], preset[1]] : [preset[1], preset[2]];
      this.widthInput.value = hwpunitToMm(w).toFixed(1);
      this.heightInput.value = hwpunitToMm(h).toFixed(1);
      this.widthInput.disabled = true;
      this.heightInput.disabled = true;
    } else {
      this.widthInput.disabled = false;
      this.heightInput.disabled = false;
    }
  }

  private onOrientChange(): void {
    // 폭/길이 교환
    const w = this.widthInput.value;
    this.widthInput.value = this.heightInput.value;
    this.heightInput.value = w;
    this.updateIconRadioState('orient');
  }

  /** 아이콘 라디오 버튼의 선택 상태를 CSS 클래스로 반영 */
  private updateIconRadioState(groupName: string): void {
    const radios = groupName === 'orient' ? this.landscapeRadios : this.bindingRadios;
    for (const r of radios) {
      const card = r.closest('.icon-radio-card');
      if (card) {
        card.classList.toggle('selected', r.checked);
      }
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
    // HTML min 속성은 .value 를 자동으로 clamp 하지 않는다(브라우저는 checkValidity()에서만
    // 검사) — 음수·비정상 값이 그대로 parseFloat 되어 wasm.setPageDef 로 전달되는 것을 막는다.
    // (#2838 번호매기기 시작 번호 clamp 누락과 동일 패턴)
    inp.addEventListener('change', () => {
      if (inp.value === '') return;
      const min = inp.min !== '' ? parseFloat(inp.min) : -Infinity;
      const max = inp.max !== '' ? parseFloat(inp.max) : Infinity;
      const v = parseFloat(inp.value);
      if (!Number.isFinite(v)) return;
      const clamped = Math.min(max, Math.max(min, v));
      if (clamped !== v) inp.value = String(clamped);
    });
    return inp;
  }

  private unit(text: string): HTMLSpanElement {
    const u = document.createElement('span');
    u.className = 'dialog-unit';
    u.textContent = text;
    return u;
  }

  /** 아이콘+텍스트 라디오 카드 — 라디오 input을 반환, 카드를 container에 추가 */
  private iconRadio(
    name: string, labelText: string, value: string,
    svgHtml: string, container: HTMLElement,
  ): HTMLInputElement {
    const card = document.createElement('label');
    card.className = 'icon-radio-card';

    const inp = document.createElement('input');
    inp.type = 'radio';
    inp.name = name;
    inp.value = value;
    inp.className = 'icon-radio-input';
    inp.addEventListener('change', () => this.updateIconRadioState(name));

    const iconWrap = document.createElement('span');
    iconWrap.className = 'icon-radio-icon';
    appendSvgMarkup(iconWrap, svgHtml);

    const text = document.createElement('span');
    text.className = 'icon-radio-text';
    text.textContent = labelText;

    card.appendChild(inp);
    card.appendChild(iconWrap);
    card.appendChild(text);
    container.appendChild(card);
    return inp;
  }
}
