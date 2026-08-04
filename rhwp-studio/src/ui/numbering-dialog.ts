/**
 * 문단 번호 모양 대화상자 (NumberingDialog)
 *
 *  ┌─────────────────────────────────────────────┐
 *  │ 문단 번호 모양                          [×]  │
 *  ├─────────────────────────────────────────────┤
 *  │ ─ 번호 형식 ─────────────────────────────── │
 *  │  ○ 1. 가. 1) 가) (1) (가) ①               │
 *  │  ○ 1. 2. 3. 4. 5. 6. 7.                   │
 *  │  ○ I. A. 1. a. (1) (a) ①                  │
 *  │  ○ 제1장 제1절 1. 가. 1) 가) (1)           │
 *  │  ○ 一 二 三 (한자)                         │
 *  │  ○ ① ② ③ (원문자)                        │
 *  │                                             │
 *  │ ─ 시작 번호 ─ ┌──────────────────────────┐ │
 *  │  [  1  ]       │  수준 1: 1.              │ │
 *  │                │  수준 2:   가.           │ │
 *  │                │  수준 3:     1)          │ │
 *  │                │  수준 4:       가)       │ │
 *  │                │  수준 5:         (1)     │ │
 *  │                │  수준 6:           (가)  │ │
 *  │                │  수준 7:             ①   │ │
 *  │                └──────────────────────────┘ │
 *  ├─────────────────────────────────────────────┤
 *  │                        [확인]  [취소]        │
 *  └─────────────────────────────────────────────┘
 */

import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import { ModalDialog } from './dialog';
import { BULLET_PRESETS } from '@/core/numbering-defaults';

/** 번호 형식 코드 (HWP 표 43) */
const NUM_FMT = {
  DIGIT: 0,       // 1, 2, 3
  CIRCLE_DIGIT: 1,// ①, ②, ③
  ROMAN_UPPER: 2, // I, II, III
  ROMAN_LOWER: 3, // i, ii, iii
  ALPHA_UPPER: 4, // A, B, C
  ALPHA_LOWER: 5, // a, b, c
  HANGUL_SYLLABLE: 6, // 가, 나, 다 (음절)
  HANGUL_JAMO: 10,    // ㄱ, ㄴ, ㄷ (자모)
  HANGUL_MIXED: 8,    // 가, 나, 다 (혼합)
  HANJA: 7,           // 一, 二, 三
};

/** 프리셋 정의 */
interface NumberingPreset {
  label: string;
  levelFormats: string[];
  numberFormats: number[];
}

const PRESETS: NumberingPreset[] = [
  {
    label: '(없음)',
    levelFormats: ['', '', '', '', '', '', ''],
    numberFormats: [0, 0, 0, 0, 0, 0, 0],
  },
  {
    label: '1. 가. 1) 가) (1) (가) ①',
    levelFormats: ['^1.', '^2.', '^3)', '^4)', '(^5)', '(^6)', '^7'],
    numberFormats: [NUM_FMT.DIGIT, NUM_FMT.HANGUL_MIXED, NUM_FMT.DIGIT, NUM_FMT.HANGUL_MIXED, NUM_FMT.DIGIT, NUM_FMT.HANGUL_MIXED, NUM_FMT.CIRCLE_DIGIT],
  },
  {
    label: '1. 2. 3. 4. 5. 6. 7.',
    levelFormats: ['^1.', '^2.', '^3.', '^4.', '^5.', '^6.', '^7.'],
    numberFormats: [NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT],
  },
  {
    label: 'I. A. 1. a. (1) (a) ①',
    levelFormats: ['^1.', '^2.', '^3.', '^4.', '(^5)', '(^6)', '^7'],
    numberFormats: [NUM_FMT.ROMAN_UPPER, NUM_FMT.ALPHA_UPPER, NUM_FMT.DIGIT, NUM_FMT.ALPHA_LOWER, NUM_FMT.DIGIT, NUM_FMT.ALPHA_LOWER, NUM_FMT.CIRCLE_DIGIT],
  },
  {
    label: '제1장 제1절 1. 가. 1) 가) (1)',
    levelFormats: ['제^1장', '제^2절', '^3.', '^4.', '^5)', '^6)', '(^7)'],
    numberFormats: [NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.DIGIT, NUM_FMT.HANGUL_MIXED, NUM_FMT.DIGIT, NUM_FMT.HANGUL_MIXED, NUM_FMT.DIGIT],
  },
  {
    label: '一 二 三 (한자)',
    levelFormats: ['^1', '^2', '^3', '^4', '^5', '^6', '^7'],
    numberFormats: [NUM_FMT.HANJA, NUM_FMT.HANJA, NUM_FMT.HANJA, NUM_FMT.HANJA, NUM_FMT.HANJA, NUM_FMT.HANJA, NUM_FMT.HANJA],
  },
  {
    label: '① ② ③ (원문자)',
    levelFormats: ['^1', '^2', '^3', '^4', '^5', '^6', '^7'],
    numberFormats: [NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT, NUM_FMT.CIRCLE_DIGIT],
  },
];

/** 번호 형식 코드로 샘플 번호 문자열 생성 */
function formatNumber(num: number, fmt: number): string {
  switch (fmt) {
    case NUM_FMT.DIGIT: return String(num);
    case NUM_FMT.CIRCLE_DIGIT: {
      const circled = '①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳';
      return num >= 1 && num <= 20 ? circled[num - 1] : String(num);
    }
    case NUM_FMT.ROMAN_UPPER: return toRoman(num);
    case NUM_FMT.ROMAN_LOWER: return toRoman(num).toLowerCase();
    case NUM_FMT.ALPHA_UPPER: return num >= 1 && num <= 26 ? String.fromCharCode(64 + num) : String(num);
    case NUM_FMT.ALPHA_LOWER: return num >= 1 && num <= 26 ? String.fromCharCode(96 + num) : String(num);
    case NUM_FMT.HANGUL_SYLLABLE:
    case NUM_FMT.HANGUL_MIXED: {
      const hangul = '가나다라마바사아자차카타파하';
      return num >= 1 && num <= 14 ? hangul[num - 1] : String(num);
    }
    case NUM_FMT.HANGUL_JAMO: {
      const jamo = 'ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎ';
      return num >= 1 && num <= 14 ? jamo[num - 1] : String(num);
    }
    case NUM_FMT.HANJA: {
      const hanja = '一二三四五六七八九十';
      return num >= 1 && num <= 10 ? hanja[num - 1] : String(num);
    }
    default: return String(num);
  }
}

function toRoman(n: number): string {
  const vals = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
  const syms = ['M', 'CM', 'D', 'CD', 'C', 'XC', 'L', 'XL', 'X', 'IX', 'V', 'IV', 'I'];
  let result = '';
  let num = n;
  for (let i = 0; i < vals.length; i++) {
    while (num >= vals[i]) {
      result += syms[i];
      num -= vals[i];
    }
  }
  return result;
}

/** 프리셋의 미리보기 텍스트 생성 (7수준) */
function generatePreview(preset: NumberingPreset, startNumber: number): string {
  const lines: string[] = [];
  for (let level = 0; level < 7; level++) {
    const indent = '  '.repeat(level);
    const fmt = preset.levelFormats[level];
    const numStr = formatNumber(startNumber, preset.numberFormats[level]);
    // ^(level+1)을 numStr로 치환
    const marker = fmt.replace(`^${level + 1}`, numStr);
    lines.push(`${indent}수준 ${level + 1}: ${marker}`);
  }
  return lines.join('\n');
}

export class NumberingDialog extends ModalDialog {
  private selectedPreset = 0;
  private startNumber = 1;
  /** 시작 번호 방식: 0=앞 번호 이어, 1=이전 번호 이어, 2=새 번호 시작 */
  private restartMode = 0;
  private previewEl!: HTMLPreElement;
  private radioButtons: HTMLInputElement[] = [];

  onApply?: (numberingId: number, restartMode: number, startNumber: number) => void;
  /** 글머리표 적용 콜백 */
  onApplyBullet?: (bulletChar: string) => void;
  onClose?: () => void;

  /** 현재 문단의 번호 정보 (대화상자 초기값 설정용) */
  currentHeadType: string = 'None';
  currentNumberingId: number = 0;
  currentRestartMode: number = 0;
  /** 현재 글머리표 문자 (Bullet 모드일 때) */
  currentBulletChar: string = '';

  /** 현재 활성 탭: 'number' | 'bullet' */
  private activeTab: 'number' | 'bullet' = 'number';
  private numberPanel!: HTMLElement;
  private bulletPanel!: HTMLElement;
  private selectedBulletIdx = -1;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
  ) {
    super('문단 번호/글머리표', 480);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'nd-body';

    // ─── 탭 바 ───
    const tabBar = document.createElement('div');
    tabBar.className = 'nd-tab-bar';
    const tabNumber = document.createElement('button');
    tabNumber.className = 'nd-tab active';
    tabNumber.textContent = '문단 번호';
    tabNumber.addEventListener('click', () => this.switchTab('number'));
    const tabBullet = document.createElement('button');
    tabBullet.className = 'nd-tab';
    tabBullet.textContent = '글머리표';
    tabBullet.addEventListener('click', () => this.switchTab('bullet'));
    tabBar.appendChild(tabNumber);
    tabBar.appendChild(tabBullet);
    body.appendChild(tabBar);

    // ─── 문단 번호 패널 ───
    this.numberPanel = document.createElement('div');
    this.numberPanel.className = 'nd-panel';
    this.buildNumberPanel(this.numberPanel);
    body.appendChild(this.numberPanel);

    // ─── 글머리표 패널 ───
    this.bulletPanel = document.createElement('div');
    this.bulletPanel.className = 'nd-panel';
    this.bulletPanel.style.display = 'none';
    this.buildBulletPanel(this.bulletPanel);
    body.appendChild(this.bulletPanel);

    // 현재 headType에 따라 초기 탭 선택
    if (this.currentHeadType === 'Bullet') {
      this.switchTab('bullet');
      tabBullet.classList.add('active');
      tabNumber.classList.remove('active');
    }

    return body;
  }

  private switchTab(tab: 'number' | 'bullet'): void {
    this.activeTab = tab;
    this.numberPanel.style.display = tab === 'number' ? '' : 'none';
    this.bulletPanel.style.display = tab === 'bullet' ? '' : 'none';
    // 탭 버튼 active 클래스 토글
    const tabs = this.numberPanel.parentElement?.querySelectorAll('.nd-tab');
    tabs?.forEach((t, i) => {
      t.classList.toggle('active', (i === 0 && tab === 'number') || (i === 1 && tab === 'bullet'));
    });
  }

  private buildBulletPanel(panel: HTMLElement): void {
    
    const grid = document.createElement('div');
    grid.className = 'nd-bullet-grid';

    // (없음)
    const noneCell = document.createElement('div');
    noneCell.className = 'nd-bullet-cell';
    noneCell.textContent = '(없음)';
    noneCell.addEventListener('click', () => {
      grid.querySelectorAll('.nd-bullet-cell').forEach(c => c.classList.remove('selected'));
      noneCell.classList.add('selected');
      this.selectedBulletIdx = -1;
    });
    grid.appendChild(noneCell);

    for (let i = 0; i < BULLET_PRESETS.length; i++) {
      const preset = BULLET_PRESETS[i];
      const cell = document.createElement('div');
      cell.className = 'nd-bullet-cell';
      // 현재 bullet 문자와 매칭되면 선택 상태
      if (this.currentBulletChar && preset.char === this.currentBulletChar) {
        cell.classList.add('selected');
        this.selectedBulletIdx = i;
      }
      const char = document.createElement('span');
      char.className = 'nd-bullet-char';
      char.textContent = preset.displayChar;
      cell.appendChild(char);
      cell.title = preset.name;
      cell.addEventListener('click', () => {
        grid.querySelectorAll('.nd-bullet-cell').forEach(c => c.classList.remove('selected'));
        cell.classList.add('selected');
        this.selectedBulletIdx = i;
      });
      grid.appendChild(cell);
    }
    panel.appendChild(grid);
  }

  private buildNumberPanel(panel: HTMLElement): void {

    // 번호 형식 섹션
    const fmtSection = document.createElement('div');
    fmtSection.className = 'dialog-section';
    const fmtTitle = document.createElement('div');
    fmtTitle.className = 'dialog-section-title';
    fmtTitle.textContent = '번호 형식';
    fmtSection.appendChild(fmtTitle);

    const radioGroup = document.createElement('div');
    radioGroup.className = 'nd-radio-group';
    // 현재 문단 상태에 따라 초기 선택 결정
    const initialPreset = (this.currentHeadType === 'None' || this.currentNumberingId === 0) ? 0 : 1;
    this.selectedPreset = initialPreset;
    this.restartMode = this.currentRestartMode;

    for (let i = 0; i < PRESETS.length; i++) {
      const preset = PRESETS[i];
      const lbl = document.createElement('label');
      lbl.className = 'nd-radio-label';
      const radio = document.createElement('input');
      radio.type = 'radio';
      radio.name = 'nd-preset';
      radio.value = String(i);
      if (i === initialPreset) radio.checked = true;
      radio.addEventListener('change', () => {
        this.selectedPreset = i;
        this.updatePreview();
      });
      this.radioButtons.push(radio);
      lbl.appendChild(radio);
      lbl.appendChild(document.createTextNode(` ${preset.label}`));
      radioGroup.appendChild(lbl);
    }
    fmtSection.appendChild(radioGroup);
    panel.appendChild(fmtSection);

    // 시작 번호 방식 섹션
    const restartSection = document.createElement('div');
    restartSection.className = 'dialog-section';
    const restartTitle = document.createElement('div');
    restartTitle.className = 'dialog-section-title';
    restartTitle.textContent = '시작 번호 방식';
    restartSection.appendChild(restartTitle);

    const restartRadioGroup = document.createElement('div');
    restartRadioGroup.className = 'nd-restart-group';
    const restartModes = [
      { value: '0', label: '앞 번호 목록에 이어(C)', checked: this.restartMode === 0 },
      { value: '1', label: '이전 번호 목록에 이어(P)', checked: this.restartMode === 1 },
      { value: '2', label: '새 번호 목록 시작(G)', checked: this.restartMode === 2 },
    ];
    for (const mode of restartModes) {
      const lbl = document.createElement('label');
      lbl.className = 'nd-radio-label';
      lbl.style.display = 'block';
      lbl.style.marginBottom = '4px';
      const radio = document.createElement('input');
      radio.type = 'radio';
      radio.name = 'nd-restart-mode';
      radio.value = mode.value;
      radio.checked = mode.checked;
      radio.addEventListener('change', () => {
        this.restartMode = parseInt(radio.value);
        startInput.disabled = this.restartMode !== 2;
      });
      lbl.appendChild(radio);
      lbl.appendChild(document.createTextNode(` ${mode.label}`));
      restartRadioGroup.appendChild(lbl);
    }
    restartSection.appendChild(restartRadioGroup);
    panel.appendChild(restartSection);

    // 하단: 시작 번호 + 미리보기
    const bottomRow = document.createElement('div');
    bottomRow.className = 'nd-bottom-row';

    // 시작 번호 (새 번호 목록 시작 시에만 활성)
    const startSection = document.createElement('div');
    startSection.className = 'nd-start-section';
    const startLabel = document.createElement('label');
    startLabel.className = 'dialog-label';
    startLabel.textContent = '시작 번호';
    const startInput = document.createElement('input');
    startInput.type = 'number';
    startInput.className = 'dialog-input';
    startInput.value = '1';
    startInput.min = '1';
    startInput.max = '999';
    startInput.style.width = '60px';
    startInput.disabled = this.restartMode !== 2;
    startInput.addEventListener('input', () => {
      const parsed = parseInt(startInput.value) || 1;
      // WASM createNumbering으로 넘어가기 전에 min/max(1~999) 범위로 clamp한다.
      // (input의 min/max HTML 속성은 이 다이얼로그에 <form> submit 흐름이 없어 강제되지 않음)
      this.startNumber = Math.min(999, Math.max(1, parsed));
      this.updatePreview();
    });
    startSection.appendChild(startLabel);
    startSection.appendChild(startInput);
    bottomRow.appendChild(startSection);

    // 미리보기
    const previewSection = document.createElement('div');
    previewSection.className = 'nd-preview-section';
    const previewTitle = document.createElement('div');
    previewTitle.className = 'dialog-section-title';
    previewTitle.textContent = '미리보기';
    this.previewEl = document.createElement('pre');
    this.previewEl.className = 'nd-preview';
    previewSection.appendChild(previewTitle);
    previewSection.appendChild(this.previewEl);
    bottomRow.appendChild(previewSection);

    panel.appendChild(bottomRow);
    this.updatePreview();
  }

  private updatePreview(): void {
    if (!this.previewEl) return;
    const preset = PRESETS[this.selectedPreset];
    this.previewEl.textContent = generatePreview(preset, this.startNumber);
  }

  protected onConfirm(): void {
    if (this.activeTab === 'bullet') {
      // 글머리표 탭
      if (this.selectedBulletIdx < 0) {
        // "(없음)": 글머리표 해제
        this.onApply?.(0, 0, 1);
      } else {
        
        const preset = BULLET_PRESETS[this.selectedBulletIdx];
        this.onApplyBullet?.(preset.char);
      }
      return;
    }
    // 문단 번호 탭
    if (this.selectedPreset === 0) {
      // "(없음)" 선택: 번호 해제
      this.onApply?.(0, 0, 1);
      return;
    }
    const preset = PRESETS[this.selectedPreset];
    const json = JSON.stringify({
      levelFormats: preset.levelFormats,
      numberFormats: preset.numberFormats,
      startNumber: this.startNumber,
    });
    const nid = this.wasm.createNumbering(json);
    if (nid > 0) {
      this.onApply?.(nid, this.restartMode, this.startNumber);
    }
  }

  override hide(): void {
    super.hide();
    this.onClose?.();
  }

  show(): void {
    super.show();
  }
}
