import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { CharProperties, ParaProperties } from '@/core/types';
import type { CommandDispatcher } from '@/command/dispatcher';
import { userSettings } from '@/core/user-settings';
import type { FontSet } from '@/core/user-settings';
import { getLocalFonts } from '@/core/local-fonts';

/** 서식 도구 모음 (style-bar) 컨트롤러 */
export class Toolbar {
  private styleName: HTMLSelectElement;
  private fontName: HTMLSelectElement;
  private fontSize: HTMLInputElement;
  private btnBold: HTMLButtonElement;
  private btnItalic: HTMLButtonElement;
  private btnUnderline: HTMLButtonElement;
  private btnStrike: HTMLButtonElement;
  private btnTextColor: HTMLButtonElement;
  private colorPicker: HTMLInputElement;
  private colorBar: HTMLElement;
  private btnHighlight: HTMLButtonElement;
  private highlightDropdown: HTMLElement;
  private highlightBar: HTMLElement;
  private highlightColor = '#ffff00';
  private btnSizeUp: HTMLButtonElement;
  private btnSizeDown: HTMLButtonElement;
  private charfxDropdown: HTMLElement;
  private charfxBtn: HTMLButtonElement;
  private charfxIcon: HTMLElement;
  private charfxMenu: HTMLElement;
  private lsSelect: HTMLSelectElement;
  private btnLsUp: HTMLButtonElement;
  private btnLsDown: HTMLButtonElement;
  private fontLang: HTMLSelectElement;

  private enabled = false;
  private styleDropdownInitialized = false;
  /** 마지막으로 받은 fontFamilies (언어별 7개 배열) */
  private lastFontFamilies?: string[];

  constructor(
    private container: HTMLElement,
    private wasm: WasmBridge,
    private eventBus: EventBus,
    private dispatcher: CommandDispatcher,
  ) {
    this.styleName = container.querySelector('#style-name')!;
    this.fontName = container.querySelector('#font-name')!;
    this.fontSize = container.querySelector('#font-size')!;
    this.btnBold = container.querySelector('#btn-bold')!;
    this.btnItalic = container.querySelector('#btn-italic')!;
    this.btnUnderline = container.querySelector('#btn-underline')!;
    this.btnStrike = container.querySelector('#btn-strike')!;
    this.btnTextColor = container.querySelector('#btn-text-color')!;
    this.colorPicker = container.querySelector('#text-color-picker')!;
    this.colorBar = container.querySelector('#color-bar')!;
    this.btnHighlight = container.querySelector('#btn-highlight')!;
    this.highlightDropdown = container.querySelector('#highlight-dropdown')!;
    this.highlightBar = container.querySelector('#highlight-bar')!;
    this.btnSizeUp = container.querySelector('#btn-size-up')!;
    this.btnSizeDown = container.querySelector('#btn-size-down')!;
    this.charfxDropdown = container.querySelector('#charfx-dropdown')!;
    this.charfxBtn = container.querySelector('#btn-charfx')!;
    this.charfxIcon = container.querySelector('#charfx-icon')!;
    this.charfxMenu = container.querySelector('#charfx-menu')!;
    this.lsSelect = container.querySelector('#linespacing-select')!;
    this.btnLsUp = container.querySelector('#btn-ls-up')!;
    this.btnLsDown = container.querySelector('#btn-ls-down')!;
    this.fontLang = container.querySelector('#font-lang')!;

    this.setupFormatButtons();
    this.setupCharfxDropdown();
    this.setupLineSpacingDropdown();
    this.setupFontControls();
    this.setupColorPicker();
    this.setupHighlightPicker();
    this.setupAlignButtons();
    this.setupBulletPopup();
    this.setupStyleDropdown();

    eventBus.on('cursor-format-changed', (props) => {
      this.updateState(props as CharProperties);
    });

    eventBus.on('cursor-para-changed', (props) => {
      this.updateParaState(props as ParaProperties);
    });

    eventBus.on('cursor-style-changed', (info) => {
      this.updateStyleState(info as { id: number; name: string });
    });
  }

  /** B/I/U/S 토글 버튼 클릭 이벤트 → 커맨드 디스패치 */
  private setupFormatButtons(): void {
    const buttons: [HTMLButtonElement, string][] = [
      [this.btnBold, 'format:bold'],
      [this.btnItalic, 'format:italic'],
      [this.btnUnderline, 'format:underline'],
      [this.btnStrike, 'format:strikethrough'],
    ];
    for (const [btn, cmdId] of buttons) {
      btn.addEventListener('mousedown', (e) => {
        e.preventDefault();
        this.dispatcher.dispatch(cmdId);
      });
    }
  }

  /** 글자 효과 드롭다운 설정 */
  private setupCharfxDropdown(): void {
    // 버튼 클릭 → 드롭다운 열기/닫기
    this.charfxBtn.addEventListener('mousedown', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.charfxDropdown.classList.toggle('open');
    });

    // 메뉴 항목 클릭 → 커맨드 디스패치 + 닫기
    this.charfxMenu.addEventListener('mousedown', (e) => {
      e.preventDefault();
      e.stopPropagation();
      const item = (e.target as HTMLElement).closest('.sb-dropdown-item') as HTMLElement | null;
      if (!item) return;
      const fmt = item.dataset.format;
      if (fmt) {
        this.dispatcher.dispatch(`format:${fmt}`);
      }
      this.charfxDropdown.classList.remove('open');
    });

    // 외부 클릭 시 닫기
    document.addEventListener('mousedown', (e) => {
      if (!this.charfxDropdown.contains(e.target as Node)) {
        this.charfxDropdown.classList.remove('open');
      }
    });
  }

  /** 줄 간격 셀렉트 + 증감 버튼 + 더블클릭 직접 입력 설정 */
  private setupLineSpacingDropdown(): void {
    // 셀렉트 변경 → 적용
    this.lsSelect.addEventListener('change', () => {
      const val = Number(this.lsSelect.value);
      if (val > 0) this.dispatcher.dispatch('format:line-spacing', { value: val });
    });

    // 더블클릭 → 직접 입력 모드
    this.lsSelect.addEventListener('dblclick', (e) => {
      e.preventDefault();
      const curVal = this.lsSelect.value;
      const input = document.createElement('input');
      input.type = 'text';
      input.className = 'sb-ls-select';
      input.style.textAlign = 'center';
      input.value = curVal;
      this.lsSelect.style.display = 'none';
      this.lsSelect.parentElement!.insertBefore(input, this.lsSelect);
      input.focus();
      input.select();

      const commit = () => {
        const num = parseInt(input.value, 10);
        if (num > 0) {
          this.ensureLsOption(num);
          this.lsSelect.value = String(num);
          this.dispatcher.dispatch('format:line-spacing', { value: num });
        }
        input.remove();
        this.lsSelect.style.display = '';
      };

      input.addEventListener('keydown', (ke) => {
        if (ke.key === 'Enter') { ke.preventDefault(); commit(); }
        else if (ke.key === 'Escape') { input.remove(); this.lsSelect.style.display = ''; }
      });
      input.addEventListener('blur', commit);
    });

    // ▲ 버튼: +5%
    this.btnLsUp.addEventListener('mousedown', (e) => {
      e.preventDefault();
      const cur = Number(this.lsSelect.value) || 160;
      const next = cur + 5;
      this.ensureLsOption(next);
      this.lsSelect.value = String(next);
      this.dispatcher.dispatch('format:line-spacing', { value: next });
    });

    // ▼ 버튼: -5%
    this.btnLsDown.addEventListener('mousedown', (e) => {
      e.preventDefault();
      const cur = Number(this.lsSelect.value) || 160;
      const next = Math.max(5, cur - 5);
      this.ensureLsOption(next);
      this.lsSelect.value = String(next);
      this.dispatcher.dispatch('format:line-spacing', { value: next });
    });
  }

  /** 프리셋에 없는 줄간격 값이면 option을 동적 추가한다 */
  private ensureLsOption(val: number): void {
    const str = String(val);
    if (!this.lsSelect.querySelector(`option[value="${str}"]`)) {
      const opt = document.createElement('option');
      opt.value = str;
      opt.textContent = `${val} %`;
      // 올바른 위치에 삽입 (정렬 순서)
      let inserted = false;
      for (const existing of this.lsSelect.options) {
        if (Number(existing.value) > val) {
          this.lsSelect.insertBefore(opt, existing);
          inserted = true;
          break;
        }
      }
      if (!inserted) this.lsSelect.appendChild(opt);
    }
  }

  /** 글꼴 선택 + 크기 변경 이벤트 */
  private setupFontControls(): void {
    this.populateFontSetOptions();
    this.populateLocalFontOptions();

    this.fontName.addEventListener('change', () => {
      const name = this.fontName.value;
      if (!name) return;

      // 대표 글꼴 세트 선택인지 확인
      const fontSet = this.findFontSetByName(name);
      if (fontSet) {
        this.applyFontSet(fontSet);
        return;
      }

      const langVal = this.fontLang.value;
      if (langVal === 'all') {
        // 전체 언어 일괄 적용
        const fontId = this.wasm.findOrCreateFontId(name);
        if (fontId >= 0) {
          this.eventBus.emit('format-char', { fontId } as CharProperties);
        }
      } else {
        // 특정 언어만 적용 (fontIds 배열)
        const langIdx = parseInt(langVal, 10);
        const fontId = this.wasm.findOrCreateFontIdForLang(langIdx, name);
        if (fontId >= 0 && this.lastFontFamilies) {
          // 현재 fontIds를 기반으로 해당 언어만 교체
          const ids: number[] = [];
          for (let i = 0; i < 7; i++) {
            if (i === langIdx) {
              ids.push(fontId);
            } else {
              // 다른 언어는 현재 글꼴 ID 유지 (기존 값 조회)
              const existingName = this.lastFontFamilies[i] || this.lastFontFamilies[0];
              ids.push(this.wasm.findOrCreateFontIdForLang(i, existingName));
            }
          }
          this.eventBus.emit('format-char', { fontIds: ids } as CharProperties);
        }
      }
    });

    // 언어 선택 변경 시 해당 언어의 글꼴명을 드롭다운에 표시
    this.fontLang.addEventListener('change', () => {
      this.updateFontNameByLang();
    });

    // 크기 입력 (Enter 키로 확정)
    this.fontSize.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        const pt = parseFloat(this.fontSize.value);
        if (!isNaN(pt) && pt > 0) {
          this.eventBus.emit('format-char', { fontSize: Math.round(pt * 100) } as CharProperties);
        }
      }
    });

    // 크기 증감 버튼
    this.btnSizeUp.addEventListener('mousedown', (e) => {
      e.preventDefault();
      const pt = parseFloat(this.fontSize.value) || 10;
      const newPt = pt + 1;
      this.fontSize.value = String(newPt);
      this.eventBus.emit('format-char', { fontSize: Math.round(newPt * 100) } as CharProperties);
    });

    this.btnSizeDown.addEventListener('mousedown', (e) => {
      e.preventDefault();
      const pt = parseFloat(this.fontSize.value) || 10;
      const newPt = Math.max(1, pt - 1);
      this.fontSize.value = String(newPt);
      this.eventBus.emit('format-char', { fontSize: Math.round(newPt * 100) } as CharProperties);
    });
  }

  /** 글자색 피커 이벤트 */
  private setupColorPicker(): void {
    this.btnTextColor.addEventListener('mousedown', (e) => {
      e.preventDefault();
      this.colorPicker.click();
    });

    this.colorPicker.addEventListener('input', () => {
      const color = this.colorPicker.value;
      this.colorBar.style.background = color;
      this.eventBus.emit('format-char', { textColor: color } as CharProperties);
    });
  }

  /** 형광펜 팔레트 설정 */
  private setupHighlightPicker(): void {
    // 한컴 형광펜 색상 팔레트 (7열 × 5행 + 하단 액션)
    const PALETTE = [
      ['#ff0000', '#ff8000', '#ffff00', '#80ff00', '#00ff00', '#00ff80', '#00ffff'],
      ['#0080ff', '#0000ff', '#8000ff', '#ff00ff', '#ff0080', '#c0c0c0', '#808080'],
      ['#ff9999', '#ffcc99', '#ffff99', '#ccff99', '#99ff99', '#99ffcc', '#99ffff'],
      ['#99ccff', '#9999ff', '#cc99ff', '#ff99ff', '#ff99cc', '#e0e0e0', '#404040'],
      ['#cc0000', '#cc6600', '#cccc00', '#66cc00', '#00cc00', '#00cc66', '#00cccc'],
      ['#0066cc', '#0000cc', '#6600cc', '#cc00cc', '#cc0066', '#999999', '#000000'],
    ];

    const palette = this.container.querySelector('#highlight-palette')!;

    // "색 없음" + "다른 색..." 액션 행
    const actRow = document.createElement('div');
    actRow.className = 'sb-hl-palette-actions';
    const btnNone = document.createElement('button');
    btnNone.textContent = '색 없음';
    btnNone.addEventListener('mousedown', (e) => {
      e.preventDefault();
      this.highlightColor = '#ffffff';
      this.highlightBar.style.background = '#ffffff';
      this.eventBus.emit('format-char', { shadeColor: '#ffffff' } as CharProperties);
      this.highlightDropdown.classList.remove('open');
    });
    const btnOther = document.createElement('button');
    btnOther.textContent = '다른 색...';
    const hiddenPicker = document.createElement('input');
    hiddenPicker.type = 'color';
    hiddenPicker.value = this.highlightColor;
    hiddenPicker.style.cssText = 'position:absolute;width:0;height:0;opacity:0;';
    btnOther.appendChild(hiddenPicker);
    btnOther.addEventListener('mousedown', (e) => {
      e.preventDefault();
      hiddenPicker.click();
    });
    hiddenPicker.addEventListener('input', () => {
      this.highlightColor = hiddenPicker.value;
      this.highlightBar.style.background = this.highlightColor;
      this.eventBus.emit('format-char', { shadeColor: this.highlightColor } as CharProperties);
      this.highlightDropdown.classList.remove('open');
    });
    actRow.appendChild(btnNone);
    actRow.appendChild(btnOther);
    palette.appendChild(actRow);

    // 색상 스워치 행들
    for (const row of PALETTE) {
      const rowEl = document.createElement('div');
      rowEl.className = 'sb-hl-palette-row';
      for (const color of row) {
        const swatch = document.createElement('div');
        swatch.className = 'sb-hl-swatch';
        swatch.style.background = color;
        swatch.addEventListener('mousedown', (e) => {
          e.preventDefault();
          this.highlightColor = color;
          this.highlightBar.style.background = color;
          this.eventBus.emit('format-char', { shadeColor: color } as CharProperties);
          this.highlightDropdown.classList.remove('open');
        });
        rowEl.appendChild(swatch);
      }
      palette.appendChild(rowEl);
    }

    // 버튼 클릭 → 팔레트 토글
    this.btnHighlight.addEventListener('mousedown', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.highlightDropdown.classList.toggle('open');
    });

    // 외부 클릭 시 닫기
    document.addEventListener('mousedown', (e) => {
      if (!this.highlightDropdown.contains(e.target as Node)) {
        this.highlightDropdown.classList.remove('open');
      }
    });
  }

  /** 문단 정렬 버튼 이벤트 → 커맨드 디스패치 */
  private setupAlignButtons(): void {
    const aligns: [string, string][] = [
      ['#btn-align-left', 'format:align-left'],
      ['#btn-align-center', 'format:align-center'],
      ['#btn-align-right', 'format:align-right'],
      ['#btn-align-justify', 'format:align-justify'],
      ['#btn-align-distribute', 'format:align-distribute'],
      ['#btn-align-split', 'format:align-split'],
    ];
    for (const [sel, cmdId] of aligns) {
      const btn = this.container.querySelector(sel) as HTMLButtonElement;
      if (btn) {
        btn.addEventListener('mousedown', (e) => {
          e.preventDefault();
          this.dispatcher.dispatch(cmdId);
        });
      }
    }
  }

  /** 글머리표 버튼 팝업 (18종 선택 그리드) */
  private setupBulletPopup(): void {
    const btn = document.getElementById('tb-bullet');
    if (!btn) return;

    const BULLETS = [
      '●', '■', '◆', '▶', '○', '□',
      '◇', '▷', '★', '☆', '♠', '♣',
      '♥', '♦', '✓', '→', '-', '·',
    ];

    let popup: HTMLDivElement | null = null;
    const showPopup = () => {
      if (popup) { popup.remove(); popup = null; return; }
      popup = document.createElement('div');
      popup.className = 'bullet-popup';
      popup.style.cssText = 'position:absolute;z-index:1000;background:#fff;border:1px solid #b0b8c8;border-radius:3px;box-shadow:0 2px 6px rgba(0,0,0,0.15);padding:4px;display:grid;grid-template-columns:repeat(6,1fr);gap:2px;';
      const rect = btn.getBoundingClientRect();
      popup.style.left = `${rect.left}px`;
      popup.style.top = `${rect.bottom + 2}px`;
      for (const ch of BULLETS) {
        const cell = document.createElement('button');
        cell.type = 'button';
        cell.style.cssText = 'width:28px;height:28px;border:1px solid #ddd;border-radius:2px;background:#fff;cursor:pointer;font-size:16px;display:flex;align-items:center;justify-content:center;';
        cell.textContent = ch;
        cell.title = ch;
        cell.addEventListener('mousedown', (e) => {
          e.preventDefault();
          e.stopPropagation();
          popup?.remove();
          popup = null;
          this.dispatcher.dispatch('format:apply-bullet', { bulletChar: ch });
        });
        cell.addEventListener('mouseenter', () => { cell.style.background = '#e7eaf4'; });
        cell.addEventListener('mouseleave', () => { cell.style.background = '#fff'; });
        popup.appendChild(cell);
      }
      document.body.appendChild(popup);
      const close = (e: MouseEvent) => {
        if (popup && !popup.contains(e.target as Node) && e.target !== btn) {
          popup.remove(); popup = null;
          document.removeEventListener('mousedown', close);
        }
      };
      setTimeout(() => document.addEventListener('mousedown', close), 0);
    };

    btn.addEventListener('mousedown', (e) => {
      e.preventDefault();
      showPopup();
    });
  }

  /** 스타일 드롭다운 change 이벤트 → 커맨드 디스패치 */
  private setupStyleDropdown(): void {
    this.styleName.addEventListener('change', () => {
      const styleId = parseInt(this.styleName.value);
      if (!isNaN(styleId)) {
        this.dispatcher.dispatch('format:apply-style', { styleId });
      }
    });
  }

  /** 문서 로드 시 글꼴 드롭다운을 초기화한다 (기본 글꼴 + 문서 글꼴 + 대표/로컬) */
  initFontDropdown(docFonts?: string[]): void {
    const BASE_FONTS = ['함초롬바탕', '함초롬돋움', '맑은 고딕', '나눔고딕', '바탕', '돋움', '궁서'];
    this.fontName.replaceChildren();
    for (const name of BASE_FONTS) {
      const opt = document.createElement('option');
      opt.value = name;
      opt.textContent = name;
      this.fontName.appendChild(opt);
    }
    if (docFonts?.length) {
      const seen = new Set(BASE_FONTS);
      for (const name of docFonts) {
        if (!seen.has(name)) {
          const opt = document.createElement('option');
          opt.value = name;
          opt.textContent = name;
          this.fontName.appendChild(opt);
          seen.add(name);
        }
      }
    }
    this.populateFontSetOptions();
    this.populateLocalFontOptions();
  }

  /** 문서 로드 시 스타일 목록으로 드롭다운을 채운다 */
  initStyleDropdown(): void {
    try {
      const styles = this.wasm.getStyleList();
      this.styleName.replaceChildren();
      for (const style of styles) {
        const opt = document.createElement('option');
        opt.value = String(style.id);
        opt.textContent = style.name;
        this.styleName.appendChild(opt);
      }
      this.styleDropdownInitialized = true;
    } catch {
      // 문서 미로드 시 무시
    }
  }

  /** 커서 위치의 문단 속성(줄간격 등)을 도구 모음에 반영한다 */
  private updateParaState(props: ParaProperties): void {
    if (props.lineSpacingType === 'Percent' && props.lineSpacing !== undefined) {
      const val = Math.round(props.lineSpacing);
      this.ensureLsOption(val);
      this.lsSelect.value = String(val);
    }
  }

  /** 커서 위치의 스타일을 드롭다운에 반영한다 */
  private updateStyleState(info: { id: number; name: string }): void {
    if (!this.styleDropdownInitialized) return;
    this.styleName.value = String(info.id);
  }

  /** 커서 위치의 서식을 도구 모음에 반영한다 */
  updateState(props: CharProperties): void {
    // B/I/U/S + 양각/음각/외곽선/위첨/아래첨 토글 상태
    this.setActive(this.btnBold, !!props.bold);
    this.setActive(this.btnItalic, !!props.italic);
    this.setActive(this.btnUnderline, !!props.underline);
    this.setActive(this.btnStrike, !!props.strikethrough);
    // 글자 효과 드롭다운: 항목별 active 표시 + 메인 버튼 active
    const fxState: Record<string, boolean> = {
      emboss: !!props.emboss,
      engrave: !!props.engrave,
      outline: !!(props.outlineType),
      superscript: !!props.superscript,
      subscript: !!props.subscript,
    };
    let anyFxActive = false;
    for (const item of this.charfxMenu.querySelectorAll<HTMLElement>('.sb-dropdown-item')) {
      const fmt = item.dataset.format ?? '';
      const on = fxState[fmt] ?? false;
      item.classList.toggle('active', on);
      if (on) anyFxActive = true;
    }
    this.setActive(this.charfxBtn, anyFxActive);

    // fontFamilies 배열 저장 (언어별 글꼴 선택용)
    if (props.fontFamilies) {
      this.lastFontFamilies = props.fontFamilies;
    }

    // 글꼴명 — 선택된 언어 카테고리에 따라 표시
    const displayFont = this.getDisplayFontFamily(props);
    if (displayFont) {
      if (!this.fontName.querySelector(`option[value="${CSS.escape(displayFont)}"]`)) {
        const opt = document.createElement('option');
        opt.value = displayFont;
        opt.textContent = displayFont;
        this.fontName.appendChild(opt);
      }
      this.fontName.value = displayFont;
    }

    // 글자 크기 (HWPUNIT → pt, 1pt = 100 HWPUNIT)
    if (props.fontSize !== undefined) {
      const pt = props.fontSize / 100;
      this.fontSize.value = pt.toFixed(1);
    }

    // 글자색
    if (props.textColor) {
      this.colorBar.style.background = props.textColor;
      this.colorPicker.value = props.textColor;
    }

    // 형광펜 색상 표시
    if (props.shadeColor) {
      this.highlightBar.style.background = props.shadeColor;
      this.highlightColor = props.shadeColor;
    }
  }

  /** 문서 로드 상태에 따라 활성화/비활성화 */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
    const opacity = enabled ? '1' : '0.5';
    this.container.style.opacity = opacity;
    this.container.style.pointerEvents = enabled ? 'auto' : 'none';
  }

  /** 선택된 언어 카테고리에 해당하는 글꼴명을 반환한다 */
  private getDisplayFontFamily(props: CharProperties): string | undefined {
    const langVal = this.fontLang.value;
    if (langVal !== 'all' && props.fontFamilies) {
      const idx = parseInt(langVal, 10);
      if (idx >= 0 && idx < props.fontFamilies.length) {
        return props.fontFamilies[idx];
      }
    }
    return props.fontFamily;
  }

  /** 언어 콤보 변경 시 해당 언어의 글꼴명을 드롭다운에 표시한다 */
  private updateFontNameByLang(): void {
    if (!this.lastFontFamilies) return;
    const langVal = this.fontLang.value;
    let displayFont: string | undefined;
    if (langVal === 'all') {
      displayFont = this.lastFontFamilies[0]; // 한글 기준
    } else {
      const idx = parseInt(langVal, 10);
      if (idx >= 0 && idx < this.lastFontFamilies.length) {
        displayFont = this.lastFontFamilies[idx];
      }
    }
    if (displayFont) {
      if (!this.fontName.querySelector(`option[value="${CSS.escape(displayFont)}"]`)) {
        const opt = document.createElement('option');
        opt.value = displayFont;
        opt.textContent = displayFont;
        this.fontName.appendChild(opt);
      }
      this.fontName.value = displayFont;
    }
  }

  private setActive(btn: HTMLElement, active: boolean): void {
    btn.classList.toggle('active', active);
  }

  /** 대표 글꼴 optgroup을 #font-name 드롭다운에 추가 */
  private populateFontSetOptions(): void {
    const fontSets = userSettings.getAllFontSets();
    if (fontSets.length === 0) return;

    // 기존 optgroup 제거 (재호출 대비)
    this.fontName.querySelectorAll('optgroup[label="대표 글꼴"]').forEach(g => g.remove());

    const group = document.createElement('optgroup');
    group.label = '대표 글꼴';

    for (const fs of fontSets) {
      const opt = document.createElement('option');
      opt.value = `__fontset__${fs.name}`;
      opt.textContent = `◆ ${fs.name}`;
      group.appendChild(opt);
    }

    this.fontName.insertBefore(group, this.fontName.firstChild);
  }

  /** 로컬 글꼴 optgroup을 #font-name 드롭다운에 추가 */
  private populateLocalFontOptions(): void {
    const localFonts = getLocalFonts();
    if (localFonts.length === 0) return;

    // 기존 로컬 글꼴 optgroup 제거 (재호출 대비)
    this.fontName.querySelectorAll('optgroup[label="로컬 글꼴"]').forEach(g => g.remove());

    const group = document.createElement('optgroup');
    group.label = '로컬 글꼴';

    for (const name of localFonts) {
      const opt = document.createElement('option');
      opt.value = name;
      opt.textContent = name;
      group.appendChild(opt);
    }

    // 대표 글꼴 optgroup 다음에 삽입
    const fontSetGroup = this.fontName.querySelector('optgroup[label="대표 글꼴"]');
    if (fontSetGroup?.nextSibling) {
      this.fontName.insertBefore(group, fontSetGroup.nextSibling);
    } else {
      this.fontName.insertBefore(group, this.fontName.firstChild);
    }
  }

  /** 대표 글꼴 세트 이름으로 FontSet 검색 */
  private findFontSetByName(value: string): FontSet | undefined {
    if (!value.startsWith('__fontset__')) return undefined;
    const name = value.slice('__fontset__'.length);
    return userSettings.getAllFontSets().find(fs => fs.name === name);
  }

  /** 대표 글꼴 세트를 7개 언어에 일괄 적용 */
  private applyFontSet(fs: FontSet): void {
    const langKeys: (keyof Omit<FontSet, 'name'>)[] = [
      'korean', 'english', 'chinese', 'japanese', 'other', 'symbol', 'user',
    ];
    const ids: number[] = [];
    for (let i = 0; i < 7; i++) {
      const fontName = fs[langKeys[i]];
      ids.push(this.wasm.findOrCreateFontIdForLang(i, fontName));
    }
    this.eventBus.emit('format-char', { fontIds: ids } as CharProperties);
  }
}
