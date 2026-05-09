import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { EquationProperties } from '@/core/types';
import { appendSvgMarkup } from './dom-utils';

/**
 * 수식 편집 대화상자
 * - textarea로 수식 스크립트 편집
 * - WASM renderEquationPreview()로 실시간 SVG 미리보기
 * - 도구 모음 버튼으로 템플릿 삽입
 * - PicturePropsDialog 패턴 (ModalDialog 미상속, 자체 overlay/DOM/keyboard 관리)
 */

type EqTemplate = { label: string; script: string; group?: string };

const HWPEQ_TEMPLATES: EqTemplate[] = [
  { label: '분수', script: '{} over {}', group: 'struct' },
  { label: 'x²', script: '{}^{}', group: 'struct' },
  { label: 'x₂', script: '{}_{}', group: 'struct' },
  { label: '√', script: 'sqrt {}', group: 'struct' },
  { label: 'Σ', script: 'sum _{}^{}', group: 'struct' },
  { label: '∫', script: 'int _{}^{}', group: 'struct' },
  { label: '()', script: 'LEFT ( {} RIGHT )', group: 'struct' },
  { label: '[]', script: 'LEFT [ {} RIGHT ]', group: 'struct' },
  { label: '행렬', script: 'matrix { {} # {} ; {} # {} }', group: 'struct' },
  { label: 'hat', script: 'hat {}', group: 'struct' },
  { label: 'bar', script: 'bar {}', group: 'struct' },
  { label: 'α', script: 'alpha', group: 'greek' },
  { label: 'β', script: 'beta', group: 'greek' },
  { label: 'γ', script: 'gamma', group: 'greek' },
  { label: 'δ', script: 'delta', group: 'greek' },
  { label: 'θ', script: 'theta', group: 'greek' },
  { label: 'π', script: 'pi', group: 'greek' },
  { label: 'σ', script: 'sigma', group: 'greek' },
  { label: 'λ', script: 'lambda', group: 'greek' },
  { label: 'ω', script: 'omega', group: 'greek' },
  { label: '±', script: 'pm', group: 'op' },
  { label: '×', script: 'times', group: 'op' },
  { label: '÷', script: 'div', group: 'op' },
  { label: '≠', script: 'neq', group: 'op' },
  { label: '≤', script: 'leq', group: 'op' },
  { label: '≥', script: 'geq', group: 'op' },
  { label: '∞', script: 'inf', group: 'op' },
  { label: '→', script: 'rarrow', group: 'op' },
  { label: '∈', script: 'in', group: 'op' },
  { label: '⊂', script: 'subset', group: 'op' },
];

const LATEX_TEMPLATES: EqTemplate[] = [
  { label: '분수', script: '\\frac{}{}', group: 'struct' },
  { label: 'x²', script: '{}^{}', group: 'struct' },
  { label: 'x₂', script: '{}_{}', group: 'struct' },
  { label: '√', script: '\\sqrt{}', group: 'struct' },
  { label: 'Σ', script: '\\sum_{}^{}', group: 'struct' },
  { label: '∫', script: '\\int_{}^{}', group: 'struct' },
  { label: '()', script: '\\left( {} \\right)', group: 'struct' },
  { label: '[]', script: '\\left[ {} \\right]', group: 'struct' },
  { label: '행렬', script: '\\begin{pmatrix} {} & {} \\\\ {} & {} \\end{pmatrix}', group: 'struct' },
  { label: 'hat', script: '\\hat{}', group: 'struct' },
  { label: 'bar', script: '\\bar{}', group: 'struct' },
  { label: 'α', script: '\\alpha', group: 'greek' },
  { label: 'β', script: '\\beta', group: 'greek' },
  { label: 'γ', script: '\\gamma', group: 'greek' },
  { label: 'δ', script: '\\delta', group: 'greek' },
  { label: 'θ', script: '\\theta', group: 'greek' },
  { label: 'π', script: '\\pi', group: 'greek' },
  { label: 'σ', script: '\\sigma', group: 'greek' },
  { label: 'λ', script: '\\lambda', group: 'greek' },
  { label: 'ω', script: '\\omega', group: 'greek' },
  { label: '±', script: '\\pm', group: 'op' },
  { label: '×', script: '\\times', group: 'op' },
  { label: '÷', script: '\\div', group: 'op' },
  { label: '≠', script: '\\neq', group: 'op' },
  { label: '≤', script: '\\leq', group: 'op' },
  { label: '≥', script: '\\geq', group: 'op' },
  { label: '∞', script: '\\infty', group: 'op' },
  { label: '→', script: '\\rightarrow', group: 'op' },
  { label: '∈', script: '\\in', group: 'op' },
  { label: '⊂', script: '\\subset', group: 'op' },
];

type EqInputMode = 'hwpeq' | 'latex';

export class EquationEditorDialog {
  private wasm: WasmBridge;
  private eventBus: EventBus;

  // DOM
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private previewContainer!: HTMLDivElement;
  private scriptArea!: HTMLTextAreaElement;
  private fontSizeInput!: HTMLInputElement;
  private colorInput!: HTMLInputElement;
  private toolbarContainer!: HTMLDivElement;
  private modeIndicator!: HTMLSpanElement;
  private built = false;

  // 입력 모드
  private mode: EqInputMode = 'hwpeq';

  // 현재 편집 대상 좌표
  private sec = 0;
  private para = 0;
  private ci = 0;
  private cellIdx?: number;
  private cellParaIdx?: number;

  // 원본 속성 (비교용)
  private origProps: EquationProperties | null = null;

  // debounce 타이머
  private previewTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(wasm: WasmBridge, eventBus: EventBus) {
    this.wasm = wasm;
    this.eventBus = eventBus;
  }

  /** 대화상자 열기 */
  open(sec: number, para: number, ci: number, cellIdx?: number, cellParaIdx?: number): void {
    this.build();
    this.sec = sec;
    this.para = para;
    this.ci = ci;
    this.cellIdx = cellIdx;
    this.cellParaIdx = cellParaIdx;

    // WASM에서 수식 속성 가져오기
    try {
      this.origProps = this.wasm.getEquationProperties(sec, para, ci, cellIdx, cellParaIdx);
    } catch (err) {
      console.warn('[EquationEditor] 수식 속성 가져오기 실패:', err);
      return;
    }

    // UI에 값 채우기
    this.scriptArea.value = this.origProps.script || '';
    this.fontSizeInput.value = String(Math.round(this.origProps.fontSize / 100)); // HWPUNIT→pt
    this.colorInput.value = colorRefToHex(this.origProps.color);

    // 대화상자 표시
    document.body.appendChild(this.overlay);
    setTimeout(() => {
      this.scriptArea.focus();
      this.updatePreview();
    }, 50);
  }

  /** 대화상자 닫기 */
  hide(): void {
    if (this.previewTimer) {
      clearTimeout(this.previewTimer);
      this.previewTimer = null;
    }
    this.overlay?.remove();
  }

  /** 한 번만 DOM 구성 */
  private build(): void {
    if (this.built) return;
    this.built = true;

    // 오버레이
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    // 대화상자 컨테이너
    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap eq-dialog';

    // 타이틀바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    const titleText = document.createElement('span');
    titleText.textContent = '수식 편집';
    titleBar.appendChild(titleText);

    const modeWrap = document.createElement('span');
    modeWrap.className = 'eq-mode-wrap';
    const modeBtn = document.createElement('button');
    modeBtn.className = 'eq-mode-btn';
    modeBtn.title = 'hwpeq / LaTeX 입력 모드 전환 (Ctrl+M)';
    this.modeIndicator = document.createElement('span');
    this.modeIndicator.className = 'eq-mode-indicator';
    this.updateModeIndicator();
    modeBtn.appendChild(this.modeIndicator);
    modeBtn.addEventListener('click', () => this.toggleMode());
    modeWrap.appendChild(modeBtn);
    titleBar.appendChild(modeWrap);

    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);

    // 본문
    const body = document.createElement('div');
    body.className = 'dialog-body eq-body';

    // 1) 도구 모음
    this.toolbarContainer = document.createElement('div');
    this.rebuildToolbar();
    body.appendChild(this.toolbarContainer);

    // 2) 미리보기 영역
    this.previewContainer = document.createElement('div');
    this.previewContainer.className = 'eq-preview';
    body.appendChild(this.previewContainer);

    // 3) 텍스트 입력 영역
    this.scriptArea = document.createElement('textarea');
    this.scriptArea.className = 'eq-script';
    this.scriptArea.rows = 4;
    this.scriptArea.spellcheck = false;
    this.scriptArea.placeholder = 'hwpeq: {} over {}, sqrt {}, alpha ...';
    this.scriptArea.addEventListener('input', () => this.schedulePreview());
    body.appendChild(this.scriptArea);

    // 4) 속성 행 (글자 크기 + 색상)
    const propsRow = document.createElement('div');
    propsRow.className = 'dialog-row eq-props-row';

    const sizeLabel = document.createElement('span');
    sizeLabel.className = 'dialog-label';
    sizeLabel.textContent = '글자 크기';
    this.fontSizeInput = document.createElement('input');
    this.fontSizeInput.type = 'number';
    this.fontSizeInput.className = 'dialog-input';
    this.fontSizeInput.min = '1';
    this.fontSizeInput.max = '127';
    this.fontSizeInput.step = '1';
    this.fontSizeInput.addEventListener('change', () => this.schedulePreview());
    const sizeUnit = document.createElement('span');
    sizeUnit.className = 'dialog-unit';
    sizeUnit.textContent = 'pt';

    const colorLabel = document.createElement('span');
    colorLabel.className = 'dialog-label';
    colorLabel.textContent = '색';
    this.colorInput = document.createElement('input');
    this.colorInput.type = 'color';
    this.colorInput.className = 'eq-color-input';
    this.colorInput.addEventListener('change', () => this.schedulePreview());

    propsRow.append(sizeLabel, this.fontSizeInput, sizeUnit, colorLabel, this.colorInput);
    body.appendChild(propsRow);

    // 5) 버튼 영역
    const footer = document.createElement('div');
    footer.className = 'dialog-footer';
    const okBtn = document.createElement('button');
    okBtn.className = 'dialog-btn dialog-btn-primary';
    okBtn.textContent = '확인';
    okBtn.addEventListener('click', () => this.handleOk());
    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '취소';
    cancelBtn.addEventListener('click', () => this.hide());
    footer.append(okBtn, cancelBtn);

    // 조립
    this.dialog.append(titleBar, body, footer);
    this.overlay.appendChild(this.dialog);

    // 키보드 핸들링
    this.overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        this.hide();
      }
      if (e.key === 'Enter' && e.ctrlKey) {
        e.preventDefault();
        e.stopPropagation();
        this.handleOk();
      }
      if (e.key === 'm' && e.ctrlKey && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        this.toggleMode();
      }
      e.stopPropagation();
    });

    // 오버레이 클릭으로 닫기
    this.overlay.addEventListener('mousedown', (e) => {
      if (e.target === this.overlay) this.hide();
    });

    // 드래그 지원
    this.enableDrag(titleBar);
  }

  private toggleMode(): void {
    this.mode = this.mode === 'hwpeq' ? 'latex' : 'hwpeq';
    this.updateModeIndicator();
    this.rebuildToolbar();
    this.scriptArea.placeholder = this.mode === 'latex'
      ? 'LaTeX: \\frac{a}{b}, \\sqrt{x}, \\alpha ...'
      : 'hwpeq: {} over {}, sqrt {}, alpha ...';
  }

  private updateModeIndicator(): void {
    if (!this.modeIndicator) return;
    this.modeIndicator.textContent = this.mode === 'latex' ? 'LaTeX' : 'hwpeq';
    this.modeIndicator.dataset.mode = this.mode;
  }

  private rebuildToolbar(): void {
    if (!this.toolbarContainer) return;
    const templates = this.mode === 'latex' ? LATEX_TEMPLATES : HWPEQ_TEMPLATES;
    const toolbar = document.createElement('div');
    toolbar.className = 'eq-toolbar';
    let currentGroup = '';
    for (const tmpl of templates) {
      if (tmpl.group && tmpl.group !== currentGroup && currentGroup !== '') {
        const sep = document.createElement('span');
        sep.className = 'eq-toolbar-sep';
        toolbar.appendChild(sep);
      }
      currentGroup = tmpl.group || '';
      const btn = document.createElement('button');
      btn.className = 'eq-toolbar-btn';
      btn.textContent = tmpl.label;
      btn.title = tmpl.script;
      btn.addEventListener('click', () => this.insertTemplate(tmpl.script));
      toolbar.appendChild(btn);
    }
    this.toolbarContainer.replaceChildren(toolbar);
  }

  /** textarea 커서 위치에 템플릿 삽입 */
  private insertTemplate(script: string): void {
    const ta = this.scriptArea;
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const before = ta.value.substring(0, start);
    const after = ta.value.substring(end);

    // 앞에 공백 추가 (이전 문자가 공백/빈 문자열이 아닌 경우)
    const needSpaceBefore = before.length > 0 && !/\s$/.test(before);
    const needSpaceAfter = after.length > 0 && !/^\s/.test(after);
    const insertion = (needSpaceBefore ? ' ' : '') + script + (needSpaceAfter ? ' ' : '');

    ta.value = before + insertion + after;

    // {} 안에 커서 배치
    const bracePos = (before + insertion).indexOf('{}', start);
    if (bracePos >= 0) {
      ta.selectionStart = bracePos + 1;
      ta.selectionEnd = bracePos + 1;
    } else {
      const newPos = start + insertion.length;
      ta.selectionStart = newPos;
      ta.selectionEnd = newPos;
    }

    ta.focus();
    this.schedulePreview();
  }

  /** 디바운스된 미리보기 갱신 예약 */
  private schedulePreview(): void {
    if (this.previewTimer) clearTimeout(this.previewTimer);
    this.previewTimer = setTimeout(() => this.updatePreview(), 300);
  }

  /** WASM으로 SVG 미리보기 갱신 */
  private updatePreview(): void {
    const script = this.scriptArea.value.trim();
    if (!script) {
      this.showPreviewMessage('eq-preview-empty', '수식을 입력하세요');
      return;
    }

    const fontSizePt = parseInt(this.fontSizeInput.value, 10) || 10;
    const fontSizeHwpunit = fontSizePt * 100; // pt → HWPUNIT
    const color = hexToColorRef(this.colorInput.value);

    try {
      const svg = this.wasm.renderEquationPreview(script, fontSizeHwpunit, color);
      this.previewContainer.replaceChildren();
      appendSvgMarkup(this.previewContainer, svg);
    } catch (err) {
      this.showPreviewMessage('eq-preview-error', '미리보기 오류');
      console.warn('[EquationEditor] 미리보기 오류:', err);
    }
  }

  private showPreviewMessage(className: string, message: string): void {
    const span = document.createElement('span');
    span.className = className;
    span.textContent = message;
    this.previewContainer.replaceChildren(span);
  }

  /** 확인 버튼 핸들러 */
  private handleOk(): void {
    if (!this.origProps) return;

    const script = this.scriptArea.value;
    const fontSizePt = parseInt(this.fontSizeInput.value, 10) || 10;
    const fontSizeHwpunit = fontSizePt * 100;
    const color = hexToColorRef(this.colorInput.value);

    // 변경된 속성만 전송
    const updated: Record<string, unknown> = {};
    if (script !== this.origProps.script) updated.script = script;
    if (fontSizeHwpunit !== this.origProps.fontSize) updated.fontSize = fontSizeHwpunit;
    if (color !== this.origProps.color) updated.color = color;

    if (Object.keys(updated).length > 0) {
      try {
        this.wasm.setEquationProperties(this.sec, this.para, this.ci, this.cellIdx, this.cellParaIdx, updated);
        this.eventBus.emit('document-changed');
      } catch (err) {
        console.warn('[EquationEditor] 수식 속성 설정 실패:', err);
      }
    }

    this.hide();
  }

  /** 타이틀바 드래그 */
  private enableDrag(titleEl: HTMLElement): void {
    let offsetX = 0, offsetY = 0, isDragging = false;
    titleEl.addEventListener('mousedown', (e) => {
      if ((e.target as HTMLElement).closest('.dialog-close')) return;
      isDragging = true;
      const rect = this.dialog.getBoundingClientRect();
      offsetX = e.clientX - rect.left;
      offsetY = e.clientY - rect.top;
      e.preventDefault();
    });
    document.addEventListener('mousemove', (e) => {
      if (!isDragging) return;
      this.dialog.style.left = `${e.clientX - offsetX}px`;
      this.dialog.style.top = `${e.clientY - offsetY}px`;
      this.dialog.style.margin = '0';
    });
    document.addEventListener('mouseup', () => { isDragging = false; });
  }
}

// ── 색상 변환 유틸리티 ──────────────────────────

/** HWP 0x00BBGGRR → #RRGGBB */
function colorRefToHex(colorRef: number): string {
  const r = colorRef & 0xFF;
  const g = (colorRef >> 8) & 0xFF;
  const b = (colorRef >> 16) & 0xFF;
  return '#' + [r, g, b].map(c => c.toString(16).padStart(2, '0')).join('');
}

/** #RRGGBB → HWP 0x00BBGGRR */
function hexToColorRef(hex: string): number {
  const clean = hex.replace('#', '');
  const r = parseInt(clean.substring(0, 2), 16);
  const g = parseInt(clean.substring(2, 4), 16);
  const b = parseInt(clean.substring(4, 6), 16);
  return (b << 16) | (g << 8) | r;
}
