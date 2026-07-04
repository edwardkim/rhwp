/**
 * 유니코드 문자표 입력 대화상자
 *
 * 유니코드 블록별 문자 그리드를 표시하고 선택한 문자를 본문에 삽입한다.
 */
import type { CommandServices } from '@/command/types';
import { InsertTextCommand } from '@/engine/command';
import { enableDialogDrag } from './dialog-drag';

// ── 유니코드 블록 정의 ──

interface UnicodeBlock {
  name: string;
  start: number;
  end: number;
}

const UNICODE_BLOCKS: UnicodeBlock[] = [
  { name: '기본 라틴 문자', start: 0x0020, end: 0x007F },
  { name: '라틴 문자-1 보충', start: 0x0080, end: 0x00FF },
  { name: '라틴 확장-A', start: 0x0100, end: 0x017F },
  { name: '라틴 확장-B', start: 0x0180, end: 0x024F },
  { name: 'IPA 확장', start: 0x0250, end: 0x02AF },
  { name: '공백 변환 문자', start: 0x02B0, end: 0x02FF },
  { name: '조합 분음 부호', start: 0x0300, end: 0x036F },
  { name: '그리스·콥트 문자', start: 0x0370, end: 0x03FF },
  { name: '키릴 문자', start: 0x0400, end: 0x04FF },
  { name: '일반 구두점', start: 0x2000, end: 0x206F },
  { name: '위 첨자·아래 첨자', start: 0x2070, end: 0x209F },
  { name: '통화 기호', start: 0x20A0, end: 0x20CF },
  { name: '문자형 기호', start: 0x2100, end: 0x214F },
  { name: '숫자 형태', start: 0x2150, end: 0x218F },
  { name: '화살표', start: 0x2190, end: 0x21FF },
  { name: '수학 연산자', start: 0x2200, end: 0x22FF },
  { name: '기타 기술 기호', start: 0x2300, end: 0x23FF },
  { name: '제어 그림 문자', start: 0x2400, end: 0x243F },
  { name: '광학 문자 인식', start: 0x2440, end: 0x245F },
  { name: '테두리 문자', start: 0x2500, end: 0x257F },
  { name: '블록 요소', start: 0x2580, end: 0x259F },
  { name: '도형', start: 0x25A0, end: 0x25FF },
  { name: '여러 가지 기호', start: 0x2600, end: 0x26FF },
  { name: '딩뱃 기호', start: 0x2700, end: 0x27BF },
  { name: '여러 가지 수학 기호-A', start: 0x27C0, end: 0x27EF },
  { name: '화살표 보충-A', start: 0x27F0, end: 0x27FF },
  { name: '점자 패턴', start: 0x2800, end: 0x28FF },
  { name: '화살표 보충-B', start: 0x2900, end: 0x297F },
  { name: '여러 가지 수학 기호-B', start: 0x2980, end: 0x29FF },
  { name: 'CJK 기호 및 구두점', start: 0x3000, end: 0x303F },
  { name: '히라가나', start: 0x3040, end: 0x309F },
  { name: '가타카나', start: 0x30A0, end: 0x30FF },
  { name: '한글 호환 자모', start: 0x3130, end: 0x318F },
  { name: 'CJK 호환 문자', start: 0x3300, end: 0x33FF },
  { name: 'CJK 통합 한자 (일부)', start: 0x4E00, end: 0x4FFF },
  { name: '한글 음절 (가~깋)', start: 0xAC00, end: 0xAD0F },
  { name: '한글 음절 (나~닣)', start: 0xB098, end: 0xB1FF },
  { name: '반각·전각 형태', start: 0xFF00, end: 0xFFEF },
];

const COLS = 16;
const RECENT_KEY = 'rhwp-symbols-recent';
const MAX_RECENT = 32;

export class SymbolsDialog {
  private services: CommandServices;
  private _open = false;
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private blockList!: HTMLDivElement;
  private charGrid!: HTMLDivElement;
  private codeLabel!: HTMLSpanElement;
  private previewCell!: HTMLDivElement;
  private recentGrid!: HTMLDivElement;
  private selectedChar: string | null = null;
  private currentBlock: UnicodeBlock = UNICODE_BLOCKS[0];
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(services: CommandServices) {
    this.services = services;
  }

  isOpen(): boolean { return this._open; }

  show(): void {
    if (this._open) return;
    this._open = true;
    this.build();
    document.body.appendChild(this.overlay);

    // 키 이벤트 캡처
    this.captureHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        e.preventDefault();
        this.hide();
        return;
      }
      e.stopPropagation();
    };
    document.addEventListener('keydown', this.captureHandler, true);

    // 초기 블록 선택
    this.selectBlock(this.currentBlock);
    this.updateRecent();
  }

  hide(): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this._open = false;
    this.overlay?.remove();

    // 편집 영역 포커스 복원
    const ih = this.services.getInputHandler();
    ih?.focus();
  }

  // ── DOM 구성 ──

  private build(): void {
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap sym-dialog';

    // 타이틀
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '문자표 입력';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);
    enableDialogDrag(this.dialog, titleBar);

    // 본문
    const body = document.createElement('div');
    body.className = 'dialog-body sym-body';

    // 상단: 블록 목록 + 문자 그리드 + 미리보기/코드
    const top = document.createElement('div');
    top.className = 'sym-top';

    // 블록 목록 (왼쪽)
    const blockCol = document.createElement('div');
    blockCol.className = 'sym-block-col';
    const blockLabel = document.createElement('div');
    blockLabel.className = 'sym-label';
    blockLabel.textContent = '문자 영역(I):';
    blockCol.appendChild(blockLabel);
    this.blockList = document.createElement('div');
    this.blockList.className = 'sym-block-list';
    for (const block of UNICODE_BLOCKS) {
      const item = document.createElement('div');
      item.className = 'sym-block-item';
      item.textContent = block.name;
      item.addEventListener('click', () => this.selectBlock(block));
      this.blockList.appendChild(item);
    }
    blockCol.appendChild(this.blockList);
    top.appendChild(blockCol);

    // 문자 그리드 + 코드 (오른쪽)
    const rightCol = document.createElement('div');
    rightCol.className = 'sym-right-col';

    // 코드 행
    const codeRow = document.createElement('div');
    codeRow.className = 'sym-code-row';
    const selLabel = document.createElement('span');
    selLabel.className = 'sym-label';
    selLabel.textContent = '문자 선택(C):';
    codeRow.appendChild(selLabel);
    const codeSpacer = document.createElement('span');
    codeSpacer.style.flex = '1';
    codeRow.appendChild(codeSpacer);
    const codePrefix = document.createElement('span');
    codePrefix.className = 'sym-label';
    codePrefix.textContent = '유니코드(U):';
    codeRow.appendChild(codePrefix);
    this.codeLabel = document.createElement('span');
    this.codeLabel.className = 'sym-code-value';
    codeRow.appendChild(this.codeLabel);
    rightCol.appendChild(codeRow);

    // 그리드
    this.charGrid = document.createElement('div');
    this.charGrid.className = 'sym-char-grid';
    rightCol.appendChild(this.charGrid);

    // 미리보기
    this.previewCell = document.createElement('div');
    this.previewCell.className = 'sym-preview';
    rightCol.appendChild(this.previewCell);

    top.appendChild(rightCol);
    body.appendChild(top);

    // 최근 사용한 문자
    const recentLabel = document.createElement('div');
    recentLabel.className = 'sym-label';
    recentLabel.textContent = '최근 사용한 문자(Q):';
    recentLabel.style.marginTop = '8px';
    body.appendChild(recentLabel);

    this.recentGrid = document.createElement('div');
    this.recentGrid.className = 'sym-recent-grid';
    body.appendChild(this.recentGrid);

    this.dialog.appendChild(body);

    // 하단 버튼
    const footer = document.createElement('div');
    footer.className = 'dialog-footer';
    const insertBtn = document.createElement('button');
    insertBtn.className = 'dialog-btn dialog-btn-primary';
    insertBtn.textContent = '넣기(D)';
    insertBtn.addEventListener('click', () => this.doInsert());
    const cancelBtn = document.createElement('button');
    cancelBtn.className = 'dialog-btn';
    cancelBtn.textContent = '닫기';
    cancelBtn.addEventListener('click', () => this.hide());
    footer.appendChild(insertBtn);
    footer.appendChild(cancelBtn);
    this.dialog.appendChild(footer);

    this.overlay.appendChild(this.dialog);
  }

  // ── 블록 선택 ──

  private selectBlock(block: UnicodeBlock): void {
    this.currentBlock = block;

    // 목록 하이라이트
    const items = this.blockList.querySelectorAll('.sym-block-item');
    const idx = UNICODE_BLOCKS.indexOf(block);
    items.forEach((el, i) => {
      el.classList.toggle('selected', i === idx);
    });
    // 스크롤 into view
    items[idx]?.scrollIntoView({ block: 'nearest' });

    this.renderGrid(block);
    this.selectedChar = null;
    this.codeLabel.textContent = block.start.toString(16).toUpperCase().padStart(4, '0');
    this.previewCell.textContent = '';
  }

  private renderGrid(block: UnicodeBlock): void {
    this.charGrid.replaceChildren();
    const count = block.end - block.start + 1;
    const rows = Math.ceil(count / COLS);

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < COLS; c++) {
        const cp = block.start + r * COLS + c;
        const cell = document.createElement('div');
        cell.className = 'sym-cell';
        if (cp <= block.end) {
          const ch = String.fromCodePoint(cp);
          cell.textContent = ch;
          cell.title = `U+${cp.toString(16).toUpperCase().padStart(4, '0')}`;
          cell.addEventListener('click', () => this.selectChar(ch, cp));
          cell.addEventListener('dblclick', () => {
            this.selectChar(ch, cp);
            this.doInsert();
          });
        } else {
          cell.classList.add('empty');
        }
        this.charGrid.appendChild(cell);
      }
    }
  }

  // ── 문자 선택 ──

  private selectChar(ch: string, codePoint: number): void {
    this.selectedChar = ch;
    this.codeLabel.textContent = codePoint.toString(16).toUpperCase().padStart(4, '0');
    this.previewCell.textContent = ch;

    // 그리드 하이라이트
    this.charGrid.querySelectorAll('.sym-cell.selected').forEach(el => el.classList.remove('selected'));
    const idx = codePoint - this.currentBlock.start;
    const cells = this.charGrid.querySelectorAll('.sym-cell');
    cells[idx]?.classList.add('selected');
  }

  // ── 삽입 ──

  private doInsert(): void {
    if (!this.selectedChar) return;

    const ih = this.services.getInputHandler();
    if (!ih) return;

    const pos = ih.getCursorPosition();
    ih.executeOperation({
      kind: 'command',
      command: new InsertTextCommand(pos, this.selectedChar),
    });
    this.services.eventBus.emit('document-changed');

    // hidden textarea 포커스 복원 (후속 타이핑 가능하도록)
    ih.focus();

    // 최근 문자 저장
    this.addToRecent(this.selectedChar);
    this.updateRecent();
  }

  // ── 최근 사용 문자 ──

  private getRecentChars(): string[] {
    try {
      const raw = localStorage.getItem(RECENT_KEY);
      return raw ? JSON.parse(raw) : [];
    } catch {
      return [];
    }
  }

  private addToRecent(ch: string): void {
    const list = this.getRecentChars().filter(c => c !== ch);
    list.unshift(ch);
    if (list.length > MAX_RECENT) list.length = MAX_RECENT;
    localStorage.setItem(RECENT_KEY, JSON.stringify(list));
  }

  private updateRecent(): void {
    this.recentGrid.replaceChildren();
    const recents = this.getRecentChars();
    if (recents.length === 0) {
      const msg = document.createElement('span');
      msg.className = 'sym-recent-empty';
      msg.textContent = '최근에 [문자표]에서 사용한 문자가 없습니다.';
      this.recentGrid.appendChild(msg);
      return;
    }
    for (const ch of recents) {
      const cell = document.createElement('div');
      cell.className = 'sym-cell sym-recent-cell';
      cell.textContent = ch;
      const cp = ch.codePointAt(0) ?? 0;
      cell.title = `U+${cp.toString(16).toUpperCase().padStart(4, '0')}`;
      cell.addEventListener('click', () => this.selectChar(ch, cp));
      cell.addEventListener('dblclick', () => {
        this.selectChar(ch, cp);
        this.doInsert();
      });
      this.recentGrid.appendChild(cell);
    }
  }
}
