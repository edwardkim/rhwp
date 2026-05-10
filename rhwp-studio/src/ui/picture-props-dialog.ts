/**
 * 개체 속성 대화상자 — 한컴 개체 속성 대화상자 재현
 *
 * 탭 구성 (글상자):
 *  기본 | 여백/캡션 | 선 | 채우기 | 글상자 | 그림자
 *
 * 탭 구성 (그림):
 *  기본 | 여백/캡션 | 선 | 그림 | 그림자
 *
 * 레이아웃: 좌측(탭+컨텐츠) + 우측(버튼) 패턴
 * CSS 접두어: pp-
 */
import type { PictureProperties, ShapeProperties } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';

/** HWPUNIT ↔ mm 변환 상수 (1 inch = 25.4 mm = 7200 HWPUNIT) */
const HWP_PER_MM = 7200 / 25.4; // ≈ 283.46

function hwpToMm(hwp: number): number {
  return hwp / HWP_PER_MM;
}
function mmToHwp(mm: number): number {
  return Math.round(mm * HWP_PER_MM);
}

/** HWP ColorRef (BGR u32) → HTML hex (#rrggbb) */
function colorRefToHex(c: number): string {
  const b = (c >> 16) & 0xFF;
  const g = (c >> 8) & 0xFF;
  const r = c & 0xFF;
  return '#' + [r, g, b].map(v => v.toString(16).padStart(2, '0')).join('');
}

/** HTML hex (#rrggbb) → HWP ColorRef (BGR u32) */
function hexToColorRef(hex: string): number {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return (b << 16) | (g << 8) | r;
}

/** 탭 이름 — 그림용 */
const PICTURE_TAB_NAMES = ['기본', '여백/캡션', '선', '그림', '그림자', '반사', '네온', '열은 테두리'];
/** 탭 이름 — 글상자용 */
const SHAPE_TAB_NAMES = ['기본', '여백/캡션', '선', '채우기', '글상자', '그림자'];
/** 탭 이름 — 직선용 (채우기/글상자 불필요) */
const LINE_TAB_NAMES = ['기본', '여백/캡션', '선', '그림자'];

export class PicturePropsDialog {
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private built = false;

  private wasm: WasmBridge;
  private eventBus: EventBus;

  // 탭
  private tabs: HTMLButtonElement[] = [];
  private panels: HTMLDivElement[] = [];
  private tabGroup!: HTMLDivElement;
  private body!: HTMLDivElement;

  // 현재 편집 대상
  private sec = 0;
  private para = 0;
  private ci = 0;
  private objectType: 'image' | 'shape' | 'line' = 'image';
  private props: PictureProperties | null = null;
  private shapeProps: ShapeProperties | null = null;

  // ── 기본 탭 컨트롤 ──
  // 크기
  private widthInput!: HTMLInputElement;
  private heightInput!: HTMLInputElement;
  private sizeFixedCheck!: HTMLInputElement;
  private originalWidth = 0;
  private originalHeight = 0;

  // 위치
  private treatAsCharCheck!: HTMLInputElement;
  private wrapBtns: HTMLButtonElement[] = [];
  private wrapValues = ['TopAndBottom', 'Square', 'Tight', 'BehindText', 'InFrontOfText'];
  private bodyPosSelect!: HTMLSelectElement;
  private horzRelSelect!: HTMLSelectElement;
  private horzAlignSelect!: HTMLSelectElement;
  private horzOffsetInput!: HTMLInputElement;
  private vertRelSelect!: HTMLSelectElement;
  private vertAlignSelect!: HTMLSelectElement;
  private vertOffsetInput!: HTMLInputElement;
  private posDetailEls: HTMLElement[] = [];
  private pageAreaLimitCheck!: HTMLInputElement;
  private overlapAllowCheck!: HTMLInputElement;
  private samePageCheck!: HTMLInputElement;

  // 개체 회전
  private rotationInput!: HTMLInputElement;
  private horzFlipCheck!: HTMLInputElement;
  private vertFlipCheck!: HTMLInputElement;

  // 기울이기
  private skewHInput!: HTMLInputElement;
  private skewVInput!: HTMLInputElement;

  // 기타
  private protectCheck!: HTMLInputElement;
  private descInput!: HTMLInputElement;

  // ── 여백/캡션 탭 컨트롤 ──
  private outerMarginLeftInput!: HTMLInputElement;
  private outerMarginRightInput!: HTMLInputElement;
  private outerMarginTopInput!: HTMLInputElement;
  private outerMarginBottomInput!: HTMLInputElement;
  private captionBtns: HTMLButtonElement[] = [];
  private captionSizeInput!: HTMLInputElement;
  private captionGapInput!: HTMLInputElement;
  private captionExpandCheck!: HTMLInputElement;
  private captionSingleLineCheck!: HTMLInputElement;

  // ── 선 탭 컨트롤 ──
  private lineColorInput!: HTMLInputElement;
  private lineTypeSelect!: HTMLSelectElement;
  private lineEndSelect!: HTMLSelectElement;
  private lineWidthInput!: HTMLInputElement;
  private arrowStartSelect!: HTMLSelectElement;
  private arrowEndSelect!: HTMLSelectElement;
  private arrowStartSizeSelect!: HTMLSelectElement;
  private arrowEndSizeSelect!: HTMLSelectElement;
  private cornerBtns: HTMLButtonElement[] = [];
  private cornerCustomRadio!: HTMLInputElement;
  private cornerCustomInput!: HTMLInputElement;
  private arcBtns: HTMLButtonElement[] = [];
  private lineTransInput!: HTMLInputElement;
  private lineInnerCheck!: HTMLInputElement;

  // ── 채우기 탭 컨트롤 ──
  private fillNoneRadio!: HTMLInputElement;
  private fillSolidRadio!: HTMLInputElement;
  private fillGradientRadio!: HTMLInputElement;
  private fillImageCheck!: HTMLInputElement;
  private solidFaceColor!: HTMLInputElement;
  private solidPatColor!: HTMLInputElement;
  private solidPatternSelect!: HTMLSelectElement;
  private gradStartColor!: HTMLInputElement;
  private gradEndColor!: HTMLInputElement;
  private gradTypeSelect!: HTMLSelectElement;
  private gradDirBtns: HTMLButtonElement[] = [];
  private gradCenterXInput!: HTMLInputElement;
  private gradCenterYInput!: HTMLInputElement;
  private gradTiltInput!: HTMLInputElement;
  private gradBlurInput!: HTMLInputElement;
  private gradReverseCenterInput!: HTMLInputElement;
  private imageFileInput!: HTMLInputElement;
  private imageEmbedCheck!: HTMLInputElement;
  private imageFillTypeSelect!: HTMLSelectElement;
  private imageBrightnessInput!: HTMLInputElement;
  private imageEffectSelect!: HTMLSelectElement;
  private imageContrastInput!: HTMLInputElement;
  private imageWatermarkCheck!: HTMLInputElement;
  private fillTransInput!: HTMLInputElement;
  // 채우기 영역 참조 (라디오 전환용)
  private solidArea!: HTMLDivElement;
  private gradientArea!: HTMLDivElement;
  private imageArea!: HTMLDivElement;

  // ── 글상자 탭 컨트롤 ──
  private tbMarginLeftInput!: HTMLInputElement;
  private tbMarginRightInput!: HTMLInputElement;
  private tbMarginTopInput!: HTMLInputElement;
  private tbMarginBottomInput!: HTMLInputElement;
  private tbVertAlignBtns: HTMLButtonElement[] = [];
  private tbVertWriteCheck!: HTMLInputElement;
  private tbEngLay!: HTMLButtonElement;
  private tbEngStand!: HTMLButtonElement;
  private tbSingleLineCheck!: HTMLInputElement;
  private tbFieldNameInput!: HTMLInputElement;
  private tbFormModeCheck!: HTMLInputElement;

  // ── 그림 탭 컨트롤 ──
  // [Task #741 후속] 외부 file path 그림 영역 영역 dialog 표시 영역
  private picFileNameInput!: HTMLInputElement;
  private picEmbedCheck!: HTMLInputElement;
  private picScaleXInput!: HTMLInputElement;
  private picScaleYInput!: HTMLInputElement;
  private picKeepRatioCheck!: HTMLInputElement;
  private picCropLeftInput!: HTMLInputElement;
  private picCropTopInput!: HTMLInputElement;
  private picCropRightInput!: HTMLInputElement;
  private picCropBottomInput!: HTMLInputElement;
  private picPadLeftInput!: HTMLInputElement;
  private picPadTopInput!: HTMLInputElement;
  private picPadRightInput!: HTMLInputElement;
  private picPadBottomInput!: HTMLInputElement;
  private picEffectRadios: HTMLInputElement[] = [];
  private picBrightnessInput!: HTMLInputElement;
  private picContrastInput!: HTMLInputElement;
  private picWatermarkCheck!: HTMLInputElement;

  // ── 그림자 탭 컨트롤 ──
  private shadowTypeBtns: HTMLButtonElement[] = [];
  private shadowColorInput!: HTMLInputElement;
  private shadowHInput!: HTMLInputElement;
  private shadowVInput!: HTMLInputElement;
  private shadowDirBtns: HTMLButtonElement[] = [];
  private shadowTransInput!: HTMLInputElement;

  constructor(wasm: WasmBridge, eventBus: EventBus) {
    this.wasm = wasm;
    this.eventBus = eventBus;
  }

  // ════════════════════════════════════════════════════════
  //  공개 API
  // ════════════════════════════════════════════════════════

  open(sec: number, para: number, ci: number, type: 'image' | 'shape' | 'line' = 'image'): void {
    this.build();
    this.sec = sec;
    this.para = para;
    this.ci = ci;
    this.objectType = type;

    if (type === 'shape' || type === 'line') {
      this.shapeProps = this.wasm.getShapeProperties(sec, para, ci);
      this.props = this.shapeProps as unknown as PictureProperties;
    } else {
      this.shapeProps = null;
      this.props = this.wasm.getPictureProperties(sec, para, ci);
    }
    this.originalWidth = this.props.width;
    this.originalHeight = this.props.height;
    this.rebuildTabs();
    this.populateFromProps();
    this.switchTab(0);
    document.body.appendChild(this.overlay);
    setTimeout(() => this.widthInput?.select(), 50);
  }

  hide(): void {
    this.overlay?.remove();
  }

  // ════════════════════════════════════════════════════════
  //  빌드
  // ════════════════════════════════════════════════════════

  private build(): void {
    if (this.built) return;
    this.built = true;

    // 오버레이
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    // 다이얼로그 컨테이너
    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap pp-dialog';

    // 타이틀 바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '개체 속성';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);

    // 메인 레이아웃: 좌측(탭+컨텐츠) + 우측(버튼)
    const mainRow = document.createElement('div');
    mainRow.className = 'cs-main-row';

    // 좌측 영역
    const leftCol = document.createElement('div');
    leftCol.className = 'cs-left-col';

    // 탭 그룹
    this.tabGroup = document.createElement('div');
    this.tabGroup.className = 'dialog-tabs';
    leftCol.appendChild(this.tabGroup);

    // 탭 패널 컨테이너
    this.body = document.createElement('div');
    this.body.className = 'dialog-body';
    leftCol.appendChild(this.body);

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

    // Escape / 오버레이 클릭
    this.overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') { e.stopPropagation(); this.hide(); }
    });
    this.overlay.addEventListener('mousedown', (e) => {
      if (e.target === this.overlay) this.hide();
    });

    this.enableDrag(titleBar);
  }

  private switchTab(idx: number): void {
    this.tabs.forEach((t, i) => t.classList.toggle('active', i === idx));
    this.panels.forEach((p, i) => p.classList.toggle('active', i === idx));
  }

  /** 개체 타입에 따라 탭을 재구성한다 */
  private rebuildTabs(): void {
    this.tabGroup.replaceChildren();
    this.body.replaceChildren();
    this.tabs = [];
    this.panels = [];

    const tabNames = this.objectType === 'line' ? LINE_TAB_NAMES
      : this.objectType === 'shape' ? SHAPE_TAB_NAMES
      : PICTURE_TAB_NAMES;
    tabNames.forEach((name, i) => {
      const btn = document.createElement('button');
      btn.className = 'dialog-tab';
      btn.textContent = name;
      btn.addEventListener('click', () => this.switchTab(i));
      this.tabGroup.appendChild(btn);
      this.tabs.push(btn);
    });

    // 패널 생성
    const builders: Record<string, () => HTMLDivElement> = {
      '기본': () => this.buildBasicPanel(),
      '여백/캡션': () => this.buildMarginCaptionPanel(),
      '선': () => this.buildLinePanel(),
      '채우기': () => this.buildFillPanel(),
      '글상자': () => this.buildTextboxPanel(),
      '그림': () => this.buildPicturePanel(),
      '그림자': () => this.buildShadowPanel(),
      '반사': () => this.buildReflectionPanel(),
      '네온': () => this.buildGlowPanel(),
      '열은 테두리': () => this.buildSoftEdgePanel(),
    };
    tabNames.forEach((name) => {
      const builder = builders[name];
      const panel = builder ? builder() : this.buildStubPanel(name);
      this.panels.push(panel);
      this.body.appendChild(panel);
    });
  }

  // ════════════════════════════════════════════════════════
  //  기본 탭
  // ════════════════════════════════════════════════════════

  private buildBasicPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 크기 ──
    const sizeFs = this.fieldset('크기');
    panel.appendChild(sizeFs);

    // 너비
    const wRow = this.row();
    wRow.appendChild(this.label('너비(W)'));
    wRow.appendChild(this.sizeTypeSelect());
    this.widthInput = this.numberInput(0);
    wRow.appendChild(this.widthInput);
    wRow.appendChild(this.unit('mm'));
    sizeFs.appendChild(wRow);

    // 높이
    const hRow = this.row();
    hRow.appendChild(this.label('높이(H)'));
    hRow.appendChild(this.sizeTypeSelect());
    this.heightInput = this.numberInput(0);
    hRow.appendChild(this.heightInput);
    hRow.appendChild(this.unit('mm'));
    // 크기 고정
    const sfLabel = this.checkboxLabel('크기 고정(S)');
    this.sizeFixedCheck = sfLabel.querySelector('input') as HTMLInputElement;
    hRow.appendChild(sfLabel);
    sizeFs.appendChild(hRow);

    // 비율 유지 이벤트
    this.widthInput.addEventListener('input', () => {
      if (this.originalWidth > 0) {
        const ratio = this.originalHeight / this.originalWidth;
        const w = parseFloat(this.widthInput.value) || 0;
        this.heightInput.value = (w * ratio).toFixed(2);
      }
    });
    this.heightInput.addEventListener('input', () => {
      if (this.originalHeight > 0) {
        const ratio = this.originalWidth / this.originalHeight;
        const h = parseFloat(this.heightInput.value) || 0;
        this.widthInput.value = (h * ratio).toFixed(2);
      }
    });

    // ── 위치 ──
    const posFs = this.fieldset('위치');
    panel.appendChild(posFs);

    // 글자처럼 취급
    const tacRow = this.row();
    const tacLabel = this.checkboxLabel('글자처럼 취급(C)');
    this.treatAsCharCheck = tacLabel.querySelector('input') as HTMLInputElement;
    tacRow.appendChild(tacLabel);
    posFs.appendChild(tacRow);
    this.treatAsCharCheck.addEventListener('change', () => this.updatePositionVisibility());

    // 본문과의 배치 (아이콘 버튼 5개) + 본문 위치 드롭다운
    const wrapRow = this.row();
    wrapRow.classList.add('pp-pos-detail');
    wrapRow.appendChild(this.label('본문과의 배치:'));
    const wrapIcons = ['⬒', '⬓', '⬔', '⬕', '⬖'];
    const wrapTitles = ['자리 차지', '어울림', '빈 공간 채움', '글 뒤로', '글 앞으로'];
    this.wrapBtns = [];
    wrapTitles.forEach((title, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn';
      btn.textContent = wrapIcons[i];
      btn.title = title;
      btn.addEventListener('click', () => this.selectWrap(i));
      wrapRow.appendChild(btn);
      this.wrapBtns.push(btn);
    });
    // 본문 위치(P)
    wrapRow.appendChild(this.label('본문 위치(P):'));
    this.bodyPosSelect = this.selectEl([
      ['Both', '양쪽'], ['Left', '왼쪽'], ['Right', '오른쪽'],
      ['Larger', '큰 쪽'], ['Smaller', '작은 쪽'],
    ]);
    this.bodyPosSelect.disabled = true;
    wrapRow.appendChild(this.bodyPosSelect);
    posFs.appendChild(wrapRow);
    this.posDetailEls.push(wrapRow);

    // 가로
    const hPosRow = this.row();
    hPosRow.classList.add('pp-pos-detail');
    hPosRow.appendChild(this.label('가로(I):'));
    this.horzRelSelect = this.selectEl([
      ['Paper', '종이'], ['Page', '쪽'], ['Column', '단'],
    ]);
    hPosRow.appendChild(this.horzRelSelect);
    hPosRow.appendChild(this.unit('의'));
    this.horzAlignSelect = this.selectEl([
      ['Left', '왼쪽'], ['Center', '가운데'], ['Right', '오른쪽'], ['Outside', '바깥쪽'],
    ]);
    hPosRow.appendChild(this.horzAlignSelect);
    hPosRow.appendChild(this.unit('기준'));
    this.horzOffsetInput = this.numberInput();
    hPosRow.appendChild(this.horzOffsetInput);
    hPosRow.appendChild(this.unit('mm'));
    posFs.appendChild(hPosRow);
    this.posDetailEls.push(hPosRow);

    // 세로
    const vPosRow = this.row();
    vPosRow.classList.add('pp-pos-detail');
    vPosRow.appendChild(this.label('세로(V):'));
    this.vertRelSelect = this.selectEl([
      ['Paper', '종이'], ['Page', '쪽'], ['Paragraph', '문단'],
    ]);
    vPosRow.appendChild(this.vertRelSelect);
    vPosRow.appendChild(this.unit('의'));
    this.vertAlignSelect = this.selectEl([
      ['Top', '위'], ['Center', '가운데'], ['Bottom', '아래'],
    ]);
    vPosRow.appendChild(this.vertAlignSelect);
    vPosRow.appendChild(this.unit('기준'));
    this.vertOffsetInput = this.numberInput();
    vPosRow.appendChild(this.vertOffsetInput);
    vPosRow.appendChild(this.unit('mm'));
    posFs.appendChild(vPosRow);
    this.posDetailEls.push(vPosRow);

    // 쪽 영역 안으로 제한 / 서로 겹침 허용
    const optRow = this.row();
    optRow.classList.add('pp-pos-detail');
    const palLabel = this.checkboxLabel('쪽 영역 안으로 제한(B)');
    this.pageAreaLimitCheck = palLabel.querySelector('input') as HTMLInputElement;
    this.pageAreaLimitCheck.disabled = true;
    optRow.appendChild(palLabel);
    const oaLabel = this.checkboxLabel('서로 겹침 허용(L)');
    this.overlapAllowCheck = oaLabel.querySelector('input') as HTMLInputElement;
    this.overlapAllowCheck.checked = true;
    this.overlapAllowCheck.disabled = true;
    optRow.appendChild(oaLabel);
    posFs.appendChild(optRow);
    this.posDetailEls.push(optRow);

    // 개체와 조판 부호를 항상 같은 쪽에 놓기
    const spRow = this.row();
    spRow.classList.add('pp-pos-detail');
    const spLabel = this.checkboxLabel('개체와 조판 부호를 항상 같은 쪽에 놓기(A)');
    this.samePageCheck = spLabel.querySelector('input') as HTMLInputElement;
    this.samePageCheck.disabled = true;
    spRow.appendChild(spLabel);
    posFs.appendChild(spRow);
    this.posDetailEls.push(spRow);

    // ── 개체 회전 ──
    const rotFs = this.fieldset('개체 회전/대칭');
    panel.appendChild(rotFs);
    const rotRow = this.row();
    rotRow.appendChild(this.label('회전각(E):'));
    this.rotationInput = this.numberInput(-360, 360, 1);
    this.rotationInput.disabled = true;
    rotRow.appendChild(this.rotationInput);
    rotRow.appendChild(this.unit('°'));
    // 회전 프리뷰 원
    const rotPreview = document.createElement('div');
    rotPreview.className = 'pp-rot-preview';
    const rotLine = document.createElement('div');
    rotLine.className = 'pp-rot-line';
    rotPreview.appendChild(rotLine);
    rotRow.appendChild(rotPreview);
    rotFs.appendChild(rotRow);
    // 대칭 체크박스
    const flipRow = this.row();
    this.horzFlipCheck = document.createElement('input');
    this.horzFlipCheck.type = 'checkbox';
    this.horzFlipCheck.disabled = true;
    const horzLabel = this.label('좌우 대칭');
    horzLabel.style.cursor = 'pointer';
    horzLabel.prepend(this.horzFlipCheck);
    flipRow.appendChild(horzLabel);
    this.vertFlipCheck = document.createElement('input');
    this.vertFlipCheck.type = 'checkbox';
    this.vertFlipCheck.disabled = true;
    const vertLabel = this.label('상하 대칭');
    vertLabel.style.cursor = 'pointer';
    vertLabel.style.marginLeft = '12px';
    vertLabel.prepend(this.vertFlipCheck);
    flipRow.appendChild(vertLabel);
    rotFs.appendChild(flipRow);

    // ── 기울이기 ──
    const skewFs = this.fieldset('기울이기');
    panel.appendChild(skewFs);
    const skewRow = this.row();
    skewRow.appendChild(this.label('가로(Y):'));
    this.skewHInput = this.numberInput(0, 45, 1);
    this.skewHInput.disabled = true;
    skewRow.appendChild(this.skewHInput);
    skewRow.appendChild(this.unit('°'));
    skewRow.appendChild(this.label('세로(U):'));
    this.skewVInput = this.numberInput(0, 45, 1);
    this.skewVInput.disabled = true;
    skewRow.appendChild(this.skewVInput);
    skewRow.appendChild(this.unit('°'));
    skewFs.appendChild(skewRow);

    // ── 기타 ──
    const etcFs = this.fieldset('기타');
    panel.appendChild(etcFs);
    const etcRow = this.row();
    etcRow.appendChild(this.label('번호 종류(N):'));
    const numTypeSelect = this.selectEl([['Picture', '그림']]);
    numTypeSelect.disabled = true;
    etcRow.appendChild(numTypeSelect);
    // 개체 보호하기
    const protLabel = this.checkboxLabel('개체 보호하기(K)');
    this.protectCheck = protLabel.querySelector('input') as HTMLInputElement;
    this.protectCheck.disabled = true;
    etcRow.appendChild(protLabel);
    const descBtn = document.createElement('button');
    descBtn.className = 'dialog-btn pp-desc-btn';
    descBtn.textContent = '개체 설명문(X)...';
    descBtn.addEventListener('click', () => this.showDescriptionPrompt());
    etcRow.appendChild(descBtn);
    etcFs.appendChild(etcRow);

    // 개체 설명 값 (숨김)
    this.descInput = document.createElement('input');
    this.descInput.type = 'hidden';
    panel.appendChild(this.descInput);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  여백/캡션 탭
  // ════════════════════════════════════════════════════════

  private buildMarginCaptionPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 바깥 여백 ──
    const marginFs = this.fieldset('바깥 여백');
    panel.appendChild(marginFs);

    const row1 = this.row();
    row1.appendChild(this.label('왼쪽(L):'));
    this.outerMarginLeftInput = this.numberInput(0);
    this.outerMarginLeftInput.value = '0.00';
    row1.appendChild(this.outerMarginLeftInput);
    row1.appendChild(this.unit('mm'));
    row1.appendChild(this.label('위쪽(T):'));
    this.outerMarginTopInput = this.numberInput(0);
    this.outerMarginTopInput.value = '0.00';
    row1.appendChild(this.outerMarginTopInput);
    row1.appendChild(this.unit('mm'));
    // 모두(A) — ▲▼ 화살표만 있는 동기 스피너
    row1.appendChild(this.label('모두'));
    const syncWrap = document.createElement('div');
    syncWrap.className = 'pp-sync-arrows';
    const syncUp = document.createElement('button');
    syncUp.className = 'pp-sync-arrow-btn';
    syncUp.textContent = '▲';
    syncUp.title = '모두 증가';
    syncUp.addEventListener('click', () => {
      [this.outerMarginLeftInput, this.outerMarginRightInput,
       this.outerMarginTopInput, this.outerMarginBottomInput].forEach(inp => {
        inp.value = (parseFloat(inp.value || '0') + 0.5).toFixed(2);
      });
    });
    const syncDown = document.createElement('button');
    syncDown.className = 'pp-sync-arrow-btn';
    syncDown.textContent = '▼';
    syncDown.title = '모두 감소';
    syncDown.addEventListener('click', () => {
      [this.outerMarginLeftInput, this.outerMarginRightInput,
       this.outerMarginTopInput, this.outerMarginBottomInput].forEach(inp => {
        const v = parseFloat(inp.value || '0') - 0.5;
        inp.value = Math.max(0, v).toFixed(2);
      });
    });
    syncWrap.appendChild(syncUp);
    syncWrap.appendChild(syncDown);
    row1.appendChild(syncWrap);
    marginFs.appendChild(row1);

    const row2 = this.row();
    row2.appendChild(this.label('오른쪽(R):'));
    this.outerMarginRightInput = this.numberInput(0);
    this.outerMarginRightInput.value = '0.00';
    row2.appendChild(this.outerMarginRightInput);
    row2.appendChild(this.unit('mm'));
    row2.appendChild(this.label('아래쪽(B):'));
    this.outerMarginBottomInput = this.numberInput(0);
    this.outerMarginBottomInput.value = '0.00';
    row2.appendChild(this.outerMarginBottomInput);
    row2.appendChild(this.unit('mm'));
    marginFs.appendChild(row2);

    // ── 캡션 ──
    const captionFs = this.fieldset('캡션');
    panel.appendChild(captionFs);

    // 가로 배치: 그리드(왼) + 속성(오)
    const capLayout = document.createElement('div');
    capLayout.className = 'pp-caption-layout';

    // 3×3 캡션 위치 그리드
    const grid = document.createElement('div');
    grid.className = 'pp-caption-grid';
    this.captionBtns = [];
    const capTitles = [
      '왼쪽 위', '위', '오른쪽 위',
      '왼쪽', '가운데', '오른쪽',
      '왼쪽 아래', '아래', '오른쪽 아래',
    ];
    const capIcons = [
      '┌가1', '가1─', '가1┐',
      '│가1', '□', '가1│',
      '└가1', '가1─', '가1┘',
    ];
    capTitles.forEach((title, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-caption-btn';
      btn.textContent = capIcons[i];
      btn.title = title;
      btn.disabled = true;
      btn.addEventListener('click', () => {
        this.captionBtns.forEach((b, j) => b.classList.toggle('active', j === i));
      });
      grid.appendChild(btn);
      this.captionBtns.push(btn);
    });
    capLayout.appendChild(grid);

    // 오른쪽 속성 영역
    const capRight = document.createElement('div');
    capRight.className = 'pp-caption-attrs';

    // 크기
    const capRow1 = this.row();
    capRow1.appendChild(this.label('크기(S):'));
    this.captionSizeInput = this.numberInput(0);
    this.captionSizeInput.value = '30.00';
    this.captionSizeInput.disabled = true;
    capRow1.appendChild(this.captionSizeInput);
    capRow1.appendChild(this.unit('mm'));
    capRight.appendChild(capRow1);

    // 개체와의 간격
    const capRow2 = this.row();
    capRow2.appendChild(this.label('개체와의 간격(G):'));
    this.captionGapInput = this.numberInput(0);
    this.captionGapInput.value = '3.00';
    this.captionGapInput.disabled = true;
    capRow2.appendChild(this.captionGapInput);
    capRow2.appendChild(this.unit('mm'));
    capRight.appendChild(capRow2);

    // 체크박스
    const ceLabel = this.checkboxLabel('여백 부분까지 너비 확대(W)');
    this.captionExpandCheck = ceLabel.querySelector('input') as HTMLInputElement;
    this.captionExpandCheck.disabled = true;
    capRight.appendChild(ceLabel);
    const cslLabel = this.checkboxLabel('한 줄로 입력(O)');
    this.captionSingleLineCheck = cslLabel.querySelector('input') as HTMLInputElement;
    this.captionSingleLineCheck.disabled = true;
    capRight.appendChild(cslLabel);

    capLayout.appendChild(capRight);
    captionFs.appendChild(capLayout);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  선 탭
  // ════════════════════════════════════════════════════════

  private buildLinePanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 선 ──
    const lineFs = this.fieldset('선');
    panel.appendChild(lineFs);

    const row1 = this.row();
    row1.appendChild(this.label('색(C):'));
    this.lineColorInput = this.colorInput('#000000');
    row1.appendChild(this.lineColorInput);
    row1.appendChild(this.label('종류(L):'));
    // HWP 선 종류: attr bits 0-5 (0~17)
    this.lineTypeSelect = this.selectEl([
      ['0', '선 없음'], ['1', '실선'], ['2', '파선'], ['3', '점선'],
      ['4', '일점쇄선'], ['5', '이점쇄선'], ['6', '긴 파선'], ['7', '원형 점선'],
      ['8', '2중선'], ['9', '가는선-굵은선'], ['10', '굵은선-가는선'], ['11', '3중선'],
    ]);
    row1.appendChild(this.lineTypeSelect);
    lineFs.appendChild(row1);

    const row2 = this.row();
    row2.appendChild(this.label('끝 모양(E):'));
    // HWP 끝 모양: attr bits 6-9
    this.lineEndSelect = this.selectEl([
      ['0', '둥근'], ['1', '평면'],
    ]);
    row2.appendChild(this.lineEndSelect);
    row2.appendChild(this.label('굵기(T):'));
    this.lineWidthInput = this.numberInput(0, undefined, 0.01);
    this.lineWidthInput.value = '0.12';
    row2.appendChild(this.lineWidthInput);
    row2.appendChild(this.unit('mm'));
    lineFs.appendChild(row2);

    // ── 화살표 ──
    const arrowFs = this.fieldset('화살표');
    panel.appendChild(arrowFs);

    const aRow1 = this.row();
    aRow1.appendChild(this.label('시작 모양(S):'));
    // HWP 화살표 모양: attr bits 10-15 / 16-21
    this.arrowStartSelect = this.selectEl([
      ['0', '없음'], ['1', '화살표'], ['2', '열린 화살표'],
      ['3', '꼬리 화살표'], ['4', '마름모'], ['5', '원형'], ['6', '사각형'],
    ]);
    aRow1.appendChild(this.arrowStartSelect);
    aRow1.appendChild(this.label('끝 모양(Y):'));
    this.arrowEndSelect = this.selectEl([
      ['0', '없음'], ['1', '화살표'], ['2', '열린 화살표'],
      ['3', '꼬리 화살표'], ['4', '마름모'], ['5', '원형'], ['6', '사각형'],
    ]);
    aRow1.appendChild(this.arrowEndSelect);
    arrowFs.appendChild(aRow1);

    const aRow2 = this.row();
    aRow2.appendChild(this.label('시작 크기(Z):'));
    // HWP 화살표 크기: attr bits 22-25 / 26-29 (0~8)
    this.arrowStartSizeSelect = this.selectEl([
      ['0', '작은×작은'], ['1', '작은×중간'], ['2', '작은×큰'],
      ['3', '중간×작은'], ['4', '중간×중간'], ['5', '중간×큰'],
      ['6', '큰×작은'], ['7', '큰×중간'], ['8', '큰×큰'],
    ]);
    aRow2.appendChild(this.arrowStartSizeSelect);
    aRow2.appendChild(this.label('끝 크기(N):'));
    this.arrowEndSizeSelect = this.selectEl([
      ['0', '작은×작은'], ['1', '작은×중간'], ['2', '작은×큰'],
      ['3', '중간×작은'], ['4', '중간×중간'], ['5', '중간×큰'],
      ['6', '큰×작은'], ['7', '큰×중간'], ['8', '큰×큰'],
    ]);
    aRow2.appendChild(this.arrowEndSizeSelect);
    arrowFs.appendChild(aRow2);

    // ── 사각형 모서리 곡률 ──
    const cornerFs = this.fieldset('사각형 모서리 곡률');
    panel.appendChild(cornerFs);

    const cRow = this.row();
    this.cornerBtns = [];
    const cornerIcons = ['▢', '▢̤', '⬭'];
    const cornerTitles = ['직각(G)', '둥근 모양(O)', '반원(M)'];
    cornerTitles.forEach((title, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-corner-btn';
      btn.textContent = cornerIcons[i];
      btn.title = title;
      btn.addEventListener('click', () => {
        this.cornerBtns.forEach((b, j) => b.classList.toggle('active', j === i));
        if (this.cornerCustomRadio) this.cornerCustomRadio.checked = false;
      });
      cRow.appendChild(btn);
      this.cornerBtns.push(btn);
    });
    // 곡률 지정 라디오
    const crLabel = document.createElement('label');
    crLabel.className = 'dialog-checkbox';
    this.cornerCustomRadio = document.createElement('input');
    this.cornerCustomRadio.type = 'radio';
    this.cornerCustomRadio.name = 'corner-mode';
    crLabel.appendChild(this.cornerCustomRadio);
    crLabel.appendChild(document.createTextNode(' 곡률 지정(J):'));
    cRow.appendChild(crLabel);
    this.cornerCustomInput = this.numberInput(0, 100, 1);
    this.cornerCustomInput.value = '0';
    this.cornerCustomInput.disabled = true;
    cRow.appendChild(this.cornerCustomInput);
    cRow.appendChild(this.unit('%'));
    this.cornerCustomRadio.addEventListener('change', () => {
      this.cornerBtns.forEach(b => b.classList.remove('active'));
      this.cornerCustomInput.disabled = !this.cornerCustomRadio.checked;
    });
    cornerFs.appendChild(cRow);

    // ── 호 테두리 ──
    const arcFs = this.fieldset('호 테두리');
    panel.appendChild(arcFs);

    const arcRow = this.row();
    this.arcBtns = [];
    const arcTitles = ['호(A)', '부채꼴(B)', '활 모양(I)'];
    const arcIcons = ['⌒', '◔', '⌢'];
    arcTitles.forEach((title, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn';
      btn.textContent = arcIcons[i];
      btn.title = title;
      btn.disabled = true;
      btn.addEventListener('click', () => {
        this.arcBtns.forEach((b, j) => b.classList.toggle('active', j === i));
      });
      arcRow.appendChild(btn);
      this.arcBtns.push(btn);
    });
    arcFs.appendChild(arcRow);

    // ── 투명도 설정 + 기타 ──
    const transRow = this.row();
    transRow.appendChild(this.label('투명도(I):'));
    this.lineTransInput = this.numberInput(0, 100, 1);
    this.lineTransInput.value = '0';
    this.lineTransInput.disabled = true;
    transRow.appendChild(this.lineTransInput);
    transRow.appendChild(this.unit('%'));

    const liLabel = this.checkboxLabel('선 굵기 내부 적용(K)');
    this.lineInnerCheck = liLabel.querySelector('input') as HTMLInputElement;
    this.lineInnerCheck.disabled = true;
    transRow.appendChild(liLabel);
    panel.appendChild(transRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  채우기 탭
  // ════════════════════════════════════════════════════════

  private buildFillPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 채우기 ──
    const fillFs = this.fieldset('채우기');
    panel.appendChild(fillFs);

    const radioName = 'pp-fill-type';

    // 색 채우기 없음
    const noneRow = this.row();
    const noneLabel = document.createElement('label');
    noneLabel.className = 'dialog-checkbox';
    this.fillNoneRadio = document.createElement('input');
    this.fillNoneRadio.type = 'radio';
    this.fillNoneRadio.name = radioName;
    this.fillNoneRadio.checked = true;
    noneLabel.appendChild(this.fillNoneRadio);
    noneLabel.appendChild(document.createTextNode(' 색 채우기 없음(V)'));
    noneRow.appendChild(noneLabel);
    fillFs.appendChild(noneRow);

    // ◉ 색(O)
    const solidLabel = document.createElement('label');
    solidLabel.className = 'dialog-checkbox';
    this.fillSolidRadio = document.createElement('input');
    this.fillSolidRadio.type = 'radio';
    this.fillSolidRadio.name = radioName;
    solidLabel.appendChild(this.fillSolidRadio);
    solidLabel.appendChild(document.createTextNode(' 색(O)'));

    const solidHdr = this.row();
    solidHdr.appendChild(solidLabel);
    fillFs.appendChild(solidHdr);

    this.solidArea = document.createElement('div');
    this.solidArea.className = 'pp-fill-sub';
    const sRow = this.row();
    sRow.appendChild(this.label('면 색(C):'));
    this.solidFaceColor = this.colorInput('#ffffff');
    sRow.appendChild(this.solidFaceColor);
    sRow.appendChild(this.label('무늬 색(K):'));
    this.solidPatColor = this.colorInput('#000000');
    sRow.appendChild(this.solidPatColor);
    sRow.appendChild(this.label('무늬 모양(L):'));
    this.solidPatternSelect = this.selectEl([
      ['none', '없음'], ['hline', '수평선'], ['vline', '수직선'],
      ['dline1', '대각선1'], ['dline2', '대각선2'], ['cross', '격자'],
    ]);
    sRow.appendChild(this.solidPatternSelect);
    this.solidArea.appendChild(sRow);
    fillFs.appendChild(this.solidArea);

    // ○ 그러데이션(B)
    const gradLabel = document.createElement('label');
    gradLabel.className = 'dialog-checkbox';
    this.fillGradientRadio = document.createElement('input');
    this.fillGradientRadio.type = 'radio';
    this.fillGradientRadio.name = radioName;
    gradLabel.appendChild(this.fillGradientRadio);
    gradLabel.appendChild(document.createTextNode(' 그러데이션(B)'));

    const gradHdr = this.row();
    gradHdr.appendChild(gradLabel);
    fillFs.appendChild(gradHdr);

    this.gradientArea = document.createElement('div');
    this.gradientArea.className = 'pp-fill-sub';

    const gRow1 = this.row();
    gRow1.appendChild(this.label('시작 색(G):'));
    this.gradStartColor = this.colorInput('#ffffff');
    gRow1.appendChild(this.gradStartColor);
    gRow1.appendChild(this.label('끝 색(E):'));
    this.gradEndColor = this.colorInput('#000000');
    gRow1.appendChild(this.gradEndColor);
    this.gradientArea.appendChild(gRow1);

    const gRow2 = this.row();
    gRow2.appendChild(this.label('유형(T):'));
    this.gradTypeSelect = this.selectEl([
      ['linear', '소라'], ['horizontal', '수평'], ['rdiag', '오른쪽 대각선'],
      ['ldiag', '왼쪽 대각선'], ['center', '가운데에서'], ['classic', '클래식'],
      ['narcissus', '나르시스'],
    ]);
    gRow2.appendChild(this.gradTypeSelect);
    // 6방향 아이콘
    const dirGrid = document.createElement('div');
    dirGrid.className = 'pp-gradient-dir';
    this.gradDirBtns = [];
    const dirs = ['↗', '→', '↘', '↙', '←', '↖'];
    dirs.forEach((icon, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-grad-dir-btn';
      btn.textContent = icon;
      btn.addEventListener('click', () => {
        this.gradDirBtns.forEach((b, j) => b.classList.toggle('active', j === i));
      });
      dirGrid.appendChild(btn);
      this.gradDirBtns.push(btn);
    });
    gRow2.appendChild(dirGrid);
    this.gradientArea.appendChild(gRow2);

    const gRow3 = this.row();
    gRow3.appendChild(this.label('가로 중심(W):'));
    this.gradCenterXInput = this.numberInput();
    this.gradCenterXInput.value = '0';
    gRow3.appendChild(this.gradCenterXInput);
    gRow3.appendChild(this.label('세로 중심(X):'));
    this.gradCenterYInput = this.numberInput();
    this.gradCenterYInput.value = '0';
    gRow3.appendChild(this.gradCenterYInput);
    this.gradientArea.appendChild(gRow3);

    const gRow4 = this.row();
    gRow4.appendChild(this.label('기울임(Y):'));
    this.gradTiltInput = this.numberInput();
    this.gradTiltInput.value = '0';
    gRow4.appendChild(this.gradTiltInput);
    gRow4.appendChild(this.label('번짐 정도(Z):'));
    this.gradBlurInput = this.numberInput(0, 100);
    this.gradBlurInput.value = '0';
    gRow4.appendChild(this.gradBlurInput);
    gRow4.appendChild(this.label('반전 중심(N):'));
    this.gradReverseCenterInput = this.numberInput();
    this.gradReverseCenterInput.value = '0';
    gRow4.appendChild(this.gradReverseCenterInput);
    this.gradientArea.appendChild(gRow4);

    fillFs.appendChild(this.gradientArea);

    // ☐ 그림(B)
    const imgHdr = this.row();
    const imgLabel = this.checkboxLabel('그림(B)');
    this.fillImageCheck = imgLabel.querySelector('input') as HTMLInputElement;
    imgHdr.appendChild(imgLabel);
    fillFs.appendChild(imgHdr);

    this.imageArea = document.createElement('div');
    this.imageArea.className = 'pp-fill-sub';

    const iRow1 = this.row();
    iRow1.appendChild(this.label('그림 파일(I):'));
    this.imageFileInput = document.createElement('input');
    this.imageFileInput.type = 'text';
    this.imageFileInput.className = 'dialog-input';
    this.imageFileInput.style.flex = '1';
    this.imageFileInput.disabled = true;
    iRow1.appendChild(this.imageFileInput);
    const browseBtn = document.createElement('button');
    browseBtn.className = 'dialog-btn';
    browseBtn.textContent = '...';
    browseBtn.disabled = true;
    iRow1.appendChild(browseBtn);
    const embedLabel = this.checkboxLabel('문서에 포함(J)');
    this.imageEmbedCheck = embedLabel.querySelector('input') as HTMLInputElement;
    this.imageEmbedCheck.disabled = true;
    iRow1.appendChild(embedLabel);
    this.imageArea.appendChild(iRow1);

    const iRow2 = this.row();
    iRow2.appendChild(this.label('채우기 유형(S):'));
    this.imageFillTypeSelect = this.selectEl([
      ['tile', '바둑판식으로-모두'], ['stretch', '크기에 맞추어'], ['center', '가운데로'],
    ]);
    this.imageFillTypeSelect.disabled = true;
    iRow2.appendChild(this.imageFillTypeSelect);
    iRow2.appendChild(this.label('밝기(H):'));
    this.imageBrightnessInput = this.numberInput(-100, 100);
    this.imageBrightnessInput.value = '0';
    this.imageBrightnessInput.disabled = true;
    iRow2.appendChild(this.imageBrightnessInput);
    iRow2.appendChild(this.unit('%'));
    this.imageArea.appendChild(iRow2);

    const iRow3 = this.row();
    iRow3.appendChild(this.label('그림 효과(E):'));
    this.imageEffectSelect = this.selectEl([
      ['none', '효과 없음'], ['gray', '회색조'], ['bw', '흑백'],
    ]);
    this.imageEffectSelect.disabled = true;
    iRow3.appendChild(this.imageEffectSelect);
    iRow3.appendChild(this.label('대비(I):'));
    this.imageContrastInput = this.numberInput(-100, 100);
    this.imageContrastInput.value = '0';
    this.imageContrastInput.disabled = true;
    iRow3.appendChild(this.imageContrastInput);
    iRow3.appendChild(this.unit('%'));
    this.imageArea.appendChild(iRow3);

    const iRow4 = this.row();
    const wmLabel = this.checkboxLabel('워터마크 효과(M)');
    this.imageWatermarkCheck = wmLabel.querySelector('input') as HTMLInputElement;
    this.imageWatermarkCheck.disabled = true;
    iRow4.appendChild(wmLabel);
    this.imageArea.appendChild(iRow4);

    fillFs.appendChild(this.imageArea);

    // ── 투명도 설정 ──
    const transFs = this.fieldset('투명도 설정');
    panel.appendChild(transFs);
    const transRow = this.row();
    transRow.appendChild(this.label('투명도(I):'));
    this.fillTransInput = this.numberInput(0, 100, 1);
    this.fillTransInput.value = '0';
    this.fillTransInput.disabled = true;
    transRow.appendChild(this.fillTransInput);
    transRow.appendChild(this.unit('%'));
    transFs.appendChild(transRow);

    // 라디오 전환 이벤트
    const updateFillVisibility = () => {
      const isSolid = this.fillSolidRadio.checked;
      const isGrad = this.fillGradientRadio.checked;
      this.solidArea.style.opacity = isSolid ? '1' : '0.4';
      this.gradientArea.style.opacity = isGrad ? '1' : '0.4';
      this.setAreaDisabled(this.solidArea, !isSolid);
      this.setAreaDisabled(this.gradientArea, !isGrad);
      // 투명도: 채우기 없음이면 비활성, 색/그러데이션이면 활성
      this.fillTransInput.disabled = !(isSolid || isGrad);
    };
    this.fillNoneRadio.addEventListener('change', updateFillVisibility);
    this.fillSolidRadio.addEventListener('change', updateFillVisibility);
    this.fillGradientRadio.addEventListener('change', updateFillVisibility);
    // 초기 상태
    setTimeout(updateFillVisibility, 0);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  글상자 탭
  // ════════════════════════════════════════════════════════

  private buildTextboxPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 글상자 여백 ──
    const marginFs = this.fieldset('글상자 여백');
    panel.appendChild(marginFs);

    const lRow = this.row();
    lRow.appendChild(this.label('왼쪽(L):'));
    this.tbMarginLeftInput = this.numberInput(0);
    lRow.appendChild(this.tbMarginLeftInput);
    lRow.appendChild(this.unit('mm'));
    lRow.appendChild(this.label('위쪽(T):'));
    this.tbMarginTopInput = this.numberInput(0);
    lRow.appendChild(this.tbMarginTopInput);
    lRow.appendChild(this.unit('mm'));
    // 모두(A) 동기 스피너
    lRow.appendChild(this.label('모두(A):'));
    const tbSyncAll = this.numberInput(0);
    tbSyncAll.className = 'dialog-input pp-sync-spinner';
    tbSyncAll.addEventListener('input', () => {
      const v = tbSyncAll.value;
      this.tbMarginLeftInput.value = v;
      this.tbMarginRightInput.value = v;
      this.tbMarginTopInput.value = v;
      this.tbMarginBottomInput.value = v;
    });
    lRow.appendChild(tbSyncAll);
    marginFs.appendChild(lRow);

    const rRow = this.row();
    rRow.appendChild(this.label('오른쪽(R):'));
    this.tbMarginRightInput = this.numberInput(0);
    rRow.appendChild(this.tbMarginRightInput);
    rRow.appendChild(this.unit('mm'));
    rRow.appendChild(this.label('아래쪽(B):'));
    this.tbMarginBottomInput = this.numberInput(0);
    rRow.appendChild(this.tbMarginBottomInput);
    rRow.appendChild(this.unit('mm'));
    marginFs.appendChild(rRow);

    // ── 속성 ──
    const attrFs = this.fieldset('속성');
    panel.appendChild(attrFs);

    // 세로 정렬 (아이콘 버튼 3개)
    const vaRow = this.row();
    vaRow.appendChild(this.label('세로 정렬:'));
    this.tbVertAlignBtns = [];
    const vaIcons = ['⬆', '⬌', '⬇'];
    const vaTitles = ['위', '가운데', '아래'];
    const vaValues = ['Top', 'Center', 'Bottom'];
    vaTitles.forEach((title, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-valign-btn';
      btn.textContent = vaIcons[i];
      btn.title = title;
      btn.dataset.value = vaValues[i];
      btn.addEventListener('click', () => {
        this.tbVertAlignBtns.forEach((b, j) => b.classList.toggle('active', j === i));
      });
      vaRow.appendChild(btn);
      this.tbVertAlignBtns.push(btn);
    });

    // 세로쓰기
    const vwLabel = this.checkboxLabel('세로쓰기(E):');
    this.tbVertWriteCheck = vwLabel.querySelector('input') as HTMLInputElement;
    this.tbVertWriteCheck.disabled = true;
    vaRow.appendChild(vwLabel);
    attrFs.appendChild(vaRow);

    // 영문 눕힘/세움
    const engRow = this.row();
    engRow.appendChild(this.label(''));
    this.tbEngLay = document.createElement('button');
    this.tbEngLay.className = 'pp-wrap-btn pp-eng-btn';
    this.tbEngLay.textContent = '가\nA B';
    this.tbEngLay.title = '영문 눕힘(O)';
    this.tbEngLay.disabled = true;
    engRow.appendChild(this.tbEngLay);
    this.tbEngStand = document.createElement('button');
    this.tbEngStand.className = 'pp-wrap-btn pp-eng-btn';
    this.tbEngStand.textContent = '가\nA\nB';
    this.tbEngStand.title = '영문 세움(U)';
    this.tbEngStand.disabled = true;
    engRow.appendChild(this.tbEngStand);
    attrFs.appendChild(engRow);

    // 한 줄로 입력
    const slRow = this.row();
    const slLabel = this.checkboxLabel('한 줄로 입력(S)');
    this.tbSingleLineCheck = slLabel.querySelector('input') as HTMLInputElement;
    this.tbSingleLineCheck.disabled = true;
    slRow.appendChild(slLabel);
    attrFs.appendChild(slRow);

    // ── 필드 ──
    const fieldFs = this.fieldset('필드');
    panel.appendChild(fieldFs);

    const fnRow = this.row();
    fnRow.appendChild(this.label('필드 이름(N):'));
    this.tbFieldNameInput = document.createElement('input');
    this.tbFieldNameInput.type = 'text';
    this.tbFieldNameInput.className = 'dialog-input';
    this.tbFieldNameInput.style.flex = '1';
    this.tbFieldNameInput.disabled = true;
    fnRow.appendChild(this.tbFieldNameInput);
    fieldFs.appendChild(fnRow);

    const fmRow = this.row();
    const fmLabel = this.checkboxLabel('양식 모드에서 편집 가능(F)');
    this.tbFormModeCheck = fmLabel.querySelector('input') as HTMLInputElement;
    this.tbFormModeCheck.disabled = true;
    fmRow.appendChild(fmLabel);
    fieldFs.appendChild(fmRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  그림자 탭
  // ════════════════════════════════════════════════════════

  private buildShadowPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 종류 ──
    const typeFs = this.fieldset('종류');
    panel.appendChild(typeFs);

    const grid = document.createElement('div');
    grid.className = 'pp-shadow-grid';
    this.shadowTypeBtns = [];
    // 10개 그림자 유형 (2×5): 없음 + 9가지 방향/스타일
    const shadowLabels = [
      '없음', '왼쪽 위', '위', '오른쪽 위', '오른쪽',
      '왼쪽', '왼쪽 아래', '아래', '오른쪽 아래', '양쪽',
    ];
    const shadowIcons = [
      '□', '◰', '◱', '◲', '◳',
      '◰', '◱', '◲', '◳', '▣',
    ];
    shadowLabels.forEach((lbl, i) => {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-shadow-type-btn';
      btn.textContent = shadowIcons[i];
      btn.title = lbl;
      btn.addEventListener('click', () => {
        this.shadowTypeBtns.forEach((b, j) => b.classList.toggle('active', j === i));
        const enabled = i > 0;
        this.shadowColorInput.disabled = !enabled;
        this.shadowHInput.disabled = !enabled;
        this.shadowVInput.disabled = !enabled;
        this.shadowDirBtns.forEach(b => b.disabled = !enabled);
        // 타입 선택 시 기본 오프셋 자동 설정
        if (enabled) {
          // 방향별 기본 오프셋 (mm)
          const offsets: [number,number][] = [
            [0,0],       // 0: 없음
            [-1.2,-1.2], // 1: 왼쪽 위
            [0,-1.2],    // 2: 위
            [1.2,-1.2],  // 3: 오른쪽 위
            [1.2,0],     // 4: 오른쪽
            [-1.2,0],    // 5: 왼쪽
            [-1.2,1.2],  // 6: 왼쪽 아래
            [0,1.2],     // 7: 아래
            [1.2,1.2],   // 8: 오른쪽 아래
            [1.2,1.2],   // 9: 양쪽
          ];
          const [dx, dy] = offsets[i] ?? [1.2, 1.2];
          this.shadowHInput.value = dx.toFixed(1);
          this.shadowVInput.value = dy.toFixed(1);
        }
      });
      grid.appendChild(btn);
      this.shadowTypeBtns.push(btn);
    });
    typeFs.appendChild(grid);

    // ── 그림자 ──
    const shadowFs = this.fieldset('그림자');
    panel.appendChild(shadowFs);

    const cRow = this.row();
    cRow.appendChild(this.label('그림자 색(C):'));
    this.shadowColorInput = this.colorInput('#b2b2b2');
    this.shadowColorInput.disabled = true; // 초기 비활성 (타입 선택 시 활성)
    cRow.appendChild(this.shadowColorInput);
    shadowFs.appendChild(cRow);

    const hRow = this.row();
    hRow.appendChild(this.label('가로 방향 이동(H):'));
    this.shadowHInput = this.numberInput();
    this.shadowHInput.value = '0.0';
    this.shadowHInput.disabled = true;
    hRow.appendChild(this.shadowHInput);
    hRow.appendChild(this.unit('mm'));

    // 8방향 버튼 (3×3 - 중앙 제외)
    const dirGrid = document.createElement('div');
    dirGrid.className = 'pp-direction-grid';
    this.shadowDirBtns = [];
    const dirIcons = ['↖', '↑', '↗', '←', '', '→', '↙', '↓', '↘'];
    dirIcons.forEach((icon, i) => {
      if (i === 4) {
        // 중앙 빈칸
        const spacer = document.createElement('div');
        spacer.className = 'pp-dir-spacer';
        spacer.textContent = '✕';
        dirGrid.appendChild(spacer);
        return;
      }
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn pp-dir-btn';
      btn.textContent = icon;
      btn.disabled = true; // 초기 비활성
      btn.addEventListener('click', () => {
        this.shadowDirBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        // 방향에 따라 offset 자동 설정
        const offsets: [number,number][] = [[-1,-1],[0,-1],[1,-1],[-1,0],[0,0],[1,0],[-1,1],[0,1],[1,1]];
        const dirIdx = [0,1,2,3,/*4skip*/5,6,7,8][this.shadowDirBtns.indexOf(btn)] ?? 8;
        const [dx, dy] = offsets[dirIdx] ?? [1, 1];
        this.shadowHInput.value = (dx * 1.2).toFixed(1);
        this.shadowVInput.value = (dy * 1.2).toFixed(1);
      });
      dirGrid.appendChild(btn);
      this.shadowDirBtns.push(btn);
    });
    hRow.appendChild(dirGrid);
    shadowFs.appendChild(hRow);

    const vRow = this.row();
    vRow.appendChild(this.label('세로 방향 이동(V):'));
    this.shadowVInput = this.numberInput();
    this.shadowVInput.value = '0.0';
    this.shadowVInput.disabled = true;
    vRow.appendChild(this.shadowVInput);
    vRow.appendChild(this.unit('mm'));
    shadowFs.appendChild(vRow);

    // ── 투명도 설정 ──
    const transFs = this.fieldset('투명도 설정');
    panel.appendChild(transFs);
    const transRow = this.row();
    transRow.appendChild(this.label('투명도(I):'));
    this.shadowTransInput = this.numberInput(0, 100, 1);
    this.shadowTransInput.value = '0';
    this.shadowTransInput.disabled = true;
    transRow.appendChild(this.shadowTransInput);
    transRow.appendChild(this.unit('%'));
    transFs.appendChild(transRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  그림 탭
  // ════════════════════════════════════════════════════════

  private buildPicturePanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    // ── 파일 이름 ──
    const fileFs = this.fieldset('파일 이름');
    panel.appendChild(fileFs);
    const fileRow = this.row();
    // [Task #741 후속] 외부 file path 그림 영역 dialog 표시 영역. populateFromProps 영역
    // 영역 props.externalPath 영역 보유 시 file path + embed=false 영역 갱신.
    this.picFileNameInput = document.createElement('input');
    this.picFileNameInput.type = 'text';
    this.picFileNameInput.className = 'dialog-input';
    this.picFileNameInput.style.width = '280px';
    this.picFileNameInput.readOnly = true;
    this.picFileNameInput.value = '(문서에 포함된 그림)';
    fileRow.appendChild(this.picFileNameInput);
    const embedLabel = this.checkboxLabel('문서에 포함');
    this.picEmbedCheck = embedLabel.querySelector('input') as HTMLInputElement;
    this.picEmbedCheck.checked = true;
    this.picEmbedCheck.disabled = true;
    fileRow.appendChild(embedLabel);
    fileFs.appendChild(fileRow);

    // ── 확대/축소 비율 ──
    const scaleFs = this.fieldset('확대/축소 비율');
    panel.appendChild(scaleFs);

    const sxRow = this.row();
    sxRow.appendChild(this.label('가로'));
    this.picScaleXInput = this.numberInput(1, 1000, 0.01);
    this.picScaleXInput.style.width = '70px';
    sxRow.appendChild(this.picScaleXInput);
    sxRow.appendChild(this.unit('%'));
    // 아이콘 버튼들
    const scalePresets = [
      { label: '🔍', title: '원래 크기로', pct: 100 },
      { label: '½', title: '1/2배', pct: 50 },
      { label: '⅔', title: '2/3배', pct: 67 },
      { label: '³⁄₂', title: '3/2배', pct: 150 },
      { label: '×2', title: '2배', pct: 200 },
    ];
    for (const p of scalePresets) {
      const btn = document.createElement('button');
      btn.className = 'pp-wrap-btn';
      btn.textContent = p.label;
      btn.title = p.title;
      btn.addEventListener('click', () => {
        this.picScaleXInput.value = String(p.pct);
        if (this.picKeepRatioCheck.checked) {
          this.picScaleYInput.value = String(p.pct);
        }
      });
      sxRow.appendChild(btn);
    }
    scaleFs.appendChild(sxRow);

    const syRow = this.row();
    syRow.appendChild(this.label('세로'));
    this.picScaleYInput = this.numberInput(1, 1000, 0.01);
    this.picScaleYInput.style.width = '70px';
    syRow.appendChild(this.picScaleYInput);
    syRow.appendChild(this.unit('%'));
    scaleFs.appendChild(syRow);

    const ratioRow = this.row();
    const ratioLabel = this.checkboxLabel('가로 세로 같은 비율 유지');
    this.picKeepRatioCheck = ratioLabel.querySelector('input') as HTMLInputElement;
    ratioRow.appendChild(ratioLabel);
    const resetBtn = document.createElement('button');
    resetBtn.className = 'dialog-btn';
    resetBtn.textContent = '원래 그림으로';
    resetBtn.style.marginLeft = '12px';
    resetBtn.addEventListener('click', () => {
      this.picScaleXInput.value = '100';
      this.picScaleYInput.value = '100';
      this.picCropLeftInput.value = '0.00';
      this.picCropTopInput.value = '0.00';
      this.picCropRightInput.value = '0.00';
      this.picCropBottomInput.value = '0.00';
      // 효과 초기화
      if (this.picEffectRadios[0]) this.picEffectRadios[0].checked = true;
      this.picBrightnessInput.value = '0';
      this.picContrastInput.value = '0';
    });
    ratioRow.appendChild(resetBtn);
    scaleFs.appendChild(ratioRow);

    // 비율 유지 이벤트
    this.picScaleXInput.addEventListener('input', () => {
      if (this.picKeepRatioCheck.checked) {
        this.picScaleYInput.value = this.picScaleXInput.value;
      }
    });
    this.picScaleYInput.addEventListener('input', () => {
      if (this.picKeepRatioCheck.checked) {
        this.picScaleXInput.value = this.picScaleYInput.value;
      }
    });

    // ── 그림 자르기 ──
    const cropFs = this.fieldset('그림 자르기');
    panel.appendChild(cropFs);
    const cropRow1 = this.row();
    cropRow1.appendChild(this.label('왼쪽'));
    this.picCropLeftInput = this.numberInput(0);
    this.picCropLeftInput.value = '0.00';
    cropRow1.appendChild(this.picCropLeftInput);
    cropRow1.appendChild(this.unit('mm'));
    cropRow1.appendChild(this.label('위쪽'));
    this.picCropTopInput = this.numberInput(0);
    this.picCropTopInput.value = '0.00';
    cropRow1.appendChild(this.picCropTopInput);
    cropRow1.appendChild(this.unit('mm'));
    // 모두 스피너
    cropRow1.appendChild(this.label('모두'));
    const cropSync = this.numberInput(0);
    cropSync.className = 'dialog-input pp-sync-spinner';
    cropSync.addEventListener('input', () => {
      const v = cropSync.value;
      this.picCropLeftInput.value = v;
      this.picCropTopInput.value = v;
      this.picCropRightInput.value = v;
      this.picCropBottomInput.value = v;
    });
    cropRow1.appendChild(cropSync);
    cropFs.appendChild(cropRow1);

    const cropRow2 = this.row();
    cropRow2.appendChild(this.label('오른쪽'));
    this.picCropRightInput = this.numberInput(0);
    this.picCropRightInput.value = '0.00';
    cropRow2.appendChild(this.picCropRightInput);
    cropRow2.appendChild(this.unit('mm'));
    cropRow2.appendChild(this.label('아래쪽'));
    this.picCropBottomInput = this.numberInput(0);
    this.picCropBottomInput.value = '0.00';
    cropRow2.appendChild(this.picCropBottomInput);
    cropRow2.appendChild(this.unit('mm'));
    cropFs.appendChild(cropRow2);

    // ── 그림 여백 ──
    const padFs = this.fieldset('그림 여백');
    panel.appendChild(padFs);
    const padRow1 = this.row();
    padRow1.appendChild(this.label('왼쪽'));
    this.picPadLeftInput = this.numberInput(0);
    this.picPadLeftInput.value = '0.00';
    padRow1.appendChild(this.picPadLeftInput);
    padRow1.appendChild(this.unit('mm'));
    padRow1.appendChild(this.label('위쪽'));
    this.picPadTopInput = this.numberInput(0);
    this.picPadTopInput.value = '0.00';
    padRow1.appendChild(this.picPadTopInput);
    padRow1.appendChild(this.unit('mm'));
    padRow1.appendChild(this.label('모두'));
    const padSync = this.numberInput(0);
    padSync.className = 'dialog-input pp-sync-spinner';
    padSync.addEventListener('input', () => {
      const v = padSync.value;
      this.picPadLeftInput.value = v;
      this.picPadTopInput.value = v;
      this.picPadRightInput.value = v;
      this.picPadBottomInput.value = v;
    });
    padRow1.appendChild(padSync);
    padFs.appendChild(padRow1);

    const padRow2 = this.row();
    padRow2.appendChild(this.label('오른쪽'));
    this.picPadRightInput = this.numberInput(0);
    this.picPadRightInput.value = '0.00';
    padRow2.appendChild(this.picPadRightInput);
    padRow2.appendChild(this.unit('mm'));
    padRow2.appendChild(this.label('아래쪽'));
    this.picPadBottomInput = this.numberInput(0);
    this.picPadBottomInput.value = '0.00';
    padRow2.appendChild(this.picPadBottomInput);
    padRow2.appendChild(this.unit('mm'));
    padFs.appendChild(padRow2);

    // ── 그림 효과 ──
    const effectFs = this.fieldset('그림 효과');
    panel.appendChild(effectFs);

    const effectMain = this.row();
    effectMain.style.alignItems = 'flex-start';

    // 좌측: 라디오 4개 (세로 배치)
    const radioCol = document.createElement('div');
    radioCol.className = 'pp-effect-radios';
    const effectNames = [
      { value: 'RealPic', label: '효과 없음' },
      { value: 'GrayScale', label: '회색조' },
      { value: 'BlackWhite', label: '흑백' },
      { value: 'Original', label: '원래 그림에서' },
    ];
    this.picEffectRadios = [];
    effectNames.forEach((e) => {
      const lbl = document.createElement('label');
      lbl.className = 'dialog-radio';
      const radio = document.createElement('input');
      radio.type = 'radio';
      radio.name = 'pp-pic-effect';
      radio.value = e.value;
      lbl.appendChild(radio);
      lbl.appendChild(document.createTextNode(` ${e.label}`));
      radioCol.appendChild(lbl);
      this.picEffectRadios.push(radio);
    });
    effectMain.appendChild(radioCol);

    // 우측: 밝기/대비/워터마크/반전
    const attrCol = document.createElement('div');
    attrCol.className = 'pp-effect-attrs';
    const brRow = this.row();
    brRow.appendChild(this.label('밝기'));
    this.picBrightnessInput = this.numberInput(-100, 100, 1);
    this.picBrightnessInput.value = '0';
    this.picBrightnessInput.style.width = '60px';
    brRow.appendChild(this.picBrightnessInput);
    brRow.appendChild(this.unit('%'));
    attrCol.appendChild(brRow);
    const ctRow = this.row();
    ctRow.appendChild(this.label('대비'));
    this.picContrastInput = this.numberInput(-100, 100, 1);
    this.picContrastInput.value = '0';
    this.picContrastInput.style.width = '60px';
    ctRow.appendChild(this.picContrastInput);
    ctRow.appendChild(this.unit('%'));
    attrCol.appendChild(ctRow);
    const wmLabel = this.checkboxLabel('워터마크 효과');
    this.picWatermarkCheck = wmLabel.querySelector('input') as HTMLInputElement;
    this.picWatermarkCheck.addEventListener('change', () => {
      if (this.picWatermarkCheck.checked) {
        this.picBrightnessInput.value = '70';
        this.picContrastInput.value = '-50';
      }
    });
    attrCol.appendChild(wmLabel);
    const invertLabel = this.checkboxLabel('그림 반전');
    const invertCheck = invertLabel.querySelector('input') as HTMLInputElement;
    invertCheck.disabled = true;
    attrCol.appendChild(invertLabel);
    effectMain.appendChild(attrCol);
    effectFs.appendChild(effectMain);

    // ── 투명도 설정 ──
    const transFs = this.fieldset('투명도 설정');
    panel.appendChild(transFs);
    const transRow = this.row();
    transRow.appendChild(this.label('투명도'));
    const transInput = this.numberInput(0, 100, 1);
    transInput.value = '0';
    transInput.disabled = true;
    transRow.appendChild(transInput);
    transRow.appendChild(this.unit('%'));
    transFs.appendChild(transRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  반사 탭
  // ════════════════════════════════════════════════════════

  private buildReflectionPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    const fs = this.fieldset('반사 효과');
    panel.appendChild(fs);

    const noneLabel = this.checkboxLabel('반사 없음');
    const noneCheck = noneLabel.querySelector('input') as HTMLInputElement;
    noneCheck.checked = true;
    fs.appendChild(noneLabel);

    // 3×5 프리셋 그리드 (비활성)
    const grid = document.createElement('div');
    grid.className = 'pp-preset-grid pp-reflect-grid';
    for (let i = 0; i < 15; i++) {
      const btn = document.createElement('button');
      btn.className = 'pp-preset-btn';
      btn.textContent = '🖼';
      btn.disabled = true;
      grid.appendChild(btn);
    }
    fs.appendChild(grid);

    // 속성
    const sizeRow = this.row();
    sizeRow.appendChild(this.label('크기'));
    const sizeSlider = document.createElement('input');
    sizeSlider.type = 'range';
    sizeSlider.className = 'pp-slider';
    sizeSlider.disabled = true;
    sizeRow.appendChild(sizeSlider);
    const sizeInput = this.numberInput(0, 100, 1);
    sizeInput.disabled = true;
    sizeRow.appendChild(sizeInput);
    fs.appendChild(sizeRow);

    const distRow = this.row();
    distRow.appendChild(this.label('거리'));
    const distSlider = document.createElement('input');
    distSlider.type = 'range';
    distSlider.className = 'pp-slider';
    distSlider.disabled = true;
    distRow.appendChild(distSlider);
    const distInput = this.numberInput(0, 100, 1);
    distInput.disabled = true;
    distRow.appendChild(distInput);
    distRow.appendChild(this.unit('pt'));
    fs.appendChild(distRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  네온 탭
  // ════════════════════════════════════════════════════════

  private buildGlowPanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    const fs = this.fieldset('네온 효과');
    panel.appendChild(fs);

    const noneLabel = this.checkboxLabel('네온 없음');
    const noneCheck = noneLabel.querySelector('input') as HTMLInputElement;
    noneCheck.checked = true;
    fs.appendChild(noneLabel);

    // 3×6 프리셋 그리드
    const grid = document.createElement('div');
    grid.className = 'pp-preset-grid pp-glow-grid';
    for (let i = 0; i < 18; i++) {
      const btn = document.createElement('button');
      btn.className = 'pp-preset-btn';
      btn.textContent = '🖼';
      btn.disabled = true;
      grid.appendChild(btn);
    }
    fs.appendChild(grid);

    // 속성
    const colorRow = this.row();
    colorRow.appendChild(this.label('색'));
    const colorInput = this.colorInput('#ffff00');
    colorInput.disabled = true;
    colorRow.appendChild(colorInput);
    fs.appendChild(colorRow);

    const transRow = this.row();
    transRow.appendChild(this.label('투명도'));
    const transSlider = document.createElement('input');
    transSlider.type = 'range';
    transSlider.className = 'pp-slider';
    transSlider.disabled = true;
    transRow.appendChild(transSlider);
    const transInput = this.numberInput(0, 100, 1);
    transInput.disabled = true;
    transRow.appendChild(transInput);
    fs.appendChild(transRow);

    const sizeRow = this.row();
    sizeRow.appendChild(this.label('크기'));
    const sizeSlider = document.createElement('input');
    sizeSlider.type = 'range';
    sizeSlider.className = 'pp-slider';
    sizeSlider.disabled = true;
    sizeRow.appendChild(sizeSlider);
    const sizeInput = this.numberInput(0, 100, 1);
    sizeInput.disabled = true;
    sizeRow.appendChild(sizeInput);
    sizeRow.appendChild(this.unit('pt'));
    fs.appendChild(sizeRow);

    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  열은 테두리 탭
  // ════════════════════════════════════════════════════════

  private buildSoftEdgePanel(): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';

    const fs = this.fieldset('열은 테두리 효과');
    panel.appendChild(fs);

    const noneLabel = this.checkboxLabel('열은 테두리 없음');
    fs.appendChild(noneLabel);

    // 6개 프리셋 버튼
    const grid = document.createElement('div');
    grid.className = 'pp-preset-grid pp-softedge-grid';
    for (let i = 0; i < 6; i++) {
      const btn = document.createElement('button');
      btn.className = 'pp-preset-btn';
      btn.textContent = '🖼';
      btn.disabled = true;
      grid.appendChild(btn);
    }
    fs.appendChild(grid);

    // 크기 슬라이더
    const sizeRow = this.row();
    sizeRow.appendChild(this.label('크기'));
    const sizeSlider = document.createElement('input');
    sizeSlider.type = 'range';
    sizeSlider.className = 'pp-slider';
    sizeSlider.min = '0';
    sizeSlider.max = '50';
    sizeSlider.value = '3';
    sizeSlider.disabled = true;
    sizeRow.appendChild(sizeSlider);
    const sizeInput = this.numberInput(0, 50, 0.1);
    sizeInput.value = '3.0';
    sizeInput.disabled = true;
    sizeRow.appendChild(sizeInput);
    sizeRow.appendChild(this.unit('pt'));
    fs.appendChild(sizeRow);

    return panel;
  }

  /** 미구현 탭 스텁 패널 */
  private buildStubPanel(name: string): HTMLDivElement {
    const panel = document.createElement('div');
    panel.className = 'dialog-tab-panel';
    const msg = document.createElement('div');
    msg.className = 'pp-stub-msg';
    msg.textContent = `[${name}] 탭은 추후 구현 예정입니다.`;
    panel.appendChild(msg);
    return panel;
  }

  // ════════════════════════════════════════════════════════
  //  설정/취소
  // ════════════════════════════════════════════════════════

  private handleOk(): void {
    if (!this.props) { this.hide(); return; }
    const updated: Record<string, unknown> = {};

    // 크기
    const newW = mmToHwp(parseFloat(this.widthInput.value) || 0);
    const newH = mmToHwp(parseFloat(this.heightInput.value) || 0);
    if (newW !== this.props.width) updated['width'] = newW;
    if (newH !== this.props.height) updated['height'] = newH;

    // 위치
    const tac = this.treatAsCharCheck.checked;
    if (tac !== this.props.treatAsChar) updated['treatAsChar'] = tac;

    if (!tac) {
      const tw = this.getSelectedWrap();
      if (tw !== this.props.textWrap) updated['textWrap'] = tw;
      const hr = this.horzRelSelect.value;
      if (hr !== this.props.horzRelTo) updated['horzRelTo'] = hr;
      const ha = this.horzAlignSelect.value;
      if (ha !== this.props.horzAlign) updated['horzAlign'] = ha;
      const ho = mmToHwp(parseFloat(this.horzOffsetInput.value) || 0);
      if (ho !== this.props.horzOffset) updated['horzOffset'] = ho;
      const vr = this.vertRelSelect.value;
      if (vr !== this.props.vertRelTo) updated['vertRelTo'] = vr;
      const va = this.vertAlignSelect.value;
      if (va !== this.props.vertAlign) updated['vertAlign'] = va;
      const vo = mmToHwp(parseFloat(this.vertOffsetInput.value) || 0);
      if (vo !== this.props.vertOffset) updated['vertOffset'] = vo;
    }

    // 기타
    const desc = this.descInput.value;
    if (desc !== this.props.description) updated['description'] = desc;

    // Shape(글상자) 전용 속성
    if ((this.objectType === 'shape' || this.objectType === 'line') && this.shapeProps) {
      // 글상자 여백
      const ml = mmToHwp(parseFloat(this.tbMarginLeftInput?.value) || 0);
      const mr = mmToHwp(parseFloat(this.tbMarginRightInput?.value) || 0);
      const mt = mmToHwp(parseFloat(this.tbMarginTopInput?.value) || 0);
      const mb = mmToHwp(parseFloat(this.tbMarginBottomInput?.value) || 0);
      if (ml !== (this.shapeProps.tbMarginLeft ?? 0)) updated['tbMarginLeft'] = ml;
      if (mr !== (this.shapeProps.tbMarginRight ?? 0)) updated['tbMarginRight'] = mr;
      if (mt !== (this.shapeProps.tbMarginTop ?? 0)) updated['tbMarginTop'] = mt;
      if (mb !== (this.shapeProps.tbMarginBottom ?? 0)) updated['tbMarginBottom'] = mb;

      // 세로 정렬 (아이콘 버튼)
      const activeVa = this.tbVertAlignBtns.find(b => b.classList.contains('active'));
      const vaVal = activeVa?.dataset.value ?? 'Top';
      if (vaVal !== (this.shapeProps.tbVerticalAlign ?? 'Top')) updated['tbVerticalAlign'] = vaVal;

      // 회전
      if (this.rotationInput && !this.rotationInput.disabled) {
        const rot = parseInt(this.rotationInput.value) || 0;
        if (rot !== (this.shapeProps.rotationAngle ?? 0)) updated['rotationAngle'] = rot;
      }
      // 대칭
      if (this.horzFlipCheck && !this.horzFlipCheck.disabled) {
        const hf = this.horzFlipCheck.checked;
        if (hf !== !!this.shapeProps.horzFlip) updated['horzFlip'] = hf;
      }
      if (this.vertFlipCheck && !this.vertFlipCheck.disabled) {
        const vf = this.vertFlipCheck.checked;
        if (vf !== !!this.shapeProps.vertFlip) updated['vertFlip'] = vf;
      }

      // 선 (색/굵기/종류/끝모양/화살표)
      if (this.lineColorInput) {
        const bc = hexToColorRef(this.lineColorInput.value);
        if (bc !== (this.shapeProps.borderColor ?? 0)) updated['borderColor'] = bc;
      }
      if (this.lineWidthInput) {
        const bw = mmToHwp(parseFloat(this.lineWidthInput.value) || 0);
        if (bw !== (this.shapeProps.borderWidth ?? 0)) updated['borderWidth'] = bw;
      }
      if (this.lineTypeSelect) {
        const lt = parseInt(this.lineTypeSelect.value) || 0;
        if (lt !== (this.shapeProps.lineType ?? 1)) updated['lineType'] = lt;
      }
      if (this.lineEndSelect) {
        const le = parseInt(this.lineEndSelect.value) || 0;
        if (le !== (this.shapeProps.lineEndShape ?? 0)) updated['lineEndShape'] = le;
      }
      if (this.arrowStartSelect) {
        const as_ = parseInt(this.arrowStartSelect.value) || 0;
        if (as_ !== (this.shapeProps.arrowStart ?? 0)) updated['arrowStart'] = as_;
      }
      if (this.arrowEndSelect) {
        const ae = parseInt(this.arrowEndSelect.value) || 0;
        if (ae !== (this.shapeProps.arrowEnd ?? 0)) updated['arrowEnd'] = ae;
      }
      if (this.arrowStartSizeSelect) {
        const ass = parseInt(this.arrowStartSizeSelect.value) || 0;
        if (ass !== (this.shapeProps.arrowStartSize ?? 0)) updated['arrowStartSize'] = ass;
      }
      if (this.arrowEndSizeSelect) {
        const aes = parseInt(this.arrowEndSizeSelect.value) || 0;
        if (aes !== (this.shapeProps.arrowEndSize ?? 0)) updated['arrowEndSize'] = aes;
      }

      // 모서리 곡률
      if (this.cornerCustomRadio?.checked && this.cornerCustomInput) {
        const rr = parseInt(this.cornerCustomInput.value) || 0;
        if (rr !== (this.shapeProps.roundRate ?? 0)) updated['roundRate'] = rr;
      } else {
        const activeCorner = this.cornerBtns.findIndex(b => b.classList.contains('active'));
        let rr = 0;
        if (activeCorner === 1) rr = 20;       // 둥근 모양
        else if (activeCorner === 2) rr = 50;   // 반원
        if (rr !== (this.shapeProps.roundRate ?? 0)) updated['roundRate'] = rr;
      }

      // 채우기
      let fillType = 'none';
      if (this.fillSolidRadio?.checked) fillType = 'solid';
      else if (this.fillGradientRadio?.checked) fillType = 'gradient';
      if (fillType !== (this.shapeProps.fillType ?? 'none')) updated['fillType'] = fillType;

      if (fillType === 'solid' && this.solidFaceColor) {
        // 항상 전송 — SolidFill이 없을 수 있으므로 비교 생략
        updated['fillBgColor'] = hexToColorRef(this.solidFaceColor.value);
        updated['fillPatColor'] = hexToColorRef(this.solidPatColor.value);
        if (this.solidPatternSelect) {
          updated['fillPatType'] = parseInt(this.solidPatternSelect.value) || -1;
        }
      }

      if (fillType === 'gradient') {
        if (this.gradTypeSelect) updated['gradientType'] = parseInt(this.gradTypeSelect.value) || 1;
        if (this.gradTiltInput) updated['gradientAngle'] = parseInt(this.gradTiltInput.value) || 0;
        if (this.gradCenterXInput) updated['gradientCenterX'] = parseInt(this.gradCenterXInput.value) || 0;
        if (this.gradCenterYInput) updated['gradientCenterY'] = parseInt(this.gradCenterYInput.value) || 0;
        if (this.gradBlurInput) updated['gradientBlur'] = parseInt(this.gradBlurInput.value) || 0;
      }

      // 채우기 투명도 (한컴 호환: alpha=0 → 불투명, alpha=255 → 완전 투명)
      if (this.fillTransInput && (fillType === 'solid' || fillType === 'gradient')) {
        const transPct = parseInt(this.fillTransInput.value) || 0;
        const alpha = Math.round(transPct * 255 / 100);
        updated['fillAlpha'] = alpha;
      }

      // 그림자
      if (this.shadowTypeBtns.length > 0) {
        const activeIdx = this.shadowTypeBtns.findIndex(b => b.classList.contains('active'));
        const shadowType = activeIdx > 0 ? activeIdx : 0;
        updated['shadowType'] = shadowType;
        if (shadowType > 0) {
          updated['shadowColor'] = hexToColorRef(this.shadowColorInput.value);
          updated['shadowOffsetX'] = mmToHwp(parseFloat(this.shadowHInput.value) || 0);
          updated['shadowOffsetY'] = mmToHwp(parseFloat(this.shadowVInput.value) || 0);
        } else {
          updated['shadowOffsetX'] = 0;
          updated['shadowOffsetY'] = 0;
        }
      }
    }

    // Picture(그림) 전용 속성
    if (this.objectType === 'image' && this.props) {
      const pp = this.props;

      // 회전/대칭
      if (this.rotationInput && !this.rotationInput.disabled) {
        const rot = (parseInt(this.rotationInput.value) || 0) * 100; // 도→raw
        if (rot !== (pp.rotationAngle ?? 0)) updated['rotationAngle'] = rot;
      }
      if (this.horzFlipCheck && !this.horzFlipCheck.disabled) {
        const hf = this.horzFlipCheck.checked;
        if (hf !== !!pp.horzFlip) updated['horzFlip'] = hf;
      }
      if (this.vertFlipCheck && !this.vertFlipCheck.disabled) {
        const vf = this.vertFlipCheck.checked;
        if (vf !== !!pp.vertFlip) updated['vertFlip'] = vf;
      }

      // 바깥 여백
      if (this.outerMarginLeftInput) {
        const ml = mmToHwp(parseFloat(this.outerMarginLeftInput.value) || 0);
        if (ml !== (pp.outerMarginLeft ?? 0)) updated['outerMarginLeft'] = ml;
      }
      if (this.outerMarginRightInput) {
        const mr = mmToHwp(parseFloat(this.outerMarginRightInput.value) || 0);
        if (mr !== (pp.outerMarginRight ?? 0)) updated['outerMarginRight'] = mr;
      }
      if (this.outerMarginTopInput) {
        const mt = mmToHwp(parseFloat(this.outerMarginTopInput.value) || 0);
        if (mt !== (pp.outerMarginTop ?? 0)) updated['outerMarginTop'] = mt;
      }
      if (this.outerMarginBottomInput) {
        const mb = mmToHwp(parseFloat(this.outerMarginBottomInput.value) || 0);
        if (mb !== (pp.outerMarginBottom ?? 0)) updated['outerMarginBottom'] = mb;
      }

      // 캡션
      if (this.captionBtns.length > 0) {
        const activeIdx = this.captionBtns.findIndex(b => b.classList.contains('active'));
        const hasCaption = activeIdx >= 0 && activeIdx !== 4; // 4 = 중앙(개체 자리)은 캡션 없음
        updated['hasCaption'] = hasCaption; // 항상 전달 — Rust 캡션 분기 진입 보장
        if (hasCaption) {
          const { direction, vertAlign } = this.gridIndexToCaption(activeIdx);
          updated['captionDirection'] = direction;
          updated['captionVertAlign'] = vertAlign;
          updated['captionWidth'] = mmToHwp(parseFloat(this.captionSizeInput.value) || 0);
          updated['captionSpacing'] = mmToHwp(parseFloat(this.captionGapInput.value) || 0);
          updated['captionIncludeMargin'] = this.captionExpandCheck.checked;
        }
      }

      // 테두리
      if (this.lineColorInput) {
        const bc = hexToColorRef(this.lineColorInput.value);
        if (bc !== (pp.borderColor ?? 0)) updated['borderColor'] = bc;
      }
      if (this.lineWidthInput) {
        const bw = mmToHwp(parseFloat(this.lineWidthInput.value) || 0);
        if (bw !== (pp.borderWidth ?? 0)) updated['borderWidth'] = bw;
      }

      // 그림 탭 — 확대/축소 → 크기 변환
      if (this.picScaleXInput && pp.originalWidth > 0) {
        const scaleX = parseFloat(this.picScaleXInput.value) || 100;
        const scaleY = parseFloat(this.picScaleYInput.value) || 100;
        const newW = Math.round(pp.originalWidth * scaleX / 100);
        const newH = Math.round(pp.originalHeight * scaleY / 100);
        if (newW !== pp.width) updated['width'] = newW;
        if (newH !== pp.height) updated['height'] = newH;
      }

      // 그림 탭 — 자르기
      if (this.picCropLeftInput) {
        const cl = mmToHwp(parseFloat(this.picCropLeftInput.value) || 0);
        if (cl !== (pp.cropLeft ?? 0)) updated['cropLeft'] = cl;
        const ct = mmToHwp(parseFloat(this.picCropTopInput.value) || 0);
        if (ct !== (pp.cropTop ?? 0)) updated['cropTop'] = ct;
        const cr = mmToHwp(parseFloat(this.picCropRightInput.value) || 0);
        if (cr !== (pp.cropRight ?? 0)) updated['cropRight'] = cr;
        const cb = mmToHwp(parseFloat(this.picCropBottomInput.value) || 0);
        if (cb !== (pp.cropBottom ?? 0)) updated['cropBottom'] = cb;
      }

      // 그림 탭 — 안쪽 여백
      if (this.picPadLeftInput) {
        const pl = mmToHwp(parseFloat(this.picPadLeftInput.value) || 0);
        if (pl !== (pp.paddingLeft ?? 0)) updated['paddingLeft'] = pl;
        const pt_ = mmToHwp(parseFloat(this.picPadTopInput.value) || 0);
        if (pt_ !== (pp.paddingTop ?? 0)) updated['paddingTop'] = pt_;
        const pr = mmToHwp(parseFloat(this.picPadRightInput.value) || 0);
        if (pr !== (pp.paddingRight ?? 0)) updated['paddingRight'] = pr;
        const pb = mmToHwp(parseFloat(this.picPadBottomInput.value) || 0);
        if (pb !== (pp.paddingBottom ?? 0)) updated['paddingBottom'] = pb;
      }

      // 그림 탭 — 효과
      if (this.picEffectRadios.length > 0) {
        const selected = this.picEffectRadios.find(r => r.checked);
        if (selected) {
          let effectVal = selected.value;
          if (effectVal === 'Original') effectVal = 'RealPic';
          if (effectVal !== (pp.effect ?? 'RealPic')) updated['effect'] = effectVal;
        }
      }
      if (this.picBrightnessInput) {
        const br = parseInt(this.picBrightnessInput.value) || 0;
        if (br !== (pp.brightness ?? 0)) updated['brightness'] = br;
      }
      if (this.picContrastInput) {
        const ct = parseInt(this.picContrastInput.value) || 0;
        if (ct !== (pp.contrast ?? 0)) updated['contrast'] = ct;
      }
    }

    if (Object.keys(updated).length > 0) {
      if (this.objectType === 'shape' || this.objectType === 'line') {
        this.wasm.setShapeProperties(this.sec, this.para, this.ci, updated);
      } else {
        this.wasm.setPictureProperties(this.sec, this.para, this.ci, updated);
      }
      this.eventBus.emit('document-changed');
    }
    this.hide();
  }

  // ════════════════════════════════════════════════════════
  //  데이터 ↔ UI
  // ════════════════════════════════════════════════════════

  private populateFromProps(): void {
    if (!this.props) return;
    // [Task #741 후속 — 한컴 viewer 정합] 외부 file path 그림 영역 dialog 표시 영역.
    // props.externalPath 영역 영역 그대로 표시 (resolved path 영역, 한컴 viewer 영역 영역
    // 원본 절대 경로 영역 영역 access 부재 시 HWP file 영역 영역 같은 dir 영역 image 영역
    // 영역 영역 path 영역 영역 갱신 — populate_external_images_from_dir / inject_external_image
    // 영역 영역 변경 영역 ~~basename~~ → resolved local path 영역 영역).
    if (this.picFileNameInput && this.picEmbedCheck) {
      if (this.props.externalPath) {
        this.picFileNameInput.value = this.props.externalPath;
        this.picEmbedCheck.checked = false;
      } else {
        this.picFileNameInput.value = '(문서에 포함된 그림)';
        this.picEmbedCheck.checked = true;
      }
    }
    this.widthInput.value = hwpToMm(this.props.width).toFixed(2);
    this.heightInput.value = hwpToMm(this.props.height).toFixed(2);
    this.treatAsCharCheck.checked = this.props.treatAsChar;
    this.selectWrap(this.wrapValues.indexOf(this.props.textWrap));
    this.horzRelSelect.value = this.props.horzRelTo;
    this.horzAlignSelect.value = this.props.horzAlign;
    this.horzOffsetInput.value = hwpToMm(this.props.horzOffset).toFixed(2);
    this.vertRelSelect.value = this.props.vertRelTo;
    this.vertAlignSelect.value = this.props.vertAlign;
    this.vertOffsetInput.value = hwpToMm(this.props.vertOffset).toFixed(2);
    this.descInput.value = this.props.description;
    this.skewHInput.value = '0';
    this.skewVInput.value = '0';
    this.updatePositionVisibility();

    // Shape/Line 전용 필드
    if ((this.objectType === 'shape' || this.objectType === 'line') && this.shapeProps) {
      const sp = this.shapeProps;

      // 기본 탭 — 회전/대칭
      if (this.rotationInput) {
        this.rotationInput.value = String(sp.rotationAngle ?? 0);
        this.rotationInput.disabled = false;
      }
      if (this.horzFlipCheck) {
        this.horzFlipCheck.checked = !!sp.horzFlip;
        this.horzFlipCheck.disabled = false;
      }
      if (this.vertFlipCheck) {
        this.vertFlipCheck.checked = !!sp.vertFlip;
        this.vertFlipCheck.disabled = false;
      }

      // 글상자 탭 — 여백
      if (this.tbMarginLeftInput) this.tbMarginLeftInput.value = hwpToMm(sp.tbMarginLeft ?? 510).toFixed(2);
      if (this.tbMarginRightInput) this.tbMarginRightInput.value = hwpToMm(sp.tbMarginRight ?? 510).toFixed(2);
      if (this.tbMarginTopInput) this.tbMarginTopInput.value = hwpToMm(sp.tbMarginTop ?? 141).toFixed(2);
      if (this.tbMarginBottomInput) this.tbMarginBottomInput.value = hwpToMm(sp.tbMarginBottom ?? 141).toFixed(2);

      // 글상자 탭 — 세로 정렬 아이콘 버튼
      const va = sp.tbVerticalAlign ?? 'Top';
      this.tbVertAlignBtns.forEach(b => b.classList.toggle('active', b.dataset.value === va));

      // 선 탭 — borderColor/borderWidth
      if (this.lineColorInput && sp.borderColor !== undefined) {
        this.lineColorInput.value = colorRefToHex(sp.borderColor);
      }
      if (this.lineWidthInput && sp.borderWidth !== undefined) {
        this.lineWidthInput.value = hwpToMm(sp.borderWidth).toFixed(2);
      }

      // 선 탭 — 선 종류/끝모양/화살표
      if (this.lineTypeSelect && sp.lineType !== undefined) this.lineTypeSelect.value = String(sp.lineType);
      if (this.lineEndSelect && sp.lineEndShape !== undefined) this.lineEndSelect.value = String(sp.lineEndShape);
      if (this.arrowStartSelect && sp.arrowStart !== undefined) this.arrowStartSelect.value = String(sp.arrowStart);
      if (this.arrowEndSelect && sp.arrowEnd !== undefined) this.arrowEndSelect.value = String(sp.arrowEnd);
      if (this.arrowStartSizeSelect && sp.arrowStartSize !== undefined) this.arrowStartSizeSelect.value = String(sp.arrowStartSize);
      if (this.arrowEndSizeSelect && sp.arrowEndSize !== undefined) this.arrowEndSizeSelect.value = String(sp.arrowEndSize);

      // 선 탭 — 모서리 곡률
      if (this.cornerBtns.length > 0 && sp.roundRate !== undefined) {
        const rr = sp.roundRate;
        if (rr === 0) {
          this.cornerBtns.forEach((b, i) => b.classList.toggle('active', i === 0));
        } else if (rr >= 50) {
          this.cornerBtns.forEach((b, i) => b.classList.toggle('active', i === 2));
        } else if (rr > 0 && rr < 50) {
          // 곡률 지정 모드
          if (this.cornerCustomRadio) this.cornerCustomRadio.checked = true;
          if (this.cornerCustomInput) {
            this.cornerCustomInput.value = String(rr);
            this.cornerCustomInput.disabled = false;
          }
          this.cornerBtns.forEach(b => b.classList.remove('active'));
        }
      }

      // 채우기 탭 — fillType
      const ft = sp.fillType ?? 'none';
      if (this.fillNoneRadio) this.fillNoneRadio.checked = (ft === 'none');
      if (this.fillSolidRadio) this.fillSolidRadio.checked = (ft === 'solid');
      if (this.fillGradientRadio) this.fillGradientRadio.checked = (ft === 'gradient');

      // 채우기 — 단색
      if (this.solidFaceColor && sp.fillBgColor !== undefined) {
        this.solidFaceColor.value = colorRefToHex(sp.fillBgColor);
      }
      if (this.solidPatColor && sp.fillPatColor !== undefined) {
        this.solidPatColor.value = colorRefToHex(sp.fillPatColor);
      }
      if (this.solidPatternSelect && sp.fillPatType !== undefined) {
        // fillPatType은 정수 — select 값으로 매핑은 추후 세분화
      }

      // 채우기 — 그러데이션
      if (this.gradTypeSelect && sp.gradientType !== undefined) {
        // gradientType 정수 → select 인덱스 매핑은 추후 세분화
      }
      if (this.gradCenterXInput && sp.gradientCenterX !== undefined) this.gradCenterXInput.value = String(sp.gradientCenterX);
      if (this.gradCenterYInput && sp.gradientCenterY !== undefined) this.gradCenterYInput.value = String(sp.gradientCenterY);
      if (this.gradBlurInput && sp.gradientBlur !== undefined) this.gradBlurInput.value = String(sp.gradientBlur);
      if (this.gradTiltInput && sp.gradientAngle !== undefined) this.gradTiltInput.value = String(sp.gradientAngle);

      // 채우기 — 투명도 (한컴 호환: alpha=0 → 불투명, alpha=255 → 완전 투명)
      if (this.fillTransInput && sp.fillAlpha !== undefined) {
        const pct = Math.round(sp.fillAlpha * 100 / 255);
        this.fillTransInput.value = String(pct);
        this.fillTransInput.disabled = false;
      }

      // 그림자 탭 초기화
      if (this.shadowTypeBtns.length > 0) {
        const st = (sp as any).shadowType ?? 0;
        this.shadowTypeBtns.forEach((b, i) => b.classList.toggle('active', i === st));
        const enabled = st > 0;
        this.shadowColorInput.disabled = !enabled;
        this.shadowHInput.disabled = !enabled;
        this.shadowVInput.disabled = !enabled;
        this.shadowDirBtns.forEach(b => b.disabled = !enabled);
        if ((sp as any).shadowColor !== undefined) {
          this.shadowColorInput.value = colorRefToHex((sp as any).shadowColor);
        }
        if ((sp as any).shadowOffsetX !== undefined) {
          this.shadowHInput.value = hwpToMm((sp as any).shadowOffsetX).toFixed(1);
        }
        if ((sp as any).shadowOffsetY !== undefined) {
          this.shadowVInput.value = hwpToMm((sp as any).shadowOffsetY).toFixed(1);
        }
      }

      // 채우기 영역 활성화 상태 업데이트
      if (this.solidArea) {
        const isSolid = ft === 'solid';
        const isGrad = ft === 'gradient';
        this.solidArea.style.opacity = isSolid ? '1' : '0.4';
        this.gradientArea.style.opacity = isGrad ? '1' : '0.4';
        this.setAreaDisabled(this.solidArea, !isSolid);
        this.setAreaDisabled(this.gradientArea, !isGrad);
      }
    } else {
      // 그림 개체일 때
      const pp = this.props!;

      // 기본 탭 — 회전/대칭 활성화
      if (this.rotationInput) {
        this.rotationInput.value = String(Math.round((pp.rotationAngle ?? 0) / 100));
        this.rotationInput.disabled = false;
      }
      if (this.horzFlipCheck) {
        this.horzFlipCheck.checked = !!pp.horzFlip;
        this.horzFlipCheck.disabled = false;
      }
      if (this.vertFlipCheck) {
        this.vertFlipCheck.checked = !!pp.vertFlip;
        this.vertFlipCheck.disabled = false;
      }

      // 여백/캡션 탭 — 바깥 여백
      if (this.outerMarginLeftInput) this.outerMarginLeftInput.value = hwpToMm(pp.outerMarginLeft ?? 0).toFixed(2);
      if (this.outerMarginRightInput) this.outerMarginRightInput.value = hwpToMm(pp.outerMarginRight ?? 0).toFixed(2);
      if (this.outerMarginTopInput) this.outerMarginTopInput.value = hwpToMm(pp.outerMarginTop ?? 0).toFixed(2);
      if (this.outerMarginBottomInput) this.outerMarginBottomInput.value = hwpToMm(pp.outerMarginBottom ?? 0).toFixed(2);

      // 여백/캡션 탭 — 캡션 바인딩
      if (this.captionBtns.length > 0) {
        // 캡션 버튼 활성화
        this.captionBtns.forEach(b => b.disabled = false);
        this.captionSizeInput.disabled = false;
        this.captionGapInput.disabled = false;
        this.captionExpandCheck.disabled = false;

        if (pp.hasCaption) {
          // 방향 + 세로 정렬 → 3×3 그리드 인덱스 매핑
          const gridIdx = this.captionGridIndex(pp.captionDirection, pp.captionVertAlign);
          this.captionBtns.forEach((b, j) => b.classList.toggle('active', j === gridIdx));
          this.captionSizeInput.value = hwpToMm(pp.captionWidth ?? 0).toFixed(2);
          this.captionGapInput.value = hwpToMm(pp.captionSpacing ?? 0).toFixed(2);
          this.captionExpandCheck.checked = !!pp.captionIncludeMargin;
        } else {
          this.captionBtns.forEach(b => b.classList.remove('active'));
          this.captionSizeInput.value = '30.00';
          this.captionGapInput.value = '3.00';
          this.captionExpandCheck.checked = false;
        }
      }

      // 선 탭 — 테두리 바인딩
      if (this.lineColorInput && pp.borderColor !== undefined) {
        this.lineColorInput.value = colorRefToHex(pp.borderColor);
      }
      if (this.lineWidthInput && pp.borderWidth !== undefined) {
        this.lineWidthInput.value = hwpToMm(pp.borderWidth).toFixed(2);
      }

      // 그림 탭 — 확대/축소 비율
      if (this.picScaleXInput && pp.originalWidth > 0) {
        this.picScaleXInput.value = ((pp.width / pp.originalWidth) * 100).toFixed(2);
        this.picScaleYInput.value = ((pp.height / pp.originalHeight) * 100).toFixed(2);
      }
      // 그림 탭 — 자르기
      if (this.picCropLeftInput) {
        this.picCropLeftInput.value = hwpToMm(pp.cropLeft ?? 0).toFixed(2);
        this.picCropTopInput.value = hwpToMm(pp.cropTop ?? 0).toFixed(2);
        this.picCropRightInput.value = hwpToMm(pp.cropRight ?? 0).toFixed(2);
        this.picCropBottomInput.value = hwpToMm(pp.cropBottom ?? 0).toFixed(2);
      }
      // 그림 탭 — 안쪽 여백 (그림 여백)
      if (this.picPadLeftInput) {
        this.picPadLeftInput.value = hwpToMm(pp.paddingLeft ?? 0).toFixed(2);
        this.picPadTopInput.value = hwpToMm(pp.paddingTop ?? 0).toFixed(2);
        this.picPadRightInput.value = hwpToMm(pp.paddingRight ?? 0).toFixed(2);
        this.picPadBottomInput.value = hwpToMm(pp.paddingBottom ?? 0).toFixed(2);
      }
      // 그림 탭 — 효과
      if (this.picEffectRadios.length > 0) {
        const effectVal = pp.effect ?? 'RealPic';
        this.picEffectRadios.forEach(r => {
          r.checked = r.value === effectVal || (r.value === 'Original' && effectVal === 'RealPic' && (pp.brightness !== 0 || pp.contrast !== 0));
        });
        if (!this.picEffectRadios.some(r => r.checked)) {
          this.picEffectRadios[0].checked = true;
        }
      }
      if (this.picBrightnessInput) this.picBrightnessInput.value = String(pp.brightness ?? 0);
      if (this.picContrastInput) this.picContrastInput.value = String(pp.contrast ?? 0);
      if (this.picWatermarkCheck) {
        this.picWatermarkCheck.checked = (pp.brightness === 70 && pp.contrast === -50);
      }
    }
  }

  private updatePositionVisibility(): void {
    const hidden = this.treatAsCharCheck.checked;
    this.posDetailEls.forEach(el => {
      el.style.display = hidden ? 'none' : '';
    });
  }

  private selectWrap(idx: number): void {
    this.wrapBtns.forEach((b, i) => b.classList.toggle('active', i === idx));
  }

  private getSelectedWrap(): string {
    const idx = this.wrapBtns.findIndex(b => b.classList.contains('active'));
    return idx >= 0 ? this.wrapValues[idx] : 'Square';
  }

  /** 캡션 direction + vertAlign → 3×3 그리드 인덱스 */
  private captionGridIndex(dir: string, vAlign: string): number {
    // 0:왼위 1:위 2:오위 3:왼 4:중앙 5:오 6:왼아 7:아래 8:오아
    const col = dir === 'Left' ? 0 : dir === 'Right' ? 2 : 1;
    const row = (dir === 'Left' || dir === 'Right')
      ? (vAlign === 'Top' ? 0 : vAlign === 'Bottom' ? 2 : 1)
      : (dir === 'Top' ? 0 : 2);
    return row * 3 + col;
  }

  /** 3×3 그리드 인덱스 → { direction, vertAlign } */
  private gridIndexToCaption(idx: number): { direction: string; vertAlign: string } {
    const col = idx % 3;
    const row = Math.floor(idx / 3);
    if (col === 0) return { direction: 'Left', vertAlign: row === 0 ? 'Top' : row === 1 ? 'Center' : 'Bottom' };
    if (col === 2) return { direction: 'Right', vertAlign: row === 0 ? 'Top' : row === 1 ? 'Center' : 'Bottom' };
    // col === 1 (중앙열)
    return { direction: row <= 1 ? 'Top' : 'Bottom', vertAlign: 'Top' };
  }

  /**
   * 개체 설명문 서브 대화상자 표시
   */
  private showDescriptionPrompt(): void {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';

    const dlg = document.createElement('div');
    dlg.className = 'dialog-wrap pp-desc-dialog';

    // 타이틀
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '개체 설명문';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => overlay.remove());
    titleBar.appendChild(closeBtn);
    dlg.appendChild(titleBar);

    // 메인: 좌측(textarea) + 우측(버튼)
    const mainRow = document.createElement('div');
    mainRow.className = 'cs-main-row';

    const leftCol = document.createElement('div');
    leftCol.className = 'cs-left-col';
    const textarea = document.createElement('textarea');
    textarea.className = 'pp-desc-textarea';
    textarea.value = this.descInput.value;
    leftCol.appendChild(textarea);

    const rightCol = document.createElement('div');
    rightCol.className = 'cs-right-col';
    const okBtn = document.createElement('button');
    okBtn.className = 'dialog-btn dialog-btn-primary';
    okBtn.textContent = '확인(D)';
    okBtn.addEventListener('click', () => {
      this.descInput.value = textarea.value;
      overlay.remove();
    });
    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '취소';
    cancelBtn.addEventListener('click', () => overlay.remove());
    rightCol.appendChild(okBtn);
    rightCol.appendChild(cancelBtn);

    mainRow.appendChild(leftCol);
    mainRow.appendChild(rightCol);
    dlg.appendChild(mainRow);
    overlay.appendChild(dlg);

    // Escape / 오버레이 클릭
    overlay.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') { e.stopPropagation(); overlay.remove(); }
    });
    overlay.addEventListener('mousedown', (e) => {
      if (e.target === overlay) overlay.remove();
    });

    document.body.appendChild(overlay);
    setTimeout(() => textarea.focus(), 50);
  }

  // ════════════════════════════════════════════════════════
  //  드래그
  // ════════════════════════════════════════════════════════

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

  // ════════════════════════════════════════════════════════
  //  유틸리티
  // ════════════════════════════════════════════════════════

  /** 영역 내 모든 input/select/button을 disabled 처리 */
  private setAreaDisabled(area: HTMLElement, disabled: boolean): void {
    area.querySelectorAll('input, select, button').forEach(el => {
      (el as HTMLInputElement).disabled = disabled;
    });
  }

  // ════════════════════════════════════════════════════════
  //  DOM 헬퍼
  // ════════════════════════════════════════════════════════

  private fieldset(title: string): HTMLFieldSetElement {
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

  private unit(text: string): HTMLSpanElement {
    const u = document.createElement('span');
    u.className = 'dialog-unit';
    u.textContent = text;
    return u;
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

  private colorInput(defaultVal: string): HTMLInputElement {
    const inp = document.createElement('input');
    inp.type = 'color';
    inp.className = 'cs-color-btn';
    inp.value = defaultVal;
    return inp;
  }

  private selectEl(options: [string, string][]): HTMLSelectElement {
    const sel = document.createElement('select');
    sel.className = 'dialog-select';
    for (const [val, lbl] of options) {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = lbl;
      sel.appendChild(opt);
    }
    return sel;
  }

  private sizeTypeSelect(): HTMLSelectElement {
    return this.selectEl([['fixed', '고정 값']]);
  }

  private checkboxLabel(text: string): HTMLLabelElement {
    const lb = document.createElement('label');
    lb.className = 'dialog-checkbox';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    lb.appendChild(cb);
    lb.appendChild(document.createTextNode(` ${text}`));
    return lb;
  }
}
