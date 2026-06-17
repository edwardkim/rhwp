/**
 * 표 만들기 그리드 피커 (TableCreateDialog)
 * 한컴 스타일: 도구상자 버튼 아래 드롭다운으로 행×열 그리드 선택
 *
 *  ┌────────────────────┐
 *  │      [ 취소 ]      │
 *  ├────────────────────┤
 *  │ ■ ■ ■ □ □ □ □ □   │
 *  │ ■ ■ ■ □ □ □ □ □   │
 *  │ □ □ □ □ □ □ □ □   │
 *  │ □ □ □ □ □ □ □ □   │
 *  │ □ □ □ □ □ □ □ □   │
 *  ├────────────────────┤
 *  │  ⊞ 표 만들기...    │
 *  └────────────────────┘
 */

import { makeOption } from './dom-utils';
import { enableDialogDrag } from './dialog-drag';

const GRID_ROWS = 8;
const GRID_COLS = 10;
const CELL_SIZE = 16;   // px
const CELL_GAP = 2;     // px

export class TableCreateDialog {
  private overlay!: HTMLDivElement;
  private popup!: HTMLDivElement;
  private label!: HTMLDivElement;
  private cells: HTMLDivElement[] = [];
  private built = false;

  private hoverRow = -1;
  private hoverCol = -1;

  /** 그리드 선택 콜백 */
  onApply: ((rows: number, cols: number) => void) | null = null;

  /**
   * 그리드 피커를 표시한다.
   * @param anchor 앵커 요소 — 있으면 아래에 위치, 없으면 화면 중앙
   */
  show(anchor?: HTMLElement): void {
    if (!this.built) this.build();
    this.hoverRow = -1;
    this.hoverCol = -1;
    this.updateHighlight();
    document.body.appendChild(this.overlay);
    document.body.appendChild(this.popup);

    if (anchor) {
      const rect = anchor.getBoundingClientRect();
      this.popup.style.left = rect.left + 'px';
      this.popup.style.top = rect.bottom + 2 + 'px';
      this.popup.style.transform = 'none';
      requestAnimationFrame(() => {
        const pr = this.popup.getBoundingClientRect();
        if (pr.right > window.innerWidth) {
          this.popup.style.left = (window.innerWidth - pr.width - 4) + 'px';
        }
        if (pr.bottom > window.innerHeight) {
          this.popup.style.top = (rect.top - pr.height - 2) + 'px';
        }
      });
    } else {
      this.popup.style.left = '50%';
      this.popup.style.top = '50%';
      this.popup.style.transform = 'translate(-50%,-50%)';
    }
  }

  private hide(): void {
    this.popup.remove();
    this.overlay.remove();
  }

  private build(): void {
    // 투명 오버레이 (클릭 시 닫기)
    this.overlay = document.createElement('div');
    this.overlay.style.cssText = 'position:fixed;inset:0;z-index:9998;';
    this.overlay.addEventListener('click', () => this.hide());

    // 팝업 컨테이너
    this.popup = document.createElement('div');
    this.popup.style.cssText =
      'position:fixed;z-index:9999;background:var(--color-surface-raised);border:1px solid var(--ui-border-light);' +
      'box-shadow:var(--shadow-dropdown);padding:0;user-select:none;color:var(--color-text);';

    // ── 상단: 취소 버튼 ──
    const header = document.createElement('div');
    header.style.cssText = 'padding:4px 6px;border-bottom:1px solid var(--ui-border-light);';
    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = '취소';
    cancelBtn.style.cssText =
      'width:100%;padding:3px 0;font-size:12px;border:1px solid var(--color-border);' +
      'background:var(--color-surface);color:var(--color-text);cursor:pointer;border-radius:2px;color-scheme:inherit;';
    cancelBtn.addEventListener('click', () => this.hide());
    cancelBtn.addEventListener('mouseenter', () => { cancelBtn.style.background = 'var(--ui-hover)'; });
    cancelBtn.addEventListener('mouseleave', () => { cancelBtn.style.background = 'var(--color-surface)'; });
    header.appendChild(cancelBtn);
    this.popup.appendChild(header);

    // ── 중앙: 그리드 ──
    const gridWrap = document.createElement('div');
    gridWrap.style.cssText = 'padding:6px;';
    const grid = document.createElement('div');
    const gridW = GRID_COLS * (CELL_SIZE + CELL_GAP) - CELL_GAP;
    const gridH = GRID_ROWS * (CELL_SIZE + CELL_GAP) - CELL_GAP;
    grid.style.cssText =
      `position:relative;width:${gridW}px;height:${gridH}px;cursor:pointer;`;

    this.cells = [];
    for (let r = 0; r < GRID_ROWS; r++) {
      for (let c = 0; c < GRID_COLS; c++) {
        const cell = document.createElement('div');
        cell.style.cssText =
          `position:absolute;width:${CELL_SIZE}px;height:${CELL_SIZE}px;` +
          `left:${c * (CELL_SIZE + CELL_GAP)}px;top:${r * (CELL_SIZE + CELL_GAP)}px;` +
          'border:1px solid var(--color-border);box-sizing:border-box;background:var(--color-surface);';
        cell.dataset.row = String(r);
        cell.dataset.col = String(c);
        grid.appendChild(cell);
        this.cells.push(cell);
      }
    }

    grid.addEventListener('mousemove', (e: MouseEvent) => {
      const rect = grid.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      const col = Math.min(GRID_COLS - 1, Math.max(0, Math.floor(x / (CELL_SIZE + CELL_GAP))));
      const row = Math.min(GRID_ROWS - 1, Math.max(0, Math.floor(y / (CELL_SIZE + CELL_GAP))));
      if (row !== this.hoverRow || col !== this.hoverCol) {
        this.hoverRow = row;
        this.hoverCol = col;
        this.updateHighlight();
      }
    });

    grid.addEventListener('mouseleave', () => {
      this.hoverRow = -1;
      this.hoverCol = -1;
      this.updateHighlight();
    });

    grid.addEventListener('click', () => {
      if (this.hoverRow < 0 || this.hoverCol < 0) return;
      const rows = this.hoverRow + 1;
      const cols = this.hoverCol + 1;
      this.hide();
      if (this.onApply) {
        this.onApply(rows, cols);
      }
    });

    gridWrap.appendChild(grid);

    // 라벨: "3 × 4"
    this.label = document.createElement('div');
    this.label.style.cssText =
      'text-align:center;margin-top:4px;font-size:11px;color:var(--color-text-muted);font-family:sans-serif;height:14px;';
    gridWrap.appendChild(this.label);

    this.popup.appendChild(gridWrap);

    // ── 하단: 표 만들기... 링크 ──
    const footer = document.createElement('div');
    footer.style.cssText =
      'padding:5px 8px;border-top:1px solid var(--ui-border-light);cursor:pointer;font-size:12px;color:var(--color-text);';
    const icon = document.createElement('span');
    icon.style.marginRight = '4px';
    icon.textContent = '\u229E';
    footer.appendChild(icon);
    footer.appendChild(document.createTextNode('표 만들기...'));
    footer.addEventListener('mouseenter', () => { footer.style.background = 'var(--color-accent-bg)'; });
    footer.addEventListener('mouseleave', () => { footer.style.background = ''; });
    footer.addEventListener('click', () => {
      this.hide();
      this.showInputDialog();
    });
    this.popup.appendChild(footer);

    // Esc로 닫기
    this.popup.tabIndex = 0;
    this.popup.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        this.hide();
      }
    });

    this.built = true;
  }

  private updateHighlight(): void {
    for (let i = 0; i < this.cells.length; i++) {
      const cell = this.cells[i];
      const r = parseInt(cell.dataset.row!, 10);
      const c = parseInt(cell.dataset.col!, 10);
      if (this.hoverRow >= 0 && r <= this.hoverRow && c <= this.hoverCol) {
        cell.style.background = 'var(--color-accent-bg-light)';
        cell.style.borderColor = 'var(--color-primary)';
      } else {
        cell.style.background = 'var(--color-surface)';
        cell.style.borderColor = 'var(--color-border)';
      }
    }
    if (this.hoverRow >= 0) {
      this.label.textContent = `${this.hoverCol + 1} × ${this.hoverRow + 1}`;
    } else {
      this.label.textContent = '';
    }
  }

  /**
   * "표 만들기..." 클릭 시 상세 대화상자 표시 (한컴 스타일)
   *
   *  ┌─────────────────────────────────────────┐
   *  │ 표 만들기                          [×]  │
   *  ├─────────────────────────────┬───────────┤
   *  │ 줄/칸                       │ [만들기]  │
   *  │  줄 개수 [4  ▴▾]           │ [취  소]  │
   *  │  칸 개수 [5  ▴▾]           │           │
   *  │                             │           │
   *  │ 크기 지정                   │           │
   *  │  너비 [단에 맞춤▾] 148.0 mm│           │
   *  │  높이 [자동      ▾]  22.6 mm│          │
   *  │                             │           │
   *  │ 기타                        │           │
   *  │  ☐ 글자처럼 취급            │           │
   *  └─────────────────────────────┴───────────┘
   */
  private showInputDialog(): void {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';

    const dlg = document.createElement('div');
    dlg.className = 'dialog-wrap';
    dlg.style.width = '420px';

    // 오버레이 내부에 대화상자를 넣어야 flex 센터링이 작동함
    overlay.appendChild(dlg);
    // 타이틀
    const title = document.createElement('div');
    title.className = 'dialog-title';
    title.textContent = '표 만들기';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', close);
    title.appendChild(closeBtn);
    dlg.appendChild(title);
    enableDialogDrag(dlg, title);

    // 본문: 좌측 폼 + 우측 버튼
    const body = document.createElement('div');
    body.className = 'dialog-body';
    body.style.cssText = 'display:flex;gap:16px;padding:12px 16px;';

    // ── 좌측: 폼 영역 ──
    const form = document.createElement('div');
    form.style.cssText = 'flex:1;';

    // 줄/칸 섹션
    const sec1 = createSection('줄/칸');
    const rowInput = createSpinnerRow(sec1, '줄 개수', 4, 1, 256);
    const colInput = createSpinnerRow(sec1, '칸 개수', 5, 1, 256);
    form.appendChild(sec1);

    // 크기 지정 섹션
    const sec2 = createSection('크기 지정');

    const widthRow = document.createElement('div');
    widthRow.className = 'dialog-row';
    widthRow.appendChild(makeLabel('너비'));
    const widthMode = document.createElement('select');
    widthMode.className = 'dialog-select';
    widthMode.style.width = '90px';
    widthMode.appendChild(makeOption('fit', '단에 맞춤'));
    widthMode.appendChild(makeOption('custom', '직접 지정'));
    widthRow.appendChild(widthMode);
    const widthVal = document.createElement('input');
    widthVal.className = 'dialog-input';
    widthVal.type = 'number';
    widthVal.step = '0.1';
    widthVal.value = '148.0';
    widthVal.style.cssText = 'width:60px;margin-left:6px;';
    widthVal.disabled = true;
    widthRow.appendChild(widthVal);
    widthRow.appendChild(makeUnit('mm'));
    sec2.appendChild(widthRow);

    widthMode.addEventListener('change', () => {
      widthVal.disabled = widthMode.value === 'fit';
    });

    const heightRow = document.createElement('div');
    heightRow.className = 'dialog-row';
    heightRow.appendChild(makeLabel('높이'));
    const heightMode = document.createElement('select');
    heightMode.className = 'dialog-select';
    heightMode.style.width = '90px';
    heightMode.appendChild(makeOption('auto', '자동'));
    heightMode.appendChild(makeOption('custom', '직접 지정'));
    heightRow.appendChild(heightMode);
    const heightVal = document.createElement('input');
    heightVal.className = 'dialog-input';
    heightVal.type = 'number';
    heightVal.step = '0.1';
    heightVal.value = '22.6';
    heightVal.style.cssText = 'width:60px;margin-left:6px;';
    heightVal.disabled = true;
    heightRow.appendChild(heightVal);
    heightRow.appendChild(makeUnit('mm'));
    sec2.appendChild(heightRow);

    heightMode.addEventListener('change', () => {
      heightVal.disabled = heightMode.value === 'auto';
    });

    form.appendChild(sec2);

    // 기타 섹션
    const sec3 = createSection('기타');
    const treatRow = document.createElement('div');
    treatRow.className = 'dialog-row';
    const treatChk = document.createElement('input');
    treatChk.type = 'checkbox';
    treatChk.id = 'tc-treat-as-char';
    treatChk.checked = true;
    const treatLbl = document.createElement('label');
    treatLbl.htmlFor = 'tc-treat-as-char';
    treatLbl.textContent = ' 글자처럼 취급';
    treatLbl.style.fontSize = '12px';
    treatRow.appendChild(treatChk);
    treatRow.appendChild(treatLbl);
    sec3.appendChild(treatRow);
    form.appendChild(sec3);

    body.appendChild(form);

    // ── 우측: 버튼 영역 ──
    const btnCol = document.createElement('div');
    btnCol.style.cssText = 'display:flex;flex-direction:column;gap:6px;padding-top:2px;';
    const okBtn = document.createElement('button');
    okBtn.className = 'dialog-btn dialog-btn-primary';
    okBtn.textContent = '만들기';
    okBtn.style.width = '72px';
    okBtn.addEventListener('click', doApply);
    const cancelBtn2 = document.createElement('button');
    cancelBtn2.className = 'dialog-btn';
    cancelBtn2.textContent = '취소';
    cancelBtn2.style.width = '72px';
    cancelBtn2.addEventListener('click', close);
    btnCol.appendChild(okBtn);
    btnCol.appendChild(cancelBtn2);
    body.appendChild(btnCol);

    dlg.appendChild(body);

    // 키보드
    dlg.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Enter') { e.preventDefault(); doApply(); }
      else if (e.key === 'Escape') { e.preventDefault(); close(); }
    });

    const self = this;
    function close() { overlay.remove(); }
    function doApply() {
      const rows = Math.max(1, Math.min(256, parseInt(rowInput.value, 10) || 2));
      const cols = Math.max(1, Math.min(256, parseInt(colInput.value, 10) || 3));
      close();
      if (self.onApply) self.onApply(rows, cols);
    }

    document.body.appendChild(overlay);
    rowInput.focus();
    rowInput.select();
  }
}

// ── 헬퍼 함수 ──

function createSection(titleText: string): HTMLDivElement {
  const sec = document.createElement('div');
  sec.className = 'dialog-section';
  const t = document.createElement('div');
  t.className = 'dialog-section-title';
  t.textContent = titleText;
  sec.appendChild(t);
  return sec;
}

function createSpinnerRow(parent: HTMLElement, labelText: string, defaultVal: number, min: number, max: number): HTMLInputElement {
  const row = document.createElement('div');
  row.className = 'dialog-row';
  row.appendChild(makeLabel(labelText));
  const input = document.createElement('input');
  input.className = 'dialog-input';
  input.type = 'number';
  input.min = String(min);
  input.max = String(max);
  input.value = String(defaultVal);
  input.style.width = '70px';
  row.appendChild(input);
  parent.appendChild(row);
  return input;
}

function makeLabel(text: string): HTMLLabelElement {
  const lbl = document.createElement('label');
  lbl.className = 'dialog-label';
  lbl.textContent = text;
  return lbl;
}

function makeUnit(text: string): HTMLSpanElement {
  const sp = document.createElement('span');
  sp.className = 'dialog-unit';
  sp.textContent = text;
  return sp;
}
