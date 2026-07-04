/**
 * 글자 모양 대화상자 (CharShapeDialog)
 * HWP Alt+L 대화상자에 해당하는 글자 속성 설정 UI
 *
 * 레이아웃: HWP 원본 대화상자를 최대한 재현
 *  ┌─────────────────────────────────────────────┐
 *  │ 글자 모양                              [×]  │
 *  ├────────────────────────────────┬─────────────┤
 *  │ [기본] [확장] [테두리/배경]   │  [설정(D)]  │
 *  │                                │  [취  소]   │
 *  │  기준 크기(Z): [10.0] pt ▾    │             │
 *  │                                │             │
 *  │ ─ 언어별 설정 ─────────────── │             │
 *  │ 언어(L): [대표 ▾] 글꼴(T): [맑은 고딕 ▾]  │
 *  │ 상대 크기(B): [100] %  장평(W): [100] %    │
 *  │ 글자 위치(E): [0]   %  자간(P): [0]   %    │
 *  │                                │             │
 *  │ ─ 속성 ────────────────────── │             │
 *  │ [가][가][가][가][가][가][가][가][가ˇ][가ˎ]  │
 *  │ 글자 색(C): [■]  음영 색(G): [색 없음 ▾]  │
 *  │                                │             │
 *  │ ┌─────────────────────────┐   │             │
 *  │ │ 한글Eng123漢字あいう※○ │   │             │
 *  │ └─────────────────────────┘   │             │
 *  └────────────────────────────────┴─────────────┘
 */

import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { CharProperties } from '@/core/types';
import { REGISTERED_FONTS } from '@/core/font-loader';
import { getLocalFonts } from '@/core/local-fonts';
import { enableDialogDrag } from './dialog-drag';

const LANG_NAMES = ['대표', '한글', '영문', '한자', '일어', '외국어', '기호', '사용자'];

/** 웹폰트 + 로컬 글꼴을 합친 목록 (정렬됨) */
function buildFontList(): string[] {
  const fonts = Array.from(REGISTERED_FONTS).sort((a, b) => a.localeCompare(b, 'ko'));
  return fonts;
}

/** 속성 아이콘 정의: HWP 원본의 가 문자 변형 */
const ATTR_ICONS: { id: string; title: string }[] = [
  { id: 'bold',          title: '굵게' },
  { id: 'italic',        title: '기울임' },
  { id: 'underline',     title: '밑줄' },
  { id: 'strikethrough', title: '취소선' },
  { id: 'outline',       title: '외곽선' },
  { id: 'shadow',        title: '그림자' },
  { id: 'superscript',   title: '위 첨자' },
  { id: 'subscript',     title: '아래 첨자' },
];

function createAttrIconContent(id: string): HTMLSpanElement {
  const span = document.createElement('span');
  span.textContent = '가';
  switch (id) {
    case 'bold':
      span.style.fontWeight = 'bold';
      break;
    case 'italic':
      span.style.fontStyle = 'italic';
      break;
    case 'underline':
      span.style.textDecoration = 'underline';
      break;
    case 'strikethrough':
      span.style.textDecoration = 'line-through';
      break;
    case 'outline':
      span.style.color = 'transparent';
      span.style.setProperty('-webkit-text-stroke', '1px #333');
      break;
    case 'shadow':
      span.style.textShadow = '2px 2px 0 #999';
      break;
    case 'superscript': {
      span.textContent = '';
      span.style.fontSize = '12px';
      span.appendChild(document.createTextNode('가'));
      const sup = document.createElement('sup');
      sup.style.fontSize = '8px';
      sup.textContent = '1';
      span.appendChild(sup);
      break;
    }
    case 'subscript': {
      span.textContent = '';
      span.style.fontSize = '12px';
      span.appendChild(document.createTextNode('가'));
      const sub = document.createElement('sub');
      sub.style.fontSize = '8px';
      sub.textContent = '1';
      span.appendChild(sub);
      break;
    }
  }
  return span;
}

export class CharShapeDialog {
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private built = false;

  // 탭
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];

  // 기본 탭 컨트롤
  private baseSizeInput!: HTMLInputElement;
  private langSelect!: HTMLSelectElement;
  private fontSelect!: HTMLSelectElement;
  private langInputs: Record<string, HTMLInputElement> = {};
  private attrBtns: Record<string, HTMLButtonElement> = {};
  private textColorInput!: HTMLInputElement;
  private shadeColorInput!: HTMLInputElement;
  private previewEl!: HTMLDivElement;

  // 확장 탭 컨트롤
  private shadowRadios!: HTMLInputElement[];       // 없음/비연속/연속
  private shadowColorInput!: HTMLInputElement;
  private shadowXInput!: HTMLInputElement;
  private shadowYInput!: HTMLInputElement;
  private ulPosSelect!: HTMLSelectElement;          // 밑줄 위치
  private ulShapeSelect!: HTMLSelectElement;        // 밑줄 모양
  private ulColorInput!: HTMLInputElement;          // 밑줄 색
  private strikeShapeSelect!: HTMLSelectElement;    // 취소선 모양
  private strikeColorInput!: HTMLInputElement;
  private outlineTypeSelect!: HTMLSelectElement;
  private emphasisSelect!: HTMLSelectElement;       // 강조점
  private kerningCheckbox!: HTMLInputElement;       // 커닝
  private extPreviewEl!: HTMLDivElement;            // 확장 탭 미리보기

  // 테두리/배경 탭 컨트롤
  private borderTypeSelect!: HTMLSelectElement;
  private borderWidthSelect!: HTMLSelectElement;
  private borderColorInput!: HTMLInputElement;
  private borderPreviewInner!: HTMLDivElement;
  private faceColorSelect!: HTMLSelectElement;
  private faceColorPicker!: HTMLInputElement;
  private patColorInput!: HTMLInputElement;
  private patShapeSelect!: HTMLSelectElement;

  private currentLang = 0;
  private props: CharProperties | null = null;
  private initialProps: CharProperties | null = null;

  /** 적용 콜백 */
  onApply: ((mods: Partial<CharProperties>) => void) | null = null;
  /** 대화상자 닫힘 콜백 (적용·취소·ESC 모두) */
  onClose: (() => void) | null = null;

  constructor(
    private wasm: WasmBridge,
    private eventBus: EventBus,
  ) {}

  show(charProps: CharProperties): void {
    this.build();
    this.props = JSON.parse(JSON.stringify(charProps));
    this.initialProps = JSON.parse(JSON.stringify(charProps));
    this.currentLang = 0;
    this.langSelect.value = '0';
    this.populateFromProps();
    this.switchTab(0);
    document.body.appendChild(this.overlay);
    setTimeout(() => this.baseSizeInput?.select(), 50);
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
    this.dialog.className = 'dialog-wrap cs-dialog';

    // 타이틀 바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '글자 모양';
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
    const tabNames = ['기본', '확장', '테두리/배경'];
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

    // ── 기준 크기
    const sizeRow = this.row();
    sizeRow.appendChild(this.label('기준 크기(Z):'));
    this.baseSizeInput = this.numberInput(1, 4096, 0.5);
    this.baseSizeInput.style.width = '60px';
    sizeRow.appendChild(this.baseSizeInput);
    sizeRow.appendChild(this.unit('pt'));
    panel.appendChild(sizeRow);

    // ── 언어별 설정
    const langSection = this.createFieldset('언어별 설정');

    // 언어 + 글꼴 (한 줄)
    const langFontRow = this.row();
    langFontRow.appendChild(this.label('언어(L):'));
    this.langSelect = document.createElement('select');
    this.langSelect.className = 'dialog-select';
    this.langSelect.style.width = '72px';
    LANG_NAMES.forEach((name, i) => {
      const opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = name;
      this.langSelect.appendChild(opt);
    });
    this.langSelect.addEventListener('change', () => {
      this.saveLangFields();
      this.currentLang = parseInt(this.langSelect.value);
      this.updateLangFields();
    });
    langFontRow.appendChild(this.langSelect);

    const fontLabel = this.label('글꼴(T):');
    fontLabel.style.marginLeft = '12px';
    langFontRow.appendChild(fontLabel);
    this.fontSelect = document.createElement('select');
    this.fontSelect.className = 'dialog-select';
    this.fontSelect.style.width = '140px';

    // 웹폰트 optgroup
    const webGroup = document.createElement('optgroup');
    webGroup.label = '웹 글꼴';
    buildFontList().forEach(f => {
      const opt = document.createElement('option');
      opt.value = f;
      opt.textContent = f;
      webGroup.appendChild(opt);
    });
    this.fontSelect.appendChild(webGroup);

    // 로컬 글꼴 optgroup (감지된 경우에만)
    const localFonts = getLocalFonts();
    if (localFonts.length > 0) {
      const localGroup = document.createElement('optgroup');
      localGroup.label = '로컬 글꼴';
      localFonts.forEach(f => {
        const opt = document.createElement('option');
        opt.value = f;
        opt.textContent = f;
        localGroup.appendChild(opt);
      });
      this.fontSelect.appendChild(localGroup);
    }
    langFontRow.appendChild(this.fontSelect);
    langSection.appendChild(langFontRow);

    // 상대 크기 + 장평 (2열)
    const row1 = this.row();
    row1.appendChild(this.label('상대 크기(B):'));
    this.langInputs['cs-relative-size'] = this.numberInput(10, 250);
    this.langInputs['cs-relative-size'].style.width = '50px';
    row1.appendChild(this.langInputs['cs-relative-size']);
    row1.appendChild(this.unit('%'));
    const ratioLabel = this.label('장평(W):');
    ratioLabel.style.marginLeft = '16px';
    row1.appendChild(ratioLabel);
    this.langInputs['cs-ratio'] = this.numberInput(50, 200);
    this.langInputs['cs-ratio'].style.width = '50px';
    row1.appendChild(this.langInputs['cs-ratio']);
    row1.appendChild(this.unit('%'));
    langSection.appendChild(row1);

    // 글자 위치 + 자간 (2열)
    const row2 = this.row();
    row2.appendChild(this.label('글자 위치(E):'));
    this.langInputs['cs-char-offset'] = this.numberInput(-100, 100);
    this.langInputs['cs-char-offset'].style.width = '50px';
    row2.appendChild(this.langInputs['cs-char-offset']);
    row2.appendChild(this.unit('%'));
    const spacingLabel = this.label('자간(P):');
    spacingLabel.style.marginLeft = '16px';
    row2.appendChild(spacingLabel);
    this.langInputs['cs-spacing'] = this.numberInput(-50, 50);
    this.langInputs['cs-spacing'].style.width = '50px';
    row2.appendChild(this.langInputs['cs-spacing']);
    row2.appendChild(this.unit('%'));
    langSection.appendChild(row2);
    panel.appendChild(langSection);

    // ── 속성
    const attrSection = this.createFieldset('속성');

    // 가 아이콘 버튼 행
    const attrRow = this.row();
    attrRow.style.gap = '2px';
    ATTR_ICONS.forEach(a => {
      const btn = document.createElement('button');
      btn.className = 'cs-icon-btn';
      btn.title = a.title;
      btn.appendChild(createAttrIconContent(a.id));
      btn.addEventListener('click', () => {
        if (a.id === 'superscript' && !btn.classList.contains('active')) {
          this.attrBtns['subscript']?.classList.remove('active');
        } else if (a.id === 'subscript' && !btn.classList.contains('active')) {
          this.attrBtns['superscript']?.classList.remove('active');
        }
        btn.classList.toggle('active');
        this.updatePreview();
      });
      attrRow.appendChild(btn);
      this.attrBtns[a.id] = btn;
    });
    attrSection.appendChild(attrRow);

    // 글자 색 + 음영 색
    const colorRow = this.row();
    colorRow.appendChild(this.label('글자 색(C):'));
    this.textColorInput = document.createElement('input');
    this.textColorInput.type = 'color';
    this.textColorInput.className = 'cs-color-btn';
    this.textColorInput.addEventListener('input', () => this.updatePreview());
    colorRow.appendChild(this.textColorInput);

    const shadeLabel = this.label('음영 색(G):');
    shadeLabel.style.marginLeft = '16px';
    colorRow.appendChild(shadeLabel);
    this.shadeColorInput = document.createElement('input');
    this.shadeColorInput.type = 'color';
    this.shadeColorInput.className = 'cs-color-btn';
    colorRow.appendChild(this.shadeColorInput);
    attrSection.appendChild(colorRow);
    panel.appendChild(attrSection);

    // ── 미리보기
    this.previewEl = document.createElement('div');
    this.previewEl.className = 'cs-preview';
    this.previewEl.textContent = '한글Eng123漢字あいう※○';
    panel.appendChild(this.previewEl);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  확장 탭
  // ════════════════════════════════════════════════════════

  private buildExtendedPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 그림자
    const shadowFs = this.createFieldset('그림자');

    // 라디오 버튼: 없음(N) / 비연속(U) / 연속(T)
    const radioRow = this.row();
    this.shadowRadios = [];
    const shadowOpts: [string, string, string][] = [
      ['0', '없음(N)', 'N'], ['1', '비연속(U)', 'U'], ['2', '연속(T)', 'T'],
    ];
    shadowOpts.forEach(([val, lbl, key]) => {
      const lb = document.createElement('label');
      lb.className = 'cs-radio-label';
      const radio = document.createElement('input');
      radio.type = 'radio';
      radio.name = 'cs-shadow-type';
      radio.value = val;
      radio.accessKey = key.toLowerCase();
      lb.appendChild(radio);
      lb.appendChild(document.createTextNode(` ${lbl}`));
      radioRow.appendChild(lb);
      this.shadowRadios.push(radio);
    });
    shadowFs.appendChild(radioRow);

    // X 방향 / Y 방향 / 색
    const xyRow = this.row();
    xyRow.appendChild(this.label('X 방향(X):'));
    this.shadowXInput = this.numberInput(-100, 100);
    this.shadowXInput.style.width = '42px';
    xyRow.appendChild(this.shadowXInput);
    xyRow.appendChild(this.unit('%'));
    const yLabel = this.label('Y 방향(Y):');
    yLabel.style.marginLeft = '10px';
    xyRow.appendChild(yLabel);
    this.shadowYInput = this.numberInput(-100, 100);
    this.shadowYInput.style.width = '42px';
    xyRow.appendChild(this.shadowYInput);
    xyRow.appendChild(this.unit('%'));
    const scLabel = this.label('색(C):');
    scLabel.style.marginLeft = '10px';
    xyRow.appendChild(scLabel);
    this.shadowColorInput = document.createElement('input');
    this.shadowColorInput.type = 'color';
    this.shadowColorInput.className = 'cs-color-btn';
    xyRow.appendChild(this.shadowColorInput);
    shadowFs.appendChild(xyRow);
    panel.appendChild(shadowFs);

    // ── 밑줄
    const ulFs = this.createFieldset('밑줄');
    const ulRow = this.row();
    ulRow.appendChild(this.label('위치(L):'));
    this.ulPosSelect = document.createElement('select');
    this.ulPosSelect.className = 'dialog-select';
    this.ulPosSelect.style.width = '68px';
    for (const [val, lbl] of [['None', '없음'], ['Bottom', '아래'], ['Top', '위']] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.ulPosSelect.appendChild(o);
    }
    ulRow.appendChild(this.ulPosSelect);

    const ulmLabel = this.label('모양(M):');
    ulmLabel.style.marginLeft = '10px';
    ulRow.appendChild(ulmLabel);
    this.ulShapeSelect = document.createElement('select');
    this.ulShapeSelect.className = 'dialog-select';
    this.ulShapeSelect.style.width = '90px';
    for (const [val, lbl] of [
      ['0', '━━━━ 실선'], ['1', '- - - 긴점선'], ['2', '········ 점선'],
      ['3', '━·━· 일점쇄선'], ['4', '━··━ 이점쇄선'],
      ['5', '━━━ 긴파선'], ['6', '●●●● 원형점'],
      ['7', '══ 이중선'], ['8', '━═ 가는+굵은'],
      ['9', '═━ 굵은+가는'], ['10', '≡≡ 삼중선'],
    ] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.ulShapeSelect.appendChild(o);
    }
    ulRow.appendChild(this.ulShapeSelect);

    const ulcLabel = this.label('색(B):');
    ulcLabel.style.marginLeft = '10px';
    ulRow.appendChild(ulcLabel);
    this.ulColorInput = document.createElement('input');
    this.ulColorInput.type = 'color';
    this.ulColorInput.className = 'cs-color-btn';
    ulRow.appendChild(this.ulColorInput);
    ulFs.appendChild(ulRow);
    panel.appendChild(ulFs);

    // ── 취소선
    const stFs = this.createFieldset('취소선');
    const stRow = this.row();
    stRow.appendChild(this.label('모양(S):'));
    this.strikeShapeSelect = document.createElement('select');
    this.strikeShapeSelect.className = 'dialog-select';
    this.strikeShapeSelect.style.width = '90px';
    for (const [val, lbl] of [
      ['0', '━━━━ 실선'], ['1', '- - - 긴점선'], ['2', '········ 점선'],
      ['3', '━·━· 일점쇄선'], ['4', '━··━ 이점쇄선'],
      ['5', '━━━ 긴파선'], ['6', '●●●● 원형점'],
      ['7', '══ 이중선'], ['8', '━═ 가는+굵은'],
      ['9', '═━ 굵은+가는'], ['10', '≡≡ 삼중선'],
    ] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.strikeShapeSelect.appendChild(o);
    }
    stRow.appendChild(this.strikeShapeSelect);

    const stcLabel = this.label('색(H):');
    stcLabel.style.marginLeft = '10px';
    stRow.appendChild(stcLabel);
    this.strikeColorInput = document.createElement('input');
    this.strikeColorInput.type = 'color';
    this.strikeColorInput.className = 'cs-color-btn';
    stRow.appendChild(this.strikeColorInput);
    stFs.appendChild(stRow);
    panel.appendChild(stFs);

    // ── 기타
    const etcFs = this.createFieldset('기타');
    const etcRow1 = this.row();
    etcRow1.appendChild(this.label('외곽선(O):'));
    this.outlineTypeSelect = document.createElement('select');
    this.outlineTypeSelect.className = 'dialog-select';
    this.outlineTypeSelect.style.width = '80px';
    ['없음', '실선', '점선', '굵은 선', '파선', '일점쇄선', '이점쇄선'].forEach((lbl, i) => {
      const o = document.createElement('option');
      o.value = String(i); o.textContent = lbl;
      this.outlineTypeSelect.appendChild(o);
    });
    etcRow1.appendChild(this.outlineTypeSelect);

    const emLabel = this.label('강조점(E):');
    emLabel.style.marginLeft = '10px';
    etcRow1.appendChild(emLabel);
    this.emphasisSelect = document.createElement('select');
    this.emphasisSelect.className = 'dialog-select';
    this.emphasisSelect.style.width = '80px';
    for (const [val, lbl] of [
      ['0', '없음'], ['1', '● 검정 동그라미'], ['2', '○ 속빈 동그라미'],
      ['3', 'ˇ'], ['4', '˜'], ['5', '･'], ['6', '˸'],
    ] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.emphasisSelect.appendChild(o);
    }
    etcRow1.appendChild(this.emphasisSelect);
    etcFs.appendChild(etcRow1);

    // 체크박스 행
    const etcRow2 = this.row();
    const fitCb = this.checkbox('글꼴에 어울리는 빈칸(F)');
    const kerningCb = this.checkbox('커닝(K)');
    this.kerningCheckbox = kerningCb.querySelector('input')!;
    etcRow2.appendChild(fitCb);
    etcRow2.appendChild(kerningCb);
    etcFs.appendChild(etcRow2);
    panel.appendChild(etcFs);

    // ── 미리보기
    this.extPreviewEl = document.createElement('div');
    this.extPreviewEl.className = 'cs-preview';
    this.extPreviewEl.textContent = '한글Eng123漢字あいう※○';
    panel.appendChild(this.extPreviewEl);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  테두리/배경 탭
  // ════════════════════════════════════════════════════════

  private buildBorderPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 테두리 섹션
    const borderFs = this.createFieldset('테두리');
    const borderContent = document.createElement('div');
    borderContent.className = 'cs-border-layout';

    // 좌측: 컨트롤들
    const borderLeft = document.createElement('div');
    borderLeft.className = 'cs-border-left';

    // 종류(Y) — HWP 스펙 선 종류 값에 매핑
    const typeRow = this.row();
    typeRow.appendChild(this.label('종류(Y):'));
    this.borderTypeSelect = document.createElement('select');
    this.borderTypeSelect.className = 'dialog-select';
    this.borderTypeSelect.style.width = '100px';
    for (const [val, lbl] of [
      ['0', '선 없음'], ['1', '실선'], ['2', '파선'],
      ['3', '점선'], ['4', '일점쇄선'], ['8', '이중선'],
    ] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.borderTypeSelect.appendChild(o);
    }
    this.borderTypeSelect.addEventListener('change', () => this.updateBorderPreview());
    typeRow.appendChild(this.borderTypeSelect);
    borderLeft.appendChild(typeRow);

    // 굵기(I) — HWP 스펙 width 인덱스에 매핑
    const widthRow = this.row();
    widthRow.appendChild(this.label('굵기(I):'));
    this.borderWidthSelect = document.createElement('select');
    this.borderWidthSelect.className = 'dialog-select';
    this.borderWidthSelect.style.width = '100px';
    const widths = ['0.1 mm', '0.12 mm', '0.15 mm', '0.2 mm', '0.25 mm',
                    '0.3 mm', '0.4 mm', '0.5 mm', '0.6 mm', '0.7 mm', '1.0 mm'];
    widths.forEach((w, i) => {
      const o = document.createElement('option');
      o.value = String(i); o.textContent = w;
      this.borderWidthSelect.appendChild(o);
    });
    const widthPreview = document.createElement('span');
    widthPreview.className = 'cs-line-preview';
    widthPreview.textContent = '\u2014\u2014\u2014';
    widthRow.appendChild(this.borderWidthSelect);
    widthRow.appendChild(widthPreview);
    borderLeft.appendChild(widthRow);

    // 색(C)
    const colorRow = this.row();
    colorRow.appendChild(this.label('색(C):'));
    this.borderColorInput = document.createElement('input');
    this.borderColorInput.type = 'color';
    this.borderColorInput.value = '#000000';
    this.borderColorInput.className = 'cs-color-btn';
    this.borderColorInput.style.width = '100px';
    this.borderColorInput.addEventListener('input', () => this.updateBorderPreview());
    colorRow.appendChild(this.borderColorInput);
    borderLeft.appendChild(colorRow);

    borderContent.appendChild(borderLeft);

    // 우측: 테두리 미리보기 + 프리셋 버튼
    const borderRight = document.createElement('div');
    borderRight.className = 'cs-border-right';

    const previewBox = document.createElement('div');
    previewBox.className = 'cs-border-preview';
    this.borderPreviewInner = document.createElement('div');
    this.borderPreviewInner.className = 'cs-border-inner';
    previewBox.appendChild(this.borderPreviewInner);
    borderRight.appendChild(previewBox);

    // 프리셋 버튼 행
    const presetRow = document.createElement('div');
    presetRow.className = 'cs-border-presets';
    // 테두리 없음
    const btnNone = document.createElement('button');
    btnNone.className = 'cs-preset-btn'; btnNone.textContent = '┄'; btnNone.title = '테두리 없음';
    btnNone.addEventListener('click', () => { this.borderTypeSelect.value = '0'; this.updateBorderPreview(); });
    presetRow.appendChild(btnNone);
    // 상자형
    const btnBox = document.createElement('button');
    btnBox.className = 'cs-preset-btn'; btnBox.textContent = '□'; btnBox.title = '상자형';
    btnBox.addEventListener('click', () => { this.borderTypeSelect.value = '1'; this.updateBorderPreview(); });
    presetRow.appendChild(btnBox);
    // 격자형 (글자 테두리에는 같은 효과)
    const btnGrid = document.createElement('button');
    btnGrid.className = 'cs-preset-btn'; btnGrid.textContent = '╬'; btnGrid.title = '격자형';
    btnGrid.addEventListener('click', () => { this.borderTypeSelect.value = '1'; this.updateBorderPreview(); });
    presetRow.appendChild(btnGrid);
    // 사용자 정의
    const btnCustom = document.createElement('button');
    btnCustom.className = 'cs-preset-btn'; btnCustom.textContent = '▣'; btnCustom.title = '사용자 정의';
    presetRow.appendChild(btnCustom);
    borderRight.appendChild(presetRow);

    borderContent.appendChild(borderRight);
    borderFs.appendChild(borderContent);
    panel.appendChild(borderFs);

    // ── 배경 섹션
    const bgFs = this.createFieldset('배경');

    // 면 색(Q) — 색 없음 + 색 선택
    const faceRow = this.row();
    faceRow.appendChild(this.label('면 색(Q):'));
    this.faceColorSelect = document.createElement('select');
    this.faceColorSelect.className = 'dialog-select';
    this.faceColorSelect.style.width = '100px';
    for (const [val, lbl] of [['none', '색 없음'], ['solid', '색 지정']] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.faceColorSelect.appendChild(o);
    }
    faceRow.appendChild(this.faceColorSelect);
    // 색 선택 컬러 피커 (면색 지정 시)
    this.faceColorPicker = document.createElement('input');
    this.faceColorPicker.type = 'color';
    this.faceColorPicker.value = '#ffffff';
    this.faceColorPicker.className = 'cs-color-btn';
    this.faceColorPicker.style.marginLeft = '6px';
    faceRow.appendChild(this.faceColorPicker);
    bgFs.appendChild(faceRow);

    // 무늬 색(P) + 무늬 모양(L)
    const patRow = this.row();
    patRow.appendChild(this.label('무늬 색(P):'));
    this.patColorInput = document.createElement('input');
    this.patColorInput.type = 'color';
    this.patColorInput.value = '#000000';
    this.patColorInput.className = 'cs-color-btn';
    patRow.appendChild(this.patColorInput);

    const patLabel = this.label('무늬 모양(L):');
    patLabel.style.marginLeft = '10px';
    patRow.appendChild(patLabel);
    this.patShapeSelect = document.createElement('select');
    this.patShapeSelect.className = 'dialog-select';
    this.patShapeSelect.style.width = '90px';
    for (const [val, lbl] of [
      ['0', '없음'], ['1', '━'], ['2', '┃'],
      ['3', '╲'], ['4', '╱'], ['5', '╳'], ['6', '┼'],
    ] as const) {
      const o = document.createElement('option');
      o.value = val; o.textContent = lbl;
      this.patShapeSelect.appendChild(o);
    }
    patRow.appendChild(this.patShapeSelect);
    bgFs.appendChild(patRow);
    panel.appendChild(bgFs);

    return panel;
  }

  /** 테두리 미리보기 업데이트 */
  private updateBorderPreview(): void {
    if (!this.borderPreviewInner) return;
    const typeVal = parseInt(this.borderTypeSelect?.value || '0');
    const color = this.borderColorInput?.value || '#000000';
    if (typeVal === 0) {
      this.borderPreviewInner.style.border = '1px dashed #ccc';
    } else {
      const style = typeVal === 8 ? 'double' : typeVal === 3 ? 'dotted' : typeVal === 2 ? 'dashed' : 'solid';
      this.borderPreviewInner.style.border = `2px ${style} ${color}`;
    }
  }

  // ════════════════════════════════════════════════════════
  //  언어별 필드
  // ════════════════════════════════════════════════════════

  private updateLangFields(): void {
    if (!this.props) return;
    const arrIdx = this.currentLang === 0 ? 0 : this.currentLang - 1;

    const families = this.props.fontFamilies || [];
    const fontName = families[arrIdx] || '';
    this.fontSelect.value = fontName;
    if (this.fontSelect.value !== fontName && fontName) {
      const opt = document.createElement('option');
      opt.value = fontName;
      opt.textContent = fontName;
      this.fontSelect.appendChild(opt);
      this.fontSelect.value = fontName;
    }

    const ratios = this.props.ratios || [100, 100, 100, 100, 100, 100, 100];
    const spacings = this.props.spacings || [0, 0, 0, 0, 0, 0, 0];
    const relativeSizes = this.props.relativeSizes || [100, 100, 100, 100, 100, 100, 100];
    const charOffsets = this.props.charOffsets || [0, 0, 0, 0, 0, 0, 0];

    this.langInputs['cs-ratio'].value = String(ratios[arrIdx]);
    this.langInputs['cs-spacing'].value = String(spacings[arrIdx]);
    this.langInputs['cs-relative-size'].value = String(relativeSizes[arrIdx]);
    this.langInputs['cs-char-offset'].value = String(charOffsets[arrIdx]);
  }

  private saveLangFields(): void {
    if (!this.props) return;
    const arrIdx = this.currentLang === 0 ? -1 : this.currentLang - 1;

    const ratio = parseInt(this.langInputs['cs-ratio'].value) || 100;
    const spacing = parseInt(this.langInputs['cs-spacing'].value) || 0;
    const relSize = parseInt(this.langInputs['cs-relative-size'].value) || 100;
    const charOff = parseInt(this.langInputs['cs-char-offset'].value) || 0;

    if (arrIdx === -1) {
      this.props.ratios = Array(7).fill(ratio) as number[];
      this.props.spacings = Array(7).fill(spacing) as number[];
      this.props.relativeSizes = Array(7).fill(relSize) as number[];
      this.props.charOffsets = Array(7).fill(charOff) as number[];
    } else {
      if (!this.props.ratios) this.props.ratios = [100, 100, 100, 100, 100, 100, 100];
      if (!this.props.spacings) this.props.spacings = [0, 0, 0, 0, 0, 0, 0];
      if (!this.props.relativeSizes) this.props.relativeSizes = [100, 100, 100, 100, 100, 100, 100];
      if (!this.props.charOffsets) this.props.charOffsets = [0, 0, 0, 0, 0, 0, 0];
      this.props.ratios[arrIdx] = ratio;
      this.props.spacings[arrIdx] = spacing;
      this.props.relativeSizes[arrIdx] = relSize;
      this.props.charOffsets[arrIdx] = charOff;
    }
  }

  // ════════════════════════════════════════════════════════
  //  속성 채우기 + 미리보기
  // ════════════════════════════════════════════════════════

  private populateFromProps(): void {
    const p = this.props;
    if (!p) return;

    this.baseSizeInput.value = ((p.fontSize || 1000) / 100).toFixed(1);
    this.updateLangFields();

    this.setAttrBtn('bold', p.bold);
    this.setAttrBtn('italic', p.italic);
    this.setAttrBtn('underline', p.underline);
    this.setAttrBtn('strikethrough', p.strikethrough);
    this.setAttrBtn('superscript', p.superscript);
    this.setAttrBtn('subscript', p.subscript);

    this.textColorInput.value = p.textColor || '#000000';
    this.shadeColorInput.value = p.shadeColor || '#ffffff';

    // 확장 탭
    const shadowIdx = p.shadowType || 0;
    this.shadowRadios.forEach((r, i) => { r.checked = (i === shadowIdx); });
    this.shadowColorInput.value = p.shadowColor || '#b2b2b2';
    this.shadowXInput.value = String(p.shadowOffsetX || 10);
    this.shadowYInput.value = String(p.shadowOffsetY || 10);
    this.ulPosSelect.value = p.underlineType || 'None';
    this.ulShapeSelect.value = String(p.underlineShape ?? 0);
    this.ulColorInput.value = p.underlineColor || '#000000';
    this.strikeShapeSelect.value = String(p.strikeShape ?? 0);
    this.strikeColorInput.value = p.strikeColor || '#000000';
    this.outlineTypeSelect.value = String(p.outlineType || 0);
    this.emphasisSelect.value = String(p.emphasisDot ?? 0);
    this.kerningCheckbox.checked = p.kerning ?? false;

    // 테두리/배경 탭
    const bl = p.borderLeft;
    if (bl) {
      this.borderTypeSelect.value = String(bl.type);
      // borderTypeSelect에 해당 값이 없으면 기본값(0) 사용
      if (!this.borderTypeSelect.value) this.borderTypeSelect.value = '0';
      this.borderWidthSelect.value = String(bl.width);
      this.borderColorInput.value = bl.color || '#000000';
    } else {
      this.borderTypeSelect.value = '0';
      this.borderWidthSelect.value = '0';
      this.borderColorInput.value = '#000000';
    }
    this.updateBorderPreview();

    // 배경
    const ft = p.fillType || 'none';
    this.faceColorSelect.value = ft === 'solid' ? 'solid' : 'none';
    this.faceColorPicker.value = p.fillColor || '#ffffff';
    this.patColorInput.value = p.patternColor || '#000000';
    this.patShapeSelect.value = String(p.patternType || 0);

    this.updatePreview();
  }

  private setAttrBtn(id: string, active?: boolean): void {
    const btn = this.attrBtns[id];
    if (btn) btn.classList.toggle('active', !!active);
  }

  private updatePreview(): void {
    if (!this.previewEl) return;
    const styles: string[] = [];
    const size = parseFloat(this.baseSizeInput?.value || '10') || 10;
    styles.push(`font-size:${Math.min(size, 24)}pt`);
    if (this.fontSelect?.value) styles.push(`font-family:'${this.fontSelect.value}'`);
    if (this.attrBtns['bold']?.classList.contains('active')) styles.push('font-weight:bold');
    if (this.attrBtns['italic']?.classList.contains('active')) styles.push('font-style:italic');
    const decs: string[] = [];
    if (this.attrBtns['underline']?.classList.contains('active')) decs.push('underline');
    if (this.attrBtns['strikethrough']?.classList.contains('active')) decs.push('line-through');
    if (decs.length) styles.push(`text-decoration:${decs.join(' ')}`);
    if (this.textColorInput?.value) styles.push(`color:${this.textColorInput.value}`);
    this.previewEl.style.cssText = styles.join(';');
  }

  // ════════════════════════════════════════════════════════
  //  변경사항 수집
  // ════════════════════════════════════════════════════════

  private collectMods(): Partial<CharProperties> {
    const mods: Partial<CharProperties> = {};
    const p = this.initialProps;
    if (!p) return mods;

    // 기준 크기
    const newSize = Math.round(parseFloat(this.baseSizeInput.value) * 100);
    if (newSize !== p.fontSize && newSize >= 100 && newSize <= 409600) {
      mods.fontSize = newSize;
    }

    // 속성 토글
    for (const f of ['bold', 'italic', 'superscript', 'subscript'] as const) {
      const active = this.attrBtns[f]?.classList.contains('active') || false;
      if (active !== (p[f] || false)) (mods as Record<string, unknown>)[f] = active;
    }
    const ulActive = this.attrBtns['underline']?.classList.contains('active') || false;
    if (ulActive !== (p.underline || false)) mods.underline = ulActive;
    const stActive = this.attrBtns['strikethrough']?.classList.contains('active') || false;
    if (stActive !== (p.strikethrough || false)) mods.strikethrough = stActive;

    // outline/shadow 토글 (기본 탭 아이콘)
    const outlineActive = this.attrBtns['outline']?.classList.contains('active') || false;
    if (outlineActive !== ((p.outlineType || 0) > 0)) {
      mods.outlineType = outlineActive ? 1 : 0;
    }
    const shadowActive = this.attrBtns['shadow']?.classList.contains('active') || false;
    if (shadowActive !== ((p.shadowType || 0) > 0)) {
      mods.shadowType = shadowActive ? 1 : 0;
    }

    // 색상
    if (this.textColorInput.value !== (p.textColor || '#000000')) mods.textColor = this.textColorInput.value;
    if (this.shadeColorInput.value !== (p.shadeColor || '#ffffff')) mods.shadeColor = this.shadeColorInput.value;

    // 확장 탭 — 그림자
    const shadowType = this.shadowRadios.findIndex(r => r.checked);
    if (shadowType >= 0 && shadowType !== (p.shadowType || 0)) mods.shadowType = shadowType;
    if (this.shadowColorInput.value !== (p.shadowColor || '#b2b2b2')) mods.shadowColor = this.shadowColorInput.value;
    const sx = parseInt(this.shadowXInput.value) || 0;
    if (sx !== (p.shadowOffsetX || 0)) mods.shadowOffsetX = sx;
    const sy = parseInt(this.shadowYInput.value) || 0;
    if (sy !== (p.shadowOffsetY || 0)) mods.shadowOffsetY = sy;

    // 확장 탭 — 밑줄
    const ulPos = this.ulPosSelect.value;
    if (ulPos !== (p.underlineType || 'None')) mods.underlineType = ulPos;
    if (this.ulColorInput.value !== (p.underlineColor || '#000000')) mods.underlineColor = this.ulColorInput.value;

    // 확장 탭 — 밑줄 모양
    const ulShape = parseInt(this.ulShapeSelect.value);
    if (ulShape !== (p.underlineShape ?? 0)) mods.underlineShape = ulShape;
    // 밑줄 위치/모양/색이 변경되면 underline 자동 활성화
    if ((mods.underlineType !== undefined || mods.underlineShape !== undefined || mods.underlineColor !== undefined) && !p.underline) {
      mods.underline = true;
    }

    // 확장 탭 — 취소선
    const strikeShape = parseInt(this.strikeShapeSelect.value);
    if (strikeShape !== (p.strikeShape ?? 0)) mods.strikeShape = strikeShape;
    if (this.strikeColorInput.value !== (p.strikeColor || '#000000')) mods.strikeColor = this.strikeColorInput.value;
    // 취소선 모양/색이 변경되면 strikethrough 자동 활성화
    if ((mods.strikeShape !== undefined || mods.strikeColor !== undefined) && !p.strikethrough) {
      mods.strikethrough = true;
    }

    // 확장 탭 — 외곽선
    const outlineType = parseInt(this.outlineTypeSelect.value);
    if (outlineType !== (p.outlineType || 0)) mods.outlineType = outlineType;

    // 확장 탭 — 강조점
    const emphDot = parseInt(this.emphasisSelect.value);
    if (emphDot !== (p.emphasisDot ?? 0)) mods.emphasisDot = emphDot;

    // 확장 탭 — 커닝
    const kerning = this.kerningCheckbox.checked;
    if (kerning !== (p.kerning ?? false)) mods.kerning = kerning;

    // 언어별 배열
    this.saveLangFields();
    for (const prop of ['ratios', 'spacings', 'relativeSizes', 'charOffsets'] as const) {
      const orig = p[prop] || [];
      const curr = this.props?.[prop] || [];
      if (JSON.stringify(orig) !== JSON.stringify(curr)) {
        (mods as Record<string, unknown>)[prop] = curr;
      }
    }

    // 글꼴 변경
    const arrIdx = this.currentLang === 0 ? 0 : this.currentLang - 1;
    const origFont = (p.fontFamilies || [])[arrIdx] || '';
    if (this.fontSelect.value && this.fontSelect.value !== origFont) {
      mods.fontName = this.fontSelect.value;
    }

    // 테두리/배경 탭
    const bType = parseInt(this.borderTypeSelect.value);
    const bWidth = parseInt(this.borderWidthSelect.value);
    const bColor = this.borderColorInput.value;
    const origBl = p.borderLeft || { type: 0, width: 0, color: '#000000' };
    if (bType !== origBl.type || bWidth !== origBl.width || bColor !== origBl.color) {
      const border = { type: bType, width: bWidth, color: bColor };
      mods.borderLeft = border;
      mods.borderRight = border;
      mods.borderTop = border;
      mods.borderBottom = border;
    }

    // 배경
    const fillType = this.faceColorSelect.value === 'solid' ? 'solid' : 'none';
    const fillColor = this.faceColorPicker.value;
    const patColor = this.patColorInput.value;
    const patType = parseInt(this.patShapeSelect.value);
    const origFillType = p.fillType || 'none';
    const origFillColor = p.fillColor || '#ffffff';
    const origPatColor = p.patternColor || '#000000';
    const origPatType = p.patternType || 0;
    if (fillType !== origFillType) mods.fillType = fillType;
    if (fillColor !== origFillColor) mods.fillColor = fillColor;
    if (patColor !== origPatColor) mods.patternColor = patColor;
    if (patType !== origPatType) mods.patternType = patType;

    return mods;
  }

  // ════════════════════════════════════════════════════════
  //  설정/취소
  // ════════════════════════════════════════════════════════

  private handleOk(): void {
    this.saveLangFields();
    const mods = this.collectMods();
    if (Object.keys(mods).length === 0) {
      this.hide();
      return;
    }
    if (this.onApply) this.onApply(mods);
    this.hide();
  }

  // ════════════════════════════════════════════════════════
  //  DOM 헬퍼
  // ════════════════════════════════════════════════════════

  private createFieldset(title: string): HTMLFieldSetElement {
    const fs = document.createElement('fieldset');
    fs.className = 'cs-fieldset';
    const legend = document.createElement('legend');
    legend.textContent = title;
    fs.appendChild(legend);
    return fs;
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

  private numberInput(min?: number, max?: number, step?: number): HTMLInputElement {
    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'dialog-input';
    if (min !== undefined) inp.min = String(min);
    if (max !== undefined) inp.max = String(max);
    if (step !== undefined) inp.step = String(step);
    return inp;
  }

  private unit(text: string): HTMLSpanElement {
    const u = document.createElement('span');
    u.className = 'dialog-unit';
    u.textContent = text;
    return u;
  }

  private checkbox(text: string): HTMLLabelElement {
    const lb = document.createElement('label');
    lb.className = 'cs-checkbox-label';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    lb.appendChild(cb);
    lb.appendChild(document.createTextNode(` ${text}`));
    return lb;
  }
}
