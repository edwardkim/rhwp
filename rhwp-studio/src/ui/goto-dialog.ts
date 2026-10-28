import type { CommandServices } from '@/command/types';
import type { BookmarkInfo } from '@/core/types';
import { ModalDialog } from './dialog';

import { t } from '../i18n/index.ts';
/**
 * 찾아가기 대화상자 — 쪽/책갈피 탭으로 이동 위치 선택
 */
export class GotoDialog extends ModalDialog {
  private services: CommandServices;
  private pageInput!: HTMLInputElement;
  private statusLabel!: HTMLSpanElement;
  private activeTab: 'page' | 'bookmark' = 'page';
  private pagePanel!: HTMLElement;
  private bookmarkPanel!: HTMLElement;
  private tabBtnPage!: HTMLButtonElement;
  private tabBtnBookmark!: HTMLButtonElement;
  private bookmarkList!: HTMLElement;
  private selectedBookmark: BookmarkInfo | null = null;
  private pageEnterHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(services: CommandServices, tab?: 'page' | 'bookmark') {
    super(t('dialog.goto.title'), 300);
    this.services = services;
    if (tab) this.activeTab = tab;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'goto-dialog-body';

    // ─── 탭 바 ───
    const tabBar = document.createElement('div');
    tabBar.className = 'goto-tab-bar';

    this.tabBtnPage = document.createElement('button');
    this.tabBtnPage.className = 'goto-tab-btn';
    this.tabBtnPage.textContent = t('dialog.goto.tabBtnPage.text');
    this.tabBtnPage.addEventListener('click', () => this.switchTab('page'));

    this.tabBtnBookmark = document.createElement('button');
    this.tabBtnBookmark.className = 'goto-tab-btn';
    this.tabBtnBookmark.textContent = t('dialog.goto.tabBtnBookmark.text');
    this.tabBtnBookmark.addEventListener('click', () => this.switchTab('bookmark'));

    tabBar.appendChild(this.tabBtnPage);
    tabBar.appendChild(this.tabBtnBookmark);
    body.appendChild(tabBar);

    // ─── 쪽 패널 ───
    this.pagePanel = document.createElement('div');
    this.pagePanel.className = 'goto-panel';

    const totalPages = this.services.wasm.pageCount;
    const row = document.createElement('div');
    row.className = 'dialog-row';
    const label = document.createElement('label');
    label.textContent = t('dialog.goto.label.text');
    label.style.width = '60px';
    this.pageInput = document.createElement('input');
    this.pageInput.type = 'number';
    this.pageInput.min = '1';
    this.pageInput.max = String(totalPages);
    this.pageInput.value = '1';
    this.pageInput.style.width = '80px';
    this.pageInput.style.height = '24px';
    this.pageInput.style.border = '1px solid #aaa';
    this.pageInput.style.padding = '0 4px';
    this.pageInput.style.fontSize = '12px';

    const rangeLabel = document.createElement('span');
    rangeLabel.textContent = t('dialog.goto.rangeLabel.text', { p1: totalPages });
    rangeLabel.style.color = '#666';

    row.appendChild(label);
    row.appendChild(this.pageInput);
    row.appendChild(rangeLabel);
    this.pagePanel.appendChild(row);

    this.statusLabel = document.createElement('div');
    this.statusLabel.style.color = '#c00';
    this.statusLabel.style.fontSize = '11px';
    this.statusLabel.style.marginTop = '4px';
    this.pagePanel.appendChild(this.statusLabel);
    body.appendChild(this.pagePanel);

    // ─── 책갈피 패널 ───
    this.bookmarkPanel = document.createElement('div');
    this.bookmarkPanel.className = 'goto-panel';

    this.bookmarkList = document.createElement('div');
    this.bookmarkList.className = 'goto-bookmark-list';
    this.bookmarkPanel.appendChild(this.bookmarkList);
    body.appendChild(this.bookmarkPanel);

    this.switchTab(this.activeTab);
    return body;
  }

  private switchTab(tab: 'page' | 'bookmark'): void {
    this.activeTab = tab;
    this.tabBtnPage.classList.toggle('active', tab === 'page');
    this.tabBtnBookmark.classList.toggle('active', tab === 'bookmark');
    this.pagePanel.style.display = tab === 'page' ? '' : 'none';
    this.bookmarkPanel.style.display = tab === 'bookmark' ? '' : 'none';
    if (tab === 'bookmark') {
      this.refreshBookmarks();
    }
  }

  private refreshBookmarks(): void {
    this.bookmarkList.replaceChildren();
    this.selectedBookmark = null;
    const bookmarks = this.services.wasm.getBookmarks();
    if (bookmarks.length === 0) {
      const empty = document.createElement('div');
      empty.style.cssText = 'color:#999;font-size:11px;padding:12px;text-align:center';
      empty.textContent = t('dialog.goto.empty.text');
      this.bookmarkList.appendChild(empty);
      return;
    }
    // 이름순 정렬
    bookmarks.sort((a, b) => a.name.localeCompare(b.name));
    for (const bm of bookmarks) {
      const item = document.createElement('div');
      item.className = 'goto-bookmark-item';
      item.textContent = bm.name || t('dialog.goto.item.text');
      item.addEventListener('click', () => {
        this.bookmarkList.querySelectorAll('.goto-bookmark-item').forEach(el => el.classList.remove('selected'));
        item.classList.add('selected');
        this.selectedBookmark = bm;
      });
      item.addEventListener('dblclick', () => {
        this.selectedBookmark = bm;
        this.moveToBookmark();
      });
      this.bookmarkList.appendChild(item);
    }
  }

  private moveToBookmark(): void {
    if (!this.selectedBookmark) return;
    const bm = this.selectedBookmark;
    const ih = this.services.getInputHandler();
    if (!ih) { this.hide(); return; }

    if (!this.tryMoveCursor(ih, bm.sec, bm.para, bm.charPos)) {
      // 표 문단 등 커서 이동 불가 → 인접 문단 탐색
      this.fallbackMove(ih, bm.sec, bm.para);
    }
    this.hide();
  }

  /** moveCursorTo 래퍼 — 성공하면 true */
  private tryMoveCursor(ih: any, sec: number, para: number, offset: number): boolean {
    return ih.moveCursorTo({ sectionIndex: sec, paragraphIndex: para, charOffset: offset });
  }

  /** 지정 문단 전후 ±5 범위에서 커서 이동 가능한 문단을 찾아 결과를 반환한다. */
  private fallbackMove(ih: any, sec: number, para: number): boolean {
    for (let d = 1; d <= 5; d++) {
      if (this.tryMoveCursor(ih, sec, para + d, 0)) return true;
      if (para - d >= 0 && this.tryMoveCursor(ih, sec, para - d, 0)) return true;
    }
    return false;
  }

  show(): void {
    super.show();
    this.installPageEnterHandler();
    if (this.activeTab === 'page') {
      const ih = this.services.getInputHandler();
      if (ih) {
        const pos = ih.getCursorPosition();
        const pageResult = this.services.wasm.getPageOfPosition(pos.sectionIndex, pos.paragraphIndex);
        if (pageResult.ok && pageResult.page != null) {
          this.pageInput.value = String(pageResult.page + 1);
        }
      }
      this.pageInput.focus();
      this.pageInput.select();
    }
  }

  hide(): void {
    this.removePageEnterHandler();
    super.hide();
    // 모달 input이 제거되면 키보드 포커스가 문서 밖으로 빠진다. InputHandler는 active인
    // 동안 전역 단축키를 양보하므로 textarea에 다시 포커스하지 않으면 Option+G를 비롯한
    // 편집 단축키가 다음 입력에서 사라진다.
    this.services.getInputHandler()?.focus();
  }

  private installPageEnterHandler(): void {
    if (this.pageEnterHandler) return;
    this.pageEnterHandler = (e: KeyboardEvent) => {
      if (this.activeTab !== 'page') return;
      if (e.target !== this.pageInput) return;
      if (e.key !== 'Enter' || e.altKey || e.ctrlKey || e.metaKey || e.isComposing) return;

      e.preventDefault();
      e.stopPropagation();
      const shouldClose = this.onConfirm();
      if (shouldClose !== false) this.hide();
    };
    document.addEventListener('keydown', this.pageEnterHandler, true);
  }

  private removePageEnterHandler(): void {
    if (!this.pageEnterHandler) return;
    document.removeEventListener('keydown', this.pageEnterHandler, true);
    this.pageEnterHandler = null;
  }

  protected onConfirm(): void | boolean {
    if (this.activeTab === 'bookmark') {
      this.moveToBookmark();
      return;
    }

    const totalPages = this.services.wasm.pageCount;
    const pageNum = parseInt(this.pageInput.value, 10);

    if (isNaN(pageNum) || pageNum < 1 || pageNum > totalPages) {
      this.statusLabel.textContent = t('dialog.goto.statusLabel.text', { p1: totalPages });
      return false;
    }

    const globalPage = pageNum - 1;
    const posResult = this.services.wasm.getPositionOfPage(globalPage);
    if (!posResult.ok) {
      this.statusLabel.textContent = t('dialog.goto.statusLabel.text.x314a63');
      return false;
    }

    // 표가 쪽의 첫 항목이면 해당 상위 문단에 커서를 둘 수 없을 수 있다. 화면 이동은
    // 커서 배치와 독립적으로 먼저 완료해야 대형 문서의 후반부도 항상 표시된다.
    if (!this.services.gotoPage(globalPage)) {
      this.statusLabel.textContent = t('dialog.goto.statusLabel.text.x71ce6d');
      return false;
    }

    const ih = this.services.getInputHandler();
    if (!ih) {
      this.statusLabel.textContent = t('dialog.goto.statusLabel.text.x614cbb');
      return false;
    }

    // 화면 이동이 성공한 경우에만 각주 편집 컨텍스트를 끝낸다. 이동이 실패하면 사용자는
    // 기존 각주를 계속 편집할 수 있어야 하고, 본문 커서는 아래에서만 배치해야 한다.
    ih.exitFootnoteModeForBodyNavigation();
    const moved = this.tryMoveCursor(
      ih,
      posResult.sec!,
      posResult.para!,
      posResult.charOffset ?? 0,
    ) || this.fallbackMove(ih, posResult.sec!, posResult.para!);
    if (!moved) {
      // 화면은 대상 쪽으로 이동했지만 커서를 놓을 문단을 찾지 못했다. 다음 입력을
      // 허용하려 모달을 유지한다.
      this.statusLabel.textContent = t('dialog.goto.statusLabel.text.x97f664');
      this.pageInput.focus();
      this.pageInput.select();
      return false;
    }
  }
}
