/**
 * 책갈피 대화상자
 *
 * 책갈피 추가/이동/삭제/이름 바꾸기를 수행한다.
 */
import type { CommandServices } from '@/command/types';
import type { BookmarkInfo } from '@/core/types';
import { enableDialogDrag } from './dialog-drag';

type SortMode = 'name' | 'position';

export class BookmarkDialog {
  private services: CommandServices;
  private _open = false;
  private overlay!: HTMLDivElement;
  private dialog!: HTMLDivElement;
  private nameInput!: HTMLInputElement;
  private listEl!: HTMLDivElement;
  private statusLabel!: HTMLSpanElement;
  private sortMode: SortMode = 'position';
  private bookmarks: BookmarkInfo[] = [];
  private selectedIdx = -1;
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(services: CommandServices) {
    this.services = services;
  }

  isOpen(): boolean { return this._open; }

  show(): void {
    if (this._open) { this.nameInput?.focus(); return; }
    this._open = true;
    this.build();
    document.body.appendChild(this.overlay);

    this.captureHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation(); e.preventDefault();
        this.hide(); return;
      }
      e.stopPropagation();
    };
    document.addEventListener('keydown', this.captureHandler, true);

    this.refreshList();
    this.suggestName();
    this.nameInput.focus();
    this.nameInput.select();
  }

  hide(): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this._open = false;
    this.overlay?.remove();
    this.services.getInputHandler()?.focus();
  }

  // ── DOM 구성 ──

  private build(): void {
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    this.dialog = document.createElement('div');
    this.dialog.className = 'dialog-wrap bm-dialog';

    // 타이틀
    const titleBar = document.createElement('div');
    titleBar.className = 'dialog-title';
    titleBar.textContent = '책갈피';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.hide());
    titleBar.appendChild(closeBtn);
    this.dialog.appendChild(titleBar);
    enableDialogDrag(this.dialog, titleBar);

    // 본문
    const body = document.createElement('div');
    body.className = 'dialog-body bm-body';

    // 이름 입력 행
    const nameRow = document.createElement('div');
    nameRow.className = 'bm-row';
    const nameLabel = document.createElement('label');
    nameLabel.className = 'bm-label';
    nameLabel.textContent = '책갈피 이름(N):';
    this.nameInput = document.createElement('input');
    this.nameInput.type = 'text';
    this.nameInput.className = 'bm-name-input';
    this.nameInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') { e.preventDefault(); this.doAdd(); }
    });
    nameRow.appendChild(nameLabel);
    nameRow.appendChild(this.nameInput);
    body.appendChild(nameRow);

    // 목록 헤더 + 버튼 영역
    const midRow = document.createElement('div');
    midRow.className = 'bm-mid';

    const listCol = document.createElement('div');
    listCol.className = 'bm-list-col';
    const listLabel = document.createElement('div');
    listLabel.className = 'bm-label';
    listLabel.textContent = '책갈피 목록(L):';
    listCol.appendChild(listLabel);

    // 목록 헤더
    const listHeader = document.createElement('div');
    listHeader.className = 'bm-list-header';
    const hName = document.createElement('span');
    hName.textContent = '이름';
    hName.className = 'bm-list-hcol';
    const hType = document.createElement('span');
    hType.textContent = '종류';
    hType.className = 'bm-list-hcol bm-list-hcol-type';
    listHeader.appendChild(hName);
    listHeader.appendChild(hType);
    listCol.appendChild(listHeader);

    // 목록 본문
    this.listEl = document.createElement('div');
    this.listEl.className = 'bm-list';
    listCol.appendChild(this.listEl);

    // 상태 라벨
    this.statusLabel = document.createElement('div');
    this.statusLabel.className = 'bm-status';
    listCol.appendChild(this.statusLabel);

    midRow.appendChild(listCol);

    // 버튼 컬럼
    const btnCol = document.createElement('div');
    btnCol.className = 'bm-btn-col';
    const addBtn = this.createButton('넣기(D)', () => this.doAdd());
    addBtn.classList.add('dialog-btn-primary');
    const cancelBtn = this.createButton('취소', () => this.hide());
    const moveBtn = this.createButton('이동(M)', () => this.doMove());
    btnCol.appendChild(addBtn);
    btnCol.appendChild(cancelBtn);
    btnCol.appendChild(moveBtn);
    midRow.appendChild(btnCol);

    body.appendChild(midRow);

    // 아이콘 버튼 행 (이름 바꾸기, 삭제)
    const iconRow = document.createElement('div');
    iconRow.className = 'bm-icon-row';
    const renameBtn = document.createElement('button');
    renameBtn.className = 'bm-icon-btn';
    renameBtn.title = '책갈피 이름 바꾸기';
    renameBtn.textContent = '✏';
    renameBtn.addEventListener('click', () => this.doRename());
    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'bm-icon-btn';
    deleteBtn.title = '삭제';
    deleteBtn.textContent = '✕';
    deleteBtn.addEventListener('click', () => this.doDelete());
    iconRow.appendChild(renameBtn);
    iconRow.appendChild(deleteBtn);
    body.appendChild(iconRow);

    // 정렬 기준
    const sortRow = document.createElement('div');
    sortRow.className = 'bm-sort-row';
    const sortLabel = document.createElement('span');
    sortLabel.className = 'bm-label';
    sortLabel.textContent = '책갈피 정렬 기준';
    sortRow.appendChild(sortLabel);

    const radioName = document.createElement('label');
    radioName.className = 'bm-radio';
    const rName = document.createElement('input');
    rName.type = 'radio'; rName.name = 'bm-sort'; rName.value = 'name';
    rName.addEventListener('change', () => { this.sortMode = 'name'; this.refreshList(); });
    radioName.appendChild(rName);
    radioName.appendChild(document.createTextNode(' 이름(A)'));

    const radioPos = document.createElement('label');
    radioPos.className = 'bm-radio';
    const rPos = document.createElement('input');
    rPos.type = 'radio'; rPos.name = 'bm-sort'; rPos.value = 'position'; rPos.checked = true;
    rPos.addEventListener('change', () => { this.sortMode = 'position'; this.refreshList(); });
    radioPos.appendChild(rPos);
    radioPos.appendChild(document.createTextNode(' 위치(P)'));

    sortRow.appendChild(radioName);
    sortRow.appendChild(radioPos);
    body.appendChild(sortRow);

    this.dialog.appendChild(body);
    this.overlay.appendChild(this.dialog);
  }

  private createButton(text: string, handler: () => void): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.className = 'dialog-btn';
    btn.textContent = text;
    btn.addEventListener('click', handler);
    return btn;
  }

  // ── 목록 갱신 ──

  private refreshList(): void {
    this.bookmarks = this.services.wasm.getBookmarks();
    if (this.sortMode === 'name') {
      this.bookmarks.sort((a, b) => a.name.localeCompare(b.name));
    }
    // position은 이미 문서 순서대로 반환됨

    this.listEl.replaceChildren();
    this.selectedIdx = -1;

    if (this.bookmarks.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'bm-empty';
      empty.textContent = '최근에 등록한 [책갈피]가 없습니다.\n사용자가 편집 문서에 책갈피를 삽입하면 [책갈피 목록]에 등록됩니다.';
      this.listEl.appendChild(empty);
      return;
    }

    for (let i = 0; i < this.bookmarks.length; i++) {
      const bm = this.bookmarks[i];
      const row = document.createElement('div');
      row.className = 'bm-item';
      const nameSpan = document.createElement('span');
      nameSpan.className = 'bm-item-name';
      nameSpan.textContent = bm.name || '(이름 없음)';
      const typeSpan = document.createElement('span');
      typeSpan.className = 'bm-item-type';
      typeSpan.textContent = '위치';
      row.appendChild(nameSpan);
      row.appendChild(typeSpan);
      row.addEventListener('click', () => this.selectItem(i));
      row.addEventListener('dblclick', () => { this.selectItem(i); this.doMove(); });
      this.listEl.appendChild(row);
    }
  }

  private selectItem(idx: number): void {
    this.selectedIdx = idx;
    const items = this.listEl.querySelectorAll('.bm-item');
    items.forEach((el, i) => el.classList.toggle('selected', i === idx));
    if (idx >= 0 && idx < this.bookmarks.length) {
      this.nameInput.value = this.bookmarks[idx].name;
    }
    this.statusLabel.textContent = '';
  }

  // ── 이름 제안 ──

  private suggestName(): void {
    const ih = this.services.getInputHandler();
    if (!ih) return;
    // 기존 선택 텍스트나 커서 주변 텍스트를 기본 이름으로 제안
    const count = this.bookmarks.length + 1;
    this.nameInput.value = `책갈피${count}`;
  }

  // ── 동작 ──

  private doAdd(): void {
    const name = this.nameInput.value.trim();
    if (!name) {
      this.statusLabel.textContent = '책갈피 이름을 입력하세요.';
      this.statusLabel.style.color = '#c00';
      return;
    }

    const ih = this.services.getInputHandler();
    if (!ih) return;
    const pos = ih.getCursorPosition();

    const result = this.services.wasm.addBookmark(
      pos.sectionIndex, pos.paragraphIndex, pos.charOffset, name,
    );

    if (result.ok) {
      this.services.eventBus.emit('document-changed');
      this.hide();
    } else {
      this.statusLabel.style.color = '#c00';
      this.statusLabel.textContent = result.error ?? '책갈피 추가 실패';
    }
  }

  private doMove(): void {
    if (this.selectedIdx < 0 || this.selectedIdx >= this.bookmarks.length) return;
    const bm = this.bookmarks[this.selectedIdx];
    const ih = this.services.getInputHandler();
    if (!ih) return;

    const tryMove = (s: number, p: number, o: number) =>
      ih.moveCursorTo({ sectionIndex: s, paragraphIndex: p, charOffset: o });

    if (!tryMove(bm.sec, bm.para, bm.charPos)) {
      // 표 문단 등 커서 이동 불가 → 인접 문단 탐색
      let moved = false;
      for (let d = 1; d <= 5 && !moved; d++) {
        if (tryMove(bm.sec, bm.para + d, 0)) { moved = true; break; }
        if (bm.para - d >= 0 && tryMove(bm.sec, bm.para - d, 0)) { moved = true; break; }
      }
    }
    this.hide();
  }

  private doDelete(): void {
    if (this.selectedIdx < 0 || this.selectedIdx >= this.bookmarks.length) return;
    const bm = this.bookmarks[this.selectedIdx];

    if (!confirm(`선택한 책갈피 '${bm.name}'를 지울까요?`)) return;

    const result = this.services.wasm.deleteBookmark(bm.sec, bm.para, bm.ctrlIdx);
    if (result.ok) {
      this.services.eventBus.emit('document-changed');
      this.refreshList();
      this.statusLabel.textContent = '';
    }
  }

  private doRename(): void {
    if (this.selectedIdx < 0 || this.selectedIdx >= this.bookmarks.length) return;
    const bm = this.bookmarks[this.selectedIdx];
    const newName = prompt('새 책갈피 이름:', bm.name);
    if (!newName || newName.trim() === '' || newName === bm.name) return;

    const result = this.services.wasm.renameBookmark(bm.sec, bm.para, bm.ctrlIdx, newName.trim());
    if (result.ok) {
      this.services.eventBus.emit('document-changed');
      this.refreshList();
    } else {
      this.statusLabel.style.color = '#c00';
      this.statusLabel.textContent = result.error ?? '이름 변경 실패';
    }
  }
}
