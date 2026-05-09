import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { EquationProperties } from '@/core/types';
import { appendSvgMarkup } from './dom-utils';

/**
 * 수식 편집 대화상자
 * - 듀얼 모드: HWP 네이티브 ↔ LaTeX 입력
 * - 명령어 자동완성 드롭다운
 * - 탭 기반 템플릿 (구조, 그리스, 연산자, 화살표, 함수, 장식, 괄호)
 * - 기호 검색 (이름/유니코드)
 * - PicturePropsDialog 패턴 (ModalDialog 미상속, 자체 overlay/DOM/keyboard 관리)
 */

type InputMode = 'hwp' | 'latex';

interface TemplateEntry { label: string; hwp: string; latex: string }
interface TemplateGroup { id: string; name: string; items: TemplateEntry[] }

interface CommandEntry { name: string; display: string; insert: string; group: string }

const TEMPLATE_GROUPS: TemplateGroup[] = [
  { id: 'struct', name: '구조', items: [
    { label: '분수', hwp: '{} over {}', latex: '\\frac{}{}'  },
    { label: 'x²', hwp: '{}^{}', latex: '{}^{}' },
    { label: 'x₂', hwp: '{}_{}', latex: '{}_{}' },
    { label: '√', hwp: 'sqrt {}', latex: '\\sqrt{}' },
    { label: '³√', hwp: 'root 3 of {}', latex: '\\sqrt[3]{}' },
    { label: 'Σ', hwp: 'sum _{}^{}', latex: '\\sum_{}^{}' },
    { label: '∫', hwp: 'int _{}^{}', latex: '\\int_{}^{}' },
    { label: '∬', hwp: 'dint _{}^{}', latex: '\\iint_{}^{}' },
    { label: '∮', hwp: 'oint _{}^{}', latex: '\\oint_{}^{}' },
    { label: '∏', hwp: 'prod _{}^{}', latex: '\\prod_{}^{}' },
    { label: '()', hwp: 'LEFT ( {} RIGHT )', latex: '\\left( {} \\right)' },
    { label: '[]', hwp: 'LEFT [ {} RIGHT ]', latex: '\\left[ {} \\right]' },
    { label: '{}', hwp: 'LEFT lbrace {} RIGHT rbrace', latex: '\\left\\{ {} \\right\\}' },
    { label: '⌈⌉', hwp: 'LEFT lceil {} RIGHT rceil', latex: '\\left\\lceil {} \\right\\rceil' },
    { label: '⌊⌋', hwp: 'LEFT lfloor {} RIGHT rfloor', latex: '\\left\\lfloor {} \\right\\rfloor' },
    { label: '||', hwp: 'LEFT | {} RIGHT |', latex: '\\left| {} \\right|' },
    { label: '행렬', hwp: 'matrix { {} # {} ; {} # {} }', latex: '\\begin{matrix} {} & {} \\\\ {} & {} \\end{matrix}' },
    { label: 'cases', hwp: 'cases { {} ; {} }', latex: '\\begin{cases} {} \\\\ {} \\end{cases}' },
    { label: 'lim', hwp: 'lim _{} {}', latex: '\\lim_{} {}' },
  ] },
  { id: 'greek', name: '그리스', items: [
    { label: 'α', hwp: 'alpha', latex: '\\alpha' },
    { label: 'β', hwp: 'beta', latex: '\\beta' },
    { label: 'γ', hwp: 'gamma', latex: '\\gamma' },
    { label: 'δ', hwp: 'delta', latex: '\\delta' },
    { label: 'ε', hwp: 'epsilon', latex: '\\epsilon' },
    { label: 'ζ', hwp: 'zeta', latex: '\\zeta' },
    { label: 'η', hwp: 'eta', latex: '\\eta' },
    { label: 'θ', hwp: 'theta', latex: '\\theta' },
    { label: 'ι', hwp: 'iota', latex: '\\iota' },
    { label: 'κ', hwp: 'kappa', latex: '\\kappa' },
    { label: 'λ', hwp: 'lambda', latex: '\\lambda' },
    { label: 'μ', hwp: 'mu', latex: '\\mu' },
    { label: 'ν', hwp: 'nu', latex: '\\nu' },
    { label: 'ξ', hwp: 'xi', latex: '\\xi' },
    { label: 'π', hwp: 'pi', latex: '\\pi' },
    { label: 'ρ', hwp: 'rho', latex: '\\rho' },
    { label: 'σ', hwp: 'sigma', latex: '\\sigma' },
    { label: 'τ', hwp: 'tau', latex: '\\tau' },
    { label: 'υ', hwp: 'upsilon', latex: '\\upsilon' },
    { label: 'φ', hwp: 'phi', latex: '\\phi' },
    { label: 'χ', hwp: 'chi', latex: '\\chi' },
    { label: 'ψ', hwp: 'psi', latex: '\\psi' },
    { label: 'ω', hwp: 'omega', latex: '\\omega' },
    { label: 'Γ', hwp: 'Gamma', latex: '\\Gamma' },
    { label: 'Δ', hwp: 'Delta', latex: '\\Delta' },
    { label: 'Θ', hwp: 'Theta', latex: '\\Theta' },
    { label: 'Λ', hwp: 'Lambda', latex: '\\Lambda' },
    { label: 'Ξ', hwp: 'Xi', latex: '\\Xi' },
    { label: 'Π', hwp: 'Pi', latex: '\\Pi' },
    { label: 'Σ', hwp: 'Sigma', latex: '\\Sigma' },
    { label: 'Φ', hwp: 'Phi', latex: '\\Phi' },
    { label: 'Ψ', hwp: 'Psi', latex: '\\Psi' },
    { label: 'Ω', hwp: 'Omega', latex: '\\Omega' },
  ] },
  { id: 'op', name: '연산자', items: [
    { label: '±', hwp: 'pm', latex: '\\pm' },
    { label: '∓', hwp: 'mp', latex: '\\mp' },
    { label: '×', hwp: 'times', latex: '\\times' },
    { label: '÷', hwp: 'div', latex: '\\div' },
    { label: '·', hwp: 'cdot', latex: '\\cdot' },
    { label: '≠', hwp: 'neq', latex: '\\neq' },
    { label: '≤', hwp: 'leq', latex: '\\leq' },
    { label: '≥', hwp: 'geq', latex: '\\geq' },
    { label: '≈', hwp: 'approx', latex: '\\approx' },
    { label: '≡', hwp: 'equiv', latex: '\\equiv' },
    { label: '∼', hwp: 'sim', latex: '\\sim' },
    { label: '≅', hwp: 'cong', latex: '\\cong' },
    { label: '∝', hwp: 'propto', latex: '\\propto' },
    { label: '∞', hwp: 'inf', latex: '\\infty' },
    { label: '∂', hwp: 'partial', latex: '\\partial' },
    { label: '∅', hwp: 'emptyset', latex: '\\emptyset' },
    { label: '∈', hwp: 'in', latex: '\\in' },
    { label: '∉', hwp: 'notin', latex: '\\notin' },
    { label: '⊂', hwp: 'subset', latex: '\\subset' },
    { label: '⊃', hwp: 'superset', latex: '\\supset' },
    { label: '⊆', hwp: 'subseteq', latex: '\\subseteq' },
    { label: '⊇', hwp: 'supseteq', latex: '\\supseteq' },
    { label: '∪', hwp: 'union', latex: '\\cup' },
    { label: '∩', hwp: 'inter', latex: '\\cap' },
    { label: '∀', hwp: 'forall', latex: '\\forall' },
    { label: '∃', hwp: 'exist', latex: '\\exists' },
    { label: '¬', hwp: 'lnot', latex: '\\neg' },
    { label: '∧', hwp: 'wedge', latex: '\\wedge' },
    { label: '∨', hwp: 'vee', latex: '\\vee' },
    { label: '⊕', hwp: 'oplus', latex: '\\oplus' },
    { label: '⊗', hwp: 'otimes', latex: '\\otimes' },
    { label: '∴', hwp: 'therefore', latex: '\\therefore' },
    { label: '∵', hwp: 'because', latex: '\\because' },
  ] },
  { id: 'arrow', name: '화살표', items: [
    { label: '←', hwp: 'larrow', latex: '\\leftarrow' },
    { label: '→', hwp: 'rarrow', latex: '\\rightarrow' },
    { label: '↑', hwp: 'uparrow', latex: '\\uparrow' },
    { label: '↓', hwp: 'downarrow', latex: '\\downarrow' },
    { label: '↔', hwp: 'lrarrow', latex: '\\leftrightarrow' },
    { label: '⇐', hwp: 'LARROW', latex: '\\Leftarrow' },
    { label: '⇒', hwp: 'RARROW', latex: '\\Rightarrow' },
    { label: '⇔', hwp: 'LRARROW', latex: '\\Leftrightarrow' },
    { label: '↦', hwp: 'mapsto', latex: '\\mapsto' },
    { label: '↗', hwp: 'nearrow', latex: '\\nearrow' },
    { label: '↘', hwp: 'searrow', latex: '\\searrow' },
  ] },
  { id: 'func', name: '함수', items: [
    { label: 'sin', hwp: 'sin', latex: '\\sin' },
    { label: 'cos', hwp: 'cos', latex: '\\cos' },
    { label: 'tan', hwp: 'tan', latex: '\\tan' },
    { label: 'cot', hwp: 'cot', latex: '\\cot' },
    { label: 'sec', hwp: 'sec', latex: '\\sec' },
    { label: 'csc', hwp: 'csc', latex: '\\csc' },
    { label: 'arcsin', hwp: 'arcsin', latex: '\\arcsin' },
    { label: 'arccos', hwp: 'arccos', latex: '\\arccos' },
    { label: 'arctan', hwp: 'arctan', latex: '\\arctan' },
    { label: 'sinh', hwp: 'sinh', latex: '\\sinh' },
    { label: 'cosh', hwp: 'cosh', latex: '\\cosh' },
    { label: 'tanh', hwp: 'tanh', latex: '\\tanh' },
    { label: 'log', hwp: 'log', latex: '\\log' },
    { label: 'ln', hwp: 'ln', latex: '\\ln' },
    { label: 'exp', hwp: 'exp', latex: '\\exp' },
    { label: 'det', hwp: 'det', latex: '\\det' },
    { label: 'max', hwp: 'max', latex: '\\max' },
    { label: 'min', hwp: 'min', latex: '\\min' },
    { label: 'gcd', hwp: 'gcd', latex: '\\gcd' },
    { label: 'mod', hwp: 'mod', latex: '\\mod' },
  ] },
  { id: 'deco', name: '장식', items: [
    { label: 'â', hwp: 'hat {}', latex: '\\hat{}' },
    { label: 'ā', hwp: 'bar {}', latex: '\\bar{}' },
    { label: 'ã', hwp: 'tilde {}', latex: '\\tilde{}' },
    { label: 'à', hwp: 'grave {}', latex: '\\grave{}' },
    { label: 'á', hwp: 'acute {}', latex: '\\acute{}' },
    { label: 'ȧ', hwp: 'dot {}', latex: '\\dot{}' },
    { label: 'ä', hwp: 'ddot {}', latex: '\\ddot{}' },
    { label: 'a⃗', hwp: 'vec {}', latex: '\\vec{}' },
    { label: 'a̲', hwp: 'UNDERLINE {}', latex: '\\underline{}' },
    { label: 'a̅', hwp: 'OVERLINE {}', latex: '\\overline{}' },
  ] },
  { id: 'special', name: '특수', items: [
    { label: 'ℓ', hwp: 'ell', latex: '\\ell' },
    { label: 'ℏ', hwp: 'hbar', latex: '\\hbar' },
    { label: 'ℵ', hwp: 'aleph', latex: '\\aleph' },
    { label: '⋯', hwp: 'cdots', latex: '\\cdots' },
    { label: '…', hwp: 'ldots', latex: '\\ldots' },
    { label: '⋮', hwp: 'vdots', latex: '\\vdots' },
    { label: '⋱', hwp: 'ddots', latex: '\\ddots' },
    { label: '△', hwp: 'triangle', latex: '\\triangle' },
    { label: '∠', hwp: 'angle', latex: '\\angle' },
    { label: '⊥', hwp: 'bot', latex: '\\bot' },
    { label: '°', hwp: 'deg', latex: '^{\\circ}' },
    { label: '†', hwp: 'dagger', latex: '\\dagger' },
    { label: '‡', hwp: 'ddagger', latex: '\\ddagger' },
    { label: '★', hwp: 'star', latex: '\\star' },
    { label: '℃', hwp: 'CENTIGRADE', latex: '^{\\circ}\\text{C}' },
    { label: '′', hwp: 'prime', latex: "'" },
  ] },
];

function buildCommandList(): CommandEntry[] {
  const cmds: CommandEntry[] = [];
  for (const grp of TEMPLATE_GROUPS) {
    for (const t of grp.items) {
      cmds.push({ name: t.hwp.split(/[\s{}_^]/)[0], display: t.label, insert: t.hwp, group: grp.name });
    }
  }
  return cmds;
}

const ALL_COMMANDS = buildCommandList();

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
  private built = false;

  // 모드
  private mode: InputMode = 'hwp';
  private modeBtn!: HTMLButtonElement;
  private latexHint!: HTMLDivElement;

  // 탭 UI
  private tabContainer!: HTMLDivElement;
  private toolbarContent!: HTMLDivElement;
  private activeTabId = 'struct';

  // 자동완성
  private acDropdown!: HTMLDivElement;
  private acItems: CommandEntry[] = [];
  private acSelectedIdx = -1;
  private acVisible = false;

  // 기호 검색
  private searchInput!: HTMLInputElement;
  private searchResults!: HTMLDivElement;

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

    try {
      this.origProps = this.wasm.getEquationProperties(sec, para, ci, cellIdx, cellParaIdx);
    } catch (err) {
      console.warn('[EquationEditor] 수식 속성 가져오기 실패:', err);
      return;
    }

    this.scriptArea.value = this.origProps.script || '';
    this.fontSizeInput.value = String(Math.round(this.origProps.fontSize / 100));
    this.colorInput.value = colorRefToHex(this.origProps.color);

    // 기존 스크립트에 \ 가 있으면 LaTeX 모드로 시작
    if (this.origProps.script && /\\[a-zA-Z]/.test(this.origProps.script)) {
      this.setMode('latex');
    } else {
      this.setMode('hwp');
    }

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
    this.hideAutocomplete();
    this.overlay?.remove();
  }

  /** 한 번만 DOM 구성 */
  private build(): void {
    if (this.built) return;
    this.built = true;

    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap eq-dialog';

    // 타이틀바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';

    const titleText = document.createElement('span');
    titleText.textContent = '수식 편집';

    // 모드 토글 버튼
    this.modeBtn = document.createElement('button');
    this.modeBtn.className = 'eq-mode-btn';
    this.modeBtn.addEventListener('click', () => this.toggleMode());

    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '×';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.append(titleText, this.modeBtn, closeBtn);

    // 본문
    const body = document.createElement('div');
    body.className = 'dialog-body eq-body';

    // 1) 탭 + 도구 모음
    this.tabContainer = document.createElement('div');
    this.tabContainer.className = 'eq-tabs';
    this.toolbarContent = document.createElement('div');
    this.toolbarContent.className = 'eq-toolbar';
    this.buildTabs();
    body.append(this.tabContainer, this.toolbarContent);

    // 2) 기호 검색
    const searchRow = document.createElement('div');
    searchRow.className = 'eq-search-row';
    this.searchInput = document.createElement('input');
    this.searchInput.type = 'text';
    this.searchInput.className = 'eq-search-input';
    this.searchInput.placeholder = '기호 검색 (이름 또는 유니코드)';
    this.searchInput.addEventListener('input', () => this.onSearchInput());
    this.searchResults = document.createElement('div');
    this.searchResults.className = 'eq-search-results';
    this.searchResults.style.display = 'none';
    searchRow.append(this.searchInput, this.searchResults);
    body.appendChild(searchRow);

    // 3) LaTeX 전환 힌트
    this.latexHint = document.createElement('div');
    this.latexHint.className = 'eq-latex-hint';
    this.latexHint.style.display = 'none';
    this.latexHint.innerHTML = '<span>💡 백슬래시(\\) 명령어가 감지됨 — </span>';
    const hintLink = document.createElement('a');
    hintLink.href = '#';
    hintLink.textContent = 'LaTeX 모드로 전환';
    hintLink.addEventListener('click', (e) => { e.preventDefault(); this.setMode('latex'); });
    this.latexHint.appendChild(hintLink);
    body.appendChild(this.latexHint);

    // 4) 미리보기 영역
    this.previewContainer = document.createElement('div');
    this.previewContainer.className = 'eq-preview';
    body.appendChild(this.previewContainer);

    // 5) 텍스트 입력 영역 (자동완성 포함)
    const scriptWrap = document.createElement('div');
    scriptWrap.className = 'eq-script-wrap';
    this.scriptArea = document.createElement('textarea');
    this.scriptArea.className = 'eq-script';
    this.scriptArea.rows = 4;
    this.scriptArea.spellcheck = false;
    this.scriptArea.addEventListener('input', () => { this.schedulePreview(); this.onScriptInput(); });
    this.scriptArea.addEventListener('keydown', (e) => this.onScriptKeydown(e));

    this.acDropdown = document.createElement('div');
    this.acDropdown.className = 'eq-autocomplete';
    this.acDropdown.style.display = 'none';

    scriptWrap.append(this.scriptArea, this.acDropdown);
    body.appendChild(scriptWrap);

    // 6) 속성 행
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

    // 7) 버튼 영역
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

    this.dialog.append(titleBar, body, footer);
    this.overlay.appendChild(this.dialog);

    // 키보드
    this.overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        if (this.acVisible) { this.hideAutocomplete(); e.stopPropagation(); return; }
        e.stopPropagation();
        this.hide();
      }
      if (e.key === 'Enter' && e.ctrlKey) {
        e.preventDefault();
        e.stopPropagation();
        this.handleOk();
      }
      e.stopPropagation();
    });

    this.overlay.addEventListener('mousedown', (e) => {
      if (e.target === this.overlay) this.hide();
    });

    this.enableDrag(titleBar);
  }

  // ── 모드 ──────────────────────────────────────

  private setMode(m: InputMode): void {
    this.mode = m;
    this.modeBtn.textContent = m === 'hwp' ? 'HWP' : 'LaTeX';
    this.modeBtn.title = m === 'hwp' ? 'LaTeX 모드로 전환' : 'HWP 모드로 전환';
    this.latexHint.style.display = 'none';
    this.refreshToolbar();
  }

  private toggleMode(): void {
    this.setMode(this.mode === 'hwp' ? 'latex' : 'hwp');
  }

  // ── 탭 ────────────────────────────────────────

  private buildTabs(): void {
    this.tabContainer.replaceChildren();
    for (const grp of TEMPLATE_GROUPS) {
      const tab = document.createElement('button');
      tab.className = 'eq-tab' + (grp.id === this.activeTabId ? ' eq-tab-active' : '');
      tab.textContent = grp.name;
      tab.dataset.tabId = grp.id;
      tab.addEventListener('click', () => {
        this.activeTabId = grp.id;
        this.tabContainer.querySelectorAll('.eq-tab').forEach(t => t.classList.remove('eq-tab-active'));
        tab.classList.add('eq-tab-active');
        this.refreshToolbar();
      });
      this.tabContainer.appendChild(tab);
    }
    this.refreshToolbar();
  }

  private refreshToolbar(): void {
    this.toolbarContent.replaceChildren();
    const grp = TEMPLATE_GROUPS.find(g => g.id === this.activeTabId);
    if (!grp) return;
    for (const tmpl of grp.items) {
      const btn = document.createElement('button');
      btn.className = 'eq-toolbar-btn';
      btn.textContent = tmpl.label;
      const script = this.mode === 'hwp' ? tmpl.hwp : tmpl.latex;
      btn.title = script;
      btn.addEventListener('click', () => this.insertTemplate(script));
      this.toolbarContent.appendChild(btn);
    }
  }

  // ── 자동완성 ──────────────────────────────────

  private onScriptInput(): void {
    const ta = this.scriptArea;
    const pos = ta.selectionStart;
    const text = ta.value.substring(0, pos);

    // HWP 모드: 백슬래시 감지 시 힌트
    if (this.mode === 'hwp' && /\\[a-zA-Z]/.test(text)) {
      this.latexHint.style.display = '';
    } else {
      this.latexHint.style.display = 'none';
    }

    // 현재 입력 중인 단어 추출
    const wordMatch = text.match(/([a-zA-Z]{2,})$/);
    if (!wordMatch) { this.hideAutocomplete(); return; }
    const word = wordMatch[1].toLowerCase();

    const matches = ALL_COMMANDS.filter(c =>
      c.name.toLowerCase().startsWith(word) || c.display.includes(word)
    ).slice(0, 8);

    if (matches.length === 0) { this.hideAutocomplete(); return; }

    this.acItems = matches;
    this.acSelectedIdx = 0;
    this.renderAutocomplete();
  }

  private renderAutocomplete(): void {
    this.acDropdown.replaceChildren();
    this.acItems.forEach((cmd, i) => {
      const row = document.createElement('div');
      row.className = 'eq-ac-item' + (i === this.acSelectedIdx ? ' eq-ac-selected' : '');
      row.innerHTML = `<span class="eq-ac-display">${cmd.display}</span> <span class="eq-ac-name">${cmd.name}</span> <span class="eq-ac-group">${cmd.group}</span>`;
      row.addEventListener('mousedown', (e) => { e.preventDefault(); this.acceptAutocomplete(i); });
      this.acDropdown.appendChild(row);
    });
    this.acDropdown.style.display = '';
    this.acVisible = true;
  }

  private acceptAutocomplete(idx: number): void {
    const cmd = this.acItems[idx];
    if (!cmd) return;
    const ta = this.scriptArea;
    const pos = ta.selectionStart;
    const text = ta.value;
    const before = text.substring(0, pos);
    const match = before.match(/([a-zA-Z]+)$/);
    if (!match) return;
    const wordStart = pos - match[1].length;
    const script = this.mode === 'hwp' ? cmd.insert : (cmd.insert.startsWith('\\') ? cmd.insert : '\\' + cmd.insert);
    ta.value = text.substring(0, wordStart) + script + text.substring(pos);
    const newPos = wordStart + script.length;
    ta.selectionStart = newPos;
    ta.selectionEnd = newPos;
    ta.focus();
    this.hideAutocomplete();
    this.schedulePreview();
  }

  private hideAutocomplete(): void {
    this.acDropdown.style.display = 'none';
    this.acVisible = false;
    this.acItems = [];
    this.acSelectedIdx = -1;
  }

  private onScriptKeydown(e: KeyboardEvent): void {
    if (!this.acVisible) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      this.acSelectedIdx = Math.min(this.acSelectedIdx + 1, this.acItems.length - 1);
      this.renderAutocomplete();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      this.acSelectedIdx = Math.max(this.acSelectedIdx - 1, 0);
      this.renderAutocomplete();
    } else if (e.key === 'Tab' || e.key === 'Enter') {
      if (this.acSelectedIdx >= 0) {
        e.preventDefault();
        this.acceptAutocomplete(this.acSelectedIdx);
      }
    } else if (e.key === 'Escape') {
      this.hideAutocomplete();
    }
  }

  // ── 기호 검색 ─────────────────────────────────

  private onSearchInput(): void {
    const q = this.searchInput.value.trim().toLowerCase();
    if (q.length < 1) { this.searchResults.style.display = 'none'; return; }

    const matches = ALL_COMMANDS.filter(c =>
      c.name.toLowerCase().includes(q) ||
      c.display.includes(q) ||
      c.group.toLowerCase().includes(q)
    ).slice(0, 12);

    if (matches.length === 0) {
      this.searchResults.style.display = 'none';
      return;
    }

    this.searchResults.replaceChildren();
    for (const cmd of matches) {
      const btn = document.createElement('button');
      btn.className = 'eq-search-result-btn';
      btn.innerHTML = `<span class="eq-sr-display">${cmd.display}</span><span class="eq-sr-name">${cmd.name}</span>`;
      btn.addEventListener('click', () => {
        const script = this.mode === 'hwp' ? cmd.insert : (cmd.insert.startsWith('\\') ? cmd.insert : '\\' + cmd.insert);
        this.insertTemplate(script);
        this.searchInput.value = '';
        this.searchResults.style.display = 'none';
      });
      this.searchResults.appendChild(btn);
    }
    this.searchResults.style.display = '';
  }

  // ── 템플릿 삽입 ───────────────────────────────

  private insertTemplate(script: string): void {
    const ta = this.scriptArea;
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const before = ta.value.substring(0, start);
    const after = ta.value.substring(end);

    const needSpaceBefore = before.length > 0 && !/\s$/.test(before);
    const needSpaceAfter = after.length > 0 && !/^\s/.test(after);
    const insertion = (needSpaceBefore ? ' ' : '') + script + (needSpaceAfter ? ' ' : '');

    ta.value = before + insertion + after;

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

  // ── 미리보기 ──────────────────────────────────

  private schedulePreview(): void {
    if (this.previewTimer) clearTimeout(this.previewTimer);
    this.previewTimer = setTimeout(() => this.updatePreview(), 300);
  }

  private updatePreview(): void {
    const script = this.scriptArea.value.trim();
    if (!script) {
      this.showPreviewMessage('eq-preview-empty', '수식을 입력하세요');
      return;
    }

    const fontSizePt = parseInt(this.fontSizeInput.value, 10) || 10;
    const fontSizeHwpunit = fontSizePt * 100;
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

  // ── 확인 ──────────────────────────────────────

  private handleOk(): void {
    if (!this.origProps) return;

    const script = this.scriptArea.value;
    const fontSizePt = parseInt(this.fontSizeInput.value, 10) || 10;
    const fontSizeHwpunit = fontSizePt * 100;
    const color = hexToColorRef(this.colorInput.value);

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

  // ── 드래그 ────────────────────────────────────

  private enableDrag(titleEl: HTMLElement): void {
    let offsetX = 0, offsetY = 0, isDragging = false;
    titleEl.addEventListener('mousedown', (e) => {
      if ((e.target as HTMLElement).closest('.dialog-close, .eq-mode-btn')) return;
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
