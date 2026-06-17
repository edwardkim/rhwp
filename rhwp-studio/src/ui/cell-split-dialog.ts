/**
 * 셀 나누기 대화상자
 *
 * 한컴 셀 나누기 대화상자를 재현한다.
 * ModalDialog를 상속하지 않고 독립 빌드 (커스텀 버튼 텍스트 + 2단 레이아웃).
 */
import { enableDialogDrag } from './dialog-drag';

export class CellSplitDialog {
  onApply: ((nRows: number, mCols: number, equalHeight: boolean, mergeFirst: boolean) => void) | null = null;

  private overlay: HTMLDivElement | null = null;
  private dialog: HTMLDivElement | null = null;
  private escHandler: ((e: KeyboardEvent) => void) | null = null;

  private rowCheck!: HTMLInputElement;
  private rowInput!: HTMLInputElement;
  private colCheck!: HTMLInputElement;
  private colInput!: HTMLInputElement;
  private equalHeightCheck!: HTMLInputElement;
  private mergeFirstCheck!: HTMLInputElement;

  private isMerged: boolean;

  constructor(isMerged: boolean) {
    this.isMerged = isMerged;
  }

  show(): void {
    this.build();
    this.escHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') this.hide();
    };
    document.body.appendChild(this.overlay!);
    document.addEventListener('keydown', this.escHandler);
  }

  hide(): void {
    if (this.escHandler) {
      document.removeEventListener('keydown', this.escHandler);
      this.escHandler = null;
    }
    this.overlay?.remove();
  }

  private build(): void {
    // 오버레이
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    // 다이얼로그
    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap';
    this.dialog.style.width = '340px';

    // 타이틀 바
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '셀 나누기';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);
    enableDialogDrag(this.dialog, titleBar);

    // 본문: 2단 레이아웃 (좌측=폼, 우측=버튼)
    const body = document.createElement('div');
    body.className = 'dialog-body';
    body.style.display = 'flex';
    body.style.gap = '12px';

    const leftPanel = document.createElement('div');
    leftPanel.style.flex = '1';

    const rightPanel = document.createElement('div');
    rightPanel.style.display = 'flex';
    rightPanel.style.flexDirection = 'column';
    rightPanel.style.gap = '6px';
    rightPanel.style.paddingTop = '4px';

    // --- 좌측: 줄/칸 나누기 섹션 ---
    const splitSection = document.createElement('div');
    splitSection.className = 'dialog-section';

    const splitTitle = document.createElement('div');
    splitTitle.className = 'dialog-section-title';
    splitTitle.textContent = '줄/칸 나누기';
    splitSection.appendChild(splitTitle);

    // 줄 수
    const rowRow = document.createElement('div');
    rowRow.className = 'dialog-row';
    this.rowCheck = document.createElement('input');
    this.rowCheck.type = 'checkbox';
    this.rowCheck.id = 'csd-row-check';
    const rowLabel = document.createElement('label');
    rowLabel.htmlFor = 'csd-row-check';
    rowLabel.className = 'dialog-label';
    rowLabel.textContent = '줄 수(R):';
    rowLabel.style.textAlign = 'left';
    this.rowInput = document.createElement('input');
    this.rowInput.type = 'number';
    this.rowInput.className = 'dialog-input';
    this.rowInput.style.width = '50px';
    this.rowInput.min = '1';
    this.rowInput.max = '256';
    this.rowInput.value = '2';
    this.rowInput.disabled = true;
    this.rowCheck.addEventListener('change', () => {
      this.rowInput.disabled = !this.rowCheck.checked;
    });
    rowRow.appendChild(this.rowCheck);
    rowRow.appendChild(rowLabel);
    rowRow.appendChild(this.rowInput);
    splitSection.appendChild(rowRow);

    // 칸 수
    const colRow = document.createElement('div');
    colRow.className = 'dialog-row';
    this.colCheck = document.createElement('input');
    this.colCheck.type = 'checkbox';
    this.colCheck.id = 'csd-col-check';
    this.colCheck.checked = true;
    const colLabel = document.createElement('label');
    colLabel.htmlFor = 'csd-col-check';
    colLabel.className = 'dialog-label';
    colLabel.textContent = '칸 수(C):';
    colLabel.style.textAlign = 'left';
    this.colInput = document.createElement('input');
    this.colInput.type = 'number';
    this.colInput.className = 'dialog-input';
    this.colInput.style.width = '50px';
    this.colInput.min = '1';
    this.colInput.max = '256';
    this.colInput.value = '2';
    this.colCheck.addEventListener('change', () => {
      this.colInput.disabled = !this.colCheck.checked;
    });
    colRow.appendChild(this.colCheck);
    colRow.appendChild(colLabel);
    colRow.appendChild(this.colInput);
    splitSection.appendChild(colRow);

    leftPanel.appendChild(splitSection);

    // --- 좌측: 선택 사항 섹션 ---
    const optSection = document.createElement('div');
    optSection.className = 'dialog-section';
    optSection.style.marginTop = '8px';

    const optTitle = document.createElement('div');
    optTitle.className = 'dialog-section-title';
    optTitle.textContent = '선택 사항';
    optSection.appendChild(optTitle);

    // 줄 높이를 같게 나누기
    const eqRow = document.createElement('div');
    eqRow.className = 'dialog-row';
    this.equalHeightCheck = document.createElement('input');
    this.equalHeightCheck.type = 'checkbox';
    this.equalHeightCheck.id = 'csd-eq-check';
    const eqLabel = document.createElement('label');
    eqLabel.htmlFor = 'csd-eq-check';
    eqLabel.textContent = '줄 높이를 같게 나누기(H)';
    eqRow.appendChild(this.equalHeightCheck);
    eqRow.appendChild(eqLabel);
    optSection.appendChild(eqRow);

    // 셀을 합친 후 나누기
    const mfRow = document.createElement('div');
    mfRow.className = 'dialog-row';
    this.mergeFirstCheck = document.createElement('input');
    this.mergeFirstCheck.type = 'checkbox';
    this.mergeFirstCheck.id = 'csd-mf-check';
    this.mergeFirstCheck.disabled = !this.isMerged;
    const mfLabel = document.createElement('label');
    mfLabel.htmlFor = 'csd-mf-check';
    mfLabel.textContent = '셀을 합친 후 나누기(M)';
    if (!this.isMerged) {
      mfLabel.style.color = '#999';
    }
    mfRow.appendChild(this.mergeFirstCheck);
    mfRow.appendChild(mfLabel);
    optSection.appendChild(mfRow);

    leftPanel.appendChild(optSection);

    // --- 우측: 버튼 ---
    const applyBtn = document.createElement('button');
    applyBtn.className = 'dialog-btn dialog-btn-primary';
    applyBtn.textContent = '나누기(D)';
    applyBtn.style.minWidth = '80px';
    applyBtn.addEventListener('click', () => this.doApply());

    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '취소';
    cancelBtn.style.minWidth = '80px';
    cancelBtn.addEventListener('click', () => this.hide());

    rightPanel.appendChild(applyBtn);
    rightPanel.appendChild(cancelBtn);

    body.appendChild(leftPanel);
    body.appendChild(rightPanel);
    this.dialog.appendChild(body);

    this.overlay.appendChild(this.dialog);
  }

  private doApply(): void {
    const nRows = this.rowCheck.checked ? Math.max(1, Math.min(256, parseInt(this.rowInput.value, 10) || 1)) : 1;
    const mCols = this.colCheck.checked ? Math.max(1, Math.min(256, parseInt(this.colInput.value, 10) || 1)) : 1;

    if (nRows === 1 && mCols === 1) {
      this.hide();
      return;
    }

    const equalHeight = this.equalHeightCheck.checked;
    const mergeFirst = this.mergeFirstCheck.checked;

    this.hide();
    if (this.onApply) {
      this.onApply(nRows, mCols, equalHeight, mergeFirst);
    }
  }
}
