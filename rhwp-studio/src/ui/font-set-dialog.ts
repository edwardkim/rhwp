/**
 * 대표 글꼴 등록하기 대화상자
 *
 * 내장 프리셋 + 사용자 정의 대표 글꼴 목록을 표시하고
 * 추가/편집/삭제 기능을 제공한다.
 */
import { ModalDialog } from './dialog';
import { userSettings, BUILTIN_FONT_SETS, LANG_LABELS } from '@/core/user-settings';
import type { FontSet } from '@/core/user-settings';
import { FontSetEditDialog } from './font-set-edit-dialog';

import { t } from '../i18n/index.ts';
export class FontSetDialog extends ModalDialog {
  private listEl!: HTMLDivElement;
  private infoEl!: HTMLDivElement;
  private selectedIndex = -1; // -N = 내장(N-1), 양수 = 사용자
  private editBtn!: HTMLButtonElement;
  private deleteBtn!: HTMLButtonElement;

  constructor() {
    super(t('dialog.fontSet.title'), 520);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'fs-body';

    // 상단: 아이콘 버튼
    const toolbar = document.createElement('div');
    toolbar.className = 'fs-toolbar';

    const addBtn = document.createElement('button');
    addBtn.className = 'dialog-btn fs-icon-btn';
    addBtn.title = t('dialog.fontSet.addBtn.tooltip');
    addBtn.textContent = '+';
    addBtn.addEventListener('click', () => this.onAdd());

    this.editBtn = document.createElement('button');
    this.editBtn.className = 'dialog-btn fs-icon-btn';
    this.editBtn.title = t('dialog.fontSet.editBtn.tooltip');
    this.editBtn.textContent = '✎';
    this.editBtn.disabled = true;
    this.editBtn.addEventListener('click', () => this.onEdit());

    this.deleteBtn = document.createElement('button');
    this.deleteBtn.className = 'dialog-btn fs-icon-btn';
    this.deleteBtn.title = t('dialog.fontSet.deleteBtn.tooltip');
    this.deleteBtn.textContent = '−';
    this.deleteBtn.disabled = true;
    this.deleteBtn.addEventListener('click', () => this.onDelete());

    toolbar.appendChild(addBtn);
    toolbar.appendChild(this.editBtn);
    toolbar.appendChild(this.deleteBtn);
    body.appendChild(toolbar);

    // 중앙: 2열 레이아웃 (목록 + 상세)
    const content = document.createElement('div');
    content.className = 'fs-content';

    this.listEl = document.createElement('div');
    this.listEl.className = 'fs-list';
    content.appendChild(this.listEl);

    this.infoEl = document.createElement('div');
    this.infoEl.className = 'fs-info';
    content.appendChild(this.infoEl);

    body.appendChild(content);

    this.refreshList();
    return body;
  }

  private refreshList(): void {
    this.listEl.replaceChildren();
    const builtins = BUILTIN_FONT_SETS;
    const customs = userSettings.getUserFontSets();

    // 내장
    builtins.forEach((fs, i) => {
      const item = this.createListItem(fs, -(i + 1), true);
      this.listEl.appendChild(item);
    });

    // 사용자 정의
    customs.forEach((fs, i) => {
      const item = this.createListItem(fs, i, false);
      this.listEl.appendChild(item);
    });

    this.selectedIndex = -1;
    this.updateButtons();
    this.showInfo(null);
  }

  private createListItem(fs: FontSet, index: number, isBuiltin: boolean): HTMLDivElement {
    const item = document.createElement('div');
    item.className = 'fs-list-item';
    if (isBuiltin) item.classList.add('fs-builtin');
    item.dataset.index = String(index);

    const nameSpan = document.createElement('span');
    nameSpan.className = 'fs-item-name';
    nameSpan.textContent = fs.name;
    item.appendChild(nameSpan);

    if (isBuiltin) {
      const badge = document.createElement('span');
      badge.className = 'fs-badge';
      badge.textContent = t('dialog.fontSet.badge.text');
      item.appendChild(badge);
    }

    item.addEventListener('click', () => {
      this.listEl.querySelectorAll('.fs-list-item').forEach(el => el.classList.remove('selected'));
      item.classList.add('selected');
      this.selectedIndex = index;
      this.updateButtons();
      this.showInfo(fs);
    });

    return item;
  }

  private updateButtons(): void {
    const isCustom = this.selectedIndex >= 0;
    this.editBtn.disabled = !isCustom;
    this.deleteBtn.disabled = !isCustom;
  }

  private showInfo(fs: FontSet | null): void {
    this.infoEl.replaceChildren();
    if (!fs) {
      const empty = document.createElement('div');
      empty.className = 'fs-info-empty';
      empty.textContent = t('dialog.fontSet.empty.text');
      this.infoEl.appendChild(empty);
      return;
    }

    const name = document.createElement('div');
    name.className = 'fs-info-name';
    name.textContent = fs.name;
    this.infoEl.appendChild(name);

    LANG_LABELS.forEach((label, i) => {
      const val = UserSettingsService_getFontByLang(fs, i);
      const row = document.createElement('div');
      row.className = 'fs-info-row';
      const labelSpan = document.createElement('span');
      labelSpan.className = 'fs-info-label';
      labelSpan.textContent = label;
      const valueSpan = document.createElement('span');
      valueSpan.className = 'fs-info-value';
      valueSpan.textContent = val;
      row.appendChild(labelSpan);
      row.appendChild(valueSpan);
      this.infoEl.appendChild(row);
    });
  }

  private onAdd(): void {
    const dlg = new FontSetEditDialog(null, (newFs) => {
      if (userSettings.addFontSet(newFs)) {
        this.refreshList();
      } else {
        alert(t('dialog.fontSet.onAdd.message'));
      }
    });
    dlg.show();
  }

  private onEdit(): void {
    if (this.selectedIndex < 0) return;
    const customs = userSettings.getUserFontSets();
    const fs = customs[this.selectedIndex];
    if (!fs) return;

    const dlg = new FontSetEditDialog(fs, (updated) => {
      // 이름 변경 시 중복 체크 (자기 자신 제외)
      const allSets = userSettings.getAllFontSets();
      const dup = allSets.some((s, i) => {
        if (i === BUILTIN_FONT_SETS.length + this.selectedIndex) return false;
        return s.name === updated.name;
      });
      if (dup) {
        alert(t('dialog.fontSet.onEdit.message'));
        return;
      }
      userSettings.updateFontSet(this.selectedIndex, updated);
      this.refreshList();
    });
    dlg.show();
  }

  private onDelete(): void {
    if (this.selectedIndex < 0) return;
    const customs = userSettings.getUserFontSets();
    const fs = customs[this.selectedIndex];
    if (!fs) return;

    if (confirm(t('dialog.fontSet.onDelete.message', { p1: fs.name }))) {
      userSettings.removeFontSet(this.selectedIndex);
      this.refreshList();
    }
  }

  protected onConfirm(): void {
    // 설정 버튼 역할: 변경사항은 즉시 저장되므로 추가 동작 없음
  }
}

/** UserSettingsService.getFontByLang 정적 호출 헬퍼 */
function UserSettingsService_getFontByLang(fs: FontSet, langIndex: number): string {
  const keys: (keyof FontSet)[] = ['korean', 'english', 'chinese', 'japanese', 'other', 'symbol', 'user'];
  return fs[keys[langIndex] ?? 'korean'] as string;
}
